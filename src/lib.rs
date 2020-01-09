use physics::common::random_color;
use physics::dynamics;
use physics::dynamics::orbital;
use physics::dynamics::point::Point3;
use physics::geometry;
use physics::geometry::common::{Metric, Reset};
use physics::geometry::common::coordinates::Homogeneous;
use physics::geometry::matrix::Algebra;
use physics::geometry::trajectory::Trajectory4;
use physics::geometry::vector::{Vector3, Vector4};
use piston::input::{Event, Key, MouseButton, UpdateArgs};
use piston_window;
use piston_window::{Glyphs, PistonWindow};

use crate::common::*;
use crate::core::{Config, Status};
use crate::draw::{Circle, Drawer};
use crate::log::Logger;

pub mod common;
pub mod core;
pub mod draw;
pub mod log;
pub mod keys;

pub struct App {
    pub cluster: dynamics::Cluster,
    pub config: Config,
    pub status: Status,
    pub logger: Logger,
    pub drawer: Drawer,
}

impl From<Config> for App {
    fn from(config: Config) -> Self {
        App::new(dynamics::Cluster::empty(), config)
    }
}

impl App {
    pub fn new(cluster: dynamics::Cluster, config: Config) -> App {
        let size = config.size.clone();
        let scale = config.scale.distance;
        let drawer = Drawer::new(&cluster, &config.orientation, scale, &size);
        App {
            cluster,
            config,
            status: Status::new(),
            logger: Logger::new(),
            drawer,
        }
    }

    pub fn from_orbital(cluster: orbital::Cluster, config: Config) -> App {
        let mut ret = App::new(dynamics::Cluster::orbital_at_random(&cluster), config);
        ret.drawer.set_appearance(&cluster);
        ret
    }

    pub fn on_key(&mut self, key: &Key) {
        self.config.update(key);
        self.logger.update(key);
        self.status.update(&Some(*key), &Option::None);
    }

    pub fn on_click(&mut self, button: &MouseButton) {
        self.status.update(&Option::None, &Some(*button));
    }

    pub fn render(&mut self, cursor: &[f64; 2], window: &mut PistonWindow, event: &Event, glyphs: &mut Glyphs) {
        let scale = self.config.scale.distance;
        self.logger.print(true);
        self.logger.clear();
        window.draw_2d(
            event,
            |c, g, device| {
                piston_window::clear(BLACK, g);
                if self.cluster.is_empty() {
                    self.drawer.draw_barycenter(&self.cluster.barycenter().state.position, &c, g);
                    self.drawer.draw_scale(scale, &self.config.size, &c, g, glyphs);
                    self.drawer.draw_basis(&self.config.size, &c, g);
                    glyphs.factory.encoder.flush(device);
                    return;
                }
                if self.config.trajectory {
                    self.drawer.draw_trajectories(&c, g);
                }

                if self.status.state == core::State::WaitSpeed {
                    self.drawer.draw_speed(cursor, &c, g);
                }
                self.drawer.draw_bodies(&c, g);
                self.drawer.draw_barycenter(&self.cluster.barycenter().state.position, &c, g);
                self.drawer.draw_scale(scale, &self.config.size, &c, g, glyphs);
                self.drawer.draw_basis(&self.config.size, &c, g);
                glyphs.factory.encoder.flush(device);
            },
        );
    }

    pub fn update(&mut self, _window: &mut PistonWindow, args: &UpdateArgs, cursor: &[f64; 2]) {
        use crate::core::State::*;

        if self.status.update_current {
            self.cluster.update_current_index(self.status.increase_current, self.status.is_waiting_to_add());
        }

        if self.status.next_frame {
            self.cluster.update_frame();
        }

        if self.status.next_method {
            self.cluster.solver.method.next();
        }

        match self.status.state {
            Move => self.do_move(args.dt),
            Translate => self.do_translate(),
            Reset => self.do_reset(),
            Add => self.do_add(cursor),
            Remove => self.do_remove(cursor),
            WaitDrop => self.do_wait_drop(cursor),
            WaitSpeed => self.do_wait_speed(cursor),
            CancelDrop => self.do_cancel_drop()
        };

        if self.status.update_transform {
            self.drawer.update_transform(&self.config.orientation, self.config.scale.distance, &self.config.size);
        }

        if self.status.reset_circles {
            self.drawer.reset_circles(&self.cluster);
        }

        self.drawer.update_circles(&self.cluster);

        self.status.clear();
    }

    //noinspection RsTypeCheck
    pub fn log(&mut self, input: &common::Input) {
        self.logger.log(
            &self.cluster,
            &self.drawer,
            &self.status,
            &self.config,
            input,
        );
    }

    fn do_translate(&mut self) {
        if self.cluster.is_empty() {
            return;
        }
        let scale = TRANSLATION_SCALING_FACTOR / self.config.scale.distance;
        let direction = self.config.orientation.rotation().inverse() * (self.status.direction.as_vector() * scale);
        self.cluster.translate_current(&direction);
        self.cluster.update_current_trajectory();
    }

    fn do_move(&mut self, dt: f64) {
        use physics::dynamics::forces;
        if self.config.pause || self.cluster.is_empty() {
            return;
        }
        self.status.step.push(dt, self.config.scale.time);
        // self.cluster.remove_aways();
        let dt = dt / self.config.oversampling as f64 * self.config.scale.time;
        self.cluster.apply(dt, self.config.oversampling, |bodies, i| {
            forces::gravity(&bodies[i].center, bodies)
        });
    }

    fn do_reset(&mut self) {
        if !self.cluster.is_empty() {
            self.cluster.reset0_current();
        }
    }

    //noinspection RsTypeCheck
    fn do_add(&mut self, cursor: &[f64; 2]) {
        let kind = if self.cluster.is_empty() {
            orbital::Kind::Star
        } else {
            orbital::Kind::random()
        };
        let mass = kind.random_mass();
        let radius = kind.scaled_radius(kind.random_radius());
        let name = format!("{:?}", kind);
        let cursor = Vector4::new(cursor[0], cursor[1], 0., 1.);
        let position = self.drawer.inverse_transform * cursor;
        let body_state = geometry::point::Point3::from(Vector3::from_homogeneous(&position));
        let circle_trajectory = Trajectory4::from(cursor);
        self.drawer.circles.push(Circle::new(circle_trajectory, radius, random_color()));
        self.cluster.push(dynamics::Body::new(name.as_str(), Point3::new(body_state, mass)));
    }

    //noinspection RsTypeCheck
    fn do_remove(&mut self, cursor: &[f64; 2]) {
        let len = self.cluster.len();
        let cursor_position = Vector3::new(cursor[0], cursor[1], 0.);
        let mut position;
        for i in 0..len {
            position = Vector3::from_homogeneous(self.drawer.circles[i].trajectory.last());
            if cursor_position.distance(&position) < self.drawer.circles[i].radius {
                self.cluster.remove(i);
                self.drawer.circles.remove(i);
                break;
            }
        }
    }

    fn do_wait_drop(&mut self, cursor: &[f64; 2]) {
        let cursor = Vector4::new(cursor[0], cursor[1], 0., 1.);
        let transformed_cursor = self.drawer.inverse_transform * cursor;
        let last_circle = self.drawer.circles.last_mut().unwrap();
        let mut last_body = self.cluster.last_mut().unwrap();
        last_body.center.state.position = Vector3::from_homogeneous(&transformed_cursor);
        last_body.center.state.trajectory.reset(&last_body.center.state.position);
        last_circle.trajectory.reset(&cursor);
        self.cluster.update_barycenter();
    }

    //noinspection RsTypeCheck
    fn do_wait_speed(&mut self, cursor: &[f64; 2]) {
        let cursor = Vector4::new(cursor[0], cursor[1], 0., 1.);
        let transformed_cursor = self.drawer.inverse_transform * cursor;
        let mut last_body = self.cluster.last_mut().unwrap();
        last_body.center.state.speed = Vector3::from_homogeneous(&transformed_cursor);
        last_body.center.state.speed -= last_body.center.state.position;
        last_body.center.state.speed *= SPEED_SCALING_FACTOR;

        self.cluster.update_barycenter();
    }

    fn do_cancel_drop(&mut self) {
        self.cluster.pop();
        self.drawer.circles.pop();
    }
}
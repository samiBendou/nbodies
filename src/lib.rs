use dynamics::orbital;
use dynamics::orbital::Body;
use dynamics::point::Point3;
use dynamics::solver::{Method, Solver};
use geomath::common::*;
use geomath::common::coordinates::Homogeneous;
use geomath::point;
use geomath::trajectory::Trajectory4;
use geomath::vector::{Vector3, Vector4};
use piston::input::{Event, Key, MouseButton, UpdateArgs};
use piston_window;
use piston_window::{Glyphs, PistonWindow};

use crate::common::*;
use crate::core::{Config, Simulator, Status};
use crate::draw::{Circle, Drawer};
use crate::log::Logger;

pub mod common;
pub mod core;
pub mod draw;
pub mod log;
pub mod keys;

pub struct App {
    pub simulator: Simulator,
    pub config: Config,
    pub status: Status,
    pub logger: Logger,
    pub drawer: Drawer,
}

impl App {
    pub fn new(simulator: Simulator, config: Config) -> App {
        let size = config.size.clone();
        let scale = config.scale.distance;
        let drawer = Drawer::new(&simulator.cluster, &config.orientation, scale, &size);
        let mut ret = App {
            simulator,
            config,
            status: Status::new(),
            logger: Logger::new(),
            drawer,
        };
        ret.drawer.set_appearance(&ret.simulator.system);
        ret
    }

    pub fn from_orbital(system: orbital::Cluster, config: Config) -> App {
        let solver = Solver::new(1., 1, Method::RungeKutta4);
        let simulator = Simulator::orbital_at(system, 0., solver);
        App::new(simulator, config)
    }

    pub fn on_key(&mut self, key: &Key) {
        self.config.update(key);
        self.logger.update(key);
        self.simulator.update(&Some(*key), self.status.is_waiting_to_add());
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
                if self.simulator.cluster.is_empty() {
                    self.drawer.draw_barycenter(&self.simulator.cluster.barycenter().state.position, &c, g);
                    self.drawer.draw_scale(scale, &self.config.size, &c, g, glyphs);
                    self.drawer.draw_basis(&self.config.size, &c, g);
                    glyphs.factory.encoder.flush(device);
                    return;
                }
                if self.config.trajectory {
                    self.drawer.draw_trajectories(&c, g);
                }

                if self.config.orbits {
                    self.drawer.draw_orbits(&self.simulator, &c, g);
                }

                if self.status.state == core::State::WaitSpeed {
                    self.drawer.draw_speed(cursor, &c, g);
                }
                self.drawer.draw_points(&c, g);
                self.drawer.draw_barycenter(&self.simulator.cluster.barycenter().state.position, &c, g);
                self.drawer.draw_scale(scale, &self.config.size, &c, g, glyphs);
                self.drawer.draw_basis(&self.config.size, &c, g);
                glyphs.factory.encoder.flush(device);
            },
        );
    }

    pub fn update(&mut self, _window: &mut PistonWindow, args: &UpdateArgs, cursor: &[f64; 2]) {
        use crate::core::State::*;

        if let Some(index) = self.simulator.remove_aways() {
            self.drawer.circles.remove(index);
        }

        match self.status.state {
            Move => self.do_move(args.dt),
            Translate => self.do_translate(),
            Reset => self.do_reset(),
            Add => self.do_add(),
            Remove => self.do_remove(cursor),
            WaitDrop => self.do_wait_drop(cursor),
            WaitSpeed => self.do_wait_speed(cursor),
            CancelDrop => self.do_cancel_drop()
        };

        if self.status.update_transform {
            self.drawer.update_transform(&self.config.orientation, self.config.scale.distance, &self.config.size);
        }

        if self.status.reset_circles {
            self.drawer.reset_circles(&self.simulator.cluster);
        }

        self.drawer.update_circles(&self.simulator.cluster);

        self.status.clear();
    }

    //noinspection RsTypeCheck
    pub fn log(&mut self, input: &common::Input) {
        self.logger.log(
            &self.simulator,
            &self.drawer,
            &self.status,
            &self.config,
            input,
        );
    }

    fn do_translate(&mut self) {
        if self.simulator.cluster.is_empty() {
            return;
        }
        let scale = TRANSLATION_SCALING_FACTOR / self.config.scale.distance;
        let direction = self.config.orientation.inverse_rotation() * (self.status.direction.as_vector() * scale);
        self.simulator.cluster.translate_at(self.simulator.current, &direction);
    }

    fn do_move(&mut self, dt: f64) {
        use dynamics::forces;
        if self.config.pause || self.simulator.cluster.is_empty() {
            return;
        }
        self.status.step.push(dt, self.config.scale.time);
        let dt = dt / self.config.oversampling as f64 * self.config.scale.time;
        self.simulator.apply(dt, self.config.oversampling, |points, i| {
            forces::gravity(&points[i], points)
        });
    }

    fn do_reset(&mut self) {
        if !self.simulator.cluster.is_empty() {
            self.simulator.cluster.reset0_at(self.simulator.current);
        }
    }

    //noinspection RsTypeCheck
    fn do_add(&mut self) {
        let body = Body::random();
        self.drawer.circles.push(
            Circle::new(Trajectory4::zeros(), body.kind.scaled_radius(body.radius), body.color)
        );
        self.simulator.push(Point3::new(point::Point3::zeros(), body.mass), body);
    }

    //noinspection RsTypeCheck
    fn do_remove(&mut self, cursor: &[f64; 2]) {
        let cursor = Vector3::new(cursor[0], cursor[1], 0.);
        let mut position;
        for i in 0..self.simulator.cluster.len() {
            position = Vector3::from_homogeneous(self.drawer.circles[i].trajectory.last());
            if cursor.distance(&position) < self.drawer.circles[i].radius {
                self.simulator.cluster.remove(i);
                self.drawer.circles.remove(i);
                break;
            }
        }
    }

    fn do_wait_drop(&mut self, cursor: &[f64; 2]) {
        let cursor = Vector4::new(cursor[0], cursor[1], 0., 1.);
        let transformed_cursor = self.drawer.inverse_transform * cursor;
        let circle = self.drawer.circles.last_mut().unwrap();
        let mut point = self.simulator.last_mut().unwrap();
        circle.trajectory.reset(&cursor);
        point.state.position = Vector3::from_homogeneous(&transformed_cursor);
        point.state.trajectory.reset(&point.state.position);
        self.simulator.cluster.update_barycenter();
    }

    //noinspection RsTypeCheck
    fn do_wait_speed(&mut self, cursor: &[f64; 2]) {
        let cursor = self.drawer.inverse_transform * Vector4::new(cursor[0], cursor[1], 0., 1.);
        let mut point = self.simulator.last_mut().unwrap();
        point.state.speed = Vector3::from_homogeneous(&cursor);
        point.state.speed -= point.state.position;
        point.state.speed *= SPEED_SCALING_FACTOR;
    }

    fn do_cancel_drop(&mut self) {
        self.simulator.cluster.pop();
        self.drawer.circles.pop();
    }
}
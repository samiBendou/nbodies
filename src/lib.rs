use std::error::Error;
use std::path::Path;

use piston::input::{Event, Key, MouseButton, UpdateArgs};
use piston_window;
use piston_window::{Glyphs, PistonWindow};

use crate::common::*;
use crate::core::{Arguments, Config, Status, Step};
use crate::log::Logger;
use crate::physics::dynamics;
use crate::physics::dynamics::orbital;
use crate::physics::vector::transforms::Cartesian2;
use crate::physics::vector::Vector2;
use crate::shapes::{BLACK, Drawer};

pub mod common;
pub mod shapes;
pub mod physics;
pub mod core;
pub mod log;

pub struct App {
    pub cluster: dynamics::Cluster,
    pub config: Config,
    pub status: Status,
    pub step: Step,
    pub logger: Logger,
    pub drawer: Drawer,
}

impl From<dynamics::Cluster> for App {
    fn from(cluster: dynamics::Cluster) -> Self {
        let mut config = Config::default();
        config.scale.distance = 0.5 * config.size.width / cluster.max_distance().0;
        App::new(cluster, config)
    }
}

impl From<Config> for App {
    fn from(config: Config) -> Self {
        App::new(dynamics::Cluster::empty(), config)
    }
}

impl App {
    pub fn new(cluster: dynamics::Cluster, config: Config) -> App {
        let size = config.size.clone();
        App {
            cluster,
            config,
            status: Status::default(),
            step: Step::new(),
            logger: Logger::new(),
            drawer: Drawer::new(&size),
        }
    }

    pub fn default() -> App {
        App::new(dynamics::Cluster::empty(), Config::default())
    }

    pub fn from_args(args: Arguments) -> Result<App, Box<dyn Error>> {
        if let Some(path) = args.path {
            let cluster = orbital::Cluster::from_file(Path::new(path.as_str()))?;

            return Ok(App::from(dynamics::Cluster::from_orbits_at(cluster, 0.)));
        }
        Ok(App::from(Config::from(args)))
    }

    pub fn on_key(&mut self, key: &Key) {
        self.config.update(key);
        self.logger.update(key);
        self.status.update(&Some(*key), &Option::None);
        self.update_cluster(key);
    }

    pub fn on_click(&mut self, button: &MouseButton) {
        self.status.update(&Option::None, &Some(*button));
    }

    pub fn render(&mut self, window: &mut PistonWindow, event: &Event, glyphs: &mut Glyphs) {
        let scale = self.config.scale.distance;
        self.logger.print(true);
        self.logger.clear();
        window.draw_2d(
            event,
            |c, g, device| {
                piston_window::clear(BLACK, g);
                if self.cluster.count() == 0 {
                    self.drawer.draw_barycenter(self.cluster.barycenter(), scale, &c, g);
                    self.drawer.draw_scale(scale, &c, g, glyphs);
                    glyphs.factory.encoder.flush(device);
                    return;
                }
                if self.status.trajectory {
                    self.drawer.draw_trajectories(&self.cluster, scale, &c, g);
                }
                if self.status.state == core::State::WaitSpeed {
                    self.drawer.draw_speed(self.cluster.last().unwrap(), self.config.scale.distance, &c, g);
                }
                self.drawer.draw_bodies(&self.cluster, scale, &c, g);
                self.drawer.draw_barycenter(self.cluster.barycenter(), scale, &c, g);
                self.drawer.draw_scale(scale, &c, g, glyphs);
                glyphs.factory.encoder.flush(device);
            },
        );
    }

    pub fn update(&mut self, _window: &mut PistonWindow, args: &UpdateArgs, cursor: &[f64; 2]) {
        use crate::core::State::*;
        match self.status.state {
            Move => self.do_move(args.dt),
            Reset => self.do_reset(),
            Add => self.do_add(cursor),
            Remove => self.do_remove(cursor),
            WaitDrop => self.do_wait_drop(cursor),
            WaitSpeed => self.do_wait_speed(cursor),
            CancelDrop => self.do_cancel_drop()
        };
        self.status.update(&Option::None, &Option::None);
        if self.status.pause {
            return;
        }
    }

    pub fn update_cluster(&mut self, key: &Key) {
        if *key == KEY_INCREASE_CURRENT_INDEX || *key == KEY_DECREASE_CURRENT_INDEX {
            let increase = *key == KEY_INCREASE_CURRENT_INDEX;
            let mut bypass_last = false;
            if self.status.is_waiting_to_add() {
                bypass_last = true;
            }
            self.cluster.update_current_index(increase, bypass_last);
        } else if *key == KEY_NEXT_FRAME_STATE {
            self.cluster.update_frame();
        }
    }

    pub fn log(&mut self, input: &common::Input) {
        self.logger.log(&self.cluster, &self.status, &self.config, &self.step, input);
    }

    fn do_move(&mut self, dt: f64) {
        let scaled_middle = *self.drawer.middle() / self.config.scale.distance;

        if self.status.pause || self.cluster.is_empty() {
            return;
        }
        self.step.update(dt, self.config.scale.time);
        if self.status.translate {
            self.cluster.translate_current(&self.status.direction.as_vector());
            if self.status.bounded {
                self.cluster.bound_current(&scaled_middle);
            }
            self.cluster.update_current_trajectory();
            return;
        }
        self.cluster.remove_aways();
        self.do_accelerate(dt / self.config.oversampling as f64 * self.config.scale.time);
        if self.status.bounded {
            self.cluster.bound(&scaled_middle);
        }
        self.cluster.update_trajectory();
    }

    fn do_accelerate(&mut self, dt: f64) {
        use physics::dynamics::forces;
        self.cluster.apply(dt, self.config.oversampling, |bodies, i| {
            /*
            let position = bodies[i].shape.center.position.clone();
            let k1 = forces::gravity(&bodies[i].shape.center, bodies);
            bodies[i].shape.center.position = bodies[i].shape.center.speed * 0.5 * dt * dt + position;
            let k2 = forces::gravity(&bodies[i].shape.center, bodies);
            bodies[i].shape.center.position = k2 * 0.5 * dt * dt + position;
            let k3 = forces::gravity(&bodies[i].shape.center, bodies);
            bodies[i].shape.center.position = k3 * dt * dt + position;
            let k4 = forces::gravity(&bodies[i].shape.center, bodies);
            bodies[i].shape.center.position = position;
            *force = (k1 + (k2 + k3) * 2. + k4) / 6.;
            */
            forces::gravity(&bodies[i].shape.center, bodies)
        });
    }

    fn do_reset(&mut self) {
        if !self.cluster.is_empty() {
            self.cluster.reset_current();
        }
    }

    fn do_add(&mut self, cursor: &[f64; 2]) {
        use shapes::ellipse;
        let kind = if self.cluster.is_empty() {
            orbital::Kind::Star
        } else {
            orbital::Kind::random()
        };
        let mass = kind.random_mass();
        let color = random_color();
        let scale = self.config.scale.distance;
        let radius = kind.scaled_radius(kind.random_radius());
        let circle = ellipse::Circle::at_cursor(cursor, radius, color, self.drawer.middle(), scale);
        let mut body = dynamics::Body::new(mass, "", circle);
        body.shape.center.scale_position(self.config.scale.distance);
        self.cluster.push(body);
        self.cluster.last_mut().unwrap().name = format!("body {}", self.cluster.count());
    }

    fn do_remove(&mut self, cursor: &[f64; 2]) {
        let scale = self.config.scale.distance;
        let len = self.cluster.count();
        let cursor_position = Vector2::from(*cursor);
        let mut position;
        for i in 0..len {
            position = self.cluster[i].shape.center.position.left_up(self.drawer.middle(), scale);
            if cursor_position % position < self.cluster[i].shape.radius {
                self.cluster.remove(i);
                break;
            }
        }
    }

    fn do_wait_drop(&mut self, cursor: &[f64; 2]) {
        self.cluster.wait_drop(cursor, self.drawer.middle(), self.config.scale.distance);
    }

    fn do_wait_speed(&mut self, cursor: &[f64; 2]) {
        self.cluster.wait_speed(cursor, self.drawer.middle(), self.config.scale.distance);
    }

    fn do_cancel_drop(&mut self) {
        self.cluster.pop();
    }
}
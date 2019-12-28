use std::io;
use std::path::Path;

use piston::input::{Event, Key, MouseButton, UpdateArgs};
use piston_window;
use piston_window::{Glyphs, PistonWindow};

use crate::common::*;
use crate::core::{Config, Status, Step};
use crate::log::Logger;
use crate::physics::dynamics;
use crate::physics::dynamics::orbital;
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
        config.scale.distance = 1. / cluster.max_distance() * config.size.width;
        App::new(cluster, config)
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

    pub fn from_args(args: Vec<String>) -> Result<App, io::Error> {
        if args.len() == 1 {
            Ok(App::default())
        } else {
            let cluster = orbital::Cluster::from_file(Path::new(args[1].as_str()))?;
            Ok(App::from(dynamics::Cluster::from_orbits_at(cluster, 0.)))
        }
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
            WaitDrop => self.do_wait_drop(cursor),
            WaitSpeed => self.do_wait_speed(cursor),
            CancelDrop => self.do_cancel_drop()
        };
        self.status.update(&Option::None, &Option::None);
        if self.status.pause {
            return;
        }
        self.cluster.update_trajectory();
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

        self.do_accelerate(dt / self.config.oversampling as f64 * self.config.scale.time);

        if self.status.bounded {
            self.cluster.bound(&scaled_middle);
        }
    }

    fn do_accelerate(&mut self, dt: f64) {
        use physics::dynamics::forces;
        self.cluster.apply(dt, self.config.oversampling, |force, bodies, i| {
            *force = forces::gravity(&bodies[i], bodies);
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
        let radius = kind.scaled_radius(kind.random_radius());
        let circle = ellipse::Circle::at_cursor(cursor, radius, color, self.drawer.middle());
        let mut body = dynamics::Body::new(mass, "", circle);
        body.shape.center.scale_position(self.config.scale.distance);
        self.cluster.push(body);
        self.cluster.last_mut().unwrap().name = format!("body {}", self.cluster.count());
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
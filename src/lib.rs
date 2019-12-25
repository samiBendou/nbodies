use piston::input::{Event, Key, MouseButton, UpdateArgs};
use piston_window;
use piston_window::{Glyphs, PistonWindow};

use crate::common::*;
use crate::core::{Config, Status, Step};
use crate::log::Logger;
use crate::physics::dynamics::body::{Body, Cluster};
use crate::shapes::Drawer;

pub mod common;
pub mod shapes;
pub mod physics;
pub mod core;
pub mod log;

pub struct App {
    pub bodies: Cluster,
    pub config: Config,
    pub status: Status,
    pub step: Step,
    pub logger: Logger,
    pub drawer: Drawer,
}

impl App {
    pub fn new(bodies: Cluster, status: Status, config: Config) -> App {
        let config = config;
        App {
            bodies,
            config,
            status,
            step: Step::new(),
            logger: Logger::new(),
            drawer: Drawer::new(&config.size),
        }
    }

    pub fn default() -> App {
        let config = Config::default();
        App {
            bodies: Cluster::empty(),
            config,
            status: Status::default(),
            step: Step::new(),
            logger: Logger::new(),
            drawer: Drawer::new(&config.size),
        }
    }

    pub fn on_key(&mut self, key: &Key) {
        self.config.update(key);
        self.status.update(&Some(*key), &Option::None);
        if *key == KEY_INCREASE_CURRENT_INDEX || *key == KEY_DECREASE_CURRENT_INDEX {
            let increase = *key == KEY_INCREASE_CURRENT_INDEX;
            self.bodies.update_current_index(increase);
        } else if *key == KEY_NEXT_FRAME_STATE {
            self.bodies.update_frame();
        } else if *key == KEY_NEXT_LOGGER_STATE {
            self.logger.update();
        }
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
                piston_window::clear([1.0; 4], g);
                if self.bodies.count() == 0 {
                    self.drawer.draw_barycenter(self.bodies.barycenter(), scale, &c, g);
                    self.drawer.draw_scale(scale, &c, g, glyphs);
                    return;
                }
                if self.status.trajectory {
                    self.drawer.draw_trajectories(&self.bodies, scale, &c, g);
                }
                if self.status.state == core::State::WaitSpeed {
                    self.drawer.draw_speed(self.bodies.current(), self.config.scale.distance, &c, g);
                }
                self.drawer.draw_bodies(&self.bodies, scale, &c, g);
                self.drawer.draw_barycenter(self.bodies.barycenter(), scale, &c, g);
                self.drawer.draw_scale(scale, &c, g, glyphs);
                glyphs.factory.encoder.flush(device);
            }
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
        self.bodies.update_trajectory();
    }

    pub fn log(&mut self, input: &common::Input) {
        self.logger.log(&self.bodies, &self.status, &self.config, &self.step, input);
    }

    fn do_move(&mut self, dt: f64) {
        let scaled_middle = *self.drawer.middle() / self.config.scale.distance;

        if self.status.pause || self.bodies.is_empty() {
            return;
        }
        self.step.update(dt, self.config.scale.time);
        if self.status.translate {
            self.bodies.translate_current(&self.status.direction.as_vector());
            if self.status.bounded {
                self.bodies.bound_current(&scaled_middle);
            }
            self.bodies.update_current_trajectory();
            return;
        }
        self.do_accelerate(dt / self.config.updates_per_frame as f64 * self.config.scale.time);

        if self.status.bounded {
            self.bodies.bound(&scaled_middle);

        }
    }

    fn do_accelerate(&mut self, dt: f64) {
        use crate::physics::vector::Vector2;
        use physics::dynamics::forces;
        let current_index = self.bodies.current_index();
        let current_direction = self.status.direction;

        let mut direction: Direction = Direction::Hold;
        self.bodies.apply(dt, self.config.updates_per_frame, |mut force, bodies, i| {
            direction = if i == current_index {
                current_direction
            } else {
                Direction::Hold
            };
            *force = forces::gravity(&bodies[i], bodies); // forces::push(&direction);
        });
    }

    fn do_reset(&mut self) {
        if !self.bodies.is_empty() {
            self.bodies.reset_current();
        }
    }

    fn do_add(&mut self, cursor: &[f64; 2]) {
        use shapes::ellipse;
        let circle = ellipse::Circle::at_cursor_random(cursor, self.drawer.middle());
        let mut body = Body::new(circle.radius / 10., "", circle);
        body.shape.center.scale_position(self.config.scale.distance);
        self.bodies.push(body);
        self.bodies.current_mut().name = format!("body {}", self.bodies.current_index() + 1);
    }

    fn do_wait_drop(&mut self, cursor: &[f64; 2]) {
        self.bodies.wait_drop(cursor, self.drawer.middle(), self.config.scale.distance);
    }

    fn do_wait_speed(&mut self, cursor: &[f64; 2]) {
        self.bodies.wait_speed(cursor, self.drawer.middle(), self.config.scale.distance);
    }

    fn do_cancel_drop(&mut self) {
        self.bodies.pop();
    }
}
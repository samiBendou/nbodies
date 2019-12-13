use piston::input::{Event, Key, MouseButton, UpdateArgs};
use piston::window::Size;
use piston_window;
use piston_window::{Glyphs, PistonWindow};

use crate::common::*;
use crate::core::{Config, Status, Step};
use crate::log::Logger;
use crate::physics::dynamics::{Body, VecBody};
use crate::shapes::Drawer;

pub mod common;
pub mod shapes;
pub mod physics;
pub mod core;
pub mod log;

pub struct App {
    pub bodies: VecBody,
    pub config: Config,
    pub status: Status,
    pub step: Step,
    pub logger: Logger,
    pub drawer: Drawer,
}

impl App {
    pub fn new(bodies: VecBody, status: Status, config: Config) -> App {
        let config = config;
        App {
            bodies,
            config,
            status,
            step: Step::new(),
            logger: Logger::new(),
            drawer: Drawer::new(&config.size)
        }
    }

    pub fn default() -> App {
        let config = Config::default();
        App {
            bodies: VecBody::empty(),
            config,
            status: Status::default(),
            step: Step::new(),
            logger: Logger::new(),
            drawer: Drawer::new(&config.size),
        }
    }

    pub fn on_key(&mut self, key: &Key) {
        let option_key = Some(*key);
        self.config.update(key);
        self.status.update(&option_key, &Option::None);
        if *key == Key::Z {
            self.bodies.update_current_index(true);
        } else if *key == Key::X {
            self.bodies.update_current_index(false);
        }
        self.logger.update(&option_key);
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
                self.drawer.draw_bodies(&self.bodies, scale, &c, g);
                self.drawer.draw_barycenter(self.bodies.barycenter(), scale, &c, g);
                self.drawer.draw_scale(scale, &c, g, glyphs);
                glyphs.factory.encoder.flush(device);
            }
        );
    }

    pub fn update(&mut self, _window: &mut PistonWindow, args: &UpdateArgs, cursor: &[f64; 2]) {
        use crate::core::State::*;

        self.step.update(args.dt);
        match self.status.state {
            Move => self.do_move(self.step.frame / self.config.updates_per_frame as f64),
            Reset => self.do_reset(),
            Add => self.do_add(cursor),
            WaitDrop => self.do_wait_drop(cursor),
            CancelDrop => self.do_cancel_drop()
        };
        self.status.update(&Option::None, &Option::None);
    }

    pub fn log(&mut self, input: &common::Input) {
        self.logger.log(&self.bodies, &self.status, &self.config, &self.step, input);
    }

    fn do_move(&mut self, dt: f64) {
        use crate::physics::units::PX_PER_METER;
        let width = self.config.size.width / (self.config.scale.distance * PX_PER_METER);
        let height = self.config.size.height / (self.config.scale.distance * PX_PER_METER);
        let scaled_size = Size::from([width, height]);
        if self.status.pause || self.bodies.is_empty() {
            return;
        }
        if self.status.translate {
            self.bodies.translate_current(&self.status.direction.as_vector());
            if self.status.bounded {
                self.bodies.bound_current(&scaled_size);
            }
            self.bodies.update_current_trajectory(&Some(self.config.size));
            return;
        }
        self.do_accelerate(dt);

        if self.status.bounded {
            self.bodies.bound(&scaled_size);
        }
        self.bodies.update_trajectory(&Some(self.config.size));
        self.bodies.update_barycenter();
    }

    fn do_accelerate(&mut self, dt: f64) {
        use crate::physics::vector::Vector2;
        use physics::forces;
        let count = self.bodies.count();
        let mut directions = vec![Direction::Hold; count];
        let mut force: Vector2;

        directions[self.bodies.current_index()] = self.status.direction;
        for _ in 0..self.config.updates_per_frame {
            for i in 0..count {
                force = forces::push(&directions[i]);
                // force += forces::nav_stokes(&self.bodies[i].shape.center.speed);
                self.bodies[i].shape.center.acceleration = force * (self.bodies[i].mass);
            }
            self.bodies.accelerate(dt);
        }
    }

    fn do_reset(&mut self) {
        if !self.bodies.is_empty() {
            self.bodies.reset_current();
            self.bodies.clear_current_trajectory(&Some(self.config.size));
            self.bodies.update_barycenter();
        }
    }

    fn do_add(&mut self, cursor: &[f64; 2]) {
        use shapes::ellipse;
        let mut circle = ellipse::Circle::at_cursor_random(cursor, &self.config.size);
        circle.center.position *= self.config.scale.distance;
        let body = Body::new(circle.radius / 10., format!(""), circle);
        self.bodies.push(body);
        self.bodies.current_mut().name = format!("body {}", self.bodies.current_index() + 1);
    }

    fn do_wait_drop(&mut self, cursor: &[f64; 2]) {
        self.bodies.wait_drop(cursor, &self.config.size, self.config.scale.distance);
        self.bodies.clear_current_trajectory(&Some(self.config.size));
    }

    fn do_cancel_drop(&mut self) {
        self.bodies.pop();
    }

    fn has_to_render(&self) -> bool {
        self.step.count > self.config.frames_per_update
    }
}
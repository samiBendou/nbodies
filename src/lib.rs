use piston::input::{Event, Key, MouseButton, UpdateArgs};
use piston_window;
use piston_window::{Glyphs, PistonWindow};

use crate::common::*;
use crate::core::{Config, Status, Step};
use crate::log::Logger;
use crate::physics::dynamics::{Body, Cluster};
use crate::shapes::{BLACK, Drawer};

pub mod common;
pub mod shapes;
pub mod physics;
pub mod core;
pub mod log;

pub struct App {
    pub cluster: Cluster,
    pub config: Config,
    pub status: Status,
    pub step: Step,
    pub logger: Logger,
    pub drawer: Drawer,
}

impl App {
    pub fn new(cluster: Cluster, config: Config) -> App {
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
        App::new(Cluster::empty(), Config::default())
    }

    pub fn cluster(cluster: Cluster) -> App {
        let mut config = Config::default();
        let mut max_distance = 0.;

        let mut distance: f64;
        for body in cluster.bodies.iter() {
            distance = body.shape.center.position.magnitude();
            if distance > max_distance {
                max_distance = distance;
            }
        }
        println!("there {:?}", cluster);
        config.scale.distance = 1. / max_distance * config.size.width;
        App::new(cluster, config)
    }

    pub fn on_key(&mut self, key: &Key) {
        self.config.update(key);
        self.status.update(&Some(*key), &Option::None);
        if *key == KEY_INCREASE_CURRENT_INDEX || *key == KEY_DECREASE_CURRENT_INDEX {
            let increase = *key == KEY_INCREASE_CURRENT_INDEX;
            self.cluster.update_current_index(increase);
        } else if *key == KEY_NEXT_FRAME_STATE {
            self.cluster.update_frame();
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
                    self.drawer.draw_speed(self.cluster.current(), self.config.scale.distance, &c, g);
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
        self.do_accelerate(dt / self.config.updates_per_frame as f64 * self.config.scale.time);

        if self.status.bounded {
            self.cluster.bound(&scaled_middle);
        }
    }

    fn do_accelerate(&mut self, dt: f64) {
        use physics::dynamics::forces;
        let current_index = self.cluster.current_index();
        let current_direction = self.status.direction;

        let mut direction: Direction = Direction::Hold;
        self.cluster.apply(dt, self.config.updates_per_frame, |force, bodies, i| {
            direction = if i == current_index {
                current_direction
            } else {
                Direction::Hold
            };
            *force = forces::gravity(&bodies[i], bodies);
            *force += forces::push(&direction);
        });
    }

    fn do_reset(&mut self) {
        if !self.cluster.is_empty() {
            self.cluster.reset_current();
        }
    }

    fn do_add(&mut self, cursor: &[f64; 2]) {
        use shapes::ellipse;
        let circle = ellipse::Circle::at_cursor_random(cursor, self.drawer.middle());
        let mut body = Body::new(circle.radius * 10e24, "", circle);
        body.shape.center.scale_position(self.config.scale.distance);
        self.cluster.push(body);
        self.cluster.current_mut().name = format!("body {}", self.cluster.current_index() + 1);
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
use piston::input::{Event, Key, MouseButton, UpdateArgs};
use piston::window::Size;
use piston_window;
use piston_window::{G2d, Glyphs, PistonWindow};
use piston_window::*;

use crate::common::*;
use crate::core::{Config, Status, Step};
use crate::log::Logger;
use crate::physics::dynamics::{Body, VecBody};
use crate::shape::*;

pub mod common;
pub mod shape;
pub mod physics;
pub mod core;
pub mod log;

pub struct App {
    pub bodies: VecBody,
    pub config: Config,
    pub status: Status,
    pub step: Step,
    pub logger: Logger,
}

impl App {
    pub fn new(bodies: VecBody, status: Status, config: Config) -> App {
        App {
            bodies,
            config,
            status,
            step: Step::new(),
            logger: Logger::new(),
        }
    }

    pub fn default() -> App {
        App {
            bodies: VecBody::empty(),
            config: Config::default(),
            status: Status::default(),
            step: Step::new(),
            logger: Logger::new(),
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
        self.logger.print(true);
        self.logger.clear();
        window.draw_2d(
            event,
            |c, g, device| {
                piston_window::clear([1.0; 4], g);
                if self.bodies.count() == 0 {
                    self.draw_static(&c, g, glyphs);
                    return;
                }
                if self.status.trajectory {
                    self.draw_trajectory(&c, g);
                }
                self.draw_bodies(&c, g);
                self.draw_static(&c, g, glyphs);
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
        let mut circle = Circle::at_cursor_random(cursor, &self.config.size);
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

    fn draw_trajectory(&self, c: &Context, g: &mut G2d) {
        use crate::physics::vector::Vector2;
        use crate::physics::dynamics::TRAJECTORY_SIZE;
        let middle: Vector2 = Vector2::new(self.config.size.width, self.config.size.height) / 2.;
        let scale = self.config.scale.distance;

        let mut from: Vector2;
        let mut to: Vector2;
        let mut color: [f32; 4];
        for i in 0..self.bodies.count() {
            color = self.bodies[i].shape.color;
            for k in 1..TRAJECTORY_SIZE - 1 {
                color[3] = k as f32 / (TRAJECTORY_SIZE as f32 - 1.);
                from = (*self.bodies[i].shape.center.position(k - 1) - middle) * scale;
                to = (*self.bodies[i].shape.center.position(k) - middle) * scale;
                from += middle;
                to += middle;
                piston_window::line_from_to(
                    color,
                    2.5,
                    from.as_array(),
                    to.as_array(),
                    c.transform, g,
                );
            }
        }
    }

    fn draw_bodies(&self, c: &piston_window::Context, g: &mut G2d) {
        let mut rect: [f64; 4];
        let scale = self.config.scale.distance;
        for i in 0..self.bodies.count() {
            rect = self.bodies[i].shape.rounding_rect(&self.config.size, scale);
            piston_window::ellipse(
                self.bodies[i].shape.color,
                rect,
                c.transform, g,
            );
        }
    }

    fn draw_static(&self, c: &Context, g: &mut G2d, glyphs: &mut Glyphs) {
        use crate::physics::units::PX_PER_METER;
        let size = self.config.size;
        let scale = self.config.scale.distance;
        let x_offset = size.width - 160.;
        let y_offset = size.height - 48.;
        let mut barycenter_rect = *self.bodies.barycenter() * scale;
        to_left_up!(barycenter_rect, size);

        piston_window::rectangle(
            [255., 0., 0., 1.],
            [barycenter_rect[0] - 4., barycenter_rect[1] - 4., 8., 8.],
            c.transform, g,
        );
        piston_window::line_from_to(
            [0., 0., 0., 1.],
            3.,
            [x_offset, y_offset],
            [x_offset + PX_PER_METER, y_offset],
            c.transform, g,
        );
        piston_window::text::Text
        ::new_color([0.0, 0.0, 0.0, 1.0], 16).draw(
            format!("scale: {:.2e} (m/px)", PX_PER_METER * scale).as_str(),
            glyphs,
            &c.draw_state,
            c.transform.trans(x_offset, y_offset - 16.),
            g,
        ).unwrap();
    }

    fn has_to_render(&self) -> bool {
        self.step.count > self.config.frames_per_update
    }
}
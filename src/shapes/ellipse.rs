use std::fmt::{Debug, Error, Formatter};

use rand::Rng;

use crate::physics::dynamics::point::Point2;
use crate::physics::vector::Vector2;

#[derive(Copy, Clone)]
pub struct Circle {
    pub center: Point2,
    pub color: [f32; 4],
    pub radius: f64,
}

impl Circle {
    pub fn new(center: Point2, radius: f64, color: [f32; 4]) -> Circle {
        Circle {
            center,
            color,
            radius,
        }
    }

    pub fn centered(radius: f64, color: [f32; 4]) -> Circle {
        Circle::new(Point2::zeros(), radius, color)
    }

    pub fn at_cursor(cursor: &[f64; 2], radius: f64, color: [f32; 4], middle: &Vector2) -> Circle {
        let position = Vector2::from(*cursor);
        let center = Point2::stationary(position);
        let mut circle = Circle::new(center, radius, color);
        Circle::set_centered(&mut circle.center.position, middle);
        circle
    }

    pub fn at_cursor_random(cursor: &[f64; 2], middle: &Vector2) -> Circle {
        let mut rng = rand::thread_rng();
        let radius: f64 = rng.gen();
        let color: [f32; 4] = [rng.gen(), rng.gen(), rng.gen(), 1.];
        Circle::at_cursor(cursor, 20. * radius + 20., color, middle)
    }

    pub fn rounding_rect(&self, middle: &Vector2, scale: f64) -> [f64; 4] {
        let diameter = 2. * self.radius;
        let mut position_scaled = self.center.position * scale;
        Circle::set_left_up(&mut position_scaled, middle);
        [position_scaled.x - self.radius, position_scaled.y - self.radius, diameter, diameter]
    }

    pub fn bound(&mut self, middle: &Vector2) -> &mut Circle {
        let x_left = -self.radius - middle.x;
        let x_right = self.radius + middle.x;
        let y_up = self.radius + middle.y;
        let y_down = -self.radius - middle.y;

        if self.center.position.x < x_left {
            self.center.position.x = x_right;
        } else if self.center.position.x > x_right {
            self.center.position.x = x_left;
        }

        if self.center.position.y < y_down {
            self.center.position.y = y_up;
        } else if self.center.position.y > y_up {
            self.center.position.y = y_down;
        }

        self
    }

    pub fn set_cursor_pos(&mut self, cursor: &[f64; 2], middle: &Vector2) -> &mut Circle {
        self.center.position.set_array(&cursor);
        Circle::set_centered(&mut self.center.position, middle);
        self
    }

    pub fn set_cursor_speed(&mut self, cursor: &[f64; 2], middle: &Vector2, scale: f64) -> &mut Circle {
        let mut speed = Vector2::from(*cursor);
        Circle::set_centered(&mut speed, middle);
        speed -= self.center.position * scale;
        self.center.speed = speed;
        self
    }

    pub fn set_centered(position: &mut Vector2, middle: &Vector2) {
        position.x = position.x - middle.x;
        position.y = middle.y - position.y;
    }

    pub fn set_left_up(position: &mut Vector2, middle: &Vector2) {
        position.x = position.x + middle.x;
        position.y = middle.y - position.y;
    }
}

impl Debug for Circle {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:?}", self.center)
    }
}
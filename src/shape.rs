use std::fmt::{Debug, Error, Formatter};

use piston::window::Size;
use rand::Rng;

use crate::physics::Point;
use crate::to_centered;
use crate::to_left_up;
use crate::vector::Vector2;

#[derive(Copy, Clone)]
pub struct Circle {
    pub center: Point,
    pub color: [f32; 4],
    pub radius: f64,
}

impl Circle {
    pub fn new(center: Point, radius: f64, color: [f32; 4]) -> Circle {
        Circle {
            center,
            color,
            radius,
        }
    }

    pub fn centered(radius: f64, color: [f32; 4], size: &Size) -> Circle {
        Circle::new(Point::zeros(&Some(*size)), radius, color)
    }

    pub fn at_cursor(cursor: &[f64; 2], radius: f64, color: [f32; 4], size: &Size) -> Circle {
        let position = Vector2::from(*cursor);
        let mut center = Point::stationary(position, &Some(*size));

        to_centered!(center.position, size);
        Circle::new(center, radius, color)
    }

    pub fn at_cursor_random(cursor: &[f64; 2], size: &Size) -> Circle {
        let mut rng = rand::thread_rng();
        let radius: f64 = rng.gen();
        let red: f32 = rng.gen();
        let green: f32 = rng.gen();
        let blue: f32 = rng.gen();
        let color: [f32; 4] = [red, green, blue, 1.];
        Circle::at_cursor(cursor, 40. * radius + 20., color, size)
    }

    pub fn rounding_rect(&self, size: &Size, scale: f64) -> [f64; 4] {
        let diameter = 2. * self.radius;
        let mut position_scaled = self.center.position * scale;
        to_left_up!(position_scaled, size);

        [position_scaled[0] - self.radius, position_scaled[1] - self.radius, diameter, diameter]
    }

    pub fn bound(&mut self, size: Size) -> &mut Circle {
        let x_mid = size.width / 2.;
        let x_left = -self.radius - x_mid;
        let x_right = self.radius + x_mid;
        let y_mid = size.height / 2.;
        let y_up = self.radius + y_mid;
        let y_down = -self.radius - y_mid;

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

    pub fn set_cursor_pos(&mut self, cursor: &[f64; 2], size: &Size) -> &mut Circle {
        self.center.position.set_array(&cursor);
        to_centered!(self.center.position, size);

        self
    }
}

impl Debug for Circle {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "***center***\n{:?}", self.center)
    }
}
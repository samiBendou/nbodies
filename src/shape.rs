use std::fmt::{Debug, Error, Formatter};

use piston::window::Size;

use crate::common::{Color, to_centered, to_left_up};
use crate::physics::Point;
use crate::vector::Vector2;

#[derive(Copy, Clone)]
pub struct Circle {
    pub center: Point,
    pub color: Color,
    pub radius: f64,
}

impl Circle {
    pub fn new(center: Point, radius: f64, color: Color) -> Circle {
        Circle {
            center,
            color,
            radius,
        }
    }

    pub fn centered(radius: f64, color: Color, size: &Size) -> Circle {
        Circle::new(Point::zeros(&Some(*size)), radius, color)
    }

    pub fn at_cursor(cursor: &[f64; 2], radius: f64, color: Color, size: &Size) -> Circle {
        let position = Vector2::from(to_centered(*cursor, size));
        let center = Point::stationary(position, &Some(*size));

        Circle::new(center, radius, color)
    }

    pub fn rounding_rect(&self, size: &Size) -> [f64; 4] {
        let diameter = 2. * self.radius;
        let position = to_left_up(self.center.position.as_array(), &size);

        [position[0] - self.radius, position[1] - self.radius, diameter, diameter]
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
        self.center.position.set_array(&to_centered(*cursor, size));

        self
    }
}

impl Debug for Circle {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "***center***\n{:?}", self.center)
    }
}
use std::fmt::{Debug, Error, Formatter};

use piston::window::Size;

use crate::common::{BASE_ACCELERATION, BASE_SPEED, Color, Direction, RESISTANCE};
use crate::vector::Vector2;

#[derive(Copy, Clone)]
pub struct Circle {
    pub position: Vector2,
    pub speed: Vector2,

    pub radius: f64,
    pub color: Color,
}

impl Circle {
    pub fn new(x: f64, y: f64, radius: f64, color: Color) -> Circle {
        let position = Vector2::new(x, y);
        let speed = Vector2::zeros();
        Circle { position, speed, radius, color }
    }

    pub fn centered(radius: f64, color: Color) -> Circle {
        Circle::new(0., 0., radius, color)
    }

    pub fn at_cursor(cursor: &[f64; 2], size: Size, radius: f64, color: Color) -> Circle {
        let mut circle = Circle::new(0., 0., radius, color);

        *circle.set_pos_from_cursor(cursor, size)
    }

    pub fn rounding_rect(&self, width: f64, height: f64) -> [f64; 4] {
        let diameter = 2. * self.radius;
        let x = self.position.x - self.radius + width / 2.;
        let y = -(self.position.y + self.radius - height / 2.);

        [x, y, diameter, diameter]
    }

    pub fn replace(&mut self, width: f64, height: f64) -> &mut Circle {
        let x_mid = width / 2.;
        let x_left = -self.radius - x_mid;
        let x_right = self.radius + x_mid;
        let y_mid = height / 2.;
        let y_up = self.radius + y_mid;
        let y_down = -self.radius - y_mid;

        if self.position.x < x_left {
            self.position.x = x_right;
        } else if self.position.x > x_right {
            self.position.x = x_left;
        }

        if self.position.y < y_down {
            self.position.y = y_up;
        } else if self.position.y > y_up {
            self.position.y = y_down;
        }

        self
    }

    pub fn reset(&mut self, x: f64, y: f64) -> &mut Circle {
        self.position = Vector2::new(x, y);
        self.speed = Vector2::zeros();

        self
    }

    pub fn translate(&mut self, direction: &Direction) -> &mut Circle {
        self.position += direction.as_vector() * BASE_SPEED;

        self
    }

    pub fn accelerate(&mut self, direction: &Direction, dt: f64) -> &mut Circle {
        let resistance = self.speed * (RESISTANCE * self.speed.magnitude());
        let push = direction.as_vector() * BASE_ACCELERATION;

        self.speed += (push - resistance) * dt;
        self.position += self.speed * dt;

        self
    }

    pub fn set_pos_from_cursor(&mut self, cursor: &[f64; 2], size: Size) -> &mut Circle {
        let x_mid = size.width / 2.;
        let y_mid = size.height / 2.;
        self.position.x = cursor[0] - x_mid;
        self.position.y = y_mid - cursor[1];

        self
    }
}

impl Debug for Circle {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "position: {:?} (px)\nspeed: {:?} (px/s)", self.position, self.speed)
    }
}
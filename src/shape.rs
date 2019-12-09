use crate::common::{BASE_ACCELERATION, BASE_SPEED, Color, Direction, RESISTANCE};
use crate::vector::Vector2;

#[derive(Debug)]
pub struct Circle {
    pub center: Vector2,
    pub speed: Vector2,

    pub radius: f64,
    pub color: Color,
}

impl Circle {
    pub fn new(x: f64, y: f64, radius: f64, color: Color) -> Circle {
        let center = Vector2::new(x, y);
        let speed = Vector2::zeros();
        Circle { center, speed, radius, color }
    }

    pub fn diameter(&self) -> f64 {
        2. * self.radius
    }

    pub fn rounding_rect(&self) -> [f64; 4] {
        let diameter = self.diameter();
        [self.center.x - self.radius, self.center.y - self.radius, diameter, diameter]
    }

    pub fn replace(&mut self, width: f64, height: f64) -> &mut Circle {
        if self.center.x < -self.radius {
            self.center.x = width + self.radius;
        } else if self.center.x > width + self.radius {
            self.center.x = -self.radius;
        }

        if self.center.y < -self.radius {
            self.center.y = height + self.radius;
        } else if self.center.y > height + self.radius {
            self.center.y = -self.radius;
        }

        self
    }

    pub fn reset(&mut self, x: f64, y: f64) -> &mut Circle {
        self.center = Vector2::new(x, y);
        self.speed = Vector2::zeros();

        self
    }

    pub fn translate(&mut self, direction: &Direction) -> &mut Circle {
        self.center += direction.as_vector() * BASE_SPEED;

        self
    }

    pub fn update(&mut self, direction: &Direction, dt: f64) -> &mut Circle {
        let resistance = self.speed * (RESISTANCE * self.speed.magnitude());
        let push = direction.as_vector() * BASE_ACCELERATION;

        self.speed += (push - resistance) * dt;
        self.center += self.speed * dt;

        self
    }
}
use std::fmt::{Debug, Error, Formatter};

use piston::window::Size;

use crate::vector::Vector2;

const BASE_ACCELERATION: f64 = 20000.;
const BASE_TRANSLATION: f64 = 50.;
const RESISTANCE: f64 = 0.01;
pub(crate) const TRAJECTORY_SIZE: usize = 256;

pub fn to_centered(position: [f64; 2], size: &Size) -> [f64; 2] {
    [position[0] - size.width / 2., size.height / 2. - position[1]]
}

pub fn to_left_up(position: [f64; 2], size: &Size) -> [f64; 2] {
    [position[0] + size.width / 2., size.height / 2. - position[1]]
}

fn offset_or_position(position: [f64; 2], size: &Option<Size>) -> [f64; 2] {
    match size {
        Some(size) => to_left_up(position, size),
        None => position
    }
}

#[derive(Copy, Clone)]
pub struct Point {
    pub position: Vector2,
    pub speed: Vector2,
    pub acceleration: Vector2,

    trajectory: [Vector2; TRAJECTORY_SIZE],
    index: usize,
}

impl Point {
    pub fn new(position: Vector2, speed: Vector2, acceleration: Vector2, size: &Option<Size>) -> Point {
        let position_offset = Vector2::from(&offset_or_position(position.as_array(), size));

        Point {
            position,
            speed,
            acceleration,
            trajectory: [position_offset; TRAJECTORY_SIZE],
            index: 0,
        }
    }

    pub fn zeros(size: &Option<Size>) -> Point {
        Point::new(Vector2::zeros(), Vector2::zeros(), Vector2::zeros(), size)
    }

    pub fn zeros_acceleration(position: Vector2, speed: Vector2, size: &Option<Size>) -> Point {
        Point::new(position, speed, Vector2::zeros(), size)
    }

    pub fn at_position(position: Vector2, size: &Option<Size>) -> Point {
        Point::new(position, Vector2::zeros(), Vector2::zeros(), size)
    }

    pub fn reset(&mut self, position: Vector2) -> &mut Point {
        self.position = position;
        self.speed = Vector2::zeros();
        self.acceleration = Vector2::zeros();

        self
    }

    pub fn translate(&mut self, direction: &Vector2) -> &mut Point {
        self.position += *direction * BASE_TRANSLATION;

        self
    }

    pub fn accelerate(&mut self, dt: f64) -> &mut Point {
        self.speed += self.acceleration * dt;
        self.position += self.speed * dt;

        self
    }

    pub fn position(&self, k: usize) -> &Vector2 {
        let index = (k + self.index + 1) % TRAJECTORY_SIZE;

        &self.trajectory[index]
    }

    pub fn update_trajectory(&mut self, size: &Option<Size>) {
        let position_offset = &offset_or_position(self.position.as_array(), size);
        self.trajectory[self.index].set_array(position_offset);
        self.index = (self.index + 1) % TRAJECTORY_SIZE;
    }

    pub fn clear_trajectory(&mut self, size: &Option<Size>) {
        let position_offset = &offset_or_position(self.position.as_array(), size);
        for position in self.trajectory.iter_mut() {
            position.set_array(position_offset);
        }
    }
}

impl Debug for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "position: {:?} (px)\nspeed: {:?} (px/s)\nacceleration: {:?} (px/s2)",
            self.position, self.speed, self.acceleration
        )
    }
}

pub mod forces {
    use crate::common::Direction;
    use crate::physics::{BASE_ACCELERATION, RESISTANCE};
    use crate::vector::Vector2;

    pub fn push(direction: &Direction) -> Vector2 {
        direction.as_vector() * BASE_ACCELERATION
    }

    pub fn nav_stokes(speed: &Vector2) -> Vector2 {
        *speed * (-RESISTANCE * speed.magnitude())
    }
}
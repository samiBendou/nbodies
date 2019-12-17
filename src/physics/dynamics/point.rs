use std::fmt::{Debug, Error, Formatter};
use std::ops::{AddAssign, DivAssign, Mul, MulAssign, Rem, SubAssign};

use crate::physics::units::{Compound, Rescale, Scale, Serialize, Unit};
use crate::physics::units::suffix::{Distance, Time};
use crate::physics::vector::Vector2;

pub const TRAJECTORY_SIZE: usize = 256;

#[derive(Copy, Clone)]
pub struct Point2 {
    pub position: Vector2,
    pub speed: Vector2,
    pub acceleration: Vector2,

    trajectory: [Vector2; TRAJECTORY_SIZE],
    index: usize,
}

impl Point2 {

    pub fn new(position: Vector2, speed: Vector2, acceleration: Vector2) -> Point2 {
        Point2 {
            position,
            speed,
            acceleration,
            trajectory: [position.clone(); TRAJECTORY_SIZE],
            index: 0,
        }
    }

    pub fn inertial(position: Vector2, speed: Vector2) -> Point2 {
        Point2::new(position, speed, Vector2::zeros())
    }

    pub fn stationary(position: Vector2) -> Point2 {
        Point2::new(position, Vector2::zeros(), Vector2::zeros())
    }

    pub fn zeros() -> Point2 {
        Point2::new(Vector2::zeros(), Vector2::zeros(), Vector2::zeros())
    }

    pub fn reset0(&mut self) -> &mut Self {
        self.position.reset0();
        self.speed.reset0();
        self.acceleration.reset0();
        self
    }

    pub fn reset(&mut self, position: Vector2) -> &mut Self {
        self.position = position;
        self.speed.reset0();
        self.acceleration.reset0();
        self
    }

    pub fn scale_position(&mut self, scale: f64) -> &mut Self {
        self.position *= scale;
        self
    }

    pub fn scale_speed(&mut self, scale: f64) -> &mut Self {
        self.speed *= scale;
        self
    }

    pub fn translate(&mut self, direction: &Vector2) -> &mut Self {
        self.position += *direction;
        self
    }

    pub fn accelerate(&mut self, dt: f64) -> &mut Self {
        self.speed += self.acceleration * dt;
        self.position += self.speed * dt;
        self
    }

    pub fn position(&self, k: usize) -> &Vector2 {
        let index = (k + self.index + 1) % TRAJECTORY_SIZE;

        &self.trajectory[index]
    }

    pub fn update_trajectory(&mut self) {
        self.trajectory[self.index] = self.position;
        self.index = (self.index + 1) % TRAJECTORY_SIZE;
    }

    pub fn clear_trajectory(&mut self) {
        for position in self.trajectory.iter_mut() {
            *position = self.position;
        }
    }

    pub fn set_origin(&mut self, origin: &Point2, old_origin: &Option<Point2>) -> &mut Self {
        let mut translation = *origin;
        if let Some(old_origin) = old_origin {
            translation -= *old_origin
        }
        *self -= translation;
        self
    }
}

impl Debug for Point2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let time_unit = Unit::from(Scale::from(Time::Second));
        let mut position_unit = Unit::from(Scale::from(Distance::Meter));
        let mut speed_unit = position_unit.clone() / time_unit.clone();
        let mut acceleration_unit = speed_unit.clone() / time_unit;
        position_unit.rescale(self.position.magnitude());
        speed_unit.units[0].rescale(self.speed.magnitude());
        acceleration_unit.units[0].rescale(self.acceleration.magnitude());
        write!(
            f,
            "position: {}\nspeed: {}\nacceleration: {}",
            position_unit.string_of(self.position),
            speed_unit.string_of(self.speed),
            acceleration_unit.string_of(self.acceleration),
        )
    }
}

impl PartialEq for Point2 {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position && self.speed == other.speed
    }

    fn ne(&self, other: &Self) -> bool {
        self.position != other.position || self.speed != other.speed
    }
}

impl AddAssign<Point2> for Point2 {
    fn add_assign(&mut self, rhs: Point2) {
        self.position += rhs.position;
        self.speed += rhs.speed;
        for k in 0..TRAJECTORY_SIZE {
            self.trajectory[k] += rhs.trajectory[k];
        }
    }
}

impl SubAssign<Point2> for Point2 {
    fn sub_assign(&mut self, rhs: Point2) {
        self.position -= rhs.position;
        self.speed -= rhs.speed;
        for k in 0..TRAJECTORY_SIZE {
            self.trajectory[k] -= rhs.trajectory[k];
        }
    }
}

impl Mul<f64> for Point2 {
    type Output = Point2;

    fn mul(self, rhs: f64) -> Self::Output {
        let mut output = self;
        output *= rhs;
        output
    }
}

impl MulAssign<f64> for Point2 {
    fn mul_assign(&mut self, rhs: f64) {
        self.position *= rhs;
        self.speed *= rhs;
        for k in 0..TRAJECTORY_SIZE {
            self.trajectory[k] *= rhs;
        }
    }
}

impl DivAssign<f64> for Point2 {
    fn div_assign(&mut self, rhs: f64) {
        self.position /= rhs;
        self.speed /= rhs;
        for k in 0..TRAJECTORY_SIZE {
            self.trajectory[k] /= rhs;
        }
    }
}

impl Rem<Point2> for Point2 {
    type Output = f64;

    fn rem(self, rhs: Point2) -> Self::Output {
        self.position % rhs.position
    }
}


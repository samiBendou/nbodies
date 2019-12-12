use crate::common::Direction;
use crate::physics::vector::*;

const BASE_ACCELERATION: f64 = 500.;
const RESISTANCE: f64 = 0.001;

pub fn push(direction: &Direction) -> Vector2 {
    direction.as_vector() * BASE_ACCELERATION
}

pub fn nav_stokes(speed: &Vector2) -> Vector2 {
    *speed * (-RESISTANCE * speed.magnitude())
}

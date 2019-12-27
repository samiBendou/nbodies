use crate::common::Direction;
use crate::physics::dynamics::{Body, Cluster};
use crate::physics::units::consts::G_UNIV;
use crate::physics::vector::*;

const BASE_ACCELERATION: f64 = 500.;
const RESISTANCE: f64 = 0.001;

pub fn push(direction: &Direction) -> Vector2 {
    direction.as_vector() * BASE_ACCELERATION
}

pub fn nav_stokes(speed: &Vector2) -> Vector2 {
    *speed * (-RESISTANCE * speed.magnitude())
}

pub fn gravity(body: &Body, cluster: &Cluster) -> Vector2 {
    let mut result = Vector2::zeros();
    let mut distance: Vector2;
    let mut magnitude: f64;
    for i in 0..cluster.count() {
        distance = cluster[i].shape.center.position - body.shape.center.position;
        magnitude = distance.magnitude();
        if magnitude < std::f64::EPSILON {
            continue;
        }
        result += distance * G_UNIV * body.mass * cluster[i].mass / (magnitude * magnitude * magnitude);
    }
    result
}

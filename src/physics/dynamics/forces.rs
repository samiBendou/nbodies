use crate::common::Direction;
use crate::physics::dynamics::{Cluster, point::Point2};
use crate::physics::units::consts::G_UNIV;
use crate::physics::vector::*;

const BASE_ACCELERATION: f64 = 500.;
const RESISTANCE: f64 = 0.001;

pub fn push(direction: &Direction) -> Vector2 {
    direction.as_vector() * BASE_ACCELERATION
}

pub fn nav_stokes(point: &Point2) -> Vector2 {
    point.speed * (-RESISTANCE * point.speed.magnitude())
}

pub fn gravity(point: &Point2, cluster: &Cluster) -> Vector2 {
    let mut result = Vector2::zeros();
    let mut distance: Vector2;
    let mut magnitude: f64;
    for i in 0..cluster.count() {
        distance = cluster[i].shape.center.position - point.position;
        magnitude = distance.magnitude();
        if magnitude < std::f64::EPSILON {
            continue;
        }
        result += distance * G_UNIV * point.mass * cluster[i].mass / (magnitude * magnitude * magnitude);
    }
    result
}

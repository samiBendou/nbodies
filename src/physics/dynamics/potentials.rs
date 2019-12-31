use crate::physics::dynamics::{Cluster, point::Point2};
use crate::physics::units::consts::G_UNIV;

pub fn gravity(point: &Point2, cluster: &Cluster) -> f64 {
    let mut ret = 0.;
    let mut distance: f64;
    for i in 0..cluster.count() {
        distance = cluster[i].shape.center.position % point.position;
        if distance < std::f64::EPSILON {
            continue;
        }
        ret += G_UNIV * cluster[i].mass / distance;
    }
    ret
}
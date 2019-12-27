use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::exit;

use serde::{Deserialize, Serialize};

use crate::physics::vector::Vector2;

#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
pub struct Inclination {
    pub value: f64,
    pub argument: f64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
pub struct Orbit {
    pub mu: f64,
    pub apoapsis: f64,
    pub periapsis: f64,
    pub argument: f64,
    pub inclination: Inclination,
}

impl Orbit {
    pub fn semi_minor(&self) -> f64 {
        (self.apoapsis * self.periapsis).sqrt()
    }
    pub fn semi_major(&self) -> f64 {
        0.5 * (self.apoapsis + self.periapsis)
    }

    pub fn eccentricity(&self) -> f64 {
        let ra = self.apoapsis;
        let rp = self.periapsis;
        let total_r = ra + rp;
        if total_r > 0. {
            (ra - rp) / total_r
        } else {
            0.
        }
    }

    pub fn radius_at(&self, true_anomaly: f64) -> f64 {
        let a = self.semi_major();
        let epsilon = self.eccentricity();
        a * (1. - epsilon * epsilon) / (1. + epsilon * true_anomaly.cos())
    }

    pub fn eccentric_anomaly_at(&self, true_anomaly: f64) -> f64 {
        let epsilon = self.eccentricity();
        (true_anomaly.sin() * (1. - epsilon * epsilon).sqrt() / (1. + epsilon * true_anomaly.cos())).atan()
    }

    pub fn flight_angle_at(&self, true_anomaly: f64) -> f64 {
        let epsilon = self.eccentricity();
        let ec = epsilon * true_anomaly.cos();
        ((1. + ec) / (1. + epsilon * epsilon + 2. * ec).sqrt()).min(1.0).acos()
    }

    pub fn position_at(&self, true_anomaly: f64) -> Vector2 {
        let r = self.radius_at(true_anomaly);
        Vector2::radial(r, true_anomaly + self.argument)
    }

    pub fn speed_at(&self, true_anomaly: f64) -> Vector2 {
        let a = self.semi_major();
        if a == 0. {
            return Vector2::zeros();
        }
        let pi_frac_2 = std::f64::consts::FRAC_PI_2;
        let epsilon = self.eccentricity();
        let ang = true_anomaly + pi_frac_2 - self.flight_angle_at(true_anomaly);
        let mag = (self.mu * (2. / self.radius_at(true_anomaly) - 1. / a)).sqrt();
        Vector2::radial(mag, ang + self.argument)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Body {
    pub name: String,
    pub mass: f64,
    pub color: [f32; 4],
    pub radius: f64,
    pub orbit: Orbit,
}

impl From<String> for Body {
    fn from(json: String) -> Self {
        serde_json::from_str(&json).unwrap()
    }
}

impl From<&Path> for Body {
    fn from(path: &Path) -> Self {
        let mut contents = String::new();
        if let mut file = File::open(path).unwrap() {
            file.read_to_string(&mut contents).unwrap_err();
        }
        Body::from(contents)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Cluster {
    pub bodies: Vec<Body>
}

impl From<String> for Cluster {
    fn from(json: String) -> Self {
        Cluster { bodies: serde_json::from_str(&json).unwrap() }
    }
}

impl From<&Path> for Cluster {
    fn from(path: &Path) -> Self {
        let mut contents = String::new();
        if let mut file = File::open(path).unwrap() {
            file.read_to_string(&mut contents).unwrap();
        }
        Cluster::from(contents)
    }
}

#[cfg(test)]
mod tests {
    mod cluster {
        use std::path::Path;

        use super::super::Cluster;

        #[test]
        fn simple_deserialize() {
            let path: &Path = Path::new("data/solar_system.json");
            let cluster = Cluster::from(path);
            println!("{:?}", cluster);
        }
    }
}


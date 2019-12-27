use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

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

    pub fn is_degenerated(&self) -> bool {
        let epsilon_f64 = std::f64::EPSILON;
        if self.semi_minor() < epsilon_f64 || self.semi_major() < epsilon_f64 {
            true
        } else {
            false
        }
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
        let mag = self.radius_at(true_anomaly);
        Vector2::radial(mag, true_anomaly + self.argument)
    }

    pub fn speed_at(&self, true_anomaly: f64) -> Vector2 {
        if self.is_degenerated() {
            return Vector2::zeros();
        }
        let ang = true_anomaly + std::f64::consts::FRAC_PI_2 - self.flight_angle_at(true_anomaly);
        let mag = (self.mu * (2. / self.radius_at(true_anomaly) - 1. / self.semi_major())).sqrt();
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Cluster {
    pub bodies: Vec<Body>
}

impl Cluster {
    pub fn from_file(path: &Path) -> Result<Self, io::Error> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        match serde_json::from_str(&contents) {
            Ok(bodies) => Ok(Cluster { bodies }),
            Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e))
        }
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
            let cluster = Cluster::from_file(path);
            println!("{:?}", cluster);
        }
    }
}


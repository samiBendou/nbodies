use std::error::Error;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

use rand::prelude::*;
use serde::{Deserialize, Serialize};

use crate::physics::vector::Vector2;

#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
pub enum Kind {
    Artificial,
    Terrestrial,
    Giant,
    Star,
    Hole,
}

impl Kind {
    pub fn random() -> Kind {
        use Kind::*;
        let mut rng = rand::thread_rng();
        match rng.gen_range(0, 4) {
            1 => Terrestrial,
            2 => Giant,
            3 => Star,
            4 => Hole,
            _ => Artificial,
        }
    }

    pub fn random_mass(&self) -> f64 {
        let mut rng = rand::thread_rng();
        match self {
            Kind::Artificial => rng.gen_range(1., 1e6),
            Kind::Terrestrial => rng.gen_range(1e22, 1e25),
            Kind::Giant => rng.gen_range(1e25, 1e28),
            Kind::Star => rng.gen_range(1e28, 1e31),
            Kind::Hole => rng.gen_range(1e32, 1e31),
        }
    }


    pub fn random_radius(&self) -> f64 {
        let mut rng = rand::thread_rng();
        match self {
            Kind::Artificial => rng.gen_range(1., 100.),
            Kind::Terrestrial => rng.gen_range(1e6, 1e7),
            Kind::Giant => rng.gen_range(1e7, 1e8),
            Kind::Star => rng.gen_range(1e7, 1e9),
            Kind::Hole => rng.gen_range(1e6, 1e10),
        }
    }

    pub fn scaled_radius(&self, radius: f64) -> f64 {
        radius / 10f64.powf(radius.log10()) + match self {
            Kind::Artificial => 1.,
            Kind::Terrestrial => 20.,
            Kind::Giant => 22.,
            Kind::Star => 24.,
            Kind::Hole => 24.,
        }
    }
}

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
    pub kind: Kind,
    pub color: [f32; 4],
    pub radius: f64,
    pub orbit: Orbit,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Cluster {
    pub bodies: Vec<Body>
}

impl Cluster {
    pub fn from_file(path: &Path) -> Result<Self, Box<dyn Error>> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let bodies: Vec<Body> = serde_json::from_str(&contents)?;
        Ok(Cluster { bodies })
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


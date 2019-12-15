use std::ops::{Add, AddAssign, Sub, SubAssign};

use super::*;

pub struct Real {
    pub raw: f64,
    pub unit: Unit,
}

impl Real {
    pub fn new(raw: f64, suffix: Scale) -> Real {
        Real { raw, unit: Unit::from(suffix) }
    }
}

impl Rescale<Scale> for Real {
    fn rescale(&mut self, prefix: Scale) -> &mut Self {
        self.unit.rescale(prefix);
        self
    }
}

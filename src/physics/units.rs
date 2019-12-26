use std::fmt::Debug;
use std::ops::{Div, DivAssign, Mul, MulAssign};

use consts::*;

use crate::physics::vector::Vector2;

pub mod prefix;
pub mod suffix;
pub mod consts;
pub mod date;

pub trait Rescale<T> {
    fn rescale(&mut self, prefix: &T) -> &mut Self;
}

pub trait Serialize<T> {
    fn string_of(&self, val: &T) -> String;
}

pub trait Convert<T> {
    fn value_of(&self, val: &T) -> T;
}

#[derive(Debug, Clone)]
pub struct Scale {
    pub label: String,
    pub multiplier: f64,
    pub pow: i8,
}

impl Scale {
    pub fn new(label: &str, multiplier: f64, pow: i8) -> Scale {
        Scale { label: String::from(label), multiplier, pow }
    }
}

impl Rescale<prefix::Calendar> for Scale {
    fn rescale(&mut self, prefix: &prefix::Calendar) -> &mut Self {
        use prefix::Calendar::*;
        *self = match prefix {
            Year => Scale::new("years", YEAR_PER_SEC, 1),
            Month => Scale::new("months", MONTH_PER_SEC, 1),
            Week => Scale::new("weeks", WEEK_PER_SEC, 1),
            Day => Scale::new("days", DAY_PER_SEC, 1),
            Hour => Scale::new("hours", HOUR_PER_SEC, 1),
            Minute => Scale::new("minutes", MIN_PER_SEC, 1),
            Second => Scale::new("seconds", 1., 1),
        };
        self
    }
}

impl Rescale<prefix::Standard> for Scale {
    fn rescale(&mut self, prefix: &prefix::Standard) -> &mut Self {
        use prefix::Standard::*;
        *self = match prefix {
            Femto => Scale::new("f", 1e15, 1),
            Pico => Scale::new("p", 1e12, 1),
            Nano => Scale::new("n", 1e9, 1),
            Micro => Scale::new("µ", 1e6, 1),
            Milli => Scale::new("m", 1e3, 1),
            Base => Scale::new("", 1., 1),
            Kilo => Scale::new("k", 1e-3, 1),
            Mega => Scale::new("M", 1e-6, 1),
            Giga => Scale::new("G", 1e-9, 1),
            Tera => Scale::new("T", 1e-12, 1),
            Peta => Scale::new("P", 1e-15, 1),
        };
        self
    }
}

impl Rescale<f64> for Scale {
    fn rescale(&mut self, val: &f64) -> &mut Self {
        self.rescale(&prefix::Standard::from(*val));
        self
    }
}


impl From<prefix::Calendar> for Scale {
    fn from(prefix: prefix::Calendar) -> Self {
        let mut scale = Scale::new("", 0., 1);
        scale.rescale(&prefix);
        scale
    }
}

impl From<prefix::Standard> for Scale {
    fn from(prefix: prefix::Standard) -> Self {
        let mut scale = Scale::new("", 0., 1);
        scale.rescale(&prefix);
        scale
    }
}

impl From<f64> for Scale {
    fn from(val: f64) -> Self {
        let mut scale = Scale::new("", 1., 1);
        scale.rescale(&val);
        scale
    }
}

impl From<suffix::Distance> for Scale {
    fn from(suffix: suffix::Distance) -> Self {
        use suffix::Distance::*;
        match suffix {
            Meter => Scale::new("m", 1., 1),
            Astronomic => Scale::new("au", AU_PER_METER, 1),
            Light => Scale::new("ls", LS_PER_METER, 1),
            Pixel => Scale::new("px", PX_PER_METER, 1),
        }
    }
}

impl From<suffix::Time> for Scale {
    fn from(suffix: suffix::Time) -> Self {
        use suffix::Time::*;
        match suffix {
            Second => Scale::new("s", 1., 1),
            Light => Scale::new("lm", LM_PER_SEC, 1),
            Calendar => Scale::new("", 1., 1),
        }
    }
}

impl From<suffix::Mass> for Scale {
    fn from(suffix: suffix::Mass) -> Self {
        use suffix::Mass::*;
        match suffix {
            Grams => Scale::new("g", 1., 1),
            Kilograms => Scale::new("kg", 1., 1),
            Tons => Scale::new("t", TONS_PER_KG, 1),
        }
    }
}

impl From<suffix::Angle> for Scale {
    fn from(suffix: suffix::Angle) -> Self {
        use suffix::Angle::*;
        use std::f64::consts::PI;
        match suffix {
            Radians => Scale::new("rad", 1., 1),
            Degrees => Scale::new("deg", 180. / PI, 1),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Unit {
    pub prefix: Scale,
    suffix: Scale,
}

impl Unit {
    pub fn new(prefix: Scale, suffix: Scale) -> Unit {
        Unit { prefix, suffix }
    }

    pub fn label(&self) -> String {
        let pow = if self.suffix.pow == 1 {
            String::new()
        } else {
            format!("{}", self.suffix.pow)
        };
        format!("{}{}{}", self.prefix.label, self.suffix.label, pow.as_str())
    }
}

impl Rescale<Scale> for Unit {
    fn rescale(&mut self, prefix: &Scale) -> &mut Self {
        self.prefix = prefix.clone();
        self
    }
}

impl Rescale<f64> for Unit {
    fn rescale(&mut self, val: &f64) -> &mut Self {
        self.prefix = Scale::from(*val);
        self
    }
}

impl From<Scale> for Unit {
    fn from(suffix: Scale) -> Self {
        Unit::new(Scale::new("", 1., 1), suffix)
    }
}


impl Mul<Unit> for Unit {
    type Output = Compound;

    fn mul(self, rhs: Unit) -> Self::Output {
        Compound::new(vec![self]) * rhs
    }
}

impl Div<Unit> for Unit {
    type Output = Compound;

    fn div(self, rhs: Unit) -> Self::Output {
        Compound::new(vec![self]) / rhs
    }
}


impl<T> Convert<T> for Unit where
    T: MulAssign<f64> + DivAssign<f64> + Clone {
    fn value_of(&self, val: &T) -> T {
        let mut result = val.clone();
        if self.suffix.pow > 0 {
            result *= self.prefix.multiplier * self.suffix.multiplier;
        } else {
            result /= self.prefix.multiplier * self.suffix.multiplier;
        }
        result
    }
}

impl Serialize<f64> for Unit {
    fn string_of(&self, val: &f64) -> String {
        let value_dim = self.value_of(val);
        if value_dim > 1000. {
            format!("{:.5e} ({})", value_dim, self.label())
        } else {
            format!("{:.3} ({})", value_dim, self.label())
        }
    }
}

impl Serialize<Vector2> for Unit {
    fn string_of(&self, vector: &Vector2) -> String {
        format!("{:?} ({})", self.value_of(vector), self.label())
    }
}

#[derive(Clone)]
pub struct Compound {
    pub units: Vec<Unit>
}

impl Compound {
    pub fn new(units: Vec<Unit>) -> Compound {
        Compound { units }
    }

    pub fn label(&self) -> String {
        let mut num = String::new();
        let mut den = String::new();
        let mut pow: String;
        for unit in self.units.iter() {
            pow = if unit.suffix.pow.abs() == 1 {
                String::new()
            } else {
                format!("{}", unit.suffix.pow.abs())
            };
            if unit.suffix.pow < 0 {
                den += format!("{}{}{}", unit.prefix.label, unit.suffix.label, pow.as_str()).as_str();
            } else {
                num += format!("{}{}{}", unit.prefix.label, unit.suffix.label, pow.as_str()).as_str();
            }
        }
        if den.is_empty() {
            num
        } else {
            num + "/" + den.as_str()
        }
    }
}

impl<T> Convert<T> for Compound where T: MulAssign<f64> + Clone {
    fn value_of(&self, val: &T) -> T {
        let mut result = val.clone();
        for unit in self.units.iter() {
            result *= unit.value_of(&1.);
        }
        result
    }
}

impl Serialize<f64> for Compound {
    fn string_of(&self, val: &f64) -> String {
        format!("{:.3} ({})", self.value_of(val), self.label())
    }
}

impl Serialize<Vector2> for Compound {
    fn string_of(&self, vector: &Vector2) -> String {
        format!("{:?} ({})", self.value_of(vector), self.label())
    }
}

impl MulAssign<Unit> for Compound {
    fn mul_assign(&mut self, rhs: Unit) {
        let mut same: Vec<&mut Unit> = self.units.iter_mut()
            .filter(|unit| unit.suffix.label == rhs.suffix.label)
            .collect();
        if same.len() == 0 {
            self.units.push(rhs);
        } else {
            same[0].suffix.pow += rhs.suffix.pow;
        }
    }
}

impl DivAssign<Unit> for Compound {
    fn div_assign(&mut self, rhs: Unit) {
        let mut unit = rhs.clone();
        unit.suffix.pow = -unit.suffix.pow;
        *self *= unit
    }
}


impl Mul<Unit> for Compound {
    type Output = Compound;

    fn mul(self, rhs: Unit) -> Self::Output {
        let mut result = self.clone();
        result *= rhs;
        result
    }
}

impl Div<Unit> for Compound {
    type Output = Compound;

    fn div(self, rhs: Unit) -> Self::Output {
        let mut result = self.clone();
        result /= rhs;
        result
    }
}

impl Mul<Compound> for Compound {
    type Output = Compound;

    fn mul(self, rhs: Compound) -> Self::Output {
        let mut result = self.clone();
        for unit in rhs.units.iter() {
            result *= unit.clone();
        }
        result
    }
}

impl Div<Compound> for Compound {
    type Output = Compound;

    fn div(self, rhs: Compound) -> Self::Output {
        let mut result = self.clone();
        for unit in rhs.units.iter() {
            result /= unit.clone();
        }
        result
    }
}
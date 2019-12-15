use std::fmt::Debug;

use consts::*;

use crate::physics::vector::Vector2;

pub mod prefix;
pub mod suffix;
pub mod consts;
pub mod date;

pub trait Rescale<T> {
    fn rescale(&mut self, prefix: T) -> &mut Self;
}

pub trait Serialize<T> {
    fn string_of(&self, val: T) -> String;
}

pub trait Convert<T> {
    fn value_of(&self, val: T) -> T;
}

#[derive(Debug)]
pub struct Scale {
    pub label: String,
    pub multiplier: f64,
}

impl Scale {
    pub fn new(label: &str, multiplier: f64) -> Scale {
        Scale { label: String::from(label), multiplier }
    }
}

impl Rescale<prefix::Calendar> for Scale {
    fn rescale(&mut self, prefix: prefix::Calendar) -> &mut Self {
        use prefix::Calendar::*;
        *self = match prefix {
            Year => Scale::new("years", YEAR_PER_SEC),
            Month => Scale::new("months", MONTH_PER_SEC),
            Week => Scale::new("weeks", WEEK_PER_SEC),
            Day => Scale::new("days", DAY_PER_SEC),
            Hour => Scale::new("hours", HOUR_PER_SEC),
            Minute => Scale::new("minutes", MIN_PER_SEC),
            Second => Scale::new("seconds", 1.),
        };
        self
    }
}

impl Rescale<prefix::Standard> for Scale {
    fn rescale(&mut self, prefix: prefix::Standard) -> &mut Self {
        use prefix::Standard::*;
        *self = match prefix {
            Base => Scale::new("", 1.),
            Kilo => Scale::new("k", 1e-3),
            Mega => Scale::new("M", 1e-6),
            Giga => Scale::new("G", 1e-9),
            Tera => Scale::new("T", 1e-12),
        };
        self
    }
}

impl Rescale<f64> for Scale {
    fn rescale(&mut self, val: f64) -> &mut Self {
        self.rescale(prefix::Standard::from(val));
        self
    }
}


impl From<prefix::Calendar> for Scale {
    fn from(prefix: prefix::Calendar) -> Self {
        let mut scale = Scale::new("", 0.);
        scale.rescale(prefix);
        scale
    }
}

impl From<prefix::Standard> for Scale {
    fn from(prefix: prefix::Standard) -> Self {
        let mut scale = Scale::new("", 0.);
        scale.rescale(prefix);
        scale
    }
}

impl From<f64> for Scale {
    fn from(val: f64) -> Self {
        let mut scale = Scale::new("", 0.);
        scale.rescale(val);
        scale
    }
}

impl From<suffix::Distance> for Scale {
    fn from(suffix: suffix::Distance) -> Self {
        use suffix::Distance::*;
        match suffix {
            Standard => Scale::new("m", 1.),
            Astronomic => Scale::new("au", AU_PER_METER),
            Light => Scale::new("ls", LS_PER_METER),
        }
    }
}

impl From<suffix::Time> for Scale {
    fn from(suffix: suffix::Time) -> Self {
        use suffix::Time::*;
        match suffix {
            Standard => Scale::new("s", 1.),
            Light => Scale::new("lm", LM_PER_SEC),
            Calendar => Scale::new("", 1.),
        }
    }
}

impl From<suffix::Mass> for Scale {
    fn from(suffix: suffix::Mass) -> Self {
        use suffix::Mass::*;
        match suffix {
            Standard => Scale::new("g", 1.),
            Tons => Scale::new("t", TONS_PER_KG),
        }
    }
}

#[derive(Debug)]
pub struct Unit {
    pub(crate) prefix: Scale,
    suffix: Scale,
}

impl Unit {
    pub fn new(prefix: Scale, suffix: Scale) -> Unit {
        Unit { prefix, suffix }
    }

    pub fn label(&self) -> String {
        format!("{}{}", self.prefix.label, self.suffix.label)
    }
}

impl Rescale<Scale> for Unit {
    fn rescale(&mut self, prefix: Scale) -> &mut Self {
        self.prefix = prefix;
        self
    }
}

impl Rescale<f64> for Unit {
    fn rescale(&mut self, val: f64) -> &mut Self {
        self.prefix = Scale::from(val);
        self
    }
}

impl From<Scale> for Unit {
    fn from(suffix: Scale) -> Self {
        Unit::new(Scale::new("", 1.), suffix)
    }
}

impl Convert<f64> for Unit {
    fn value_of(&self, val: f64) -> f64 {
        val * (self.prefix.multiplier * self.suffix.multiplier)
    }
}

impl Convert<Vector2> for Unit {
    fn value_of(&self, val: Vector2) -> Vector2 {
        val * (self.prefix.multiplier * self.suffix.multiplier)
    }
}


impl Serialize<f64> for Unit {
    fn string_of(&self, val: f64) -> String {
        format!("{:.3} ({})", self.value_of(val), self.label())
    }
}

impl Serialize<Vector2> for Unit {
    fn string_of(&self, vector: Vector2) -> String {
        format!("{:?} ({})", self.value_of(vector), self.label())
    }
}
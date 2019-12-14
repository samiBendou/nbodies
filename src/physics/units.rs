use std::fmt::{Debug, Error, Formatter};
use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

use crate::physics::units::suffix::Calendar;
use crate::physics::vector::Vector2;

pub const PX_PER_METER: f64 = 1.;
pub const AU_PER_METER: f64 = 6.684587122268445e-12;
// astronomic units
pub const LS_PER_METER: f64 = 3.3356409519815204e-09; // light second (distance)

pub const MIN_PER_SEC: f64 = 0.016666666666666666;
pub const HOUR_PER_SEC: f64 = 0.0002777777777777778;
pub const DAY_PER_SEC: f64 = 1.1574074074074073e-05;
pub const WEEK_PER_SEC: f64 = 1.6534391534391533e-06;
pub const MONTH_PER_SEC: f64 = 5.5114638447971775e-08;
pub const YEAR_PER_SEC: f64 = 4.5928865373309815e-09;
pub const LM_PER_SEC: f64 = 3.3356409519815204e-09; // light meter (time)

pub const TONS_PER_KG: f64 = 1e-3;

pub mod prefix {
    use super::Scale;

    pub enum Standard {
        Base,
        Kilo,
        Mega,
        Giga,
        Tera,
    }

    impl From<f64> for Standard {
        fn from(val: f64) -> Self {
            use Standard::*;
            match val.abs().log10().floor() as i32 {
                0 | 1 | 2 => Base,
                3 | 4 | 5 => Kilo,
                6 | 7 | 8 => Mega,
                9 | 10 | 11 => Giga,
                12 | 13 | 14 => Tera,
                _ => Base,
            }
        }
    }
}

pub mod suffix {
    use crate::physics::units::{DAY_PER_SEC, HOUR_PER_SEC, MIN_PER_SEC, MONTH_PER_SEC, WEEK_PER_SEC, YEAR_PER_SEC};

    pub enum Distance {
        Standard,
        Astronomic,
        Light,
    }

    pub enum Time {
        Standard,
        Calendar,
        Light,
    }

    pub enum Calendar {
        Second,
        Minute,
        Hour,
        Day,
        Week,
        Month,
        Year,
    }

    impl From<f64> for Calendar {
        fn from(val: f64) -> Self {
            use Calendar::*;

            if val * YEAR_PER_SEC > 1. {
                Year
            } else if val * MONTH_PER_SEC > 1. {
                Month
            } else if val * WEEK_PER_SEC > 1. {
                Week
            } else if val * DAY_PER_SEC > 1. {
                Day
            } else if val * HOUR_PER_SEC > 1. {
                Hour
            } else if val * MIN_PER_SEC > 1. {
                Minute
            } else {
                Second
            }
        }
    }

    pub enum Mass {
        Standard,
        Tons,
    }
}

#[derive(Debug)]
pub struct Scale<'a> {
    pub label: &'a str,
    pub multiplier: f64,
}

impl From<f64> for Scale<'static> {
    fn from(val: f64) -> Self {
        use prefix::Standard::*;
        match prefix::Standard::from(val) {
            Base => Scale::new("", 1.),
            Kilo => Scale::new("k", 1e-3),
            Mega => Scale::new("M", 1e-6),
            Giga => Scale::new("G", 1e-9),
            Tera => Scale::new("T", 1e-12),
        }
    }
}

impl From<Vector2> for Scale<'static> {
    fn from(vector: Vector2) -> Self {
        Scale::from(vector.magnitude())
    }
}

impl Scale<'static> {
    pub fn new(label: &str, multiplier: f64) -> Scale {
        Scale { label, multiplier }
    }

    pub fn value_of(&self, val: f64) -> f64 {
        val * self.multiplier
    }
}

pub trait Rescale<T> {
    fn rescale(&mut self, val: T) -> &mut Self;
}

pub trait Serialize<T> {
    fn string_of(&self, val: T) -> String;
}

pub trait Convert<T> {
    fn value_of(&self, val: T) -> T;
}

#[derive(Debug)]
pub struct Unit<'a> {
    pub prefix: Scale<'a>,
    pub suffix: Scale<'a>,
}

impl Unit<'static> {
    pub fn new(prefix: Scale<'static>, suffix: Scale<'static>) -> Unit<'static> {
        Unit { prefix, suffix }
    }
}

impl Convert<f64> for Unit<'static> {
    fn value_of(&self, val: f64) -> f64 {
        val * (self.prefix.multiplier * self.suffix.multiplier)
    }
}

impl Convert<Vector2> for Unit<'static> {
    fn value_of(&self, val: Vector2) -> Vector2 {
        val * (self.prefix.multiplier * self.suffix.multiplier)
    }
}

impl Rescale<f64> for Unit<'static> {
    fn rescale(&mut self, val: f64) -> &mut Self {
        self.prefix = Scale::from(val);
        self
    }
}

impl Rescale<Vector2> for Unit<'static> {
    fn rescale(&mut self, vector: Vector2) -> &mut Self {
        self.prefix = Scale::from(vector);
        self
    }
}

impl Serialize<f64> for Unit<'static> {
    fn string_of(&self, val: f64) -> String {
        format!("{:.3} ({}{})", self.value_of(val), self.prefix.label, self.suffix.label)
    }
}

impl Serialize<Vector2> for Unit<'static> {
    fn string_of(&self, vector: Vector2) -> String {
        format!("{:?} ({}{})", self.value_of(vector), self.prefix.label, self.suffix.label)
    }
}

impl From<suffix::Distance> for Unit<'static> {
    fn from(suffix: suffix::Distance) -> Self {
        use suffix::Distance::*;
        let prefix = Scale::from(1.);
        let suffix = match suffix {
            Standard => Scale::new("m", 1.),
            Astronomic => Scale::new("au", AU_PER_METER),
            Light => Scale::new("ls", LS_PER_METER),
        };
        Unit::new(prefix, suffix)
    }
}

impl From<suffix::Time> for Unit<'static> {
    fn from(suffix: suffix::Time) -> Self {
        use suffix::Time::*;
        use suffix::Calendar::*;
        let prefix = Scale::from(1.);
        let suffix = match suffix {
            Standard => Scale::new("s", 1.),
            Calendar => match suffix::Calendar::from(1.) {
                Year => Scale::new("years", YEAR_PER_SEC),
                Month => Scale::new("months", MONTH_PER_SEC),
                Week => Scale::new("weeks", WEEK_PER_SEC),
                Day => Scale::new("days", DAY_PER_SEC),
                Hour => Scale::new("hours", HOUR_PER_SEC),
                Minute => Scale::new("minutes", MIN_PER_SEC),
                Second => Scale::new("seconds", 1.),
            },
            Light => Scale::new("lm", LM_PER_SEC),
        };
        Unit::new(prefix, suffix)
    }
}

impl From<suffix::Mass> for Unit<'static> {
    fn from(suffix: suffix::Mass) -> Self {
        use suffix::Mass::*;
        let prefix = Scale::from(1e3);
        let suffix = match suffix {
            Standard => Scale::new("g", 1.),
            Tons => Scale::new("t", TONS_PER_KG),
        };
        Unit::new(prefix, suffix)
    }
}

#[derive(Copy, Clone)]
pub struct Duration {
    seconds: f64,
    minutes: i32,
    hours: i32,
    days: i32,
    weeks: i32,
    months: i32,
    years: i32,
}

impl Duration {
    fn new() -> Self {
        Duration {
            seconds: 0.0,
            minutes: 0,
            hours: 0,
            days: 0,
            weeks: 0,
            months: 0,
            years: 0,
        }
    }

    fn as_seconds(&self) -> f64 {
        let minutes = (self.minutes * 60) as f64;
        let hours = (self.hours * 3600) as f64;
        let days = (self.days * 86400) as f64;
        let weeks = (self.weeks * 604800) as f64;
        let months = (self.months * 2419200) as f64;
        let years = (self.years * 29030400) as f64;
        self.seconds + minutes + hours + days + weeks + months + years
    }

    fn set_seconds(&mut self, sec: f64) -> &mut Self {
        let years = (sec * YEAR_PER_SEC).floor() as i32;
        let months = (sec * MONTH_PER_SEC).floor() as i32;
        let weeks = (sec * WEEK_PER_SEC).floor() as i32;
        let days = (sec * DAY_PER_SEC).floor() as i32;
        let hours = (sec * HOUR_PER_SEC).floor() as i32;
        let minutes = (sec * MIN_PER_SEC).floor() as i32;
        self.seconds = sec - (minutes * 60) as f64;
        self.minutes = minutes - hours * 60;
        self.hours = hours - days * 24;
        self.days = days - weeks * 7;
        self.weeks = weeks - months * 4;
        self.months = months - years * 12;
        self
    }
}

impl From<f64> for Duration {
    fn from(sec: f64) -> Self {
        let mut duration = Duration::new();
        duration.set_seconds(sec);
        duration
    }
}

impl Debug for Duration {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use suffix::Calendar::*;
        let ms = ((self.seconds - self.seconds.floor()) * 1e3) as i32;
        let hms = format!("{:02}:{:02}:{:02.0}:{:3}", self.hours, self.minutes, self.seconds, ms);
        match suffix::Calendar::from(self.as_seconds()) {
            Hour | Second | Minute => write!(f, "{}", hms.as_str()),
            Day => write!(f, "{}d {}", self.days, hms.as_str()),
            Week => write!(f, "{}w {}d {}", self.weeks, self.days, hms.as_str()),
            Month => write!(f, "{}m {}w {}d {}", self.months, self.weeks, self.days, hms.as_str()),
            Year => write!(f, "{}y {}m {}w {}d {}", self.years, self.months, self.weeks, self.days, hms.as_str()),
        }
    }
}

impl Add<f64> for Duration {
    type Output = Duration;

    fn add(self, rhs: f64) -> Self::Output {
        Duration::from((self.as_seconds() + rhs).abs())
    }
}

impl AddAssign<f64> for Duration {
    fn add_assign(&mut self, rhs: f64) {
        self.set_seconds((self.as_seconds() + rhs).abs());
    }
}

impl Sub<f64> for Duration {
    type Output = Duration;

    fn sub(self, rhs: f64) -> Self::Output {
        Duration::from((self.as_seconds() - rhs).abs())
    }
}

impl SubAssign<f64> for Duration {
    fn sub_assign(&mut self, rhs: f64) {
        self.set_seconds((self.as_seconds() - rhs).abs());
    }
}
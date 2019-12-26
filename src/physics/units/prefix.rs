use crate::physics::units::consts::*;

pub enum Standard {
    Femto,
    Pico,
    Nano,
    Micro,
    Milli,
    Base,
    Kilo,
    Mega,
    Giga,
    Tera,
    Peta,
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

impl From<f64> for Standard {
    fn from(val: f64) -> Self {
        use Standard::*;
        match val.abs().log10().floor() as i32 {
            -13 | -14 | -15 => Femto,
            -10 | -11 | -12 => Pico,
            -7 | -8 | -9 => Nano,
            -4 | -5 | -6 => Micro,
            -1 | -2 | -3 => Milli,
            0 | 1 | 2 => Base,
            3 | 4 | 5 => Kilo,
            6 | 7 | 8 => Mega,
            9 | 10 | 11 => Giga,
            12 | 13 | 14 => Tera,
            15 | 16 | 17 => Tera,
            _ => Base,
        }
    }
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

use crate::physics::units::constants::*;

pub enum Standard {
    Base,
    Kilo,
    Mega,
    Giga,
    Tera,
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
            0 | 1 | 2 => Base,
            3 | 4 | 5 => Kilo,
            6 | 7 | 8 => Mega,
            9 | 10 | 11 => Giga,
            12 | 13 | 14 => Tera,
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

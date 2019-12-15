use std::fmt::{Debug, Error, Formatter};
use std::ops::{Add, AddAssign, Sub, SubAssign};

use super::constants::*;

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
        use super::prefix::{Calendar::*, Calendar};
        let ms = ((self.seconds - self.seconds.floor()) * 1e3) as i32;
        let hms = format!("{:02}:{:02}:{:02.0}:{:3}", self.hours, self.minutes, self.seconds, ms);
        match Calendar::from(self.as_seconds()) {
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
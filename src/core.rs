use std::cmp::{max, min};
use std::fmt::{Debug, Error, Formatter};

use piston::input::{Key, MouseButton};
use piston::window::Size;

use crate::common::*;
use crate::physics::dynamics::body::Frame;
use crate::physics::units::date::Duration;
use crate::toggle;

#[derive(Copy, Clone)]
pub struct Step {
    pub count: u32,
    pub total: Duration,
    pub simulated: Duration,
    pub frame: f64,
}

impl Step {
    pub fn new() -> Step {
        Step {
            count: 0,
            total: Duration::from(0.),
            simulated: Duration::from(0.),
            frame: 0.,
        }
    }

    pub fn update(&mut self, dt: f64, scale: f64) {
        self.frame = dt;
        self.total += dt;
        self.simulated += dt * scale;
        self.count = (self.count + 1) % std::u32::MAX;
    }
}

impl Debug for Step {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use super::physics::units::suffix::Time;
        use super::physics::units::prefix::Standard;
        use super::physics::units::*;
        let framerate = (1. / self.frame).floor() as u8;
        let mut time_unit = Unit::new(
            Scale::from(Standard::Base),
            Scale::from(Time::Standard),
        );
        time_unit.rescale(self.frame);
        write!(f, "dt: {}\nframerate: {} (fps)\ntotal: {:?}\nsimulated: {:?}",
               time_unit.string_of(self.frame), framerate, self.total, self.simulated)
    }
}

#[derive(Copy, Clone)]
pub struct Scale {
    pub time: f64,
    pub distance: f64,
}

impl Scale {
    pub fn new(time: f64, distance: f64) -> Scale {
        assert!(time > 0. && distance > 0.);
        Scale { time, distance }
    }

    pub fn unit() -> Scale {
        Scale { time: 1., distance: 1. }
    }

    pub fn increase_time(&mut self) {
        self.time *= 1.10;
    }

    pub fn decrease_time(&mut self) {
        self.time /= 1.10;
    }

    pub fn increase_distance(&mut self) {
        self.distance *= 1.10;
    }
    pub fn decrease_distance(&mut self) {
        self.distance /= 1.10;
    }

    pub fn update(&mut self, key: &Key) {
        match *key {
            Key::I => self.increase_distance(),
            Key::U => self.decrease_distance(),
            Key::Y => self.increase_time(),
            Key::T => self.decrease_time(),
            _ => (),
        };
    }
}

impl Debug for Scale {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use crate::physics::units::*;
        use crate::physics::units::suffix::{Distance, Time};
        let mut time_unit = Unit::from(Scale::from(Time::Calendar));
        let mut distance_unit = Unit::from(Scale::from(Distance::Standard));
        time_unit.prefix.rescale(prefix::Calendar::from(self.time));
        write!(f, "time: {} per (s)\ndistance: {} per (m)",
               time_unit.string_of(self.time),
               distance_unit.rescale(self.distance).string_of(self.distance),
        )
    }
}


#[derive(Copy, Clone, Debug)]
pub enum State {
    Move,
    Add,
    WaitDrop,
    CancelDrop,
    Reset,
}

impl State {
    pub fn next(&mut self, key: &Option<Key>, button: &Option<MouseButton>) {
        use State::*;

        if let Some(key) = key {
            match key {
                Key::Backspace => {
                    *self = Reset;
                    return;
                }
                _ => *self,
            };
        }

        *self = match self {
            Reset => Move,
            Add => WaitDrop,
            CancelDrop => Move,
            Move => {
                if let Some(button) = button {
                    match button {
                        MouseButton::Left => Add,
                        _ => *self,
                    }
                } else {
                    *self
                }
            }
            WaitDrop => {
                if let Some(button) = button {
                    match button {
                        MouseButton::Left => Move,
                        MouseButton::Right => CancelDrop,
                        _ => WaitDrop,
                    }
                } else {
                    *self
                }
            }
        };
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub size: Size,
    pub scale: Scale,
    pub frames_per_update: u32,
    pub updates_per_frame: u32,
}

impl Config {
    pub fn new(size: Size, scale: Scale, frames_per_update: u32, updates_per_frame: u32) -> Config {
        Config {
            size,
            scale,
            frames_per_update,
            updates_per_frame,
        }
    }

    pub fn default() -> Config {
        Config {
            size: Size::from([640., 640.]),
            scale: Scale::unit(),
            frames_per_update: 1,
            updates_per_frame: 1024,
        }
    }

    pub fn update(&mut self, key: &Key) {
        match *key {
            Key::P => self.increase_updates_per_frame(),
            Key::O => self.decrease_updates_per_frame(),
            _ => (),
        };
        self.scale.update(key);
    }

    fn increase_updates_per_frame(&mut self) {
        self.updates_per_frame = min(self.updates_per_frame << 1, std::u32::MAX);
    }

    fn decrease_updates_per_frame(&mut self) {
        self.updates_per_frame = max(self.updates_per_frame >> 1, std::u32::MIN + 1);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Status {
    pub direction: Direction,
    pub bounded: bool,
    pub translate: bool,
    pub trajectory: bool,
    pub pause: bool,
    pub state: State,
    pub frame: Frame,
}

impl Status {
    pub fn new(bounded: bool, translate: bool) -> Status {
        Status {
            direction: Direction::Hold,
            bounded,
            translate,
            trajectory: true,
            pause: true,
            state: State::Reset,
            frame: Frame::Zero,
        }
    }

    pub fn default() -> Status {
        Status::new(true, false)
    }

    pub fn update(&mut self, key: &Option<Key>, button: &Option<MouseButton>) {
        let hold = Direction::Hold;
        if let Some(key) = key {
            match *key {
                Key::B => toggle!(self.bounded),
                Key::T => toggle!(self.translate),
                Key::R => toggle!(self.trajectory),
                Key::Space => toggle!(self.pause),
                Key::K => self.frame.next(),
                Key::Left | Key::Up | Key::Down | Key::Right => self.direction = Direction::from(key),
                _ => ()
            };
        } else if self.direction != hold {
            self.direction = hold;
        }
        self.state.next(key, button);
    }
}
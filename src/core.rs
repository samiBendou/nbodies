use std::cmp::{max, min};
use std::fmt::{Debug, Error, Formatter};

use piston::input::{Key, MouseButton};
use piston::window::Size;

use crate::common::*;
use crate::toggle;

#[derive(Copy, Clone)]
pub struct Step {
    pub count: u32,
    pub total: f64,
    pub frame: f64,
}

impl Step {
    pub fn new() -> Step {
        Step { count: 0, total: 0., frame: 0. }
    }

    pub fn update(&mut self, dt: f64) {
        self.frame = dt;
        self.total += dt;
        self.count = (self.count + 1) % std::u32::MAX;
    }
}

impl Debug for Step {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let dt = self.frame * 1e3;
        let framerate = 1. / self.frame;
        write!(f, "dt: {:.4} (ms)\nframerate: {:.2} (fps)\ntotal time: {:.2} (s)", dt, framerate, self.total)
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
            _ => (),
        };
    }
}

impl Debug for Scale {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use crate::physics::units::prefix;
        let time_prefix = prefix::Scale::from(self.time);
        let distance_prefix = prefix::Scale::from(self.distance);
        write!(f, "time: {:.4} ({}s/s)\ndistance: {:.4} ({}m/m)",
               time_prefix.value_of(self.time), time_prefix.label,
               distance_prefix.value_of(self.distance), distance_prefix.label,
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

#[derive(Clone, Debug)]
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
}

impl Status {
    pub fn new(bounded: bool, translate: bool) -> Status {
        let direction = Direction::Hold;
        let state = State::Reset;
        Status {
            direction,
            bounded,
            translate,
            trajectory: true,
            pause: true,
            state,
        }
    }

    pub fn default() -> Status {
        Status::new(true, false)
    }

    pub fn update(&mut self, key: &Option<Key>, button: &Option<MouseButton>) {
        if let Some(key) = key {
            match *key {
                Key::B => toggle!(self.bounded),
                Key::T => toggle!(self.translate),
                Key::R => toggle!(self.trajectory),
                Key::Space => toggle!(self.pause),
                _ => ()
            }
            self.direction = Direction::from(key);
        } else {
            self.direction = Direction::Hold;
        }
        self.state.next(key, button);
    }
}
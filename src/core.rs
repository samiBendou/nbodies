use std::cmp::{max, min};
use std::fmt::{Debug, Error, Formatter};
use std::time::SystemTime;

use piston::input::{Key, MouseButton};
use piston::window::Size;

use crate::common::*;
use crate::physics::units::{Rescale, Unit};
use crate::physics::units::date::Duration;
use crate::toggle;

#[derive(Clone)]
pub struct Step {
    pub count: u32,
    pub total: Duration,
    pub simulated: Duration,
    pub frame: f64,
    pub system: f64,
    time: SystemTime,
    frame_unit: Unit,
}

impl Step {
    pub fn new() -> Step {
        use crate::physics::units;
        use crate::physics::units::suffix::Time;
        use crate::physics::units::prefix::Standard;
        Step {
            count: 0,
            total: Duration::from(0.),
            simulated: Duration::from(0.),
            frame: 0.,
            system: 0.,
            time: SystemTime::now(),
            frame_unit: Unit::new(
                units::Scale::from(Standard::Base),
                units::Scale::from(Time::Second),
            ),
        }
    }

    pub fn update(&mut self, dt: f64, scale: f64) {
        use crate::physics::units::*;
        let time = SystemTime::now();
        self.system = time.duration_since(self.time).unwrap().as_secs_f64();
        self.time = time;
        self.frame = dt;
        self.total += dt;
        self.simulated += dt * scale;
        self.count = (self.count + 1) % std::u32::MAX;
        self.frame_unit.rescale(&self.frame);
    }
}

impl Debug for Step {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use crate::physics::units::*;
        let framerate = (1. / self.frame).floor() as u8;
        let framerate_system = (1. / self.system).floor() as u8;
        write!(f,
               "\
dt: {} framerate: {} (fps)\n\
(system) dt: {} framerate: {} (fps)\n\
total: {:?}\n\
simulated: {:?}",
               self.frame_unit.string_of(&self.frame),
               framerate,
               self.frame_unit.string_of(&self.system),
               framerate_system,
               self.total,
               self.simulated
        )
    }
}

pub struct Scale {
    pub time: f64,
    pub distance: f64,
    pub time_unit: Unit,
    pub distance_unit: Unit,
}

impl Scale {
    pub fn new(time: f64, distance: f64) -> Scale {
        use crate::physics::units;
        use units::suffix::{Distance, Time};
        assert!(time > 0. && distance > 0.);
        Scale {
            time,
            distance,
            time_unit: Unit::from(units::Scale::from(Time::Second)),
            distance_unit: Unit::from(units::Scale::from(Distance::Pixel)),
        }
    }

    pub fn unit() -> Scale {
        Scale::new(1., 1.)
    }

    pub fn increase_time(&mut self) {
        self.time *= 2.;
    }

    pub fn decrease_time(&mut self) {
        self.time /= 2.;
    }

    pub fn increase_distance(&mut self) {
        self.distance *= 2.;
    }
    pub fn decrease_distance(&mut self) {
        self.distance /= 2.;
    }

    pub fn update(&mut self, key: &Key) {
        if *key == KEY_INCREASE_DISTANCE {
            self.increase_distance();
        } else if *key == KEY_DECREASE_DISTANCE {
            self.decrease_distance();
        } else if *key == KEY_INCREASE_TIME {
            self.increase_time();
        } else if *key == KEY_DECREASE_TIME {
            self.decrease_time();
        }
        self.time_unit.rescale(&self.time);
        self.distance_unit.rescale(&self.distance);
    }
}

impl Debug for Scale {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use crate::physics::units::*;
        write!(f, "time: {} per (second)\ndistance: {} per (meter)",
               self.time_unit.string_of(&self.time),
               self.distance_unit.string_of(&self.distance),
        )
    }
}


#[derive(Copy, Clone, Debug, PartialEq)]
pub enum State {
    Move,
    Add,
    WaitDrop,
    WaitSpeed,
    CancelDrop,
    Reset,
}

impl State {
    pub fn next(&mut self, key: &Key, button: &MouseButton) {
        use State::*;

        *self = match self {
            Reset => Move,
            Add => WaitDrop,
            CancelDrop => Move,
            Move => if *button == MOUSE_MOVE_ADD {
                Add
            } else {
                *self
            },
            WaitDrop => if *button == MOUSE_WAIT_DROP_DO {
                WaitSpeed
            } else if *button == MOUSE_WAIT_DROP_CANCEL {
                CancelDrop
            } else {
                *self
            }
            WaitSpeed => if *button == MOUSE_WAIT_DROP_DO {
                Move
            } else if *button == MOUSE_WAIT_DROP_CANCEL {
                WaitDrop
            } else {
                *self
            }
        };

        if *key == KEY_RESET {
            *self = Reset;
        }
    }
}

pub struct Config {
    pub size: Size,
    pub scale: Scale,
    pub undersampling: u32,
    pub oversampling: u32,
}

impl Config {
    pub fn new(size: Size, scale: Scale, undersampling: u32, oversampling: u32) -> Config {
        Config {
            size,
            scale,
            undersampling,
            oversampling,
        }
    }

    pub fn default() -> Config {
        Config {
            size: Size::from([640., 640.]),
            scale: Scale::unit(),
            undersampling: 1,
            oversampling: 1024,
        }
    }

    pub fn update(&mut self, key: &Key) {
        if *key == KEY_INCREASE_OVERSAMPLING {
            self.increase_oversampling();
        } else if *key == KEY_DECREASE_OVERSAMPLING {
            self.decrease_oversampling();
        }
        self.scale.update(key);
    }

    fn increase_oversampling(&mut self) {
        self.oversampling = min(self.oversampling << 1, std::u32::MAX);
    }

    fn decrease_oversampling(&mut self) {
        self.oversampling = max(self.oversampling >> 1, std::u32::MIN + 1);
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
        Status {
            direction: Direction::Hold,
            bounded,
            translate,
            trajectory: true,
            pause: true,
            state: State::Reset,
        }
    }

    pub fn default() -> Status {
        Status::new(false, false)
    }

    pub fn update(&mut self, key: &Option<Key>, button: &Option<MouseButton>) {
        match key {
            None => {
                self.direction = HOLD;
                match button {
                    None => self.state.next(&KEY_UNKNOWN, &BUTTON_UNKNOWN),
                    Some(button) => self.state.next(&KEY_UNKNOWN, button),
                };
            },
            Some(key) => {
                if *key == KEY_TOGGLE_BOUNDED {
                    toggle!(self.bounded);
                } else if *key == KEY_TOGGLE_TRANSLATE {
                    toggle!(self.translate);
                } else if *key == KEY_TOGGLE_TRAJECTORY {
                    toggle!(self.trajectory);
                } else if *key == KEY_TOGGLE_PAUSE {
                    toggle!(self.pause);
                } else {
                    self.direction = Direction::from(key);
                }
                match button {
                    None => self.state.next(key, &BUTTON_UNKNOWN),
                    Some(button) => self.state.next(key, button),
                };
            },
        };
    }
}
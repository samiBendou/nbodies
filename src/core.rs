use std::cmp::{max, min};
use std::fmt::{Debug, Error, Formatter};
use std::time::SystemTime;

use piston::input::{Key, MouseButton};
use piston::input::Input::Button;
use piston::window::Size;

use crate::common::*;
use crate::physics::dynamics::body::Frame;
use crate::physics::units::date::Duration;
use crate::physics::units::Unit;
use crate::toggle;

static HOLD: Direction = Direction::Hold;
static BUTTON_UNKNOWN: MouseButton = MouseButton::Unknown;
static KEY_UNKNOWN: Key = Key::Unknown;

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
        self.frame_unit.rescale(self.frame);
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
               self.frame_unit.string_of(self.frame),
               framerate,
               self.frame_unit.string_of(self.system),
               framerate_system,
               self.total,
               self.simulated
        )
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
        if *key == KEY_INCREASE_DISTANCE {
            self.increase_distance();
        } else if *key == KEY_DECREASE_DISTANCE {
            self.decrease_distance();
        } else if *key == KEY_INCREASE_TIME {
            self.increase_time();
        } else if *key == KEY_DECREASE_TIME {
            self.decrease_time();
        }
    }
}

impl Debug for Scale {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use crate::physics::units::*;
        use crate::physics::units::suffix::{Distance, Time};
        let mut time_unit = Unit::from(Scale::from(Time::Calendar));
        let mut distance_unit = Unit::from(Scale::from(Distance::Meter));
        time_unit.prefix.rescale(prefix::Calendar::from(self.time));
        write!(f, "time: {} per (second)\ndistance: {} per (px)",
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
            WaitDrop => if *button == MOUSE_WAIT_DROP_MOVE {
                Move
            } else if *button == MOUSE_WAIT_DROP_CANCEL {
                CancelDrop
            } else {
                *self
            }
        };

        if *key == KEY_RESET {
            *self = Reset;
        }
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
        if *key == KEY_INCREASE_UP_FRAME {
            self.increase_updates_per_frame();
        } else if *key == KEY_DECREASE_UP_FRAME {
            self.decrease_updates_per_frame();
        }
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
        Status::new(true, false)
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
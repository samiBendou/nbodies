use std::cmp::{max, min};
use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use std::time::SystemTime;

use getopts::Options;
use physics::geometry::common::transforms::Rotation3;
use physics::geometry::matrix::Matrix3;
use physics::units::{Rescale, Unit};
use physics::units::date::Duration;
use piston::input::{Key, MouseButton};
use piston::window::Size;

use crate::common::*;

#[derive(Clone)]
pub struct Step {
    pub count: u32,
    pub total: Duration,
    pub simulated: Duration,
    pub frame: Average,
    pub system: Average,
    time: SystemTime,
    frame_unit: Unit,
}

impl Step {
    pub fn new() -> Step {
        use physics::units;
        use physics::units::suffix::Time;
        use physics::units::prefix::Standard;
        Step {
            count: 0,
            total: Duration::from(0.),
            simulated: Duration::from(0.),
            frame: Average::new(),
            system: Average::new(),
            time: SystemTime::now(),
            frame_unit: Unit::new(
                units::Scale::from(Standard::Base),
                units::Scale::from(Time::Second),
            ),
        }
    }

    pub fn update(&mut self, dt: f64, scale: f64) {
        use physics::units::*;
        let time = SystemTime::now();
        self.system.push(time.duration_since(self.time).unwrap().as_secs_f64());
        self.time = time;
        self.frame.push(dt);
        self.total += dt;
        self.simulated += dt * scale;
        self.count = (self.count + 1) % std::u32::MAX;
        self.frame_unit.rescale(&self.frame.value());
    }
}

impl Debug for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        use physics::units::*;
        let frame = self.frame.value();
        let system = self.system.value();
        let framerate = (1. / frame).floor() as u8;
        let framerate_system = (1. / system).floor() as u8;
        write!(f,
               "\
dt: {} framerate: {} (fps)\n\
(system) dt: {} framerate: {} (fps)\n\
total: {:?}\n\
simulated: {:?}",
               self.frame_unit.string_of(&frame),
               framerate,
               self.frame_unit.string_of(&system),
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
        use physics::units;
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        use physics::units::*;
        write!(f, "time: {} per (second)\ndistance: {} per (meter)",
               self.time_unit.string_of(&self.time),
               self.distance_unit.string_of(&self.distance),
        )
    }
}


#[derive(Copy, Clone, Debug, PartialEq)]
pub enum State {
    Move,
    Translate,
    Add,
    Remove,
    WaitDrop,
    WaitSpeed,
    CancelDrop,
    Reset,
}

impl State {
    pub fn next(&mut self, key: &Key, button: &MouseButton) {
        use State::*;

        if *key == KEY_RESET {
            *self = Reset;
            return;
        }

        *self = match self {
            Reset => Move,
            Add => WaitDrop,
            Remove => Move,
            CancelDrop => Move,
            Move => if *button == MOUSE_MOVE_ADD {
                Add
            } else if *button == MOUSE_MOVE_REMOVE {
                Remove
            } else if *key == KEY_TOGGLE_TRANSLATE {
                Translate
            } else {
                *self
            },
            Translate => if *key == KEY_TOGGLE_TRANSLATE {
                Move
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
    }
}

pub struct Arguments {
    pub path: Option<String>,
    pub scale: Scale,
    pub size: Option<Size>,
}

impl Arguments {
    /*
    fn print_usage(program: &str, opts: Options) {
        let brief = format!("Usage: {} FILE [options]", program);
        print!("{}", opts.usage(&brief));
    }
    */

    pub fn new(args: Vec<String>) -> Result<Arguments, Box<dyn Error>> {
        let mut opts = Options::new();
        opts.optopt("o", "orbital", "Loads an orbital cluster from file", "FILEPATH");
        opts.optopt("d", "distance", "Sets the distance scale in px/meters", "NUMBER");
        opts.optopt("t", "time", "Sets the time scale in secs/real sec", "NUMBER");
        let matches = opts.parse(&args[1..])?;
        let path = matches.opt_str("o");
        let mut scale: Scale = Scale::unit();
        if let Some(distance_str) = matches.opt_str("d") {
            scale.distance = distance_str.parse()?;
        }
        if let Some(time_str) = matches.opt_str("t") {
            scale.time = time_str.parse()?;
        }
        let size: Option<Size> = None;
        Ok(Arguments { path, scale, size })
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

impl From<Arguments> for Config {
    fn from(args: Arguments) -> Self {
        let mut ret = Config::default();
        ret.scale = args.scale;
        if let Some(size) = args.size {
            ret.size = size;
        }
        ret
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Orientation {
    pub rotation_x: Matrix3,
    pub rotation_y: Matrix3,
    pub rotation_z: Matrix3,
    increment_x: Matrix3,
    increment_y: Matrix3,
    increment_z: Matrix3,
    decrement_x: Matrix3,
    decrement_y: Matrix3,
    decrement_z: Matrix3,
}

impl Orientation {
    pub fn new(angle_x: f64, angle_y: f64, angle_z: f64) -> Orientation {
        Orientation {
            rotation_x: Matrix3::from_rotation_x(angle_x),
            rotation_y: Matrix3::from_rotation_y(angle_y),
            rotation_z: Matrix3::from_rotation_z(angle_z),
            increment_x: Matrix3::from_rotation_x(DEFAULT_ANGLE_INCREMENT),
            increment_y: Matrix3::from_rotation_y(DEFAULT_ANGLE_INCREMENT),
            increment_z: Matrix3::from_rotation_z(DEFAULT_ANGLE_INCREMENT),
            decrement_x: Matrix3::from_rotation_x(-DEFAULT_ANGLE_INCREMENT),
            decrement_y: Matrix3::from_rotation_y(-DEFAULT_ANGLE_INCREMENT),
            decrement_z: Matrix3::from_rotation_z(-DEFAULT_ANGLE_INCREMENT),
        }
    }

    pub fn zeros() -> Self {
        Orientation::new(0., 0., 0.)
    }

    pub fn increment_x(&mut self) -> &mut Self {
        self.rotation_x *= self.increment_x;
        self
    }

    pub fn increment_y(&mut self) -> &mut Self {
        self.rotation_y *= self.increment_y;
        self
    }

    pub fn increment_z(&mut self) -> &mut Self {
        self.rotation_z *= self.increment_z;
        self
    }
    pub fn decrement_x(&mut self) -> &mut Self {
        self.rotation_x *= self.decrement_x;
        self
    }

    pub fn decrement_y(&mut self) -> &mut Self {
        self.rotation_y *= self.decrement_y;
        self
    }

    pub fn decrement_z(&mut self) -> &mut Self {
        self.rotation_z *= self.decrement_z;
        self
    }

    pub fn rotation(&self) -> Matrix3 {
        self.rotation_z * self.rotation_y * self.rotation_x
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Status {
    pub direction: Direction,
    pub trajectory: bool,
    pub pause: bool,
    pub reset_circles: bool,
    pub update_transform: bool,
    pub state: State,
    pub orientation: Orientation,
}

impl Status {
    pub fn new() -> Status {
        Status {
            direction: Direction::Hold,
            trajectory: true,
            pause: true,
            reset_circles: true,
            update_transform: true,
            state: State::Reset,
            orientation: Orientation::new(0., 0., 0.),
        }
    }

    pub fn is_waiting_to_add(&self) -> bool {
        self.state == State::WaitSpeed || self.state == State::WaitDrop
    }

    pub fn update(&mut self, key: &Option<Key>, button: &Option<MouseButton>) {
        match key {
            None => {
                self.direction = HOLD;
                match button {
                    None => self.state.next(&KEY_UNKNOWN, &BUTTON_UNKNOWN),
                    Some(button) => self.state.next(&KEY_UNKNOWN, button),
                };
            }
            Some(key) => {
                if *key == KEY_TOGGLE_TRAJECTORY {
                    self.trajectory = !self.trajectory;
                } else if *key == KEY_TOGGLE_PAUSE {
                    self.pause = !self.pause;
                } else if *key == KEY_ROTATION_DOWN {
                    self.orientation.increment_x();
                    self.reset_circles = true;
                    self.update_transform = true;
                } else if *key == KEY_ROTATION_UP {
                    self.orientation.decrement_x();
                    self.reset_circles = true;
                    self.update_transform = true;
                } else if *key == KEY_ROTATION_LEFT {
                    self.orientation.increment_z();
                    self.reset_circles = true;
                    self.update_transform = true;
                } else if *key == KEY_ROTATION_RIGHT {
                    self.orientation.decrement_z();
                    self.reset_circles = true;
                    self.update_transform = true;
                } else if
                *key == KEY_INCREASE_CURRENT_INDEX ||
                    *key == KEY_DECREASE_CURRENT_INDEX ||
                    *key == KEY_NEXT_FRAME_STATE ||
                    *key == KEY_INCREASE_DISTANCE ||
                    *key == KEY_DECREASE_DISTANCE {
                    self.reset_circles = true;
                    self.update_transform = true
                } else {
                    self.direction = Direction::from(key);
                }
                match button {
                    None => self.state.next(key, &BUTTON_UNKNOWN),
                    Some(button) => self.state.next(key, button),
                };
            }
        };
    }

    pub fn clear(&mut self) {
        self.state.next(&KEY_UNKNOWN, &BUTTON_UNKNOWN);
        self.direction = Direction::from(&KEY_UNKNOWN);
        self.reset_circles = false;
        self.update_transform = false;
    }
}
use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use std::time::SystemTime;

use physics::geometry::common::coordinates::Cartesian2;
use physics::geometry::common::Initializer;
use physics::geometry::common::transforms::Rotation3;
use physics::geometry::matrix::Matrix3;
use physics::geometry::vector::*;
use physics::units::{Rescale, Unit};
use physics::units::date::Duration;
use piston::input::{Key, MouseButton};

pub static KEY_RESET: Key = Key::Backspace;

pub static KEY_TOGGLE_TRANSLATE: Key = Key::J;
pub static KEY_TOGGLE_TRAJECTORY: Key = Key::R;
pub static KEY_TOGGLE_PAUSE: Key = Key::Space;

pub static KEY_DIRECTION_UP: Key = Key::W;
pub static KEY_DIRECTION_DOWN: Key = Key::S;
pub static KEY_DIRECTION_LEFT: Key = Key::A;
pub static KEY_DIRECTION_RIGHT: Key = Key::D;

pub static KEY_ROTATION_UP: Key = Key::Up;
pub static KEY_ROTATION_DOWN: Key = Key::Down;
pub static KEY_ROTATION_LEFT: Key = Key::Left;
pub static KEY_ROTATION_RIGHT: Key = Key::Right;

pub static KEY_INCREASE_OVERSAMPLING: Key = Key::P;
pub static KEY_DECREASE_OVERSAMPLING: Key = Key::O;

pub static KEY_INCREASE_DISTANCE: Key = Key::I;
pub static KEY_DECREASE_DISTANCE: Key = Key::U;

pub static KEY_INCREASE_TIME: Key = Key::Comma;
pub static KEY_DECREASE_TIME: Key = Key::M;

pub static KEY_INCREASE_CURRENT_INDEX: Key = Key::V;
pub static KEY_DECREASE_CURRENT_INDEX: Key = Key::C;

pub static KEY_NEXT_LOGGER_STATE: Key = Key::L;
pub static KEY_NEXT_FRAME_STATE: Key = Key::K;
pub static KEY_NEXT_METHOD_STATE: Key = Key::Semicolon;

pub static MOUSE_MOVE_ADD: MouseButton = MouseButton::Left;
pub static MOUSE_MOVE_REMOVE: MouseButton = MouseButton::Right;
pub static MOUSE_WAIT_DROP_DO: MouseButton = MouseButton::Left;
pub static MOUSE_WAIT_DROP_CANCEL: MouseButton = MouseButton::Right;

pub static HOLD: Direction = Direction::Hold;
pub static BUTTON_UNKNOWN: MouseButton = MouseButton::Unknown;
pub static KEY_UNKNOWN: Key = Key::Unknown;

pub static DEFAULT_ANGLE_INCREMENT: f64 = std::f64::consts::FRAC_PI_8 / 6.;
pub const SPEED_SCALING_FACTOR: f64 = 5e-7;
pub const TRANSLATION_SCALING_FACTOR: f64 = 100.;

pub const BLACK: [f32; 4] = [0., 0., 0., 1.];
pub const WHITE: [f32; 4] = [1., 1., 1., 1.];
pub const RED: [f32; 4] = [1., 0., 0., 1.];
pub const GREEN: [f32; 4] = [0., 1., 0., 1.];
pub const BLUE: [f32; 4] = [0., 0., 1., 1.];

#[derive(Copy, Clone)]
pub struct Average {
    pub count: usize,
    pub values: [f64; 60],
}

impl Average {
    pub fn new() -> Average {
        Average { count: 0, values: [0.; 60] }
    }

    pub fn push(&mut self, val: f64) -> &mut Self {
        self.values[self.count] = val;
        self.count = (self.count + 1) % 60;
        self
    }

    pub fn value(&self) -> f64 {
        let mut ret = 0.;
        for val in self.values.iter() {
            ret += *val;
        }
        ret / 60.
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Input {
    pub key: Option<Key>,
    pub button: Option<MouseButton>,
    pub cursor: [f64; 2],
}

impl Input {
    pub fn new() -> Input {
        Input {
            key: Option::None,
            button: Option::None,
            cursor: [0., 0.],
        }
    }
}


#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Direction {
    Left = -1,
    Right = 1,
    Up = -2,
    Down = 2,
    Hold = 0,
}

impl Direction {
    pub fn opposite(&self, other: &Direction) -> bool {
        let self_val = *self as i8;
        let other_val = *other as i8;

        self_val == -other_val
    }

    pub fn from(key: &Key) -> Direction {
        use super::*;
        if *key == KEY_DIRECTION_LEFT {
            Direction::Left
        } else if *key == KEY_DIRECTION_RIGHT {
            Direction::Right
        } else if *key == KEY_DIRECTION_UP {
            Direction::Up
        } else if *key == KEY_DIRECTION_DOWN {
            Direction::Down
        } else {
            Direction::Hold
        }
    }

    pub fn as_vector(&self) -> Vector3 {
        match *self {
            Direction::Left => Vector3::unit_neg_x(),
            Direction::Right => Vector3::unit_x(),
            Direction::Up => Vector3::unit_y(),
            Direction::Down => Vector3::unit_neg_y(),
            Direction::Hold => Vector3::zeros()
        }
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

    pub fn push(&mut self, dt: f64, scale: f64) {
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

    pub fn rescale(&mut self) {
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



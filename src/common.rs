use std::fmt::Debug;
use std::time::SystemTime;

use physics::geometry::common::coordinates::Cartesian2;
use physics::geometry::common::Initializer;
use physics::geometry::common::transforms::Rotation3;
use physics::geometry::matrix::{Algebra, Matrix3};
use physics::geometry::vector::*;
use physics::units::date::Duration;
use piston::input::{Key, MouseButton};
use serde::export::fmt::{Error, Formatter};

use crate::keys::*;

pub static HOLD: Direction = Direction::Hold;

pub static DEFAULT_ANGLE_INCREMENT: f64 = std::f64::consts::FRAC_PI_8 / 6.;
pub const SPEED_SCALING_FACTOR: f64 = 5e-7;
pub const TRANSLATION_SCALING_FACTOR: f64 = 100.;

pub const DEFAULT_WINDOW_SIZE: [f64; 2] = [640., 640.];
pub const DEFAULT_OVERSAMPLING: u32 = 1024;

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

impl Debug for Average {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.value())
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


#[derive(Clone, Copy)]
pub struct Orientation {
    rotation: Matrix3,
    rotation_x: Matrix3,
    rotation_y: Matrix3,
    rotation_z: Matrix3,
    increment_x: Matrix3,
    increment_y: Matrix3,
    increment_z: Matrix3,
    decrement_x: Matrix3,
    decrement_y: Matrix3,
    decrement_z: Matrix3,
}

impl Orientation {
    pub fn new(angle_x: f64, angle_y: f64, angle_z: f64) -> Orientation {
        let mut ret = Orientation {
            rotation: Matrix3::eye(),
            rotation_x: Matrix3::from_rotation_x(angle_x),
            rotation_y: Matrix3::from_rotation_y(angle_y),
            rotation_z: Matrix3::from_rotation_z(angle_z),
            increment_x: Matrix3::from_rotation_x(DEFAULT_ANGLE_INCREMENT),
            increment_y: Matrix3::from_rotation_y(DEFAULT_ANGLE_INCREMENT),
            increment_z: Matrix3::from_rotation_z(DEFAULT_ANGLE_INCREMENT),
            decrement_x: Matrix3::from_rotation_x(-DEFAULT_ANGLE_INCREMENT),
            decrement_y: Matrix3::from_rotation_y(-DEFAULT_ANGLE_INCREMENT),
            decrement_z: Matrix3::from_rotation_z(-DEFAULT_ANGLE_INCREMENT),
        };
        ret.update_rotation();
        ret
    }

    pub fn zeros() -> Self {
        Orientation::new(0., 0., 0.)
    }

    pub fn increment_x(&mut self) -> &mut Self {
        self.rotation_x *= self.increment_x;
        self.update_rotation();
        self
    }

    pub fn increment_y(&mut self) -> &mut Self {
        self.rotation_y *= self.increment_y;
        self.update_rotation();
        self
    }

    pub fn increment_z(&mut self) -> &mut Self {
        self.rotation_z *= self.increment_z;
        self.update_rotation();
        self
    }
    pub fn decrement_x(&mut self) -> &mut Self {
        self.rotation_x *= self.decrement_x;
        self.update_rotation();
        self
    }

    pub fn decrement_y(&mut self) -> &mut Self {
        self.rotation_y *= self.decrement_y;
        self.update_rotation();
        self
    }

    pub fn decrement_z(&mut self) -> &mut Self {
        self.rotation_z *= self.decrement_z;
        self.update_rotation();
        self
    }

    pub fn rotation(&self) -> Matrix3 {
        self.rotation
    }

    fn update_rotation(&mut self) -> &mut Self {
        self.rotation = self.rotation_z * self.rotation_y * self.rotation_x;
        self
    }
}

impl Debug for Orientation {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:?}", self.rotation())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Step {
    pub count: u32,
    pub total: Duration,
    pub simulated: Duration,
    pub frame: Average,
    pub system: Average,
    time: SystemTime,
}

impl Step {
    pub fn new() -> Step {
        Step {
            count: 0,
            total: Duration::from(0.),
            simulated: Duration::from(0.),
            frame: Average::new(),
            system: Average::new(),
            time: SystemTime::now(),
        }
    }

    pub fn push(&mut self, dt: f64, scale: f64) {
        let time = SystemTime::now();
        self.system.push(time.duration_since(self.time).unwrap().as_secs_f64());
        self.time = time;
        self.frame.push(dt);
        self.total += dt;
        self.simulated += dt * scale;
        self.count = (self.count + 1) % std::u32::MAX;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Scale {
    pub time: f64,
    pub distance: f64,
}

impl Scale {
    pub fn new(time: f64, distance: f64) -> Scale {
        Scale { time, distance }
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
}


use std::fmt::Debug;

use physics::geometry::common::coordinates::Cartesian2;
use physics::geometry::common::Initializer;
use physics::geometry::vector::*;
use piston::input::{Key, MouseButton};

pub static KEY_TOGGLE_BOUNDED: Key = Key::B;
pub static KEY_TOGGLE_TRANSLATE: Key = Key::J;
pub static KEY_TOGGLE_TRAJECTORY: Key = Key::R;
pub static KEY_TOGGLE_PAUSE: Key = Key::Space;
pub static KEY_RESET: Key = Key::Backspace;

pub static KEY_ROTATION_UP: Key = Key::Up;
pub static KEY_ROTATION_DOWN: Key = Key::Down;
pub static KEY_ROTATION_LEFT: Key = Key::Left;
pub static KEY_ROTATION_RIGHT: Key = Key::Right;

pub static KEY_INCREASE_OVERSAMPLING: Key = Key::P;
pub static KEY_DECREASE_OVERSAMPLING: Key = Key::O;
pub static KEY_INCREASE_DISTANCE: Key = Key::I;
pub static KEY_DECREASE_DISTANCE: Key = Key::U;
pub static KEY_INCREASE_TIME: Key = Key::Y;
pub static KEY_DECREASE_TIME: Key = Key::T;
pub static KEY_INCREASE_CURRENT_INDEX: Key = Key::Z;
pub static KEY_DECREASE_CURRENT_INDEX: Key = Key::X;
pub static KEY_NEXT_LOGGER_STATE: Key = Key::L;
pub static KEY_NEXT_FRAME_STATE: Key = Key::K;
pub static KEY_NEXT_METHOD_STATE: Key = Key::M;

pub static MOUSE_MOVE_ADD: MouseButton = MouseButton::Left;
pub static MOUSE_MOVE_REMOVE: MouseButton = MouseButton::Right;
pub static MOUSE_WAIT_DROP_DO: MouseButton = MouseButton::Left;
pub static MOUSE_WAIT_DROP_CANCEL: MouseButton = MouseButton::Right;

pub static HOLD: Direction = Direction::Hold;
pub static BUTTON_UNKNOWN: MouseButton = MouseButton::Unknown;
pub static KEY_UNKNOWN: Key = Key::Unknown;

pub static DEFAULT_ANGLE_INCREMENT: f64 = std::f64::consts::FRAC_PI_8 / 6.;
pub const SPEED_SCALING_FACTOR: f64 = 5e-7;

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
        if *key == KEY_ROTATION_LEFT {
            Direction::Left
        } else if *key == KEY_ROTATION_RIGHT {
            Direction::Right
        } else if *key == KEY_ROTATION_UP {
            Direction::Up
        } else if *key == KEY_ROTATION_DOWN {
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
            Direction::Down => Vector3::unit_neg_x(),
            Direction::Hold => Vector3::zeros()
        }
    }
}



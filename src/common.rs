use std::fmt::Debug;

use piston::input::{Key, MouseButton};

use crate::physics::vector::*;

pub static KEY_TOGGLE_BOUNDED: Key = Key::B;
pub static KEY_TOGGLE_TRANSLATE: Key = Key::J;
pub static KEY_TOGGLE_TRAJECTORY: Key = Key::R;
pub static KEY_TOGGLE_PAUSE: Key = Key::Space;
pub static KEY_RESET: Key = Key::Backspace;

pub static KEY_MOVE_UP: Key = Key::Up;
pub static KEY_MOVE_DOWN: Key = Key::Down;
pub static KEY_MOVE_LEFT: Key = Key::Left;
pub static KEY_MOVE_RIGHT: Key = Key::Right;

pub static KEY_INCREASE_UP_FRAME: Key = Key::P;
pub static KEY_DECREASE_UP_FRAME: Key = Key::O;
pub static KEY_INCREASE_DISTANCE: Key = Key::I;
pub static KEY_DECREASE_DISTANCE: Key = Key::U;
pub static KEY_INCREASE_TIME: Key = Key::Y;
pub static KEY_DECREASE_TIME: Key = Key::T;
pub static KEY_INCREASE_CURRENT_INDEX: Key = Key::Z;
pub static KEY_DECREASE_CURRENT_INDEX: Key = Key::X;
pub static KEY_NEXT_LOGGER_STATE: Key = Key::L;
pub static KEY_NEXT_FRAME_STATE: Key = Key::K;

pub static MOUSE_MOVE_ADD: MouseButton = MouseButton::Left;
pub static MOUSE_WAIT_DROP_DO: MouseButton = MouseButton::Left;
pub static MOUSE_WAIT_DROP_CANCEL: MouseButton = MouseButton::Right;

pub static HOLD: Direction = Direction::Hold;
pub static BUTTON_UNKNOWN: MouseButton = MouseButton::Unknown;
pub static KEY_UNKNOWN: Key = Key::Unknown;

#[macro_export]
macro_rules! toggle {
    ($boolean: expr) => {
    $boolean = !$boolean;
    };
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
        if *key == KEY_MOVE_LEFT {
            Direction::Left
        } else if *key == KEY_MOVE_RIGHT {
            Direction::Right
        } else if *key == KEY_MOVE_UP {
            Direction::Up
        } else if *key == KEY_MOVE_DOWN {
            Direction::Down
        } else {
            Direction::Hold
        }
    }

    pub fn as_vector(&self) -> Vector2 {
        match *self {
            Direction::Left => N_EX,
            Direction::Right => EX,
            Direction::Up => EY,
            Direction::Down => N_EY,
            Direction::Hold => ZERO
        }
    }
}



use std::fmt::Debug;

use piston::input::{Key, MouseButton};

use crate::physics::vector::Vector2;

static EX: Vector2 = Vector2 { x: 1., y: 0. };
static N_EX: Vector2 = Vector2 { x: -1., y: 0. };
static EY: Vector2 = Vector2 { x: 0., y: 1. };
static N_EY: Vector2 = Vector2 { x: 0., y: -1. };
static ZERO: Vector2 = Vector2 { x: 0., y: 0. };

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
        match *key {
            Key::Left => Direction::Left,
            Key::Right => Direction::Right,
            Key::Up => Direction::Up,
            Key::Down => Direction::Down,
            _ => Direction::Hold,
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



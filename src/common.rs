use std::fmt::Debug;

use piston::input::{Key, MouseButton};

use crate::physics::vector::Vector2;

#[macro_use]
#[macro_export]
macro_rules! toggle {
    ($boolean: expr) => {
    $boolean = !$boolean;
    };
}
#[macro_export]
macro_rules! to_centered {
    ($position: expr, $size: path) => {
    $position[0] = $position[0] - $size.width / 2.;
    $position[1] = $size.height / 2. - $position[1];
    };
}
#[macro_export]
#[macro_use]
macro_rules! to_left_up {
    ($position: expr, $size: path) => {
    $position[0] = $position[0] + $size.width / 2.;
    $position[1] = $size.height / 2. - $position[1];
    };
}
#[macro_export]
macro_rules! offset_or_position {
    ($position: expr, $size: path) => {
        if let Some(size) = $size {
            crate::to_left_up!($position, size);
        }
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
            Direction::Left => -Vector2::ex(),
            Direction::Right => Vector2::ex(),
            Direction::Up => Vector2::ey(),
            Direction::Down => -Vector2::ey(),
            Direction::Hold => Vector2::zeros()
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Color {
    Red,
    Green,
    Blue,
}

impl Color {
    pub fn rgba_array(&self) -> [f32; 4] {
        match self {
            Color::Red => [1.0, 0.0, 0.0, 1.0],
            Color::Green => [0.0, 1.0, 0.0, 1.0],
            Color::Blue => [0.0, 0.0, 1.0, 1.0],
        }
    }
}




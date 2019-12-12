use std::fmt::{Debug, Error, Formatter};

use piston::input::Key;
use piston::window::Size;

use crate::vector::Vector2;

pub fn to_centered(position: [f64; 2], size: &Size) -> [f64; 2] {
    [position[0] - size.width / 2., size.height / 2. - position[1]]
}

pub fn to_left_up(position: [f64; 2], size: &Size) -> [f64; 2] {
    [position[0] + size.width / 2., size.height / 2. - position[1]]
}

pub fn offset_or_position(position: [f64; 2], size: &Option<Size>) -> [f64; 2] {
    match size {
        Some(size) => to_left_up(position, size),
        None => position
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


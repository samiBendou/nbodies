use std::fmt::{Debug, Error, Formatter};

use piston::input::Key;

use crate::vector::Vector2;

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

#[derive(Copy, Clone, Debug)]
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
}



use piston::input::Key;

pub const WINDOW_WIDTH: f64 = 800.;
pub const WINDOW_HEIGHT: f64 = 800.;
const BASE_SPEED: f64 = 0.25;
const MAX_SPEED: i8 = 20;

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

    pub fn from(key: Key) -> Direction {
        match key {
            Key::Left => Direction::Left,
            Key::Right => Direction::Right,
            Key::Up => Direction::Up,
            Key::Down => Direction::Down,
            Key::Space => Direction::Hold,
            _ => Direction::Hold,
        }
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Circle {
    pub x: f64,
    pub y: f64,
    pub radius: f64,

    pub direction: Direction,
    pub speed: i8,
    pub color: Color,
}

impl Circle {
    pub fn new(x: f64, y: f64, radius: f64, color: Color) -> Circle {
        let speed = 0;
        let direction = Direction::Hold;
        Circle { x, y, radius, direction, speed, color }
    }

    pub fn diameter(&self) -> f64 {
        2. * self.radius
    }

    pub fn rounding_rect(&self) -> [f64; 4] {
        let diameter = self.diameter();
        [self.x - self.radius, self.y - self.radius, diameter, diameter]
    }

    pub fn replace(&mut self, width: f64, height: f64) -> &mut Circle {
        if self.x < -self.radius {
            self.x = width + self.radius;
        } else if self.x > width + self.radius {
            self.x = -self.radius;
        }

        if self.y < -self.radius {
            self.y = height + self.radius;
        } else if self.y > height + self.radius {
            self.y = -self.radius;
        }

        self
    }

    pub fn accelerate(&mut self) -> &mut Circle {
        self.speed = (self.speed + 1) % MAX_SPEED;

        self
    }

    pub fn decelerate(&mut self) -> &mut Circle {
        self.speed = (self.speed - 1) % MAX_SPEED;

        self
    }

    pub fn turn(&mut self, direction: &Direction) -> &mut Circle {
        self.direction = direction.clone();

        self
    }

    pub fn translate(&mut self, direction: &Direction) -> &mut Circle {
        if *direction == Direction::Hold {
            self.turn(&direction)
        } else if *direction == self.direction {
            self.accelerate()
        } else if direction.opposite(&self.direction) {
            if self.speed > 0 {
                self.decelerate()
            } else {
                self.turn(&direction)
            }
        } else {
            self.turn(&direction)
        }
    }

    pub fn update(&mut self) -> &mut Circle {
        let speed = self.speed as f64 * BASE_SPEED + BASE_SPEED;
        match self.direction {
            Direction::Left => self.x = self.x - speed,
            Direction::Right => self.x = self.x + speed,
            Direction::Up => self.y = self.y - speed,
            Direction::Down => self.y = self.y + speed,
            Direction::Hold => (),
        };

        self
    }
}
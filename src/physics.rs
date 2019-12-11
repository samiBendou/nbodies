use std::cmp::{max, min};
use std::fmt::{Debug, Error, Formatter};
use std::ops::{Index, IndexMut};

use piston::input::Key;
use piston::window::Size;

use crate::common::offset_or_position;
use crate::shape::Circle;
use crate::vector::Vector2;

const BASE_ACCELERATION: f64 = 500.;
const RESISTANCE: f64 = 0.001;
pub const PX_PER_METER: f64 = 10.;
pub(crate) const TRAJECTORY_SIZE: usize = 256;

#[derive(Copy, Clone)]
pub struct Point {
    pub position: Vector2,
    pub speed: Vector2,
    pub acceleration: Vector2,

    trajectory: [Vector2; TRAJECTORY_SIZE],
    index: usize,
}

impl Point {
    pub fn new(position: Vector2, speed: Vector2, acceleration: Vector2, size: &Option<Size>) -> Point {
        let position_offset = Vector2::from(offset_or_position(position.as_array(), size));

        Point {
            position,
            speed,
            acceleration,
            trajectory: [position_offset; TRAJECTORY_SIZE],
            index: 0,
        }
    }

    pub fn inertial(position: Vector2, speed: Vector2, size: &Option<Size>) -> Point {
        Point::new(position, speed, Vector2::zeros(), size)
    }

    pub fn stationary(position: Vector2, size: &Option<Size>) -> Point {
        Point::new(position, Vector2::zeros(), Vector2::zeros(), size)
    }

    pub fn zeros(size: &Option<Size>) -> Point {
        Point::new(Vector2::zeros(), Vector2::zeros(), Vector2::zeros(), size)
    }

    pub fn reset(&mut self, position: Vector2) -> &mut Point {
        self.position = position;
        self.speed = Vector2::zeros();
        self.acceleration = Vector2::zeros();
        self
    }

    pub fn translate(&mut self, direction: &Vector2) -> &mut Point {
        self.position += *direction * PX_PER_METER;
        self
    }

    pub fn accelerate(&mut self, dt: f64) -> &mut Point {
        self.speed += self.acceleration * dt;
        self.position += self.speed * dt;
        self
    }

    pub fn position(&self, k: usize) -> &Vector2 {
        let index = (k + self.index + 1) % TRAJECTORY_SIZE;

        &self.trajectory[index]
    }

    pub fn update_trajectory(&mut self, size: &Option<Size>) {
        let position_offset = offset_or_position(self.position.as_array(), size);
        self.trajectory[self.index].set_array(&position_offset);
        self.index = (self.index + 1) % TRAJECTORY_SIZE;
    }

    pub fn clear_trajectory(&mut self, size: &Option<Size>) {
        let position_offset = offset_or_position(self.position.as_array(), size);
        for position in self.trajectory.iter_mut() {
            position.set_array(&position_offset);
        }
    }
}

impl Debug for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "position: {:?} (px)\nspeed: {:?} (px/s)\nacceleration: {:?} (px/s2)",
            self.position, self.speed, self.acceleration
        )
    }
}

pub struct Body {
    pub mass: f64,
    pub name: String,
    pub shape: Circle,
}

impl Body {
    pub fn new(mass: f64, name: String, shape: Circle) -> Body {
        Body { mass, name, shape }
    }
}

impl Debug for Body {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "\
*** {} ***\n\
mass {:.4} (kg)\n\
position: {:?} (m)\nspeed: {:?} (m/s)\nacceleration: {:?} (m/s2)\
",
            self.name,
            self.mass,
            self.shape.center.position / PX_PER_METER,
            self.shape.center.speed / PX_PER_METER,
            self.shape.center.acceleration / PX_PER_METER
        )
    }
}


pub struct VecBody {
    bodies: Vec<Body>,
    current: usize,
}

impl Index<usize> for VecBody {
    type Output = Body;

    fn index(&self, index: usize) -> &Self::Output {
        &self.bodies[index]
    }
}

impl IndexMut<usize> for VecBody {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.bodies[index]
    }
}

impl VecBody {
    pub fn new(bodies: Vec<Body>) -> Self {
        VecBody { bodies, current: 0 }
    }

    pub fn empty() -> Self {
        VecBody { bodies: vec![], current: 0 }
    }

    pub fn is_empty(&self) -> bool {
        self.bodies.len() == 0
    }

    pub fn current(&self) -> &Body {
        &self.bodies[self.current]
    }

    pub fn current_mut(&mut self) -> &mut Body {
        &mut self.bodies[self.current]
    }

    pub fn current_index(&self) -> usize {
        self.current
    }

    pub fn increase_current(&mut self) -> &mut Self {
        let current = self.current as isize;
        self.current = max(current - 1, 0) as usize;
        self
    }

    pub fn count(&self) -> usize {
        self.bodies.len()
    }

    pub fn decrease_current(&mut self) -> &mut Self {
        let current = self.current as isize;
        let count = self.count() as isize;
        self.current = min(current + 1, max(count - 1, 0)) as usize;
        self
    }

    pub fn reset_current(&mut self) -> &mut Self {
        self.bodies[self.current].shape.center.reset(Vector2::zeros());
        self
    }

    pub fn clear_current_trajectory(&mut self, size: &Option<Size>) -> &mut Self {
        self.bodies[self.current].shape.center.clear_trajectory(size);
        self
    }

    pub fn update_current_trajectory(&mut self, size: &Option<Size>) -> &mut Self {
        self.bodies[self.current].shape.center.update_trajectory(size);
        self
    }

    pub fn bound_current(&mut self, size: &Size) -> &mut Self {
        self.bodies[self.current].shape.bound(*size);
        self
    }

    pub fn translate_current(&mut self, direction: &Vector2) -> &mut Self {
        self.bodies[self.current].shape.center.translate(direction);
        self
    }

    pub fn accelerate_current(&mut self, dt: f64) -> &mut Self {
        self.bodies[self.current].shape.center.accelerate(dt);
        self
    }

    pub fn accelerate(&mut self, dt: f64) -> &mut Self {
        for body in self.bodies.iter_mut() {
            body.shape.center.accelerate(dt);
        }
        self
    }

    pub fn bound(&mut self, size: &Size) -> &mut Self {
        for body in self.bodies.iter_mut() {
            body.shape.bound(*size);
        }
        self
    }

    pub fn update_trajectory(&mut self, size: &Option<Size>) -> &mut Self {
        for body in self.bodies.iter_mut() {
            body.shape.center.update_trajectory(size);
        }
        self
    }

    pub fn clear_trajectory(&mut self, size: &Option<Size>) -> &mut Self {
        for body in self.bodies.iter_mut() {
            body.shape.center.clear_trajectory(size);
        }
        self
    }

    pub fn update_current(&mut self, key: &Key) -> &mut Self {
        match key {
            Key::Z => self.increase_current(),
            Key::X => self.decrease_current(),
            _ => self
        }
    }

    pub fn push(&mut self, body: Body) -> &mut Self {
        self.current = self.bodies.len();
        self.bodies.push(body);
        self
    }

    pub fn pop(&mut self) -> Option<Body> {
        if self.current == self.bodies.len() - 1 {
            self.current -= 1;
        }
        self.bodies.pop()
    }

    pub fn wait_drop(&mut self, cursor: &[f64; 2], size: &Size) -> &mut Self {
        self.bodies[self.current].shape.set_cursor_pos(cursor, size);
        self.bodies[self.current].shape.center.clear_trajectory(&Some(*size));
        self
    }
}

impl Debug for VecBody {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let mut str = String::from("***body system***");
        for body in self.bodies.iter() {
            str.push_str(format!("\n{:?}", body).as_str());
        }
        write!(f, "{}", str)
    }
}

pub mod forces {
    use crate::common::Direction;
    use crate::physics::{BASE_ACCELERATION, RESISTANCE};
    use crate::vector::Vector2;

    pub fn push(direction: &Direction) -> Vector2 {
        direction.as_vector() * BASE_ACCELERATION
    }

    pub fn nav_stokes(speed: &Vector2) -> Vector2 {
        *speed * (-RESISTANCE * speed.magnitude())
    }
}
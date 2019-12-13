use std::cmp::{max, min};
use std::fmt::{Debug, Error, Formatter};
use std::ops::{Index, IndexMut};

use piston::window::Size;

use crate::offset_or_position;
use crate::physics::vector::Vector2;
use crate::shapes::ellipse::Circle;

pub const TRAJECTORY_SIZE: usize = 256;

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
        let mut position_offset = position.clone();

        offset_or_position!(position_offset, size);
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

    pub fn scale(&mut self, scale: f64) -> &mut Self {
        self.position *= scale;
        self
    }

    pub fn translate(&mut self, direction: &Vector2) -> &mut Point {
        self.position += *direction;
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
        self.trajectory[self.index] = self.position;
        offset_or_position!(self.trajectory[self.index], size);
        self.index = (self.index + 1) % TRAJECTORY_SIZE;
    }

    pub fn clear_trajectory(&mut self, size: &Option<Size>) {
        let mut position_offset = self.position;
        offset_or_position!(position_offset, size);
        for position in self.trajectory.iter_mut() {
            *position = position_offset;
        }
    }
}

impl Debug for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use super::units::prefix;
        let position_prefix = prefix::Scale::from(self.position.magnitude());
        let speed_prefix = prefix::Scale::from(self.speed.magnitude());
        let acceleration_prefix = prefix::Scale::from(self.acceleration.magnitude());
        let position = self.position * position_prefix.multiplier;
        let speed = self.speed * speed_prefix.multiplier;
        let acceleration = self.acceleration * acceleration_prefix.multiplier;
        write!(
            f,
            "position: {:?} ({}m)\nspeed: {:?} ({}m/s)\nacceleration: {:?} ({}m/s2)",
            position, position_prefix.label,
            speed, speed_prefix.label,
            acceleration, acceleration_prefix.label
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
        use super::units::prefix;
        let mass_kg = self.mass * 1e3;
        let prefix = prefix::Scale::from(mass_kg);
        write!(f, "name:{}\nmass {:.4} ({}g)\n{:?}",
               self.name, prefix.value_of(mass_kg), prefix.label, self.shape)
    }
}


pub struct VecBody {
    bodies: Vec<Body>,
    barycenter: Vector2,
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
        VecBody { bodies, barycenter: Vector2::zeros(), current: 0 }
    }

    pub fn empty() -> Self {
        VecBody { bodies: vec![], barycenter: Vector2::zeros(), current: 0 }
    }

    pub fn is_empty(&self) -> bool {
        self.bodies.len() == 0
    }

    pub fn barycenter(&self) -> &Vector2 {
        &self.barycenter
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

    pub fn update_current_index(&mut self, increase: bool) -> &mut Self {
        match increase {
            true => self.increase_current(),
            false => self.decrease_current(),
        }
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

    pub fn update_barycenter(&mut self) -> &mut Self {
        let mut total_mass = 0.;
        self.barycenter.reset0();
        for body in self.bodies.iter() {
            self.barycenter += body.shape.center.position * body.mass;
            total_mass += body.mass;
        }
        self.barycenter /= total_mass;
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

    pub fn wait_drop(&mut self, cursor: &[f64; 2], size: &Size, scale: f64) -> &mut Self {
        self.bodies[self.current].shape.set_cursor_pos(cursor, size);
        self.bodies[self.current].shape.center.position /= scale;
        self.bodies[self.current].shape.center.clear_trajectory(&Some(*size));
        self
    }
}

impl Debug for VecBody {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let mut str = String::from("");
        for body in self.bodies.iter() {
            str.push_str(format!("{:?}\n", body).as_str());
        }
        write!(f, "{}", str)
    }
}
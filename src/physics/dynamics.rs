use std::cmp::{max, min};
use std::fmt::{Debug, Error, Formatter};
use std::ops::{AddAssign, DivAssign, Index, IndexMut, Mul, MulAssign, Rem, SubAssign};

use crate::physics::units::{Rescale, Serialize};
use crate::physics::units::suffix::Distance;
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
    pub fn new(position: Vector2, speed: Vector2, acceleration: Vector2) -> Point {
        Point {
            position,
            speed,
            acceleration,
            trajectory: [position.clone(); TRAJECTORY_SIZE],
            index: 0,
        }
    }

    pub fn inertial(position: Vector2, speed: Vector2) -> Point {
        Point::new(position, speed, Vector2::zeros())
    }

    pub fn stationary(position: Vector2) -> Point {
        Point::new(position, Vector2::zeros(), Vector2::zeros())
    }

    pub fn zeros() -> Point {
        Point::new(Vector2::zeros(), Vector2::zeros(), Vector2::zeros())
    }

    pub fn reset0(&mut self) -> &mut Self {
        self.position.reset0();
        self.speed.reset0();
        self.acceleration.reset0();
        self
    }

    pub fn reset(&mut self, position: Vector2) -> &mut Self {
        self.position = position;
        self.speed.reset0();
        self.acceleration.reset0();
        self
    }

    pub fn scale(&mut self, scale: f64) -> &mut Self {
        self.position *= scale;
        self
    }

    pub fn translate(&mut self, direction: &Vector2) -> &mut Self {
        self.position += *direction;
        self
    }

    pub fn accelerate(&mut self, dt: f64) -> &mut Self {
        self.speed += self.acceleration * dt;
        self.position += self.speed * dt;
        self
    }

    pub fn position(&self, k: usize) -> &Vector2 {
        let index = (k + self.index + 1) % TRAJECTORY_SIZE;

        &self.trajectory[index]
    }

    pub fn update_trajectory(&mut self) {
        self.trajectory[self.index] = self.position;
        self.index = (self.index + 1) % TRAJECTORY_SIZE;
    }

    pub fn clear_trajectory(&mut self) {
        for position in self.trajectory.iter_mut() {
            *position = self.position;
        }
    }

    pub fn set_origin(&mut self, origin: &Point, old_origin: &Option<Point>) -> &mut Self {
        let mut translation = *origin;
        if let Some(old_origin) = old_origin {
            translation -= *old_origin
        }
        *self -= translation;
        self
    }
}

impl Debug for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use super::units::Unit;
        let mut position_unit = Unit::from(Distance::Standard);
        let mut speed_unit = Unit::from(Distance::Standard);
        let mut acceleration_unit = Unit::from(Distance::Standard);
        write!(
            f,
            "position: {}\nspeed: {}\nacceleration: {}",
            position_unit.rescale(self.position).string_of(self.position),
            speed_unit.rescale(self.speed).string_of(self.speed),
            acceleration_unit.rescale(self.acceleration).string_of(self.acceleration),
        )
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position && self.speed == other.speed
    }

    fn ne(&self, other: &Self) -> bool {
        self.position != other.position || self.speed != other.speed
    }
}

impl AddAssign<Point> for Point {
    fn add_assign(&mut self, rhs: Point) {
        self.position += rhs.position;
        self.speed += rhs.speed;
        for k in 0..TRAJECTORY_SIZE {
            self.trajectory[k] += rhs.trajectory[k];
        }
    }
}

impl SubAssign<Point> for Point {
    fn sub_assign(&mut self, rhs: Point) {
        self.position -= rhs.position;
        self.speed -= rhs.speed;
        for k in 0..TRAJECTORY_SIZE {
            self.trajectory[k] -= rhs.trajectory[k];
        }
    }
}

impl Mul<f64> for Point {
    type Output = Point;

    fn mul(self, rhs: f64) -> Self::Output {
        let mut output = self;
        output *= rhs;
        output
    }
}

impl MulAssign<f64> for Point {
    fn mul_assign(&mut self, rhs: f64) {
        self.position *= rhs;
        self.speed *= rhs;
        for k in 0..TRAJECTORY_SIZE {
            self.trajectory[k] *= rhs;
        }
    }
}

impl DivAssign<f64> for Point {
    fn div_assign(&mut self, rhs: f64) {
        self.position /= rhs;
        self.speed /= rhs;
        for k in 0..TRAJECTORY_SIZE {
            self.trajectory[k] /= rhs;
        }
    }
}

impl Rem<Point> for Point {
    type Output = f64;

    fn rem(self, rhs: Point) -> Self::Output {
        self.position % rhs.position
    }
}

pub struct Body {
    pub mass: f64,
    pub name: String,
    pub shape: Circle,
}

impl Body {
    pub fn new(mass: f64, name: &str, shape: Circle) -> Body {
        Body { mass, name: String::from(name), shape }
    }
}

impl Debug for Body {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use super::units::Unit;
        use super::units::suffix::Mass;
        let mut mass_unit = Unit::from(Mass::Standard);
        write!(f, "name:{}\nmass: {}\n{:?}",
               self.name, mass_unit.rescale(1e3).string_of(self.mass * 1e3), self.shape)
    }
}


pub struct VecBody {
    bodies: Vec<Body>,
    barycenter: Body,
    origin: Option<Point>,
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
        let shape = Circle::new(Point::zeros(), 0., [1., 0., 0., 0.]);
        let barycenter = Body::new(0., "barycenter", shape);
        VecBody { bodies, barycenter, origin: None, current: 0 }
    }

    pub fn empty() -> Self {
        let shape = Circle::new(Point::zeros(), 0., [1., 0., 0., 0.]);
        let barycenter = Body::new(0., "barycenter", shape);
        VecBody { bodies: vec![], barycenter, origin: None, current: 0 }
    }

    pub fn is_empty(&self) -> bool {
        self.bodies.len() == 0
    }

    pub fn count(&self) -> usize {
        self.bodies.len()
    }

    pub fn barycenter(&self) -> &Body {
        &self.barycenter
    }

    pub fn origin(&self) -> Point {
        match self.origin {
            Some(origin) => origin,
            None => Point::zeros()
        }
    }

    pub fn update_origin(&mut self, origin: &Point) -> &mut Self {
        if self.is_empty() {
            return self;
        }
        for body in self.bodies.iter_mut() {
            body.shape.center.set_origin(origin, &self.origin);
        }
        self
    }

    pub fn update_barycenter(&mut self) -> &mut Self {
        let mut total_mass = 0.;
        self.barycenter.shape.center.reset0();
        for body in self.bodies.iter() {
            self.barycenter.shape.center += body.shape.center * body.mass;
            total_mass += body.mass;
        }
        self.barycenter.shape.center /= total_mass;
        self.barycenter.mass = total_mass;
        self
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

    pub fn update_current_index(&mut self, increase: bool) -> &mut Self {
        let current = self.current;
        match increase {
            true => self.increase_current(),
            false => self.decrease_current(),
        }
    }


    pub fn reset_current(&mut self) -> &mut Self {
        self.bodies[self.current].shape.center.reset(Vector2::zeros());
        self
    }

    pub fn clear_current_trajectory(&mut self) -> &mut Self {
        self.bodies[self.current].shape.center.clear_trajectory();
        self
    }

    pub fn update_current_trajectory(&mut self) -> &mut Self {
        self.bodies[self.current].shape.center.update_trajectory();
        self
    }

    pub fn bound_current(&mut self, middle: &Vector2) -> &mut Self {
        self.bodies[self.current].shape.bound(middle);
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

    pub fn apply<T>(&mut self, dt: f64, iterations: u32, mut f: T) where
        T: FnMut(&Body, usize) -> Vector2 {
        let mut mass: f64;
        for _ in 0..iterations {
            for i in 0..self.bodies.len() {
                mass = self.bodies[i].mass;
                self.bodies[i].shape.center.acceleration = f(&self.bodies[i], i);
                self.bodies[i].shape.center.acceleration /= mass;
            }
            self.accelerate(dt);
        }
    }

    pub fn bound(&mut self, middle: &Vector2) -> &mut Self {
        for body in self.bodies.iter_mut() {
            body.shape.bound(middle);
        }
        self
    }

    pub fn update_trajectory(&mut self) -> &mut Self {
        for body in self.bodies.iter_mut() {
            body.shape.center.update_trajectory();
        }
        self
    }

    pub fn clear_trajectory(&mut self) -> &mut Self {
        for body in self.bodies.iter_mut() {
            body.shape.center.clear_trajectory();
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

    pub fn wait_drop(&mut self, cursor: &[f64; 2], middle: &Vector2, scale: f64) -> &mut Self {
        self.bodies[self.current].shape.set_cursor_pos(cursor, middle);
        self.bodies[self.current].shape.center.position /= scale;
        self.bodies[self.current].shape.center.clear_trajectory();
        self
    }

    fn increase_current(&mut self) -> &mut Self {
        let current = self.current as isize;
        self.current = max(current - 1, 0) as usize;
        self
    }

    fn decrease_current(&mut self) -> &mut Self {
        let current = self.current as isize;
        let count = self.count() as isize;
        self.current = min(current + 1, max(count - 1, 0)) as usize;
        self
    }
}

impl Debug for VecBody {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let mut buffer = String::from("");
        buffer.push_str(format!("{:?}\n", self.barycenter).as_str());
        for body in self.bodies.iter() {
            buffer.push_str(format!("{:?}\n", body).as_str());
        }
        write!(f, "{}", buffer)
    }
}
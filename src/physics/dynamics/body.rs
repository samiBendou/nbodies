use std::cmp::{max, min};
use std::fmt::{Debug, Error, Formatter};
use std::ops::{Index, IndexMut};

use crate::physics::dynamics::point::Point2;
use crate::physics::vector::Vector2;
use crate::shapes::ellipse::Circle;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Frame {
    Zero,
    Current,
    Barycenter,
}

impl Frame {
    pub fn next(&mut self) {
        use Frame::*;
        *self = match self {
            Zero => Current,
            Current => Barycenter,
            Barycenter => Zero,
        }
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
        use crate::physics::units::{Scale, Unit, Rescale, Serialize};
        use crate::physics::units::suffix::Mass;
        let mut mass_unit = Unit::from(Scale::from(Mass::Grams));
        write!(f, "name:{}\nmass: {}\n{:?}",
               self.name, mass_unit.rescale(self.mass * 1e3).string_of(self.mass * 1e3), self.shape)
    }
}

pub struct Cluster {
    bodies: Vec<Body>,
    barycenter: Body,
    origin: Point2,
    current: usize,
    pub frame: Frame,
}

impl Cluster {
    pub fn new(bodies: Vec<Body>) -> Self {
        let shape = Circle::new(Point2::zeros(), 0., [1., 0., 0., 0.]);
        let barycenter = Body::new(0., "barycenter", shape);
        Cluster { bodies, barycenter, origin: Point2::zeros(), current: 0, frame: Frame::Zero }
    }

    pub fn empty() -> Self {
        Cluster::new(vec![])
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

    pub fn origin(&self) -> &Point2 {
        &self.origin
    }

    fn update_origin(&mut self) -> &mut Self {
        self.origin = match self.frame {
            Frame::Zero => Point2::zeros(),
            Frame::Current => self.bodies[self.current].shape.center,
            Frame::Barycenter => self.barycenter.shape.center,
        };
        self
    }

    fn clear_origin(&mut self) -> &mut Self {
        if self.is_empty() {
            return self;
        }
        let origin = self.origin;
        self.update_origin();
        for body in self.bodies.iter_mut() {
            body.shape.center.set_origin(&self.origin, &Some(origin));
        }
        self
    }

    fn clear_barycenter(&mut self) -> &mut Self {
        self.barycenter.mass = 0.;
        self.barycenter.shape.center.reset0();
        for body in self.bodies.iter() {
            self.barycenter.mass += body.mass;
            self.barycenter.shape.center += body.shape.center * body.mass;
        }
        self.barycenter.shape.center /= self.barycenter.mass;
        self.barycenter.mass = self.barycenter.mass;
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

    pub fn update_frame(&mut self) -> &mut Self {
        self.frame.next();
        self.clear_origin();
        self.clear_barycenter()
    }

    pub fn update_current_index(&mut self, increase: bool) -> &mut Self {
        match increase {
            true => self.increase_current(),
            false => self.decrease_current(),
        };
        if self.frame == Frame::Current {
            self.clear_origin();
        }
        self.clear_barycenter()
    }


    pub fn reset_current(&mut self) -> &mut Self {
        self.bodies[self.current].shape.center.reset(Vector2::zeros());
        self.bodies[self.current].shape.center.clear_trajectory();
        self.clear_barycenter();
        self
    }

    pub fn clear_current_trajectory(&mut self) -> &mut Self {
        self.bodies[self.current].shape.center.clear_trajectory();
        self.clear_barycenter();
        self
    }

    pub fn update_current_trajectory(&mut self) -> &mut Self {
        self.bodies[self.current].shape.center.update_trajectory();
        self
    }

    pub fn bound_current(&mut self, middle: &Vector2) -> &mut Self {
        self.bodies[self.current].shape.bound(middle);
        self.clear_barycenter();
        self
    }

    pub fn translate_current(&mut self, direction: &Vector2) -> &mut Self {
        self.bodies[self.current].shape.center.translate(direction);
        self
    }

    pub fn accelerate(&mut self, dt: f64) -> &mut Self {
        self.barycenter.shape.center.accelerate(dt);
        for body in self.bodies.iter_mut() {
            body.shape.center.accelerate(dt);
        }
        self
    }

    pub fn apply<T>(&mut self, dt: f64, iterations: u32, mut f: T) where
        T: FnMut(&Body, usize) -> Vector2 {
        let count = self.bodies.len();
        let mass: Vec<f64> = self.bodies.iter()
            .map(|body| body.mass)
            .collect();
        let mut force: Vector2;
        self.barycenter.shape.center.position += self.origin.position;
        self.barycenter.shape.center.speed += self.origin.speed;
        for body in self.bodies.iter_mut() {
            body.shape.center.position += self.origin.position;
            body.shape.center.speed += self.origin.speed;
        }
        for _ in 0..iterations {
            self.barycenter.shape.center.acceleration.reset0();
            for i in 0..count {
                force = f(&self.bodies[i], i);
                self.barycenter.shape.center.acceleration += force;
                self.bodies[i].shape.center.acceleration = force;
                self.bodies[i].shape.center.acceleration /= mass[i];
            }
            self.barycenter.shape.center.acceleration /= self.barycenter.mass;
            self.accelerate(dt);
        }
        self.update_origin();
        self.barycenter.shape.center.position -= self.origin.position;
        self.barycenter.shape.center.speed -= self.origin.speed;
        for body in self.bodies.iter_mut() {
            body.shape.center.position -= self.origin.position;
            body.shape.center.speed -= self.origin.speed;
        }
    }

    pub fn bound(&mut self, middle: &Vector2) -> &mut Self {
        for body in self.bodies.iter_mut() {
            body.shape.bound(middle);
        }
        self.clear_barycenter();
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
        self.clear_barycenter();
        self
    }

    pub fn pop(&mut self) -> Option<Body> {
        if self.current == self.bodies.len() - 1 {
            self.current -= 1;
        }
        let body = self.bodies.pop();
        self.clear_barycenter();
        body
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

impl Debug for Cluster {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let mut buffer = String::from("");
        buffer.push_str(format!("{:?}\n", self.barycenter).as_str());
        for body in self.bodies.iter() {
            buffer.push_str(format!("{:?}\n", body).as_str());
        }
        write!(f, "{}", buffer)
    }
}

impl Index<usize> for Cluster {
    type Output = Body;

    fn index(&self, index: usize) -> &Self::Output {
        &self.bodies[index]
    }
}

impl IndexMut<usize> for Cluster {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.bodies[index]
    }
}
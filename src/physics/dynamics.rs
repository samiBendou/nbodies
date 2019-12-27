use std::fmt::{Debug, Error, Formatter};
use std::ops::{Index, IndexMut};

use rand::Rng;

use crate::physics::dynamics::point::Point2;
use crate::physics::vector::Vector2;
use crate::shapes::ellipse::Circle;

pub mod point;
pub mod forces;
pub mod orbital;

pub const SPEED_SCALING_FACTOR: f64 = 10e7;

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

    pub fn planet(body: &orbital::Body, true_anomaly: f64) -> Body {
        let position = body.orbit.position_at(true_anomaly);
        let speed = body.orbit.speed_at(true_anomaly);
        let center = Point2::inertial(position, speed);
        let shape = Circle::new(center, 30.0, body.color);
        Body::new(body.mass, body.name.as_str(), shape)
    }
}

impl Debug for Body {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "name: {}\nmass: {:.5e}\n{:?}",
               self.name, self.mass, self.shape.center)
    }
}

pub struct Cluster {
    pub bodies: Vec<Body>,
    barycenter: Body,
    origin: Point2,
    current: usize,
    pub frame: Frame,
}

impl Cluster {
    pub fn new(bodies: Vec<Body>) -> Self {
        let shape = Circle::new(Point2::zeros(), 0., [1., 0., 0., 0.]);
        let barycenter = Body::new(0., "barycenter", shape);
        Cluster {
            bodies,
            barycenter,
            origin: Point2::zeros(),
            current: 0,
            frame: Frame::Zero,
        }
    }

    pub fn from_orbits(cluster: orbital::Cluster, true_anomalies: Vec<f64>) -> Self {
        let len = cluster.bodies.len();
        let mut bodies: Vec<Body> = Vec::with_capacity(len);
        for i in 0..len {
            bodies.push(Body::planet(&cluster.bodies[i], true_anomalies[i]));
        }
        Cluster::new(bodies)
    }

    pub fn from_orbits_at(cluster: orbital::Cluster, true_anomaly: f64) -> Self {
        let len = cluster.bodies.len();
        let mut bodies: Vec<Body> = Vec::with_capacity(len);
        for i in 0..len {
            bodies.push(Body::planet(&cluster.bodies[i], true_anomaly));
        }
        Cluster::new(bodies)
    }

    pub fn from_orbits_random(cluster: orbital::Cluster) -> Self {
        let two_pi = 2. * std::f64::consts::PI;
        let len = cluster.bodies.len();
        let mut rng = rand::thread_rng();
        let mut bodies: Vec<Body> = Vec::with_capacity(len);
        let mut anomaly: f64;
        for i in 0..len {
            anomaly = rng.gen_range(0., two_pi);
            bodies.push(Body::planet(&cluster.bodies[i], anomaly));
        }
        Cluster::new(bodies)
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

    pub fn max_distance(&self) -> f64 {
        let mut max_distance = 0.;
        let mut distance: f64;
        for body in self.bodies.iter() {
            distance = body.shape.center.position.magnitude();
            if distance > max_distance {
                max_distance = distance;
            }
        }
        max_distance
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
        self.update_origin();
        self.barycenter.shape.center.set_origin(&self.origin, &None);
        for body in self.bodies.iter_mut() {
            body.shape.center.set_origin(&self.origin, &None);
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
        if increase {
            self.increase_current();
        } else {
            self.decrease_current();
        }
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

    pub fn translate(&mut self, direction: &Vector2) -> &mut Self {
        self.barycenter.shape.center.translate(direction);
        for body in self.bodies.iter_mut() {
            body.shape.center.translate(direction);
        }
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
        T: FnMut(&mut Vector2, &Cluster, usize) {
        let count = self.bodies.len();
        let mass: Vec<f64> = self.bodies.iter()
            .map(|body| body.mass)
            .collect();
        let mut force = Vector2::zeros();
        self.deframe();
        for _ in 0..iterations {
            self.barycenter.shape.center.acceleration.reset0();
            for i in 0..count {
                f(&mut force, &self, i);
                self.barycenter.shape.center.acceleration += force;
                self.bodies[i].shape.center.acceleration = force;
                self.bodies[i].shape.center.acceleration /= mass[i];
            }
            self.barycenter.shape.center.acceleration /= self.barycenter.mass;
            self.accelerate(dt);
        }
        self.update_origin();
        self.reframe();
    }

    pub fn bound(&mut self, middle: &Vector2) -> &mut Self {
        for body in self.bodies.iter_mut() {
            body.shape.bound(middle);
        }
        self.clear_barycenter();
        self
    }

    pub fn deframe(&mut self) -> &mut Self {
        self.barycenter.shape.center.position += self.origin.position;
        self.barycenter.shape.center.speed += self.origin.speed;
        for body in self.bodies.iter_mut() {
            body.shape.center.position += self.origin.position;
            body.shape.center.speed += self.origin.speed;
        }
        self
    }

    pub fn reframe(&mut self) -> &mut Self {
        self.barycenter.shape.center.position -= self.origin.position;
        self.barycenter.shape.center.speed -= self.origin.speed;
        for body in self.bodies.iter_mut() {
            body.shape.center.position -= self.origin.position;
            body.shape.center.speed -= self.origin.speed;
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
        self.clear_barycenter();
        self
    }

    pub fn pop(&mut self) -> Option<Body> {
        let len = self.bodies.len();
        if self.current != 0 && self.current == len - 1 {
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
        self.clear_barycenter();
        self
    }

    pub fn wait_speed(&mut self, cursor: &[f64; 2], middle: &Vector2, scale: f64) -> &mut Self {
        self.bodies[self.current].shape.set_cursor_speed(cursor, middle, scale);
        self.bodies[self.current].shape.center.speed /= scale * SPEED_SCALING_FACTOR;
        self.bodies[self.current].shape.center.clear_trajectory();
        self.clear_barycenter();
        self
    }

    fn decrease_current(&mut self) -> &mut Self {
        if self.current > 0 {
            self.current -= 1;
        }
        self
    }

    fn increase_current(&mut self) -> &mut Self {
        if self.current < self.count() - 1 {
            self.current += 1;
        }
        self
    }
}

impl Debug for Cluster {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let mut buffer = String::from("");
        buffer.push_str(format!("{:?}\n\n", self.barycenter).as_str());
        for body in self.bodies.iter() {
            buffer.push_str(format!("{:?}\n\n", body).as_str());
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
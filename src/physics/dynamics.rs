use std::fmt::{Debug, Error, Formatter};
use std::ops::{Index, IndexMut};

use rand::Rng;

use crate::physics::dynamics::point::Point2;
use crate::physics::vector::{Vector2, Vector4};
use crate::shapes::ellipse::Circle;

pub mod point;
pub mod forces;
pub mod potentials;
pub mod orbital;

pub const SPEED_SCALING_FACTOR: f64 = 2e6;

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
        let mut shape = Circle::new(center, 0., body.color);
        shape.center.mass = body.mass;
        let mut result = Body::new(body.mass, body.name.as_str(), shape);
        result.set_radius_from(body);
        result
    }

    pub fn set_radius_from(&mut self, body: &orbital::Body) {
        self.shape.radius = body.kind.scaled_radius(body.radius);
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
    frame: Frame,
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

    pub fn kinetic_energy(&self) -> f64 {
        self.bodies.iter().map(|body| body.shape.center.kinetic_energy()).sum()
    }

    pub fn potential_energy<T>(&self, mut f: T) -> f64 where
        T: FnMut(&Cluster, usize) -> f64 {
        let len = self.bodies.len();
        let mut ret = 0.;
        for i in 0..len {
            ret += f(self, i);
        }
        ret
    }

    pub fn max_distance(&self) -> (f64, usize) {
        let mut max_distance = 0.;
        let mut max_index: usize = 0;
        let mut distance: f64;
        let len = self.bodies.len();
        for i in 0..len {
            distance = self.bodies[i].shape.center % self.barycenter.shape.center;
            if distance > max_distance {
                max_distance = distance;
                max_index = i;
            }
        }
        (max_distance, max_index)
    }

    pub fn stats_distance_without(&self, index: Option<usize>) -> (f64, f64, Vec<f64>) {
        let len = self.bodies.len();
        let mut mean = 0.;
        let mut sum2 = 0.;
        let mut distances: Vec<f64> = Vec::with_capacity(len);
        let index = match index {
            None => len,
            Some(index) => index,
        };
        for i in 0..len {
            distances.push(self.bodies[i].shape.center % self.barycenter.shape.center);
            if i == index {
                continue;
            }
            mean += distances[i];
            sum2 += distances[i] * distances[i];
        }
        let len = len as f64;
        mean /= len;
        (mean, (sum2 / len - mean * mean).sqrt(), distances)
    }

    pub fn remove_aways(&mut self) -> &mut Self {
        let (max_distance, max_index) = self.max_distance();
        let (mean, deviation, _distances) = if self.bodies.len() < 3 {
            self.stats_distance_without(None)
        } else {
            self.stats_distance_without(Some(max_index))
        };
        if max_distance > mean + 10e2 * deviation {
            self.remove(max_index);
            self.clear_barycenter();
        }
        self
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
            self.barycenter.shape.center.position += body.shape.center.position * body.mass;
            self.barycenter.shape.center.speed += body.shape.center.speed * body.mass;
        }
        self.barycenter.shape.center.position /= self.barycenter.mass;
        self.barycenter.shape.center.speed /= self.barycenter.mass;
        self
    }

    pub fn current(&self) -> Option<&Body> {
        self.bodies.get(self.current)
    }

    pub fn current_mut(&mut self) -> Option<&mut Body> {
        self.bodies.get_mut(self.current)
    }

    pub fn last(&self) -> Option<&Body> { self.bodies.last() }

    pub fn last_mut(&mut self) -> Option<&mut Body> { self.bodies.last_mut() }

    pub fn current_index(&self) -> usize {
        self.current
    }

    pub fn update_frame(&mut self) -> &mut Self {
        self.frame.next();
        self.clear_origin();
        self.clear_barycenter()
    }

    pub fn update_current_index(&mut self, increase: bool, bypass_last: bool) -> &mut Self {
        if increase {
            self.increase_current(bypass_last);
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
        T: FnMut(&Cluster, usize) -> Vector4 {
        let len = self.bodies.len();
        let mut acceleration;
        let mut state;
        let mut k1;
        let mut k2;
        let mut k3;
        let mut k4;

        self.deframe();
        for _ in 0..iterations {
            for i in 0..len {
                k1 = f(self, i);
                state = self.bodies[i].shape.center.state();
                self.bodies[i].shape.center.set_state(&(k1 * 0.5 * dt + state));
                k2 = f(self, i);
                self.bodies[i].shape.center.set_state(&(k2 * 0.5 * dt + state));
                k3 = f(self, i);
                self.bodies[i].shape.center.set_state(&(k3 * dt + state));
                k4 = f(self, i);
                self.bodies[i].shape.center.set_state(&state);
                acceleration = (k1 + (k2 + k3) * 2. + k4) * (1. / 6.);
                self.bodies[i].shape.center.acceleration = acceleration;
            }
            self.accelerate(dt);
        }
        self.clear_barycenter();
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
        self.barycenter.shape.center.update_trajectory();
        for body in self.bodies.iter_mut() {
            body.shape.center.update_trajectory();
        }
        self
    }

    pub fn clear_trajectory(&mut self) -> &mut Self {
        self.barycenter.shape.center.clear_trajectory();
        for body in self.bodies.iter_mut() {
            body.shape.center.clear_trajectory();
        }
        self
    }

    pub fn push(&mut self, body: Body) -> &mut Self {
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

    pub fn remove(&mut self, index: usize) -> Body {
        let len = self.bodies.len();
        if index == len - 1 {
            self.pop().unwrap()
        } else {
            if self.current == len - 1 {
                self.current -= 1;
            }
            self.bodies.remove(index)
        }
    }

    pub fn wait_drop(&mut self, cursor: &[f64; 2], middle: &Vector2, scale: f64) -> &mut Self {
        let last = self.bodies.len() - 1;
        self.bodies[last].shape.set_cursor_pos(cursor, middle, scale);
        self.bodies[last].shape.center.clear_trajectory();
        self.clear_barycenter();
        self
    }

    pub fn wait_speed(&mut self, cursor: &[f64; 2], middle: &Vector2, scale: f64) -> &mut Self {
        let last = self.bodies.len() - 1;
        self.bodies[last].shape.set_cursor_speed(cursor, middle, scale);
        self.bodies[last].shape.center.clear_trajectory();
        self.clear_barycenter();
        self
    }

    fn decrease_current(&mut self) -> &mut Self {
        if self.current > 0 {
            self.current -= 1;
        }
        self
    }

    fn increase_current(&mut self, bypass_last: bool) -> &mut Self {
        let offset = if bypass_last { 2 } else { 1 };
        if self.current < self.count() - offset {
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
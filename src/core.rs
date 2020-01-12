use std::cmp::{max, min};
use std::error::Error;

use getopts::Options;
use physics::dynamics::{Cluster, orbital};
use physics::dynamics::orbital::Body;
use physics::dynamics::point::Point3;
use physics::dynamics::solver::{Method, Solver};
use physics::geometry::point;
use physics::geometry::point::ZERO;
use physics::geometry::vector::Vector6;
use piston::input::{Key, MouseButton};
use piston::window::Size;
use rand::Rng;

use crate::common::*;
use crate::keys::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum State {
    Move,
    Translate,
    Add,
    Remove,
    WaitDrop,
    WaitSpeed,
    CancelDrop,
    Reset,
}

impl State {
    pub fn next(&mut self, key: &Key, button: &MouseButton) {
        use State::*;

        if *key == KEY_RESET {
            *self = Reset;
            return;
        }

        *self = match self {
            Reset => Move,
            Add => WaitDrop,
            Remove => Move,
            CancelDrop => Move,
            Move => if *button == MOUSE_MOVE_ADD {
                Add
            } else if *button == MOUSE_MOVE_REMOVE {
                Remove
            } else if *key == KEY_TOGGLE_TRANSLATE {
                Translate
            } else {
                *self
            },
            Translate => if *key == KEY_TOGGLE_TRANSLATE {
                Move
            } else {
                *self
            },
            WaitDrop => if *button == MOUSE_WAIT_DROP_DO {
                WaitSpeed
            } else if *button == MOUSE_WAIT_DROP_CANCEL {
                CancelDrop
            } else {
                *self
            }
            WaitSpeed => if *button == MOUSE_WAIT_DROP_DO {
                Move
            } else if *button == MOUSE_WAIT_DROP_CANCEL {
                WaitDrop
            } else {
                *self
            }
        };
    }
}

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


#[derive(Debug)]
pub struct Config {
    pub path: Option<String>,
    pub size: Size,
    pub scale: Scale,
    pub oversampling: u32,
    pub orientation: Orientation,
    pub trajectory: bool,
    pub orbits: bool,
    pub pause: bool,
}

impl Config {
    pub fn new(path: Option<String>, size: Size, scale: Scale, oversampling: u32) -> Config {
        Config {
            path,
            size,
            scale,
            oversampling,
            orientation: Orientation::new(0., 0., 0.),
            trajectory: false,
            orbits: true,
            pause: true,
        }
    }

    pub fn from_args(args: Vec<String>) -> Result<Config, Box<dyn Error>> {
        let mut opts = Options::new();
        opts.optopt("o", "orbital", "Loads an orbital cluster from file", "FILEPATH");
        opts.optopt("d", "distance", "Sets the distance scale in px/meters", "NUMBER");
        opts.optopt("t", "time", "Sets the time scale in secs/real sec", "NUMBER");
        opts.optopt("s", "oversampling", "Sets oversampling", "NUMBER");
        opts.optopt("w", "width", "Sets window width", "NUMBER");
        opts.optopt("h", "height", "Sets window height", "NUMBER");
        let matches = opts.parse(&args[1..])?;

        let path = matches.opt_str("o");
        let mut scale = Scale::unit();
        let mut oversampling: u32 = DEFAULT_OVERSAMPLING;
        let mut size = Size::from(DEFAULT_WINDOW_SIZE);

        if let Some(distance_str) = matches.opt_str("d") {
            scale.distance = distance_str.parse()?;
        }
        if let Some(time_str) = matches.opt_str("t") {
            scale.time = time_str.parse()?;
        }
        if let Some(oversampling_str) = matches.opt_str("s") {
            oversampling = oversampling_str.parse()?;
        }
        if let Some(width_str) = matches.opt_str("w") {
            size.width = width_str.parse()?;
        }
        if let Some(height_str) = matches.opt_str("h") {
            size.height = height_str.parse()?;
        }
        Ok(Config::new(path, size, scale, oversampling))
    }

    pub fn default() -> Config {
        Config::new(None, Size::from(DEFAULT_WINDOW_SIZE), Scale::unit(), DEFAULT_OVERSAMPLING)
    }

    pub fn update(&mut self, key: &Key) {
        if *key == KEY_TOGGLE_TRAJECTORY {
            self.trajectory = !self.trajectory;
        } else if *key == KEY_TOGGLE_PAUSE {
            self.pause = !self.pause;
        } else if *key == KEY_TOGGLE_ORBITS {
            self.orbits = !self.orbits;
        } else if *key == KEY_INCREASE_OVERSAMPLING {
            self.increase_oversampling();
        } else if *key == KEY_DECREASE_OVERSAMPLING {
            self.decrease_oversampling();
        } else if *key == KEY_ROTATION_DOWN {
            self.orientation.increment_x();
        } else if *key == KEY_ROTATION_UP {
            self.orientation.decrement_x();
        } else if *key == KEY_ROTATION_LEFT {
            self.orientation.increment_z();
        } else if *key == KEY_ROTATION_RIGHT {
            self.orientation.decrement_z();
        } else if *key == KEY_INCREASE_DISTANCE {
            self.scale.increase_distance();
        } else if *key == KEY_DECREASE_DISTANCE {
            self.scale.decrease_distance();
        } else if *key == KEY_INCREASE_TIME {
            self.scale.increase_time();
        } else if *key == KEY_DECREASE_TIME {
            self.scale.decrease_time();
        }
    }

    fn increase_oversampling(&mut self) {
        self.oversampling = min(self.oversampling << 1, std::u32::MAX);
    }

    fn decrease_oversampling(&mut self) {
        self.oversampling = max(self.oversampling >> 1, std::u32::MIN + 1);
    }
}

#[derive(Clone, Debug)]
pub struct Status {
    pub direction: Direction,
    pub reset_circles: bool,
    pub update_transform: bool,
    pub state: State,
    pub step: Step,
}

impl Status {
    pub fn new() -> Status {
        Status {
            direction: Direction::Hold,
            reset_circles: true,
            update_transform: true,
            state: State::Reset,
            step: Step::new(),
        }
    }

    pub fn is_waiting_to_add(&self) -> bool {
        self.state == State::WaitSpeed || self.state == State::WaitDrop
    }

    pub fn update(&mut self, key: &Option<Key>, button: &Option<MouseButton>) {
        match key {
            None => {
                self.direction = HOLD;
                match button {
                    None => self.state.next(&KEY_UNKNOWN, &BUTTON_UNKNOWN),
                    Some(button) => self.state.next(&KEY_UNKNOWN, button),
                };
            }
            Some(key) => {
                self.reset_circles = true;
                self.update_transform = true;
                self.direction = Direction::from(key);
                match button {
                    None => self.state.next(key, &BUTTON_UNKNOWN),
                    Some(button) => self.state.next(key, button),
                };
            }
        };
    }

    pub fn clear(&mut self) {
        self.state.next(&KEY_UNKNOWN, &BUTTON_UNKNOWN);
        self.direction = Direction::from(&KEY_UNKNOWN);
        self.reset_circles = false;
        self.update_transform = false;
    }
}

pub struct Simulator {
    pub cluster: Cluster,
    pub system: orbital::Cluster,
    pub current: usize,
    pub origin: point::Point3,
    pub frame: Frame,
    pub solver: Solver,
    pub stats: Statistics,
}

impl From<Cluster> for Simulator {
    fn from(cluster: Cluster) -> Self {
        let bodies: Vec<Body> = cluster.points.iter()
            .map(|_point| orbital::Body::new())
            .collect();
        Simulator::new(cluster, orbital::Cluster::from(bodies), Solver::from(Method::RungeKutta4))
    }
}

impl Simulator {
    pub fn new(cluster: Cluster, system: orbital::Cluster, solver: Solver) -> Self {
        Simulator {
            cluster,
            system,
            current: 0,
            origin: point::ZERO,
            frame: Frame::Zero,
            solver,
            stats: Statistics::new(),
        }
    }

    pub fn orbital(system: orbital::Cluster, true_anomalies: Vec<f64>, solver: Solver) -> Self {
        let mut points: Vec<Point3> = Vec::with_capacity(system.bodies.len());
        for i in 0..system.bodies.len() {
            points.push(Point3::inertial(
                system[i].orbit.position_at(true_anomalies[i]),
                system[i].orbit.speed_at(true_anomalies[i]),
                system[i].mass,
            ));
        }
        Simulator::new(Cluster::new(points), system, solver)
    }

    pub fn orbital_at(system: orbital::Cluster, true_anomaly: f64, solver: Solver) -> Self {
        let mut true_anomalies = Vec::with_capacity(system.bodies.len());
        for _ in system.bodies.iter() {
            true_anomalies.push(true_anomaly)
        }
        Simulator::orbital(system, true_anomalies, solver)
    }

    pub fn orbital_at_random(system: orbital::Cluster, solver: Solver) -> Self {
        let two_pi = 2. * std::f64::consts::PI;
        let mut rng = rand::thread_rng();
        let mut true_anomalies: Vec<f64> = Vec::with_capacity(system.bodies.len());
        for _ in system.bodies.iter() {
            true_anomalies.push(rng.gen_range(0., two_pi))
        }
        Simulator::orbital(system, true_anomalies, solver)
    }

    #[inline]
    pub fn current(&self) -> Option<&Point3> { self.cluster.points.get(self.current) }

    #[inline]
    pub fn current_mut(&mut self) -> Option<&mut Point3> { self.cluster.points.get_mut(self.current) }

    #[inline]
    pub fn current_index(&self) -> usize { self.current }

    #[inline]
    pub fn last(&self) -> Option<&Point3> { self.cluster.points.last() }

    #[inline]
    pub fn last_mut(&mut self) -> Option<&mut Point3> { self.cluster.points.last_mut() }

    #[inline]
    pub fn last_index(&self) -> usize { self.cluster.len() - 1 }

    pub fn update(&mut self, key: &Option<Key>, bypass_last: bool) -> &mut Self {
        if let Some(key) = key {
            if *key == KEY_NEXT_METHOD_STATE {
                self.solver.method.next();
            } else if *key == KEY_NEXT_FRAME_STATE {
                self.next_frame();
            } else if *key == KEY_INCREASE_CURRENT_INDEX {
                self.increment_current(bypass_last);
            } else if *key == KEY_DECREASE_CURRENT_INDEX {
                self.decrement_current();
            }
        }
        self
    }

    pub fn remove_aways(&mut self) -> Option<usize> {
        self.stats.update(&self.cluster, None);
        let max_distance = self.stats.max_distance;
        let max_index = self.stats.max_index;
        if self.cluster.len() < 4 {
            return None;
        }
        self.stats.update(&self.cluster, Some(self.stats.max_index));
        if max_distance > self.stats.mean + 10e2 * self.stats.deviation {
            self.remove(max_index);
            return Some(max_index);
        }
        return None;
    }

    #[inline]
    pub fn apply<T>(&mut self, dt: f64, iterations: u32, f: T) -> &mut Self where
        T: FnMut(&Vec<Point3>, usize) -> Vector6 {
        self.solver.dt = dt;
        self.solver.iterations = iterations;
        self.cluster.set_absolute(&self.origin);
        self.cluster.apply(&mut self.solver, f);
        self.update_origin();
        self.origin.update_trajectory();
        self.cluster.set_relative(&self.origin);
        self.system.update_orbits(&self.cluster.points, self.cluster.barycenter());
        self
    }

    #[inline]
    pub fn push(&mut self, point: Point3, body: Body) -> &mut Self {
        self.cluster.push(point);
        self.system.push(body);
        self
    }

    #[inline]
    pub fn pop(&mut self) -> Option<Point3> {
        if self.current == self.cluster.len() - 1 {
            self.decrement_current();
            self.reset_origin();
        }
        self.system.pop();
        self.cluster.pop()
    }

    #[inline]
    pub fn remove(&mut self, i: usize) -> Point3 {
        if self.current == self.cluster.len() - 1 && i == self.current {
            self.decrement_current();
            self.reset_origin();
        }
        self.system.remove(i);
        self.cluster.remove(i)
    }

    #[inline]
    fn next_frame(&mut self) -> &mut Self {
        self.frame.next();
        self.reset_origin();
        self
    }

    #[inline]
    fn decrement_current(&mut self) -> &mut Self {
        if self.current > 0 {
            self.current -= 1;
        }
        if self.frame == Frame::Current {
            self.reset_origin();
        }
        self
    }

    #[inline]
    fn increment_current(&mut self, bypass_last: bool) -> &mut Self {
        let offset = if bypass_last { 2 } else { 1 };
        if self.current < self.cluster.len() - offset {
            self.current += 1;
        }
        if self.frame == Frame::Current {
            self.reset_origin();
        }
        self
    }

    #[inline]
    fn update_origin(&mut self) -> &mut Self {
        self.origin.position = self.current_origin().position;
        self.origin.speed = self.current_origin().speed;
        self
    }

    #[inline]
    fn current_origin(&mut self) -> &point::Point3 {
        if self.cluster.is_empty() {
            return &point::ZERO;
        }
        match self.frame {
            Frame::Zero => &point::ZERO,
            Frame::Current => &self.cluster.points[self.current].state,
            Frame::Barycenter => &self.cluster.barycenter().state,
        }
    }

    #[inline]
    fn reset_origin(&mut self) -> &mut Self {
        if self.cluster.is_empty() {
            return self;
        }
        let origin = *self.current_origin();
        self.cluster.reset_origin(&origin, &ZERO);
        self.origin = origin;
        self
    }
}
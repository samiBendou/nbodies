use std::fmt;
use std::fmt::Debug;

use physics::dynamics::{Body, Cluster, orbital};
use physics::geometry::common::{Array, Initializer};
use physics::geometry::point::Point3;
use physics::geometry::trajectory::TRAJECTORY_SIZE;
use physics::geometry::vector::{Vector2, Vector3, ZERO};
use physics::geometry::vector::transforms::Cartesian2;
use physics::units::{Rescale, Scale, Serialize, Unit};
use physics::units::suffix::*;
use piston::window::Size;
use piston_window::*;
use piston_window::context::Context;

const SCALE_LENGTH: f64 = 50.;

pub const BLACK: [f32; 4] = [0., 0., 0., 1.];
pub const WHITE: [f32; 4] = [1., 1., 1., 1.];
pub const RED: [f32; 4] = [1., 0., 0., 1.];
const BLUE: [f32; 4] = [0., 0., 1., 1.];

#[derive(Copy, Clone)]
pub struct Circle {
    pub center: Point3,
    pub color: [f32; 4],
    pub radius: f64,
    pub rect: [f64; 4],
}

impl Circle {
    pub fn new(center: Point3, radius: f64, color: [f32; 4]) -> Circle {
        Circle {
            center,
            color,
            radius,
            rect: [0.; 4],
        }
    }


    pub fn from_body(body: &Body, color: [f32; 4], middle: &Vector2, scale: f64) -> Circle {
        let mut ret = Circle::centered(0., color);
        *ret.reset(&body, middle, scale)
    }

    pub fn centered(radius: f64, color: [f32; 4]) -> Circle {
        Circle::new(Point3::zeros(), radius, color)
    }

    pub fn update_rect(&mut self) -> &mut Self {
        let diameter = 2. * self.radius;
        self.rect[0] = self.center.position.x - self.radius;
        self.rect[1] = self.center.position.y - self.radius;
        self.rect[2] = diameter;
        self.rect[3] = diameter;
        self
    }

    pub fn bound(&mut self, middle: &Vector3) -> &mut Circle {
        let x_left = -self.radius - middle.x;
        let x_right = self.radius + middle.x;
        let y_up = self.radius + middle.y;
        let y_down = -self.radius - middle.y;

        if self.center.position.x < x_left {
            self.center.position.x = x_right;
        } else if self.center.position.x > x_right {
            self.center.position.x = x_left;
        }

        if self.center.position.y < y_down {
            self.center.position.y = y_up;
        } else if self.center.position.y > y_up {
            self.center.position.y = y_down;
        }

        self
    }

    pub fn reset(&mut self, body: &Body, middle: &Vector2, scale: f64) -> &mut Self {
        self.center.trajectory = body.center.state.trajectory;
        for i in 0..TRAJECTORY_SIZE {
            self.center.trajectory.position_mut(i).set_left_up(middle, scale);
        }
        self.update(body, middle, scale);
        self
    }

    pub fn update(&mut self, body: &Body, middle: &Vector2, scale: f64) -> &mut Self {
        self.center.position = body.center.state.position;
        self.center.speed = body.center.state.speed;
        self.center.position.set_left_up(middle, scale);
        self.center.speed.set_left_up(&ZERO, scale);
        self.update_rect();
        self
    }
}

impl Debug for Circle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self.center)
    }
}

pub struct Drawer {
    pub circles: Vec<Circle>,
    pub middle: Vector2,
    offset: Vector2,
    color: [f32; 4],
    unit: Unit,
}


impl Drawer {
    pub fn new(size: &Size, cluster: &Cluster, scale: f64) -> Drawer {
        let middle = Vector2::new(size.width, size.height) * 0.5;
        let mut circles: Vec<Circle> = Vec::with_capacity(cluster.len());
        for body in cluster.bodies.iter() {
            circles.push(Circle::from_body(body, BLUE, &middle, scale))
        }
        Drawer {
            circles,
            middle,
            offset: Vector2::zeros(),
            color: BLACK,
            unit: Unit::from(Scale::from(Distance::Meter)),
        }
    }

    pub fn set_appearance(&mut self, cluster: &orbital::Cluster) -> &mut Self {
        let len = self.circles.len();
        for i in 0..len {
            self.circles[i].color = cluster.bodies[i].color;
            self.circles[i].radius = cluster.bodies[i].kind.scaled_radius(cluster.bodies[i].radius);
        }
        self
    }

    pub fn set_size(&mut self, size: &Size) {
        self.middle.x = 0.5 * size.width;
        self.middle.y = 0.5 * size.height;
    }

    pub fn update_circles(&mut self, cluster: &Cluster, scale: f64) -> &mut Self {
        let len = self.circles.len();
        for i in 0..len {
            self.circles[i].update(&cluster[i], &self.middle, scale);
        }
        self
    }

    pub fn update_circles_trajectory(&mut self) -> &mut Self {
        for circle in self.circles.iter_mut() {
            circle.center.trajectory.push(&circle.center.position)
        }
        self
    }

    pub fn reset_circles(&mut self, cluster: &Cluster, scale: f64) -> &mut Self {
        let len = self.circles.len();
        for i in 0..len {
            self.circles[i].reset(&cluster[i], &self.middle, scale);
        }
        self
    }

    pub fn push(&mut self, body: &Body, color: [f32; 4], scale: f64) {
        self.circles.push(Circle::from_body(body, color, &self.middle, scale));
    }

    pub fn pop(&mut self) -> Option<Circle> {
        self.circles.pop()
    }

    pub fn draw_scale(&mut self, scale: f64, c: &Context, g: &mut G2d, glyphs: &mut Glyphs) {
        let scale_distance = SCALE_LENGTH / scale;

        self.offset = self.middle * 2.;
        self.offset.x -= 160.;
        self.offset.y -= 48.;
        self.unit.rescale(&scale_distance);

        piston_window::line_from_to(
            WHITE,
            3.,
            [self.offset.x, self.offset.y],
            [self.offset.x + SCALE_LENGTH, self.offset.y],
            c.transform, g,
        );

        piston_window::text::Text::new_color(WHITE, 16).draw(
            format!("{}", self.unit.string_of(&scale_distance)).as_str(),
            glyphs,
            &c.draw_state,
            c.transform.trans(self.offset.x, self.offset.y - 16.),
            g,
        ).unwrap();
    }

    pub fn draw_barycenter(&mut self, position: &Vector3, scale: f64, c: &Context, g: &mut G2d) {
        let barycenter = position.left_up(&self.middle, scale);
        piston_window::rectangle(
            RED,
            [barycenter.x - 4., barycenter.y - 4., 8., 8.],
            c.transform, g,
        );
    }

    pub fn draw_bodies(&mut self, c: &Context, g: &mut G2d) {
        let len = self.circles.len();
        for i in 0..len {
            piston_window::ellipse(
                self.circles[i].color,
                self.circles[i].rect,
                c.transform, g,
            );
        }
    }

    pub fn draw_trajectories(&mut self, c: &Context, g: &mut G2d) {
        let len = self.circles.len();
        let mut from;
        let mut to;
        for i in 0..len {
            self.color = self.circles[i].color;
            for k in 1..TRAJECTORY_SIZE - 1 {
                from = self.circles[i].center.trajectory.position(k - 1).array();
                to = self.circles[i].center.trajectory.position(k).array();
                self.color[3] = k as f32 / (TRAJECTORY_SIZE as f32 - 1.);
                piston_window::line_from_to(
                    self.color,
                    2.5,
                    [from[0], from[1]],
                    [to[0], to[1]],
                    c.transform, g,
                );
            }
        }
    }

    pub fn draw_speed(&mut self, cursor: &[f64; 2], c: &Context, g: &mut G2d) {
        let last = self.circles.last().unwrap();
        let last_pos = last.center.position.array();
        piston_window::line_from_to(
            last.color,
            2.5,
            [last_pos[0], last_pos[1]],
            *cursor,
            c.transform, g,
        );
    }
}
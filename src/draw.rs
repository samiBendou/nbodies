use std::fmt;
use std::fmt::Debug;

use physics::dynamics::{Cluster, orbital};
use physics::geometry::common::{Array, Initializer, Metric};
use physics::geometry::common::coordinates::{Cartesian2, Cartesian3, Cartesian4};
use physics::geometry::common::coordinates::Homogeneous;
use physics::geometry::common::transforms::{Rotation3, Similarity};
use physics::geometry::matrix::{Algebra, Matrix3, Matrix4};
use physics::geometry::trajectory::{Trajectory3, Trajectory4, TRAJECTORY_SIZE};
use physics::geometry::vector::{Vector2, Vector3, Vector4};
use physics::units::{Rescale, Scale, Serialize, Unit};
use physics::units::suffix::*;
use piston::window::Size;
use piston_window::*;
use piston_window::context::Context;

use crate::core::Orientation;

const SCALE_LENGTH: f64 = 50.;

pub const BLACK: [f32; 4] = [0., 0., 0., 1.];
pub const WHITE: [f32; 4] = [1., 1., 1., 1.];
pub const RED: [f32; 4] = [1., 0., 0., 1.];
pub const GREEN: [f32; 4] = [0., 1., 0., 1.];
const BLUE: [f32; 4] = [0., 0., 1., 1.];

#[derive(Copy, Clone)]
pub struct Circle {
    pub trajectory: Trajectory4,
    pub color: [f32; 4],
    pub radius: f64,
    pub rect: [f64; 4],
}

impl Circle {
    pub fn new(trajectory: Trajectory4, radius: f64, color: [f32; 4]) -> Circle {
        Circle {
            trajectory,
            color,
            radius,
            rect: [0.; 4],
        }
    }

    pub fn centered(radius: f64, color: [f32; 4]) -> Circle {
        Circle::new(Trajectory4::from(Vector4::unit_w()), radius, color)
    }

    pub fn reset(&mut self, trajectory: &Trajectory3, transform: &Matrix4) -> &mut Self {
        let mut position;
        for i in 0..TRAJECTORY_SIZE {
            position = self.trajectory.position_mut(i);
            *position = trajectory.position(i).homogeneous();
            *position *= *transform;
        }
        self
    }

    pub fn update(&mut self, position: &Vector3, transform: &Matrix4) -> &mut Self {
        self.trajectory.push(&position.homogeneous());
        *self.trajectory.last_mut() *= *transform;
        self
    }

    fn update_rect(&mut self) -> &mut Self {
        let diameter = 2. * self.radius;
        let last = self.trajectory.last();
        self.rect[0] = last.x - self.radius;
        self.rect[1] = last.y - self.radius;
        self.rect[2] = diameter;
        self.rect[3] = diameter;
        self
    }
}

impl Debug for Circle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self.trajectory)
    }
}

pub struct Drawer {
    pub circles: Vec<Circle>,
    offset: Vector2,
    color: [f32; 4],
    unit: Unit,
    u_x: Vector4,
    u_y: Vector4,
    u_z: Vector4,
    pub transform: Matrix4,
    pub inverse_transform: Matrix4,
}


impl Drawer {
    pub fn new(size: &Size, cluster: &Cluster, scale: f64) -> Drawer {
        let scale_distance = SCALE_LENGTH / scale;
        let rotation = Matrix3::from_rotation_x(std::f64::consts::PI);
        let middle = Vector3::new(size.width * 0.5, size.height * 0.5, 0.);
        let transform = Matrix4::from_similarity(scale, &rotation, &middle);
        let u_x = transform * (Vector3::unit_x() * scale_distance).homogeneous();
        let u_y = transform * (Vector3::unit_y() * scale_distance).homogeneous();
        let u_z = transform * (Vector3::unit_z() * scale_distance).homogeneous();
        let circles: Vec<Circle> = cluster.bodies.iter()
            .map({ |_body| Circle::centered(0., BLUE) })
            .collect();
        Drawer {
            circles,
            offset: Vector2::zeros(),
            color: BLACK,
            unit: Unit::from(Scale::from(Distance::Meter)),
            u_x,
            u_y,
            u_z,
            transform,
            inverse_transform: transform.inverse(),
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

    pub fn update_transform(&mut self, orientation: &Orientation, scale: f64, size: &Size) -> &mut Self {
        let scale_distance = SCALE_LENGTH / scale;
        let middle = Vector3::new(size.width * 0.5, size.height * 0.5, 0.);
        let rotation = Matrix3::from_rotation_x(std::f64::consts::PI) * orientation.rotation();
        self.transform.set_similarity(scale, &rotation, &middle);
        self.inverse_transform = self.transform.inverse();
        self.u_x = self.transform * (Vector3::unit_x() * scale_distance).homogeneous();
        self.u_y = self.transform * (Vector3::unit_y() * scale_distance).homogeneous();
        self.u_z = self.transform * (Vector3::unit_z() * scale_distance).homogeneous();
        self
    }

    pub fn update_circles(&mut self, cluster: &Cluster) -> &mut Self {
        let len = self.circles.len();
        for i in 0..len {
            self.circles[i].update(&cluster[i].center.state.position, &self.transform);
        }
        self
    }

    pub fn reset_circles(&mut self, cluster: &Cluster) -> &mut Self {
        let len = self.circles.len();
        for i in 0..len {
            self.circles[i].reset(&cluster[i].center.state.trajectory, &self.transform);
        }
        self
    }

    pub fn draw_scale(&mut self, scale: f64, size: &Size, c: &Context, g: &mut G2d, glyphs: &mut Glyphs) {
        let scale_distance = SCALE_LENGTH / scale;
        self.offset.x = size.width - 160.;
        self.offset.y = size.height - 48.;
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

        self.offset.x = size.width * 0.5;
        self.offset.y = size.height * 0.5;

        piston_window::line_from_to(
            RED,
            3.,
            [self.offset.x, self.offset.y],
            [self.u_x.x, self.u_x.y],
            c.transform, g,
        );

        piston_window::line_from_to(
            GREEN,
            3.,
            [self.offset.x, self.offset.y],
            [self.u_y.x, self.u_y.y],
            c.transform, g,
        );

        piston_window::line_from_to(
            BLUE,
            3.,
            [self.offset.x, self.offset.y],
            [self.u_z.x, self.u_z.y],
            c.transform, g,
        );
    }

    pub fn draw_barycenter(&mut self, position: &Vector3, c: &Context, g: &mut G2d) {
        let mut barycenter = position.homogeneous();
        barycenter *= self.transform;
        piston_window::rectangle(
            RED,
            [barycenter.x - 4., barycenter.y - 4., 8., 8.],
            c.transform, g,
        );
    }

    pub fn draw_bodies(&mut self, c: &Context, g: &mut G2d) {
        let len = self.circles.len();
        for i in 0..len {
            self.circles[i].update_rect();
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
                from = self.circles[i].trajectory.position(k - 1).array();
                to = self.circles[i].trajectory.position(k).array();
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
        let last_pos = last.trajectory.last().array();
        piston_window::line_from_to(
            last.color,
            2.5,
            [last_pos[0], last_pos[1]],
            *cursor,
            c.transform, g,
        );
    }
}
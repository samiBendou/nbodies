use std::fmt;
use std::fmt::Debug;

use dynamics::{Cluster, orbital};
use geomath::common::coordinates::{Cartesian2, Cartesian3, Cartesian4};
use geomath::common::coordinates::Homogeneous;
use geomath::common::Initializer;
use geomath::common::transforms::{Rotation3, Similarity};
use geomath::matrix::{Algebra, Matrix3, Matrix4};
use geomath::trajectory::{Trajectory3, Trajectory4, TRAJECTORY_SIZE};
use geomath::vector::{Vector2, Vector3, Vector4};
use piston::window::Size;
use piston_window::*;
use piston_window::context::Context;
use unitflow::{Rescale, Scale, Serialize, Unit};
use unitflow::suffix::*;

use crate::common::{BLACK, BLUE, GREEN, RED, WHITE};
use crate::common::Orientation;
use crate::core::Simulator;

const SCALE_LENGTH: f64 = 50.;

#[derive(Copy, Clone)]
pub struct Circle {
    pub trajectory: Trajectory3,
    pub color: [f32; 4],
    pub radius: f64,
    pub rect: [f64; 4],
}

impl Circle {
    pub fn new(trajectory: Trajectory3, radius: f64, color: [f32; 4]) -> Circle {
        Circle {
            trajectory,
            color,
            radius,
            rect: [0.; 4],
        }
    }

    pub fn centered(radius: f64, color: [f32; 4]) -> Circle {
        Circle::new(Trajectory3::from(Vector3::zeros()), radius, color)
    }

    pub fn reset(&mut self, trajectory: &Trajectory3, transform: &Matrix4) -> &mut Self {
        let mut position;
        for i in 0..TRAJECTORY_SIZE {
            position = self.trajectory.position_mut(i);
            *position *= *transform;
        }
        self
    }

    pub fn update(&mut self, position: &Vector3, transform: &Matrix4) -> &mut Self {
        self.trajectory.push(&position);
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
    buffer_offset: Vector2,
    buffer_color: [f32; 4],
    distance_unit: Unit,
    unit_x: Vector3,
    unit_y: Vector3,
    unit_z: Vector3,
    pub transform: Matrix4,
    pub inverse_transform: Matrix4,
}


impl Drawer {
    pub fn new(cluster: &Cluster, orientation: &Orientation, scale: f64, size: &Size) -> Drawer {
        let circles: Vec<Circle> = cluster.points.iter()
            .map({ |_point| Circle::centered(10., BLUE) })
            .collect();
        let mut ret = Drawer {
            circles,
            buffer_offset: Vector2::zeros(),
            buffer_color: BLACK,
            distance_unit: Unit::from(Scale::from(Distance::Meter)),
            unit_x: Vector3::unit_x(),
            unit_y: Vector3::unit_y(),
            unit_z: Vector3::unit_z(),
            transform: Matrix4::eye(),
            inverse_transform: Matrix4::eye(),
        };
        ret.update_transform(orientation, scale, size);
        ret.reset_circles(cluster);
        ret
    }

    pub fn set_appearance(&mut self, cluster: &orbital::Cluster) -> &mut Self {
        for i in 0..self.circles.len() {
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
        self.unit_x = self.transform * (Vector3::unit_x() * scale_distance);
        self.unit_y = self.transform * (Vector3::unit_y() * scale_distance);
        self.unit_z = self.transform * (Vector3::unit_z() * scale_distance);
        self
    }

    pub fn update_circles(&mut self, cluster: &Cluster) -> &mut Self {
        let len = self.circles.len();
        for i in 0..len {
            self.circles[i].update(&cluster[i].state.position, &self.transform);
        }
        self
    }

    pub fn reset_circles(&mut self, cluster: &Cluster) -> &mut Self {
        let len = self.circles.len();
        for i in 0..len {
            self.circles[i].reset(&cluster[i].state.trajectory, &self.transform);
        }
        self
    }

    pub fn draw_scale(&mut self, scale: f64, size: &Size, c: &Context, g: &mut G2d, glyphs: &mut Glyphs) {
        let scale_distance = SCALE_LENGTH / scale;
        self.buffer_offset.x = size.width - 160.;
        self.buffer_offset.y = size.height - 48.;
        self.distance_unit.rescale(&scale_distance);

        piston_window::line_from_to(
            WHITE,
            3.,
            [self.buffer_offset.x, self.buffer_offset.y],
            [self.buffer_offset.x + SCALE_LENGTH, self.buffer_offset.y],
            c.transform, g,
        );

        piston_window::text::Text::new_color(WHITE, 16).draw(
            format!("{}", self.distance_unit.string_of(&scale_distance)).as_str(),
            glyphs,
            &c.draw_state,
            c.transform.trans(self.buffer_offset.x, self.buffer_offset.y - 16.),
            g,
        ).unwrap();
    }

    pub fn draw_basis(&mut self, size: &Size, c: &Context, g: &mut G2d) {
        self.buffer_offset.x = size.width * 0.5;
        self.buffer_offset.y = size.height * 0.5;

        piston_window::line_from_to(
            RED,
            3.,
            [self.buffer_offset.x, self.buffer_offset.y],
            [self.unit_x.x, self.unit_x.y],
            c.transform, g,
        );

        piston_window::line_from_to(
            GREEN,
            3.,
            [self.buffer_offset.x, self.buffer_offset.y],
            [self.unit_y.x, self.unit_y.y],
            c.transform, g,
        );

        piston_window::line_from_to(
            BLUE,
            3.,
            [self.buffer_offset.x, self.buffer_offset.y],
            [self.unit_z.x, self.unit_z.y],
            c.transform, g,
        );
    }

    pub fn draw_barycenter(&mut self, position: &Vector3, c: &Context, g: &mut G2d) {
        let mut barycenter = *position;
        barycenter *= self.transform;
        piston_window::rectangle(
            RED,
            [barycenter.x - 4., barycenter.y - 4., 8., 8.],
            c.transform, g,
        );
    }

    pub fn draw_points(&mut self, c: &Context, g: &mut G2d) {
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
        let mut from;
        let mut to;
        for i in 0..self.circles.len() {
            self.buffer_color = self.circles[i].color;
            for k in 1..TRAJECTORY_SIZE {
                from = &self.circles[i].trajectory[k - 1];
                to = &self.circles[i].trajectory[k];
                piston_window::line_from_to(
                    self.buffer_color,
                    2.5,
                    [from.x, from.y],
                    [to.x, to.y],
                    c.transform, g,
                );
            }
        }
    }

    pub fn draw_orbits(&mut self, simulator: &Simulator, c: &Context, g: &mut G2d) {
        let mut from;
        let mut to;
        let mut angle;
        let d_angle = 2. * std::f64::consts::PI / TRAJECTORY_SIZE as f64;
        for i in 0..self.circles.len() {
            angle = 0.;
            for _ in 0..TRAJECTORY_SIZE {
                from = self.transform * simulator.system[i].orbit.position_at(angle);
                to = self.transform * simulator.system[i].orbit.position_at(angle + d_angle);
                angle += d_angle;
                piston_window::line_from_to(
                    self.circles[i].color,
                    2.5,
                    [from.x, from.y],
                    [to.x, to.y],
                    c.transform, g,
                );
            }
        }
    }

    pub fn draw_speed(&mut self, cursor: &[f64; 2], c: &Context, g: &mut G2d) {
        let last = self.circles.last().unwrap();
        let last_pos = last.trajectory.last();
        piston_window::line_from_to(
            last.color,
            2.5,
            [last_pos.x, last_pos.y],
            *cursor,
            c.transform, g,
        );
    }
}
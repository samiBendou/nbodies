use piston::window::Size;
use piston_window::*;
use piston_window::context::Context;

use crate::physics::dynamics::{Body, VecBody};
use crate::physics::units;
use crate::physics::vector::Vector2;

pub mod ellipse;

const SCALE_LENGTH: f64 = 50.; // in px

pub struct Drawer {
    from: Vector2,
    to: Vector2,
    offset: Vector2,
    middle: Vector2,
    rect: [f64; 4],
    color: [f32; 4],
}


impl Drawer {
    pub fn new(size: &Size) -> Drawer {
        let middle = Vector2::new(size.width, size.height) * 0.5;
        Drawer {
            from: Vector2::zeros(),
            to: Vector2::zeros(),
            offset: Vector2::zeros(),
            middle,
            rect: [0.; 4],
            color: [0.; 4],
        }
    }

    pub fn middle(&self) -> &Vector2 {
        &self.middle
    }

    pub fn update_middle(&mut self, size: &Size) {
        self.middle.x = 0.5 * size.width;
        self.middle.y = 0.5 * size.height;
    }

    pub fn draw_scale(&mut self, scale: f64, c: &Context, g: &mut G2d, glyphs: &mut Glyphs) {
        let scale_distance = SCALE_LENGTH / scale;
        let prefix = units::prefix::Scale::from(scale_distance);
        self.offset = self.middle * 2.;
        self.offset.x -= 160.;
        self.offset.y -= 48.;

        piston_window::line_from_to(
            [0., 0., 0., 1.],
            3.,
            [self.offset.x, self.offset.y],
            [self.offset.x + SCALE_LENGTH, self.offset.y],
            c.transform, g,
        );

        piston_window::text::Text::new_color([0.0, 0.0, 0.0, 1.0], 16).draw(
            format!("{:.3} ({}m)", prefix.value_of(scale_distance), prefix.label).as_str(),
            glyphs,
            &c.draw_state,
            c.transform.trans(self.offset.x, self.offset.y - 16.),
            g,
        ).unwrap();
    }

    pub fn draw_barycenter(&mut self, barycenter: &Body, scale: f64, c: &Context, g: &mut G2d) {
        let mut barycenter_rect = barycenter.shape.center.position * scale;
        barycenter_rect.y = -barycenter_rect.y;
        barycenter_rect += self.middle;
        piston_window::rectangle(
            [255., 0., 0., 1.],
            [barycenter_rect.x - 4., barycenter_rect.y - 4., 8., 8.],
            c.transform, g,
        );
    }

    pub fn draw_bodies(&mut self, bodies: &VecBody, scale: f64, c: &Context, g: &mut G2d) {
        let size = Size::from((self.middle * 2.).as_array());
        self.offset = self.middle * 2.;
        for i in 0..bodies.count() {
            self.rect = bodies[i].shape.rounding_rect(&self.middle, scale);
            piston_window::ellipse(
                bodies[i].shape.color,
                self.rect,
                c.transform, g,
            );
        }
    }

    pub fn draw_trajectories(&mut self, bodies: &VecBody, scale: f64, c: &Context, g: &mut G2d) {
        use crate::physics::dynamics::TRAJECTORY_SIZE;
        use crate::shapes::ellipse;

        for i in 0..bodies.count() {
            self.color = bodies[i].shape.color;
            for k in 1..TRAJECTORY_SIZE - 1 {
                self.color[3] = k as f32 / (TRAJECTORY_SIZE as f32 - 1.);
                self.from = *bodies[i].shape.center.position(k - 1) * scale;
                self.to = *bodies[i].shape.center.position(k) * scale;
                ellipse::Circle::set_left_up(&mut self.from, &self.middle);
                ellipse::Circle::set_left_up(&mut self.to, &self.middle);
                piston_window::line_from_to(
                    self.color,
                    2.5,
                    self.from.as_array(),
                    self.to.as_array(),
                    c.transform, g,
                );
            }
        }
    }
}
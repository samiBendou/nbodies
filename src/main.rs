extern crate opengl_graphics;
extern crate piston_window;

use opengl_graphics::OpenGL;
use piston::input::{Button, Event, Key, PressEvent, RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::{Window, WindowSettings};
use piston_window::*;

use piston_start::common::*;
use piston_start::shape::*;

pub struct App {
    window: PistonWindow,
    circle: Circle,
    direction: Direction,
    outputs_log: bool,
    outputs_dt: bool,
    bounded: bool,
    translate: bool,
    reset: bool,
}

impl App {
    fn new(width: f64, height: f64) -> App {
        let opengl = OpenGL::V3_2;
        let window =
            WindowSettings::new("Circle Keeps Moving Like a Rollin' Stone!", [width, height])
                .exit_on_esc(true)
                .resizable(false)
                .graphics_api(opengl)
                .build()
                .unwrap();

        App {
            window,
            circle: Circle::new(width / 2., height / 2., 50., Color::Blue),
            direction: Direction::Hold,
            outputs_log: true,
            outputs_dt: false,
            bounded: true,
            translate: false,
            reset: false,
        }
    }

    fn render(&mut self, _args: &RenderArgs, event: &Event) {
        let color = self.circle.color.rgba_array();
        let rect = self.circle.rounding_rect();

        self.window.draw_2d(event, |c, g, _device| {
            clear([1.0; 4], g);
            ellipse(color, rect, c.transform, g);
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        let window_size = self.window.size();

        if self.translate {
            self.circle.translate(&self.direction);
        } else {
            self.circle.update(&self.direction, args.dt);
        }
        if self.bounded {
            self.circle.replace(window_size.width, window_size.height);
        }
        if self.reset {
            self.circle.reset(window_size.width / 2., window_size.height / 2.);
            self.reset = false;
        }
        self.direction = Direction::Hold;
    }

    fn on_key(&mut self, key: Key) {
        self.direction = Direction::from(key);
        match key {
            Key::L => self.outputs_log = !self.outputs_log,
            Key::T => self.outputs_dt = !self.outputs_dt,
            Key::D => self.outputs_dt = !self.outputs_dt,
            Key::B => self.bounded = !self.bounded,
            Key::Space => self.reset = true,
            _ => (),
        };
    }

}


fn main() {
    let mut app = App::new(640., 640.);

    while let Some(event) = app.window.next() {

        if let Some(Button::Mouse(button)) = event.press_args() {
            if app.outputs_log {
                println!("Pressed mouse button '{:?}'", button);
            }
        }

        if let Some(Button::Keyboard(key)) = event.press_args() {
            app.on_key(key);
            if app.outputs_log {
                println!("Pressed keyboard key: '{:?}'", key);
            }
        };

        if let Some(args) = event.render_args() {
            app.render(&args, &event);

            if app.outputs_log {
                println!("Direction: {:?}", app.circle.center);
                println!("Speed: {:?}", app.circle.speed);
            }
        }

        if let Some(args) = event.update_args() {
            app.update(&args);

            if app.outputs_dt {
                println!("Time step dt: {:.2} sec.", args.dt);
                println!("Framerate: {:.2} fps", 1. / args.dt);
            }
        }
    }
}

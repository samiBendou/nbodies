extern crate opengl_graphics;
extern crate piston_window;

use opengl_graphics::OpenGL;
use piston::input::{Button, Event, Key, PressEvent, RenderArgs, RenderEvent, ResizeArgs, ResizeEvent, UpdateArgs, UpdateEvent};
use piston::window::{Window, WindowSettings};
use piston_window::*;

use piston_start::*;

pub struct App {
    window: PistonWindow,
    circle: Circle,
    outputs_log: bool,
}

impl App {
    fn new(width: f64, height: f64) -> App {
        let opengl = OpenGL::V3_2;
        let window =
            WindowSettings::new("Circle Keeps Moving Like a Rollin' Stone!", [width, height])
                .exit_on_esc(true)
                .fullscreen(true)
                .graphics_api(opengl)
                .build()
                .unwrap();
        let circle = Circle::new(width / 2., height / 2., 50., Color::Blue);
        let outputs_log = true;

        App { window, circle, outputs_log }
    }

    fn render(&mut self, args: &RenderArgs, event: &Event) {
        let color = self.circle.color.rgba_array();
        let rect = self.circle.rounding_rect();

        self.window.draw_2d(event, |c, g, _device| {
            clear([1.0; 4], g);
            ellipse(color, rect, c.transform, g);
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        let window_size = self.window.size();
        self.circle.update().replace(window_size.width, window_size.height);
    }

    fn on_key(&mut self, key: Key) {
        let direction = Direction::from(key);
        match key {
            Key::Left | Key::Right | Key::Up | Key::Down | Key::Space => {
                self.circle.translate(&direction);
            }
            Key::T => self.outputs_log = !self.outputs_log,
            _ => (),
        };
    }

    fn on_resize(&mut self, args: &ResizeArgs) {
        let viewport = args.viewport();
    }
}


fn main() {
    let mut app = App::new(WINDOW_WIDTH, WINDOW_HEIGHT);

    while let Some(event) = app.window.next() {
        if let Some(args) = event.resize_args() {
            app.on_resize(&args);
            if app.outputs_log {
                println!("New window size {}x{}", args.window_size[0], args.window_size[1]);
            }
        }

        if let Some(Button::Mouse(button)) = event.press_args() {
            if app.outputs_log {
                println!("Pressed mouse button '{:?}'", button);
            }
        }

        if let Some(Button::Keyboard(key)) = event.press_args() {
            app.on_key(key);
            if app.outputs_log {
                println!("Pressed keyboard key: '{:?}'", key);
                println!("Direction: {:?}", app.circle.direction);
                println!("Speed: {:?}", app.circle.speed);
            }
        };

        if let Some(args) = event.render_args() {
            app.render(&args, &event);
        }

        if let Some(args) = event.update_args() {
            app.update(&args);
        }
    }
}

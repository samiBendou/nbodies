extern crate opengl_graphics;
extern crate piston_window;

use opengl_graphics::OpenGL;
use piston::input::*;
use piston_window::{PistonWindow, WindowSettings};

use ::piston_start::App;

fn main() {
    let opengl = OpenGL::V3_2;
    let mut cursor = [0., 0.];
    let mut pressed_key: Key = Key::Unknown;
    let mut clicked_button: MouseButton = MouseButton::Unknown;
    let mut app = App::default_circle();

    let mut window: PistonWindow =
        WindowSettings::new("Circle Keeps Moving Like a Rollin' Stone!", app.config.size)
            .exit_on_esc(true)
            .resizable(false)
            .graphics_api(opengl)
            .build()
            .unwrap();

    while let Some(event) = window.next() {
        event.mouse_cursor(|pos| {
            cursor = pos;
        });

        if let Some(Button::Mouse(button)) = event.press_args() {
            clicked_button = button;
            app.on_click(clicked_button, &cursor);
        }

        if let Some(Button::Keyboard(key)) = event.press_args() {
            pressed_key = key;
            app.on_key(key, &cursor);
        };

        if let Some(_args) = event.render_args() {
            app.render(&mut window, &event);
        }

        if let Some(args) = event.update_args() {
            app.update(&mut window, &args);
        }

        app.log(clicked_button, pressed_key, cursor);
    }
}

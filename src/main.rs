extern crate opengl_graphics;
extern crate piston_window;

use opengl_graphics::OpenGL;
use piston::input::*;
use piston_window::{PistonWindow, WindowSettings};

use ::piston_start::App;

fn main() {
    let opengl = OpenGL::V3_2;

    let mut window: PistonWindow =
        WindowSettings::new("Circle Keeps Moving Like a Rollin' Stone!", [640., 640.])
            .exit_on_esc(true)
            .resizable(false)
            .graphics_api(opengl)
            .build()
            .unwrap();


    let mut app = App::from_size(640., 640.);

    while let Some(event) = window.next() {
        if let Some(Button::Mouse(button)) = event.press_args() {
            if app.config.display_log {
                println!("Pressed mouse button '{:?}'", button);
            }
        }

        if let Some(Button::Keyboard(key)) = event.press_args() {
            app.on_key(key);
            if app.config.display_log {
                println!("Pressed keyboard key: '{:?}'", key);
            }
        };

        if let Some(_args) = event.render_args() {
            app.render(&mut window, &event);
            if app.config.display_log && app.config.display_state {
                println!("Direction: {:?}", app.circle.center);
                println!("Speed: {:?}", app.circle.speed);
                println!("Application state {:?}", app.state)
            }
        }

        if let Some(args) = event.update_args() {
            app.update(&mut window, &args);
            if app.config.display_log && app.config.display_dt {
                println!("Time step dt: {:.2} sec.", args.dt);
                println!("Framerate: {:.2} fps", 1. / args.dt);
            }
        }
    }
}

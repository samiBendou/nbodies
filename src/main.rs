extern crate opengl_graphics;
extern crate piston_window;

use opengl_graphics::OpenGL;
use piston::input::*;
use piston_window::{PistonWindow, WindowSettings};

use ::piston_start::App;

fn main() {
    let opengl = OpenGL::V3_2;
    let mut cursor = [0., 0.];
    let mut app = App::default_circle();

    let mut window: PistonWindow =
        WindowSettings::new("Circle Keeps Moving Like a Rollin' Stone!", app.config.size)
            .exit_on_esc(true)
            .resizable(false)
            .graphics_api(opengl)
            .build()
            .unwrap();

    while let Some(event) = window.next() {
        if app.has_to_render() {
            print!("{}[2J", 27 as char);
        }

        event.mouse_cursor(|pos| {
            cursor = pos;
        });

        if let Some(Button::Mouse(button)) = event.press_args() {
            app.on_click(Button::from(button), &cursor);
            if app.config.display_log && app.has_to_render() {
                println!("pressed mouse button: '{:?}'", button);
                println!("mouse at: {:?} (px)", cursor);
            }
        }

        if let Some(Button::Keyboard(key)) = event.press_args() {
            app.on_key(key);
            if app.config.display_log && app.has_to_render() {
                println!("pressed keyboard key: '{:?}'", key);
            }
        };

        if let Some(_args) = event.render_args() {
            app.render(&mut window, &event);
            if app.config.display_log && app.config.display_circle && app.has_to_render() {
                println!("current circle: {}", app.config.status.current_circle);
                println!("position: {:?} (px)", app.circles[app.config.status.current_circle].position);
                println!("speed: {:?} (px/s)", app.circles[app.config.status.current_circle].speed);
            }
        }

        if let Some(args) = event.update_args() {
            app.update(&mut window, &args);
            if app.config.display_log && app.config.display_dt && app.has_to_render() {
                println!("dt: {:.4} (ms)", args.dt * 1e3);
                println!("framerate: {:.2} (fps)", 1. / args.dt);
                println!("frames per updates: {}", app.config.frames_per_update);
                println!("updates per frame: {}", app.config.updates_per_frame);
            }
            if app.config.display_log && app.config.display_state && app.has_to_render() {
                println!("state: {:?}", app.state)
            }
        }
    }
}

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
    let mut app = App::default();

    let mut window: PistonWindow =
        WindowSettings::new("Bodies Keeps Moving Like a Rollin' Stone!", app.config.size)
            .exit_on_esc(true)
            .resizable(false)
            .graphics_api(opengl)
            .build()
            .unwrap();
    let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();
    let mut glyphs = window.load_font(assets.join("FiraSans-Regular.ttf")).unwrap();

    while let Some(event) = window.next() {
        event.mouse_cursor(|pos| {
            cursor = pos;
        });

        if let Some(Button::Mouse(button)) = event.press_args() {
            clicked_button = button;
            app.on_click(clicked_button);
        }

        if let Some(Button::Keyboard(key)) = event.press_args() {
            pressed_key = key;
            app.on_key(key);
        };

        if let Some(_args) = event.render_args() {
            app.render(&mut window, &event, &mut glyphs);
        }

        if let Some(args) = event.update_args() {
            app.update(&mut window, &args, &cursor);
        }

        app.log(clicked_button, pressed_key, cursor);
    }
}

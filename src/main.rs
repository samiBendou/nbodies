extern crate find_folder;
extern crate opengl_graphics;
extern crate piston_window;

use opengl_graphics::OpenGL;
use piston::event_loop::EventLoop;
use piston::input::{Button, MouseCursorEvent, PressEvent, RenderEvent, UpdateEvent};
use piston_window::{Events, PistonWindow, WindowSettings};

use piston_start::App;
use piston_start::common::Input;

fn main() {
    let opengl = OpenGL::V3_2;
    let mut app: App = App::default();
    let mut input = Input::new();
    let mut window: PistonWindow =
        WindowSettings::new("Bodies Keeps Moving Like Rollin' Stones!", app.config.size)
            .exit_on_esc(true)
            .resizable(false)
            .graphics_api(opengl)
            .build()
            .unwrap();

    window.events.set_max_fps(30);
    window.events.set_ups(30);


    let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();
    let mut glyphs = window.load_font(assets.join("FiraSans-Regular.ttf")).unwrap();
    while let Some(event) = window.next() {
        event.mouse_cursor(|pos| {
            input.cursor = pos;
        });

        if let Some(Button::Mouse(button)) = event.press_args() {
            input.button = Some(button);
            app.on_click(&button);
        }

        if let Some(Button::Keyboard(key)) = event.press_args() {
            input.key = Some(key);
            app.on_key(&key);
        }

        if let Some(_args) = event.render_args() {
            app.render(&mut window, &event, &mut glyphs);
            app.log(&input);
        }
        if let Some(args) = event.update_args() {
            app.update(&mut window, &args, &input.cursor);
        }
    }
}

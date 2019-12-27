extern crate find_folder;
extern crate opengl_graphics;
extern crate piston_window;

use std::path::Path;
use std::process::exit;

use opengl_graphics::OpenGL;
use piston::event_loop::EventLoop;
use piston::input::{Button, MouseCursorEvent, PressEvent, RenderEvent, UpdateEvent};
use piston_window::{PistonWindow, WindowSettings};

use piston_start::App;
use piston_start::common::Input;
use piston_start::physics::dynamics;
use piston_start::physics::dynamics::orbital;

fn main() {
    let opengl = OpenGL::V3_2;
    let path: &Path = Path::new("data/solar_system.json");
    let orbit_cluster = orbital::Cluster::from(path);
    let mut body_cluster = dynamics::Cluster::empty();
    for body in orbit_cluster.bodies.iter() {
        body_cluster.bodies.push(dynamics::Body::planet(body, 0.));
    }
    let mut app: App = App::cluster(body_cluster);
    let mut input = Input::new();
    let mut window: PistonWindow =
        WindowSettings::new("Bodies Keeps Moving Like Rollin' Stones!", app.config.size)
            .exit_on_esc(true)
            .resizable(false)
            .graphics_api(opengl)
            .build()
            .unwrap();

    window.events.set_max_fps(60);
    window.events.set_ups(60);

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

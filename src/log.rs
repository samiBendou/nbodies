use piston::input::{Key, MouseButton};

use crate::common::*;
use crate::core;
use crate::physics::dynamics::VecBody;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum State {
    Hide,
    Status,
    Timing,
    Cinematic,
    Physics,
    Bodies,
}

impl State {
    pub fn next(&mut self, key: &Key) {
        use State::*;
        *self = match key {
            Key::L => {
                match self {
                    Hide => Status,
                    Status => Timing,
                    Timing => Cinematic,
                    Cinematic => Physics,
                    Physics => Bodies,
                    Bodies => Hide,
                }
            }
            _ => *self
        };
    }
}

pub struct Logger {
    state: State,
    buffer: String,
}

impl Logger {
    pub fn new() -> Logger {
        Logger { state: State::Hide, buffer: String::from("") }
    }

    pub fn update(&mut self, key: &Option<Key>) {
        if let Some(key) = key {
            self.state.next(key);
        }
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn print(&self, clear_screen: bool) {
        if clear_screen {
            print!("{}[2J", 27 as char);
        }
        println!("{}", self.buffer);
    }

    pub fn log(
        &mut self,
        bodies: &VecBody,
        status: &core::Status,
        config: &core::Config,
        step: &core::Step,
        input: &Input,
    ) {
        use crate::log::State::*;

        match self.state {
            Hide => (),
            Status => self.log_status(status, input),
            Timing => self.log_timing(step, config),
            Cinematic => self.log_cinematic(bodies, config),
            Physics => self.log_physics(bodies, config),
            Bodies => self.log_bodies(bodies)
        };
    }

    fn log_status(&mut self, status: &core::Status, input: &Input) {
        self.buffer += &format!("*** status info ***\n")[..];
        self.buffer += &format!("{:#?}\n", status)[..];
        self.buffer += &format!("pressed mouse button: '{:?}'\n", input.button)[..];
        self.buffer += &format!("mouse at: {:?} (px)\n", input.cursor)[..];
        self.buffer += &format!("pressed keyboard key: '{:?}'\n", input.key)[..];
    }

    fn log_timing(&mut self, step: &core::Step, config: &core::Config) {
        self.buffer += &format!("*** timing info ***\n")[..];
        self.buffer += &format!("{:?}\n", step)[..];
        self.buffer += &format!("frames per updates: {}\n", config.frames_per_update)[..];
        self.buffer += &format!("updates per frame: {}\n", config.updates_per_frame)[..];
    }

    fn log_cinematic(&mut self, bodies: &VecBody, config: &core::Config) {
        if bodies.is_empty() {
            return;
        }
        let mut scaled_point = bodies.current().shape.center.clone();
        scaled_point.scale(config.scale.distance);
        self.buffer += &format!("*** current shape ***\n")[..];
        self.buffer += &format!("{:?}\n", scaled_point)[..];
        self.buffer += &format!("*** scale ***\n")[..];
        self.buffer += &format!("{:?}\n", config.scale)[..];
    }

    fn log_physics(&mut self, bodies: &VecBody, config: &core::Config) {
        if bodies.is_empty() {
            return;
        }

        self.buffer += &format!("*** current body ***\n")[..];
        self.buffer += &format!("{:?}\n", bodies.current())[..];
        self.buffer += &format!("*** scale ***\n")[..];
        self.buffer += &format!("{:?}\n", config.scale)[..];
    }

    fn log_bodies(&mut self, bodies: &VecBody) {
        self.buffer += &format!("*** body list ***\n")[..];
        self.buffer += &format!("{:?}\n", bodies)[..];
    }
}
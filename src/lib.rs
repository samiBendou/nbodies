use std::cmp::{max, min};

use piston::input::{Event, Key, MouseButton, UpdateArgs};
use piston::window::Size;
use piston_window;
use piston_window::PistonWindow;

use crate::common::*;
use crate::physics::{Body, PX_PER_METER, TRAJECTORY_SIZE, VecBody};
use crate::shape::*;
use crate::vector::Vector2;

pub mod vector;
pub mod common;
pub mod shape;
pub mod physics;

macro_rules! toggle {
    ($boolean: expr) => {
    $boolean = !$boolean;
    };
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LogState {
    Hide,
    Default,
    Timing,
    Cinematic,
    Physics,
    Bodies,
}

impl LogState {
    pub fn next(&mut self, key: &Option<Key>) {
        use LogState::*;

        if let Some(key) = key {
            *self = match key {
                Key::L => {
                    match self {
                        Hide => Default,
                        Default => Timing,
                        Timing => Cinematic,
                        Cinematic => Physics,
                        Physics => Bodies,
                        Bodies => Hide,
                    }
                },
                _ => *self
            };
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum AppState {
    Move,
    Add,
    WaitDrop,
    CancelDrop,
    Reset,
}

impl AppState {
    pub fn next(&mut self, key: &Option<Key>, button: &Option<MouseButton>) {
        use AppState::*;

        if let Some(key) = key {
            match key {
                Key::Backspace => {
                    *self = Reset;
                    return;
                },
                _ => *self,
            };
        }

        *self = match self {
            Reset => Move,
            Add => WaitDrop,
            CancelDrop => Move,
            Move => {
                if let Some(button) = button {
                    match button {
                        MouseButton::Left => Add,
                        _ => *self,
                    }
                } else {
                    *self
                }
            },
            WaitDrop => {
                if let Some(button) = button {
                    match button {
                        MouseButton::Left => Move,
                        MouseButton::Right => CancelDrop,
                        _ => WaitDrop,
                    }
                } else {
                    *self
                }
            }
        };
    }
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub size: Size,
    pub frames_per_update: u32,
    pub updates_per_frame: u32,
}

impl AppConfig {
    pub fn new(size: Size, frames_per_update: u32, updates_per_frame: u32) -> AppConfig {
        AppConfig {
            size,
            frames_per_update,
            updates_per_frame,
        }
    }

    pub fn default() -> AppConfig {
        AppConfig {
            size: Size::from([640., 640.]),
            frames_per_update: 1,
            updates_per_frame: 1024,
        }
    }

    pub fn update(&mut self, key: &Key) {
        match *key {
            Key::O => self.updates_per_frame = max(self.updates_per_frame >> 1, std::u32::MIN + 1),
            Key::P => self.updates_per_frame = min(self.updates_per_frame << 1, std::u32::MAX),
            _ => (),
        };
    }

    pub fn clear(&mut self, size: Size) {
        self.size = size;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AppStatus {
    pub direction: Direction,
    pub bounded: bool,
    pub translate: bool,
    pub trajectory: bool,
    pub pause: bool,
    pub state: AppState,
    pub state_log: LogState,
}

impl AppStatus {
    pub fn new(bounded: bool, translate: bool) -> AppStatus {
        let direction = Direction::Hold;
        let state = AppState::Reset;
        let state_log = LogState::Default;
        AppStatus {
            direction,
            bounded,
            translate,
            trajectory: true,
            pause: true,
            state,
            state_log,
        }
    }

    pub fn default() -> AppStatus {
        AppStatus::new(true, false)
    }

    pub fn update(&mut self, key: &Option<Key>, button: &Option<MouseButton>) {
        if let Some(key) = key {
            match *key {
                Key::B => toggle!(self.bounded),
                Key::T => toggle!(self.translate),
                Key::R => toggle!(self.trajectory),
                Key::Space => toggle!(self.pause),
                _ => ()
            }
            self.direction = Direction::from(key);
        } else {
            self.direction = Direction::Hold;
        }
        self.state.next(key, button);
        self.state_log.next(key);
    }
}

pub struct App {
    pub bodies: VecBody,
    pub config: AppConfig,
    pub status: AppStatus,
    pub step: Step,
}

impl App {
    pub fn new(bodies: VecBody, status: AppStatus, config: AppConfig) -> App {
        App {
            bodies,
            config,
            status,
            step: Step::new(),
        }
    }

    pub fn default() -> App {
        App {
            bodies: VecBody::empty(),
            config: AppConfig::default(),
            status: AppStatus::default(),
            step: Step::new(),
        }
    }

    pub fn on_key(&mut self, key: Key) {
        self.config.update(&key);
        self.status.update(&Some(key), &Option::None);
        self.bodies.update_current_index(&key);
    }

    pub fn on_click(&mut self, button: MouseButton) {
        self.status.update(&Option::None, &Some(button));
    }

    pub fn log(&self, button: MouseButton, key: Key, cursor: [f64; 2]) {
        use LogState::*;

        match self.status.state_log {
            Hide => (),
            Default => {
                print!("{}[2J", 27 as char);
                println!("state: {:?}", self.status.state);
                println!("pressed mouse button: '{:?}'", button);
                println!("mouse at: {:?} (px)", cursor);
                println!("pressed keyboard key: '{:?}'", key);
            },
            Timing => {
                print!("{}[2J", 27 as char);
                println!("{:?}", self.step);
                println!("frames per updates: {}", self.config.frames_per_update);
                println!("updates per frame: {}", self.config.updates_per_frame);
            },
            Cinematic => {
                print!("{}[2J", 27 as char);
                if self.bodies.is_empty() {
                    return;
                }
                println!("{:?}", self.bodies.current().shape);
            },
            Physics => {
                print!("{}[2J", 27 as char);
                if self.bodies.is_empty() {
                    return;
                }
                println!("{:?}", self.bodies.current());
            },
            Bodies => {
                print!("{}[2J", 27 as char);
                println!("{:?}", self.bodies);
            }
        };
    }

    pub fn render(&self, window: &mut PistonWindow, event: &Event) {
        let count = self.bodies.count();

        window.draw_2d(
            event,
            |c, g, _device| {
                let x_offset = self.config.size.height - 50.;
                let y_offset = self.config.size.height - 50.;
                let mut color: [f32; 4];
                let mut from: [f64; 2];
                let mut to: [f64; 2];
                let barycenter_rect = to_left_up(self.bodies.barycenter().as_array(), &self.config.size);
                let mut rect: [f64; 4];

                piston_window::clear([1.0; 4], g);

                if count == 0 {
                    return;
                }
                if self.status.trajectory {
                    for i in 0..count {
                        color = self.bodies[i].shape.color;
                        for k in 1..TRAJECTORY_SIZE - 1 {
                            color[3] = k as f32 / (TRAJECTORY_SIZE as f32 - 1.);
                            from = self.bodies[i].shape.center.position(k - 1).as_array();
                            to = self.bodies[i].shape.center.position(k).as_array();
                            piston_window::line_from_to(color, 2.5, from, to, c.transform, g);
                        }
                    }
                }
                for i in 0..count {
                    rect = self.bodies[i].shape.rounding_rect(&self.config.size);
                    piston_window::ellipse(self.bodies[i].shape.color, rect, c.transform, g);
                }
                rect = [barycenter_rect[0] - 5., barycenter_rect[1] - 5., 10., 10.];
                from = [x_offset - 10. * PX_PER_METER, y_offset];
                to = [x_offset, y_offset];
                piston_window::rectangle([255., 0., 0., 1.], rect, c.transform, g);
                piston_window::line_from_to([0., 0., 0., 1.], 3., from, to, c.transform, g);
            },
        );
    }

    pub fn has_to_render(&self) -> bool {
        self.step.count > self.config.frames_per_update
    }

    pub fn update(&mut self, _window: &mut PistonWindow, args: &UpdateArgs, cursor: &[f64; 2]) {
        use AppState::*;
        self.step.update(args.dt);
        match self.status.state {
            Move => self.do_move(self.step.frame / self.config.updates_per_frame as f64),
            Reset => self.do_reset(),
            Add => self.do_add(cursor),
            WaitDrop => self.do_wait_drop(cursor),
            CancelDrop => self.do_cancel_drop()
        };
        self.status.update(&Option::None, &Option::None);
    }

    fn do_move(&mut self, dt: f64) {
        let size = &Some(self.config.size);
        if self.status.pause || self.bodies.is_empty() {
            return;
        }
        if self.status.translate {
            self.bodies.translate_current(&self.status.direction.as_vector());
            if self.status.bounded {
                self.bodies.bound_current(&self.config.size);
            }
            self.bodies.update_current_trajectory(size);
            return;
        }
        self.do_accelerate(dt);

        if self.status.bounded {
            self.bodies.bound(&self.config.size);
        }
        self.bodies.update_trajectory(size);
        self.bodies.update_barycenter();
    }

    fn do_accelerate(&mut self, dt: f64) {
        use physics::forces;
        let count = self.bodies.count();
        let mut directions = vec![Direction::Hold; count];
        let mut force: Vector2;

        directions[self.bodies.current_index()] = self.status.direction;
        for _ in 0..self.config.updates_per_frame {
            for i in 0..count {
                force = forces::push(&directions[i]);
                force += forces::nav_stokes(&self.bodies[i].shape.center.speed);
                self.bodies[i].shape.center.acceleration = force * (PX_PER_METER / self.bodies[i].mass);
            }
            self.bodies.accelerate(dt);
        }
    }

    fn do_reset(&mut self) {
        if !self.bodies.is_empty() {
            self.bodies.reset_current();
            self.bodies.clear_current_trajectory(&Some(self.config.size));
            self.bodies.update_barycenter();
        }
    }

    fn do_add(&mut self, cursor: &[f64; 2]) {
        let circle = Circle::at_cursor_random(cursor, &self.config.size);
        let body = Body::new(circle.radius / 5., format!(""), circle);
        self.bodies.push(body);
        self.bodies.current_mut().name = format!("body {}", self.bodies.current_index() + 1);
    }

    fn do_wait_drop(&mut self, cursor: &[f64; 2]) {
        self.bodies.wait_drop(cursor, &self.config.size);
        self.bodies.clear_current_trajectory(&Some(self.config.size));
    }

    fn do_cancel_drop(&mut self) {
        self.bodies.pop();
    }
}
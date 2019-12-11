use std::cmp::{max, min};

use piston::input::{Event, Key, MouseButton, UpdateArgs};
use piston::window::Size;
use piston_window::clear;
use piston_window::ellipse;
use piston_window::line_from_to;
use piston_window::PistonWindow;

use crate::common::*;
use crate::physics::TRAJECTORY_SIZE;
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
}

impl LogState {
    pub fn next(&mut self, key: &Option<Key>) {
        if let Some(key) = key {
            match key {
                Key::L => {
                    match self {
                        LogState::Hide => *self = LogState::Default,
                        LogState::Default => *self = LogState::Timing,
                        LogState::Timing => *self = LogState::Cinematic,
                        LogState::Cinematic => *self = LogState::Hide,
                    }
                },
                _ => ()
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
        if let Some(key) = key {
            match key {
                Key::Backspace => {
                    *self = AppState::Reset;
                    return;
                },
                _ => (),
            };
        }

        *self = match self {
            AppState::Reset => AppState::Move,
            AppState::Add => AppState::WaitDrop,
            AppState::CancelDrop => AppState::Move,
            AppState::Move => {
                if let Some(button) = button {
                    match button {
                        MouseButton::Left => AppState::Add,
                        _ => *self,
                    }
                } else {
                    *self
                }
            },
            AppState::WaitDrop => {
                if let Some(button) = button {
                    match button {
                        MouseButton::Left => AppState::Move,
                        MouseButton::Right => AppState::CancelDrop,
                        _ => AppState::WaitDrop,
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
    pub current_circle: usize,
    pub count_circle: usize,
    pub state: AppState,
    pub state_log: LogState,
}

impl AppStatus {
    pub fn new(bounded: bool, translate: bool, count_circle: usize) -> AppStatus {
        let direction = Direction::Hold;
        let state = AppState::Reset;
        let state_log = LogState::Default;
        AppStatus {
            direction,
            bounded,
            translate,
            trajectory: true,
            pause: true,
            current_circle: 0,
            count_circle,
            state,
            state_log,
        }
    }

    pub fn default(count_circle: usize) -> AppStatus {
        AppStatus::new(true, false, count_circle)
    }

    pub fn clear(&mut self, circles: &Vec<Circle>) {
        self.direction = Direction::Hold;
        self.count_circle = circles.len();
    }

    pub fn update(&mut self, key: &Option<Key>, button: &Option<MouseButton>) {
        let current_circle = self.current_circle as isize;
        let count = self.count_circle as isize;

        if let Some(key) = key {
            match *key {
                Key::Z => self.current_circle = max(current_circle - 1, 0) as usize,
                Key::X => self.current_circle = min(current_circle + 1, max(count - 1, 0)) as usize,
                Key::B => toggle!(self.bounded),
                Key::T => toggle!(self.translate),
                Key::R => toggle!(self.trajectory),
                Key::Space => toggle!(self.pause),
                _ => ()
            }
            self.direction = Direction::from(key);
        }
        self.state.next(key, button);
        self.state_log.next(key);
    }
}

pub struct App {
    pub circles: Vec<Circle>,
    pub config: AppConfig,
    pub status: AppStatus,
    pub step: Step,
}

impl App {
    fn new(circles: Vec<Circle>, status: AppStatus, config: AppConfig) -> App {
        App { circles, config, status, step: Step::new() }
    }

    pub fn centered_circle(radius: f64, color: Color) -> App {
        let config = AppConfig::default();
        App::new(
            vec![Circle::zeros(radius, color, &config.size)],
            AppStatus::default(1),
            config)
    }

    pub fn default_circle() -> App {
        App::centered_circle(30., Color::Blue)
    }

    pub fn on_key(&mut self, key: Key) {
        self.config.update(&key);
        self.status.update(&Some(key), &Option::None);
    }

    pub fn on_click(&mut self, button: MouseButton) {
        self.status.update(&Option::None, &Some(button));
    }

    pub fn render(&mut self, window: &mut PistonWindow, event: &Event) {
        window.draw_2d(
            event,
            |c, g, _device| {
                clear([1.0; 4], g);
                if self.status.trajectory {
                    for circle in self.circles.iter() {
                        let color = circle.color.rgba_array();
                        for k in 1..TRAJECTORY_SIZE - 1 {
                            let from = circle.center.position(k - 1).as_array();
                            let to = circle.center.position(k).as_array();
                            line_from_to(color, 2.5, from, to, c.transform, g);
                        }
                    }
                }

                for circle in self.circles.iter() {
                    let color = circle.color.rgba_array();
                    let rect = circle.rounding_rect(&self.config.size);
                    ellipse(color, rect, c.transform, g);
                }
            },
        );
    }

    pub fn has_to_render(&self) -> bool {
        self.step.count > self.config.frames_per_update
    }

    pub fn update(&mut self, _window: &mut PistonWindow, args: &UpdateArgs, cursor: &[f64; 2]) {
        self.step.update(args.dt);
        if !self.has_to_render() {
            self.status.clear(&self.circles);
            return;
        }

        let dt = self.step.total / self.config.updates_per_frame as f64;
        match self.status.state {
            AppState::Move => self.do_move(dt),
            AppState::Reset => self.do_reset(),
            AppState::Add => self.do_add(cursor),
            AppState::WaitDrop => self.do_wait_drop(cursor),
            AppState::CancelDrop => self.do_cancel_add()
        };

        self.status.update(&Option::None, &Option::None);
        self.status.clear(&self.circles);
        self.step.total = 0.;
    }

    pub fn log(&self, button: MouseButton, key: Key, cursor: [f64; 2]) {
        match self.status.state_log {
            LogState::Hide => (),
            LogState::Default => {
                print!("{}[2J", 27 as char);
                println!("state: {:?}", self.status.state);
                println!("pressed mouse button: '{:?}'", button);
                println!("mouse at: {:?} (px)", cursor);
                println!("pressed keyboard key: '{:?}'", key);
            },
            LogState::Cinematic => {
                print!("{}[2J", 27 as char);
                println!("{:?}", self.circles[self.status.current_circle]);
                println!("current circle: {}", self.status.current_circle);
            },
            LogState::Timing => {
                print!("{}[2J", 27 as char);
                println!("{:?}", self.step);
                println!("frames per updates: {}", self.config.frames_per_update);
                println!("updates per frame: {}", self.config.updates_per_frame);
            },
        };
    }

    fn do_move(&mut self, dt: f64) {
        use physics::forces::push;
        use physics::forces::nav_stokes;

        if self.status.pause {
            return;
        }

        let len = self.circles.len();
        if self.status.translate {
            self.circles[self.status.current_circle].center.translate(&self.status.direction.as_vector());
        } else {
            let mut directions = vec![Direction::Hold; self.status.count_circle];

            directions[self.status.current_circle] = self.status.direction;
            for _ in 0..self.config.updates_per_frame {
                for index in 0..len {
                    self.circles[index].center.acceleration = push(&directions[index]) + nav_stokes(&self.circles[index].center.speed);
                    self.circles[index].center.accelerate(dt);
                }
            }
        }
        for index in 0..len {
            if self.status.bounded {
                self.circles[index].replace(self.config.size);
            }
            self.circles[index].center.update_trajectory(&Some(self.config.size));
        }
    }

    fn do_reset(&mut self) {
        self.circles[self.status.current_circle].center.reset(Vector2::zeros());
        self.circles[self.status.current_circle].center.clear_trajectory(&Some(self.config.size));
    }

    fn do_add(&mut self, cursor: &[f64; 2]) {
        let circle = Circle::at_cursor(cursor, 30., Color::Green, &self.config.size);
        self.circles.push(circle);
        self.status.current_circle = self.status.count_circle;
    }

    fn do_wait_drop(&mut self, cursor: &[f64; 2]) {
        self.circles.last_mut().unwrap().set_cursor_pos(cursor, &self.config.size);
        self.circles.last_mut().unwrap().center.clear_trajectory(&Some(self.config.size));
    }

    fn do_cancel_add(&mut self) {
        self.circles.pop();
        self.status.current_circle = 0;
    }
}
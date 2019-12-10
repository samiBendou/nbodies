use std::cmp::{max, min};

use piston::input::{Event, Key, MouseButton, UpdateArgs};
use piston::window::Size;
use piston_window::clear;
use piston_window::ellipse;
use piston_window::PistonWindow;

use crate::common::*;
use crate::shape::*;

pub mod vector;
pub mod common;
pub mod shape;

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
    Add { cursor: [f64; 2] },
    Reset,
}

impl AppState {
    pub fn next(&mut self, key: &Option<Key>, button: &Option<MouseButton>, cursor: &[f64; 2]) {
        if let Some(button) = button {
            match button {
                MouseButton::Left => {
                    *self = AppState::Add { cursor: *cursor };
                    return;
                },
                _ => (),
            }
        }

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
            AppState::Add { cursor: _ } => AppState::Move,
            _ => *self,
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

    pub fn update(&mut self, key: &Option<Key>, button: &Option<MouseButton>, cursor: &[f64; 2]) {
        let current_circle = self.current_circle as isize;
        let count = self.count_circle as isize;

        if let Some(key) = key {
            match *key {
                Key::Z => self.current_circle = max(current_circle - 1, 0) as usize,
                Key::X => self.current_circle = min(current_circle + 1, max(count - 1, 0)) as usize,
                Key::B => toggle!(self.bounded),
                Key::T => toggle!(self.translate),
                Key::Space => toggle!(self.pause),
                _ => ()
            }
            self.direction = Direction::from(key);
        }
        self.state.next(key, button, cursor);
        self.state_log.next(key);
    }
}

pub struct AppStep {
    pub count: u32,
    pub total: f64,
    pub frame: f64,
}

impl AppStep {
    pub fn new() -> AppStep {
        AppStep { count: 0, total: 0., frame: 0. }
    }

    pub fn update(&mut self, dt: f64) {
        self.frame = dt;
        self.total += dt;
        self.count = (self.count + 1) % std::u32::MAX;
    }
}


pub struct App {
    pub circles: Vec<Circle>,
    pub config: AppConfig,
    pub status: AppStatus,
    pub step: AppStep,
}

impl App {
    fn new(circles: Vec<Circle>, status: AppStatus, config: AppConfig) -> App {
        App { circles, config, status, step: AppStep::new() }
    }

    pub fn centered_circle(radius: f64, color: Color) -> App {
        App::new(
            vec![Circle::centered(radius, color)],
            AppStatus::default(1),
            AppConfig::default())
    }

    pub fn default_circle() -> App {
        App::centered_circle(50., Color::Blue)
    }

    pub fn on_key(&mut self, key: Key, cursor: &[f64; 2]) {
        self.config.update(&key);
        self.status.update(&Some(key), &Option::None, cursor);
    }

    pub fn on_click(&mut self, button: MouseButton, cursor: &[f64; 2]) {
        self.status.update(&Option::None, &Some(button), cursor);
    }

    pub fn render(&mut self, window: &mut PistonWindow, event: &Event) {
        window.draw_2d(
            event,
            |c, g, _device| {
                clear([1.0; 4], g);
                for circle in self.circles.iter() {
                    let color = circle.color.rgba_array();
                    let rect = circle.rounding_rect(self.config.size.width, self.config.size.height);
                    ellipse(color, rect, c.transform, g);
                }
            },
        );
    }

    pub fn has_to_render(&self) -> bool {
        self.step.count > self.config.frames_per_update
    }

    pub fn update(&mut self, _window: &mut PistonWindow, args: &UpdateArgs) {
        self.step.update(args.dt);
        if !self.has_to_render() {
            self.status.clear(&self.circles);
            return;
        }

        match self.status.state {
            AppState::Move => self.do_move(self.step.total / self.config.updates_per_frame as f64),
            AppState::Reset => self.do_reset(),
            AppState::Add { cursor } => self.do_add(&cursor)
        };

        self.status.update(&Option::None, &Option::None, &[0., 0.]);
        self.status.clear(&self.circles);
        self.step.total = 0.;
    }

    fn do_move(&mut self, dt: f64) {
        if self.status.pause {
            return;
        }

        let len = self.circles.len();
        if self.status.translate {
            self.circles[self.status.current_circle].translate(&self.status.direction);
        } else {
            let mut directions = vec![Direction::Hold; self.status.count_circle];

            directions[self.status.current_circle] = self.status.direction;
            for _ in 0..self.config.updates_per_frame {
                for index in 0..len {
                    self.circles[index].accelerate(&directions[index], dt);
                }
            }
        }
        for index in 0..len {
            if self.status.bounded {
                self.circles[index].replace(self.config.size.width, self.config.size.height);
            }
        }
    }

    fn do_reset(&mut self) {
        self.circles[self.status.current_circle].reset(0., 0.);
    }

    fn do_add(&mut self, cursor: &[f64; 2]) {
        self.circles.push(Circle::at_cursor(cursor, self.config.size, 50., Color::Green));
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
                println!("current circle: {}", self.status.current_circle);
                println!("position: {:?} (px)", self.circles[self.status.current_circle].position);
                println!("speed: {:?} (px/s)", self.circles[self.status.current_circle].speed);
            },
            LogState::Timing => {
                print!("{}[2J", 27 as char);
                println!("dt: {:.4} (ms)", self.step.frame * 1e3);
                println!("framerate: {:.2} (fps)", 1. / self.step.frame);
                println!("frames per updates: {}", self.config.frames_per_update);
                println!("updates per frame: {}", self.config.updates_per_frame);
            },
        };
    }
}
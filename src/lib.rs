use std::cmp::{max, min};

use piston::input::{Button, Event, Key, UpdateArgs};
use piston::window::{Size, Window};
use piston_window::clear;
use piston_window::ellipse;
use piston_window::PistonWindow;

use crate::common::*;
use crate::shape::*;
use crate::vector::Vector2;

pub mod vector;
pub mod common;
pub mod shape;

macro_rules! toggle {
    ($boolean: expr) => {
    $boolean = !$boolean;
    };
}

macro_rules! increase_with_overflow {
    ($frame_count: expr) => {
        $frame_count = ($frame_count + 1) % std::u32::MAX;
    }
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub status: AppStatus,
    pub size: Size,
    pub frames_per_update: u32,
    pub updates_per_frame: u32,
    pub display_log: bool,
    pub display_dt: bool,
    pub display_state: bool,
    pub display_circle: bool,
}

impl AppConfig {
    pub fn new(status: AppStatus, size: Size,
               frames_per_update: u32, updates_per_frame: u32,
               display_log: bool, display_dt: bool, display_state: bool, display_circle: bool) -> AppConfig {
        AppConfig {
            status,
            size,
            frames_per_update,
            updates_per_frame,
            display_log,
            display_dt,
            display_state,
            display_circle,
        }
    }

    pub fn from_size(count_circle: usize, width: f64, height: f64) -> AppConfig {
        let status = AppStatus::default(count_circle);
        let size = Size::from([width, height]);
        AppConfig {
            status,
            size,
            frames_per_update: 1,
            updates_per_frame: 1024,
            display_log: true,
            display_dt: false,
            display_state: false,
            display_circle: false,
        }
    }

    pub fn default(count_circle: usize) -> AppConfig {
        AppConfig::from_size(count_circle, 640., 640.)
    }

    pub fn update(&mut self, key: Key) {
        let count = self.status.directions.len() as isize;
        let current_circle = self.status.current_circle as isize;

        self.status.directions[self.status.current_circle] = Direction::from(key);
        match key {
            Key::L => toggle!(self.display_log),
            Key::D => toggle!(self.display_dt),
            Key::S => toggle!(self.display_state),
            Key::C => toggle!(self.display_circle),
            Key::B => toggle!(self.status.bounded),
            Key::T => toggle!(self.status.translate),
            Key::Space => toggle!(self.status.pause),
            Key::R => self.status.reset = true,
            Key::O => self.updates_per_frame = max(self.updates_per_frame >> 1, std::u32::MIN + 1),
            Key::P => self.updates_per_frame = min(self.updates_per_frame << 1, std::u32::MAX),
            Key::Z => self.status.current_circle = max(current_circle - 1, 0) as usize,
            Key::X => self.status.current_circle = min(current_circle + 1, max(count - 1, 0)) as usize,
            _ => (),
        };
    }

    pub fn clear(&mut self, size: Size) {
        self.status.clear();
        self.size = size;
    }
}

#[derive(Clone, Debug)]
pub struct AppStatus {
    pub directions: Vec<Direction>,
    pub bounded: bool,
    pub translate: bool,
    pub reset: bool,
    pub pause: bool,
    pub current_circle: usize,
    pub count_circle: usize,
}

impl AppStatus {
    pub fn new(count_circle: usize, bounded: bool, translate: bool, reset: bool) -> AppStatus {
        let directions = vec![Direction::Hold; count_circle];
        AppStatus { directions, bounded, translate, reset, pause: true, current_circle: 0, count_circle }
    }

    pub fn default(count_circle: usize) -> AppStatus {
        AppStatus::new(count_circle, true, false, false)
    }

    pub fn clear(&mut self) {
        self.directions[self.current_circle] = Direction::Hold;
        self.reset = false;
    }
}

#[derive(Copy, Clone, Debug)]
pub enum AppState {
    Translate,
    Accelerate { dt: f64 },
    BoundTranslate { size: Size },
    BoundAccelerate { dt: f64, size: Size },
    Reset { size: Size },
    Pause,
}

impl AppState {
    pub fn from(dt: f64, config: &AppConfig) -> AppState {
        use AppState::*;

        let size = config.size;

        if config.status.reset {
            return Reset { size };
        }

        if config.status.pause {
            return Pause;
        }

        if config.status.bounded {
            if config.status.translate {
                BoundTranslate { size }
            } else {
                BoundAccelerate { dt, size }
            }
        } else {
            if config.status.translate {
                Translate
            } else {
                Accelerate { dt }
            }
        }
    }
}

pub struct App {
    pub circles: Vec<Circle>,
    pub config: AppConfig,
    pub state: AppState,
    pub frame_count: u32,
    pub frame_step: f64,
}

impl App {
    pub fn new(circles: Vec<Circle>, config: AppConfig) -> App {
        let state = AppState::Reset { size: config.size };
        App { circles, config, state, frame_count: 0, frame_step: 0. }
    }

    pub fn from_size(circles: Vec<Circle>, width: f64, height: f64) -> App {
        let config = AppConfig::from_size(circles.len(), width, height);

        App::new(circles, config)
    }

    pub fn centered_circle(radius: f64, color: Color) -> App {
        App::new(vec![Circle::centered(radius, color)], AppConfig::default(1))
    }

    pub fn default_circle() -> App {
        App::centered_circle(50., Color::Blue)
    }

    pub fn on_key(&mut self, key: Key) {
        self.config.update(key);
    }

    pub fn on_click(&mut self, button: Button, cursor: &[f64; 2]) {
        self.circles.push(Circle::from_cursor(cursor, self.config.size, 50., Color::Green));
        self.config.status.directions.push(Direction::Hold);
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
        self.frame_count > self.config.frames_per_update
    }

    pub fn update(&mut self, window: &mut PistonWindow, args: &UpdateArgs) {
        self.frame_step += args.dt;
        self.state = AppState::from(self.frame_step / self.config.updates_per_frame as f64, &self.config);

        if !self.has_to_render() {
            increase_with_overflow!(self.frame_count);
            return;
        }
        increase_with_overflow!(self.frame_count);

        for _ in 0..self.config.updates_per_frame {
            for index in 0..self.circles.len() {
                match self.state {
                    AppState::Translate => self.circles[index]
                        .translate(&self.config.status.directions[index]),

                    AppState::Accelerate { dt } => self.circles[index]
                        .accelerate(&self.config.status.directions[index], dt),

                    AppState::BoundTranslate { size } => self.circles[index]
                        .translate(&self.config.status.directions[index])
                        .replace(size.width, size.height),

                    AppState::BoundAccelerate { dt, size } => self.circles[index]
                        .accelerate(&self.config.status.directions[index], dt)
                        .replace(size.width, size.height),

                    AppState::Reset { size } => self.circles[self.config.status.current_circle]
                        .reset(0., 0.),

                    AppState::Pause => &mut self.circles[index],
                };
            }
        }

        self.config.clear(window.size());
        self.frame_step = 0.;
    }
}
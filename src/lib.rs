use std::cmp::{max, min};

use piston::input::{Event, Key, UpdateArgs};
use piston::window::{Size, Window};
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

#[derive(Copy, Clone, Debug)]
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

    pub fn from_size(width: f64, height: f64) -> AppConfig {
        let status = AppStatus::default();
        let size = Size::from([width, height]);
        AppConfig {
            status,
            size,
            frames_per_update: 2,
            updates_per_frame: 1024,
            display_log: true,
            display_dt: false,
            display_state: false,
            display_circle: false,
        }
    }

    pub fn default() -> AppConfig {
        AppConfig::from_size(640., 640.)
    }

    pub fn update(&mut self, key: Key) {
        self.status.direction = Direction::from(key);

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
            _ => (),
        };
    }

    pub fn clear(&mut self, size: Size) {
        self.status.clear();
        self.size = size;
    }
}

#[derive(Copy, Clone, Debug)]
pub struct AppStatus {
    pub direction: Direction,
    pub bounded: bool,
    pub translate: bool,
    pub reset: bool,
    pub pause: bool,
}

impl AppStatus {
    pub fn new(bounded: bool, translate: bool, reset: bool) -> AppStatus {
        let direction = Direction::Hold;
        AppStatus { direction, bounded, translate, reset, pause: true }
    }

    pub fn default() -> AppStatus {
        let direction = Direction::Hold;
        AppStatus { direction, bounded: true, translate: false, reset: false, pause: true }
    }

    pub fn clear(&mut self) {
        self.direction = Direction::Hold;
        self.reset = false;
    }
}

#[derive(Copy, Clone, Debug)]
pub enum AppState {
    Translate { direction: Direction },
    Accelerate { direction: Direction, dt: f64 },
    BoundTranslate { direction: Direction, size: Size },
    BoundAccelerate { direction: Direction, dt: f64, size: Size },
    Reset { size: Size },
    Pause,
}

impl AppState {
    pub fn from(dt: f64, config: &AppConfig) -> AppState {
        use AppState::*;

        let size = config.size;
        let direction = config.status.direction;

        if config.status.reset {
            return Reset { size };
        }

        if config.status.pause {
            return Pause;
        }

        if config.status.bounded {
            if config.status.translate {
                BoundTranslate { direction, size }
            } else {
                BoundAccelerate { direction, dt, size }
            }
        } else {
            if config.status.translate {
                Translate { direction }
            } else {
                Accelerate { direction, dt }
            }
        }
    }
}

pub struct App {
    pub circle: Circle,
    pub config: AppConfig,
    pub state: AppState,
    pub frame_count: u32,
    pub frame_step: f64,
}

impl App {
    pub fn new(circle: Circle, config: AppConfig) -> App {
        let state = AppState::Reset { size: config.size };
        App { circle, config, state, frame_count: 0, frame_step: 0. }
    }

    pub fn from_size(circle: Circle, width: f64, height: f64) -> App {
        App::new(circle, AppConfig::from_size(width, height))
    }

    pub fn centered_circle(radius: f64, color: Color) -> App {
        App::new(Circle::centered(radius, color), AppConfig::default())
    }

    pub fn default_circle() -> App {
        App::centered_circle(50., Color::Blue)
    }

    pub fn on_key(&mut self, key: Key) {
        self.config.update(key);
    }

    pub fn render(&mut self, window: &mut PistonWindow, event: &Event) {
        let color = self.circle.color.rgba_array();
        let rect = self.circle.rounding_rect(self.config.size.width, self.config.size.height);

        window.draw_2d(
            event,
            |c, g, _device| {
                clear([1.0; 4], g);
                ellipse(color, rect, c.transform, g);
            },
        );
    }

    pub fn has_to_render(&self) -> bool {
        self.frame_count % self.config.frames_per_update == 0
    }

    pub fn update(&mut self, window: &mut PistonWindow, args: &UpdateArgs) {
        self.frame_step += args.dt;
        self.state = AppState::from(self.frame_step / self.config.updates_per_frame as f64, &self.config);

        if !self.has_to_render() {
            self.frame_count += 1;
            return;
        }

        for _ in 0..self.config.updates_per_frame {
            match self.state {
                AppState::Translate { direction } => self.circle
                    .translate(&direction),

                AppState::Accelerate { direction, dt } => self.circle
                    .accelerate(&direction, dt),

                AppState::BoundTranslate { direction, size } => self.circle
                    .translate(&direction).replace(size.width, size.height),

                AppState::BoundAccelerate { direction, dt, size } => self.circle
                    .accelerate(&direction, dt).replace(size.width, size.height),

                AppState::Reset { size } => self.circle
                    .reset(size.width / 2., size.height / 2.),

                AppState::Pause => &mut self.circle,
            };
        }

        self.config.clear(window.size());
        self.frame_count += 1;
        self.frame_step = 0.;
    }
}
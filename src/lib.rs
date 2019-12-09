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

const DEFAULT_WIDTH: f64 = 640.;
const DEFAULT_HEIGHT: f64 = 640.;

#[derive(Copy, Clone, Debug)]
pub struct AppConfig {
    pub mode: AppMode,
    pub display_log: bool,
    pub display_dt: bool,
    pub display_state: bool,
}

impl AppConfig {
    pub fn new(mode: AppMode, display_log: bool, display_dt: bool, display_state: bool) -> AppConfig {
        AppConfig { mode, display_log, display_dt, display_state }
    }

    pub fn from_size(width: f64, height: f64) -> AppConfig {
        let mode = AppMode::from_size(width, height);

        AppConfig { mode, display_log: true, display_dt: false, display_state: false }
    }

    pub fn default() -> AppConfig {
        AppConfig::from_size(DEFAULT_WIDTH, DEFAULT_HEIGHT)
    }

    pub fn update(&mut self, key: Key) {
        match key {
            Key::L => toggle!(self.display_log),
            Key::D => toggle!(self.display_dt),
            Key::S => toggle!(self.display_state),
            Key::B => toggle!(self.mode.bounded),
            Key::T => toggle!(self.mode.translate),
            Key::Space => toggle!(self.mode.pause),
            Key::R => self.mode.reset = true,
            _ => (),
        };
    }
}

#[derive(Copy, Clone, Debug)]
pub struct AppMode {
    pub size: Size,
    pub bounded: bool,
    pub translate: bool,
    pub reset: bool,
    pub pause: bool,
}

impl AppMode {
    pub fn new(size: Size, bounded: bool, translate: bool, reset: bool) -> AppMode {
        AppMode { size, bounded, translate, reset, pause: true }
    }

    pub fn from_size(width: f64, height: f64) -> AppMode {
        let size = Size::from([width, height]);

        AppMode { size, bounded: true, translate: false, reset: false, pause: true }
    }

    pub fn default() -> AppMode {
        AppMode::from_size(DEFAULT_WIDTH, DEFAULT_HEIGHT)
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
    pub fn from(direction: Direction, dt: f64, config: &AppConfig) -> AppState {
        use AppState::*;

        let size = config.mode.size;

        if config.mode.reset {
            return Reset { size };
        }

        if config.mode.pause {
            return Pause;
        }

        if config.mode.bounded {
            if config.mode.translate {
                BoundTranslate { direction, size }
            } else {
                BoundAccelerate { direction, dt, size }
            }
        } else {
            if config.mode.translate {
                Translate { direction }
            } else {
                Accelerate { direction, dt }
            }
        }
    }
}

pub struct App {
    pub circle: Circle,
    pub direction: Direction,
    pub config: AppConfig,
    pub state: AppState,
}

impl App {
    pub fn new(circle: Circle, config: AppConfig) -> App {
        let state = AppState::Reset { size: config.mode.size };
        App { circle, direction: Direction::Hold, config, state }
    }

    pub fn from_size(width: f64, height: f64) -> App {
        let circle = Circle::new(width / 2., height / 2., 50., Color::Blue);
        let config = AppConfig::from_size(width, height);

        App::new(circle, config)
    }

    pub fn default() -> App {
        App::from_size(DEFAULT_WIDTH, DEFAULT_HEIGHT)
    }

    pub fn on_key(&mut self, key: Key) {
        self.direction = Direction::from(key);
        self.config.update(key);
    }

    pub fn render(&mut self, window: &mut PistonWindow, event: &Event) {
        let color = self.circle.color.rgba_array();
        let rect = self.circle.rounding_rect();

        window.draw_2d(
            event,
            |c, g, _device| {
                clear([1.0; 4], g);
                ellipse(color, rect, c.transform, g);
            },
        );
    }

    pub fn update(&mut self, window: &mut PistonWindow, args: &UpdateArgs) {
        self.state = AppState::from(self.direction, args.dt, &self.config);

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

        self.direction = Direction::Hold;
        self.config.mode.reset = false;
        self.config.mode.size = window.size();
    }
}
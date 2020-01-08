use std::cmp::{max, min};
use std::error::Error;

use getopts::Options;
use piston::input::{Key, MouseButton};
use piston::window::Size;

use crate::common::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum State {
    Move,
    Translate,
    Add,
    Remove,
    WaitDrop,
    WaitSpeed,
    CancelDrop,
    Reset,
}

impl State {
    pub fn next(&mut self, key: &Key, button: &MouseButton) {
        use State::*;

        if *key == KEY_RESET {
            *self = Reset;
            return;
        }

        *self = match self {
            Reset => Move,
            Add => WaitDrop,
            Remove => Move,
            CancelDrop => Move,
            Move => if *button == MOUSE_MOVE_ADD {
                Add
            } else if *button == MOUSE_MOVE_REMOVE {
                Remove
            } else if *key == KEY_TOGGLE_TRANSLATE {
                Translate
            } else {
                *self
            },
            Translate => if *key == KEY_TOGGLE_TRANSLATE {
                Move
            } else {
                *self
            },
            WaitDrop => if *button == MOUSE_WAIT_DROP_DO {
                WaitSpeed
            } else if *button == MOUSE_WAIT_DROP_CANCEL {
                CancelDrop
            } else {
                *self
            }
            WaitSpeed => if *button == MOUSE_WAIT_DROP_DO {
                Move
            } else if *button == MOUSE_WAIT_DROP_CANCEL {
                WaitDrop
            } else {
                *self
            }
        };
    }
}

pub struct Arguments {
    pub path: Option<String>,
    pub scale: Scale,
    pub size: Option<Size>,
}

impl Arguments {
    /*
    fn print_usage(program: &str, opts: Options) {
        let brief = format!("Usage: {} FILE [options]", program);
        print!("{}", opts.usage(&brief));
    }
    */

    pub fn new(args: Vec<String>) -> Result<Arguments, Box<dyn Error>> {
        let mut opts = Options::new();
        opts.optopt("o", "orbital", "Loads an orbital cluster from file", "FILEPATH");
        opts.optopt("d", "distance", "Sets the distance scale in px/meters", "NUMBER");
        opts.optopt("t", "time", "Sets the time scale in secs/real sec", "NUMBER");
        let matches = opts.parse(&args[1..])?;
        let path = matches.opt_str("o");
        let mut scale: Scale = Scale::unit();
        if let Some(distance_str) = matches.opt_str("d") {
            scale.distance = distance_str.parse()?;
        }
        if let Some(time_str) = matches.opt_str("t") {
            scale.time = time_str.parse()?;
        }
        let size: Option<Size> = None;
        Ok(Arguments { path, scale, size })
    }
}

pub struct Config {
    pub size: Size,
    pub scale: Scale,
    pub undersampling: u32,
    pub oversampling: u32,
    pub orientation: Orientation,
    pub trajectory: bool,
    pub pause: bool,
}

impl Config {
    pub fn new(size: Size, scale: Scale, undersampling: u32, oversampling: u32) -> Config {
        Config {
            size,
            scale,
            undersampling,
            oversampling,
            orientation: Orientation::new(0., 0., 0.),
            trajectory: true,
            pause: true,
        }
    }

    pub fn default() -> Config {
        Config::new(Size::from([640., 640.]), Scale::unit(), 1, 1024)
    }

    pub fn update(&mut self, key: &Key) {
        if *key == KEY_INCREASE_OVERSAMPLING {
            self.increase_oversampling();
        } else if *key == KEY_DECREASE_OVERSAMPLING {
            self.decrease_oversampling();
        } else if *key == KEY_ROTATION_DOWN {
            self.orientation.increment_x();
        } else if *key == KEY_ROTATION_UP {
            self.orientation.decrement_x();
        } else if *key == KEY_ROTATION_LEFT {
            self.orientation.increment_z();
        } else if *key == KEY_ROTATION_RIGHT {
            self.orientation.decrement_z();
        } else if *key == KEY_INCREASE_DISTANCE {
            self.scale.increase_distance();
        } else if *key == KEY_DECREASE_DISTANCE {
            self.scale.decrease_distance();
        } else if *key == KEY_INCREASE_TIME {
            self.scale.increase_time();
        } else if *key == KEY_DECREASE_TIME {
            self.scale.decrease_time();
        }
        self.scale.rescale();
    }

    fn increase_oversampling(&mut self) {
        self.oversampling = min(self.oversampling << 1, std::u32::MAX);
    }

    fn decrease_oversampling(&mut self) {
        self.oversampling = max(self.oversampling >> 1, std::u32::MIN + 1);
    }
}

impl From<Arguments> for Config {
    fn from(args: Arguments) -> Self {
        let mut ret = Config::default();
        ret.scale = args.scale;
        if let Some(size) = args.size {
            ret.size = size;
        }
        ret
    }
}

#[derive(Clone, Debug)]
pub struct Status {
    pub direction: Direction,
    pub reset_circles: bool,
    pub update_transform: bool,
    pub update_current: bool,
    pub increase_current: bool,
    pub next_frame: bool,
    pub next_method: bool,
    pub state: State,
    pub step: Step,
}

impl Status {
    pub fn new() -> Status {
        Status {
            direction: Direction::Hold,
            reset_circles: true,
            update_transform: true,
            update_current: false,
            increase_current: false,
            next_frame: false,
            next_method: false,
            state: State::Reset,
            step: Step::new(),
        }
    }

    pub fn is_waiting_to_add(&self) -> bool {
        self.state == State::WaitSpeed || self.state == State::WaitDrop
    }

    pub fn update(&mut self, key: &Option<Key>, button: &Option<MouseButton>) {
        match key {
            None => {
                self.direction = HOLD;
                match button {
                    None => self.state.next(&KEY_UNKNOWN, &BUTTON_UNKNOWN),
                    Some(button) => self.state.next(&KEY_UNKNOWN, button),
                };
            }
            Some(key) => {
                if *key == KEY_ROTATION_DOWN {
                    self.reset_circles = true;
                    self.update_transform = true;
                } else if *key == KEY_ROTATION_UP {
                    self.reset_circles = true;
                    self.update_transform = true;
                } else if *key == KEY_ROTATION_LEFT {
                    self.reset_circles = true;
                    self.update_transform = true;
                } else if *key == KEY_ROTATION_RIGHT {
                    self.reset_circles = true;
                    self.update_transform = true;
                } else if *key == KEY_INCREASE_CURRENT_INDEX {
                    self.increase_current = true;
                    self.update_current = true;
                    self.reset_circles = true;
                    self.update_transform = true;
                } else if *key == KEY_DECREASE_CURRENT_INDEX {
                    self.increase_current = false;
                    self.update_current = true;
                    self.reset_circles = true;
                    self.update_transform = true;
                } else if *key == KEY_NEXT_FRAME_STATE {
                    self.next_frame = true;
                    self.reset_circles = true;
                    self.update_transform = true;
                } else if *key == KEY_NEXT_METHOD_STATE {
                    self.next_method = true;
                } else if *key == KEY_INCREASE_DISTANCE || *key == KEY_DECREASE_DISTANCE {
                    self.reset_circles = true;
                    self.update_transform = true;
                } else {
                    self.direction = Direction::from(key);
                }
                match button {
                    None => self.state.next(key, &BUTTON_UNKNOWN),
                    Some(button) => self.state.next(key, button),
                };
            }
        };
    }

    pub fn clear(&mut self) {
        self.state.next(&KEY_UNKNOWN, &BUTTON_UNKNOWN);
        self.direction = Direction::from(&KEY_UNKNOWN);
        self.reset_circles = false;
        self.update_transform = false;
        self.update_current = false;
        self.next_frame = false;
    }
}
use piston::input::{Key, MouseButton};

pub static KEY_RESET: Key = Key::Backspace;

pub static KEY_TOGGLE_TRANSLATE: Key = Key::J;
pub static KEY_TOGGLE_TRAJECTORY: Key = Key::R;
pub static KEY_TOGGLE_PAUSE: Key = Key::Space;
pub static KEY_TOGGLE_ORBITS: Key = Key::Y;

pub static KEY_DIRECTION_UP: Key = Key::W;
pub static KEY_DIRECTION_DOWN: Key = Key::S;
pub static KEY_DIRECTION_LEFT: Key = Key::A;
pub static KEY_DIRECTION_RIGHT: Key = Key::D;

pub static KEY_ROTATION_UP: Key = Key::Up;
pub static KEY_ROTATION_DOWN: Key = Key::Down;
pub static KEY_ROTATION_LEFT: Key = Key::Left;
pub static KEY_ROTATION_RIGHT: Key = Key::Right;

pub static KEY_INCREASE_OVERSAMPLING: Key = Key::P;
pub static KEY_DECREASE_OVERSAMPLING: Key = Key::O;

pub static KEY_INCREASE_DISTANCE: Key = Key::I;
pub static KEY_DECREASE_DISTANCE: Key = Key::U;

pub static KEY_INCREASE_TIME: Key = Key::Comma;
pub static KEY_DECREASE_TIME: Key = Key::M;

pub static KEY_INCREASE_CURRENT_INDEX: Key = Key::V;
pub static KEY_DECREASE_CURRENT_INDEX: Key = Key::C;

pub static KEY_NEXT_LOGGER_STATE: Key = Key::L;
pub static KEY_NEXT_FRAME_STATE: Key = Key::K;
pub static KEY_NEXT_METHOD_STATE: Key = Key::Semicolon;

pub static MOUSE_MOVE_ADD: MouseButton = MouseButton::Left;
pub static MOUSE_MOVE_REMOVE: MouseButton = MouseButton::Right;
pub static MOUSE_WAIT_DROP_DO: MouseButton = MouseButton::Left;
pub static MOUSE_WAIT_DROP_CANCEL: MouseButton = MouseButton::Right;

pub static BUTTON_UNKNOWN: MouseButton = MouseButton::Unknown;
pub static KEY_UNKNOWN: Key = Key::Unknown;
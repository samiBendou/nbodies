use crate::common::*;
use crate::core;
use crate::physics::dynamics::body::{Body, Frame};
use crate::physics::dynamics::body::Cluster;
use crate::physics::dynamics::point::Point2;
use crate::physics::units;
use crate::physics::units::{Compound, Rescale, Serialize, Unit};
use crate::physics::units::date::Duration;
use crate::physics::vector::Vector2;
use crate::toggle;

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
    pub fn next(&mut self) {
        use State::*;
        *self = match self {
            Hide => Status,
            Status => Timing,
            Timing => Cinematic,
            Cinematic => Physics,
            Physics => Bodies,
            Bodies => Hide,
        };
    }
}

pub struct Logger {
    state: State,
    buffer: String,
    units: Units,
    px_units: Units,
}

impl Logger {
    pub fn new() -> Logger {
        Logger {
            state: State::Hide,
            buffer: String::from(""),
            units: Units::default(),
            px_units: Units::pixel(),
        }
    }

    pub fn update(&mut self) {
        self.state.next();
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
        bodies: &Cluster,
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
        self.buffer += &format!("\
*** status info ***\n\
{:#?}\n\
pressed mouse button: '{:?}'\n\
mouse at: {:?} (px)\n\
pressed keyboard key: '{:?}'\n",
                                status, input.button, input.cursor, input.key)[..];
    }

    fn log_timing(&mut self, step: &core::Step, config: &core::Config) {
        self.buffer += &format!("\
*** timing info ***\n\
{:?}\n\
updates per frame: {}\n\n\
*** scale ***\n\
{:?}",
                                step, config.updates_per_frame, config.scale
        );
    }

    fn log_cinematic(&mut self, bodies: &Cluster, config: &core::Config) {
        if bodies.is_empty() {
            return;
        }
        let mut current = bodies.current().shape.center.clone();
        current.scale_position(config.scale.distance);
        current.scale_speed(config.scale.distance);
        self.px_units.rescale(&current);
        self.buffer += &format!("\
*** current shape ***\n\
{}\n",
                                self.px_units.string_of(&current)
        );
    }

    fn log_physics(&mut self, bodies: &Cluster, config: &core::Config) {
        if bodies.is_empty() {
            return;
        }
        let current = bodies.current();
        self.units.rescale(current);
        self.buffer += &format!("
*** current body ***\n\
{}\n",
                                self.units.string_of(current)
        );
    }

    fn log_bodies(&mut self, bodies: &Cluster) {
        self.buffer += &format!("
*** body list ***\n\
{:?}\n",
                                bodies
        );
    }
}

pub struct Units {
    pub time: Unit,
    pub distance: Unit,
    pub mass: Unit,
    pub speed: Compound,
    pub acceleration: Compound,
}

impl Units {
    pub fn new(distance: Unit, mass: Unit, time: Unit) -> Units {
        let speed = distance.clone() / time.clone();
        let acceleration = speed.clone() / time.clone();
        Units {
            time,
            distance,
            mass,
            speed,
            acceleration,
        }
    }

    pub fn default() -> Units {
        use crate::physics::units::suffix::*;
        let time = Unit::from(units::Scale::from(Time::Second));
        let distance = Unit::from(units::Scale::from(Distance::Meter));
        let mass = Unit::from(units::Scale::from(Mass::Kilograms));
        Units::new(distance, mass, time)
    }

    pub fn pixel() -> Units {
        use crate::physics::units::suffix::*;
        let time = Unit::from(units::Scale::from(Time::Second));
        let distance = Unit::from(units::Scale::from(Distance::Pixel));
        let mass = Unit::from(units::Scale::from(Mass::Kilograms));
        Units::new(distance, mass, time)
    }
}

impl units::Rescale<Point2> for Units {
    fn rescale(&mut self, val: &Point2) -> &mut Self {
        self.distance.rescale(&val.position.magnitude());
        self.speed.units[0].rescale(&val.speed.magnitude());
        self.acceleration.units[0].rescale(&val.acceleration.magnitude());
        self
    }
}


impl units::Rescale<Body> for Units {
    fn rescale(&mut self, val: &Body) -> &mut Self {
        self.rescale(&val.shape.center);
        self
    }
}

impl units::Serialize<Point2> for Units {
    fn string_of(&self, val: &Point2) -> String {
        format!(
            "position: {}\nspeed: {}\nacceleration: {}",
            self.distance.string_of(&val.position),
            self.speed.string_of(&val.speed),
            self.acceleration.string_of(&val.acceleration),
        )
    }
}

impl units::Serialize<Body> for Units {
    fn string_of(&self, val: &Body) -> String {
        format!(
            "name: {}\nmass: {}\nposition: {}\nspeed: {}\nacceleration: {}",
            val.name,
            self.mass.string_of(&val.mass),
            self.distance.string_of(&val.shape.center.position),
            self.speed.string_of(&val.shape.center.speed),
            self.acceleration.string_of(&val.shape.center.acceleration),
        )
    }
}
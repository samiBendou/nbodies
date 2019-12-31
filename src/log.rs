use physics::dynamics;
use physics::dynamics::point::Point2;
use physics::units;
use physics::units::{Compound, Rescale, Serialize, Unit};
use physics::vector::Split;
use piston::input::Key;

use crate::common::*;
use crate::core;

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
    energy_units: Unit,
}

impl Logger {
    pub fn new() -> Logger {
        use physics::units::suffix::*;
        Logger {
            state: State::Hide,
            buffer: String::from(""),
            units: Units::default(),
            px_units: Units::pixel(),
            energy_units: Unit::from(units::Scale::from(Energy::Joules)),
        }
    }

    pub fn update(&mut self, key: &Key) {
        if *key == KEY_NEXT_LOGGER_STATE {
            self.state.next();
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
        cluster: &dynamics::Cluster,
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
            Cinematic => self.log_cinematic(cluster, config, status),
            Physics => self.log_physics(cluster, status),
            Bodies => self.log_cluster(cluster)
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
                                step, config.oversampling, config.scale
        );
    }

    fn log_cinematic(&mut self, cluster: &dynamics::Cluster, config: &core::Config, status: &core::Status) {
        let count = cluster.count();
        if count == 0 {
            return;
        }
        self.log_shape(cluster.current().unwrap(), config);
        if !status.is_waiting_to_add() || count == 1 {
            return;
        }
        self.log_shape(cluster.last().unwrap(), config);
    }

    fn log_physics(&mut self, cluster: &dynamics::Cluster, status: &core::Status) {
        let count = cluster.count();
        if count == 0 {
            return;
        }
        self.log_energy(cluster);
        self.log_body(cluster.current().unwrap());
        if !status.is_waiting_to_add() || count == 1 {
            return;
        }
        self.log_body(cluster.last().unwrap());
    }

    fn log_cluster(&mut self, cluster: &dynamics::Cluster) {
        self.buffer += &format!("
*** body list ***\n\
{:?}\n",
                                cluster
        );
    }
    fn log_shape(&mut self, body: &dynamics::Body, config: &core::Config) {
        let mut shape = body.shape.center;
        shape.scale_position(config.scale.distance);
        shape.scale_speed(config.scale.distance);
        self.px_units.rescale(&shape);
        self.buffer += &format!("\
*** {} ***\n\
{}\n",
                                body.name,
                                self.px_units.string_of(&shape)
        );
    }

    fn log_body(&mut self, body: &dynamics::Body) {
        self.units.rescale(body);
        self.buffer += &format!("
*** {} ***\n\
{}\n",
                                body.name,
                                self.units.string_of(body)
        );
    }

    fn log_energy(&mut self, cluster: &dynamics::Cluster) {
        use physics::dynamics::potentials;
        let kinetic_energy = cluster.kinetic_energy();
        let angular_momentum = cluster.angular_momentum();
        let potential_energy = cluster.potential_energy(|bodies, i| {
            bodies[i].mass * potentials::gravity(&bodies[i].shape.center, bodies)
        });
        let total_energy = kinetic_energy + potential_energy;
        self.energy_units.rescale(&total_energy);
        self.buffer += &format!("
total kinetic energy: {}
total potential energy: {}
total energy: {}
angular momentum: {:.5e}
\
",
                                self.energy_units.string_of(&kinetic_energy),
                                self.energy_units.string_of(&potential_energy),
                                self.energy_units.string_of(&total_energy),
                                angular_momentum
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
        use physics::units::suffix::*;
        let time = Unit::from(units::Scale::from(Time::Second));
        let distance = Unit::from(units::Scale::from(Distance::Meter));
        let mass = Unit::from(units::Scale::from(Mass::Kilograms));
        Units::new(distance, mass, time)
    }

    pub fn pixel() -> Units {
        use physics::units::suffix::*;
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


impl units::Rescale<dynamics::Body> for Units {
    fn rescale(&mut self, val: &dynamics::Body) -> &mut Self {
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
            self.acceleration.string_of(&val.acceleration.lower()),
        )
    }
}

impl units::Serialize<dynamics::Body> for Units {
    fn string_of(&self, val: &dynamics::Body) -> String {
        format!(
            "name: {}\nmass: {}\nposition: {}\nspeed: {}\nacceleration: {}",
            val.name,
            self.mass.string_of(&val.mass),
            self.distance.string_of(&val.shape.center.position),
            self.speed.string_of(&val.shape.center.speed),
            self.acceleration.string_of(&val.shape.center.acceleration.lower()),
        )
    }
}
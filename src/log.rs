use physics::dynamics;
use physics::geometry::common::coordinates::Homogeneous;
use physics::geometry::common::Metric;
use physics::geometry::point::Point3;
use physics::geometry::vector::Vector3;
use physics::units;
use physics::units::{Compound, Rescale, Serialize, Unit};
use piston::input::Key;

use crate::common::*;
use crate::common::Scale;
use crate::core;
use crate::draw::{Circle, Drawer};
use crate::keys::KEY_NEXT_LOGGER_STATE;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum State {
    Hide,
    Status,
    Config,
    Timing,
    Cinematic,
    Body,
    Physics,
    Bodies,
}

impl State {
    pub fn next(&mut self) {
        use State::*;
        *self = match self {
            Hide => Status,
            Status => Config,
            Config => Timing,
            Timing => Cinematic,
            Cinematic => Body,
            Body => Bodies,
            Bodies => Physics,
            Physics => Hide,
        };
    }
}

pub struct Logger {
    state: State,
    buffer: String,
    units: Units,
    px_units: Unit,
    energy_units: Unit,
}

impl Logger {
    pub fn new() -> Logger {
        use physics::units::suffix::*;
        Logger {
            state: State::Hide,
            buffer: String::from(""),
            units: Units::default(),
            px_units: Unit::from(units::Scale::from(Distance::Pixel)),
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
        drawer: &Drawer,
        status: &core::Status,
        config: &core::Config,
        input: &Input,
    ) {
        use crate::log::State::*;
        match self.state {
            Hide => (),
            Status => self.log_status(status, input),
            Config => self.log_config(config),
            Timing => self.log_timing(status, config),
            Cinematic => self.log_cinematic(cluster.current_index(), drawer, status),
            Body => self.log_bodies(cluster, status),
            Bodies => self.log_cluster(cluster),
            Physics => self.log_physics(cluster)
        };
        self.buffer += "\n";
        match self.state {
            Timing | Body | Cinematic | Physics => self.log_scale(&config.scale),
            _ => ()
        };
    }

    fn log_status(&mut self, status: &core::Status, input: &Input) {
        self.buffer += &format!("\
*** status info ***
{:#?}
pressed mouse button: '{:?}'
mouse at: {:?} (px)
pressed keyboard key: '{:?}'",
                                status, input.button, input.cursor, input.key)[..];
    }

    fn log_config(&mut self, config: &core::Config) {
        self.buffer += &format!("*** config info ***\n{:#?}", config)[..];
    }

    fn log_timing(&mut self, status: &core::Status, config: &core::Config) {
        self.buffer += &format!("\
*** timing info ***
{:?}
oversampling: {}",
                                status.step, config.oversampling);
    }

    fn log_cinematic(&mut self, current: usize, drawer: &Drawer, status: &core::Status) {
        let len = drawer.circles.len();
        if len == 0 {
            return;
        }
        self.log_shape(&drawer.circles[current]);
        if status.is_waiting_to_add() && len == 1 {
            self.buffer += "\n";
            self.log_shape(&drawer.circles.last().unwrap());
        }
        self.buffer += "\n";
    }

    fn log_bodies(&mut self, cluster: &dynamics::Cluster, status: &core::Status) {
        let len = cluster.len();
        if len == 0 {
            return;
        }
        self.log_body(cluster.current().unwrap());
        if status.is_waiting_to_add() && len == 1 {
            self.buffer += "\n";
            self.log_body(cluster.last().unwrap());
        }
    }

    fn log_cluster(&mut self, cluster: &dynamics::Cluster) {
        let barycenter = cluster.barycenter();
        self.units.rescale(&barycenter.state);
        self.buffer += &format!("*** barycenter ***\n{}",
                                self.units.string_of(&barycenter.state));
        for body in cluster.bodies.iter() {
            self.buffer += "\n";
            self.log_body(body)
        }
    }
    fn log_physics(&mut self, cluster: &dynamics::Cluster) {
        self.log_energy(cluster);
    }

    fn log_shape(&mut self, circle: &Circle) {
        let circle = Vector3::from_homogeneous(circle.trajectory.last());
        self.px_units.rescale(&circle.magnitude());
        self.buffer += &format!("*** circle ***\n{}",
                                self.px_units.string_of(&circle));
    }

    fn log_body(&mut self, body: &dynamics::Body) {
        self.units.rescale(body);
        self.buffer += &format!("{}", self.units.string_of(body));
    }

    fn log_energy(&mut self, cluster: &dynamics::Cluster) {
        use physics::dynamics::potentials;
        let kinetic_energy = cluster.kinetic_energy();
        let angular_momentum = cluster.angular_momentum();
        let potential_energy = cluster.potential_energy(|bodies, i| {
            bodies[i].center.mass * potentials::gravity(&bodies[i].center, bodies)
        });
        let total_energy = kinetic_energy + potential_energy;
        self.energy_units.rescale(&total_energy);
        self.buffer += &format!("\
*** energy ***
kinetic energy: {}
potential energy: {}
total energy: {}
angular momentum: {:.10e}",
                                self.energy_units.string_of(&kinetic_energy),
                                self.energy_units.string_of(&potential_energy),
                                self.energy_units.string_of(&total_energy),
                                angular_momentum
        );

        let barycenter = cluster.barycenter();
        self.units.rescale(&barycenter.state);
        self.buffer += &format!("\n*** barycenter ***\n{}",
                                self.units.string_of(&barycenter.state));
    }

    fn log_scale(&mut self, scale: &Scale) {
        self.buffer += &format!("*** scale ***\n{:?}", scale);
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

impl units::Rescale<Point3> for Units {
    fn rescale(&mut self, val: &Point3) -> &mut Self {
        self.distance.rescale(&val.position.magnitude());
        self.speed.units[0].rescale(&val.speed.magnitude());
        self
    }
}


impl units::Rescale<dynamics::Body> for Units {
    fn rescale(&mut self, val: &dynamics::Body) -> &mut Self {
        self.rescale(&val.center.state);
        self
    }
}

impl units::Serialize<Point3> for Units {
    fn string_of(&self, val: &Point3) -> String {
        format!(
            "position: {}\nspeed: {}",
            self.distance.string_of(&val.position),
            self.speed.string_of(&val.speed),
        )
    }
}

impl units::Serialize<dynamics::Body> for Units {
    fn string_of(&self, val: &dynamics::Body) -> String {
        format!(
            "***{}***\nmass: {}\nposition: {}\nspeed: {}",
            val.name,
            self.mass.string_of(&val.center.mass),
            self.distance.string_of(&val.center.state.position),
            self.speed.string_of(&val.center.state.speed),
        )
    }
}
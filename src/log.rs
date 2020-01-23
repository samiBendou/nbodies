use dynamics::point::Point3;
use geomath::point;
use geomath::prelude::Metric;
use piston::input::Key;
use unitflow;
use unitflow::{Compound, Rescale, Serialize, Unit};

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
    Step,
    Cinematic,
    Points,
    Physics,
    Bodies,
}

impl State {
    pub fn next(&mut self) {
        use State::*;
        *self = match self {
            Hide => Status,
            Status => Config,
            Config => Step,
            Step => Cinematic,
            Cinematic => Points,
            Points => Bodies,
            Bodies => Physics,
            Physics => Hide,
        };
    }
}

pub struct Logger {
    state: State,
    buffer: String,
    units: Units,
    px_unit: Unit,
    energy_unit: Unit,
    time_unit: Unit,
    distance_unit: Unit,
}

impl Logger {
    pub fn new() -> Logger {
        use unitflow::suffix::*;
        Logger {
            state: State::Hide,
            buffer: String::from(""),
            units: Units::default(),
            px_unit: Unit::from(unitflow::Scale::from(Distance::Pixel)),
            energy_unit: Unit::from(unitflow::Scale::from(Energy::Joules)),
            time_unit: Unit::from(unitflow::Scale::from(Time::Second)),
            distance_unit: Unit::from(unitflow::Scale::from(Distance::Meter)),
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
        simulator: &core::Simulator,
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
            Step => self.log_step(&status.step),
            Cinematic => self.log_cinematic(simulator.current_index(), drawer, status),
            Points => self.log_points(simulator, status),
            Bodies => self.log_cluster(&simulator.cluster),
            Physics => self.log_physics(simulator)
        };
        self.buffer += "\n";
        match self.state {
            Step | Points | Cinematic | Physics => self.log_scale(&config.scale),
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

    fn log_step(&mut self, step: &Step) {
        use unitflow::*;
        let frame = step.frame.value();
        let system = step.system.value();
        let framerate = (1. / frame).floor() as u8;
        let framerate_system = (1. / system).floor() as u8;
        self.time_unit.rescale(&frame);
        self.buffer += &format!("*** step info ***
dt: {} fps: {} (update)
dt: {} fps: {} (system)
total: {:?}
simulated: {:?}",
                                self.time_unit.string_of(&frame),
                                framerate,
                                self.time_unit.string_of(&system),
                                framerate_system,
                                step.total,
                                step.simulated);
    }

    fn log_cinematic(&mut self, current: usize, drawer: &Drawer, status: &core::Status) {
        self.buffer += &format!("*** transform ***{:?}\n", drawer.transform);
        self.buffer += &format!("*** inverse transform ***{:?}\n", drawer.inverse_transform);

        let len = drawer.circles.len();
        if len == 0 {
            return;
        }
        self.log_shape(&drawer.circles[current]);
        if status.is_waiting_to_add() && len != 1 {
            self.buffer += "\n";
            self.log_shape(&drawer.circles.last().unwrap());
        }
    }

    fn log_points(&mut self, simulator: &core::Simulator, status: &core::Status) {
        let len = simulator.cluster.len();
        if len == 0 {
            return;
        }
        let point = simulator.current().unwrap();
        let body = &simulator.system[simulator.current_index()];
        self.log_point(point, body.name.as_str());
        if status.is_waiting_to_add() && len != 1 {
            self.buffer += "\n";
            self.log_point(simulator.last().unwrap(), &simulator.system[simulator.last_index()].name);
        }
    }

    fn log_cluster(&mut self, cluster: &dynamics::Cluster) {
        let barycenter = cluster.barycenter();
        self.units.rescale(&barycenter.state);
        self.buffer += &format!("*** barycenter ***\n{}", self.units.string_of(&barycenter.state));
        for point in cluster.points.iter() {
            self.buffer += "\n";
            self.log_point(point, "")
        }
    }
    fn log_physics(&mut self, simulator: &core::Simulator) {
        self.log_energy(&simulator.cluster);
        self.buffer += &format!("\n*** orbital ***\n{:#?}", simulator.system[simulator.current_index()].orbit);
    }

    fn log_shape(&mut self, circle: &Circle) {
        let circle = circle.trajectory.last();
        self.px_unit.rescale(&circle.magnitude());
        self.buffer += &format!("*** circle ***\n{}", self.px_unit.string_of(circle));
    }

    fn log_point(&mut self, point: &Point3, name: &str) {
        self.units.rescale(point);
        self.buffer += &format!("*** {} ***\n", name);
        self.buffer += &format!("{}", self.units.string_of(point));
    }

    fn log_energy(&mut self, cluster: &dynamics::Cluster) {
        use dynamics::potentials;
        let kinetic_energy = cluster.kinetic_energy();
        let angular_momentum = cluster.angular_momentum();
        let potential_energy = cluster.potential_energy(|points, i| {
            points[i].mass * potentials::gravity(&points[i], points)
        });
        let total_energy = kinetic_energy + potential_energy;
        self.energy_unit.rescale(&total_energy);
        self.buffer += &format!("\
*** energy ***
kinetic energy: {}
potential energy: {}
total energy: {}
angular momentum: {:.10e}",
                                self.energy_unit.string_of(&kinetic_energy),
                                self.energy_unit.string_of(&potential_energy),
                                self.energy_unit.string_of(&total_energy),
                                angular_momentum
        );

        let barycenter = cluster.barycenter();
        self.units.rescale(&barycenter.state);
        self.buffer += &format!("\n*** barycenter ***\n{}",
                                self.units.string_of(&barycenter.state));
    }

    fn log_scale(&mut self, scale: &Scale) {
        self.time_unit.rescale(&scale.time);
        self.distance_unit.rescale(&scale.distance);
        self.buffer += &format!("*** scale ***
time: {} per (second)
distance: {} per (meter)",
                                self.time_unit.string_of(&scale.time),
                                self.distance_unit.string_of(&scale.distance));
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
        use unitflow::suffix::*;
        let time = Unit::from(unitflow::Scale::from(Time::Second));
        let distance = Unit::from(unitflow::Scale::from(Distance::Meter));
        let mass = Unit::from(unitflow::Scale::from(Mass::Kilograms));
        Units::new(distance, mass, time)
    }

    pub fn pixel() -> Units {
        use unitflow::suffix::*;
        let time = Unit::from(unitflow::Scale::from(Time::Second));
        let distance = Unit::from(unitflow::Scale::from(Distance::Pixel));
        let mass = Unit::from(unitflow::Scale::from(Mass::Kilograms));
        Units::new(distance, mass, time)
    }
}

impl unitflow::Rescale<point::Point3> for Units {
    fn rescale(&mut self, val: &point::Point3) -> &mut Self {
        self.distance.rescale(&val.position.magnitude());
        self.speed.units[0].rescale(&val.speed.magnitude());
        self
    }
}


impl unitflow::Rescale<Point3> for Units {
    fn rescale(&mut self, val: &Point3) -> &mut Self {
        self.rescale(&val.state);
        self
    }
}

impl unitflow::Serialize<point::Point3> for Units {
    fn string_of(&self, val: &point::Point3) -> String {
        format!(
            "position: {}\nspeed: {}",
            self.distance.string_of(&val.position),
            self.speed.string_of(&val.speed),
        )
    }
}

impl unitflow::Serialize<Point3> for Units {
    fn string_of(&self, val: &Point3) -> String {
        format!(
            "mass: {}\nposition: {}\nspeed: {}",
            self.mass.string_of(&val.mass),
            self.distance.string_of(&val.state.position),
            self.speed.string_of(&val.state.speed),
        )
    }
}
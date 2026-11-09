//! The run the window shows: the discrete run (the 2D flow model) or the physics run (the plant on Avian), and what a frame
//! needs from it: where each pallet is, the turntables' angles, the decks' heights, each object's state and statistics.

use std::collections::BTreeMap;

use bevy::math::{DMat3, DQuat, DVec3};
use intracraft_avian::AvianBench;
use intracraft_des::flow::sim::PalletView;
use intracraft_des::flow::{FlowSimulation, Statistics, simulate};
use intracraft_des::time::{SimTime, from_seconds, to_seconds};
use intracraft_library::Library;
use intracraft_library::definition::Value;
use intracraft_library::layout::{Instance, Layout};
use intracraft_library::params::{Values, instance_values, num};
use intracraft_physics::flow::PhysicsFlow;

/// The speeds each run plays at: simulated seconds per real second. Physics is real time at ×1.
pub const DISCRETE_SPEEDS: [f64; 5] = [1.0, 10.0, 100.0, 1000.0, 10_000.0];
pub const PHYSICS_SPEEDS: [f64; 3] = [1.0, 2.0, 5.0];
/// A frame never runs more physics than this, s: a slow frame slows the run instead of stalling the window.
const MAX_PHYSICS_PER_FRAME: f64 = 0.1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Module {
    Discrete,
    Physics,
}

/// A pallet as the window draws it.
#[derive(Clone, Debug, PartialEq)]
pub struct Shown {
    pub id: u64,
    pub type_id: String,
    /// The centre of its underside, and its turn: its own x along the pallet type's length.
    pub centre: DVec3,
    pub rotation: DQuat,
    pub load_height: f64,
    /// Its own height, where the load stands.
    pub height: f64,
}

/// What a frame draws.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Picture {
    pub pallets: Vec<Shown>,
    pub angles: BTreeMap<String, f64>,
    pub heights: BTreeMap<String, f64>,
    pub status: BTreeMap<String, &'static str>,
}

enum Engine {
    Discrete(Box<FlowSimulation>),
    Physics(Box<PhysicsFlow>),
}

pub struct Run {
    pub module: Module,
    pub layout: Layout,
    pub seed: u64,
    pub playing: bool,
    pub speed: f64,
    pub error: Option<String>,
    engine: Engine,
    library: Library,
}

impl Run {
    pub fn start(module: Module, layout: Layout, seed: u64, library: Library) -> Result<Run, String> {
        let engine = make(module, &layout, seed, &library)?;
        let speed = if module == Module::Discrete { 10.0 } else { 1.0 };
        Ok(Run { module, layout, seed, playing: false, speed, error: None, engine, library })
    }

    /// The same run again from the start.
    pub fn restart(&mut self) {
        match make(self.module, &self.layout, self.seed, &self.library) {
            Ok(engine) => (self.engine, self.error) = (engine, None),
            Err(e) => self.error = Some(e),
        }
    }

    pub fn speeds(&self) -> &'static [f64] {
        match self.module {
            Module::Discrete => &DISCRETE_SPEEDS,
            Module::Physics => &PHYSICS_SPEEDS,
        }
    }

    pub fn now(&self) -> SimTime {
        match &self.engine {
            Engine::Discrete(s) => s.now(),
            Engine::Physics(f) => f.now(),
        }
    }

    /// Moves the run on by `real` seconds of the window's time at its speed.
    pub fn advance(&mut self, real: f64) {
        if !self.playing || self.error.is_some() {
            return;
        }
        match &mut self.engine {
            Engine::Discrete(s) => {
                let target = s.now() + from_seconds(real * self.speed).max(1);
                if let Err(e) = s.run_until(target) {
                    self.error = Some(e);
                    self.playing = false;
                }
            }
            Engine::Physics(f) => {
                let target = f.now() + from_seconds((real * self.speed).min(MAX_PHYSICS_PER_FRAME)).max(1);
                f.run_until(target);
            }
        }
    }

    pub fn statistics(&mut self) -> Statistics {
        match &mut self.engine {
            Engine::Discrete(s) => s.statistics(),
            Engine::Physics(f) => f.statistics(),
        }
    }

    /// The layout's warnings, as one line each.
    pub fn issues(&self) -> Vec<String> {
        let issues = match &self.engine {
            Engine::Discrete(s) => &s.issues,
            Engine::Physics(f) => &f.issues,
        };
        issues.iter().map(|i| format!("{}: {}", i.instance_id, i.message)).collect()
    }

    /// Pallets that fell off a deck (physics only).
    pub fn fallen(&self) -> Option<u64> {
        match &self.engine {
            Engine::Discrete(_) => None,
            Engine::Physics(f) => Some(f.fallen_pallets()),
        }
    }

    pub fn picture(&mut self) -> Picture {
        match &mut self.engine {
            Engine::Discrete(s) => {
                let snap = s.snapshot();
                let pallets = snap.pallets.iter().filter_map(|p| discrete_pallet(&self.layout, &self.library, p, &snap.heights)).collect();
                Picture { pallets, angles: snap.angles, heights: snap.heights, status: snap.status }
            }
            Engine::Physics(f) => {
                let pallets = f
                    .pallets()
                    .into_iter()
                    .map(|p| {
                        let (x, z) = (DVec3::from_array(p.axes.0), DVec3::from_array(p.axes.1));
                        let along_width =
                            self.library.reference.pallet_type(&p.type_id).is_some_and(|t| (t.length - p.size[0]).abs() > 1e-3);
                        Shown {
                            id: p.id,
                            type_id: p.type_id,
                            centre: DVec3::from_array(p.centre),
                            rotation: rotation_of(x, z, along_width),
                            load_height: p.load_height,
                            height: p.size[1],
                        }
                    })
                    .collect();
                Picture { pallets, angles: f.angles(), heights: f.heights(), status: f.status() }
            }
        }
    }

    pub fn clock(&self) -> String {
        let s = to_seconds(self.now());
        format!("{:02}:{:02}:{:04.1}", (s / 3600.0).floor(), (s / 60.0).floor() % 60.0, s % 60.0)
    }
}

fn make(module: Module, layout: &Layout, seed: u64, library: &Library) -> Result<Engine, String> {
    Ok(match module {
        Module::Discrete => Engine::Discrete(Box::new(simulate(layout, library, seed)?)),
        Module::Physics => Engine::Physics(Box::new(PhysicsFlow::on(Box::new(AvianBench::new()), layout, library, seed)?)),
    })
}

/// A pallet's turn from its own x and z axes in the world; a pallet that travels short side first is turned a quarter, so
/// the model's x, its type's length, lies across its travel.
pub fn rotation_of(x: DVec3, z: DVec3, along_width: bool) -> DQuat {
    let (x, z) = (x.normalize_or(DVec3::X), z.normalize_or(DVec3::Z));
    let y = z.cross(x).normalize_or(DVec3::Y);
    let travel = DQuat::from_mat3(&DMat3::from_cols(x, y, x.cross(y)));
    if along_width { travel * DQuat::from_rotation_y(std::f64::consts::FRAC_PI_2) } else { travel }
}

/// Where a pallet of the discrete run is: on its carrier's path, at the carrier's surface (a lift's deck height, a gravity
/// line's slope).
pub fn discrete_pallet(layout: &Layout, library: &Library, p: &PalletView, heights: &BTreeMap<String, f64>) -> Option<Shown> {
    let carrier = layout.instance(&p.carrier)?;
    let values = library.get(&carrier.definition_id).map(|d| instance_values(d, &carrier.params)).unwrap_or_default();
    let along = p.front - p.length / 2.0;
    let (c, s) = (p.heading.cos(), p.heading.sin());
    let local = (p.path_x + along * c, p.path_z - along * s);
    let surface = match heights.get(&p.carrier) {
        Some(h) => *h,
        None if p.slope != 0.0 => num(&values, "exitHeight", 0.1) + p.slope * (num(&values, "length", 0.0) - along),
        None => surface_of(&values),
    };
    let centre = world(carrier, local, surface);
    let travel = DQuat::from_rotation_y(carrier.yaw + p.heading) * DQuat::from_rotation_z(-p.slope.atan());
    let along_width = library.reference.pallet_type(&p.type_id).is_some_and(|t| (t.length - p.length).abs() > 1e-3);
    let rotation = if along_width { travel * DQuat::from_rotation_y(std::f64::consts::FRAC_PI_2) } else { travel };
    Some(Shown { id: p.id, type_id: p.type_id.clone(), centre, rotation, load_height: p.load_height, height: p.height })
}

/// The conveying height of an object that has one, m above its base.
fn surface_of(v: &Values) -> f64 {
    ["surfaceHeight", "conveyorHeight", "entryHeight"].iter().find_map(|k| v.get(*k).and_then(Value::number)).unwrap_or(0.6)
}

/// A point `(x, z)` of an instance's own frame at `height` above its base, in the world.
fn world(i: &Instance, (x, z): (f64, f64), height: f64) -> DVec3 {
    let (s, c) = i.yaw.sin_cos();
    DVec3::new(i.position.x + x * c + z * s, i.position.y + height, i.position.z - x * s + z * c)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FLOW_T1: &str = include_str!("../../../../packages/scenarios/scenarios/layouts/flow-t1.intracraft.json");
    const REFERENCE: &str = include_str!("../../../../packages/scenarios/scenarios/layouts/reference-all-five.intracraft.json");

    fn project(text: &str) -> intracraft_library::layout::Project {
        intracraft_library::layout::Project::read(text).unwrap()
    }

    #[test]
    fn a_pallet_on_its_way_stands_on_the_conveyor_in_both_runs() {
        for module in [Module::Discrete, Module::Physics] {
            let p = project(FLOW_T1);
            let mut run = Run::start(module, p.layout, p.seed, Library::built_in().unwrap()).unwrap();
            (run.playing, run.speed) = (true, 1.0);
            for _ in 0..60 {
                run.advance(0.1);
            }
            let pic = run.picture();
            let pallet = pic.pallets.first().unwrap_or_else(|| panic!("{module:?}: no pallet"));
            assert!((pallet.centre.y - 0.6).abs() < 0.01, "{module:?}: {}", pallet.centre.y);
            assert!(pallet.centre.x > 0.5 && pallet.centre.x < 3.5, "{module:?}: {}", pallet.centre.x);
            assert!((pallet.rotation * DVec3::X).x > 0.99, "{module:?}: the pallet runs long side first along +x");
        }
    }

    #[test]
    fn both_runs_report_the_turntables_and_the_same_pallets_come_out() {
        let p = project(REFERENCE);
        let mut out = Vec::new();
        for module in [Module::Discrete, Module::Physics] {
            let mut run = Run::start(module, p.layout.clone(), p.seed, Library::built_in().unwrap()).unwrap();
            run.playing = true;
            while to_seconds(run.now()) < 120.0 {
                run.advance(0.1 / run.speed);
            }
            assert_eq!(run.picture().angles.len(), 2, "{module:?}");
            out.push(run.statistics().get("sink1").and_then(|s| s.get("palletsReceived")).copied());
        }
        assert_eq!(out[0], out[1]);
    }

    #[test]
    fn a_short_side_first_pallet_turns_its_model_a_quarter() {
        let q = rotation_of(DVec3::X, DVec3::Z, true);
        assert!((q * DVec3::X).z.abs() > 0.99);
        let tilted = rotation_of(DVec3::new(1.0, -0.04, 0.0), DVec3::Z, false);
        assert!((tilted * DVec3::X).y < -0.03);
    }
}

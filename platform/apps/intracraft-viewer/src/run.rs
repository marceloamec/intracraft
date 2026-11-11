//! The run the window shows: the discrete run (the 2D flow model) or the physics run (the plant on Avian), and what a frame
//! needs from it: where each pallet is, the turntables' angles, the decks' heights, each object's state and statistics.

use std::collections::BTreeMap;

use intracraft_avian::AvianBench;
use intracraft_des::flow::{FlowSimulation, Statistics, simulate};
use intracraft_des::time::{SimTime, from_seconds, to_seconds};
use intracraft_library::Library;
use intracraft_library::layout::Layout;
use intracraft_physics::flow::PhysicsFlow;
use intracraft_scene::pose::{PalletShow, along_width, discrete_pallet, rotation_of, storage_show};

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

/// What a frame draws.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Picture {
    pub pallets: Vec<PalletShow>,
    pub angles: BTreeMap<String, f64>,
    pub heights: BTreeMap<String, f64>,
    /// How far each part of a stacker crane has moved from where the scene draws it, m.
    pub shifts: BTreeMap<String, [f64; 3]>,
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

    /// Pallets that fell: off a conveyor that runs off its open end, or (physics) off any deck.
    pub fn fallen(&self) -> u64 {
        match &self.engine {
            Engine::Discrete(s) => s.fallen_pallets(),
            Engine::Physics(f) => f.fallen_pallets(),
        }
    }

    pub fn picture(&mut self) -> Picture {
        match &mut self.engine {
            Engine::Discrete(s) => {
                let snap = s.snapshot();
                let mut pallets: Vec<PalletShow> =
                    snap.pallets.iter().filter_map(|p| discrete_pallet(&self.layout, &self.library, p, &snap.heights)).collect();
                let (stored, shifts) = storage_show(&self.layout, &self.library, &snap.storage);
                pallets.extend(stored);
                Picture { pallets, angles: snap.angles, heights: snap.heights, shifts, status: snap.status }
            }
            Engine::Physics(f) => {
                let pallets = f
                    .pallets()
                    .into_iter()
                    .map(|p| PalletShow {
                        rotation: rotation_of(p.axes.0, p.axes.1, along_width(&self.library, &p.type_id, p.size[0])),
                        id: p.id,
                        type_id: p.type_id,
                        source: p.source,
                        centre: p.centre,
                        height: p.size[1],
                        load_height: p.load_height,
                        wrapped: p.wrapped,
                    })
                    .collect();
                let mut pallets: Vec<PalletShow> = pallets;
                let (stored, shifts) = storage_show(&self.layout, &self.library, &f.storage());
                pallets.extend(stored);
                Picture { pallets, angles: f.angles(), heights: f.heights(), shifts, status: f.status() }
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
            assert!((pallet.centre[1] - 0.6).abs() < 0.01, "{module:?}: {}", pallet.centre[1]);
            assert!(pallet.centre[0] > 0.5 && pallet.centre[0] < 3.5, "{module:?}: {}", pallet.centre[0]);
            let along = intracraft_scene::pose::rotate(pallet.rotation, [1.0, 0.0, 0.0]);
            assert!(along[0] > 0.99, "{module:?}: the pallet runs long side first along +x");
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
}

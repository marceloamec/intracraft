//! A high-bay warehouse laid out from its size (A22, ADR-0033): N storage aisles side by side (S26) and, in front of each,
//! the chain conveyors its stacker crane picks up from and sets down on. `plan` only says what to place and what to
//! expect of it; the Design module places it through its commands, and the CLI writes it as a layout file.

use std::collections::BTreeMap;

use intracraft_library::Library;
use intracraft_library::definition::{Definition, ParamType, Value};
use intracraft_library::layout::{Instance, Vec3};
use intracraft_library::params::{Values, instance_values, num};
use intracraft_library::rules::{Severity, validate_instance};

use crate::flow::storage::{LOAD_CLEARANCE, MAX_STATION_HEIGHTS, STATION_X, StorageData};

/// The library objects a warehouse is made of.
pub const AISLE: &str = "storage.aisle";
pub const CHAINS: &str = "conveyor.chain";
/// The section every object of a warehouse gets (ADR-0029).
pub const SECTION: &str = "HBW";
/// The room between the racks of two aisles, back to back, m.
pub const AISLE_GAP: f64 = 0.2;
/// The gap between two objects joined port to port, m (the library's).
const PORT_GAP: f64 = 0.02;
/// The most aisles one warehouse is laid out with.
pub const MAX_AISLES: usize = 20;
/// A level pitch worked out from the load is rounded up to this, m.
const PITCH_STEP: f64 = 0.05;

/// What the chain conveyors of one station height are for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StationUse {
    InAndOut,
    In,
    Out,
}

impl StationUse {
    pub fn takes_in(self) -> bool {
        self != StationUse::Out
    }

    pub fn gives_out(self) -> bool {
        self != StationUse::In
    }
}

/// A height with pick-up and deposit stations: the conveying height of its chain conveyors, m, and their use.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Station {
    pub height: f64,
    pub usage: StationUse,
}

/// What the wizard asks (A22 §1), in SI.
#[derive(Clone, Debug, PartialEq)]
pub struct WarehouseSpec {
    /// The rack's length along the aisles and its height, m: they give the bays and the levels.
    pub length: f64,
    pub height: f64,
    pub aisles: usize,
    /// Pallets deep on each side of an aisle: 1 or 2.
    pub deep: usize,
    /// The load on a EUR pallet, m: it gives the level pitch, unless `level_pitch` says it.
    pub load_height: f64,
    pub level_pitch: Option<f64>,
    /// One to three heights, lowest first.
    pub stations: Vec<Station>,
    /// The length of every pick-up and deposit chain conveyor, m.
    pub conveyor_length: f64,
    /// Other values of every aisle (its crane, its operation), by parameter key, in SI.
    pub aisle_values: Values,
    /// Where the front of the first aisle is, on its centre line, and how the warehouse is turned (rad).
    pub origin: Vec3,
    pub yaw: f64,
}

impl Default for WarehouseSpec {
    fn default() -> Self {
        WarehouseSpec {
            length: 20.0,
            height: 15.0,
            aisles: 2,
            deep: 1,
            load_height: 1.2,
            level_pitch: None,
            stations: vec![Station { height: 0.6, usage: StationUse::InAndOut }],
            conveyor_length: 2.4,
            aisle_values: Values::new(),
            origin: Vec3::new(0.0, 0.0, 0.0),
            yaw: 0.0,
        }
    }
}

/// What a specification gives (A22 §2–3).
#[derive(Clone, Debug, PartialEq)]
pub struct WarehousePlan {
    pub bays: usize,
    pub levels: usize,
    pub level_pitch: f64,
    /// Storage positions of the whole warehouse.
    pub positions: usize,
    /// The racks' length along the aisles, the warehouse's width across them and its height, m.
    pub size: [f64; 3],
    /// From one aisle's centre line to the next, m.
    pub aisle_pitch: f64,
    /// The mean time of a crane's single cycle (storage only) and of a combined cycle (a storage and a retrieval), s.
    pub single_cycle: f64,
    pub combined_cycle: f64,
    /// The objects to place, aisle by aisle: the aisle, then its chain conveyors.
    pub instances: Vec<Instance>,
}

impl WarehousePlan {
    /// Pallets an aisle stores in an hour doing single cycles, and pallets it stores and gives out doing combined ones.
    pub fn pallets_per_hour(&self) -> (f64, f64) {
        (3600.0 / self.single_cycle, 3600.0 / self.combined_cycle)
    }
}

/// The most aisles of `deep` pallets deep that fit across `width` m.
pub fn aisles_that_fit(width: f64, deep: usize, aisle_values: &Values, library: &Library) -> usize {
    let Some(d) = library.get(AISLE) else { return 0 };
    let module = module_width(&instance_values(d, aisle_values), deep);
    if !width.is_finite() || width < module {
        return 0;
    }
    (((width + AISLE_GAP) / (module + AISLE_GAP) + 1e-9).floor() as usize).min(MAX_AISLES)
}

/// The warehouse a specification asks for; an error says which value to change.
pub fn plan(spec: &WarehouseSpec, library: &Library) -> Result<WarehousePlan, String> {
    let aisle = library.get(AISLE).ok_or_else(|| format!("the library has no {AISLE}"))?;
    let chains = library.get(CHAINS).ok_or_else(|| format!("the library has no {CHAINS}"))?;
    for (what, v) in [("length", spec.length), ("height", spec.height), ("load height", spec.load_height)] {
        if !v.is_finite() || v < 0.0 {
            return Err(format!("the {what} must be a number, 0 or more"));
        }
    }
    if !(1..=MAX_AISLES).contains(&spec.aisles) {
        return Err(format!("a warehouse has 1 to {MAX_AISLES} aisles, not {}", spec.aisles));
    }
    if !(1..=2).contains(&spec.deep) {
        return Err(format!("an aisle is 1 or 2 pallets deep, not {}", spec.deep));
    }
    if !(1..=MAX_STATION_HEIGHTS).contains(&spec.stations.len()) {
        return Err(format!("an aisle has stations at 1 to {MAX_STATION_HEIGHTS} heights, not {}", spec.stations.len()));
    }
    for pair in spec.stations.windows(2) {
        if pair[1].height < pair[0].height + 0.5 {
            return Err("the station heights go from the lowest up, at least 0.5 m apart".into());
        }
    }
    let base = instance_values(aisle, &spec.aisle_values);
    let pallet = library.reference.pallet_type("eur").map_or(0.144, |t| t.height);
    let level_pitch = match spec.level_pitch {
        Some(p) => p,
        None => ((pallet + spec.load_height + LOAD_CLEARANCE) / PITCH_STEP - 1e-9).ceil() * PITCH_STEP,
    };
    let (bay_pitch, first) = (num(&base, "bayPitch", 1.0), num(&base, "firstLevelHeight", 0.6));
    if level_pitch <= 0.0 || bay_pitch <= 0.0 {
        return Err("the bay pitch and the level pitch must be more than 0".into());
    }
    let bays = (spec.length / bay_pitch + 1e-9).floor();
    let levels = ((spec.height - first) / level_pitch + 1e-9).floor();
    if bays < 1.0 {
        return Err(format!("a rack {} m long holds no bay of {bay_pitch} m", spec.length));
    }
    if levels < 1.0 {
        return Err(format!("a rack {} m high holds no level: the first is at {first} m and each takes {level_pitch:.2} m", spec.height));
    }

    let mut own = spec.aisle_values.clone();
    let number = |own: &mut Values, key: &str, v: f64| _ = own.insert(key.to_string(), Value::Number(v));
    number(&mut own, "bays", bays);
    number(&mut own, "levels", levels);
    number(&mut own, "deep", spec.deep as f64);
    number(&mut own, "levelPitch", round_mm(level_pitch));
    number(&mut own, "pdLevels", spec.stations.len() as f64);
    for (n, station) in spec.stations.iter().enumerate() {
        number(&mut own, &format!("pdHeight{}", n + 1), station.height);
    }
    in_range(aisle, &own)?;
    if let Some(issue) = validate_instance(aisle, &own, &library.reference).into_iter().find(|i| i.severity == Severity::Error) {
        return Err(issue.message);
    }
    let mut conveyor = Values::new();
    number(&mut conveyor, "length", spec.conveyor_length);
    in_range(chains, &conveyor)?;

    let values = instance_values(aisle, &own);
    let data = StorageData::read(&values);
    let module = module_width(&values, spec.deep);
    let aisle_pitch = module + AISLE_GAP;
    let surface = num(&instance_values(chains, &conveyor), "surfaceHeight", 0.6);
    let (sin, cos) = spec.yaw.sin_cos();
    let at = |x: f64, y: f64, z: f64| Vec3::new(spec.origin.x + x * cos + z * sin, spec.origin.y + y, spec.origin.z - x * sin + z * cos);
    let section: BTreeMap<String, Value> = BTreeMap::from([("section".to_string(), Value::Text(SECTION.into()))]);
    let instance = |id: String, name: String, definition: &Definition, position: Vec3, yaw: f64, params: Values| Instance {
        id,
        definition_id: definition.id.clone(),
        definition_version: definition.version.clone(),
        name,
        position,
        yaw,
        params,
        attributes: section.clone(),
    };
    let mut instances = Vec::new();
    for k in 0..spec.aisles {
        let z = k as f64 * aisle_pitch;
        let (id, name) = (format!("aisle{:02}", k + 1), format!("Aisle {:02}", k + 1));
        instances.push(instance(id.clone(), name.clone(), aisle, at(0.0, 0.0, z), spec.yaw, own.clone()));
        for (n, station) in spec.stations.iter().enumerate() {
            // A conveyor's own height, on a base that makes up the rest; a station lower than that is the conveyor's own.
            let mut params = conveyor.clone();
            let base = if station.height < surface - 1e-9 {
                number(&mut params, "surfaceHeight", station.height);
                in_range(chains, &params)?;
                0.0
            } else {
                station.height - surface
            };
            if station.usage.takes_in() {
                let from = at(-(spec.conveyor_length + PORT_GAP), base, z - data.front_reach());
                let (id, name) = (format!("{id}_in{}", n + 1), format!("{name} in {}", n + 1));
                instances.push(instance(id, name, chains, from, spec.yaw, params.clone()));
            }
            if station.usage.gives_out() {
                let from = at(-PORT_GAP, base, z + data.front_reach());
                let (id, name) = (format!("{id}_out{}", n + 1), format!("{name} out {}", n + 1));
                instances.push(instance(id, name, chains, from, spec.yaw + std::f64::consts::PI, params.clone()));
            }
        }
    }

    let entry = spec.stations.iter().find(|s| s.usage.takes_in()).or(spec.stations.first()).map_or(0.6, |s| s.height);
    let exit = spec.stations.iter().find(|s| s.usage.gives_out()).or(spec.stations.first()).map_or(0.6, |s| s.height);
    let (single_cycle, combined_cycle) = cycles(&data, entry, exit);
    Ok(WarehousePlan {
        bays: data.bays,
        levels: data.levels,
        level_pitch: data.level_pitch,
        positions: data.positions() * spec.aisles,
        size: [data.bays as f64 * data.bay_pitch, spec.aisles as f64 * module + (spec.aisles - 1) as f64 * AISLE_GAP, data.rack_height()],
        aisle_pitch,
        single_cycle,
        combined_cycle,
        instances,
    })
}

/// An aisle's mean single cycle, s: over every position, there from the entry station and back, with a fork cycle at
/// each end. And its combined cycle through the two representative points of the usual guideline for crane cycle times,
/// (1/5 length, 2/3 height) and (2/3 length, 1/5 height): entry → first → second → exit, with four fork cycles.
fn cycles(data: &StorageData, entry: f64, exit: f64) -> (f64, f64) {
    let from = [STATION_X, entry, 0.0];
    let positions = data.positions().max(1);
    let total: f64 = (0..data.positions())
        .map(|s| 2.0 * data.move_time(from, data.place_of(s)) + data.fork_cycle(None) + data.fork_cycle(Some(s)))
        .sum();
    let (length, height) = (data.bays as f64 * data.bay_pitch, data.rack_height());
    let (p1, p2) = ([length / 5.0, height * 2.0 / 3.0, 0.0], [length * 2.0 / 3.0, height / 5.0, 0.0]);
    let moves = data.move_time(from, p1) + data.move_time(p1, p2) + data.move_time(p2, [STATION_X, exit, 0.0]);
    (total / positions as f64, moves + 4.0 * data.fork_cycle(None))
}

/// The width of an aisle with its two rows, `deep` pallets deep, m.
fn module_width(values: &Values, deep: usize) -> f64 {
    num(values, "aisleWidth", 1.5) + 2.0 * deep as f64 * num(values, "positionDepth", 1.3)
}

fn round_mm(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

/// Every number among `own` lies within its parameter's range, and every key is a parameter that can be set.
fn in_range(definition: &Definition, own: &Values) -> Result<(), String> {
    for (key, value) in own {
        let p =
            definition.parameter(key).filter(|p| !p.derived).ok_or_else(|| format!("{} has no value {key} to set", definition.name.en))?;
        match (p.kind, value) {
            (ParamType::Number | ParamType::Integer, Value::Number(x)) => {
                let below = p.min.is_some_and(|m| *x < m - 1e-9);
                let above = p.max.is_some_and(|m| *x > m + 1e-9);
                if !x.is_finite() || below || above {
                    let range =
                        format!("{} to {}", p.min.map_or("any".into(), |m| m.to_string()), p.max.map_or("any".into(), |m| m.to_string()));
                    return Err(format!("{}: {} would be {x}; it goes from {range}", definition.name.en, p.label.en.to_lowercase()));
                }
            }
            (ParamType::Enum, Value::Text(t)) if p.options.as_ref().is_some_and(|o| !o.contains(t)) => {
                return Err(format!("{}: {} can't be {t:?}", definition.name.en, p.label.en.to_lowercase()));
            }
            (ParamType::Enum | ParamType::Text, Value::Text(_)) | (ParamType::Boolean, Value::Bool(_)) => {}
            _ => return Err(format!("{}: {} takes another kind of value", definition.name.en, p.label.en.to_lowercase())),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flow::simulate;
    use crate::time::from_seconds;
    use intracraft_library::layout::Layout;
    use intracraft_library::snap::derive_connections;

    fn library() -> Library {
        Library::built_in().unwrap()
    }

    #[test]
    fn the_defaults_give_two_aisles_of_20_bays_and_8_levels_with_their_chain_conveyors() {
        let plan = plan(&WarehouseSpec::default(), &library()).unwrap();
        // 20 m / 1.0 m; (15 − 0.6) / 1.6 m: a EUR pallet (0.144) with 1.2 m of load and 0.25 m of clearance, in 50 mm steps.
        assert_eq!((plan.bays, plan.levels, plan.positions), (20, 9, 720));
        assert!((plan.level_pitch - 1.6).abs() < 1e-9);
        // Two modules of 4.1 m and 0.2 m between them.
        assert!((plan.size[1] - 8.4).abs() < 1e-9 && (plan.aisle_pitch - 4.3).abs() < 1e-9, "{:?}", plan.size);
        let names: Vec<&str> = plan.instances.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(names, ["Aisle 01", "Aisle 01 in 1", "Aisle 01 out 1", "Aisle 02", "Aisle 02 in 1", "Aisle 02 out 1"]);
        assert!(plan.instances.iter().all(|i| i.attributes.get("section") == Some(&Value::Text("HBW".into()))));
    }

    #[test]
    fn every_conveyor_joins_its_aisle_s_port_at_its_height_and_nothing_else() {
        let spec = WarehouseSpec {
            aisles: 3,
            deep: 2,
            stations: vec![
                Station { height: 0.6, usage: StationUse::In },
                Station { height: 4.6, usage: StationUse::Out },
                Station { height: 8.6, usage: StationUse::InAndOut },
            ],
            origin: Vec3::new(12.0, 0.0, -7.0),
            yaw: 0.7,
            ..WarehouseSpec::default()
        };
        let lib = library();
        let plan = plan(&spec, &lib).unwrap();
        let layout = Layout { instances: plan.instances.clone(), ..Layout::default() };
        let mut joined: Vec<String> = derive_connections(&layout, &lib)
            .unwrap()
            .iter()
            .map(|c| format!("{}.{} → {}.{}", c.from.instance_id, c.from.port_id, c.to.instance_id, c.to.port_id))
            .collect();
        joined.sort();
        let mut expected = Vec::new();
        for k in 1..=3 {
            let a = format!("aisle{k:02}");
            expected.extend([
                format!("{a}.OUT2 → {a}_out2.A"),
                format!("{a}.OUT3 → {a}_out3.A"),
                format!("{a}_in1.B → {a}.IN1"),
                format!("{a}_in3.B → {a}.IN3"),
            ]);
        }
        expected.sort();
        assert_eq!(joined, expected);
        // Double deep: modules of 6.7 m.
        assert!((plan.aisle_pitch - 6.9).abs() < 1e-9);
        // The conveyors of the upper heights stand on a base that makes up the height.
        let out2 = plan.instances.iter().find(|i| i.id == "aisle01_out2").unwrap();
        assert!((out2.position.y - 4.0).abs() < 1e-9);
    }

    #[test]
    fn estimates_the_single_and_the_combined_cycle_of_an_aisle() {
        let plan = plan(&WarehouseSpec::default(), &library()).unwrap();
        // Combined, by hand: station (-0.4, 0.6) to (4, 10) to (13.33, 3) and back to the station, 4 fork cycles of 8 s.
        //   First move: 9.4 m up at 1 m/s and 0.5 m/s2 takes 11.4 s, longer than 4.4 m along.
        //   Second: 7 m down takes 9 s, longer than 9.33 m along (8.64 s, never reaching 3 m/s).
        //   Third: 13.73 m along takes 2 x sqrt(13.73 / 0.5) = 10.48 s, longer than 2.4 m down.
        let by_hand = 11.4 + 9.0 + 2.0 * ((13.0 + 1.0 / 3.0 + 0.4) / 0.5_f64).sqrt() + 32.0;
        assert!((plan.combined_cycle - by_hand).abs() < 1e-6, "{} / {by_hand}", plan.combined_cycle);
        // A single cycle is two moves and two fork cycles: more than 16 s, and here well under two minutes.
        assert!(plan.single_cycle > 16.0 + 2.0 * 1.9 && plan.single_cycle < 60.0, "{}", plan.single_cycle);
        let (single, combined) = plan.pallets_per_hour();
        assert!(single > 60.0 && combined > 40.0 && combined < single);
    }

    #[test]
    fn says_which_value_to_change() {
        let lib = library();
        let refused = |spec: WarehouseSpec| plan(&spec, &lib).unwrap_err();
        assert!(refused(WarehouseSpec { aisles: 0, ..WarehouseSpec::default() }).contains("1 to 20 aisles"));
        assert!(refused(WarehouseSpec { length: 0.5, ..WarehouseSpec::default() }).contains("holds no bay"));
        assert!(refused(WarehouseSpec { length: 400.0, ..WarehouseSpec::default() }).contains("bays would be 400"));
        assert!(refused(WarehouseSpec { height: 1.0, ..WarehouseSpec::default() }).contains("holds no level"));
        assert!(refused(WarehouseSpec { deep: 3, ..WarehouseSpec::default() }).contains("1 or 2 pallets deep"));
        let high = vec![Station { height: 0.6, usage: StationUse::In }, Station { height: 14.9, usage: StationUse::Out }];
        assert!(refused(WarehouseSpec { stations: high, ..WarehouseSpec::default() }).contains("above the top level"));
        let unknown = Values::from([("speed".to_string(), Value::Number(2.0))]);
        assert!(refused(WarehouseSpec { aisle_values: unknown, ..WarehouseSpec::default() }).contains("no value speed"));
        let fast = Values::from([("travelSpeed".to_string(), Value::Number(9.0))]);
        assert!(refused(WarehouseSpec { aisle_values: fast, ..WarehouseSpec::default() }).contains("travel speed would be 9"));
    }

    #[test]
    fn as_many_aisles_as_fit_across_a_width() {
        let lib = library();
        // 4.1 m each and 0.2 m between: three take 12.7 m.
        assert_eq!(aisles_that_fit(12.7, 1, &Values::new(), &lib), 3);
        assert_eq!(aisles_that_fit(12.6, 1, &Values::new(), &lib), 2);
        assert_eq!(aisles_that_fit(12.7, 2, &Values::new(), &lib), 1);
        assert_eq!(aisles_that_fit(3.0, 1, &Values::new(), &lib), 0);
    }

    #[test]
    fn a_planned_warehouse_runs_without_a_warning_once_it_has_a_way_in_and_a_way_out() {
        let lib = library();
        let spec = WarehouseSpec { aisles: 2, ..WarehouseSpec::default() };
        let mut plan = plan(&spec, &lib).unwrap();
        // A source in front of every infeed and a sink after every outfeed.
        let mut ends = Vec::new();
        for i in &plan.instances {
            let end = |id: String, definition: &str, x: f64, yaw: f64, params: &[(&str, Value)]| Instance {
                id,
                definition_id: definition.into(),
                definition_version: "0.1.0".into(),
                name: String::new(),
                position: Vec3::new(x, 0.0, i.position.z),
                yaw,
                params: params.iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
                attributes: BTreeMap::new(),
            };
            if i.id.contains("_in") {
                let source = [("orientation", Value::Text("short".into())), ("interval", Value::Number(60.0))];
                ends.push(end(format!("{}_source", i.id), "flow.source", i.position.x - 0.42, 0.0, &source));
            } else if i.id.contains("_out") {
                ends.push(end(format!("{}_sink", i.id), "flow.sink", i.position.x - 2.42, std::f64::consts::PI, &[]));
            }
        }
        plan.instances.extend(ends);
        let layout = Layout { instances: plan.instances, ..Layout::default() };
        let mut sim = simulate(&layout, &lib, 3).unwrap();
        assert!(sim.issues.is_empty(), "{:?}", sim.issues);
        sim.run_until(from_seconds(3600.0)).unwrap();
        let stats = sim.statistics();
        for aisle in ["aisle01", "aisle02"] {
            assert!(stats[aisle]["palletsStored"] > 50.0 && stats[aisle]["palletsRetrieved"] > 50.0, "{:?}", stats[aisle]);
            assert!(stats[&format!("{aisle}_out1_sink")]["palletsReceived"] > 50.0);
        }
    }
}

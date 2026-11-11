//! The plant as the 2D run sees it (A06): one node per instance with the values its behaviour needs and the instance
//! connected to its output port, built from a layout and the library. A port of the reference's `network.ts`.
//!
//! Ported so far: straight conveyors (rollers, chains), sources, sinks and photoelectric sensors. Turntables, transfer
//! tables, process stations, lifts and gravity lines are still to come (phase R4 of A19); until then they are inert:
//! the run says so, and pallets stop in front of them.

use intracraft_library::Library;
use intracraft_library::definition::{Definition, Value};
use intracraft_library::layout::{Layout, Rule, Vec3};
use intracraft_library::params::{Values, instance_values, num};
use intracraft_library::reference::{PalletType, Reference};

use crate::flow::storage::{self, Retrieval, StorageData};
use crate::flow::wrap::WrapData;

/// `behaviors.des` of definitions the 2D run leaves out (scenery, S17).
pub const NO_2D_BEHAVIOR: &str = "des.none";
/// `behaviors.des` of a code block (S21): it carries no pallets; its code runs with the run (`sim::script`).
pub const SCRIPT_BEHAVIOR: &str = "des.script";
/// A catalogue pallet whose footprint differs more than this from its pallet type gets a warning, m.
pub const MODEL_MISMATCH: f64 = 0.05;

#[derive(Clone, Debug, PartialEq)]
pub struct FlowIssue {
    pub instance_id: String,
    pub rule: &'static str,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Base {
    pub id: String,
    pub name: String,
    pub position: Vec3,
    pub yaw: f64,
    pub surface_height: f64,
    /// The instance connected to its output port, the port there, and the gap to it (m).
    pub downstream: Option<String>,
    pub downstream_port: Option<String>,
    pub downstream_gap: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Long,
    Short,
}

fn side(p: &Values, key: &str) -> Side {
    if p.get(key) == Some(&Value::Text("short".into())) { Side::Short } else { Side::Long }
}

/// A straight conveyor (rollers or chains) without accumulation: all its pallets move together.
#[derive(Clone, Debug, PartialEq)]
pub struct LineData {
    pub length: f64,
    pub nominal_speed: f64,
    pub accel: f64,
    pub decel: f64,
    pub min_gap: f64,
    pub pallet_side_along_travel: Side,
    /// At an output that isn't connected, pallets run off the end and fall instead of stopping there (`openEnd` =
    /// `runOff`, ADR-0026).
    pub run_off: bool,
}

/// What a source creates (S09 §1.1): its footprint, long and short side, the pallet part's height and the load's.
#[derive(Clone, Debug, PartialEq)]
pub struct SourceItem {
    pub pallet_model: String,
    pub long: f64,
    pub short: f64,
    pub pallet_height: f64,
    pub load_height: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Generation {
    FixedInterval,
    Exponential,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SourceData {
    pub pallet_type: PalletType,
    pub orientation: Side,
    pub load_mass: f64,
    pub item: SourceItem,
    pub generation: Generation,
    pub interval: f64,
    pub max_pallets: f64,
    pub block_if_occupied: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SinkData {
    pub removal_time: f64,
}

/// A photoelectric sensor on a conveyor (S06): positions in the host's frame, m.
#[derive(Clone, Debug, PartialEq)]
pub struct SensorData {
    pub host: String,
    pub sensor_s: f64,
    pub sensor_lateral: f64,
    pub reflector_s: f64,
    pub reflector_lateral: f64,
    pub sensor_height: f64,
    pub reflector_height: f64,
    /// The height detection uses: the mean of both ends (exact for a level beam).
    pub beam_height: f64,
    pub range: f64,
    pub response_time: f64,
    pub filter: f64,
    pub active_on_dark: bool,
    pub output_signal: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitRule {
    Alternate,
    FirstExit,
    Random,
}

/// A station port of a turntable connected to the entry of another object.
#[derive(Clone, Debug, PartialEq)]
pub struct TurntableExit {
    pub port_id: String,
    /// The station's angle (rad): the direction a pallet leaves in, in the turntable's frame.
    pub angle: f64,
    pub target: String,
    pub gap: f64,
}

/// A chain or roller turntable (S03, S04): it receives, centres, turns to an exit station and discharges.
#[derive(Clone, Debug, PartialEq)]
pub struct TurntableData {
    pub platform_length: f64,
    /// The deck's conveying speed and ramp (chains or rollers).
    pub nominal_speed: f64,
    pub accel: f64,
    pub max_angular_speed: f64,
    pub angular_accel: f64,
    pub angular_decel: f64,
    pub settle_time: f64,
    /// The station angles (rad) the platform may stop at.
    pub stations: Vec<f64>,
    pub pallet_side_along_travel: Side,
    pub exit_rule: ExitRule,
    pub exits: Vec<TurntableExit>,
}

/// The station ports of a turntable and their angles (rad, right-handed about +Y; 0 = its own +x).
pub const STATION_PORTS: [(&str, f64); 4] =
    [("P0", 0.0), ("P90", std::f64::consts::FRAC_PI_2), ("P180", std::f64::consts::PI), ("P270", 3.0 * std::f64::consts::FRAC_PI_2)];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransferDirection {
    Right,
    Left,
    Bidirectional,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperationMode {
    RollersToChains,
    ChainsToRollers,
    Bidirectional,
}

/// A connected output port of a station and what it leads to.
#[derive(Clone, Debug, PartialEq)]
pub struct StationExit {
    pub target: String,
    pub gap: f64,
    /// The port of the target it reaches.
    pub port: String,
}

/// A roller-to-chain transfer table (S05): rollers along x (A → B), liftable chains across (C1 at −z, C2 at +z) that
/// raise the pallet and move it sideways without turning it.
#[derive(Clone, Debug, PartialEq)]
pub struct TransferData {
    pub length: f64,
    pub usable_width: f64,
    pub nominal_speed: f64,
    pub accel: f64,
    pub transfer_speed: f64,
    pub transfer_accel: f64,
    pub lift_time: f64,
    pub lower_time: f64,
    pub lift_stroke: f64,
    pub transfer_direction: TransferDirection,
    pub operation_mode: OperationMode,
    pub pallet_side_along_travel: Side,
    /// Its connected output ports (B, C1, C2) and where they lead.
    pub exits: std::collections::BTreeMap<String, StationExit>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessingMode {
    Fixed,
    Normal,
    Exponential,
}

/// The process station (S11): a short deck where one pallet at a time is processed for a time.
#[derive(Clone, Debug, PartialEq)]
pub struct ProcessData {
    pub length: f64,
    pub nominal_speed: f64,
    pub accel: f64,
    pub pallet_side_along_travel: Side,
    pub mode: ProcessingMode,
    /// The fixed time, or the distribution's mean, s.
    pub processing_time: f64,
    pub processing_std_dev: f64,
    /// A stretch wrapper's programme (S25): the time follows the load and the table turns.
    pub wrap: Option<WrapData>,
}

/// A port of a lift station: which deck end it is at and the deck's height there, m.
#[derive(Clone, Debug, PartialEq)]
pub struct LiftPort {
    pub port_id: String,
    /// At x = length (the far end); else at x = 0.
    pub at_end: bool,
    pub height: f64,
}

/// A manual lift table, a scissor lift table or a pallet elevator (S12–S14, ADR-0013): one deck takes a pallet at the
/// entry height, the station moves it to the exit height and gives it out.
#[derive(Clone, Debug, PartialEq)]
pub struct LiftData {
    pub length: f64,
    pub nominal_speed: f64,
    pub accel: f64,
    pub pallet_side_along_travel: Side,
    pub entry: LiftPort,
    /// The exit ports in order of preference (the elevator prefers OUT_NEAR, S14 §1).
    pub exit_ports: Vec<LiftPort>,
    pub up_speed: f64,
    pub down_speed: f64,
    pub vertical_accel: f64,
    /// The operator side of a manual lift table (S12), checked for a source or a sink there.
    pub operator_port: Option<String>,
    pub exits: std::collections::BTreeMap<String, StationExit>,
}

/// A gravity roller line (S15, ADR-0014): pallets roll down one by one to a separator and an end stop.
#[derive(Clone, Debug, PartialEq)]
pub struct GravityData {
    /// The plan length, m.
    pub length: f64,
    pub pallet_side_along_travel: Side,
    /// The fall per metre.
    pub slope: f64,
    /// The rollers' height at the entry, m, from the exit height and the slope.
    pub entry_height: f64,
    pub brake_speed: f64,
    pub rolling_resistance: f64,
    pub start_delay: f64,
    pub pickup_length: f64,
    pub separator_time: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    Line(LineData),
    Gravity(Box<GravityData>),
    Turntable(Box<TurntableData>),
    Transfer(Box<TransferData>),
    Process(Box<ProcessData>),
    Lift(Box<LiftData>),
    /// A storage aisle of a high-bay warehouse (S26).
    Storage(Box<StorageData>),
    Source(Box<SourceData>),
    Sink(SinkData),
    Sensor(SensorData),
    /// A behaviour the 2D run doesn't have (yet): it never accepts a pallet.
    Inert {
        behavior: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct FlowNode {
    pub base: Base,
    pub kind: Kind,
}

impl FlowNode {
    pub fn id(&self) -> &str {
        &self.base.id
    }

    /// The port it gives pallets out of.
    fn output_port(&self) -> Option<&'static str> {
        match self.kind {
            Kind::Line(_) => Some("B"),
            Kind::Source(_) | Kind::Process(_) | Kind::Gravity(_) => Some("OUT"),
            _ => None,
        }
    }
}

pub struct FlowNetwork {
    pub nodes: Vec<FlowNode>,
    pub issues: Vec<FlowIssue>,
    pub rules: Vec<Rule>,
}

/// The network of a layout whose connections are known (`derive_connections`).
pub fn build_flow_network(layout: &Layout, library: &Library) -> FlowNetwork {
    let mut issues = Vec::new();
    let mut nodes: Vec<FlowNode> = Vec::new();
    for instance in &layout.instances {
        let Some(definition) = library.get(&instance.definition_id) else { continue };
        if definition.behaviors.des == NO_2D_BEHAVIOR || definition.behaviors.des == SCRIPT_BEHAVIOR {
            continue;
        }
        let p = instance_values(definition, &instance.params);
        let base = Base {
            id: instance.id.clone(),
            name: instance.name.clone(),
            position: instance.position,
            yaw: instance.yaw,
            surface_height: num(&p, "surfaceHeight", 0.0),
            downstream: None,
            downstream_port: None,
            downstream_gap: 0.0,
        };
        let kind = create_kind(definition, &base, &p, &library.reference, &mut issues);
        nodes.push(FlowNode { base, kind });
    }
    for c in &layout.connections {
        let Some(from) = nodes.iter().position(|n| n.base.id == c.from.instance_id) else { continue };
        let Some(to) = nodes.iter().position(|n| n.base.id == c.to.instance_id) else { continue };
        if let Kind::Storage(s) = &mut nodes[to].kind {
            s.feeders.insert(c.to.port_id.clone(), c.from.instance_id.clone());
        }
        if matches!(nodes[from].kind, Kind::Turntable(_)) {
            let angle = STATION_PORTS.iter().find(|(p, _)| *p == c.from.port_id).map(|(_, a)| *a);
            if matches!(nodes[to].kind, Kind::Turntable(_)) {
                let text = format!("pallets cannot yet pass directly to the turntable {}", c.to.instance_id);
                issues.push(issue(&nodes[from].base, "turntableToTurntable", text));
            } else if let (Some(angle), Kind::Turntable(t)) = (angle, &mut nodes[from].kind) {
                t.exits.push(TurntableExit { port_id: c.from.port_id.clone(), angle, target: c.to.instance_id.clone(), gap: c.gap });
            }
            continue;
        }
        let exit = StationExit { target: c.to.instance_id.clone(), gap: c.gap, port: c.to.port_id.clone() };
        match &mut nodes[from].kind {
            Kind::Transfer(t) => {
                t.exits.insert(c.from.port_id.clone(), exit);
                continue;
            }
            Kind::Lift(l) => {
                l.exits.insert(c.from.port_id.clone(), exit);
                continue;
            }
            Kind::Storage(s) => {
                s.exits.insert(c.from.port_id.clone(), exit);
                continue;
            }
            _ => {}
        }
        if nodes[from].output_port() != Some(c.from.port_id.as_str()) {
            continue;
        }
        let b = &mut nodes[from].base;
        b.downstream = Some(c.to.instance_id.clone());
        b.downstream_port = Some(c.to.port_id.clone());
        b.downstream_gap = c.gap;
    }
    let chains: Vec<&str> = layout
        .instances
        .iter()
        .filter(|i| library.get(&i.definition_id).is_some_and(|d| d.behaviors.des == "des.chain_conveyor"))
        .map(|i| i.id.as_str())
        .collect();
    for n in &nodes {
        issues.extend(connection_issues(n, &nodes, &library.reference));
        if let Kind::Storage(s) = &n.kind {
            issues.extend(storage_issues(&n.base, s, &chains));
        }
    }
    FlowNetwork { nodes, issues, rules: layout.rules.clone() }
}

fn create_kind(d: &Definition, base: &Base, p: &Values, reference: &Reference, issues: &mut Vec<FlowIssue>) -> Kind {
    let text = |k: &str| p.get(k).and_then(Value::text).map(str::to_string);
    match d.behaviors.des.as_str() {
        "des.roller_conveyor" | "des.chain_conveyor" => {
            if let Some(mode) = text("accumulationMode").filter(|m| m != "none") {
                issues.push(issue(base, "accumulationNotSimulated", format!("accumulation \"{mode}\" is simulated as \"none\" for now")));
            }
            if let Some(dir) = text("direction").filter(|d| d != "forward") {
                issues.push(issue(base, "directionNotSimulated", format!("direction \"{dir}\" is simulated as \"forward\" for now")));
            }
            Kind::Line(LineData {
                length: num(p, "length", 0.0),
                nominal_speed: num(p, "nominalSpeed", 0.0),
                accel: num(p, "accel", 0.0),
                decel: num(p, "decel", 0.0),
                min_gap: num(p, "minGap", 0.0),
                pallet_side_along_travel: side(p, "palletSideAlongTravel"),
                run_off: p.get("openEnd").and_then(Value::text) == Some("runOff"),
            })
        }
        "des.turntable" => {
            let rule = text("exitRule");
            Kind::Turntable(Box::new(TurntableData {
                platform_length: num(p, "platformLength", 0.0),
                nominal_speed: num(p, "nominalSpeed", num(p, "loadSpeed", 0.0)),
                accel: num(p, "accel", 0.0),
                max_angular_speed: num(p, "maxAngularSpeed", 0.0),
                angular_accel: num(p, "angularAccel", 0.0),
                angular_decel: num(p, "angularDecel", 0.0),
                settle_time: num(p, "settleTime", 0.0),
                stations: parse_stations(p.get("stations")),
                pallet_side_along_travel: side(p, "palletSideAlongTravel"),
                exit_rule: match rule.as_deref() {
                    Some("firstExit") => ExitRule::FirstExit,
                    Some("random") => ExitRule::Random,
                    _ => ExitRule::Alternate,
                },
                exits: Vec::new(),
            }))
        }
        "des.transfer_rc" => Kind::Transfer(Box::new(TransferData {
            length: num(p, "length", 0.0),
            usable_width: num(p, "usableWidth", 0.0),
            nominal_speed: num(p, "nominalSpeed", 0.0),
            accel: num(p, "accel", 0.0),
            transfer_speed: num(p, "transferSpeed", 0.0),
            transfer_accel: num(p, "transferAccel", 0.0),
            lift_time: num(p, "liftTime", 0.0),
            lower_time: num(p, "lowerTime", 0.0),
            lift_stroke: num(p, "liftStroke", 0.0),
            transfer_direction: match text("transferDirection").as_deref() {
                Some("left") => TransferDirection::Left,
                Some("bidirectional") => TransferDirection::Bidirectional,
                _ => TransferDirection::Right,
            },
            operation_mode: match text("operationMode").as_deref() {
                Some("chainsToRollers") => OperationMode::ChainsToRollers,
                Some("bidirectional") => OperationMode::Bidirectional,
                _ => OperationMode::RollersToChains,
            },
            pallet_side_along_travel: side(p, "palletSideAlongTravel"),
            exits: Default::default(),
        })),
        "des.process_station" => Kind::Process(Box::new(ProcessData {
            length: num(p, "length", 0.0),
            nominal_speed: num(p, "nominalSpeed", 0.0),
            accel: num(p, "accel", 0.0),
            pallet_side_along_travel: side(p, "palletSideAlongTravel"),
            mode: match text("processingMode").as_deref() {
                Some("normal") => ProcessingMode::Normal,
                Some("exponential") => ProcessingMode::Exponential,
                _ => ProcessingMode::Fixed,
            },
            processing_time: num(p, "processingTime", 0.0),
            processing_std_dev: num(p, "processingStdDev", 0.0).max(0.0),
            wrap: None,
        })),
        "des.stretch_wrapper" => Kind::Process(Box::new(ProcessData {
            length: num(p, "length", 0.0),
            nominal_speed: num(p, "nominalSpeed", 0.0),
            accel: num(p, "accel", 0.0),
            pallet_side_along_travel: side(p, "palletSideAlongTravel"),
            mode: ProcessingMode::Fixed,
            processing_time: 0.0,
            processing_std_dev: 0.0,
            wrap: Some(WrapData {
                turn_speed: num(p, "turnSpeed", 0.0),
                turn_accel: num(p, "turnAccel", 0.0),
                bottom_wraps: num(p, "bottomWraps", 0.0),
                top_wraps: num(p, "topWraps", 0.0),
                film_width: num(p, "filmWidth", 0.0),
                overlap: num(p, "overlap", 0.0).clamp(0.0, 0.99),
                up_and_down: text("passes").as_deref() != Some("up"),
                attach_time: num(p, "attachTime", 0.0).max(0.0),
                cut_time: num(p, "cutTime", 0.0).max(0.0),
                max_load_height: num(p, "maxLoadHeight", 0.0),
            }),
        })),
        "des.lift_station" => match lift(d, p) {
            Ok(l) => Kind::Lift(Box::new(l)),
            Err(e) => {
                issues.push(issue(base, "behaviorNotAvailable", e));
                Kind::Inert { behavior: "des.lift_station".into() }
            }
        },
        storage::BEHAVIOR => Kind::Storage(Box::new(StorageData::read(p))),
        "des.gravity" => {
            let (length, slope, exit_height) = (num(p, "length", 0.0), num(p, "slope", 0.0), num(p, "exitHeight", 0.0));
            Kind::Gravity(Box::new(GravityData {
                length,
                pallet_side_along_travel: side(p, "palletSideAlongTravel"),
                slope,
                entry_height: exit_height + slope * length,
                brake_speed: num(p, "brakeSpeed", 0.0),
                rolling_resistance: num(p, "rollingResistance", 0.0),
                start_delay: num(p, "startDelay", 0.0).max(0.0),
                pickup_length: num(p, "pickupLength", 0.0),
                separator_time: num(p, "separatorTime", 0.0).max(0.0),
            }))
        }
        "des.source" => source(base, p, reference, issues),
        "des.sink" => {
            let after_time = text("removalMode").as_deref() == Some("afterTime");
            Kind::Sink(SinkData { removal_time: if after_time { num(p, "removalTime", 0.0) } else { 0.0 } })
        }
        "des.photoeye" => {
            let diffuse = text("type").as_deref() == Some("diffuse");
            let (sensor_s, sensor_lateral, sensor_height) =
                (num(p, "sensorS", 0.0), num(p, "sensorLateral", 0.0), num(p, "sensorHeight", 0.0));
            let reflector_height = if diffuse { sensor_height } else { num(p, "reflectorHeight", sensor_height) };
            Kind::Sensor(SensorData {
                host: text("host").unwrap_or_default(),
                sensor_s,
                sensor_lateral,
                // A diffuse sensor is a beam straight across the conveyor for now.
                reflector_s: if diffuse { sensor_s } else { num(p, "reflectorS", sensor_s) },
                reflector_lateral: if diffuse { -sensor_lateral } else { num(p, "reflectorLateral", -sensor_lateral) },
                sensor_height,
                reflector_height,
                beam_height: (sensor_height + reflector_height) / 2.0,
                range: num(p, "range", f64::INFINITY),
                response_time: num(p, "responseTime", 0.0),
                filter: num(p, "filter", 0.0),
                active_on_dark: text("polarity").as_deref() == Some("activeOnDark"),
                output_signal: text("outputSignal").map(|s| s.trim().to_string()).unwrap_or_default(),
            })
        }
        behavior => {
            issues.push(issue(
                base,
                "behaviorNotAvailable",
                "has no 2D behaviour in the Rust engine yet; pallets stop in front of it".into(),
            ));
            Kind::Inert { behavior: behavior.to_string() }
        }
    }
}

/// The entry and exit ports of a lift station with their deck heights, from the definition's ports. The manual lift
/// table's ports work both ways: `flowDirection` makes OP (operator) or LINE the entry.
fn lift(d: &Definition, p: &Values) -> Result<LiftData, String> {
    let length = num(p, "length", 0.0);
    let ports = intracraft_library::ports::resolve_ports(d, p)?;
    let lift_port = |r: &intracraft_library::ports::ResolvedPort| LiftPort {
        port_id: r.id.clone(),
        at_end: r.local.x > length / 2.0,
        height: r.local.y,
    };
    let by_id = |id: &str| ports.iter().find(|r| r.id == id);
    use intracraft_library::definition::Direction;
    let manual = p.contains_key("flowDirection");
    let mut entry = ports.iter().find(|r| r.direction == Direction::In).map(lift_port);
    let mut exits: Vec<LiftPort> = ports.iter().filter(|r| r.direction == Direction::Out).map(lift_port).collect();
    if manual && let (Some(op), Some(line)) = (by_id("OP"), by_id("LINE")) {
        let out = p.get("flowDirection") == Some(&Value::Text("out".into()));
        entry = Some(lift_port(if out { line } else { op }));
        exits = vec![lift_port(if out { op } else { line })];
    }
    let entry = entry.ok_or_else(|| format!("{} has no entry port", d.id))?;
    if exits.is_empty() {
        return Err(format!("{} has no exit port", d.id));
    }
    // A speed each way (tables), or one speed both ways (elevator).
    let vertical = num(p, "verticalSpeed", 0.0);
    Ok(LiftData {
        length,
        nominal_speed: num(p, "nominalSpeed", 0.0),
        accel: num(p, "accel", 0.0),
        pallet_side_along_travel: side(p, "palletSideAlongTravel"),
        entry,
        exit_ports: exits,
        up_speed: num(p, "liftSpeed", vertical),
        down_speed: num(p, "lowerSpeed", vertical),
        vertical_accel: num(p, "verticalAccel", 0.0),
        operator_port: manual.then(|| "OP".to_string()),
        exits: Default::default(),
    })
}

/// A station list such as "0, 90, 180, 270" (degrees) as angles in rad; entries that aren't numbers are skipped.
fn parse_stations(value: Option<&Value>) -> Vec<f64> {
    let Some(Value::Text(t)) = value else { return STATION_PORTS.iter().map(|(_, a)| *a).collect() };
    t.split(|c: char| c == ',' || c == ';' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse::<f64>().ok())
        .filter(|d| d.is_finite())
        .map(|d| ((d % 360.0) + 360.0) % 360.0 * (std::f64::consts::PI / 180.0))
        .collect()
}

fn source(base: &Base, p: &Values, reference: &Reference, issues: &mut Vec<FlowIssue>) -> Kind {
    let text = |k: &str| p.get(k).and_then(Value::text).map(str::to_string);
    let wanted = text("palletType").unwrap_or_default();
    let pallet_type = match reference.pallet_type(&wanted).or_else(|| reference.pallet_types.first()) {
        Some(t) => t.clone(),
        None => {
            issues.push(issue(base, "noPalletTypes", "the library has no pallet types; it creates nothing".into()));
            return Kind::Inert { behavior: "des.source".into() };
        }
    };
    let pallet_model = text("palletModel").unwrap_or_else(|| "auto".into());
    let load_kind = text("loadKind").filter(|k| ["block", "none", "asset"].contains(&k.as_str())).unwrap_or_else(|| "block".into());
    let load_asset = text("loadAsset").unwrap_or_default();
    let nothing = pallet_model == "none" && load_kind == "none";
    if nothing {
        issues.push(issue(base, "sourceCreatesNothing", "has neither pallet nor load; it sends an empty pallet of its type".into()));
    }
    let pallet_model = if nothing { "auto".to_string() } else { pallet_model };
    let pallet = (pallet_model != "none").then_some((pallet_type.length, pallet_type.width));
    let load = match load_kind.as_str() {
        "block" => Some((pallet_type.length, pallet_type.width, num(p, "loadHeight", 0.0))),
        "asset" => reference.asset(&load_asset).map(|a| {
            let d = a.dimensions;
            (d.x.max(d.z), d.x.min(d.z), d.y)
        }),
        _ => None,
    };
    let item = SourceItem {
        long: pallet.map_or(0.0, |p| p.0).max(load.map_or(0.0, |l| l.0)),
        short: pallet.map_or(0.0, |p| p.1).max(load.map_or(0.0, |l| l.1)),
        pallet_height: if pallet.is_some() { pallet_type.height } else { 0.0 },
        load_height: load.map_or(0.0, |l| l.2),
        pallet_model: pallet_model.clone(),
    };
    if pallet_model != "auto"
        && pallet_model != "none"
        && let Some(a) = reference.asset(&pallet_model)
    {
        let d = a.dimensions;
        let mismatch = (d.x.max(d.z) - pallet_type.length).abs().max((d.x.min(d.z) - pallet_type.width).abs());
        if mismatch > MODEL_MISMATCH {
            issues.push(issue(
                base,
                "palletModelMismatch",
                format!("the pallet model differs by {} mm from the {} footprint", round1(mismatch * 1000.0), pallet_type.id),
            ));
        }
    }
    Kind::Source(Box::new(SourceData {
        orientation: side(p, "orientation"),
        load_mass: num(p, "loadMass", 0.0),
        item,
        generation: if text("generationMode").as_deref() == Some("exponential") {
            Generation::Exponential
        } else {
            Generation::FixedInterval
        },
        interval: num(p, "interval", 1.0),
        max_pallets: num(p, "maxPallets", 0.0),
        block_if_occupied: p.get("blockIfOccupied") != Some(&Value::Bool(false)),
        pallet_type,
    }))
}

fn connection_issues(node: &FlowNode, nodes: &[FlowNode], reference: &Reference) -> Vec<FlowIssue> {
    let next = node.base.downstream.as_ref().and_then(|d| nodes.iter().find(|n| &n.base.id == d));
    match &node.kind {
        Kind::Line(l) if next.is_none() && !l.run_off => {
            vec![issue(&node.base, "openEnd", "output is not connected; pallets stop at its end".into())]
        }
        Kind::Transfer(t) => transfer_issues(&node.base, t),
        Kind::Gravity(_) => match next.map(|n| &n.kind) {
            None => vec![issue(&node.base, "noExit", "OUT is not connected: place a sink for the operator there".into())],
            Some(Kind::Sink(_)) => vec![],
            Some(_) => vec![issue(&node.base, "exitNotSink", "OUT can only hand pallets to a sink (the operator) for now".into())],
        },
        Kind::Process(d) => process_issues(&node.base, d, next.is_some(), reference),
        Kind::Lift(l) => lift_issues(node, l, nodes),
        Kind::Turntable(t) if t.exits.is_empty() => {
            vec![issue(&node.base, "noExit", "no station is connected to the entry of another object; pallets stay on it".into())]
        }
        Kind::Sensor(s) => sensor_issues(&node.base, s, nodes.iter().find(|n| n.base.id == s.host), reference),
        Kind::Source(_) if !matches!(next.map(|n| &n.kind), Some(Kind::Line(_) | Kind::Lift(_))) => {
            vec![issue(&node.base, "sourceNotConnected", "must be connected to the entry of a conveyor; it creates no pallets".into())]
        }
        _ => vec![],
    }
}

fn process_issues(base: &Base, d: &ProcessData, connected: bool, reference: &Reference) -> Vec<FlowIssue> {
    let mut out = Vec::new();
    if !connected {
        out.push(issue(base, "noExit", "OUT is not connected; the pallet stays on it after processing".into()));
    }
    let along = reference.pallet_type("eur").map_or(0.0, |e| if d.pallet_side_along_travel == Side::Long { e.length } else { e.width });
    if d.length < along + 0.1 {
        out.push(issue(
            base,
            "deckTooShort",
            format!("the deck ({} mm) is shorter than the pallet plus 100 mm", round1(d.length * 1000.0)),
        ));
    }
    out
}

/// The exits of a lift station (S12–S14 §4) and the operator side of a manual table.
fn lift_issues(node: &FlowNode, l: &LiftData, nodes: &[FlowNode]) -> Vec<FlowIssue> {
    let base = &node.base;
    let mut out = Vec::new();
    let connected: Vec<&LiftPort> = l.exit_ports.iter().filter(|p| l.exits.contains_key(&p.port_id)).collect();
    if let Some(operator) = &l.operator_port {
        let feeder =
            nodes.iter().find(|n| n.base.downstream.as_deref() == Some(&base.id) && n.base.downstream_port.as_deref() == Some(operator));
        let taker = l.exits.get(operator);
        if feeder.is_none() && taker.is_none() {
            out.push(issue(
                base,
                "noOperator",
                format!("{operator} is not connected: place a source (input) or a sink (output) for the operator there"),
            ));
        }
        let operator_is_entry = &l.entry.port_id == operator;
        let taker_is_sink =
            taker.and_then(|t| nodes.iter().find(|n| n.base.id == t.target)).is_some_and(|n| matches!(n.kind, Kind::Sink(_)));
        let feeder_is_source = feeder.is_some_and(|n| matches!(n.kind, Kind::Source(_)));
        if (operator_is_entry && taker_is_sink) || (!operator_is_entry && feeder_is_source) {
            let text = if operator_is_entry {
                "an input table needs a source on OP, not a sink"
            } else {
                "an output table needs a sink on OP, not a source"
            };
            out.push(issue(base, "operatorMismatch", text.into()));
        }
        if !operator_is_entry {
            return out;
        }
    }
    let names = |ports: &[&LiftPort]| ports.iter().map(|p| p.port_id.clone()).collect::<Vec<String>>();
    if connected.is_empty() {
        let all: Vec<&LiftPort> = l.exit_ports.iter().collect();
        out.push(issue(base, "noExit", format!("{} is not connected; the pallet stays on it", names(&all).join(" / "))));
    } else if connected.len() > 1 {
        out.push(issue(
            base,
            "twoExits",
            format!("{} are both connected; {} is used", names(&connected).join(" and "), connected[0].port_id),
        ));
    }
    out
}

/// The stations of a storage aisle (S26 §6): none connected, exits without retrieval or retrieval without exits, a
/// station that isn't a chain conveyor, and a connected port of a height that isn't in use.
fn storage_issues(base: &Base, s: &StorageData, chains: &[&str]) -> Vec<FlowIssue> {
    let mut out = Vec::new();
    if s.feeders.is_empty() && s.exits.is_empty() {
        out.push(issue(base, "noStation", "no IN and no OUT is connected to a chain conveyor; its crane has nothing to do".into()));
        return out;
    }
    if s.retrieval != Retrieval::None && s.exits.is_empty() {
        out.push(issue(base, "retrievalWithoutExit", "no OUT is connected; pallets are stored and never leave".into()));
    }
    if s.retrieval == Retrieval::None && !s.exits.is_empty() {
        out.push(issue(base, "exitWithoutRetrieval", "an OUT is connected, but retrieval is set to none; nothing leaves".into()));
    }
    let others = s.feeders.iter().chain(s.exits.iter().map(|(port, e)| (port, &e.target)));
    for (port, other) in others {
        if s.station_height(port).is_none() {
            let text = format!("{port} is connected, but its station height is not in use; it is ignored");
            out.push(issue(base, "stationLevelNotUsed", text));
        } else if !chains.contains(&other.as_str()) {
            let text = format!("{port} is not connected to a chain conveyor; the forks only work on chains");
            out.push(issue(base, "stationNotOnChains", text));
        }
    }
    out
}

fn transfer_issues(base: &Base, t: &TransferData) -> Vec<FlowIssue> {
    let mut out = Vec::new();
    let side: &[&str] = match t.transfer_direction {
        TransferDirection::Right => &["C2"],
        TransferDirection::Left => &["C1"],
        TransferDirection::Bidirectional => &["C1", "C2"],
    };
    if t.operation_mode != OperationMode::ChainsToRollers && !side.iter().any(|p| t.exits.contains_key(*p)) {
        out.push(issue(base, "noExit", format!("the chain side {} is not connected; pallets from A stay on it", side.join("/"))));
    }
    if t.operation_mode != OperationMode::RollersToChains && !t.exits.contains_key("B") {
        out.push(issue(base, "noExit", "B is not connected; pallets from the chains stay on it".into()));
    }
    out
}

fn sensor_issues(base: &Base, s: &SensorData, host: Option<&FlowNode>, reference: &Reference) -> Vec<FlowIssue> {
    let Some(Kind::Line(line)) = host.map(|h| &h.kind) else {
        return vec![issue(base, "sensorWithoutHost", "is not mounted on a roller or chain conveyor; it detects nothing".into())];
    };
    let mut out = Vec::new();
    if (s.sensor_height - s.reflector_height).abs() > 1e-9 {
        out.push(issue(
            base,
            "inclinedBeam",
            format!("inclined beam: detection uses the mean height {} mm", round1(s.beam_height * 1000.0)),
        ));
    }
    let length = ((s.reflector_s - s.sensor_s).powi(2)
        + (s.reflector_lateral - s.sensor_lateral).powi(2)
        + (s.reflector_height - s.sensor_height).powi(2))
    .sqrt();
    if length > s.range {
        out.push(issue(
            base,
            "beamOutOfRange",
            format!("beam length {} mm exceeds the range {} mm", round1(length * 1000.0), round1(s.range * 1000.0)),
        ));
    }
    let gap = largest_void(reference, line.pallet_side_along_travel, s.beam_height);
    if gap > 0.0 {
        let seconds = if line.nominal_speed > 0.0 { gap / line.nominal_speed } else { 0.0 };
        out.push(issue(
            base,
            "beamInVoidBand",
            format!(
                "at {} mm the beam crosses pallet voids up to {} mm ({seconds:.2} s at {} m/s): expect several pulses per pallet; choose a continuous band or a filter",
                round1(s.beam_height * 1000.0),
                round1(gap * 1000.0),
                line.nominal_speed
            ),
        ));
    }
    out
}

/// The longest void a level beam meets in the EUR profile for the host's orientation, m.
fn largest_void(reference: &Reference, side: Side, height: f64) -> f64 {
    let Some(eur) = reference.pallet_type("eur") else { return 0.0 };
    let bands = if side == Side::Long { &eur.occupancy.along_length } else { &eur.occupancy.along_width };
    let Some(band) = bands.iter().find(|b| height >= b.from_height && height < b.to_height) else { return 0.0 };
    band.solid.windows(2).map(|w| w[1][0] - w[0][1]).fold(0.0, f64::max)
}

fn round1(v: f64) -> f64 {
    (v * 10.0 + 0.5).floor() / 10.0
}

fn issue(base: &Base, rule: &'static str, text: String) -> FlowIssue {
    let who = if base.name.is_empty() { &base.id } else { &base.name };
    FlowIssue { instance_id: base.id.clone(), rule, message: format!("{who}: {text}") }
}

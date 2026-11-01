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

/// `behaviors.des` of definitions the 2D run leaves out (scenery, S17).
pub const NO_2D_BEHAVIOR: &str = "des.none";
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

#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    Line(LineData),
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
            Kind::Source(_) => Some("OUT"),
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
        if definition.behaviors.des == NO_2D_BEHAVIOR {
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
        if !nodes.iter().any(|n| n.base.id == c.to.instance_id) {
            continue;
        }
        if nodes[from].output_port() != Some(c.from.port_id.as_str()) {
            continue;
        }
        let b = &mut nodes[from].base;
        b.downstream = Some(c.to.instance_id.clone());
        b.downstream_port = Some(c.to.port_id.clone());
        b.downstream_gap = c.gap;
    }
    for n in &nodes {
        issues.extend(connection_issues(n, &nodes, &library.reference));
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
            })
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
        Kind::Line(_) if next.is_none() => vec![issue(&node.base, "openEnd", "output is not connected; pallets stop at its end".into())],
        Kind::Sensor(s) => sensor_issues(&node.base, s, nodes.iter().find(|n| n.base.id == s.host), reference),
        Kind::Source(_) if !matches!(next.map(|n| &n.kind), Some(Kind::Line(_))) => {
            vec![issue(&node.base, "sourceNotConnected", "must be connected to the entry of a conveyor; it creates no pallets".into())]
        }
        _ => vec![],
    }
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

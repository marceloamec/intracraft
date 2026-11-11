//! IntraScript in the 2D run (A17 §14.2, S21 §5): the code blocks of the layout run with the plant. After each event
//! settles the plant and the If/Then rules, the run tells the code what happened (pallets created, entering and leaving
//! objects, signals changing), the code's writes take effect at once, and the plant settles again. The code's timers
//! are events of the run's own queue, and a turntable asks the code where a centred pallet goes.
//!
//! The 2D run is the engine IntraScript's [`Host`] talks to here: it describes each object of the layout to the checker
//! (its members: signals, parameters with their units, statistics) and answers reads and writes while the code runs.

use std::collections::BTreeMap;

use intracraft_library::Library;
use intracraft_library::definition::{ParamType, Value as LibValue};
use intracraft_library::layout::Layout;
use intracraft_library::params::instance_values;
use intracraft_script::host::{Host, Member, MemberKind, ModelInfo, ObjectInfo};
use intracraft_script::units::{Dim, dim_of_library_unit};
use intracraft_script::value::Dict;
use intracraft_script::vm::{BlockStats, ConsoleLine, Event as ScriptEvent, Runtime};
use intracraft_script::{CODE_BLOCK, Fault, Source, Value, code_name, snake_case};

use super::{Event, FlowSimulation, Pallet};
use crate::flow::network::Kind;
use crate::flow::station::Station;
use crate::time::{SimTime, US_PER_S};

/// The code blocks of a run and what the run remembers to tell them what changed.
pub(crate) struct Scripting {
    runtime: Runtime,
    map: ScriptMap,
    started: bool,
    last_signals: BTreeMap<String, bool>,
    last_carrier: BTreeMap<u64, String>,
    /// The pallets of the last look, so code can still read one that left the plant this instant (into a sink).
    last_pallets: BTreeMap<u64, Pallet>,
    ticket: Option<u64>,
    pub(crate) error: Option<intracraft_script::RunError>,
}

/// How the code's objects are the run's instances.
struct ScriptMap {
    ids: Vec<String>,
    names: Vec<String>,
    kinds: Vec<String>,
    /// Each object's address, in the layout's order (ADR-0029).
    addresses: Vec<String>,
    index: BTreeMap<String, u32>,
    params: Vec<BTreeMap<String, Value>>,
    /// Signal key in the run → object and member.
    signals: BTreeMap<String, (u32, String)>,
    /// Object and member → signal key and whether code may write it.
    members: BTreeMap<(u32, String), (String, bool)>,
    seed: u64,
}

/// What the code of a layout makes of it: the description the checker uses, and every block's code.
pub struct CodeBlocks {
    pub model: ModelInfo,
    /// Object index of each enabled block and its code.
    pub sources: Vec<(u32, String)>,
}

/// Describes a layout for IntraScript: every instance an object code can name, with its members.
pub fn describe(layout: &Layout, library: &Library) -> CodeBlocks {
    let network = crate::flow::build_flow_network(layout, library);
    let mut model = ModelInfo::default();
    let mut sources = Vec::new();
    let mut taken: Vec<String> = Vec::new();
    for (i, instance) in layout.instances.iter().enumerate() {
        let mut name = code_name(if instance.name.is_empty() { &instance.id } else { &instance.name });
        if taken.contains(&name) {
            name = format!("{name}_{}", code_name(&instance.id).trim_start_matches('_'));
        }
        taken.push(name.clone());
        let definition = library.get(&instance.definition_id);
        let node = network.nodes.iter().find(|n| n.base.id == instance.id);
        // The address: the section's code name and the object's (`Seccion1.Conveyor1`), or the object's alone (ADR-0029).
        let section = instance.attributes.get("section").and_then(LibValue::text).map(code_name);
        let address = match &section {
            Some(s) if !s.is_empty() => format!("{s}.{name}"),
            _ => name.clone(),
        };
        let mut info = ObjectInfo { name, id: instance.id.clone(), kind: instance.definition_id.clone(), address, ..Default::default() };
        let value = |n: &str, help: &str| Member { name: n.into(), kind: MemberKind::Value, help: help.into() };
        let signal = |n: &str, w: bool, help: &str| Member { name: n.into(), kind: MemberKind::Signal { writable: w }, help: help.into() };
        info.members.extend([value("name", "Its name in the layout"), value("id", "Its id"), value("kind", "Its library definition")]);
        if instance.definition_id == CODE_BLOCK {
            let enabled = instance.params.get("enabled").is_none_or(|v| !matches!(v, LibValue::Bool(false)));
            if enabled && let Some(LibValue::Text(code)) = instance.params.get("source") {
                sources.push((i as u32, code.clone()));
            }
            model.objects.push(info);
            continue;
        }
        if let Some(node) = node {
            let carries = matches!(
                node.kind,
                Kind::Line(_)
                    | Kind::Turntable(_)
                    | Kind::Transfer(_)
                    | Kind::Process(_)
                    | Kind::Lift(_)
                    | Kind::Gravity(_)
                    | Kind::Sink(_)
            );
            info.carries_pallets = carries;
            info.creates_pallets = matches!(node.kind, Kind::Source(_));
            info.decides_exit = matches!(node.kind, Kind::Turntable(_));
            match &node.kind {
                Kind::Line(_) => info.members.extend([
                    signal("run", true, "Input: False stops the conveyor (it brakes), True runs it"),
                    signal("running", false, "Output: the conveyor moves"),
                    signal("occupied", false, "Output: a pallet is on it"),
                ]),
                Kind::Source(_) => info.members.push(signal("enable", true, "Input: False stops making pallets")),
                Kind::Sensor(_) => info.members.push(signal("detected", false, "Output: the beam is interrupted")),
                _ => {}
            }
            if carries {
                if !matches!(node.kind, Kind::Line(_)) {
                    info.members.push(value("occupied", "A pallet is on it"));
                }
                info.members.extend([value("count", "Pallets on it now"), value("pallets", "Its pallets, the one furthest ahead first")]);
            }
            if info.decides_exit {
                info.members.push(value("exits", "The objects connected to its exits"));
            }
            info.members
                .extend([value("next", "The object its output leads to, or None"), value("stats", "Its statistics now, a dictionary")]);
        }
        if let Some(d) = definition {
            for p in &d.parameters {
                let key = snake_case(&p.key);
                if info.member(&key).is_none() {
                    info.members.push(Member { name: key, kind: MemberKind::Value, help: p.label.en.clone() });
                }
            }
        }
        model.objects.push(info);
    }
    CodeBlocks { model, sources }
}

impl Scripting {
    /// The run's scripting for a layout, or none when it has no enabled code block; the check errors of every block when
    /// a block has any (a run doesn't start then).
    pub(crate) fn build(layout: &Layout, library: &Library, sim: &FlowSimulation, seed: u64) -> Result<Option<Scripting>, String> {
        let blocks = describe(layout, library);
        if blocks.sources.is_empty() {
            return Ok(None);
        }
        let sources: Vec<Source> = blocks.sources.iter().map(|(o, code)| Source { object: *o, code }).collect();
        let runtime = Runtime::build(blocks.model.clone(), &sources, seed).map_err(|errors| {
            let mut lines = Vec::new();
            for (o, diags) in errors {
                let name = blocks.model.object(o).map_or("?", |i| i.name.as_str());
                for d in diags.iter().filter(|d| d.is_error()) {
                    lines.push(format!("Code block {name}, {d}"));
                }
            }
            format!("The code has errors; the run can't start.\n{}", lines.join("\n"))
        })?;
        let mut map = ScriptMap {
            ids: Vec::new(),
            names: Vec::new(),
            kinds: Vec::new(),
            addresses: Vec::new(),
            index: BTreeMap::new(),
            params: Vec::new(),
            signals: BTreeMap::new(),
            members: BTreeMap::new(),
            seed,
        };
        for (i, (instance, info)) in layout.instances.iter().zip(&blocks.model.objects).enumerate() {
            let o = i as u32;
            map.ids.push(instance.id.clone());
            map.names.push(instance.name.clone());
            map.addresses.push(info.address.clone());
            map.kinds.push(instance.definition_id.clone());
            map.index.insert(instance.id.clone(), o);
            let mut params = BTreeMap::new();
            if let Some(d) = library.get(&instance.definition_id)
                && instance.definition_id != CODE_BLOCK
            {
                let values = instance_values(d, &instance.params);
                for p in &d.parameters {
                    let Some(v) = values.get(&p.key) else { continue };
                    params.insert(snake_case(&p.key), to_script(p.kind, p.unit.as_deref(), v));
                }
            }
            map.params.push(params);
            let Some(node) = sim.nodes.iter().find(|n| n.base.id == instance.id) else { continue };
            let keys: Vec<(&str, String, bool)> = match &node.kind {
                Kind::Line(_) => vec![
                    ("run", format!("{}.run", instance.id), true),
                    ("running", format!("{}.running", instance.id), false),
                    ("occupied", format!("{}.occupied", instance.id), false),
                ],
                Kind::Source(_) => vec![("enable", format!("{}.enable", instance.id), true)],
                Kind::Sensor(_) => {
                    sim.sensors.iter().filter(|s| s.id == instance.id).map(|s| ("detected", s.signal.clone(), false)).collect()
                }
                _ => Vec::new(),
            };
            for (member, key, writable) in keys {
                if info.member(member).is_some() {
                    map.signals.insert(key.clone(), (o, member.to_string()));
                    map.members.insert((o, member.to_string()), (key, writable));
                }
            }
        }
        Ok(Some(Scripting {
            runtime,
            map,
            started: false,
            last_signals: BTreeMap::new(),
            last_carrier: BTreeMap::new(),
            last_pallets: BTreeMap::new(),
            ticket: None,
            error: None,
        }))
    }
}

/// A library value as code sees it: numbers with their unit as quantities.
fn to_script(kind: ParamType, unit: Option<&str>, v: &LibValue) -> Value {
    match v {
        LibValue::Bool(b) => Value::Bool(*b),
        LibValue::Text(t) => Value::str(t),
        LibValue::Number(x) => match unit.and_then(dim_of_library_unit) {
            Some(d) => Value::Qty(*x, d),
            None if kind == ParamType::Integer && x.fract() == 0.0 && x.abs() < 9.0e15 => Value::Int(*x as i64),
            None => Value::Float(*x),
        },
    }
}

impl FlowSimulation {
    /// Attaches the layout's code blocks to the run; an error lists the check errors when the code has any.
    pub(crate) fn attach_scripts(&mut self, layout: &Layout, library: &Library, seed: u64) -> Result<(), String> {
        if let Some(s) = Scripting::build(layout, library, self, seed)? {
            self.scripting = Some(Box::new(s));
            // The code's start runs at time 0, before the first pallet.
            self.sim.schedule(0, Event::Script, -1);
        }
        Ok(())
    }

    /// Tells the code what changed and runs it; true when its writes changed the plant, which must then settle again.
    pub(super) fn script(&mut self) -> Result<bool, String> {
        let Some(mut sc) = self.scripting.take() else { return Ok(false) };
        let result = self.script_with(&mut sc);
        if let Err(e) = &result {
            sc.error = Some(e.clone());
        }
        self.scripting = Some(sc);
        result.map_err(|e| e.to_string())
    }

    fn script_with(&mut self, sc: &mut Scripting) -> Result<bool, intracraft_script::RunError> {
        self.update_outputs();
        let carriers = self.carriers();
        let mut events = Vec::new();
        let mut gone = Vec::new();
        if sc.started {
            for (key, (o, member)) in &sc.map.signals {
                let now = self.signals.get(key).copied().unwrap_or(false);
                if now != sc.last_signals.get(key).copied().unwrap_or(false) {
                    events.push(ScriptEvent::Signal { object: *o, signal: member.clone(), value: now });
                }
            }
            let ids: std::collections::BTreeSet<u64> = carriers.keys().chain(sc.last_carrier.keys()).copied().collect();
            for id in ids {
                let (before, after) = (sc.last_carrier.get(&id), carriers.get(&id));
                if before == after {
                    continue;
                }
                let at = |c: &String| sc.map.index.get(c).copied();
                if let Some(b) = before.and_then(at) {
                    events.push(ScriptEvent::Leave { object: b, pallet: id });
                }
                match (before, after) {
                    (None, Some(c)) => {
                        if let Some(source) = self.source_feeding(c).and_then(|s| sc.map.index.get(&s).copied()) {
                            events.push(ScriptEvent::Create { source, pallet: id });
                        }
                        if let Some(o) = at(c) {
                            events.push(ScriptEvent::Enter { object: o, pallet: id });
                        }
                    }
                    (Some(_), Some(c)) => {
                        if let Some(o) = at(c) {
                            events.push(ScriptEvent::Enter { object: o, pallet: id });
                        }
                    }
                    (Some(b), None) => {
                        if let Some(sink) = self.sink_after(b).and_then(|s| sc.map.index.get(&s).copied()) {
                            events.push(ScriptEvent::Enter { object: sink, pallet: id });
                        }
                        gone.push(id);
                    }
                    (None, None) => {}
                }
            }
        }
        sc.last_signals = sc.map.signals.keys().map(|k| (k.clone(), self.signals.get(k).copied().unwrap_or(false))).collect();
        sc.last_carrier = carriers;
        let departed: BTreeMap<u64, Pallet> = gone.iter().filter_map(|id| sc.last_pallets.remove(id).map(|p| (*id, p))).collect();
        sc.last_pallets = self.all_pallets();
        let mut view = View { sim: self, map: &sc.map, departed: &departed, dirty: false };
        if sc.started {
            sc.runtime.react(&events, &mut view)?;
        } else {
            sc.started = true;
            sc.runtime.start(&mut view)?;
        }
        let dirty = view.dirty;
        for id in gone {
            sc.runtime.forget_pallet(id);
        }
        if let Some(t) = sc.ticket.take() {
            self.sim.cancel(t);
        }
        if let Some(w) = sc.runtime.next_wake() {
            let delay = (w - self.sim.now()).max(1);
            sc.ticket = Some(self.sim.schedule(delay as SimTime, Event::Script, -1));
        }
        Ok(dirty)
    }

    /// Asks the code where the pallet centred on each turntable goes, before the turntable chooses by its rule.
    pub(super) fn ask_exits(&mut self) {
        let Some(mut sc) = self.scripting.take() else { return };
        for k in 0..self.stations.len() {
            let Station::Turntable(t) = &self.stations[k] else { continue };
            let Some((pallet, targets)) = t.exit_question(&self.lines) else { continue };
            let Some(&object) = sc.map.index.get(&t.id) else { continue };
            if !sc.runtime.decides_exit(object) {
                if let Station::Turntable(t) = &mut self.stations[k] {
                    t.answer_exit(None);
                }
                continue;
            }
            let candidates: Vec<u32> = targets.iter().filter_map(|id| sc.map.index.get(id).copied()).collect();
            let none = BTreeMap::new();
            let mut view = View { sim: self, map: &sc.map, departed: &none, dirty: false };
            let answer = sc.runtime.choose_exit(object, pallet, &candidates, &mut view);
            let answer = match answer {
                Ok(a) => a.and_then(|o| sc.map.ids.get(o as usize).cloned()),
                Err(e) => {
                    self.script_fault.get_or_insert_with(|| e.to_string());
                    sc.error.get_or_insert(e);
                    None
                }
            };
            if let Station::Turntable(t) = &mut self.stations[k] {
                t.answer_exit(answer);
            }
        }
        self.scripting = Some(sc);
    }

    /// Every pallet of the run and the object it belongs to now.
    fn carriers(&self) -> BTreeMap<u64, String> {
        let mut out = BTreeMap::new();
        for l in &self.lines {
            for p in &l.pallets {
                out.insert(p.id, l.carrier.clone());
            }
        }
        for g in &self.gravity {
            for p in &g.pallets {
                out.insert(p.pallet.id, g.id.clone());
            }
        }
        out
    }

    fn all_pallets(&self) -> BTreeMap<u64, Pallet> {
        let mut out = BTreeMap::new();
        for l in &self.lines {
            for p in &l.pallets {
                out.insert(p.id, p.clone());
            }
        }
        for g in &self.gravity {
            for p in &g.pallets {
                out.insert(p.pallet.id, p.pallet.clone());
            }
        }
        out
    }

    /// The source whose pallets land on `carrier`.
    fn source_feeding(&self, carrier: &str) -> Option<String> {
        self.sources
            .iter()
            .find(|s| {
                self.line_after(s.downstream.as_deref(), s.downstream_port.as_deref()).is_some_and(|l| self.lines[l].carrier == carrier)
            })
            .map(|s| s.id.clone())
    }

    /// The sink a pallet leaving `carrier` goes into, if any.
    fn sink_after(&self, carrier: &str) -> Option<String> {
        let downstream: Vec<&str> = self
            .lines
            .iter()
            .filter(|l| l.carrier == carrier)
            .filter_map(|l| l.downstream.as_deref())
            .chain(self.gravity.iter().filter(|g| g.id == carrier).filter_map(|g| g.downstream.as_deref()))
            .chain(self.nodes.iter().filter(|n| n.base.id == carrier).filter_map(|n| n.base.downstream.as_deref()))
            .collect();
        downstream.into_iter().find(|d| self.sinks.iter().any(|s| s.id == *d)).map(str::to_string)
    }

    /// A pallet still in the plant: itself, the object carrying it and its speed now.
    fn find_pallet(&self, id: u64) -> Option<(Pallet, String, f64)> {
        let t = self.sim.now() as f64 / US_PER_S as f64;
        for l in &self.lines {
            if let Some(p) = l.pallets.iter().find(|p| p.id == id) {
                return Some((p.clone(), l.carrier.clone(), l.speed_at(t)));
            }
        }
        for g in &self.gravity {
            if let Some(p) = g.pallets.iter().find(|p| p.pallet.id == id) {
                return Some((p.pallet.clone(), g.id.clone(), g.speed_at(p, t)));
            }
        }
        None
    }

    /// What the code printed.
    pub fn script_console(&self) -> &[ConsoleLine] {
        self.scripting.as_ref().map_or(&[], |s| s.runtime.console())
    }

    /// A code block's variables now, as text.
    pub fn script_variables(&self, block_id: &str) -> Vec<(String, String)> {
        let Some(s) = &self.scripting else { return Vec::new() };
        s.map.index.get(block_id).map(|o| s.runtime.variables(*o)).unwrap_or_default()
    }

    /// The attributes the code gave a pallet, as text (for the load unit's panel, ADR-0029).
    pub fn pallet_attributes(&self, pallet: u64) -> Vec<(String, String)> {
        self.scripting.as_ref().map(|s| s.runtime.pallet_attributes(pallet)).unwrap_or_default()
    }

    /// The error that stopped the code, with its block and line.
    pub fn script_error(&self) -> Option<&intracraft_script::RunError> {
        self.scripting.as_ref().and_then(|s| s.error.as_ref())
    }

    /// Each code block's id and what it did.
    pub(super) fn script_statistics(&self) -> Vec<(String, BlockStats, usize)> {
        let Some(s) = &self.scripting else { return Vec::new() };
        s.runtime.block_stats().into_iter().filter_map(|(o, st, w)| s.map.ids.get(o as usize).map(|id| (id.clone(), st, w))).collect()
    }

    /// The end of the run: the code's `@on_finish` handlers.
    pub fn finish_scripts(&mut self) -> Result<(), String> {
        let Some(mut sc) = self.scripting.take() else { return Ok(()) };
        let none = BTreeMap::new();
        let mut view = View { sim: self, map: &sc.map, departed: &none, dirty: false };
        let r = sc.runtime.finish(&mut view);
        if let Err(e) = &r {
            sc.error = Some(e.clone());
        }
        self.scripting = Some(sc);
        r.map_err(|e| e.to_string())
    }
}

/// The run as the code's engine, while code runs.
struct View<'a> {
    sim: &'a mut FlowSimulation,
    map: &'a ScriptMap,
    /// Pallets that left the plant at this instant: still readable, carried by nothing.
    departed: &'a BTreeMap<u64, Pallet>,
    dirty: bool,
}

impl View<'_> {
    fn object(&self, id: &str) -> Value {
        self.map.index.get(id).map_or(Value::None, |o| Value::Object(*o))
    }

    fn pallets_on(&self, id: &str) -> Vec<u64> {
        let mut out: Vec<u64> = self.sim.lines.iter().filter(|l| l.carrier == id).flat_map(|l| l.pallets.iter().map(|p| p.id)).collect();
        for g in self.sim.gravity.iter().filter(|g| g.id == id) {
            out.extend(g.pallets.iter().map(|p| p.pallet.id));
        }
        out
    }
}

fn unknown(what: &str, member: &str) -> Fault {
    Fault::new("unknownAttribute", format!("{what} has no member `{member}`"))
}

impl Host for View<'_> {
    fn now(&self) -> i64 {
        self.sim.sim.now()
    }

    fn mode(&self) -> &'static str {
        "2D"
    }

    fn seed(&self) -> u64 {
        self.map.seed
    }

    fn get(&mut self, object: u32, member: &str) -> Result<Value, Fault> {
        let o = object as usize;
        let id = self.map.ids.get(o).cloned().unwrap_or_default();
        if let Some((key, _)) = self.map.members.get(&(object, member.to_string())) {
            return Ok(Value::Bool(self.sim.signals.get(key).copied().unwrap_or(false)));
        }
        match member {
            "name" => return Ok(Value::str(self.map.names.get(o).map_or("", String::as_str))),
            "id" => return Ok(Value::str(&id)),
            "kind" => return Ok(Value::str(self.map.kinds.get(o).map_or("", String::as_str))),
            "count" => return Ok(Value::Int(self.pallets_on(&id).len() as i64)),
            "occupied" => return Ok(Value::Bool(!self.pallets_on(&id).is_empty())),
            "pallets" => return Ok(Value::list(self.pallets_on(&id).into_iter().map(Value::Pallet).collect())),
            "next" => {
                let next = self.sim.nodes.iter().find(|n| n.base.id == id).and_then(|n| n.base.downstream.clone());
                return Ok(next.map_or(Value::None, |d| self.object(&d)));
            }
            "exits" => {
                let exits: Vec<Value> = self
                    .sim
                    .nodes
                    .iter()
                    .find(|n| n.base.id == id)
                    .map(|n| match &n.kind {
                        Kind::Turntable(t) => t.exits.iter().map(|e| self.object(&e.target)).collect(),
                        _ => Vec::new(),
                    })
                    .unwrap_or_default();
                return Ok(Value::list(exits));
            }
            "stats" => {
                let stats = self.sim.statistics();
                let mut d = Dict::default();
                for (k, v) in stats.get(&id).into_iter().flatten() {
                    d.set(Value::str(&snake_case(k)), Value::Float(*v)).map_err(Fault::types)?;
                }
                return Ok(Value::dict(d));
            }
            _ => {}
        }
        self.map.params.get(o).and_then(|p| p.get(member)).cloned().ok_or_else(|| unknown(&id, member))
    }

    fn set(&mut self, object: u32, member: &str, value: &Value) -> Result<(), Fault> {
        match self.map.members.get(&(object, member.to_string())) {
            Some((key, true)) => {
                let Value::Bool(b) = value else {
                    return Err(Fault::types(format!("`{member}` is a signal: True or False, not a {}", value.type_name())));
                };
                if self.sim.signals.get(key) != Some(b) {
                    self.sim.signals.insert(key.clone(), *b);
                    self.dirty = true;
                }
                Ok(())
            }
            Some((_, false)) => Err(Fault::new("notWritable", format!("`{member}` is an output: the object sets it"))),
            None => Err(Fault::new("notWritable", format!("`{member}` can't be changed while the model runs"))),
        }
    }

    fn call(&mut self, object: u32, method: &str, _args: &[Value]) -> Result<Value, Fault> {
        Err(unknown(self.map.ids.get(object as usize).map_or("?", String::as_str), method))
    }

    fn pallet(&mut self, pallet: u64, member: &str) -> Result<Option<Value>, Fault> {
        if member == "id" {
            return Ok(Some(Value::Int(i64::try_from(pallet).unwrap_or(i64::MAX))));
        }
        let known = intracraft_script::host::PALLET_MEMBERS.iter().any(|(m, _)| *m == member);
        if !known {
            return Ok(None);
        }
        let found = self.sim.find_pallet(pallet).or_else(|| self.departed.get(&pallet).map(|p| (p.clone(), String::new(), 0.0)));
        let Some((p, carrier, speed)) = found else {
            return if member == "location" {
                Ok(Some(Value::None))
            } else {
                Err(Fault::new("noneReference", format!("pallet {pallet} has left the model")))
            };
        };
        let seconds = |t: SimTime| Value::Qty(t as f64 / US_PER_S as f64, Dim::TIME);
        Ok(Some(match member {
            "name" => Value::str(&format!("MU{pallet}")),
            "address" => {
                // The load unit sits under its carrier: `Seccion1.Conveyor1.MU3`; off the plant, just `MU3`.
                let carrier_address = self.map.index.get(&carrier).and_then(|o| self.map.addresses.get(*o as usize)).cloned();
                match carrier_address {
                    Some(a) if !a.is_empty() => Value::str(&format!("{a}.MU{pallet}")),
                    _ => Value::str(&format!("MU{pallet}")),
                }
            }
            "type" => Value::str(&p.type_id),
            "length" => Value::Qty(p.length, Dim::LENGTH),
            "width" => Value::Qty(p.width, Dim::LENGTH),
            "height" => Value::Qty(p.height, Dim::LENGTH),
            "load_mass" => Value::Qty(p.load_mass, Dim::MASS),
            "created_at" => seconds(p.created_at),
            "age" => seconds(self.sim.sim.now() - p.created_at),
            "location" => self.object(&carrier),
            "speed" => Value::Qty(speed, Dim::SPEED),
            _ => return Ok(None),
        }))
    }

    fn stop(&mut self, reason: &str) {
        self.sim.stopped.get_or_insert_with(|| if reason.is_empty() { "simulation.stop()".into() } else { reason.to_string() });
    }
}

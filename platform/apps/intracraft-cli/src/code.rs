//! `intracraft-cli code check|run|names LAYOUT [CODE.py] [--seconds N]`: IntraScript from a terminal, for people,
//! scripts and agents. `check` lists the problems of the code against the layout (the layout's own code blocks, or
//! `CODE.py` added to it as one more block called `Code`); `run` runs the layout in the 2D engine for `N` simulated
//! seconds (3600 by default) and prints what the code printed, then each code block's statistics; `names` lists every
//! name code can use with the layout: its objects with their addresses and members, the load units' members, the
//! events, the built-ins and the units. `code names` without a layout lists one object of every kind of the library.
//! Exits with 1 on an error.

use std::process::ExitCode;

use intracraft_des::flow::{describe, simulate};
use intracraft_des::time::from_seconds;
use intracraft_library::Library;
use intracraft_library::definition::Value;
use intracraft_library::layout::{Instance, Layout, Project, Vec3};
use intracraft_script::vm::clock;
use intracraft_script::{CODE_BLOCK, Source, compile_blocks};

/// Longest file read, bytes.
const MAX_FILE: u64 = 20_000_000;

pub fn main(args: &[String]) -> ExitCode {
    match run(args) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<ExitCode, String> {
    let seconds = match args.iter().position(|a| a == "--seconds") {
        Some(k) => args
            .get(k + 1)
            .and_then(|s| s.parse::<f64>().ok())
            .filter(|s| s.is_finite() && *s > 0.0)
            .ok_or("--seconds takes a positive number")?,
        None => 3600.0,
    };
    let plain: Vec<&String> =
        args.iter().enumerate().filter(|(k, a)| !a.starts_with("--") && (*k == 0 || args[k - 1] != "--seconds")).map(|(_, a)| a).collect();
    let library = Library::built_in().map_err(|e| format!("the library doesn't read: {e}"))?;
    if plain.first().map(|v| v.as_str()) == Some("names") && plain.get(1).is_none() {
        print!("{}", names(&every_kind(&library), &library));
        return Ok(ExitCode::SUCCESS);
    }
    let (Some(verb), Some(layout_path)) = (plain.first(), plain.get(1)) else { return Err(crate::USAGE.into()) };
    let project = Project::read(&read(layout_path)?)?;
    let mut layout = project.layout;
    if let Some(code_path) = plain.get(2) {
        add_block(&mut layout, &read(code_path)?);
    }
    match verb.as_str() {
        "check" => Ok(check(&layout, &library)),
        "run" => Ok(run_layout(&layout, &library, project.seed, seconds)),
        "names" => {
            print!("{}", names(&layout, &library));
            Ok(ExitCode::SUCCESS)
        }
        other => Err(format!("unknown `code {other}`\n{}", crate::USAGE)),
    }
}

/// A layout with one object of every kind of the library, each named as the library names its kind.
fn every_kind(library: &Library) -> Layout {
    let instances = library
        .list()
        .iter()
        .enumerate()
        .map(|(k, d)| Instance {
            id: format!("o{k}"),
            definition_id: d.id.clone(),
            definition_version: d.version.clone(),
            name: d.name.en.clone(),
            position: Vec3::new(10.0 * k as f64, 0.0, 0.0),
            yaw: 0.0,
            params: Default::default(),
            attributes: Default::default(),
        })
        .collect();
    Layout { instances, ..Layout::default() }
}

/// Every name code can use with `layout`, as text: objects (address, kind, members), load units, events, built-ins
/// and units.
pub fn names(layout: &Layout, library: &Library) -> String {
    use std::fmt::Write as _;
    let model = describe(layout, library).model;
    let mut out = String::new();
    let _ = writeln!(out, "# Objects");
    for o in &model.objects {
        let address = if o.address.is_empty() { o.name.as_str() } else { o.address.as_str() };
        let _ = writeln!(
            out,
            "
{address}  ({})",
            o.kind
        );
        let mut events = Vec::new();
        if o.creates_pallets {
            events.push("@on_create");
        }
        if o.carries_pallets {
            events.extend(["@on_enter", "@on_leave"]);
        }
        if o.decides_exit {
            events.push("@choose_exit");
        }
        if !events.is_empty() {
            let _ = writeln!(out, "  events: {}", events.join(", "));
        }
        for m in &o.members {
            let tag = match m.kind {
                intracraft_script::MemberKind::Signal { writable: true } => "signal, read and write",
                intracraft_script::MemberKind::Signal { writable: false } => "signal, read",
                intracraft_script::MemberKind::Value => "read",
                intracraft_script::MemberKind::Method => "call",
            };
            let _ = writeln!(out, "  .{:<24} {:<24} {}", m.name, tag, m.help);
        }
    }
    let _ = writeln!(
        out,
        "
# Load units (pallet.…)"
    );
    for (n, h) in intracraft_script::host::PALLET_MEMBERS {
        let _ = writeln!(out, "  .{n:<24} {h}");
    }
    let _ = writeln!(
        out,
        "
# Events"
    );
    for (n, h) in intracraft_script::compile::DECORATORS {
        let _ = writeln!(out, "  @{n:<23} {h}");
    }
    let _ = writeln!(
        out,
        "
# Built-ins"
    );
    for (n, _, h) in intracraft_script::builtins::GLOBALS {
        let _ = writeln!(out, "  {n:<25} {h}");
    }
    for m in [
        intracraft_script::builtins::Builtin::Math,
        intracraft_script::builtins::Builtin::Random,
        intracraft_script::builtins::Builtin::Simulation,
    ] {
        let module = intracraft_script::builtins::GLOBALS.iter().find(|(_, b, _)| *b == m).map_or("", |(n, ..)| *n);
        let _ = writeln!(out, "  {module}: {}", intracraft_script::builtins::module_names(m).join(", "));
    }
    let _ = writeln!(
        out,
        "
# Units"
    );
    for (n, size, dim) in intracraft_script::units::UNITS {
        let _ = writeln!(out, "  {n:<10} {size} SI ({})", dim.name());
    }
    out
}

fn read(path: &str) -> Result<String, String> {
    let size = std::fs::metadata(path).map_err(|e| format!("{path}: {e}"))?.len();
    if size > MAX_FILE {
        return Err(format!("{path}: larger than {MAX_FILE} bytes"));
    }
    std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))
}

/// The code as one more block of the layout, called `Code`.
fn add_block(layout: &mut Layout, code: &str) {
    layout.instances.push(Instance {
        id: "code_cli".into(),
        definition_id: CODE_BLOCK.into(),
        definition_version: "0.1.0".into(),
        name: "Code".into(),
        position: Vec3::new(0.0, 0.0, 0.0),
        yaw: 0.0,
        params: [("source".to_string(), Value::Text(code.into()))].into_iter().collect(),
        attributes: Default::default(),
    });
}

fn check(layout: &Layout, library: &Library) -> ExitCode {
    let blocks = describe(layout, library);
    if blocks.sources.is_empty() {
        println!("The layout has no enabled code block.");
        return ExitCode::SUCCESS;
    }
    let sources: Vec<Source> = blocks.sources.iter().map(|(o, code)| Source { object: *o, code }).collect();
    let mut errors = 0;
    for ((object, _), compiled) in blocks.sources.iter().zip(compile_blocks(&blocks.model, &sources)) {
        let name = blocks.model.object(*object).map_or("?", |o| o.name.as_str());
        if compiled.diagnostics.is_empty() {
            println!("{name}: no problems");
        }
        for d in &compiled.diagnostics {
            errors += usize::from(d.is_error());
            println!("{name}, {d}");
        }
    }
    if errors > 0 { ExitCode::FAILURE } else { ExitCode::SUCCESS }
}

fn run_layout(layout: &Layout, library: &Library, seed: u64, seconds: f64) -> ExitCode {
    let mut sim = match simulate(layout, library, seed) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    let result = sim.run_until(from_seconds(seconds)).and_then(|()| sim.finish_scripts());
    for l in sim.script_console() {
        let tag = if l.warning { " warning" } else { "" };
        println!("{}  {}{tag}: {}", clock(l.time), l.block, l.text);
    }
    if let Some(reason) = sim.stopped() {
        println!("Stopped by the code at {}: {reason}", clock(sim.now()));
    }
    let stats = sim.statistics();
    for i in layout.instances.iter().filter(|i| i.definition_id == CODE_BLOCK) {
        if let Some(s) = stats.get(&i.id) {
            let get = |k: &str| s.get(k).copied().unwrap_or(0.0);
            println!("{}: {} activations, {} steps, {} waiting", i.name, get("activations"), get("steps"), get("waiting"));
        }
    }
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

//! `intracraft-cli warehouse … --out FILE`: a high-bay warehouse laid out from its size (A22) and written as a layout
//! file: its storage aisles (S26) and their pick-up and deposit chain conveyors. The same plan the Design module's
//! wizard places.
//!
//! `intracraft-cli run LAYOUT [--seconds N]`: a layout run in the 2D engine for `N` simulated seconds (3600 by default);
//! prints its warnings and every object's statistics. Exits with 1 when the layout doesn't run or has warnings.

use std::process::ExitCode;

use intracraft_des::flow::simulate;
use intracraft_des::flow::warehouse::{Station, StationUse, WarehousePlan, WarehouseSpec, plan};
use intracraft_des::time::from_seconds;
use intracraft_library::Library;
use intracraft_library::definition::Value;
use intracraft_library::layout::{Instance, Layout, Project, Vec3};

/// Longest file read, bytes.
const MAX_FILE: u64 = 20_000_000;

pub const USAGE: &str =
    "       intracraft-cli warehouse [--length M] [--height M] [--aisles N] [--deep 1|2] [--load-height M] [--level-pitch M]
                                [--station HEIGHT:in|out|both]… [--conveyor M] [--set KEY=VALUE]… [--ends] [--name TEXT]
                                [--seed N] --out FILE.intracraft.json
       intracraft-cli run LAYOUT.intracraft.json [--seconds N]";

pub fn main(args: &[String]) -> ExitCode {
    report(warehouse(args))
}

pub fn run_main(args: &[String]) -> ExitCode {
    report(run(args))
}

fn report(r: Result<ExitCode, String>) -> ExitCode {
    r.unwrap_or_else(|e| {
        eprintln!("{e}");
        ExitCode::FAILURE
    })
}

fn warehouse(args: &[String]) -> Result<ExitCode, String> {
    let library = Library::built_in().map_err(|e| format!("the library doesn't read: {e}"))?;
    let mut spec = WarehouseSpec::default();
    let (mut stations, mut out, mut ends, mut name, mut seed) = (Vec::new(), None, false, None, 1);
    let mut k = 0;
    while k < args.len() {
        let flag = args[k].as_str();
        if flag == "--ends" {
            ends = true;
            k += 1;
            continue;
        }
        let value = args.get(k + 1).ok_or_else(|| format!("{flag} takes a value"))?;
        let number = || value.parse::<f64>().ok().filter(|x| x.is_finite()).ok_or_else(|| format!("{flag} takes a number, not {value:?}"));
        let count = || value.parse::<usize>().map_err(|_| format!("{flag} takes a whole number, not {value:?}"));
        match flag {
            "--length" => spec.length = number()?,
            "--height" => spec.height = number()?,
            "--aisles" => spec.aisles = count()?,
            "--deep" => spec.deep = count()?,
            "--load-height" => spec.load_height = number()?,
            "--level-pitch" => spec.level_pitch = Some(number()?),
            "--conveyor" => spec.conveyor_length = number()?,
            "--station" => stations.push(station(value)?),
            "--set" => {
                let (key, text) = value.split_once('=').ok_or("--set takes KEY=VALUE")?;
                let v = match text.parse::<f64>() {
                    Ok(x) => Value::Number(x),
                    Err(_) if text == "true" || text == "false" => Value::Bool(text == "true"),
                    Err(_) => Value::Text(text.to_string()),
                };
                spec.aisle_values.insert(key.to_string(), v);
            }
            "--name" => name = Some(value.clone()),
            "--seed" => seed = value.parse::<u64>().map_err(|_| "--seed takes a whole number")?,
            "--out" => out = Some(value.clone()),
            other => return Err(format!("unknown option {other}")),
        }
        k += 2;
    }
    if !stations.is_empty() {
        spec.stations = stations;
    }
    let out = out.ok_or("say where to write the layout: --out FILE.intracraft.json")?;
    let plan = plan(&spec, &library)?;
    println!("{}", summary(&spec, &plan));
    let mut instances = plan.instances.clone();
    if ends {
        instances.extend(sources_and_sinks(&plan, &spec));
    }
    let project = Project {
        schema_version: 1,
        name: name.unwrap_or_else(|| format!("High-bay warehouse, {} aisles, {} positions", spec.aisles, plan.positions)),
        seed,
        layout: Layout { instances, ..Layout::default() },
    };
    std::fs::write(&out, project.write()?).map_err(|e| format!("{out}: {e}"))?;
    println!("written to {out}");
    Ok(ExitCode::SUCCESS)
}

/// `0.6:in`, `4.6:out`, `0.6:both` (or the height alone).
fn station(text: &str) -> Result<Station, String> {
    let (height, usage) = text.split_once(':').unwrap_or((text, "both"));
    let usage = match usage {
        "in" => StationUse::In,
        "out" => StationUse::Out,
        "both" => StationUse::InAndOut,
        other => return Err(format!("a station is in, out or both, not {other:?}")),
    };
    let height = height.parse::<f64>().ok().filter(|h| h.is_finite()).ok_or_else(|| format!("--station takes HEIGHT:USE, not {text:?}"))?;
    Ok(Station { height, usage })
}

/// What the wizard shows (A22 §2).
pub fn summary(spec: &WarehouseSpec, plan: &WarehousePlan) -> String {
    let (single, combined) = plan.pallets_per_hour();
    format!(
        "{} aisles, {} pallets deep: {} bays x {} levels, {} storage positions\n\
         racks {:.1} m long, {:.1} m high, levels every {:.0} mm; the warehouse is {:.1} m wide\n\
         per aisle (estimate): single cycle {:.1} s = {:.0} pallets/h in; combined cycle {:.1} s = {:.0} pallets/h in and out",
        spec.aisles,
        spec.deep,
        plan.bays,
        plan.levels,
        plan.positions,
        plan.size[0],
        plan.size[2],
        plan.level_pitch * 1000.0,
        plan.size[1],
        plan.single_cycle,
        single,
        plan.combined_cycle,
        combined
    )
}

/// A source in front of every infeed conveyor and a sink after every outfeed: the plan runs as it is.
fn sources_and_sinks(plan: &WarehousePlan, spec: &WarehouseSpec) -> Vec<Instance> {
    let (sin, cos) = spec.yaw.sin_cos();
    let mut out = Vec::new();
    for i in &plan.instances {
        let infeed = i.id.contains("_in");
        if !infeed && !i.id.contains("_out") {
            continue;
        }
        // Along the conveyor's own travel: a source 0.42 m before its start, a sink 0.02 m after its end.
        let along = if infeed { -0.42 } else { spec.conveyor_length + 0.02 };
        let (dx, dz) = if infeed { (along * cos, -along * sin) } else { (-along * cos, along * sin) };
        let mut params = std::collections::BTreeMap::new();
        if infeed {
            params.insert("orientation".to_string(), Value::Text("short".into()));
            params.insert("interval".to_string(), Value::Number(60.0));
        }
        let height = i.params.get("surfaceHeight").and_then(Value::number);
        if let Some(h) = height {
            params.insert("surfaceHeight".to_string(), Value::Number(h));
        }
        out.push(Instance {
            id: format!("{}_{}", i.id, if infeed { "source" } else { "sink" }),
            definition_id: if infeed { "flow.source" } else { "flow.sink" }.into(),
            definition_version: "0.1.0".into(),
            name: format!("{} {}", i.name, if infeed { "source" } else { "sink" }),
            position: Vec3::new(i.position.x + dx, i.position.y, i.position.z + dz),
            yaw: i.yaw,
            params,
            attributes: i.attributes.clone(),
        });
    }
    out
}

fn run(args: &[String]) -> Result<ExitCode, String> {
    let path = args.first().ok_or("say which layout to run")?;
    let seconds = match args.iter().position(|a| a == "--seconds") {
        Some(k) => args
            .get(k + 1)
            .and_then(|s| s.parse::<f64>().ok())
            .filter(|s| s.is_finite() && *s > 0.0)
            .ok_or("--seconds takes a positive number")?,
        None => 3600.0,
    };
    let size = std::fs::metadata(path).map_err(|e| format!("{path}: {e}"))?.len();
    if size > MAX_FILE {
        return Err(format!("{path} is too large"));
    }
    let project = Project::read(&std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?)?;
    let library = Library::built_in().map_err(|e| format!("the library doesn't read: {e}"))?;
    let mut sim = simulate(&project.layout, &library, project.seed)?;
    sim.run_until(from_seconds(seconds))?;
    println!("{}: {} objects, {seconds} s", project.name, project.layout.instances.len());
    for issue in &sim.issues {
        println!("WARNING {} [{}]", issue.message, issue.rule);
    }
    let status = sim.snapshot().status;
    let stats = sim.statistics();
    for i in &project.layout.instances {
        let Some(s) = stats.get(&i.id) else { continue };
        let numbers: Vec<String> =
            s.iter().filter(|(_, v)| **v != 0.0).map(|(k, v)| format!("{k} {}", (v * 1000.0).round() / 1000.0)).collect();
        let state = status.get(&i.id).copied().unwrap_or("");
        println!("{:<28} {:<10} {}", if i.name.is_empty() { &i.id } else { &i.name }, state, numbers.join(", "));
    }
    Ok(if sim.issues.is_empty() { ExitCode::SUCCESS } else { ExitCode::FAILURE })
}

//! `intracraft-cli scenario <id|all>`: runs the reference scenarios on the Rust engines and prints PASS or FAIL for each,
//! as `pnpm run scenario` does for the TypeScript reference. Exits with 1 when one fails.
//!
//! `intracraft-cli code check|run LAYOUT [CODE.py] [--seconds N]`: IntraScript code (A17) checked against a layout, or run
//! with it in the 2D engine (`code`).
//!
//! `intracraft-cli warehouse … --out FILE`: a high-bay warehouse laid out from its size (A22). `intracraft-cli run LAYOUT`:
//! a layout run in the 2D engine, with its warnings and statistics (`warehouse`).

mod code;
mod warehouse;

use std::process::ExitCode;

use intracraft_scenarios::{Outcome, Scenario, find, run, scenarios};

const USAGE: &str = "USAGE: intracraft-cli scenario <id|all>
       intracraft-cli code check|run|names LAYOUT.intracraft.json [CODE.py] [--seconds N]
       intracraft-cli code names";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("code") => return code::main(&args[1..]),
        Some("warehouse") => return warehouse::main(&args[1..]),
        Some("run") => return warehouse::run_main(&args[1..]),
        _ => {}
    }
    let (Some("scenario"), Some(which)) = (args.first().map(String::as_str), args.get(1)) else {
        eprintln!(
            "{USAGE}
{}",
            warehouse::USAGE
        );
        return ExitCode::from(2);
    };
    let library = match intracraft_library::Library::built_in() {
        Ok(l) => l,
        Err(e) => {
            eprintln!("the library doesn't read: {e}");
            return ExitCode::FAILURE;
        }
    };
    let chosen: Vec<Scenario> =
        match if which.eq_ignore_ascii_case("all") { scenarios() } else { find(which).map(|s| s.into_iter().collect()) } {
            Ok(list) if !list.is_empty() => list,
            Ok(_) => {
                eprintln!("no scenario {which}");
                return ExitCode::from(2);
            }
            Err(e) => {
                eprintln!("{e}");
                return ExitCode::FAILURE;
            }
        };
    let mut failed = 0;
    for s in &chosen {
        match run(s, &library) {
            Outcome::Ran { actual, passed } => {
                println!(
                    "{} {:<12} {actual:.4} {} (expected {} {}) {}",
                    if passed { "PASS" } else { "FAIL" },
                    s.id,
                    s.unit,
                    s.expected,
                    s.unit,
                    s.title.en
                );
                if !passed {
                    failed += 1;
                }
            }
            Outcome::NotRun(why) => println!("---- {:<12} not run: {why}", s.id),
            Outcome::Error(e) => {
                println!("FAIL {:<12} {e}", s.id);
                failed += 1;
            }
        }
    }
    if failed > 0 { ExitCode::FAILURE } else { ExitCode::SUCCESS }
}

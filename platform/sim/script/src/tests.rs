//! The language driven as an engine drives it: a small fake plant answers the program's reads and writes.

use std::collections::BTreeMap;

use crate::host::{Host, Member, MemberKind, ModelInfo, ObjectInfo};
use crate::ops::Fault;
use crate::units::Dim;
use crate::value::Value;
use crate::vm::{Event, Runtime};
use crate::{CODE_BLOCK, Source};

const SOURCE: u32 = 0;
const ROLLER3: u32 = 1;
const PHOTOEYE3: u32 = 2;
const TURNTABLE1: u32 = 3;
const ROLLER5: u32 = 4;
const ROLLER7: u32 = 5;
const MAIN: u32 = 6;
const ROUTING: u32 = 7;

#[derive(Default)]
struct Plant {
    now: i64,
    signals: BTreeMap<(u32, String), bool>,
    stopped: Option<String>,
    writes: Vec<(i64, u32, String, bool)>,
}

impl Host for Plant {
    fn now(&self) -> i64 {
        self.now
    }

    fn mode(&self) -> &'static str {
        "2D"
    }

    fn seed(&self) -> u64 {
        7
    }

    fn get(&mut self, object: u32, member: &str) -> Result<Value, Fault> {
        match member {
            "length" => Ok(Value::Qty(3.0, Dim::LENGTH)),
            "count" => Ok(Value::Int(0)),
            _ => Ok(Value::Bool(self.signals.get(&(object, member.to_string())).copied().unwrap_or(false))),
        }
    }

    fn set(&mut self, object: u32, member: &str, value: &Value) -> Result<(), Fault> {
        let Value::Bool(b) = value else { return Err(Fault::types("a signal is True or False")) };
        self.signals.insert((object, member.to_string()), *b);
        self.writes.push((self.now, object, member.to_string(), *b));
        Ok(())
    }

    fn call(&mut self, _object: u32, method: &str, _args: &[Value]) -> Result<Value, Fault> {
        Err(Fault::new("unknownAttribute", format!("no method {method}")))
    }

    fn pallet(&mut self, pallet: u64, member: &str) -> Result<Option<Value>, Fault> {
        Ok(match member {
            "id" => Some(Value::Int(pallet as i64)),
            "type" => Some(Value::str("EUR")),
            _ => None,
        })
    }

    fn stop(&mut self, reason: &str) {
        self.stopped = Some(reason.into());
    }
}

fn model() -> ModelInfo {
    let signal = |n: &str, w: bool| Member { name: n.into(), kind: MemberKind::Signal { writable: w }, help: String::new() };
    let value = |n: &str| Member { name: n.into(), kind: MemberKind::Value, help: String::new() };
    let conveyor = |name: &str| ObjectInfo {
        name: name.into(),
        id: name.to_lowercase(),
        kind: "conveyor.roller".into(),
        members: vec![signal("run", true), signal("occupied", false), value("length"), value("count")],
        carries_pallets: true,
        ..Default::default()
    };
    let code = |name: &str| ObjectInfo { name: name.into(), id: name.to_lowercase(), kind: CODE_BLOCK.into(), ..Default::default() };
    ModelInfo {
        objects: vec![
            ObjectInfo {
                name: "Source1".into(),
                id: "source1".into(),
                kind: "flow.source".into(),
                members: vec![signal("enable", true)],
                creates_pallets: true,
                ..Default::default()
            },
            conveyor("Roller3"),
            ObjectInfo {
                name: "Photoeye3".into(),
                id: "pe3".into(),
                kind: "sensor.photoeye".into(),
                members: vec![signal("detected", false)],
                ..Default::default()
            },
            ObjectInfo { decides_exit: true, ..conveyor("Turntable1") },
            conveyor("Roller5"),
            conveyor("Roller7"),
            code("Main"),
            code("Routing"),
        ],
    }
}

fn runtime(main: &str) -> (Runtime, Plant) {
    runtime_with(&[Source { object: MAIN, code: main }])
}

fn runtime_with(sources: &[Source]) -> (Runtime, Plant) {
    let mut plant = Plant::default();
    let mut rt = match Runtime::build(model(), sources, 7) {
        Ok(rt) => rt,
        Err(e) => panic!("{e:?}"),
    };
    rt.start(&mut plant).unwrap();
    (rt, plant)
}

/// Moves the plant's clock to `t` µs, firing the program's timers on the way, as an engine's event queue does.
fn advance(rt: &mut Runtime, plant: &mut Plant, t: i64) {
    while let Some(w) = rt.next_wake().filter(|w| *w <= t) {
        plant.now = w;
        rt.react(&[], plant).unwrap();
    }
    plant.now = t;
}

fn printed(rt: &Runtime) -> Vec<String> {
    rt.console().iter().map(|l| l.text.clone()).collect()
}

fn signal(object: u32, value: bool) -> Event {
    Event::Signal { object, signal: "detected".into(), value }
}

#[test]
fn a_photocell_stops_the_conveyor_for_exactly_two_seconds() {
    let (mut rt, mut plant) =
        runtime("@on_rise(Photoeye3.detected)\ndef hold():\n    Roller3.run = False\n    wait(2 * s)\n    Roller3.run = True\n");
    plant.now = 5_000_000;
    rt.react(&[signal(PHOTOEYE3, true)], &mut plant).unwrap();
    assert_eq!(plant.writes, [(5_000_000, ROLLER3, "run".into(), false)]);
    assert_eq!(rt.next_wake(), Some(7_000_000));
    advance(&mut rt, &mut plant, 10_000_000);
    assert_eq!(plant.writes[1], (7_000_000, ROLLER3, "run".into(), true));
    // A falling edge is not a rise.
    rt.react(&[signal(PHOTOEYE3, false)], &mut plant).unwrap();
    assert_eq!(plant.writes.len(), 2);
}

#[test]
fn block_variables_count_and_timers_repeat() {
    let (mut rt, mut plant) = runtime(
        "counted = 0\n\n@on_rise(Photoeye3.detected)\ndef count():\n    global counted\n    counted += 1\n\n@every(1 * minute)\ndef report():\n    print(f\"{now()}: {counted} pallets\")\n",
    );
    for k in 1..=3 {
        plant.now = k * 10_000_000;
        rt.react(&[signal(PHOTOEYE3, true)], &mut plant).unwrap();
    }
    advance(&mut rt, &mut plant, 125_000_000);
    assert_eq!(printed(&rt), ["60 s: 3 pallets", "120 s: 3 pallets"]);
    assert_eq!(rt.variables(MAIN), [("counted".to_string(), "3".to_string())]);
}

#[test]
fn wait_until_resumes_on_the_condition_or_gives_up_at_its_timeout() {
    let (mut rt, mut plant) = runtime(
        "@on_start\ndef watch():\n    if wait_until(Roller3.occupied, timeout=30 * s):\n        print('occupied at', now())\n    if not wait_until(Roller5.occupied, timeout=10 * s):\n        warn('Roller5 stayed empty')\n",
    );
    plant.now = 12_000_000;
    plant.signals.insert((ROLLER3, "occupied".into()), true);
    rt.react(&[], &mut plant).unwrap();
    assert_eq!(printed(&rt), ["occupied at 12 s"]);
    advance(&mut rt, &mut plant, 60_000_000);
    assert_eq!(printed(&rt)[1], "Roller5 stayed empty");
    assert!(rt.console()[1].warning);
    assert_eq!(rt.console()[1].time, 22_000_000);
}

#[test]
fn an_exit_decision_reads_the_attribute_given_to_the_pallet() {
    let (mut rt, mut plant) = runtime(
        "@on_create(Source1)\ndef tag(pallet):\n    pallet.destination = 'B' if pallet.id % 2 == 0 else 'A'\n\n@choose_exit(Turntable1)\ndef route(pallet):\n    return Roller5 if pallet.destination == 'B' else Roller7\n",
    );
    assert!(rt.decides_exit(TURNTABLE1));
    for id in 1..=2 {
        rt.react(&[Event::Create { source: SOURCE, pallet: id }], &mut plant).unwrap();
    }
    assert_eq!(rt.choose_exit(TURNTABLE1, 1, &[ROLLER5, ROLLER7], &mut plant).unwrap(), Some(ROLLER7));
    assert_eq!(rt.choose_exit(TURNTABLE1, 2, &[ROLLER5, ROLLER7], &mut plant).unwrap(), Some(ROLLER5));
    assert_eq!(rt.pallet_attributes(2), [("destination".to_string(), "'B'".to_string())]);
    let e = rt.choose_exit(TURNTABLE1, 1, &[ROLLER5], &mut plant).unwrap_err();
    assert_eq!(e.code, "notAnExit");
}

#[test]
fn a_loop_that_never_waits_stops_the_run_instead_of_freezing_it() {
    let mut plant = Plant::default();
    let mut rt = Runtime::build(
        model(),
        &[Source { object: MAIN, code: "@on_start\ndef spin():\n    n = 0\n    while True:\n        n += 1\n" }],
        1,
    )
    .unwrap();
    let e = rt.start(&mut plant).unwrap_err();
    assert_eq!((e.code.as_str(), e.block.as_str()), ("stepLimit", "Main"));
    assert!(e.line == 4 || e.line == 5, "line {}", e.line);
}

#[test]
fn run_errors_say_block_line_time_and_what_to_do() {
    let (mut rt, mut plant) = runtime("n = 0\n\n@on_rise(Photoeye3.detected)\ndef count():\n    n += 1\n");
    plant.now = 3_500_000;
    let e = rt.react(&[signal(PHOTOEYE3, true)], &mut plant).unwrap_err();
    assert_eq!((e.code.as_str(), e.line), ("unboundLocal", 5));
    assert!(e.to_string().contains("global n"), "{e}");
    assert!(e.to_string().contains("0:00:03.500"), "{e}");
    let e = Runtime::build(model(), &[Source { object: MAIN, code: "@on_start\ndef bad():\n    print(Roller3.length + 2 * s)\n" }], 1)
        .unwrap()
        .start(&mut plant)
        .unwrap_err();
    assert_eq!(e.code, "unitMismatch");
}

#[test]
fn pythons_everyday_features_work() {
    let (rt, _) = runtime(
        r#"
@on_start
def go():
    zones = [Roller3, Roller5, Roller7]
    free = [z for z in zones if not z.occupied]
    print(len(free), 1 < len(free) <= 3)
    d = {"A": 2, "B": 5}
    d["C"] = 1
    for k, v in sorted(d.items(), key=second, reverse=True):
        print(k, v)
    a, b = divmod_(17, 5)
    print(a, b, "pallet"[:3], list(range(10))[::3])
    gap = 150 * mm
    print(f"{gap / (0.25 * m / s):.1f}", gap / mm, round(gap / mm), 2.5 * m / s**2)
    total = sum([1 * m, 500 * mm])
    print(total, f"{0.256:.1%}", str(None), max(3, 9, 1), abs(-2 * s))

def second(pair):
    return pair[1]

def divmod_(x, y):
    return x // y, x % y
"#,
    );
    assert_eq!(
        printed(&rt),
        ["3 True", "B 5", "A 2", "C 1", "3 2 pal [0, 3, 6, 9]", "0.6 s 150.0 150 2.5 m/s²", "1500 mm 25.6% None 9 2 s"]
    );
}

#[test]
fn random_numbers_repeat_for_the_same_seed_and_differ_between_blocks() {
    let code = "@on_start\ndef draw():\n    print(random.uniform(20 * s, 40 * s), random.randint(1, 6), random.choice(['A', 'B', 'C']))\n";
    let (a, _) = runtime(code);
    let (b, _) = runtime(code);
    assert_eq!(printed(&a), printed(&b));
    let (both, _) = runtime_with(&[Source { object: MAIN, code }, Source { object: ROUTING, code }]);
    let lines = printed(&both);
    assert_ne!(lines[0], lines[1]);
}

#[test]
fn blocks_call_each_others_functions_and_share_variables() {
    let routing = "routed = 0\n\ndef pick(kind):\n    global routed\n    routed += 1\n    return Roller5 if kind == 'B' else Roller7\n";
    let main = "@on_start\ndef go():\n    print(Routing.pick('B') == Roller5, Routing.routed)\n";
    let (rt, _) = runtime_with(&[Source { object: ROUTING, code: routing }, Source { object: MAIN, code: main }]);
    assert_eq!(printed(&rt), ["True 1"]);
}

#[test]
fn handlers_of_one_instant_run_by_priority_then_in_order() {
    let (mut rt, mut plant) = runtime(
        "@on_rise(Photoeye3.detected)\ndef second():\n    print('second')\n\n@on_rise(Photoeye3.detected, priority=5)\ndef first():\n    print('first')\n    wait(0 * s)\n    print('first, after the others')\n\n@on_change(Photoeye3.detected)\ndef third():\n    print('third')\n",
    );
    rt.react(&[signal(PHOTOEYE3, true)], &mut plant).unwrap();
    advance(&mut rt, &mut plant, 0);
    assert_eq!(printed(&rt), ["first", "second", "third", "first, after the others"]);
}

#[test]
fn deep_recursion_and_errors_from_code_stop_the_run() {
    let mut plant = Plant::default();
    let deep = "def down(n):\n    return down(n + 1)\n\n@on_start\ndef go():\n    down(0)\n";
    let e = Runtime::build(model(), &[Source { object: MAIN, code: deep }], 1).unwrap().start(&mut plant).unwrap_err();
    assert_eq!(e.code, "depthLimit");
    let stop = "@at(1 * h)\ndef end():\n    error('the buffer overflowed')\n";
    let (mut rt, mut plant) = runtime(stop);
    plant.now = 3_600_000_000;
    let e = rt.react(&[], &mut plant).unwrap_err();
    assert_eq!((e.code.as_str(), e.message.as_str()), ("userError", "the buffer overflowed"));
}

#[test]
fn a_block_with_errors_does_not_build() {
    let errors = match Runtime::build(model(), &[Source { object: MAIN, code: "@on_start\ndef go():\n    Roler3.run = False\n" }], 1) {
        Err(e) => e,
        Ok(_) => panic!("built"),
    };
    assert_eq!(errors[0].0, MAIN);
    assert_eq!(errors[0].1[0].code, "unknownName");
}

#[test]
fn simulation_stop_and_finish_handlers() {
    let (mut rt, mut plant) = runtime(
        "@at(10 * s)\ndef enough():\n    simulation.stop('done')\n\n@on_finish\ndef bye():\n    print('finished in', simulation.mode)\n",
    );
    advance(&mut rt, &mut plant, 20_000_000);
    assert_eq!(plant.stopped.as_deref(), Some("done"));
    rt.finish(&mut plant).unwrap();
    assert_eq!(printed(&rt), ["finished in 2D"]);
}

#[test]
fn names_for_code() {
    assert_eq!(crate::code_name("Roller3"), "Roller3");
    assert_eq!(crate::code_name("Roller conveyor 100023"), "Roller_conveyor_100023");
    assert_eq!(crate::code_name("100023"), "_100023");
    assert_eq!(crate::code_name("class"), "class_");
    assert_eq!(crate::snake_case("nominalSpeed"), "nominal_speed");
    assert!(!crate::is_identifier("for"));
}

/// Every Python example of the language reference and the tutorial is IntraScript the checker accepts: no syntax error, and
/// no error but the names of a layout the example assumes.
#[test]
fn the_documentations_examples_are_valid_intrascript() {
    let docs = [
        ("A17", include_str!("../../../../docs/en/app/A17 IntraScript Language.md")),
        ("A20", include_str!("../../../../docs/en/app/A20 IntraScript Tutorial.md")),
        ("S21", include_str!("../../../../docs/en/specs/S21 Code Block.md")),
    ];
    let layout_names = ["unknownName", "unknownMember", "unknownEvent"];
    let mut checked = 0;
    for (doc, text) in docs {
        // A Windows checkout may end the lines with CR LF.
        let text = text.replace("\r\n", "\n");
        for (k, block) in text.split("```python\n").skip(1).enumerate() {
            let code = block.split("```").next().unwrap_or_default();
            let problems: Vec<String> = crate::check(&ModelInfo::default(), 0, code, &[])
                .into_iter()
                .filter(|d| d.is_error() && !layout_names.contains(&d.code.as_str()))
                .map(|d| d.to_string())
                .collect();
            assert!(problems.is_empty(), "{doc}, example {}: {problems:?}\n{code}", k + 1);
            checked += 1;
        }
    }
    assert!(checked >= 18, "{checked} examples");
}

#[test]
fn print_shows_objects_by_their_names() {
    let (rt, _) = runtime(
        "@on_start
def go():
    print(Roller3, [Roller3, Roller5], Photoeye3.detected)
",
    );
    assert_eq!(printed(&rt), ["Roller3 [Roller3, Roller5] False"]);
}

/// Every program of the HTML guide and manual (`docs/tutorial/*.html`, blocks of class `prog`) is valid IntraScript: no
/// error but the names of a layout that the page doesn't draw.
#[test]
fn the_tutorial_and_manual_programs_are_valid_intrascript() {
    let pages = [
        ("guide", include_str!("../../../../docs/tutorial/intrascript-tutorial.html"), 10),
        ("manual", include_str!("../../../../docs/tutorial/intrascript-manual.html"), 12),
    ];
    for (name, page, at_least) in pages {
        let mut checked = 0;
        for block in page.split("<pre class=\"prog\"><code>").skip(1) {
            let raw = block.split("</code></pre>").next().unwrap_or_default();
            let code = raw.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&amp;", "&");
            let problems: Vec<String> = crate::check(&ModelInfo::default(), 0, &code, &[])
                .into_iter()
                .filter(|d| d.is_error() && !["unknownName", "unknownMember", "unknownEvent"].contains(&d.code.as_str()))
                .map(|d| d.to_string())
                .collect();
            assert!(problems.is_empty(), "{name} program {}: {problems:?}\n{code}", checked + 1);
            checked += 1;
        }
        assert!(checked >= at_least, "{name}: {checked} programs");
    }
}

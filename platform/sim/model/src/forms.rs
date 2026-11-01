//! The parametric 3D forms of the library's families: our own geometry after the reference's 3D models
//! (`apps/editor/src/components/models`, `packages/geometry`), no third-party models. Rollers with their axles between two
//! steel C-channel side frames, chain strands on their tracks with the drive sprockets and the chain loop, legs on
//! adjustable feet at most 2 m apart with a cross beam from 650 mm and a diagonal brace from 850 mm conveying height, the
//! geared motor, the chain guard and the end stop; and what each station has of its own.
//!
//! In the family's axes, mm: x along the travel from the entry (a turntable: from its centre), y to the left, z up from
//! the floor. Formulas read the family's parameters, so each component draws its own sizes, and arrays (`repeat`) give as
//! many rollers, strands and legs as its length, pitch and counts make. A choice (a text parameter) can't be read by a
//! formula: forms that depend on one are drawn for its default (the drive on the right, under the deck; a roller deck).

use intracraft_library::definition::Definition;
use serde_json::{Value as Json, json};

/// The forms' materials and their colours, made in the project with the families.
pub const MATERIALS: [(&str, &str); 14] = [
    (FRAME, "#2f6fb3"),
    (ROLLER, "#9aa3ad"),
    (CHAIN, "#3b3f45"),
    (SUPPORT, "#5b6470"),
    (BASE, "#4a525d"),
    (DECK, "#6d7782"),
    (GUARD, "#d9a62e"),
    (DRIVE, "#3f5f7a"),
    (BRAKE, "#c4473a"),
    (SOURCE, "#3c9d5a"),
    (SINK, "#c2453d"),
    (BEAM, "#3ddc84"),
    (GANTRY, "#c9822b"),
    (SCENERY, "#8c7b6b"),
];
const FRAME: &str = "IntraCraft frame";
const ROLLER: &str = "IntraCraft roller";
const CHAIN: &str = "IntraCraft chain";
const SUPPORT: &str = "IntraCraft support";
const BASE: &str = "IntraCraft base";
const DECK: &str = "IntraCraft deck";
const GUARD: &str = "IntraCraft guard";
const DRIVE: &str = "IntraCraft drive";
const BRAKE: &str = "IntraCraft brake";
const SOURCE: &str = "IntraCraft source";
const SINK: &str = "IntraCraft sink";
const BEAM: &str = "IntraCraft beam";
const GANTRY: &str = "IntraCraft gantry";
const SCENERY: &str = "IntraCraft scenery";

/// The side frames: C-channels this high, their flanges this deep, of this thickness (mm), a little clear of the rollers.
const FRAME_H: f64 = 120.0;
const FRAME_D: f64 = 50.0;
const FRAME_T: f64 = 5.0;
const FRAME_CLEAR: f64 = 5.0;
/// The side frames' top lies this far under the conveying surface, so a pallet wider than the deck clears them.
const FRAME_DROP: f64 = 10.0;
const CHAIN_H: f64 = 30.0;
const TRACK_H: f64 = 60.0;
/// Posts (square tubes), their adjustable feet and floor plates, and the bracing.
const POST: f64 = 50.0;
const FOOT_H: f64 = 60.0;
const FOOT_PLATE: f64 = 120.0;
const PLATE_T: f64 = 10.0;
const BRACE: f64 = 35.0;
/// The most supports a family draws one by one (a gravity line of 30 m has 16).
const MAX_SUPPORTS: usize = 16;
/// The most transfer chains a family draws one by one.
const MAX_TRANSFER_CHAINS: usize = 6;

/// The forms of a definition's family.
pub fn forms(d: &Definition) -> Json {
    let mut f = Forms::default();
    match d.id.as_str() {
        "conveyor.roller" => roller_conveyor(&mut f),
        "conveyor.chain" => chain_conveyor(&mut f),
        "conveyor.roller_turntable" => turntable(&mut f, false),
        "conveyor.chain_turntable" => turntable(&mut f, true),
        "conveyor.transfer_rc" => transfer(&mut f),
        "conveyor.lift_table_manual" => lift_table(&mut f, "lowHeight"),
        "conveyor.scissor_lift" => lift_table(&mut f, "entryHeight"),
        "conveyor.elevator" => elevator(&mut f),
        "conveyor.gravity" => gravity_line(&mut f),
        "flow.process_station" => process_station(&mut f),
        "flow.source" => flow_end(&mut f, SOURCE, true),
        "flow.sink" => flow_end(&mut f, SINK, false),
        "sensor.photoeye" => photoeye(&mut f),
        _ => {
            f.cuboid(SCENERY, ["-250", "250"], ["-250", "250"], ["0", "500"]);
        }
    }
    Json::Array(f.0)
}

/// The forms of a family as it builds them.
#[derive(Default)]
struct Forms(Vec<Json>);

type Pair<'a> = [&'a str; 2];

impl Forms {
    fn push(&mut self, form: Json) -> Form<'_> {
        self.0.push(form);
        let last = self.0.len() - 1;
        Form(&mut self.0[last])
    }

    /// A box between two corners.
    fn cuboid(&mut self, material: &str, x: Pair, y: Pair, z: Pair) -> Form<'_> {
        let profile = json!([[x[0], y[0]], [x[1], y[0]], [x[1], y[1]], [x[0], y[1]]]);
        self.push(json!({"plane": "xy", "profile": profile, "from": z[0], "to": z[1], "material": material}))
    }

    /// A round bar across the travel (along y): its axis at (x, z).
    fn rod_y(&mut self, material: &str, centre: Pair, radius: &str, y: Pair) -> Form<'_> {
        self.push(json!({"plane": "xz", "circle": {"centre": centre, "radius": radius}, "from": y[0], "to": y[1], "material": material}))
    }

    /// A round bar along the travel (along x): its axis at (y, z).
    fn rod_x(&mut self, material: &str, centre: Pair, radius: &str, x: Pair) -> Form<'_> {
        self.push(json!({"plane": "yz", "circle": {"centre": centre, "radius": radius}, "from": x[0], "to": x[1], "material": material}))
    }

    /// An upright round bar: its axis at (x, y).
    fn rod_z(&mut self, material: &str, centre: Pair, radius: &str, z: Pair) -> Form<'_> {
        self.push(json!({"plane": "xy", "circle": {"centre": centre, "radius": radius}, "from": z[0], "to": z[1], "material": material}))
    }

    /// A section in the y–z plane run along x (a steel profile along the travel).
    fn along_x(&mut self, material: &str, section: &[[String; 2]], x: Pair) -> Form<'_> {
        self.push(json!({"plane": "yz", "profile": section, "from": x[0], "to": x[1], "material": material}))
    }

    /// A section in the x–z plane run along y (a plate across the travel, seen from the side).
    fn along_y(&mut self, material: &str, section: &[[String; 2]], y: Pair) -> Form<'_> {
        self.push(json!({"plane": "xz", "profile": section, "from": y[0], "to": y[1], "material": material}))
    }
}

/// A form just added, to say when it's there and how often.
struct Form<'a>(&'a mut Json);

impl Form<'_> {
    fn visible(self, formula: &str) -> Self {
        self.0["visible"] = json!(formula);
        self
    }

    fn repeat(self, count: &str, step: [&str; 3]) -> Self {
        self.0["repeat"] = json!({"count": count, "step": step});
        self
    }
}

fn p(a: impl std::fmt::Display, b: impl std::fmt::Display) -> [String; 2] {
    [a.to_string(), b.to_string()]
}

// ── Building blocks ──────────────────────────────────────────────────────────────────────────────────────────────────

/// Rollers across the deck between `x0` and `x0 + length`, their tops at `top`, `half` either side of the axis, each on
/// an axle through the side frames: as many as fit at the pitch, centred (`rollerLayout` of the reference).
fn rollers(f: &mut Forms, x0: &str, length: &str, half: &str, top: &str, pitch: &str, diameter: &str) {
    let count = format!("rounddown(({length}) / ({pitch}))");
    let first = format!("({x0}) + (({length}) - ({count} - 1) * ({pitch})) / 2");
    let centre = [first.as_str(), &format!("({top}) - ({diameter}) / 2")];
    let step = [pitch, "0", "0"];
    f.rod_y(ROLLER, centre, &format!("({diameter}) / 2"), [&format!("-({half})"), half]).repeat(&count, step);
    let through = format!("({half}) + {}", FRAME_CLEAR + FRAME_D);
    f.rod_y(ROLLER, centre, "10", [&format!("-({through})"), &through]).repeat(&count, step);
}

/// Chain strands along the travel from `x0` to `x1`, their tops at `top`, spread across `width` as the reference's
/// `chainLayout` spreads them, each on a guide track; `count` and `strand` are formulas.
fn strands(f: &mut Forms, x: Pair, width: &str, count: &str, strand: &str, top: &str) {
    let y0 = format!("-(({width}) - 200) / 2");
    let step = format!("(({width}) - 200) / max(1, ({count}) - 1)");
    let s = [step.as_str()];
    let chain_y = [format!("{y0} - ({strand}) / 2"), format!("{y0} + ({strand}) / 2")];
    let track_y = [format!("{y0} - ({strand}) / 2 - 10"), format!("{y0} + ({strand}) / 2 + 10")];
    f.cuboid(CHAIN, x, [&chain_y[0], &chain_y[1]], [&format!("({top}) - {CHAIN_H}"), top]).repeat(count, ["0", s[0], "0"]);
    let track_z = [format!("({top}) - {}", CHAIN_H + TRACK_H), format!("({top}) - {CHAIN_H}")];
    f.cuboid(FRAME, x, [&track_y[0], &track_y[1]], [&track_z[0], &track_z[1]]).repeat(count, ["0", s[0], "0"]);
}

/// Two C-channel side frames along the travel from `x0` to `x1`, their webs outside, `half` the free width between them,
/// their top at `top`.
fn side_frames(f: &mut Forms, x: Pair, half: &str, top: &str) {
    for side in [1.0, -1.0] {
        let inner = format!("{side} * (({half}) + {FRAME_CLEAR})");
        let outer = format!("{side} * (({half}) + {})", FRAME_CLEAR + FRAME_D);
        let web = format!("{side} * (({half}) + {})", FRAME_CLEAR + FRAME_D - FRAME_T);
        let (zt, zb) = (format!("({top})"), format!("({top}) - {FRAME_H}"));
        let (zt2, zb2) = (format!("{zt} - {FRAME_T}"), format!("{zb} + {FRAME_T}"));
        let c = [
            p(&outer, &zt),
            p(&inner, &zt),
            p(&inner, &zt2),
            p(&web, &zt2),
            p(&web, &zb2),
            p(&inner, &zb2),
            p(&inner, &zb),
            p(&outer, &zb),
        ];
        f.along_x(FRAME, &c, x);
    }
}

/// Pairs of posts under side frames whose bottom is at `bottom`, `half` the free width between the frames, along
/// `length` from `x0`: at most 2 m apart and 100 mm in from the ends (`supportPositions`), on adjustable feet and floor
/// plates; a cross beam from a conveying height `top` of 650 mm, a diagonal brace from 850 mm (`frameBracing`).
fn legs(f: &mut Forms, x0: &str, length: &str, half: &str, bottom: &str, top: &str) {
    let bays = format!("max(1, roundup((({length}) - 200) / 2000))");
    let count = format!("if(({length}) > 200, {bays} + 1, 1)");
    let first = format!("({x0}) + if(({length}) > 200, 100, ({length}) / 2)");
    let step = format!("if(({length}) > 200, (({length}) - 200) / {bays}, 0)");
    let at = [step.as_str(), "0", "0"];
    let yc = format!("(({half}) + {})", FRAME_CLEAR + FRAME_D / 2.0);
    let around = |c: &str, d: f64| [format!("{c} - {}", d / 2.0), format!("{c} + {}", d / 2.0)];
    let xs_post = around(&first, POST);
    let xs_plate = around(&first, FOOT_PLATE);
    let post_top = format!("max(({bottom}), {})", FOOT_H + 10.0);
    for side in ["", "-"] {
        let y = format!("{side}{yc}");
        let ys_post = around(&y, POST);
        let ys_plate = around(&y, FOOT_PLATE);
        f.cuboid(SUPPORT, [&xs_post[0], &xs_post[1]], [&ys_post[0], &ys_post[1]], [&FOOT_H.to_string(), &post_top]).repeat(&count, at);
        f.cuboid(BASE, [&xs_plate[0], &xs_plate[1]], [&ys_plate[0], &ys_plate[1]], ["0", &PLATE_T.to_string()]).repeat(&count, at);
        f.rod_z(ROLLER, [&first, &y], "12", [&PLATE_T.to_string(), &FOOT_H.to_string()]).repeat(&count, at);
    }
    let xs_brace = around(&first, BRACE);
    let low = format!("{FOOT_H} + (({post_top}) - {FOOT_H}) * 0.25");
    let high = format!("{FOOT_H} + (({post_top}) - {FOOT_H}) * 0.9 - {BRACE}");
    f.cuboid(SUPPORT, [&xs_brace[0], &xs_brace[1]], [&format!("-{yc}"), &yc], [&low, &format!("{low} + {BRACE}")])
        .repeat(&count, at)
        .visible(&format!("({top}) >= 650"));
    let diagonal =
        [p(format!("-{yc}"), &low), p(&yc, &high), p(&yc, format!("{high} + {BRACE}")), p(format!("-{yc}"), format!("{low} + {BRACE}"))];
    f.along_x(SUPPORT, &diagonal, [&xs_brace[0], &xs_brace[1]]).repeat(&count, at).visible(&format!("({top}) >= 850"));
}

/// A geared motor: its gearbox centred at (x, y, z) and the motor beside it, along x (under a deck) or along y (outside).
fn drive(f: &mut Forms, at: [&str; 3], along_y: bool) {
    let [x, y, z] = at;
    let around = |c: &str, d: f64| [format!("({c}) - {}", d / 2.0), format!("({c}) + {}", d / 2.0)];
    let (bx, by, bz) = (around(x, 220.0), around(y, 200.0), around(z, 220.0));
    f.cuboid(DRIVE, [&bx[0], &bx[1]], [&by[0], &by[1]], [&bz[0], &bz[1]]);
    if along_y {
        f.rod_y(DRIVE, [x, z], "85", [&format!("({y}) - 390"), &format!("({y}) - 110")]);
    } else {
        f.rod_x(DRIVE, [y, z], "85", [&format!("({x}) - 390"), &format!("({x}) - 110")]);
    }
}

/// A plate across the deck's end at `x`, standing above the conveying surface `top`, when `endStop` is set.
fn end_stop(f: &mut Forms, x: &str, half: &str, top: &str) {
    f.cuboid(GUARD, [x, &format!("({x}) + 20")], [&format!("-({half})"), half], [&format!("({top}) - 30"), &format!("({top}) + 120")])
        .visible("endStop");
}

// ── The families ─────────────────────────────────────────────────────────────────────────────────────────────────────

/// S01: chain-driven rollers between two side frames, the chain guard on the drive side, the geared motor under the deck,
/// an optional end stop, on braced feet.
fn roller_conveyor(f: &mut Forms) {
    let (half, top) = ("usableWidth / 2", "surfaceHeight");
    let frame_top = format!("{top} - {FRAME_DROP}");
    rollers(f, "0", "length", half, top, "rollerPitch", "rollerDiameter");
    side_frames(f, ["0", "length"], half, &frame_top);
    f.cuboid(
        GUARD,
        ["0", "length"],
        [&format!("-({half}) - {}", FRAME_CLEAR + FRAME_D + 35.0), &format!("-({half})")],
        [&frame_top, &format!("{top} + 20")],
    );
    let bottom = format!("{frame_top} - {FRAME_H}");
    drive(f, ["length / 2", &format!("-({half}) + 200"), &format!("max({bottom} - 170, 130)")], false);
    end_stop(f, "length", half, top);
    legs(f, "0", "length", half, &bottom, top);
}

/// S02: chain strands on guide tracks, the drive shaft mid-length with a sprocket under each strand and the chain looping
/// down to it, the geared motor outside on the drive side, an optional end stop, on side frames and braced feet.
fn chain_conveyor(f: &mut Forms) {
    let (half, top) = ("usableWidth / 2", "surfaceHeight");
    strands(f, ["0", "length"], "usableWidth", "chainCount", "chainSupportWidth", top);
    let shaft_z = format!("max({top} - 220, 80)");
    let wide = format!("({half}) + 60");
    f.rod_y(ROLLER, ["length / 2", &shaft_z], "25", [&format!("-({wide})"), &wide]);
    let y0 = "-(usableWidth - 200) / 2";
    let step = "(usableWidth - 200) / max(1, chainCount - 1)";
    let sprocket_y = [format!("{y0} - chainSupportWidth / 2"), format!("{y0} + chainSupportWidth / 2")];
    f.rod_y(CHAIN, ["length / 2", &shaft_z], "60", [&sprocket_y[0], &sprocket_y[1]]).repeat("chainCount", ["0", step, "0"]);
    // The chain loops down from the track to the sprocket and back up, on both sides of it.
    let track_bottom = format!("{top} - {}", CHAIN_H + TRACK_H);
    for dir in ["-", "+"] {
        let (near, far) = (format!("length / 2 {dir} 60"), format!("length / 2 {dir} 180"));
        let loop_ =
            [p(&near, &shaft_z), p(&far, &track_bottom), p(&far, format!("{track_bottom} + 12")), p(&near, format!("{shaft_z} + 12"))];
        let ys = [format!("{y0} - chainSupportWidth * 0.4"), format!("{y0} + chainSupportWidth * 0.4")];
        f.along_y(CHAIN, &loop_, [&ys[0], &ys[1]]).repeat("chainCount", ["0", step, "0"]);
    }
    drive(f, ["length / 2", &format!("-({half}) - {}", FRAME_CLEAR + FRAME_D + 120.0), &shaft_z], true);
    end_stop(f, "length", half, top);
    let frame_top = format!("{top} - {CHAIN_H}");
    side_frames(f, ["0", "length"], half, &frame_top);
    legs(f, "0", "length", half, &format!("{frame_top} - {FRAME_H}"), top);
}

/// S03, S04: a fixed base on a floor frame, the slewing ring and the turning platform with its rollers or chains between
/// two side frames, the turning drive beside the base. Origin at the platform's centre.
fn turntable(f: &mut Forms, chains: bool) {
    let (top, half) = ("surfaceHeight", "platformWidth / 2 - 60");
    let x = ["-platformLength / 2", "platformLength / 2"];
    let frame_top = if chains { format!("{top} - {CHAIN_H}") } else { format!("{top} - {FRAME_DROP}") };
    let plate = [format!("max({frame_top} - {}, 120)", FRAME_H + 40.0), format!("max({frame_top} - {FRAME_H}, 160)")];
    let ring = [format!("max({} - 50, 70)", plate[0]), plate[0].clone()];
    f.cuboid(BASE, ["-550", "550"], ["-550", "550"], ["0", "50"]);
    f.rod_z(SUPPORT, ["0", "0"], "350", ["50", &ring[0]]);
    f.rod_z(ROLLER, ["0", "0"], "450", [&ring[0], &ring[1]]);
    f.cuboid(DECK, x, ["-platformWidth / 2", "platformWidth / 2"], [&plate[0], &plate[1]]);
    if chains {
        strands(f, ["-platformLength / 2", "platformLength / 2"], "platformWidth", "chainCount", "chainSupportWidth", top);
    } else {
        rollers(f, x[0], "platformLength", half, top, "rollerPitch", "rollerDiameter");
    }
    side_frames(f, x, half, &frame_top);
    drive(f, ["470", "0", &format!("max(({}) * 0.55, 150)", ring[0])], true);
}

/// S05: a roller section with liftable transfer chains across it in the gaps between the rollers (drawn lowered), each
/// on its track, the lifting frame and its two actuators under the deck, on braced feet.
fn transfer(f: &mut Forms) {
    let (half, top) = ("usableWidth / 2", "surfaceHeight");
    let frame_top = format!("{top} - {FRAME_DROP}");
    rollers(f, "0", "length", half, top, "rollerPitch", "rollerDiameter");
    side_frames(f, ["0", "length"], half, &frame_top);
    // The chains sit in the middle of gaps between rollers, spread from the first gap to the last
    // (`transferChainPositions`).
    let rollers = "rounddown(length / rollerPitch)";
    let first_gap = format!("(length - ({rollers} - 1) * rollerPitch) / 2 + rollerPitch / 2");
    let chain_top = format!("{top} - lowerClearance");
    for i in 0..MAX_TRANSFER_CHAINS {
        let x = format!("{first_gap} + round({i} * ({rollers} - 2) / max(1, transferChainCount - 1)) * rollerPitch");
        let xs = [format!("{x} - transferChainWidth / 2"), format!("{x} + transferChainWidth / 2")];
        let shown = format!("{i} < transferChainCount");
        f.cuboid(CHAIN, [&xs[0], &xs[1]], [&format!("-({half})"), half], [&format!("{chain_top} - {CHAIN_H}"), &chain_top]).visible(&shown);
        let track = [format!("{x} - transferChainWidth / 2 - 10"), format!("{x} + transferChainWidth / 2 + 10")];
        let track_z = [format!("{chain_top} - {}", CHAIN_H + TRACK_H), format!("{chain_top} - {CHAIN_H}")];
        f.cuboid(FRAME, [&track[0], &track[1]], [&format!("-({half})"), half], [&track_z[0], &track_z[1]]).visible(&shown);
    }
    let bottom = format!("{frame_top} - {FRAME_H}");
    let lift = [format!("max({bottom} - 60, 90)"), format!("max({bottom} - 30, 120)")];
    f.cuboid(
        SUPPORT,
        ["length / 2 - 300", "length / 2 + 300"],
        [&format!("-({half}) + 100"), &format!("({half}) - 100")],
        [&lift[0], &lift[1]],
    );
    for x in ["length / 2 - 250", "length / 2 + 250"] {
        f.rod_z(DRIVE, [x, "0"], "40", [&format!("max({} - 180, 70)", lift[0]), &lift[0]]);
    }
    drive(f, ["length / 2", &format!("-({half}) - {}", FRAME_CLEAR + FRAME_D + 120.0), &format!("max({bottom} + 40, 130)")], true);
    legs(f, "0", "length", half, &bottom, top);
}

/// S12, S13: a roller deck at `deck` height on a scissor lift (two crossed arms on each side) standing on a base frame.
fn lift_table(f: &mut Forms, deck: &str) {
    let half = "usableWidth / 2";
    let frame_top = format!("{deck} - {FRAME_DROP}");
    let frame_bottom = format!("{frame_top} - 80");
    let wide = [format!("-({half}) - {}", FRAME_CLEAR + FRAME_D), format!("({half}) + {}", FRAME_CLEAR + FRAME_D)];
    f.cuboid(BASE, ["0", "length"], [&wide[0], &wide[1]], ["0", "60"]);
    rollers(f, "0", "length", half, deck, "rollerPitch", "rollerDiameter");
    f.cuboid(DECK, ["0", "length"], [&wide[0], &wide[1]], [&format!("{frame_bottom} - 20"), &frame_bottom]);
    for side in [1.0, -1.0] {
        let (near, far) = if side > 0.0 { (100, 60) } else { (60, 100) };
        let ys = [format!("{side} * (({half}) - {near})"), format!("{side} * (({half}) - {far})")];
        let low = "60";
        let high = format!("{frame_bottom} - 20");
        let a = [p(200, low), p("length - 200", format!("{high} - 40")), p("length - 200", &high), p(200, format!("{low} + 40"))];
        let b = [p(200, format!("{high} - 40")), p("length - 200", low), p("length - 200", format!("{low} + 40")), p(200, &high)];
        for arm in [a, b] {
            f.along_y(SUPPORT, &arm, [&ys[0], &ys[1]]).visible(&format!("{deck} > 260"));
        }
    }
    // Two C-channels under the deck's rollers, lower than a conveyor's: a lift table folds flat.
    for side in [1.0, -1.0] {
        let (inner, outer) = (format!("{side} * (({half}) + {FRAME_CLEAR})"), format!("{side} * (({half}) + {})", FRAME_CLEAR + FRAME_D));
        f.along_x(
            FRAME,
            &[p(&inner, &frame_top), p(&outer, &frame_top), p(&outer, &frame_bottom), p(&inner, &frame_bottom)],
            ["0", "length"],
        );
    }
}

/// S14: two masts beside the deck with a cross beam on top, the hoist drive on it, and the carriage with its roller deck
/// at the entry height, guided on the masts.
fn elevator(f: &mut Forms) {
    let half = "usableWidth / 2";
    let top = "max(entryHeight, exitHeight) + 2000";
    let mast_y = format!("(({half}) + {})", FRAME_CLEAR + FRAME_D + 80.0 + 80.0);
    for side in ["", "-"] {
        let ys = [format!("{side}{mast_y} - 80"), format!("{side}{mast_y} + 80")];
        f.cuboid(GUARD, ["length / 2 - 80", "length / 2 + 80"], [&ys[0], &ys[1]], ["0", top]);
        f.cuboid(
            BASE,
            ["length / 2 - 200", "length / 2 + 200"],
            [&format!("{side}{mast_y} - 200"), &format!("{side}{mast_y} + 200")],
            ["0", "15"],
        );
        // The carriage's guide shoes on the mast.
        let shoe = [format!("{side}(({half}) + {})", FRAME_CLEAR + FRAME_D), format!("{side}({mast_y} - 80)")];
        f.cuboid(
            SUPPORT,
            ["length / 2 - 150", "length / 2 + 150"],
            [&shoe[0], &shoe[1]],
            [&format!("entryHeight - {}", FRAME_DROP + FRAME_H + 60.0), &format!("entryHeight - {FRAME_DROP}")],
        );
    }
    f.cuboid(
        GUARD,
        ["length / 2 - 80", "length / 2 + 80"],
        [&format!("-{mast_y} - 80"), &format!("{mast_y} + 80")],
        [top, &format!("{top} + 160")],
    );
    drive(f, ["length / 2", "0", &format!("{top} + 270")], true);
    rollers(f, "0", "length", half, "entryHeight", "rollerPitch", "rollerDiameter");
    let frame_top = format!("entryHeight - {FRAME_DROP}");
    side_frames(f, ["0", "length"], half, &frame_top);
    f.cuboid(
        DECK,
        ["0", "length"],
        [&format!("-({half})"), half],
        [&format!("{frame_top} - {FRAME_H}"), &format!("{frame_top} - {}", FRAME_H - 20.0)],
    );
}

/// S15: idle rollers on a slope from the entry height down to the exit height, a brake roller every `brakeEvery`
/// rollers, the separator at the start of the pick-up position and the end stop, on side frames and feet that follow
/// the slope.
fn gravity_line(f: &mut Forms) {
    let half = "usableWidth / 2";
    // The conveying surface falls from the entry: its height at x.
    let surface = |x: &str| format!("exitHeight + slope * (length - ({x}))");
    let count = "rounddown(length / rollerPitch)";
    let first = format!("(length - ({count} - 1) * rollerPitch) / 2");
    let centre = format!("{} - rollerDiameter / 2", surface(&first));
    let step = ["rollerPitch", "0", "-slope * rollerPitch"];
    f.rod_y(ROLLER, [&first, &centre], "rollerDiameter / 2", [&format!("-({half})"), half]).repeat(count, step);
    let through = format!("({half}) + {}", FRAME_CLEAR + FRAME_D);
    f.rod_y(ROLLER, [&first, &centre], "10", [&format!("-({through})"), &through]).repeat(count, step);
    let brakes = "roundup(rounddown(length / rollerPitch) / max(1, round(brakeEvery)))";
    let brake_step = ["rollerPitch * max(1, round(brakeEvery))", "0", "-slope * rollerPitch * max(1, round(brakeEvery))"];
    f.rod_y(BRAKE, [&first, &centre], "rollerDiameter / 2 + 6", ["-70", "70"]).repeat(brakes, brake_step);
    // Flat-bar side frames along the slope, 80 mm high: a gravity line runs low at its exit.
    let (top0, top1) = (format!("{} - {FRAME_DROP}", surface("0")), format!("{} - {FRAME_DROP}", surface("length")));
    for side in [1.0, -1.0] {
        let (near, far) = if side > 0.0 { (FRAME_CLEAR, FRAME_CLEAR + FRAME_D) } else { (FRAME_CLEAR + FRAME_D, FRAME_CLEAR) };
        let ys = [format!("{side} * (({half}) + {near})"), format!("{side} * (({half}) + {far})")];
        let bar = [p(0, format!("{top0} - 80")), p("length", format!("{top1} - 80")), p("length", &top1), p(0, &top0)];
        f.along_y(FRAME, &bar, [&ys[0], &ys[1]]);
    }
    // Posts one by one: each reaches up to the frame where it stands.
    let bays = "max(1, roundup((length - 200) / 2000))";
    let yc = format!("(({half}) + {})", FRAME_CLEAR + FRAME_D / 2.0);
    for i in 0..MAX_SUPPORTS {
        let x = format!("100 + {i} * (length - 200) / {bays}");
        let under = |x: &str| format!("{} - {}", surface(x), FRAME_DROP + 80.0);
        let (xa, xb) = (format!("{x} - {}", POST / 2.0), format!("{x} + {}", POST / 2.0));
        let post = [p(&xa, FOOT_H), p(&xb, FOOT_H), p(&xb, under(&xb)), p(&xa, under(&xa))];
        let shown = format!("and({i} <= {bays}, {} > {})", under(&x), FOOT_H + 20.0);
        for side in ["", "-"] {
            let ys = [format!("{side}{yc} - {}", POST / 2.0), format!("{side}{yc} + {}", POST / 2.0)];
            f.along_y(SUPPORT, &post, [&ys[0], &ys[1]]).visible(&shown);
            let plate = [format!("{x} - {}", FOOT_PLATE / 2.0), format!("{x} + {}", FOOT_PLATE / 2.0)];
            let yp = [format!("{side}{yc} - {}", FOOT_PLATE / 2.0), format!("{side}{yc} + {}", FOOT_PLATE / 2.0)];
            f.cuboid(BASE, [&plate[0], &plate[1]], [&yp[0], &yp[1]], ["0", &PLATE_T.to_string()]).visible(&format!("{i} <= {bays}"));
        }
    }
    let separator = "length - pickupLength";
    f.cuboid(
        GUARD,
        [separator, &format!("{separator} + 20")],
        [&format!("-({half}) * 0.6"), &format!("({half}) * 0.6")],
        [&surface(separator), &format!("{} + 80", surface(separator))],
    );
    f.cuboid(
        FRAME,
        ["length", "length + 20"],
        [&format!("-({half})"), half],
        [&format!("{} - 30", surface("length")), &format!("{} + 150", surface("length"))],
    );
}

/// S11: a short roller deck under a gantry that stands for the work being done, on braced feet.
fn process_station(f: &mut Forms) {
    let (half, top) = ("450", "surfaceHeight");
    let frame_top = format!("{top} - {FRAME_DROP}");
    rollers(f, "0", "length", half, top, "206.3", "89");
    side_frames(f, ["0", "length"], half, &frame_top);
    let bottom = format!("{frame_top} - {FRAME_H}");
    legs(f, "0", "length", half, &bottom, top);
    drive(f, ["length / 2", "-250", &format!("max({bottom} - 170, 130)")], false);
    let gantry_top = format!("{top} + 1900");
    let post_y = 450.0 + FRAME_CLEAR + FRAME_D + 100.0;
    for side in [1.0, -1.0] {
        let ys = [format!("{} - 50", side * post_y), format!("{} + 50", side * post_y)];
        f.cuboid(GANTRY, ["length / 2 - 50", "length / 2 + 50"], [&ys[0], &ys[1]], ["0", &gantry_top]);
    }
    let span = format!("{}", post_y + 50.0);
    f.cuboid(GANTRY, ["length / 2 - 150", "length / 2 + 150"], [&format!("-{span}"), &span], [&gantry_top, &format!("{gantry_top} + 150")]);
    f.cuboid(DRIVE, ["length / 2 - 120", "length / 2 + 120"], ["-120", "120"], [&format!("{gantry_top} - 250"), &gantry_top]);
}

/// A source or a sink: a stand 400 mm long at the port height with an arrow on it, out of the model (a source) or into
/// it (a sink).
fn flow_end(f: &mut Forms, material: &str, out: bool) {
    let top = "surfaceHeight";
    f.cuboid(material, ["0", "400"], ["-450", "450"], [&format!("{top} - 40"), top]);
    f.cuboid(SUPPORT, ["170", "230"], ["-30", "30"], ["0", &format!("max({top} - 40, 20)")]);
    f.cuboid(BASE, ["100", "300"], ["-150", "150"], ["0", "10"]);
    let (tail, tip) = if out { (60, 340) } else { (340, 60) };
    let arrow = json!([[tail, -120], [tail, 120], [tip, 0]]);
    f.push(json!({"plane": "xy", "profile": arrow, "from": top, "to": format!("{top} + 20"), "material": material}));
}

/// S06: the sensor and the reflector on their posts either side of the conveyor, and the beam between them. The family
/// stands on the floor: the heads are drawn over a 600 mm conveying surface, the library's default.
fn photoeye(f: &mut Forms) {
    let (zs, zr) = ("600 + sensorHeight", "600 + reflectorHeight");
    for (y, z, w, material) in [("sensorLateral", zs, 50.0, GUARD), ("reflectorLateral", zr, 70.0, SINK)] {
        f.cuboid(SUPPORT, ["-10", "10"], [&format!("({y}) - 10"), &format!("({y}) + 10")], ["0", &format!("{z} - 40")]);
        f.cuboid(
            material,
            ["-25", "25"],
            [&format!("({y}) - {}", w / 2.0), &format!("({y}) + {}", w / 2.0)],
            [&format!("{z} - 40"), &format!("{z} + 40")],
        );
    }
    let beam = [p("-6", format!("{zs} - 6")), p("6", format!("{zs} - 6")), p("6", format!("{zr} + 6")), p("-6", format!("{zr} + 6"))];
    f.along_y(BEAM, &beam, ["min(sensorLateral, reflectorLateral) + 25", "max(sensorLateral, reflectorLateral) - 25"])
        .visible("and(showBeam, abs(sensorLateral - reflectorLateral) > 60)");
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bimcraft_core::family::Family;
    use intracraft_library::Library;
    use intracraft_library::definition::{ParamType, Value};

    use super::*;
    use crate::family::{DEFAULT_TYPE, family_params, to_bim};

    /// A definition's family as BIMCraft reads it.
    fn family(d: &Definition) -> Family {
        let mut json = family_params(d);
        json.as_object_mut().unwrap().remove("name");
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn every_family_builds_at_every_parameter_s_least_and_greatest_value_and_at_all_of_them_at_once() {
        let lib = Library::built_in().unwrap();
        for d in lib.list() {
            let f = family(d);
            let numbers: Vec<_> =
                d.parameters.iter().filter(|p| !p.derived && matches!(p.kind, ParamType::Number | ParamType::Integer)).collect();
            let ends = |pick: fn(&intracraft_library::definition::Parameter) -> Option<f64>| -> BTreeMap<String, Json> {
                numbers.iter().filter_map(|p| pick(p).map(|v| (p.key.clone(), to_bim(p, &Value::Number(v))))).collect()
            };
            let low = |p: &intracraft_library::definition::Parameter| p.min.or(p.choices.as_ref().and_then(|c| c.first().copied()));
            let high = |p: &intracraft_library::definition::Parameter| p.max.or(p.choices.as_ref().and_then(|c| c.last().copied()));
            let mut cases = vec![ends(low), ends(high)];
            for p in &numbers {
                for v in [low(p), high(p)].into_iter().flatten() {
                    cases.push(BTreeMap::from([(p.key.clone(), to_bim(p, &Value::Number(v)))]));
                }
            }
            for case in cases {
                let built = f.values(DEFAULT_TYPE, &case).and_then(|v| f.pieces(&v));
                assert!(built.as_ref().is_ok_and(|pieces| !pieces.is_empty()), "{} with {case:?}: {built:?}", d.id);
            }
        }
    }

    #[test]
    fn a_roller_conveyor_has_a_roller_at_each_pitch_and_a_pair_of_legs_at_most_2_m_apart() {
        let lib = Library::built_in().unwrap();
        let f = family(lib.get("conveyor.roller").unwrap());
        let pieces = |length: f64| {
            let v = f.values(DEFAULT_TYPE, &BTreeMap::from([("length".to_string(), json!(length))])).unwrap();
            f.pieces(&v).unwrap()
        };
        // Round steel: each roller and its axle, and the screw of each foot (two per pair of legs).
        let round_steel = |length| pieces(length).iter().filter(|p| p.material == ROLLER && p.round).count();
        let posts = |length| pieces(length).iter().filter(|p| p.material == SUPPORT && !p.round).count();
        // 3 m at 206.3 mm: 14 rollers on 3 pairs of legs (2.8 m between the end pairs, at most 2 m apart: two bays); 6 m:
        // 29 rollers on 4 pairs.
        assert_eq!(round_steel(3000.0), 2 * 14 + 2 * 3);
        assert_eq!(round_steel(6000.0), 2 * 29 + 2 * 4);
        // At 600 mm: the posts alone, no cross beam or brace.
        assert_eq!(posts(6000.0), 2 * 4);
    }
}

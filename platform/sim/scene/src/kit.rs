//! The parts the objects are made of, laid out in an instance's own frame (x along the travel from its entry, y up from
//! its base, z across) and placed in the world by its position and yaw.

use intracraft_des::flow::storage::StorageData;
use intracraft_library::params::{Values, num};

use crate::{Paint, Part, Shape};

/// The side frames' top lies this far under the conveying surface (a pallet wider than the deck clears them), and the
/// frames' height, depth and clearance from the rollers, m (the families' C-channels).
pub const FRAME_DROP: f64 = 0.01;
pub const FRAME_H: f64 = 0.12;
const FRAME_D: f64 = 0.05;
const FRAME_CLEAR: f64 = 0.005;
/// A chain strand's height and its track's, m.
pub const CHAIN_H: f64 = 0.03;
pub const TRACK_H: f64 = 0.06;
/// A turntable's rollers stop this far short of its platform's width, m (the physics run's).
pub const TURNTABLE_EDGE: f64 = 0.12;
/// Chain strands keep this far from the deck's edges, m (the physics run's).
const CHAIN_EDGE_INSET: f64 = 0.1;
/// Legs: square posts, their feet's height, the floor plates, and the most a span between legs may be, m.
const POST: f64 = 0.05;
const FOOT_H: f64 = 0.06;
const FOOT_PLATE: f64 = 0.12;
const PLATE_T: f64 = 0.01;
const MAX_LEG_SPAN: f64 = 2.0;
/// Legs this tall or more get a cross beam, m.
const BEAM_FROM: f64 = 0.65;
/// A rack's uprights, its beams' height and depth, and how far a beam lies from the edge of its position, m.
const RACK_POST: f64 = 0.1;
const RACK_BEAM_H: f64 = 0.1;
const RACK_BEAM_D: f64 = 0.05;
const RACK_BEAM_INSET: f64 = 0.15;
/// A stacker crane's rails start this far in front of the rack; its masts stand this far before and behind its forks;
/// its top beam runs this far above the rack, m.
pub const CRANE_RAIL_FRONT: f64 = -2.2;
const CRANE_MAST_AT: f64 = 1.0;
const CRANE_OVER_RACK: f64 = 0.4;

pub struct Kit {
    origin: [f64; 3],
    yaw: f64,
    fixed: Vec<Part>,
    moving: Vec<Part>,
}

impl Kit {
    pub fn new(origin: [f64; 3], yaw: f64) -> Kit {
        Kit { origin, yaw, fixed: Vec::new(), moving: Vec::new() }
    }

    pub fn finish(self) -> (Vec<Part>, Vec<Part>) {
        (self.fixed, self.moving)
    }

    /// A point of the instance's frame in the world.
    pub fn world(&self, local: [f64; 3]) -> [f64; 3] {
        let (s, c) = self.yaw.sin_cos();
        [self.origin[0] + local[0] * c + local[2] * s, self.origin[1] + local[1], self.origin[2] - local[0] * s + local[2] * c]
    }

    fn part(&mut self, shape: Shape, local: [f64; 3], paint: Paint, (turn, tilt): (f64, f64), moving: bool) {
        let p = Part { shape, centre: self.world(local), yaw: self.yaw + turn, tilt, paint };
        if moving { self.moving.push(p) } else { self.fixed.push(p) }
    }

    pub fn fixed(&mut self, shape: Shape, local: [f64; 3], paint: Paint) {
        self.part(shape, local, paint, (0.0, 0.0), false);
    }

    pub fn moving(&mut self, shape: Shape, local: [f64; 3], paint: Paint) {
        self.part(shape, local, paint, (0.0, 0.0), true);
    }

    /// Rollers across the travel at their pitch from `x0`, their tops at `top`.
    pub fn roller_deck(&mut self, x0: f64, length: f64, width: f64, top: f64, v: &Values, moving: bool) {
        let radius = num(v, "rollerDiameter", 0.089) / 2.0;
        for x in roller_positions(length, num(v, "rollerPitch", 0.2063)) {
            self.part(Shape::Roller { radius, length: width }, [x0 + x, top - radius, 0.0], Paint::Roller, (0.0, 0.0), moving);
        }
    }

    /// Chain strands along the travel on their tracks, their tops at `top`, with a sprocket shaft at each end.
    pub fn strands(&mut self, x0: f64, length: f64, width: f64, top: f64, v: &Values, moving: bool) {
        let strand = num(v, "chainSupportWidth", 0.04);
        let zs = chain_layout(width, num(v, "chainCount", 3.0));
        for &z in &zs {
            let flat = (0.0, 0.0);
            self.part(Shape::Box([length, CHAIN_H, strand]), [x0 + length / 2.0, top - CHAIN_H / 2.0, z], Paint::Chain, flat, moving);
            let track = [x0 + length / 2.0, top - CHAIN_H - TRACK_H / 2.0, z];
            self.part(Shape::Box([length, TRACK_H, strand + 0.02]), track, Paint::Frame, flat, moving);
        }
        let span = zs.iter().fold(0.0f64, |m, z| m.max(z.abs())) * 2.0 + strand;
        for x in [x0 + 0.06, x0 + length - 0.06] {
            let shaft = Shape::Roller { radius: 0.05, length: span };
            self.part(shaft, [x, top - CHAIN_H - 0.05, 0.0], Paint::Chain, (0.0, 0.0), moving);
        }
    }

    /// The two steel side frames along the travel, their tops at `top`.
    pub fn side_frames(&mut self, x0: f64, length: f64, width: f64, top: f64, moving: bool) {
        for side in [-1.0, 1.0] {
            let z = side * (width / 2.0 + FRAME_CLEAR + FRAME_D / 2.0);
            let frame = Shape::Box([length, FRAME_H, FRAME_D]);
            self.part(frame, [x0 + length / 2.0, top - FRAME_H / 2.0, z], Paint::Frame, (0.0, 0.0), moving);
        }
    }

    /// Legs under the side frames, at most 2 m apart, on adjustable feet and floor plates, up to `top`; a cross beam
    /// between each pair when they are tall.
    pub fn legs(&mut self, x0: f64, length: f64, width: f64, top: f64) {
        if top <= FOOT_H + PLATE_T {
            return;
        }
        let half = width / 2.0 + FRAME_CLEAR + FRAME_D / 2.0;
        let (first, last) = (x0 + 0.1_f64.min(length / 2.0), x0 + (length - 0.1).max(length / 2.0));
        let spans = ((last - first) / MAX_LEG_SPAN).ceil().max(1.0) as usize;
        for k in 0..=spans {
            let x = first + (last - first) * k as f64 / spans as f64;
            for side in [-1.0, 1.0] {
                let z = side * half;
                self.fixed(Shape::Box([FOOT_PLATE, PLATE_T, FOOT_PLATE]), [x, PLATE_T / 2.0, z], Paint::Support);
                self.fixed(Shape::Post { radius: 0.012, height: FOOT_H }, [x, PLATE_T + FOOT_H / 2.0, z], Paint::Base);
                let (from, to) = (PLATE_T + FOOT_H, top);
                self.fixed(Shape::Box([POST, to - from, POST]), [x, (from + to) / 2.0, z], Paint::Support);
            }
            if top >= BEAM_FROM {
                self.fixed(Shape::Box([POST, POST, 2.0 * half]), [x, 0.3, 0.0], Paint::Support);
            }
        }
    }

    /// The geared motor: the motor and its gearbox.
    pub fn drive(&mut self, at: [f64; 3]) {
        self.fixed(Shape::Box([0.28, 0.2, 0.2]), at, Paint::Drive);
        self.fixed(Shape::Roller { radius: 0.09, length: 0.24 }, [at[0] - 0.22, at[1], at[2]], Paint::Drive);
    }

    /// A turntable's base plate, column and slewing ring under its platform, and its drive.
    pub fn turntable_base(&mut self, pivot: [f64; 3], top: f64) {
        let ring_top = (top - FRAME_DROP - FRAME_H - 0.04).max(0.16);
        let ring_bottom = ring_top - 0.05;
        self.fixed(Shape::Box([1.1, 0.05, 1.1]), [pivot[0], 0.025, pivot[2]], Paint::Base);
        let column = (ring_bottom - 0.05).max(0.01);
        self.fixed(Shape::Post { radius: 0.175, height: column }, [pivot[0], 0.05 + column / 2.0, pivot[2]], Paint::Support);
        self.fixed(Shape::Post { radius: 0.225, height: 0.05 }, [pivot[0], ring_bottom + 0.025, pivot[2]], Paint::Roller);
        self.drive([pivot[0] + 0.47, (ring_bottom * 0.55).max(0.15), pivot[2]]);
    }

    /// A stretch wrapper (S25): its round table on a column and a floor plate, the rollers inside the circle and centred on
    /// it as in the physics run (they turn with the table), and beside it the mast with the film carriage and its roll at
    /// the bottom of their travel.
    pub fn wrapper(&mut self, v: &Values) {
        let (length, width, top) = (num(v, "length", 1.8), num(v, "usableWidth", 1.0), num(v, "surfaceHeight", 0.6));
        let (r, cx) = (length / 2.0, length / 2.0);
        let radius = num(v, "rollerDiameter", 0.089) / 2.0;
        let disc_top = (top - 2.0 * radius - 0.02).max(0.12);
        self.fixed(Shape::Post { radius: r - 0.1, height: 0.05 }, [cx, 0.025, 0.0], Paint::Base);
        let column = (disc_top - 0.04 - 0.05).max(0.01);
        self.fixed(Shape::Post { radius: 0.35, height: column }, [cx, 0.05 + column / 2.0, 0.0], Paint::Support);
        self.moving(Shape::Post { radius: r, height: 0.04 }, [cx, disc_top - 0.02, 0.0], Paint::Deck);
        let xs = roller_positions(length, num(v, "rollerPitch", 0.2063));
        let shift = match (xs.first(), xs.last()) {
            (Some(a), Some(b)) => cx - (a + b) / 2.0,
            _ => 0.0,
        };
        for x in xs.into_iter().map(|x| x + shift) {
            let half = (width / 2.0).min((r * r - (x - cx).powi(2)).max(0.0).sqrt());
            if half >= 0.05 {
                self.moving(Shape::Roller { radius, length: 2.0 * half }, [x, top - radius, 0.0], Paint::Roller);
            }
        }
        // The mast beside the table, as high as the tallest load, its drive on top; the carriage and roll facing the load.
        let (mast_z, mast_top) = (r + 0.25, top + num(v, "maxLoadHeight", 2.0) + 0.3);
        self.fixed(Shape::Box([0.2, mast_top, 0.2]), [cx, mast_top / 2.0, mast_z], Paint::Guard);
        self.fixed(Shape::Box([0.5, 0.015, 0.4]), [cx, 0.0075, mast_z], Paint::Base);
        self.drive([cx, mast_top + 0.11, mast_z]);
        let film = num(v, "filmWidth", 0.5);
        self.fixed(Shape::Box([0.5, film, 0.09]), [cx, top + 0.1 + film / 2.0, r + 0.105], Paint::Frame);
        self.fixed(Shape::Post { radius: 0.04, height: film }, [cx, top + 0.1 + film / 2.0, r + 0.02], Paint::Roller);
    }

    /// The transfer table's chains, lowered in the gaps between its rollers, each on its track.
    pub fn transfer_chains(&mut self, length: f64, width: f64, top: f64, v: &Values) {
        let pitch = num(v, "rollerPitch", 0.2063);
        let rollers = roller_positions(length, pitch).len();
        let count = num(v, "transferChainCount", 3.0).round().max(1.0) as usize;
        let chain = num(v, "transferChainWidth", 0.04);
        let chain_top = top - num(v, "lowerClearance", 0.01);
        let gaps = rollers.saturating_sub(1).max(1);
        for k in 0..count {
            let gap = if count == 1 { gaps / 2 } else { (k * (gaps - 1)) / (count - 1) };
            let x = pitch / 2.0 + pitch * (gap as f64 + 0.5);
            self.fixed(Shape::Box([chain, CHAIN_H, width]), [x, chain_top - CHAIN_H / 2.0, 0.0], Paint::Chain);
            self.fixed(Shape::Box([chain + 0.02, TRACK_H, width]), [x, chain_top - CHAIN_H - TRACK_H / 2.0, 0.0], Paint::Frame);
        }
    }

    /// A process station's gantry: two posts beside the deck and a beam across with the tool head.
    pub fn gantry(&mut self, length: f64, width: f64, top: f64) {
        let (x, half, height) = (length / 2.0, width / 2.0 + 0.25, top + 1.6);
        for side in [-1.0, 1.0] {
            self.fixed(Shape::Box([0.12, height, 0.12]), [x, height / 2.0, side * half], Paint::Gantry);
        }
        self.fixed(Shape::Box([0.14, 0.14, 2.0 * half + 0.12]), [x, height, 0.0], Paint::Gantry);
        self.fixed(Shape::Box([0.25, 0.3, 0.25]), [x, height - 0.22, 0.0], Paint::Drive);
    }

    /// An elevator's two masts beside the deck, the beam across their tops and the hoist on it.
    pub fn elevator_masts(&mut self, length: f64, width: f64, top: f64) {
        let (x, half, height) = (length / 2.0, width / 2.0 + FRAME_CLEAR + FRAME_D + 0.16, top + 2.0);
        for side in [-1.0, 1.0] {
            self.fixed(Shape::Box([0.3, 0.02, 0.3]), [x, 0.01, side * half], Paint::Base);
            self.fixed(Shape::Box([0.16, height, 0.16]), [x, height / 2.0, side * half], Paint::Frame);
        }
        self.fixed(Shape::Box([0.2, 0.2, 2.0 * half + 0.16]), [x, height + 0.1, 0.0], Paint::Frame);
        self.fixed(Shape::Box([0.4, 0.3, 0.4]), [x, height + 0.35, 0.0], Paint::Drive);
    }

    /// A gravity roller line: rollers on the slope (every so many a brake roller), the side frames, legs as tall as the
    /// slope makes them, and the end stop.
    pub fn gravity_line(&mut self, v: &Values) {
        let (length, width, slope) = (num(v, "length", 6.0), num(v, "usableWidth", 1.0), num(v, "slope", 0.04));
        let exit = num(v, "exitHeight", 0.1);
        let entry = v.get("entryHeight").and_then(|e| e.number()).unwrap_or(exit + slope * length);
        let fall = (entry - exit) / length.max(1e-9);
        let tilt = fall.atan();
        let top = |x: f64| entry - fall * x;
        let radius = num(v, "rollerDiameter", 0.089) / 2.0;
        let brake_every = num(v, "brakeEvery", 4.0).round().max(0.0) as usize;
        for (k, x) in roller_positions(length, num(v, "rollerPitch", 0.2063)).into_iter().enumerate() {
            let paint = if brake_every > 0 && (k + 1) % brake_every == 0 { Paint::Brake } else { Paint::Roller };
            self.part(Shape::Roller { radius, length: width }, [x, top(x) - radius, 0.0], paint, (0.0, tilt), false);
        }
        let along = length / tilt.cos();
        for side in [-1.0, 1.0] {
            let z = side * (width / 2.0 + FRAME_CLEAR + FRAME_D / 2.0);
            let frame = [length / 2.0, top(length / 2.0) - FRAME_DROP - FRAME_H / 2.0, z];
            self.part(Shape::Box([along, FRAME_H, FRAME_D]), frame, Paint::Frame, (0.0, tilt), false);
        }
        let legs = ((length - 0.2) / MAX_LEG_SPAN).ceil().max(1.0) as usize;
        for k in 0..=legs {
            let x = 0.1 + (length - 0.2) * k as f64 / legs as f64;
            let bottom = top(x) - FRAME_DROP - FRAME_H;
            if bottom > FOOT_H + PLATE_T {
                for side in [-1.0, 1.0] {
                    let z = side * (width / 2.0 + FRAME_CLEAR + FRAME_D / 2.0);
                    self.fixed(Shape::Box([FOOT_PLATE, PLATE_T, FOOT_PLATE]), [x, PLATE_T / 2.0, z], Paint::Support);
                    let from = PLATE_T;
                    self.fixed(Shape::Box([POST, bottom - from, POST]), [x, (from + bottom) / 2.0, z], Paint::Support);
                }
            }
        }
        self.fixed(Shape::Box([0.02, 0.15, width * 0.8]), [length + 0.01, top(length) + 0.045, 0.0], Paint::Brake);
    }

    /// A storage aisle's rack (S26 §1): uprights at every bay line, front and back of each position, and under every
    /// level two beams along the rack per position; the crane's floor rail and its guide rail above the rack.
    pub fn rack(&mut self, a: &StorageData) {
        let (length, height) = (a.bays as f64 * a.bay_pitch, a.rack_height());
        let inner = a.aisle_width / 2.0;
        for side in [-1.0, 1.0] {
            // The upright lines: the aisle side, the back, and between two positions when double deep.
            for line in 0..=a.deep {
                let inset = if line == 0 {
                    RACK_POST / 2.0
                } else if line == a.deep {
                    -RACK_POST / 2.0
                } else {
                    0.0
                };
                let z = side * (inner + line as f64 * a.position_depth + inset);
                for bay in 0..=a.bays {
                    self.fixed(Shape::Box([RACK_POST, height, RACK_POST]), [bay as f64 * a.bay_pitch, height / 2.0, z], Paint::Frame);
                }
            }
            for level in 0..a.levels {
                let y = a.first_level_height + level as f64 * a.level_pitch - RACK_BEAM_H / 2.0;
                for deep in 0..a.deep {
                    let from = inner + deep as f64 * a.position_depth;
                    for z in [from + RACK_BEAM_INSET, from + a.position_depth - RACK_BEAM_INSET] {
                        self.fixed(Shape::Box([length, RACK_BEAM_H, RACK_BEAM_D]), [length / 2.0, y, side * z], Paint::Guard);
                    }
                }
            }
        }
        let (from, to) = (CRANE_RAIL_FRONT, length + 0.5);
        self.fixed(Shape::Box([to - from, 0.05, 0.1]), [(from + to) / 2.0, 0.025, 0.0], Paint::Support);
        self.fixed(Shape::Box([to - from, 0.08, 0.1]), [(from + to) / 2.0, height + CRANE_OVER_RACK + 0.04, 0.0], Paint::Support);
    }

    /// The stacker crane's travelling frame as it stands with its forks at `x`: the base on the floor rail, a mast in
    /// front of the forks and one behind, and the beam across their tops under the guide rail. All of it moves.
    pub fn crane_mast(&mut self, a: &StorageData, x: f64) {
        let top = a.rack_height() + CRANE_OVER_RACK;
        self.moving(Shape::Box([2.0 * CRANE_MAST_AT + 0.6, 0.25, 0.5]), [x, 0.175, 0.0], Paint::Gantry);
        for side in [-1.0, 1.0] {
            let mast = Shape::Box([0.25, top - 0.3, 0.3]);
            self.moving(mast, [x + side * CRANE_MAST_AT, 0.3 + (top - 0.3) / 2.0, 0.0], Paint::Support);
        }
        self.moving(Shape::Box([2.0 * CRANE_MAST_AT + 0.25, 0.2, 0.3]), [x, top - 0.1, 0.0], Paint::Gantry);
        self.moving(Shape::Box([0.4, 0.4, 0.45]), [x - CRANE_MAST_AT - 0.3, 0.5, 0.0], Paint::Drive);
    }

    /// The crane's lifting carriage with the forks' level at `y`: the platform between the masts.
    pub fn crane_carriage(&mut self, a: &StorageData, x: f64, y: f64) {
        let width = (a.aisle_width - 0.2).max(0.4);
        self.moving(Shape::Box([2.0 * CRANE_MAST_AT - 0.3, 0.12, width]), [x, y - 0.14, 0.0], Paint::Gantry);
        for side in [-1.0, 1.0] {
            self.moving(Shape::Box([0.08, 0.5, width]), [x + side * (CRANE_MAST_AT - 0.2), y + 0.11, 0.0], Paint::Guard);
        }
    }

    /// The crane's two telescopic forks, drawn in under the pallet's place at height `y`.
    pub fn crane_forks(&mut self, a: &StorageData, x: f64, y: f64) {
        let length = (a.position_depth - 0.05).max(0.5);
        for side in [-1.0, 1.0] {
            self.moving(Shape::Box([0.14, 0.06, length]), [x + side * 0.22, y - 0.035, 0.0], Paint::Chain);
        }
    }

    /// A source's or sink's stand: a plate at the conveying height, on four posts.
    pub fn stand(&mut self, from: f64, to: f64, width: f64, top: f64, paint: Paint) {
        let plate = top - 0.03;
        self.fixed(Shape::Box([to - from, 0.03, width]), [(from + to) / 2.0, plate - 0.015, 0.0], paint);
        if plate > 0.05 {
            for x in [from + 0.05, to - 0.05] {
                for z in [-width / 2.0 + 0.05, width / 2.0 - 0.05] {
                    self.fixed(Shape::Box([POST, plate - 0.03, POST]), [x, (plate - 0.03) / 2.0, z], Paint::Support);
                }
            }
        }
    }

    /// A photocell across its host: its post and head, the reflector's, and the beam between, in the host's frame whose
    /// conveying surface is `host_top` high.
    pub fn photoeye(&mut self, v: &Values, host_top: f64) {
        let sensor = [num(v, "sensorS", 0.0), host_top + num(v, "sensorHeight", 0.06), num(v, "sensorLateral", -0.8)];
        let reflector = [num(v, "reflectorS", 0.0), host_top + num(v, "reflectorHeight", 0.06), num(v, "reflectorLateral", 0.8)];
        for (at, head) in [(sensor, Paint::Drive), (reflector, Paint::Guard)] {
            let height = at[1] + 0.06;
            self.fixed(Shape::Post { radius: 0.02, height }, [at[0], height / 2.0, at[2]], Paint::Support);
            self.fixed(Shape::Box([0.06, 0.08, 0.05]), at, head);
        }
        let (dx, dz) = (reflector[0] - sensor[0], reflector[2] - sensor[2]);
        let span = (dx * dx + dz * dz).sqrt();
        if span > 1e-6 {
            // A box along its own x, turned so that x points from the sensor to the reflector (+x turns toward −z).
            let turn = (-dz).atan2(dx);
            let mid = [(sensor[0] + reflector[0]) / 2.0, (sensor[1] + reflector[1]) / 2.0, (sensor[2] + reflector[2]) / 2.0];
            self.part(Shape::Box([span, 0.004, 0.004]), mid, Paint::Beam, (turn, 0.0), false);
        }
    }
}

/// The rollers' positions along a deck, m from its start: the physics run's (`flow::roller_positions`).
pub fn roller_positions(length: f64, pitch: f64) -> Vec<f64> {
    if pitch <= 0.0 {
        return Vec::new();
    }
    let count = (length / pitch + 1e-9).floor() as usize;
    (0..count).map(|i| pitch / 2.0 + i as f64 * pitch).collect()
}

/// The lateral positions of `count` chain strands across `width`: the physics run's (`flow::chain_layout`).
pub fn chain_layout(width: f64, count: f64) -> Vec<f64> {
    let count = count.round().max(2.0) as usize;
    let span = (width - 2.0 * CHAIN_EDGE_INSET).max(0.0);
    (0..count).map(|i| -span / 2.0 + i as f64 * span / (count - 1) as f64).collect()
}

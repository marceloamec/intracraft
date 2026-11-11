//! The benchmark scene of spike R5b ([[ADR-0019 Game Engine Choice for the Rust Platform]]).
//!
//! A closed loop of 20 roller conveyors (8 + 2 + 8 + 2), 100 rollers each (2 000 rollers at a pitch of 206.3 mm), and
//! 200 EUR pallets with a load each, moving along the loop. Both viewers draw the same scene from the same snapshots,
//! so their frame rates compare like for like. Units SI, Y up, right-handed; the plan view looks down −Y.
//!
//! The motion is plain kinematics (no physics): the point is to measure how fast a viewer draws a snapshot of
//! thousands of moving bodies, which is what the 3D run will send it.

pub mod bench;
pub mod report;

use std::f32::consts::TAU;

/// Roller pitch, m (S01 default).
pub const PITCH: f32 = 0.2063;
/// Roller radius, m (Ø 89 mm).
pub const ROLLER_RADIUS: f32 = 0.0445;
/// Roller length across the conveyor, m.
pub const ROLLER_LENGTH: f32 = 1.0;
/// Top of the rollers above the floor, m.
pub const SURFACE_HEIGHT: f32 = 0.52;
/// Rollers on each conveyor.
pub const ROLLERS_PER_CONVEYOR: usize = 100;
/// Conveyors on each side of the loop, in order: long, short, long, short.
pub const SIDES: [usize; 4] = [8, 2, 8, 2];
/// Pallets on the loop.
pub const PALLETS: usize = 200;
/// Conveying speed, m/s.
pub const SPEED: f32 = 0.5;
/// EUR pallet, m: length (along travel on rollers), height, width.
pub const PALLET_SIZE: [f32; 3] = [1.2, 0.144, 0.8];
/// The load block on each pallet, m.
pub const LOAD_SIZE: [f32; 3] = [1.2, 1.0, 0.8];

/// Where a body is: its origin (the bottom centre for pallets, the axis centre for rollers), its turn about +Y and,
/// for rollers, its spin about its own axis (local +Z), radians.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pose {
    pub pos: [f32; 3],
    pub yaw: f32,
    pub spin: f32,
}

/// A box of the static scene (frames, legs, floor): its centre, size along its local axes, and turn about +Y.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxShape {
    pub centre: [f32; 3],
    pub size: [f32; 3],
    pub yaw: f32,
    pub color: [f32; 3],
}

/// One conveyor of the loop: where it starts, its direction in plan (unit, x and z) and its length.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Conveyor {
    pub start: [f32; 2],
    pub dir: [f32; 2],
    pub length: f32,
}

/// What the viewer draws in one frame: every roller and every pallet (the load sits on its pallet).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Snapshot {
    pub rollers: Vec<Pose>,
    pub pallets: Vec<Pose>,
}

/// The scene: the loop's conveyors and the static boxes.
#[derive(Clone, Debug)]
pub struct Scene {
    pub conveyors: Vec<Conveyor>,
}

impl Default for Scene {
    fn default() -> Self {
        Scene::new()
    }
}

impl Scene {
    pub fn new() -> Scene {
        let length = PITCH * ROLLERS_PER_CONVEYOR as f32;
        let dirs = [[1.0, 0.0], [0.0, 1.0], [-1.0, 0.0], [0.0, -1.0]];
        let mut conveyors = Vec::new();
        let mut at = [0.0_f32, 0.0];
        for (side, &count) in SIDES.iter().enumerate() {
            let dir = dirs[side];
            for _ in 0..count {
                conveyors.push(Conveyor { start: at, dir, length });
                at = [at[0] + dir[0] * length, at[1] + dir[1] * length];
            }
        }
        Scene { conveyors }
    }

    /// Length of the loop, m.
    pub fn perimeter(&self) -> f32 {
        self.conveyors.iter().map(|c| c.length).sum()
    }

    /// The middle of the loop in plan (x, z), and the radius of a circle around it.
    pub fn centre_and_radius(&self) -> ([f32; 2], f32) {
        let (mut min, mut max) = ([f32::MAX; 2], [f32::MIN; 2]);
        for c in &self.conveyors {
            for p in [c.start, [c.start[0] + c.dir[0] * c.length, c.start[1] + c.dir[1] * c.length]] {
                for i in 0..2 {
                    min[i] = min[i].min(p[i]);
                    max[i] = max[i].max(p[i]);
                }
            }
        }
        let centre = [(min[0] + max[0]) / 2.0, (min[1] + max[1]) / 2.0];
        let radius = ((max[0] - min[0]).powi(2) + (max[1] - min[1]).powi(2)).sqrt() / 2.0;
        (centre, radius)
    }

    /// The static boxes: the floor, two side frames per conveyor and its legs.
    pub fn statics(&self) -> Vec<BoxShape> {
        let (centre, radius) = self.centre_and_radius();
        let mut boxes = vec![BoxShape {
            centre: [centre[0], -0.05, centre[1]],
            size: [radius * 2.4, 0.1, radius * 2.4],
            yaw: 0.0,
            color: [0.55, 0.56, 0.58],
        }];
        let frame = [0.25, 0.32, 0.55];
        let leg = [0.35, 0.35, 0.38];
        for c in &self.conveyors {
            let yaw = yaw_of(c.dir);
            let side = [-c.dir[1], c.dir[0]];
            let mid = [c.start[0] + c.dir[0] * c.length / 2.0, c.start[1] + c.dir[1] * c.length / 2.0];
            let rail_y = SURFACE_HEIGHT - ROLLER_RADIUS;
            for s in [-1.0_f32, 1.0] {
                let off = s * (ROLLER_LENGTH / 2.0 + 0.04);
                boxes.push(BoxShape {
                    centre: [mid[0] + side[0] * off, rail_y, mid[1] + side[1] * off],
                    size: [c.length, 0.16, 0.06],
                    yaw,
                    color: frame,
                });
                let legs = (c.length / 2.0).floor() as usize + 1;
                for k in 0..legs {
                    let along = (k as f32 * c.length / (legs - 1).max(1) as f32).clamp(0.05, c.length - 0.05);
                    boxes.push(BoxShape {
                        centre: [
                            c.start[0] + c.dir[0] * along + side[0] * off,
                            (rail_y - 0.08) / 2.0,
                            c.start[1] + c.dir[1] * along + side[1] * off,
                        ],
                        size: [0.06, rail_y - 0.08, 0.06],
                        yaw,
                        color: leg,
                    });
                }
            }
        }
        boxes
    }

    /// The scene at time `t`, s, into `snap` (its vectors are reused, so a viewer allocates once).
    pub fn snapshot(&self, t: f32, snap: &mut Snapshot) {
        snap.rollers.clear();
        snap.pallets.clear();
        let spin = -(SPEED * t / ROLLER_RADIUS) % TAU;
        for c in &self.conveyors {
            let yaw = yaw_of(c.dir);
            for k in 0..ROLLERS_PER_CONVEYOR {
                let along = PITCH * (k as f32 + 0.5);
                snap.rollers.push(Pose {
                    pos: [c.start[0] + c.dir[0] * along, SURFACE_HEIGHT - ROLLER_RADIUS, c.start[1] + c.dir[1] * along],
                    yaw,
                    spin,
                });
            }
        }
        let perimeter = self.perimeter();
        let gap = perimeter / PALLETS as f32;
        for p in 0..PALLETS {
            let s = (p as f32 * gap + SPEED * t).rem_euclid(perimeter);
            snap.pallets.push(self.at(s));
        }
    }

    /// A pallet's pose at distance `s` along the loop.
    fn at(&self, s: f32) -> Pose {
        let mut rest = s;
        for c in &self.conveyors {
            if rest < c.length {
                return Pose {
                    pos: [c.start[0] + c.dir[0] * rest, SURFACE_HEIGHT, c.start[1] + c.dir[1] * rest],
                    yaw: yaw_of(c.dir),
                    spin: 0.0,
                };
            }
            rest -= c.length;
        }
        let first =
            self.conveyors.first().copied().unwrap_or(Conveyor { start: [0.0; 2], dir: [1.0, 0.0], length: 0.0 });
        Pose { pos: [first.start[0], SURFACE_HEIGHT, first.start[1]], yaw: yaw_of(first.dir), spin: 0.0 }
    }

    /// The camera at time `t` of a turn lasting `period` s: eye and target. It circles the loop from above.
    pub fn camera(&self, t: f32, period: f32) -> ([f32; 3], [f32; 3]) {
        let (centre, radius) = self.centre_and_radius();
        let a = TAU * t / period.max(1.0);
        let r = radius * 0.9;
        ([centre[0] + r * a.cos(), radius * 0.45, centre[1] + r * a.sin()], [centre[0], 0.0, centre[1]])
    }
}

/// The turn about +Y that takes +X onto the plan direction (x, z).
pub fn yaw_of(dir: [f32; 2]) -> f32 {
    (-dir[1]).atan2(dir[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    const TIMES: [f32; 4] = [0.0, 1.7, 60.0, 3600.0];

    #[test]
    fn the_loop_has_the_planned_counts() {
        let scene = Scene::new();
        let mut snap = Snapshot::default();
        scene.snapshot(0.0, &mut snap);
        assert_eq!(scene.conveyors.len(), 20);
        assert_eq!(snap.rollers.len(), 2000);
        assert_eq!(snap.pallets.len(), 200);
    }

    #[test]
    fn the_loop_closes() {
        let scene = Scene::new();
        let last = scene.conveyors[scene.conveyors.len() - 1];
        let end = [last.start[0] + last.dir[0] * last.length, last.start[1] + last.dir[1] * last.length];
        assert!(end[0].abs() < 1e-3 && end[1].abs() < 1e-3, "loop ends at {end:?}");
    }

    #[test]
    fn snapshots_are_deterministic() {
        let scene = Scene::new();
        for t in TIMES {
            let (mut a, mut b) = (Snapshot::default(), Snapshot::default());
            scene.snapshot(t, &mut a);
            Scene::new().snapshot(t, &mut b);
            assert_eq!(a, b, "t = {t}");
        }
    }

    #[test]
    fn pallets_never_overlap() {
        let scene = Scene::new();
        let gap = scene.perimeter() / PALLETS as f32;
        assert!(gap > PALLET_SIZE[0], "spacing {gap} m is shorter than a pallet");
    }

    #[test]
    fn pallets_move_at_the_conveying_speed() {
        let scene = Scene::new();
        let (mut a, mut b) = (Snapshot::default(), Snapshot::default());
        scene.snapshot(10.0, &mut a);
        scene.snapshot(11.0, &mut b);
        let (p, q) = (a.pallets[0].pos, b.pallets[0].pos);
        let d = ((q[0] - p[0]).powi(2) + (q[2] - p[2]).powi(2)).sqrt();
        assert!((d - SPEED).abs() < 1e-3, "moved {d} m in 1 s");
    }

    #[test]
    fn roller_surface_matches_the_conveying_speed() {
        let scene = Scene::new();
        let (mut a, mut b) = (Snapshot::default(), Snapshot::default());
        scene.snapshot(0.0, &mut a);
        scene.snapshot(0.01, &mut b);
        let surface = (b.rollers[0].spin - a.rollers[0].spin).abs() * ROLLER_RADIUS;
        assert!((surface - SPEED * 0.01).abs() < 1e-5);
    }

    #[test]
    fn yaw_turns_x_onto_the_direction() {
        for dir in [[1.0, 0.0], [0.0, 1.0], [-1.0, 0.0], [0.0, -1.0]] {
            let yaw = yaw_of(dir);
            let x = [yaw.cos(), -yaw.sin()];
            assert!((x[0] - dir[0]).abs() < 1e-6 && (x[1] - dir[1]).abs() < 1e-6, "{dir:?}");
        }
    }

    #[test]
    fn the_camera_sees_the_loop_from_above() {
        let scene = Scene::new();
        let (eye, target) = scene.camera(5.0, 30.0);
        assert!(eye[1] > 10.0 && target[1] == 0.0);
    }
}

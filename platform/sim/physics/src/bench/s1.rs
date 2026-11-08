//! The measurements of spike S1 (ADR-0002, ADR-0023), on any bench: each builds the same scene and returns what it
//! measures. The tests that judge them are [`s1_tests!`](crate::s1_tests), so every engine is held to the same
//! criteria with the same scenes (ADR-0026).

use super::{DT, Material, PhysicsBench};
use crate::mechanics::stopping_distance;

pub const WOOD_ON_STEEL: Material = Material { friction: 0.35, restitution: 0.1 };
pub const EUR: [f64; 3] = [1.2, 0.144, 0.8];
/// A EUR pallet with a 600 kg load.
pub const MASS: f64 = 631.4;

/// How far a pallet left on the floor for 2 s rests from it, m (its bottom's height).
pub fn rest_height<B: PhysicsBench + Default>() -> f64 {
    let mut b = B::default();
    b.floor(WOOD_ON_STEEL);
    let p = b.pallet(1.2, 0.0, EUR, MASS, 0.0, WOOD_ON_STEEL);
    b.run(2.0);
    b.position(p)[1] - EUR[1] / 2.0
}

/// How far a pallet sliding at 1 m/s on a floor of friction `mu` goes in 1 s, m.
pub fn slide<B: PhysicsBench + Default>(mu: f64) -> f64 {
    let m = Material { friction: mu, restitution: 0.0 };
    let mut b = B::default();
    b.floor(m);
    let p = b.pallet(1.2, 0.0, EUR, MASS, 1.0, m);
    let x0 = b.position(p)[0];
    b.run(1.0);
    b.position(p)[0] - x0
}

/// What μ·g says [`slide`] gives, m.
pub fn slide_expected(mu: f64) -> f64 {
    stopping_distance(1.0, mu)
}

/// P1: the speed of a pallet put at rest on rollers turning at 0.25 m/s, after 4 s, m/s.
pub fn p1_roller_speed<B: PhysicsBench + Default>() -> f64 {
    let mut b = B::default();
    b.floor(WOOD_ON_STEEL);
    rollers(&mut b, 0.25);
    let p = b.pallet(1.4, 0.6, EUR, MASS, 0.0, WOOD_ON_STEEL);
    b.run(4.0);
    b.velocity(p)[0]
}

/// P8: the speed of a pallet put at rest on two chain strips moving at 0.25 m/s after 4 s (m/s), and how far it floats
/// above them (m).
pub fn p8_chain<B: PhysicsBench + Default>() -> (f64, f64) {
    let chain = Material { friction: 0.25, restitution: 0.0 };
    let mut b = B::default();
    b.floor(WOOD_ON_STEEL);
    for z in [-0.3, 0.3] {
        b.strip((0.0, 6.0), 0.6, z, 0.04, 0.25, chain);
    }
    let p = b.pallet(1.4, 0.6, EUR, MASS, 0.0, WOOD_ON_STEEL);
    b.run(4.0);
    (b.velocity(p)[0], b.position(p)[1] - 0.6 - EUR[1] / 2.0)
}

/// What [`p3_stop`] measures.
#[derive(Clone, Copy, Debug)]
pub struct StopFigures {
    /// Where the pallet's front ends, from the stop's face, m.
    pub front_error: f64,
    /// How far it comes back from the furthest it got, m.
    pub rebound: f64,
    /// The time a step takes with 30 rollers and a pallet, ms.
    pub step_ms: f64,
}

/// P3: a pallet on rollers at 0.25 m/s runs into a stop at 4 m, for 12 s.
pub fn p3_stop<B: PhysicsBench + Default>() -> StopFigures {
    let mut b = B::default();
    b.floor(WOOD_ON_STEEL);
    rollers(&mut b, 0.25);
    b.stop(4.0, 0.75, Material { friction: 0.3, restitution: 0.1 });
    let p = b.pallet(2.0, 0.6, EUR, MASS, 0.25, WOOD_ON_STEEL);
    // The rebound is how far the front comes back from the furthest it has got so far, once it reaches the stop. (Measured
    // from the furthest it ever got, the last 10 mm of the approach counted as a rebound: ADR-0026.)
    let mut furthest = f64::NEG_INFINITY;
    let mut rebound = 0.0f64;
    let steps = (12.0 / DT) as usize;
    let t = std::time::Instant::now();
    for _ in 0..steps {
        b.step();
        let front = b.position(p)[0] + EUR[0] / 2.0;
        furthest = furthest.max(front);
        if furthest > 3.99 {
            rebound = rebound.max(furthest - front);
        }
    }
    let step_ms = t.elapsed().as_secs_f64() * 1000.0 / steps as f64;
    let front = b.position(p)[0] + EUR[0] / 2.0;
    StopFigures { front_error: front - 4.0, rebound, step_ms }
}

/// P7: a pallet's position and velocity after 3 s on rollers, to compare two runs of the same bench.
pub fn p7_run<B: PhysicsBench + Default>() -> ([f64; 3], [f64; 3]) {
    let mut b = B::default();
    b.floor(WOOD_ON_STEEL);
    rollers(&mut b, 0.25);
    let p = b.pallet(1.4, 0.6, EUR, MASS, 0.0, WOOD_ON_STEEL);
    b.run(3.0);
    (b.position(p), b.velocity(p))
}

/// P2: the acceleration of a pallet at rest put on rollers already turning at 1 m/s, the pallet's and the rollers'
/// materials as the library's pair table gives them: while it slips, μ·g (m/s²). NaN if the table lacks the pair.
pub fn traction<B: PhysicsBench + Default>(pallet: &str, roller: &str) -> f64 {
    let Some(m) = pair(pallet, roller) else { return f64::NAN };
    let mut b = B::default();
    b.floor(m);
    for k in 0..30 {
        b.roller(0.1 + 0.2063 * f64::from(k), 0.6, 0.0445, 1.0, 1.0, m);
    }
    let p = b.pallet(1.4, 0.6, EUR, MASS, 0.0, m);
    b.run(0.05);
    let (t1, v1) = (b.time(), b.velocity(p)[0]);
    b.run(0.1);
    (b.velocity(p)[0] - v1) / (b.time() - t1)
}

/// P5: a wooden pallet sliding without friction at 0.25 m/s hits a stop of `stop`'s material: its speed back over its
/// speed in. NaN if the table lacks the pair.
pub fn restitution<B: PhysicsBench + Default>(stop: &str) -> f64 {
    let Some(m) = pair("wood", stop) else { return f64::NAN };
    let slide = Material { friction: 0.0, restitution: 0.0 };
    let mut b = B::default();
    b.floor(slide);
    b.stop(3.0, 0.141, m);
    let p = b.pallet(2.0, 0.0, EUR, MASS, 0.25, Material { friction: 0.0, restitution: m.restitution });
    let mut back: f64 = 0.0;
    for _ in 0..(8.0 / DT) as usize {
        b.step();
        back = back.max(-b.velocity(p)[0]);
    }
    back / 0.25
}

/// The figures of spike S1 for a decision record, one per line.
pub fn figures<B: PhysicsBench + Default>() -> String {
    let (chain, float) = p8_chain::<B>();
    let stop = p3_stop::<B>();
    let (a, b) = (p7_run::<B>(), p7_run::<B>());
    [
        format!("P1 roller speed: {:.4} m/s", p1_roller_speed::<B>()),
        format!("P8 chain speed: {chain:.4} m/s, height error {float:.5} m"),
        format!("P3 front error: {:.5} m, rebound {:.5} m; {:.4} ms a step with 30 rollers", stop.front_error, stop.rebound, stop.step_ms),
        format!("P7 same run twice: {}", if a == b { "identical" } else { "different" }),
        format!("friction 0.5 at 1 m/s: slid {:.4} m (μ·g says {:.4})", slide::<B>(0.5), slide_expected(0.5)),
        format!(
            "P2 traction: wood on steel-zinc {:.3}, plastic on steel-zinc {:.3}, wood on rubber {:.3} m/s²",
            traction::<B>("wood", "steel-zinc"),
            traction::<B>("plastic", "steel-zinc"),
            traction::<B>("wood", "rubber")
        ),
        format!(
            "P5 restitution: steel-painted {:.3}, polyurethane {:.3}, rubber {:.3}",
            restitution::<B>("steel-painted"),
            restitution::<B>("polyurethane"),
            restitution::<B>("rubber")
        ),
    ]
    .join("\n")
}

/// Rollers Ø 89 mm at 206.3 mm pitch, top at 0.6 m, along 6 m.
fn rollers<B: PhysicsBench>(b: &mut B, speed: f64) {
    for k in 0..30 {
        b.roller(0.1 + 0.2063 * f64::from(k), 0.6, 0.0445, 1.0, speed, Material { friction: 0.35, restitution: 0.0 });
    }
}

/// The friction and restitution of a contact pair of the library (S16): both parts get the pair's values, so the contact
/// takes them as they are.
fn pair(a: &str, b: &str) -> Option<Material> {
    let lib = intracraft_library::Library::built_in().ok()?;
    let p = lib.reference.contact_pair(a, b)?;
    Some(Material { friction: p.mu, restitution: p.restitution })
}

/// The tests of spike S1 on a bench type: `intracraft_physics::s1_tests!(MyBench);` in a test module.
#[macro_export]
macro_rules! s1_tests {
    ($bench:ty) => {
        #[test]
        fn a_pallet_rests_on_the_floor() {
            let y = $crate::bench::s1::rest_height::<$bench>();
            assert!(y.abs() < 0.002, "its bottom is {y} m off the floor");
        }

        #[test]
        fn friction_stops_a_sliding_pallet_where_mu_g_says() {
            // μ = 0.5 stops a pallet sliding at 1 m/s in 0.102 m (the reference's backend test).
            let slid = $crate::bench::s1::slide::<$bench>(0.5);
            let expected = $crate::bench::s1::slide_expected(0.5);
            assert!((slid - expected).abs() / expected < 0.05, "slid {slid} m, expected {expected} m");
        }

        #[test]
        fn p1_rollers_carry_a_pallet_at_their_surface_speed() {
            let v = $crate::bench::s1::p1_roller_speed::<$bench>();
            assert!((v - 0.25).abs() / 0.25 < 0.03, "{v} m/s");
        }

        #[test]
        fn p8_a_chain_strip_carries_a_pallet_by_its_moving_surface() {
            let (v, y) = $crate::bench::s1::p8_chain::<$bench>();
            assert!((v - 0.25).abs() / 0.25 < 0.03, "{v} m/s");
            assert!(y.abs() < 0.001, "the pallet floats {y} m above the chains");
        }

        #[test]
        fn p3_a_pallet_stops_against_a_stop_within_5_mm_and_rebounds_under_10_mm() {
            let s = $crate::bench::s1::p3_stop::<$bench>();
            assert!(s.front_error.abs() <= 0.005, "front {} m from the stop", s.front_error);
            assert!(s.rebound < 0.010, "rebound {} m", s.rebound);
        }

        #[test]
        fn p7_the_same_bench_gives_the_same_run() {
            assert_eq!($crate::bench::s1::p7_run::<$bench>(), $crate::bench::s1::p7_run::<$bench>());
        }

        #[test]
        fn p2_a_pallet_slipping_on_rollers_accelerates_at_mu_g_for_each_material() {
            for (pallet, roller, expected) in [("wood", "steel-zinc", 3.4335), ("plastic", "steel-zinc", 2.4525)] {
                let a = $crate::bench::s1::traction::<$bench>(pallet, roller);
                assert!((a - expected).abs() / expected < 0.10, "{pallet} on {roller}: {a} m/s², expected {expected}");
            }
        }

        #[test]
        fn p5_a_pallet_bounces_off_a_stop_as_its_material_pair_says() {
            for (stop, expected) in [("steel-painted", 0.1), ("polyurethane", 0.2), ("rubber", 0.35)] {
                let e = $crate::bench::s1::restitution::<$bench>(stop);
                assert!((e - expected).abs() / expected < 0.10, "wood on {stop}: {e}, expected {expected}");
            }
        }

        /// The figures of spike S1, for the decision record: `cargo test --release -- --ignored --nocapture s1_figures`.
        #[test]
        #[ignore]
        fn s1_figures() {
            println!("{}", $crate::bench::s1::figures::<$bench>());
        }
    };
}

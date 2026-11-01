//! Analytic kinematics of the 2D run (T04), in SI. The same formulas move a pallet (m, m/s, m/s²) and turn a table
//! (rad, rad/s, rad/s²).

/// A move from rest: accelerate to `max_speed`, cruise, and brake to rest with `decel` (none: arrive at cruise speed).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MoveProfile {
    pub distance: f64,
    pub max_speed: f64,
    pub accel: f64,
    pub decel: Option<f64>,
}

/// A piece of motion at constant acceleration: s(t) = s0 + v0·(t − t0) + ½·a·(t − t0)².
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Segment {
    pub t0: f64,
    pub s0: f64,
    pub v0: f64,
    pub a: f64,
}

fn check(p: &MoveProfile) -> Result<(), String> {
    let not_positive = |x: f64| x.is_nan() || x <= 0.0;
    if not_positive(p.max_speed) {
        return Err(format!("maxSpeed must be > 0, got {}", p.max_speed));
    }
    if not_positive(p.accel) {
        return Err(format!("accel must be > 0, got {}", p.accel));
    }
    if p.decel.is_some_and(not_positive) {
        return Err("decel must be > 0".into());
    }
    if p.distance.is_nan() || p.distance < 0.0 {
        return Err(format!("distance must be ≥ 0, got {}", p.distance));
    }
    Ok(())
}

/// How long the move takes; a triangular profile when the distance is too short to reach cruise speed.
pub fn profile_time(p: &MoveProfile) -> Result<f64, String> {
    check(p)?;
    if p.distance == 0.0 {
        return Ok(0.0);
    }
    let v = p.max_speed;
    let accel_distance = v * v / (2.0 * p.accel);
    let decel_distance = p.decel.map_or(0.0, |d| v * v / (2.0 * d));
    if p.distance >= accel_distance + decel_distance {
        let cruise = (p.distance - accel_distance - decel_distance) / v;
        return Ok(v / p.accel + cruise + p.decel.map_or(0.0, |d| v / d));
    }
    Ok(match p.decel {
        None => (2.0 * p.distance / p.accel).sqrt(),
        Some(d) => {
            let peak = (2.0 * p.distance * p.accel * d / (p.accel + d)).sqrt();
            peak / p.accel + peak / d
        }
    })
}

/// How far the move has gone `t` s after its start.
pub fn profile_position_at(p: &MoveProfile, t: f64) -> Result<f64, String> {
    let total = profile_time(p)?;
    if t <= 0.0 {
        return Ok(0.0);
    }
    if t >= total {
        return Ok(p.distance);
    }
    let v = p.max_speed;
    let accel_distance = v * v / (2.0 * p.accel);
    let decel_distance = p.decel.map_or(0.0, |d| v * v / (2.0 * d));
    let peak = if p.distance >= accel_distance + decel_distance {
        v
    } else {
        match p.decel {
            None => (2.0 * p.distance * p.accel).sqrt(),
            Some(d) => (2.0 * p.distance * p.accel * d / (p.accel + d)).sqrt(),
        }
    };
    let accel_time = peak / p.accel;
    if t <= accel_time {
        return Ok(0.5 * p.accel * t * t);
    }
    let brake_time = p.decel.map_or(0.0, |d| peak / d);
    match p.decel {
        Some(d) if t > total - brake_time => {
            let remaining = total - t;
            Ok(p.distance - 0.5 * d * remaining * remaining)
        }
        _ => Ok(peak * peak / (2.0 * p.accel) + peak * (t - accel_time)),
    }
}

/// A transfer table's cycle: lift, the chain move (accelerate, cruise, brake), lower, and settle (T4).
pub fn transfer_cycle_time(lift: f64, lower: f64, distance: f64, speed: f64, accel: f64, settle: f64) -> Result<f64, String> {
    let m = profile_time(&MoveProfile { distance, max_speed: speed, accel, decel: Some(accel) })?;
    Ok(lift + m + lower + settle)
}

/// Steady throughput of a line limited by spacing, pallets/s: v / (L + gap) (T2).
pub fn max_throughput(speed: f64, pallet_length: f64, gap: f64) -> f64 {
    speed / (pallet_length + gap)
}

/// Pallets that fit on a conveyor: ⌊length / (L + gap)⌋.
pub fn pallet_capacity(length: f64, pallet_length: f64, gap: f64) -> f64 {
    (length / (pallet_length + gap)).floor()
}

/// Kinetic energy at impact, J: ½·m·v² (T5).
pub fn impact_energy(mass: f64, speed: f64) -> f64 {
    0.5 * mass * speed * speed
}

pub fn position_at(s: &Segment, t: f64) -> f64 {
    let dt = t - s.t0;
    s.s0 + s.v0 * dt + 0.5 * s.a * dt * dt
}

pub fn velocity_at(s: &Segment, t: f64) -> f64 {
    s.v0 + s.a * (t - s.t0)
}

/// The earliest time ≥ t0 the segment reaches position `target`, if it ever does: how the engine predicts "the front
/// reaches the sensor".
pub fn time_to_reach(s: &Segment, target: f64) -> Option<f64> {
    let ds = target - s.s0;
    if ds == 0.0 {
        return Some(s.t0);
    }
    if s.a == 0.0 {
        if s.v0 == 0.0 {
            return None;
        }
        let dt = ds / s.v0;
        return (dt >= 0.0).then_some(s.t0 + dt);
    }
    // ½·a·dt² + v0·dt − ds = 0
    let disc = s.v0 * s.v0 + 2.0 * s.a * ds;
    if disc < 0.0 {
        return None;
    }
    let root = disc.sqrt();
    [(-s.v0 + root) / s.a, (-s.v0 - root) / s.a].into_iter().filter(|dt| *dt >= 0.0).reduce(f64::min).map(|dt| s.t0 + dt)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn within(actual: f64, expected: f64) {
        assert!(((actual - expected) / expected).abs() < 0.01, "{actual} is not within 1 % of {expected}");
    }

    #[test]
    fn t1_three_metres_from_rest_at_0_25_m_s_without_braking_takes_12_25_s() {
        within(profile_time(&MoveProfile { distance: 3.0, max_speed: 0.25, accel: 0.5, decel: None }).unwrap(), 12.25);
    }

    #[test]
    fn t2_a_saturated_eur_line_carries_about_667_pallets_an_hour() {
        within(max_throughput(0.25, 1.2, 0.15) * 3600.0, 666.7);
    }

    #[test]
    fn t3_a_quarter_turn_at_15_deg_s_with_30_deg_s2_takes_6_5_s() {
        let r = |d: f64| d.to_radians();
        within(profile_time(&MoveProfile { distance: r(90.0), max_speed: r(15.0), accel: r(30.0), decel: Some(r(30.0)) }).unwrap(), 6.5);
    }

    #[test]
    fn t4_a_transfer_cycle_takes_7_4_s() {
        within(transfer_cycle_time(1.0, 1.0, 1.0, 0.2, 0.5, 0.0).unwrap(), 7.4);
    }

    #[test]
    fn t5_1025_kg_at_0_25_m_s_hits_with_32_j() {
        within(impact_energy(1025.0, 0.25), 32.03);
    }

    #[test]
    fn short_moves_take_a_triangular_profile_and_zero_takes_no_time() {
        let t = profile_time(&MoveProfile { distance: 0.5, max_speed: 1.0, accel: 1.0, decel: Some(1.0) }).unwrap();
        assert!((t - 2.0 * 0.5f64.sqrt()).abs() < 1e-12);
        assert_eq!(profile_time(&MoveProfile { distance: 0.0, max_speed: 1.0, accel: 1.0, decel: None }), Ok(0.0));
        assert!(profile_time(&MoveProfile { distance: 1.0, max_speed: 0.0, accel: 1.0, decel: None }).is_err());
        assert!(profile_time(&MoveProfile { distance: f64::NAN, max_speed: 1.0, accel: 1.0, decel: None }).is_err());
    }

    #[test]
    fn the_position_follows_the_profile_to_its_end() {
        let p = MoveProfile { distance: 3.0, max_speed: 0.25, accel: 0.5, decel: Some(0.5) };
        let total = profile_time(&p).unwrap();
        assert_eq!(profile_position_at(&p, 0.0), Ok(0.0));
        assert_eq!(profile_position_at(&p, total), Ok(3.0));
        assert!((profile_position_at(&p, 0.5).unwrap() - 0.0625).abs() < 1e-12);
        let mut last = 0.0;
        for i in 0..=100 {
            let s = profile_position_at(&p, total * f64::from(i) / 100.0).unwrap();
            assert!(s >= last - 1e-12);
            last = s;
        }
        // Halfway in time is halfway in space, by symmetry.
        assert!((profile_position_at(&p, total / 2.0).unwrap() - 1.5).abs() < 1e-9);
    }

    #[test]
    fn predicts_when_a_segment_reaches_a_point() {
        let cruise = Segment { t0: 2.0, s0: 1.0, v0: 0.25, a: 0.0 };
        assert_eq!(time_to_reach(&cruise, 2.0), Some(6.0));
        assert_eq!(time_to_reach(&cruise, 0.0), None);
        let start = Segment { t0: 0.0, s0: 0.0, v0: 0.0, a: 0.5 };
        assert!((time_to_reach(&start, 0.0625).unwrap() - 0.5).abs() < 1e-12);
        let braking = Segment { t0: 0.0, s0: 0.0, v0: 0.25, a: -0.5 };
        assert_eq!(time_to_reach(&braking, 1.0), None, "it stops after 62.5 mm");
        assert!((position_at(&braking, 0.5) - 0.0625).abs() < 1e-12);
        assert!(velocity_at(&braking, 0.5).abs() < 1e-12);
        assert_eq!(pallet_capacity(3.0, 1.2, 0.15), 2.0);
    }
}

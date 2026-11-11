//! The wrap programme of a stretch wrapper (S25 §2): from a pallet's load height, how many turns the table makes and how
//! long the wrap takes, and where the table stands during it. The 2D run and the 3D run use the same programme.

use std::f64::consts::TAU;

use crate::kinematics::{MoveProfile, profile_position_at, profile_time};

/// An angle this close to a whole turn is the line again, rad.
const WHOLE_TURN_EPS: f64 = 1e-9;

/// The wrap programme's settings (S25 §1), in SI: speeds in rad/s and rad/s², lengths in m, times in s.
#[derive(Clone, Debug, PartialEq)]
pub struct WrapData {
    pub turn_speed: f64,
    pub turn_accel: f64,
    pub bottom_wraps: f64,
    pub top_wraps: f64,
    pub film_width: f64,
    pub overlap: f64,
    /// The carriage wraps going up and coming down; else going up only.
    pub up_and_down: bool,
    pub attach_time: f64,
    pub cut_time: f64,
    pub max_load_height: f64,
}

/// What the programme does with one pallet.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Programme {
    /// Whole turns of the table; 0 for a pallet without a load, which is not wrapped.
    pub turns: u32,
    pub turning_time: f64,
    /// Attach, turn and cut, s.
    pub processing_time: f64,
    /// The load is higher than the carriage reaches.
    pub too_tall: bool,
}

impl WrapData {
    /// How far the film carriage moves in one turn, m.
    pub fn advance_per_turn(&self) -> f64 {
        self.film_width * (1.0 - self.overlap)
    }

    /// The programme for a load `load_height` m high.
    pub fn programme(&self, load_height: f64) -> Programme {
        if load_height <= 0.0 {
            return Programme::default();
        }
        let height = load_height.min(self.max_load_height);
        let advance = self.advance_per_turn();
        let climbing = if advance > 0.0 { ((height - self.film_width).max(0.0) / advance - WHOLE_TURN_EPS).ceil().max(0.0) } else { 0.0 };
        let passes = if self.up_and_down { 2.0 } else { 1.0 };
        let turns = (self.bottom_wraps.round().max(0.0) + self.top_wraps.round().max(0.0) + climbing * passes).max(1.0) as u32;
        let turning_time = profile_time(&self.profile(turns)).unwrap_or(0.0);
        Programme {
            turns,
            turning_time,
            processing_time: self.attach_time + turning_time + self.cut_time,
            too_tall: load_height > self.max_load_height + WHOLE_TURN_EPS,
        }
    }

    /// The table's angle `t` s after the processing started (rad, 0 to a whole turn): still while the film is
    /// attached, along the turning profile, and back in line once the turns are done.
    pub fn angle_at(&self, programme: &Programme, t: f64) -> f64 {
        if programme.turns == 0 {
            return 0.0;
        }
        let covered = profile_position_at(&self.profile(programme.turns), t - self.attach_time).unwrap_or(0.0);
        let angle = covered.rem_euclid(TAU);
        if TAU - angle < WHOLE_TURN_EPS { 0.0 } else { angle }
    }

    /// The table's turning speed `t` s after the processing started, rad/s.
    pub fn angular_speed_at(&self, programme: &Programme, t: f64, dt: f64) -> f64 {
        if programme.turns == 0 {
            return 0.0;
        }
        let profile = self.profile(programme.turns);
        let at = |t: f64| profile_position_at(&profile, t - self.attach_time).unwrap_or(0.0);
        (at(t + dt) - at(t)) / dt
    }

    fn profile(&self, turns: u32) -> MoveProfile {
        MoveProfile { distance: turns as f64 * TAU, max_speed: self.turn_speed, accel: self.turn_accel, decel: Some(self.turn_accel) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The defaults of S25: 12 rpm, 36 °/s², 2 + 2 reinforcing turns, 500 mm of film at 50 %, up and down, 3 s and 4 s.
    fn defaults() -> WrapData {
        WrapData {
            turn_speed: 12.0 * TAU / 60.0,
            turn_accel: 36f64.to_radians(),
            bottom_wraps: 2.0,
            top_wraps: 2.0,
            film_width: 0.5,
            overlap: 0.5,
            up_and_down: true,
            attach_time: 3.0,
            cut_time: 4.0,
            max_load_height: 2.0,
        }
    }

    #[test]
    fn w1_a_load_of_1_2_m_takes_10_turns_in_52_s_and_59_s_in_all() {
        let p = defaults().programme(1.2);
        assert_eq!(p.turns, 10);
        // 3600° at 72 °/s, plus 72 / 36 s lost to the two ramps.
        assert!((p.turning_time - 52.0).abs() < 1e-9, "{}", p.turning_time);
        assert!((p.processing_time - 59.0).abs() < 1e-9);
        assert!(!p.too_tall);
    }

    #[test]
    fn w3_up_only_takes_7_turns_and_44_s_and_no_load_is_not_wrapped() {
        let w = WrapData { up_and_down: false, ..defaults() };
        let p = w.programme(1.2);
        assert_eq!(p.turns, 7);
        assert!((p.processing_time - 44.0).abs() < 1e-9);
        assert_eq!(defaults().programme(0.0), Programme::default());
    }

    #[test]
    fn a_load_no_higher_than_the_film_needs_only_the_reinforcing_turns_and_at_least_one() {
        assert_eq!(defaults().programme(0.4).turns, 4);
        // An exact number of advances does not add a turn: (1.0 − 0.5) / 0.25 = 2.
        assert_eq!(defaults().programme(1.0).turns, 8);
        let bare = WrapData { bottom_wraps: 0.0, top_wraps: 0.0, ..defaults() };
        assert_eq!(bare.programme(0.3).turns, 1);
    }

    #[test]
    fn a_load_above_the_mast_is_wrapped_to_the_mast_and_flagged() {
        let p = defaults().programme(2.4);
        assert!(p.too_tall);
        assert_eq!(p.turns, defaults().programme(2.0).turns);
    }

    #[test]
    fn the_table_stands_still_while_the_film_is_attached_turns_and_ends_in_line() {
        let w = defaults();
        let p = w.programme(1.2);
        assert_eq!(w.angle_at(&p, 2.0), 0.0);
        // Halfway through the turning: 5 whole turns, so back at 0 up to the profile's symmetry.
        let mid = w.angle_at(&p, 3.0 + 26.0);
        assert!(mid < 1e-6 || TAU - mid < 1e-6, "{mid}");
        assert!(w.angle_at(&p, 3.0 + 10.0) > 0.0);
        assert_eq!(w.angle_at(&p, 3.0 + 52.0), 0.0);
        assert_eq!(w.angle_at(&p, 59.0), 0.0);
        assert!((w.angular_speed_at(&p, 3.0 + 26.0, 1e-3) - w.turn_speed).abs() < 1e-6);
    }
}

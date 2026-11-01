//! Simulation time: whole microseconds, so a long run never drifts (T03).

/// Microseconds since the start of a run.
pub type SimTime = i64;

pub const US_PER_S: i64 = 1_000_000;
pub const US_PER_MS: i64 = 1_000;

/// Seconds to microseconds, rounded half up as the reference's `Math.round` does.
pub fn from_seconds(seconds: f64) -> SimTime {
    round(seconds * US_PER_S as f64)
}

pub fn to_seconds(time: SimTime) -> f64 {
    time as f64 / US_PER_S as f64
}

pub fn from_ms(ms: f64) -> SimTime {
    round(ms * US_PER_MS as f64)
}

/// JavaScript's `Math.round`: halves toward +∞; a value out of range saturates.
fn round(x: f64) -> SimTime {
    (x + 0.5).floor() as SimTime
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_and_rounds_as_the_reference() {
        assert_eq!(from_seconds(12.25), 12_250_000);
        assert_eq!(from_seconds(0.0000005), 1);
        assert_eq!(from_seconds(-0.0000005), 0);
        assert_eq!(from_ms(10.0), 10_000);
        assert_eq!(to_seconds(6_500_000), 6.5);
    }
}

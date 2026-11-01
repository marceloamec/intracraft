//! Closed-form mechanics (T09–T12), a port of the reference's `mechanics.ts`: what the physics tests compare with.

/// Standard gravity, m/s².
pub const GRAVITY: f64 = 9.81;

/// A roller's angular speed for a surface speed `v` (m/s) and radius `r` (m): ω = v / r, rad/s.
pub fn roller_angular_speed(v: f64, r: f64) -> f64 {
    if r > 0.0 { v / r } else { 0.0 }
}

/// The deceleration of a body sliding on a level surface with friction μ: μ·g, m/s².
pub fn friction_decel(mu: f64) -> f64 {
    mu * GRAVITY
}

/// How far a body sliding at `v` on a level surface with friction μ goes before it stops: v² / (2·μ·g), m.
pub fn stopping_distance(v: f64, mu: f64) -> f64 {
    let a = friction_decel(mu);
    if a > 0.0 { v * v / (2.0 * a) } else { f64::INFINITY }
}

/// Kinetic energy at impact, J: ½·m·v².
pub fn impact_energy(mass: f64, v: f64) -> f64 {
    0.5 * mass * v * v
}

/// The principal moments of inertia of a solid box of mass `m` and sizes (x, y, z), kg·m².
pub fn box_inertia(m: f64, size: [f64; 3]) -> [f64; 3] {
    let [x, y, z] = size;
    [m * (y * y + z * z) / 12.0, m * (x * x + z * z) / 12.0, m * (x * x + y * y) / 12.0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_closed_forms() {
        assert!((roller_angular_speed(0.25, 0.0445) - 5.617977).abs() < 1e-6);
        assert!((stopping_distance(1.0, 0.5) - 0.101937).abs() < 1e-6);
        assert!((impact_energy(1025.0, 0.25) - 32.03125).abs() < 1e-9);
        let i = box_inertia(12.0, [1.0, 2.0, 3.0]);
        assert_eq!(i, [13.0, 10.0, 5.0]);
        assert_eq!(roller_angular_speed(1.0, 0.0), 0.0);
        assert!(stopping_distance(1.0, 0.0).is_infinite());
    }
}

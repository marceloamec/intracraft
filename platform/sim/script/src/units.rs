//! Physical units. A quantity is a number in SI units with its dimension (length, mass, time, angle); the unit names
//! (`mm`, `s`, `kg`, `deg`…) are built-in quantities of size one unit, so `150 * mm` is 0.15 m and `gap / mm` is the
//! plain number 150. Adding a length to a time is an error, not a wrong number.

use std::f64::consts::PI;

/// Exponents of length, mass, time and angle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Dim(pub [i8; 4]);

impl Dim {
    pub const NONE: Dim = Dim([0, 0, 0, 0]);
    pub const LENGTH: Dim = Dim([1, 0, 0, 0]);
    pub const MASS: Dim = Dim([0, 1, 0, 0]);
    pub const TIME: Dim = Dim([0, 0, 1, 0]);
    pub const ANGLE: Dim = Dim([0, 0, 0, 1]);
    pub const SPEED: Dim = Dim([1, 0, -1, 0]);
    pub const ACCELERATION: Dim = Dim([1, 0, -2, 0]);
    pub const ANGULAR_SPEED: Dim = Dim([0, 0, -1, 1]);
    pub const ANGULAR_ACCELERATION: Dim = Dim([0, 0, -2, 1]);
    pub const RATE: Dim = Dim([0, 0, -1, 0]);

    pub fn is_none(self) -> bool {
        self == Dim::NONE
    }

    pub fn times(self, o: Dim) -> Option<Dim> {
        let mut d = [0i8; 4];
        for (i, slot) in d.iter_mut().enumerate() {
            *slot = self.0[i].checked_add(o.0[i])?;
        }
        Some(Dim(d))
    }

    pub fn per(self, o: Dim) -> Option<Dim> {
        let mut d = [0i8; 4];
        for (i, slot) in d.iter_mut().enumerate() {
            *slot = self.0[i].checked_sub(o.0[i])?;
        }
        Some(Dim(d))
    }

    pub fn pow(self, n: i64) -> Option<Dim> {
        let n = i8::try_from(n).ok()?;
        let mut d = [0i8; 4];
        for (i, slot) in d.iter_mut().enumerate() {
            *slot = self.0[i].checked_mul(n)?;
        }
        Some(Dim(d))
    }

    /// The square root's dimension, when every exponent is even.
    pub fn sqrt(self) -> Option<Dim> {
        if self.0.iter().any(|e| e % 2 != 0) {
            return None;
        }
        Some(Dim(self.0.map(|e| e / 2)))
    }

    /// The name of the quantity, as messages say it.
    pub fn name(self) -> &'static str {
        match self {
            Dim::NONE => "number",
            Dim::LENGTH => "length",
            Dim::MASS => "mass",
            Dim::TIME => "time",
            Dim::ANGLE => "angle",
            Dim::SPEED => "speed",
            Dim::ACCELERATION => "acceleration",
            Dim::ANGULAR_SPEED => "angular speed",
            Dim::ANGULAR_ACCELERATION => "angular acceleration",
            Dim::RATE => "rate",
            _ => "quantity",
        }
    }
}

/// The built-in unit names: name, size in SI, dimension.
pub const UNITS: [(&str, f64, Dim); 16] = [
    ("mm", 0.001, Dim::LENGTH),
    ("cm", 0.01, Dim::LENGTH),
    ("m", 1.0, Dim::LENGTH),
    ("km", 1000.0, Dim::LENGTH),
    ("ms", 0.001, Dim::TIME),
    ("s", 1.0, Dim::TIME),
    ("minute", 60.0, Dim::TIME),
    ("h", 3600.0, Dim::TIME),
    ("day", 86_400.0, Dim::TIME),
    ("g", 0.001, Dim::MASS),
    ("kg", 1.0, Dim::MASS),
    ("tonne", 1000.0, Dim::MASS),
    ("deg", PI / 180.0, Dim::ANGLE),
    ("rad", 1.0, Dim::ANGLE),
    ("rpm", 2.0 * PI / 60.0, Dim::ANGULAR_SPEED),
    ("percent", 0.01, Dim::NONE),
];

/// The unit called `name`, if it is one.
pub fn unit(name: &str) -> Option<(f64, Dim)> {
    UNITS.iter().find(|(n, ..)| *n == name).map(|(_, f, d)| (*f, *d))
}

/// The dimension of a library parameter's SI unit (`m`, `s`, `m/s`, `m/s²`, `kg`, `rad`, `rad/s`…); `None` for no unit.
pub fn dim_of_library_unit(unit: &str) -> Option<Dim> {
    Some(match unit.trim() {
        "m" => Dim::LENGTH,
        "s" => Dim::TIME,
        "kg" => Dim::MASS,
        "rad" => Dim::ANGLE,
        "m/s" => Dim::SPEED,
        "m/s²" | "m/s2" | "m/s^2" => Dim::ACCELERATION,
        "rad/s" => Dim::ANGULAR_SPEED,
        "rad/s²" | "rad/s2" | "rad/s^2" => Dim::ANGULAR_ACCELERATION,
        "1/s" | "/s" => Dim::RATE,
        _ => return None,
    })
}

/// How a quantity reads: lengths in mm, times in s, speeds in m/s, angles in degrees, rates per hour, as IntraCraft
/// shows them; anything else in SI with its exponents.
pub fn display_quantity(x: f64, dim: Dim) -> String {
    let (v, unit) = display_parts(x, dim);
    format!("{} {unit}", number_text(v))
}

/// The number and the unit a quantity is shown in.
pub fn display_parts(x: f64, dim: Dim) -> (f64, String) {
    match dim {
        Dim::LENGTH => (x * 1000.0, "mm".into()),
        Dim::TIME => (x, "s".into()),
        Dim::MASS => (x, "kg".into()),
        Dim::ANGLE => (x.to_degrees(), "deg".into()),
        Dim::SPEED => (x, "m/s".into()),
        Dim::ACCELERATION => (x, "m/s²".into()),
        Dim::ANGULAR_SPEED => (x.to_degrees(), "deg/s".into()),
        Dim::ANGULAR_ACCELERATION => (x.to_degrees(), "deg/s²".into()),
        Dim::RATE => (x * 3600.0, "/h".into()),
        Dim(e) => {
            let names = ["m", "kg", "s", "rad"];
            let parts: Vec<String> = e
                .iter()
                .zip(names)
                .filter(|(p, _)| **p != 0)
                .map(|(p, n)| if *p == 1 { n.to_string() } else { format!("{n}^{p}") })
                .collect();
            (x, parts.join("·"))
        }
    }
}

/// A number for people: whole numbers without decimals, others with up to six, trailing zeros dropped.
pub fn number_text(v: f64) -> String {
    if !v.is_finite() {
        return crate::value::float_text(v);
    }
    let rounded = (v * 1e6).round() / 1e6;
    if rounded.fract() == 0.0 && rounded.abs() < 1e15 {
        return format!("{}", rounded as i64);
    }
    let s = format!("{rounded:.6}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dimensions_combine_as_physics_says() {
        assert_eq!(Dim::LENGTH.per(Dim::TIME), Some(Dim::SPEED));
        assert_eq!(Dim::SPEED.per(Dim::TIME), Some(Dim::ACCELERATION));
        assert_eq!(Dim::LENGTH.times(Dim::LENGTH).and_then(Dim::sqrt), Some(Dim::LENGTH));
        assert_eq!(Dim::LENGTH.sqrt(), None);
        assert_eq!(dim_of_library_unit("m/s²"), Some(Dim::ACCELERATION));
    }

    #[test]
    fn quantities_read_in_the_units_intracraft_shows() {
        assert_eq!(display_quantity(0.15, Dim::LENGTH), "150 mm");
        assert_eq!(display_quantity(0.25, Dim::SPEED), "0.25 m/s");
        assert_eq!(display_quantity(PI / 2.0, Dim::ANGLE), "90 deg");
        assert_eq!(display_quantity(1.0 / 30.0, Dim::RATE), "120 /h");
        assert_eq!(display_quantity(2.0, Dim([2, 0, 0, 0])), "2 m^2");
    }
}

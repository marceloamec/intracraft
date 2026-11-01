//! Deck frames: where a conveying surface lies in the plant (the reference's `frame.ts`). Local x runs along the deck from
//! its x = 0 end, y up from the conveying surface and z across, as in the instance frame (A04); a gravity line's deck also
//! falls along x with its slope.

/// A point or a direction, m (SI, y up).
pub type V3 = [f64; 3];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    /// The world point of local (0, 0, 0): the deck's x = 0 end, on the conveying surface, on its axis.
    pub origin: V3,
    /// Turn about +y, rad, as the instance's yaw (right-handed: +x turns toward −z).
    pub yaw: f64,
    /// Fall per metre along local x; 0 for a level deck.
    pub slope: f64,
}

impl Frame {
    /// A local direction in the world: tilted down along x by the slope, then turned by the yaw.
    pub fn rotate(&self, v: V3) -> V3 {
        let (sa, ca) = (-self.slope.atan()).sin_cos();
        let (x, y, z) = (v[0] * ca - v[1] * sa, v[0] * sa + v[1] * ca, v[2]);
        let (s, c) = self.yaw.sin_cos();
        [x * c + z * s, y, -x * s + z * c]
    }

    fn unrotate(&self, v: V3) -> V3 {
        let (s, c) = self.yaw.sin_cos();
        let (x, y, z) = (v[0] * c - v[2] * s, v[1], v[0] * s + v[2] * c);
        let (sa, ca) = self.slope.atan().sin_cos();
        [x * ca - y * sa, x * sa + y * ca, z]
    }

    pub fn to_world(&self, local: V3) -> V3 {
        let r = self.rotate(local);
        [self.origin[0] + r[0], self.origin[1] + r[1], self.origin[2] + r[2]]
    }

    pub fn to_local(&self, world: V3) -> V3 {
        self.unrotate([world[0] - self.origin[0], world[1] - self.origin[1], world[2] - self.origin[2]])
    }
}

pub fn dot(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn scale(v: V3, k: f64) -> V3 {
    v.map(|c| c * k)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: V3, b: V3) {
        assert!(a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-12), "{a:?} vs {b:?}");
    }

    #[test]
    fn a_turned_deck_runs_its_x_toward_minus_z_and_back() {
        let f = Frame { origin: [1.0, 0.6, 2.0], yaw: std::f64::consts::FRAC_PI_2, slope: 0.0 };
        close(f.rotate([1.0, 0.0, 0.0]), [0.0, 0.0, -1.0]);
        close(f.to_world([3.0, 0.0, 0.0]), [1.0, 0.6, -1.0]);
        close(f.to_local(f.to_world([0.4, 0.1, -0.3])), [0.4, 0.1, -0.3]);
    }

    #[test]
    fn a_sloping_deck_falls_along_its_x() {
        let f = Frame { origin: [0.0, 1.0, 0.0], yaw: 0.0, slope: 0.05 };
        let along = f.rotate([1.0, 0.0, 0.0]);
        assert!((along[1] / along[0] + 0.05).abs() < 1e-12);
        close(f.to_local(f.to_world([2.0, 0.2, 0.1])), [2.0, 0.2, 0.1]);
    }
}

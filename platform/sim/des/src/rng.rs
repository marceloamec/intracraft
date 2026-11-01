//! The project's seeded random generator (PCG32, XSH-RR), bit for bit the reference's `createRng`, so a seed gives the
//! same arrivals and process times in both implementations. The engines draw from it and nothing else.

const MULTIPLIER: u64 = 6_364_136_223_846_793_005;
const UINT32_RANGE: f64 = 4_294_967_296.0;
/// The stream the reference uses when none is given.
pub const DEFAULT_STREAM: u64 = 54;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pcg32 {
    state: u64,
    increment: u64,
}

impl Pcg32 {
    pub fn new(seed: u64, stream: u64) -> Pcg32 {
        let mut rng = Pcg32 { state: 0, increment: (stream << 1) | 1 };
        rng.next_u32();
        rng.state = rng.state.wrapping_add(seed);
        rng.next_u32();
        rng
    }

    /// Uniform in [0, 2³²).
    pub fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.state = old.wrapping_mul(MULTIPLIER).wrapping_add(self.increment);
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        xorshifted.rotate_right((old >> 59) as u32)
    }

    /// Uniform in [0, 1).
    pub fn next_f64(&mut self) -> f64 {
        f64::from(self.next_u32()) / UINT32_RANGE
    }

    /// Uniform in [min, max).
    pub fn uniform(&mut self, min: f64, max: f64) -> f64 {
        min + (max - min) * self.next_f64()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_pcg32_reference_sequence() {
        // The reference PCG32 (pcg-random.org, pcg32-demo with seed 42, stream 54) starts with these numbers; the
        // TypeScript createRng(42) gives the same.
        let mut rng = Pcg32::new(42, DEFAULT_STREAM);
        let first: Vec<u32> = (0..6).map(|_| rng.next_u32()).collect();
        assert_eq!(first, [0xa15c02b7, 0x7b47f409, 0xba1d3330, 0x83d2f293, 0xbfa4784b, 0xcbed606e]);
    }

    #[test]
    fn same_seed_same_numbers_other_stream_other_numbers() {
        let (mut a, mut b, mut c) = (Pcg32::new(7, 1), Pcg32::new(7, 1), Pcg32::new(7, 2));
        let xs: Vec<f64> = (0..100).map(|_| a.next_f64()).collect();
        let ys: Vec<f64> = (0..100).map(|_| b.next_f64()).collect();
        let zs: Vec<f64> = (0..100).map(|_| c.next_f64()).collect();
        assert_eq!(xs, ys);
        assert_ne!(xs, zs);
        assert!(xs.iter().all(|x| (0.0..1.0).contains(x)));
    }
}

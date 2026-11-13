//! PCG32 (XSH-RR 64/32), M. E. O'Neill — our own frozen implementation (DECISIONS D-016).
//! Brush randomness (jitter, spray) must replay identically forever, so this generator never
//! changes; the reference vectors below pin it.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pcg32 {
    state: u64,
    inc: u64,
}

const MULT: u64 = 6_364_136_223_846_793_005;

impl Pcg32 {
    /// Seed with a state and a stream selector (as `pcg32_srandom_r(initstate, initseq)`).
    pub fn new(initstate: u64, initseq: u64) -> Self {
        let mut r = Pcg32 {
            state: 0,
            inc: (initseq << 1) | 1,
        };
        r.next_u32();
        r.state = r.state.wrapping_add(initstate);
        r.next_u32();
        r
    }

    pub fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.state = old.wrapping_mul(MULT).wrapping_add(self.inc);
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        let rot = (old >> 59) as u32;
        xorshifted.rotate_right(rot)
    }

    /// Uniform in [0, 1) with 24 bits of precision (exactly representable in f32).
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 / 16_777_216.0
    }

    /// Uniform integer in 0..bound without modulo bias (as `pcg32_boundedrand_r`).
    pub fn below(&mut self, bound: u32) -> u32 {
        if bound == 0 {
            return 0;
        }
        let threshold = bound.wrapping_neg() % bound;
        loop {
            let r = self.next_u32();
            if r >= threshold {
                return r % bound;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_vectors_from_pcg_c_demo() {
        // pcg32-demo: pcg32_srandom_r(&rng, 42u, 54u); first six pcg32_random_r outputs.
        let mut r = Pcg32::new(42, 54);
        let got: Vec<u32> = (0..6).map(|_| r.next_u32()).collect();
        assert_eq!(
            got,
            vec![
                0xa15c02b7, 0x7b47f409, 0xba1d3330, 0x83d2f293, 0xbfa4784b, 0xcbed606e
            ]
        );
    }

    #[test]
    fn bounded_and_float_ranges() {
        let mut r = Pcg32::new(1, 2);
        for _ in 0..1000 {
            assert!(r.below(7) < 7);
            let f = r.next_f32();
            assert!((0.0..1.0).contains(&f));
        }
        assert_eq!(r.below(0), 0);
    }
}

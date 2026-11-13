//! Deterministic 64-bit FNV-1a hashing of document content, for replay tests (D-012).
//! Our own implementation: std's hashers are not guaranteed stable across Rust releases.

#[derive(Clone, Copy, Debug)]
pub struct Fnv64(u64);

impl Default for Fnv64 {
    fn default() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }
}

impl Fnv64 {
    pub fn bytes(&mut self, b: &[u8]) -> &mut Self {
        for &x in b {
            self.0 ^= u64::from(x);
            self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
        }
        self
    }
    pub fn u32(&mut self, v: u32) -> &mut Self {
        self.bytes(&v.to_le_bytes())
    }
    pub fn i32(&mut self, v: i32) -> &mut Self {
        self.bytes(&v.to_le_bytes())
    }
    pub fn str(&mut self, s: &str) -> &mut Self {
        self.u32(u32::try_from(s.len()).unwrap_or(u32::MAX))
            .bytes(s.as_bytes())
    }
    pub fn finish(&self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv1a_reference_values() {
        // Published FNV-1a 64 test vectors.
        assert_eq!(Fnv64::default().finish(), 0xcbf2_9ce4_8422_2325);
        assert_eq!(Fnv64::default().bytes(b"a").finish(), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(
            Fnv64::default().bytes(b"foobar").finish(),
            0x8594_4171_f739_67e8
        );
    }
}

/// A small generator of numbers that only look random, enough to vary what the game plays:
/// xorshift64*, seeded from the clock.
#[derive(Debug)]
pub(crate) struct Random(u64);

impl Random {
    pub(crate) fn from_clock() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| since.as_nanos());
        #[allow(clippy::cast_possible_truncation)] // the low bits vary the most
        Self::new(nanos as u64)
    }

    /// The same numbers for the same seed.
    fn new(seed: u64) -> Self {
        // Zero would give only zeros.
        Self(seed | 1)
    }

    pub(crate) fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// A number from 0 up to, but not including, `n`, which must not be 0.
    pub(crate) fn below(&mut self, n: usize) -> usize {
        assert!(n > 0, "no number is below 0");
        // The high bits are the best ones; the bias over 64 bits is too small to hear.
        #[allow(clippy::cast_possible_truncation)]
        let pick = ((u128::from(self.next_u64()) * n as u128) >> 64) as usize;
        pick
    }
}

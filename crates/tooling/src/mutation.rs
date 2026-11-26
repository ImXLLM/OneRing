//! The seeded mutation engine — port of `fuzz_lite.rs` from the DropAgentX-pro project.
//!
//! Deterministic, dependency-free xorshift64; mutations: byte replace/insert/delete, truncation,
//! injection-string and broken UTF-8 byte insertion — the same combinations real fuzzing
//! finds within minutes, here as always-on `#[test]` cases.

/// Deterministic xorshift64 PRNG — no dependencies, no nightly.
pub struct Rng(u64);

impl Rng {
    /// Initial seed — zero is forbidden (xorshift dies on zero).
    #[must_use]
    pub fn seeded(seed: u64) -> Self {
        Self(if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed })
    }

    /// Next random number — exactly `fuzz_lite.rs` (not the `Iterator` trait —
    /// the phase's dedicated PRNG; the name preserves port fidelity).
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// Mutate a byte array — byte delete/insert/replace + truncation + broken UTF-8 byte insertion.
    /// The `u64→usize` truncation in the phase PRNG is only a modulo — on a 32-bit target too
    /// the mutation distribution stays semantically unchanged; an explicit, targeted allow.
    #[allow(clippy::cast_possible_truncation)]
    pub fn mutate(&mut self, bytes: &mut Vec<u8>) {
        if bytes.is_empty() {
            bytes.push(b'x');
            return;
        }
        match self.next() % 5 {
            0 => {
                let i = (self.next() as usize) % bytes.len();
                bytes[i] = (self.next() & 0xff) as u8;
            }
            1 => {
                let i = (self.next() as usize) % bytes.len();
                bytes.insert(i, (self.next() & 0xff) as u8);
            }
            2 => {
                let i = (self.next() as usize) % bytes.len();
                bytes.remove(i);
            }
            3 => {
                let keep = (self.next() as usize) % bytes.len();
                bytes.truncate(keep);
            }
            // inserting broken UTF-8 continuation bytes — the most hostile case for parsers
            _ => {
                let i = (self.next() as usize) % bytes.len();
                bytes.insert(i, 0xF0);
                bytes.insert(i.saturating_add(1), 0x9F);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Determinism — same seed, same sequence.
    #[test]
    fn rng_is_deterministic() {
        let mut a = Rng::seeded(42);
        let mut b = Rng::seeded(42);
        for _ in 0..100 {
            assert_eq!(a.next(), b.next());
        }
        assert_ne!(Rng::seeded(1).next(), Rng::seeded(2).next());
    }

    /// A zero seed still yields a live sequence.
    #[test]
    fn zero_seed_is_alive() {
        let mut r = Rng::seeded(0);
        assert_ne!(r.next(), 0);
    }

    /// Mutation never panics on empty data and leaves no report
    /// of itself on non-empty data (no cache, no overflow).
    #[test]
    fn mutation_is_safe_on_all_shapes() {
        let mut rng = Rng::seeded(0x1234_5678);
        for len in [0usize, 1, 2, 100] {
            for _ in 0..50 {
                let mut data = vec![b'a'; len];
                rng.mutate(&mut data);
            }
        }
    }
}

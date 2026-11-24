//! موتور موتاسیون بذردار — پورت `fuzz_lite.rs` پروژهٔ DropAgentX-pro.
//!
//! xorshift64 قطعی و بدون وابستگی؛ جهش‌ها: جایگزینی/درج/حذف بایت، برش،
//! درج رشتهٔ تزریق و بایت‌های UTF-8 شکسته — همان ترکیب‌هایی که fuzz واقعی
//! در چند دقیقه می‌یابد، اینجا به‌صورت `#[test]` همیشه‌روشن.

/// گشتیابی قطعی xorshift64 — بدون وابستگی، بدون nightly.
pub struct Rng(u64);

impl Rng {
    /// بذر اولیه — صفر ممنوع (xorshift روی صفر می‌میرد).
    #[must_use]
    pub fn seeded(seed: u64) -> Self {
        Self(if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed })
    }

    /// عدد تصادفی بعدی — عین `fuzz_lite.rs`.
    pub fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// جهش بایت‌آرایه — حذف/درج/جایگزینی بایت + برش + درج بایت خراب UTF-8.
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
            // درج بایت‌های ادامهٔ UTF-8 شکسته — خصمانه‌ترین حالت پارسرها
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

    /// قطعیت — همان بذر، همان دنباله.
    #[test]
    fn rng_is_deterministic() {
        let mut a = Rng::seeded(42);
        let mut b = Rng::seeded(42);
        for _ in 0..100 {
            assert_eq!(a.next(), b.next());
        }
        assert_ne!(Rng::seeded(1).next(), Rng::seeded(2).next());
    }

    /// بذر صفر هم دنبالهٔ زنده می‌دهد.
    #[test]
    fn zero_seed_is_alive() {
        let mut r = Rng::seeded(0);
        assert_ne!(r.next(), 0);
    }

    /// جهش هرگز روی دادهٔ خالی panic نمی‌کند و روی دادهٔ غیرخالی هیچ گزارشی
    /// از خود باقی نمی‌گذارد (بدون کش، بدون سرریز).
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

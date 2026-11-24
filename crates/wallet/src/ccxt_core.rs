//! پورت خالص-راستِ پایه‌های `ccxt/ccxt` — Wave 6 (Step 3/4).
//!
//! سه کور کانونیک + ماتریس بازارِ بی‌کلید، همه بدون float روی پول و قیمت:
//!  * [`Precise`] + [`decimal_to_precision`] — عدد دهدهی مقیاس‌دار `i128`
//!    (`ts/src/base/Precise.ts`) با **درمانِ قضاییِ #1**: `TICK_SIZE` در
//!    حالتِ `Round` هم صددرصد با تقسیم/باقیماندهٔ صحیحِ `Precise` انجام
//!    می‌شود — فال‌بکِ `f64 %` سیسی‌اکس‌تا (`number.ts:7–243`) کور می‌شود و
//!    ریزشِ شناور روی تیکِ `0.0001`/`0.05` از ساختار حذف می‌گردد.
//!  * [`CcxtThrottler`] — دو الگوریتمِ `throttle.ts`: سطلِ سوراخ و پنجرهٔ
//!    لغزانِ `VecDeque` با هزینهٔ $O(1)$ (بدون shift آرایه).
//!  * [`OrderBook`]/[`OrderBookSide`] — کتابِ L2 مرتب‌سازی‌شده با جست‌وجوی
//!    دودویی + **درمانِ قضاییِ #2**: گپِ ترتیبِ nonce (`incoming > nonce+1`)
//!    دیگر بی‌صدا قبول نمی‌شود — `SequenceGapDetected` بدونِ دست‌زدن به
//!    لِیول‌ها برمی‌گردد تا resync فوریِ REST snapshot علامت بخورد.
//!  * [`SafeExtractor`] + [`omit_zero`] — استخراجِ دفاعیِ فیلدهای صرافی
//!    (`type.ts`) و [`CcxtErrorKind`] — رده‌بندیِ خطا با `is_retryable`
//!    و backoff نمایی (`errors.ts`).
//!  * [`keyless_public_market_matrix`] — ۸ مکانِ عمودیِ صفر-کلیدِ
//!    صفر-KYC (تأییدشدهٔ Arena) با مسیرها و وزن‌های واقعی.
//!
//! قانونِ شمول: هیچ شناوری روی قیمت/مبلغ/نوچه وارد نمی‌شود؛ فقط در
//! مرزهای آماریِ خالص (`bps`، `vwap`) برای مصرف‌کنندهٔ UI.

use std::collections::VecDeque;
use std::fmt;

// ── خطای دقت ────────────────────────────────────────────────────────────────

/// خطای موتور دقت — fail-closed روی سرریز/تجزیه؛ هرگز مقدارِ نادرست نمی‌دهد.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PreciseError {
    /// رشتهٔ ورودی عدد دهدهی معتبر نیست.
    #[error("precise parse error: {0}")]
    Parse(String),
    /// سرریزِ `i128` یا جدولِ توان‌ده — عملیات رد می‌شود.
    #[error("precise overflow: value out of i128/10^37 operating range")]
    Overflow,
}

/// نتیجهٔ استانداردِ دقتِ دهدهی.
pub type PreciseResult<T> = Result<T, PreciseError>;

// ── Precise ─────────────────────────────────────────────────────────────────

/// جدول توانِ ۱۰ در زمان کامپایل — `POW10[n] = 10^n` تا `10^37` (سقف `i128`).
pub const POW10: [i128; 38] = {
    let mut table = [1i128; 38];
    let mut i = 1;
    while i < 38 {
        table[i] = table[i - 1] * 10;
        i += 1;
    }
    table
};

/// توانِ ۱۰ با گیتِ محدوده — بیرون `[0, 37]` خطا می‌دهد (fail-closed).
///
/// # Errors
/// [`PreciseError::Overflow`] روی نمایِ بیرونِ جدول.
pub fn pow10_checked(exp: i32) -> PreciseResult<i128> {
    let idx = usize::try_from(exp).map_err(|_| PreciseError::Overflow)?;
    POW10.get(idx).copied().ok_or(PreciseError::Overflow)
}

/// عدد دهدهی دقیقِ مقیاس‌دار — مقدار همیشه `coeff × 10^(−decimals)`.
///
/// پورت `Precise.ts` با `i128` به‌جای `BigInt`؛ دامنهٔ عملیاتی:
/// `|coeff| < i128::MAX` و `decimals ∈ [0, 37]`. صفر همیشه کانونیک است
/// (`coeff == 0, decimals == 0`) — همان `reduce()` مرجع.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Precise {
    /// ضریب صحیحِ مقیاس‌دار.
    pub coeff: i128,
    /// تعداد اعشار — مقدار = `coeff / 10^decimals`.
    pub decimals: i32,
}

impl Precise {
    /// صفرِ کانونیک.
    #[must_use]
    pub const fn zero() -> Self {
        Self { coeff: 0, decimals: 0 }
    }

    /// عدد صحیحِ مقیاس‌دار از `i128`.
    #[must_use]
    pub const fn from_i128(v: i128) -> Self {
        Self { coeff: v, decimals: 0 }
    }

    /// تجزیهٔ هر رشتهٔ دهدهی/علمی — `"0.00012300"`, `"-42.5"`, `"1e-6"`.
    /// صفرهای انتهایی با `reduce()` خودکار نرمال می‌شوند.
    ///
    /// # Errors
    /// [`PreciseError::Parse`] روی شکل نامعتبر؛ [`PreciseError::Overflow`]
    /// روی مقادیر بیرون دامنهٔ `i128/10^37`.
    pub fn parse(input: &str) -> PreciseResult<Self> {
        let s = input.trim();
        if s.is_empty() {
            return Err(PreciseError::Parse("empty input".into()));
        }
        // علامت
        let (sign, body) = match s.as_bytes()[0] {
            b'-' => (-1i128, &s[1..]),
            b'+' => (1i128, &s[1..]),
            _ => (1i128, s),
        };
        // نم علمی — حداکثر یک e/E
        let (mantissa, exp_str) = match body.split_once(['e', 'E']) {
            Some((m, e)) if !e.is_empty() && !e.contains(['e', 'E']) => (m, e),
            Some(_) => return Err(PreciseError::Parse("malformed exponent".into())),
            None => (body, ""),
        };
        let exp: i64 = if exp_str.is_empty() {
            0
        } else {
            exp_str
                .parse()
                .map_err(|_| PreciseError::Parse(format!("bad exponent {exp_str:?}")))?
        };
        // مانتیس — حداکثر یک نقطه؛ همه رقم
        let (int_part, frac_part) = match mantissa.split_once('.') {
            Some((i, f)) if !i.contains('.') && !f.contains('.') => (i, f),
            Some(_) => return Err(PreciseError::Parse("multiple decimal points".into())),
            None => (mantissa, ""),
        };
        let all_digits =
            |x: &str| x.chars().all(|c| c.is_ascii_digit());
        if !all_digits(int_part) || !all_digits(frac_part) || (int_part.is_empty() && frac_part.is_empty()) {
            return Err(PreciseError::Parse(format!("malformed mantissa {mantissa:?}")));
        }
        let digits = format!("{int_part}{frac_part}");
        let trimmed = digits.trim_start_matches('0');
        let frac_len = i64::try_from(frac_part.len()).map_err(|_| PreciseError::Overflow)?;
        let mut decimals = i128::from(frac_len) - i128::from(exp);
        if trimmed.is_empty() {
            return Ok(Self::zero());
        }
        let mut coeff: i128 = trimmed
            .parse()
            .map_err(|_| PreciseError::Overflow)?;
        // نم منفیِ بزرگ → ضریب نمی‌شود؛ اعشار بزرگ‌تر از ۳۷ → fail-closed
        if decimals > 37 {
            return Err(PreciseError::Overflow);
        }
        if decimals < 0 {
            let shift = u32::try_from(-decimals).map_err(|_| PreciseError::Overflow)?;
            if shift > 37 {
                return Err(PreciseError::Overflow);
            }
            let idx = usize::try_from(shift).map_err(|_| PreciseError::Overflow)?;
            coeff = coeff
                .checked_mul(POW10[idx])
                .ok_or(PreciseError::Overflow)?;
            decimals = 0;
        }
        let mut out = Self {
            coeff: sign * coeff,
            decimals: i32::try_from(decimals).map_err(|_| PreciseError::Overflow)?,
        };
        out.reduce();
        Ok(out)
    }

    /// نرمال‌سازی کانونیک — حذف صفرهای انتهایی و صفرِ علامت‌دار.
    pub fn reduce(&mut self) {
        while self.decimals > 0 && self.coeff != 0 && self.coeff % 10 == 0 {
            self.coeff /= 10;
            self.decimals -= 1;
        }
        if self.coeff == 0 {
            self.decimals = 0;
        }
    }

    /// نسخهٔ کانونیکِ کاهش‌یافته — خود شیء تغییر نمی‌کند.
    #[must_use]
    pub fn reduced(mut self) -> Self {
        self.reduce();
        self
    }

    /// آیا صفر است؟ — همیشه روی فرم کانونیک درست است.
    #[must_use]
    pub const fn is_zero(&self) -> bool {
        self.coeff == 0
    }

    /// قدرمطلق.
    #[must_use]
    pub fn abs(&self) -> Self {
        Self { coeff: self.coeff.abs(), decimals: self.decimals }
    }

    /// نقض علامت.
    #[must_use]
    pub const fn neg(&self) -> Self {
        Self { coeff: -self.coeff, decimals: self.decimals }
    }

    /// هم‌مقیاس‌سازی دو عدد به بیشینهٔ اعشار مشترک.
    ///
    /// # Errors
    /// [`PreciseError::Overflow`] اگر اختلافِ اعشار از جدولِ توان بیرون بزند.
    pub fn align(&self, other: &Self) -> PreciseResult<(i128, i128, i32)> {
        let d = self.decimals.max(other.decimals);
        let a = self.scale_to(d)?;
        let b = other.scale_to(d)?;
        Ok((a, b, d))
    }

    /// ضریب در مقیاسِ `target` اعشار.
    ///
    /// # Errors
    /// [`PreciseError::Overflow`].
    fn scale_to(&self, target: i32) -> PreciseResult<i128> {
        let diff = target - self.decimals;
        if diff >= 0 {
            let p = pow10_checked(diff)?;
            self.coeff.checked_mul(p).ok_or(PreciseError::Overflow)
        } else {
            let p = pow10_checked(-diff)?;
            // مقسوم‌علیه دقیق — فرم کانونیک تضمین نمی‌کند بخش‌پذیر باشد؛
            // برای هم‌مقیاس‌سازیِ درست، باقی‌مانده باید صفر باشد.
            if self.coeff % p != 0 {
                return Err(PreciseError::Overflow);
            }
            Ok(self.coeff / p)
        }
    }

    /// جمعِ دقیق.
    ///
    /// # Errors
    /// [`PreciseError::Overflow`].
    pub fn add(&self, other: &Self) -> PreciseResult<Self> {
        let (a, b, d) = self.align(other)?;
        let mut out = Self { coeff: a.checked_add(b).ok_or(PreciseError::Overflow)?, decimals: d };
        out.reduce();
        Ok(out)
    }

    /// تفاضلِ دقیق (`self − other`).
    ///
    /// # Errors
    /// [`PreciseError::Overflow`].
    pub fn sub(&self, other: &Self) -> PreciseResult<Self> {
        self.add(&other.neg())
    }

    /// ضربِ دقیق — اعشارها جمع می‌شوند، ضرایب ضرب.
    ///
    /// # Errors
    /// [`PreciseError::Overflow`].
    pub fn mul(&self, other: &Self) -> PreciseResult<Self> {
        let coeff = self.coeff.checked_mul(other.coeff).ok_or(PreciseError::Overflow)?;
        let decimals = self
            .decimals
            .checked_add(other.decimals)
            .ok_or(PreciseError::Overflow)?;
        let mut out = Self { coeff, decimals };
        out.reduce();
        Ok(out)
    }

    /// تقسیمِ دقیق با دقتِ `precision` اعشار (پیش‌فرض ۱۸) و گردکردنِ
    /// نیم-دور-از-صفر — صددرصد صحیح، بدون شناور.
    ///
    /// # Errors
    /// [`PreciseError::Parse`] روی تقسیم بر صفر؛ [`PreciseError::Overflow`].
    pub fn div(&self, other: &Self, precision: i32) -> PreciseResult<Self> {
        if other.coeff == 0 {
            return Err(PreciseError::Parse("division by zero".into()));
        }
        let d = precision.clamp(0, 37);
        let sign: i128 = self.coeff.signum() * other.coeff.signum();
        let n = self.coeff.unsigned_abs();
        let dd = other.coeff.unsigned_abs();
        // q = round_half_away(n × 10^(d + b.dec − a.dec) / dd)
        let shift = i64::from(d)
            + i64::from(other.decimals)
            - i64::from(self.decimals);
        let (num, den): (u128, u128) = if shift >= 0 {
            let p = pow10_checked(i32::try_from(shift).map_err(|_| PreciseError::Overflow)?)?;
            (
                n.checked_mul(u128::try_from(p).map_err(|_| PreciseError::Overflow)?)
                    .ok_or(PreciseError::Overflow)?,
                dd,
            )
        } else {
            let p = pow10_checked(i32::try_from(-shift).map_err(|_| PreciseError::Overflow)?)?;
            (
                n,
                dd.checked_mul(u128::try_from(p).map_err(|_| PreciseError::Overflow)?)
                    .ok_or(PreciseError::Overflow)?,
            )
        };
        let mut q = i128::try_from(num / den).map_err(|_| PreciseError::Overflow)?;
        let rem = num % den;
        // نیم-دور-از-صفر: 2·rem ≥ den ⟺ rem ≥ den − rem (بدونِ سرریز)
        if rem >= den.saturating_sub(rem) {
            q += 1;
        }
        let mut out = Self { coeff: sign * q, decimals: d };
        out.reduce();
        Ok(out)
    }

    /// باقیماندهٔ دهدهی — معناشناسیِ `%` جاوااسکریپت (علامتِ dividend).
    ///
    /// # Errors
    /// [`PreciseError::Parse`] روی تقسیم بر صفر؛ [`PreciseError::Overflow`].
    pub fn mod_op(&self, other: &Self) -> PreciseResult<Self> {
        if other.coeff == 0 {
            return Err(PreciseError::Parse("modulo by zero".into()));
        }
        let (a, b, d) = self.align(other)?;
        let q = a / b; // rust: truncate toward zero — همان JS %
        let rem = a - q * b;
        let mut out = Self { coeff: rem, decimals: d };
        out.reduce();
        Ok(out)
    }

    /// مقایسهٔ ترتیبیِ دقیق.
    ///
    /// # Errors
    /// [`PreciseError::Overflow`] روی اختلافِ مقیاسِ بیرون جدول.
    pub fn cmp_precise(&self, other: &Self) -> PreciseResult<std::cmp::Ordering> {
        let (a, b, _) = self.align(other)?;
        Ok(a.cmp(&b))
    }

    /// `self > other`.
    ///
    /// # Errors
    /// [`PreciseError::Overflow`].
    pub fn gt(&self, other: &Self) -> PreciseResult<bool> {
        Ok(self.cmp_precise(other)? == std::cmp::Ordering::Greater)
    }

    /// `self ≥ other`.
    ///
    /// # Errors
    /// [`PreciseError::Overflow`].
    pub fn ge(&self, other: &Self) -> PreciseResult<bool> {
        Ok(self.cmp_precise(other)? != std::cmp::Ordering::Less)
    }

    /// `self < other`.
    ///
    /// # Errors
    /// [`PreciseError::Overflow`].
    pub fn lt(&self, other: &Self) -> PreciseResult<bool> {
        Ok(self.cmp_precise(other)? == std::cmp::Ordering::Less)
    }

    /// `self ≤ other`.
    ///
    /// # Errors
    /// [`PreciseError::Overflow`].
    pub fn le(&self, other: &Self) -> PreciseResult<bool> {
        Ok(self.cmp_precise(other)? != std::cmp::Ordering::Greater)
    }

    /// برابریِ مقداری (نه شیئی) — `0.10 == 0.1`.
    ///
    /// # Errors
    /// [`PreciseError::Overflow`].
    pub fn eq_val(&self, other: &Self) -> PreciseResult<bool> {
        Ok(self.cmp_precise(other)? == std::cmp::Ordering::Equal)
    }

    /// کمینهٔ دقیق.
    ///
    /// # Errors
    /// [`PreciseError::Overflow`].
    pub fn min(&self, other: &Self) -> PreciseResult<Self> {
        if self.le(other)? {
            Ok(self.reduced())
        } else {
            Ok(other.reduced())
        }
    }

    /// بیشینهٔ دقیق.
    ///
    /// # Errors
    /// [`PreciseError::Overflow`].
    pub fn max(&self, other: &Self) -> PreciseResult<Self> {
        if self.ge(other)? {
            Ok(self.reduced())
        } else {
            Ok(other.reduced())
        }
    }

    /// گردکردن/برش به `d` اعشار — هستهٔ مشترکِ `decimal_to_precision`.
    ///
    /// # Errors
    /// [`PreciseError::Overflow`].
    fn rounded_at(&self, target_decimals: i32, mode: RoundingMode) -> PreciseResult<Self> {
        let drop_digits = i64::from(self.decimals) - i64::from(target_decimals);
        let coeff = if drop_digits <= 0 {
            let scale = pow10_checked(i32::try_from(-drop_digits).map_err(|_| PreciseError::Overflow)?)?;
            self.coeff.checked_mul(scale).ok_or(PreciseError::Overflow)?
        } else {
            let scale = pow10_checked(i32::try_from(drop_digits).map_err(|_| PreciseError::Overflow)?)?;
            let quotient = self.coeff / scale;
            match mode {
                // نیم-دور-از-صفر: |rem| ≥ scale − |rem| (بدونِ سرریز — |rem| < scale)
                RoundingMode::Round => {
                    let remainder = (self.coeff - quotient * scale).abs();
                    if remainder >= scale.saturating_sub(remainder) {
                        quotient + self.coeff.signum()
                    } else {
                        quotient
                    }
                }
                RoundingMode::Truncate => quotient,
            }
        };
        let mut out = Self { coeff, decimals: target_decimals };
        out.reduce();
        Ok(out)
    }

    /// تعداد رقم‌های اعشاریِ قدرمطلقِ ضریب.
    #[must_use]
    fn digit_count(&self) -> u32 {
        if self.coeff == 0 {
            1
        } else {
            let mut v = self.coeff.abs();
            let mut n = 0u32;
            while v > 0 {
                v /= 10;
                n += 1;
            }
            n
        }
    }

    /// نمای معناییِ برای `PadWithZero` — دقیقاً `d` اعشار.
    ///
    /// # Errors
    /// [`PreciseError::Overflow`].
    fn padded_at(&self, d: i32) -> PreciseResult<Self> {
        let mut out = self.reduced();
        if out.decimals < d {
            let p = pow10_checked(d - out.decimals)?;
            out.coeff = out.coeff.checked_mul(p).ok_or(PreciseError::Overflow)?;
            out.decimals = d;
        }
        Ok(out)
    }
}

impl fmt::Display for Precise {
    /// نمایش کانونیک — بدون ریزشِ شناور؛ `decimals ≤ 0` صفرهایِ انتهایِ صحیح می‌آورد.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.coeff == 0 {
            if self.decimals > 0 {
                let d_usz = usize::try_from(self.decimals).map_err(|_| fmt::Error)?;
                return f.write_str(&format!("0.{}", "0".repeat(d_usz)));
            }
            return f.write_str("0");
        }
        let neg = self.coeff < 0;
        let mag = self.coeff.unsigned_abs().to_string();
        let d = self.decimals;
        let body = if d <= 0 {
            let mut s = mag;
            for _ in 0..(-d) {
                s.push('0');
            }
            s
        } else {
            let d_usz = usize::try_from(d).map_err(|_| fmt::Error)?;
            if mag.len() <= d_usz {
                format!("0.{}{}", "0".repeat(d_usz - mag.len()), mag)
            } else {
                let (i, frac) = mag.split_at(mag.len() - d_usz);
                format!("{i}.{frac}")
            }
        };
        if neg {
            f.write_str("-")?;
        }
        f.write_str(&body)
    }
}

impl std::str::FromStr for Precise {
    type Err = PreciseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

// ── decimal_to_precision ────────────────────────────────────────────────────

/// شمارشِ دقت — سه حالتِ `number.ts:7–243`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountingMode {
    /// تعداد رقم‌های اعشار (می‌تواند منفی باشد — گردکردنِ ده‌گان).
    DecimalPlaces,
    /// تعداد رقم‌های معنادار (`≥ 1`).
    SignificantDigits,
    /// تیکِ قیمت — `precision` یعنی تعداد اعشارِ تیکِ واحد `10^−p`.
    /// برای تیکِ دلخواهِ رشته‌ای [`to_precision_tick_size`] را ببینید.
    TickSize,
}

/// گردکردن یا برش — دو حالتِ مرجع.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundingMode {
    /// گردکردنِ نیم-دور-از-صفر (`ROUND`).
    Round,
    /// برش به سمت صفر (`TRUNCATE`).
    Truncate,
}

/// پدینگِ خروجی رشته.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaddingMode {
    /// بدون پدینگ — فرم کاهش‌یافتهٔ کانونیک.
    NoPadding,
    /// پدینگ با صفر تا دقتِ خواسته‌شده.
    PadWithZero,
}

/// پورت `decimalToPrecision` — صددرصد دهدهیِ دقیق.
///
/// # Errors
/// [`PreciseError::Parse`] روی ورودی نامعتبر / حالتِ بی‌معنا؛
/// [`PreciseError::Overflow`].
pub fn decimal_to_precision(
    x: &str,
    rounding: RoundingMode,
    precision: i32,
    counting: CountingMode,
    padding: PaddingMode,
) -> PreciseResult<String> {
    let v = Precise::parse(x)?;
    let out = match counting {
        CountingMode::DecimalPlaces => v.rounded_at(precision, rounding)?,
        CountingMode::SignificantDigits => {
            if precision < 1 {
                return Err(PreciseError::Parse("significant digits must be >= 1".into()));
            }
            if v.is_zero() {
                return Ok("0".to_owned());
            }
            // نمِ رقمرَد اول: e = (رقم‌ها − 1) − اعشار
            let e = i64::from(v.digit_count() - 1) - i64::from(v.decimals);
            let d = i64::from(precision - 1) - e;
            let d = i32::try_from(d).map_err(|_| PreciseError::Overflow)?;
            v.rounded_at(d, rounding)?
        }
        CountingMode::TickSize => {
            // تیکِ واحدِ 10^−p — مسیرِ صددرصدِ Precise (درمانِ #1)؛
            // تیکِ رشته‌ایِ دلخواه → [`to_precision_tick_size`].
            let ap = precision.unsigned_abs();
            if ap > 37 {
                return Err(PreciseError::Overflow);
            }
            let idx = ap as usize; // widening — بی‌زیان
            let tick = if precision >= 0 {
                Precise { coeff: 1, decimals: precision }
            } else {
                Precise { coeff: POW10[idx], decimals: 0 }
            };
            tick_round(&v, &tick, rounding)?
        }
    };
    match (counting, padding) {
        (CountingMode::DecimalPlaces, PaddingMode::PadWithZero) if precision >= 0 => {
            Ok(out.padded_at(precision)?.to_string())
        }
        (CountingMode::SignificantDigits, PaddingMode::PadWithZero) => {
            let mut p = out.reduced();
            if !p.is_zero() {
                let have = i64::from(p.digit_count());
                let want = i64::from(precision);
                if have < want {
                    let p10 = pow10_checked(i32::try_from(want - have).map_err(|_| PreciseError::Overflow)?)?;
                    p.coeff = p.coeff.checked_mul(p10).ok_or(PreciseError::Overflow)?;
                    p.decimals += i32::try_from(want - have).map_err(|_| PreciseError::Overflow)?;
                }
            }
            Ok(p.to_string())
        }
        _ => Ok(out.to_string()),
    }
}

/// گردکردن به مضربِ تیک با تقسیم/باقیماندهٔ صددرصدِ صحیح — درمانِ قضاییِ #1.
///
/// `q = val / tick`؛ در `Round`: `2·|rem| ≥ |tick| → q += sign(rem)`؛
/// نتیجه = `q_rounded × tick`. هیچ `f64 %`ی در مسیر نیست.
///
/// # Errors
/// [`PreciseError::Parse`] روی تیکِ صفر/منفی یا ورودیِ نامعتبر؛
/// [`PreciseError::Overflow`].
pub fn to_precision_tick_size(
    x: &str,
    tick: &str,
    rounding: RoundingMode,
) -> PreciseResult<String> {
    let v = Precise::parse(x)?;
    let t = Precise::parse(tick)?;
    if t.coeff <= 0 {
        return Err(PreciseError::Parse("tick size must be positive".into()));
    }
    Ok(tick_round(&v, &t, rounding)?.to_string())
}

/// هستهٔ تیکِ مشترک — ورودی‌های تجزیه‌شده.
///
/// # Errors
/// [`PreciseError::Overflow`].
fn tick_round(val: &Precise, tick: &Precise, rounding: RoundingMode) -> PreciseResult<Precise> {
    let (va, vb, _) = val.align(tick)?;
    let quot = va / vb; // برش به سمت صفر روی ضریبِ هم‌مقیاس
    let rem = va - quot * vb;
    let quot = match rounding {
        // نیم-دور-از-صفر: |rem| ≥ |vb| − |rem| (|rem| < |vb| — بدونِ سرریز)
        RoundingMode::Round => {
            let ra = rem.abs();
            let ba = vb.abs();
            if ra >= ba.saturating_sub(ra) {
                quot + rem.signum()
            } else {
                quot
            }
        }
        RoundingMode::Truncate => quot,
    };
    let coeff = quot.checked_mul(tick.coeff).ok_or(PreciseError::Overflow)?;
    let mut out = Precise { coeff, decimals: tick.decimals };
    out.reduce();
    Ok(out)
}

/// ردیفِ شاهدِ خودآزمونِ تیک — برای `GET /quant/venues`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TickSelftestRow {
    /// ورودی خام.
    pub input: &'static str,
    /// تیک خام.
    pub tick: &'static str,
    /// خروجی کانونیک.
    pub output: String,
}

/// خودآزمونِ قطعیِ تیک — نشانِ «صفر ریزشِ شناور» روی سیم.
///
/// # Errors
/// رخ نمی‌دهد — ورودی‌های ثابتِ داخل crate.
#[must_use]
pub fn precise_tick_selftest_rows() -> Vec<TickSelftestRow> {
    let case = |input: &'static str, tick: &'static str| -> TickSelftestRow {
        let output =
            to_precision_tick_size(input, tick, RoundingMode::Round).unwrap_or_else(|_| "ERR".into());
        TickSelftestRow { input, tick, output }
    };
    vec![
        case("0.2857", "0.0001"),
        case("0.15", "0.05"),
        case("0.125", "0.05"),
        case("1.0049", "0.05"),
        case("123.456", "0.0001"),
    ]
}

/// وضعیتِ نگهبانِ گپِ ترتیب — نشانِ سیمیِ درمانِ #2.
///
/// # Errors
/// رخ نمی‌دهد — پروبِ ثابت.
#[must_use]
pub fn orderbook_gap_guard_probe() -> [&'static str; 3] {
    let mut book = OrderBook::new(64);
    let applied = book.apply_delta(7, &[(100_000, 5_000)], &[(101_000, 4_000)]);
    let stale = book.apply_delta(5, &[(100_000, 1_000)], &[]);
    let gap = book.apply_delta(9, &[(102_000, 1_000)], &[]);
    [label_of(&applied), label_of(&stale), label_of(&gap)]
}

#[must_use]
fn label_of(outcome: &OrderBookSyncOutcome) -> &'static str {
    match outcome {
        OrderBookSyncOutcome::Applied { .. } => "Applied",
        OrderBookSyncOutcome::IgnoredStale => "IgnoredStale",
        OrderBookSyncOutcome::SequenceGapDetected { .. } => "SequenceGapDetected",
    }
}

// ── Throttler ───────────────────────────────────────────────────────────────

/// سیاستِ محدودساز — دو الگوریتمِ `throttle.ts:9–197`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LimiterPolicy {
    /// سطلِ سوراخ — `refill_rate_per_ms` مداوم، ظرفیت `capacity`.
    LeakyBucket,
    /// پنجرهٔ لغزانِ وزن‌دار — سقفِ وزن در بازهٔ زمانی.
    RollingWindow,
}

/// محدودسازِ نرخِ دوگانهٔ سیسی‌اکس‌تا — هر دو الگوریتم در یک ساختار.
#[derive(Debug, Clone)]
pub struct CcxtThrottler {
    /// سیاستِ فعال.
    pub policy: LimiterPolicy,
    /// نرخِ پرشدن (توکن/میلی‌ثانیه) — فقط `LeakyBucket`.
    pub refill_rate_per_ms: f64,
    /// ظرفیت سطل — فقط `LeakyBucket`.
    pub capacity: f64,
    /// توکن‌هایِ در دسترسِ فعلی.
    pub current_tokens: f64,
    /// هزینهٔ پیش‌فرضِ هر درخواست.
    pub default_cost: f64,
    /// مُهرِ آخرین پرشدن.
    last_refill_ms: u64,
    /// پنجرهٔ زمانی (`ms`) — فقط `RollingWindow`.
    pub window_size_ms: u64,
    /// سقفِ وزنِ پنجره — فقط `RollingWindow`.
    pub max_window_weight: i64,
    /// صفِ (`timestamp_ms`, `cost`) پنجره — بدونِ `shift` آرایه، `pop_front` $O(1)$.
    entries: VecDeque<(u64, i64)>,
    /// Σ وزنِ درون پنجره.
    window_total: i64,
}

impl CcxtThrottler {
    /// سطلِ سوراخ — مثلاً `capacity=10, rate=0.2/ms, default_cost=1`.
    #[must_use]
    pub fn new_leaky_bucket(capacity: f64, refill_rate_per_ms: f64, default_cost: f64) -> Self {
        Self {
            policy: LimiterPolicy::LeakyBucket,
            refill_rate_per_ms,
            capacity,
            current_tokens: capacity,
            default_cost,
            last_refill_ms: 0,
            window_size_ms: 0,
            max_window_weight: 0,
            entries: VecDeque::new(),
            window_total: 0,
        }
    }

    /// پنجرهٔ لغزانِ وزن‌دار — مثلاً `window=60_000ms, max=1200`.
    #[must_use]
    pub fn new_rolling_window(window_size_ms: u64, max_window_weight: i64) -> Self {
        Self {
            policy: LimiterPolicy::RollingWindow,
            refill_rate_per_ms: 0.0,
            capacity: 0.0,
            current_tokens: 0.0,
            default_cost: 1.0,
            last_refill_ms: 0,
            window_size_ms,
            max_window_weight,
            entries: VecDeque::new(),
            window_total: 0,
        }
    }

    /// تأخیرِ لازم پیش از درخواستِ `cost` در `now_ms` — و رزروِ آن.
    /// خروجی `0` یعنی بلافاصله مجاز.
    #[must_use]
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn wait_ms(&mut self, now_ms: u64, cost: f64) -> u64 {
        let cost = if cost >= 0.0 { cost } else { self.default_cost.max(0.0) };
        match self.policy {
            LimiterPolicy::LeakyBucket => {
                let elapsed = now_ms.saturating_sub(self.last_refill_ms) as f64;
                self.current_tokens =
                    (self.current_tokens + elapsed * self.refill_rate_per_ms).min(self.capacity);
                let wait = if self.current_tokens >= cost {
                    0.0
                } else if self.refill_rate_per_ms > 0.0 {
                    ((cost - self.current_tokens) / self.refill_rate_per_ms).ceil().max(0.0)
                } else {
                    f64::INFINITY
                };
                if !wait.is_finite() {
                    // بدونِ نرخِ پرشدن، هیچ تأخیرِ متناهی‌ای جواب نیست —
                    // صادقانه `u64::MAX`؛ متدِ بعدی همان مسیر را می‌بندد.
                    return u64::MAX;
                }
                let wait = wait as u64;
                let refilled = self.current_tokens + wait as f64 * self.refill_rate_per_ms;
                self.current_tokens = (refilled.min(self.capacity) - cost).max(0.0);
                self.last_refill_ms = now_ms.saturating_add(wait);
                wait
            }
            LimiterPolicy::RollingWindow => {
                let cost_i = cost as i64;
                let mut now = now_ms;
                let cutoff = |t: u64, now: u64, w: u64| t.saturating_add(w) <= now;
                while let Some(&(t, c)) = self.entries.front() {
                    if cutoff(t, now, self.window_size_ms) {
                        self.entries.pop_front();
                        self.window_total -= c;
                    } else {
                        break;
                    }
                }
                let mut wait = 0u64;
                while self.window_total + cost_i > self.max_window_weight {
                    let Some(&(t, c)) = self.entries.front() else {
                        break;
                    };
                    let step = t.saturating_add(self.window_size_ms).saturating_sub(now);
                    wait = wait.saturating_add(step.max(1));
                    now = now_ms.saturating_add(wait);
                    self.entries.pop_front();
                    self.window_total -= c;
                }
                let ts = now_ms.saturating_add(wait);
                self.entries.push_back((ts, cost_i));
                self.window_total += cost_i;
                wait
            }
        }
    }

    /// تعدادِ ردیف‌هایِ زندهٔ پنجره — تست/تلمتری.
    #[must_use]
    pub fn rolling_entries(&self) -> usize {
        self.entries.len()
    }

    /// Σ وزنِ درون پنجره — تست/تلمتری.
    #[must_use]
    pub const fn rolling_total(&self) -> i64 {
        self.window_total
    }
}

// ── OrderBook ───────────────────────────────────────────────────────────────

/// سمتِ کتاب — بالاترین bid در اندیسِ 0 / پایین‌ترین ask در اندیسِ 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SideKind {
    /// خریداران — مرتبِ نزولیِ قیمت.
    Bids,
    /// فروشندگان — مرتبِ صعودیِ قیمت.
    Asks,
}

/// یک سمتِ کتابِ L2 — لِیول‌های مرتب با جست‌وجوی دودویی (`bisect_left`).
#[derive(Debug, Clone)]
pub struct OrderBookSide {
    /// سمت.
    pub kind: SideKind,
    /// لِیول‌های (`price_micros`, `size_micros`) مرتبِ «بهترین‌اول».
    levels: Vec<(i64, i64)>,
    /// سقفِ عمق اختیاری — بعد از درج برش می‌خورد.
    pub depth_cap: Option<usize>,
}

impl OrderBookSide {
    /// سمتِ تازه — `depth_cap` اختیاری.
    #[must_use]
    pub fn new(kind: SideKind, depth_cap: Option<usize>) -> Self {
        Self { kind, levels: Vec::new(), depth_cap }
    }

    /// اندیسِ درج/یافتن — معادلِ `bisect_left` با کلیدِ علامت‌دار:
    /// bids با `−price` (بالاترین bid → اندیس 0)، asks با `+price`.
    #[must_use]
    fn bisect(&self, price: i64) -> usize {
        match self.kind {
            SideKind::Bids => self.levels.partition_point(|&(p, _)| p > price),
            SideKind::Asks => self.levels.partition_point(|&(p, _)| p < price),
        }
    }

    /// ذخیرهٔ لِیول — `size == 0` یعنی حذف؛ هم‌قیمت یعنی بازنویسیِ $O(1)$؛
    /// در غیر این‌صورت درج در اندیسِ دودویی + اجرای سقفِ عمق.
    pub fn store(&mut self, price_micros: i64, size_micros: i64) {
        let idx = self.bisect(price_micros);
        let exists = self.levels.get(idx).is_some_and(|&(p, _)| p == price_micros);
        if size_micros == 0 {
            if exists {
                self.levels.remove(idx);
            }
            return;
        }
        if exists {
            self.levels[idx].1 = size_micros;
        } else {
            self.levels.insert(idx, (price_micros, size_micros));
            if let Some(cap) = self.depth_cap {
                while self.levels.len() > cap {
                    self.levels.pop();
                }
            }
        }
    }

    /// بهترین لِیول — سرِ صف.
    #[must_use]
    pub fn best_level(&self) -> Option<(i64, i64)> {
        self.levels.first().copied()
    }

    /// n لِیولِ برتر.
    #[must_use]
    pub fn top_n(&self, n: usize) -> &[(i64, i64)] {
        let end = n.min(self.levels.len());
        &self.levels[..end]
    }

    /// تعدادِ لِیول‌های زنده.
    #[must_use]
    pub fn len(&self) -> usize {
        self.levels.len()
    }

    /// خالی بودن سمت.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.levels.is_empty()
    }

    /// اسپرد بر پایهٔ bps — میانِ `(ask − bid) / mid × 10⁴`.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn spread_bps(&self, asks: &OrderBookSide) -> Option<f64> {
        let (bid, _) = self.best_level()?;
        let (ask, _) = asks.best_level()?;
        if bid <= 0 || ask < bid {
            return None;
        }
        let mid = (bid as f64).midpoint(ask as f64);
        if mid <= 0.0 {
            return None;
        }
        Some((ask as f64 - bid as f64) / mid * 10_000.0)
    }

    /// قیمتِ میانی در مقیاس میکرو — گردِ پایینِ `(bid+ask)/2`.
    #[must_use]
    pub fn mid_price_micros(&self, asks: &OrderBookSide) -> Option<i64> {
        let (bid, _) = self.best_level()?;
        let (ask, _) = asks.best_level()?;
        if bid <= 0 || ask < bid {
            return None;
        }
        Some(bid.midpoint(ask))
    }

    /// `VWAP` برای مصرفِ `target_notional_micros` — پیمودنِ بهترین‌ها تا
    /// پر شدنِ ناتینال؛ ناکافی بودنِ عمق → `None`.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn vwap_for_notional(&self, target_notional_micros: i64) -> Option<f64> {
        if target_notional_micros <= 0 {
            return None;
        }
        let mut notional: i128 = 0;
        let mut size_acc: i128 = 0;
        for &(p, s) in &self.levels {
            notional += i128::from(p) * i128::from(s);
            size_acc += i128::from(s);
            if notional >= i128::from(target_notional_micros) && size_acc > 0 {
                let vwap = notional as f64 / size_acc as f64;
                return Some(vwap);
            }
        }
        None
    }
}

/// حاصلِ اعمالِ دلتا — درمانِ قضاییِ #2 (`OrderBook.ts:96–136`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderBookSyncOutcome {
    /// دلتا کهنه بود (`nonce ≤ current`) — نادیده گرفته شد.
    IgnoredStale,
    /// گپِ ترتیب — لِیول‌ها دست‌نخورده؛ resync فوریِ REST snapshot لازم.
    SequenceGapDetected {
        /// nonce مورد انتظار (`current + 1`).
        expected: u64,
        /// nonce دریافتی.
        received: u64,
    },
    /// دلتا اعمال شد و nonce تازه شد.
    Applied {
        /// nonce تازهٔ کتاب.
        nonce: u64,
    },
}

/// کتابِ سفارشِ L2 — دو سمتِ مرتب + نگهبانِ ترتیبِ nonce.
#[derive(Debug, Clone)]
pub struct OrderBook {
    /// nonce فعلی — ۰ یعنی هنوز اسنپ‌شات نگرفته.
    pub nonce: u64,
    /// سمتِ خرید.
    pub bids: OrderBookSide,
    /// سمتِ فروش.
    pub asks: OrderBookSide,
}

impl OrderBook {
    /// کتابِ تازه با سقفِ عمقِ یکسان برای دو سمت.
    #[must_use]
    pub fn new(depth_cap: usize) -> Self {
        Self {
            nonce: 0,
            bids: OrderBookSide::new(SideKind::Bids, Some(depth_cap)),
            asks: OrderBookSide::new(SideKind::Asks, Some(depth_cap)),
        }
    }

    /// اعمالِ دلتا با نگهبانِ کاملِ ترتیب:
    /// کهنه → `IgnoredStale`؛ گپ → `SequenceGapDetected` بدونِ جهش؛
    /// پیوسته → اعمال + `Applied`.
    pub fn apply_delta(
        &mut self,
        incoming_nonce: u64,
        bids_delta: &[(i64, i64)],
        asks_delta: &[(i64, i64)],
    ) -> OrderBookSyncOutcome {
        if incoming_nonce <= self.nonce {
            return OrderBookSyncOutcome::IgnoredStale;
        }
        if self.nonce > 0 && incoming_nonce > self.nonce + 1 {
            return OrderBookSyncOutcome::SequenceGapDetected {
                expected: self.nonce + 1,
                received: incoming_nonce,
            };
        }
        for &(p, s) in bids_delta {
            self.bids.store(p, s);
        }
        for &(p, s) in asks_delta {
            self.asks.store(p, s);
        }
        self.nonce = incoming_nonce;
        OrderBookSyncOutcome::Applied { nonce: incoming_nonce }
    }

    /// بازنشانی با اسنپ‌شات REST — پس از `SequenceGapDetected`.
    pub fn reset(&mut self, snapshot_nonce: u64, bids: &[(i64, i64)], asks: &[(i64, i64)]) {
        self.bids = OrderBookSide::new(SideKind::Bids, self.bids.depth_cap);
        self.asks = OrderBookSide::new(SideKind::Asks, self.asks.depth_cap);
        for &(p, s) in bids {
            self.bids.store(p, s);
        }
        for &(p, s) in asks {
            self.asks.store(p, s);
        }
        self.nonce = snapshot_nonce;
    }
}

// ── SafeExtractor ───────────────────────────────────────────────────────────

/// استخراجِ دفاعیِ فیلدها از `serde_json::Value` — پورتِ `safeString`/…
/// (`type.ts:17–186`): هیچ پنیک، هیچ حسرت — همه `Option`.
pub trait SafeExtractor {
    /// اولین کلیدِ قابل‌یافتن به‌صورت رشته (عدد → نمایش کانونیک).
    fn safe_string(&self, keys: &[&str]) -> Option<String>;
    /// اولین کلید به‌صورت [`Precise`] — قیمت/مبلغِ بدونِ ریزش.
    fn safe_precise(&self, keys: &[&str]) -> Option<Precise>;
    /// اولین کلید به‌صورت `i64` (رشتهٔ رقم‌دار هم می‌خواند).
    fn safe_i64(&self, keys: &[&str]) -> Option<i64>;
    /// اولین کلید به‌صورت بولی (`true`/`"true"`/…).
    fn safe_bool(&self, keys: &[&str]) -> Option<bool>;
}

impl SafeExtractor for serde_json::Value {
    fn safe_string(&self, keys: &[&str]) -> Option<String> {
        let v = walk(self, keys)?;
        match v {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Number(n) => Some(n.to_string()),
            _ => None,
        }
    }

    fn safe_precise(&self, keys: &[&str]) -> Option<Precise> {
        let s = self.safe_string(keys)?;
        Precise::parse(&s).ok()
    }

    fn safe_i64(&self, keys: &[&str]) -> Option<i64> {
        let v = walk(self, keys)?;
        match v {
            serde_json::Value::Number(n) => n.as_i64(),
            serde_json::Value::String(s) => s.trim().parse().ok(),
            _ => None,
        }
    }

    fn safe_bool(&self, keys: &[&str]) -> Option<bool> {
        let v = walk(self, keys)?;
        match v {
            serde_json::Value::Bool(b) => Some(*b),
            serde_json::Value::String(s) => match s.trim().to_ascii_lowercase().as_str() {
                "true" | "1" | "yes" => Some(true),
                "false" | "0" | "no" => Some(false),
                _ => None,
            },
            _ => None,
        }
    }
}

/// پیمودنِ مسیرِ کلیدها در اشیای تو در تو.
fn walk<'a>(v: &'a serde_json::Value, keys: &[&str]) -> Option<&'a serde_json::Value> {
    let mut cur = v;
    for k in keys {
        cur = cur.get(*k)?;
    }
    Some(cur)
}

/// صفر-سنجیِ رشتهٔ عددی — `""`/`"0"`/`"0.0"`/`"-0"` → `None`.
/// قراردادِ `omit_zero` مرجع: عرضهٔ صفر یعنی «فیلد غایب».
#[must_use]
pub fn omit_zero(s: &str) -> Option<&str> {
    match s.trim() {
        "" | "0" | "0.0" | "-0" => None,
        _ => Some(s),
    }
}

// ── Error taxonomy ──────────────────────────────────────────────────────────

/// واریانت‌هایِ شبکه‌ایِ قابلِ تلاشِ دوباره (`errors.ts:3–252`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NetworkErrorKind {
    /// سقفِ نرخ — backoff بلند.
    #[error("RateLimitExceeded")]
    RateLimitExceeded,
    /// سپرِ `DDoS` صرافی.
    #[error("DDoSProtection")]
    DdosProtection,
    /// پایانِ زمانِ درخواست.
    #[error("RequestTimeout")]
    RequestTimeout,
    /// صرافی در دسترس نیست.
    #[error("ExchangeNotAvailable")]
    ExchangeNotAvailable,
    /// nonce تکراری/کهنه (اختیارِ زمان‌محور).
    #[error("InvalidNonce")]
    InvalidNonce,
}

/// واریانت‌هایِ صرافی‌ایِ غیرقابلِ تلاشِ دوباره.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ExchangeErrorKind {
    /// احراز ناموفق.
    #[error("AuthenticationError")]
    AuthenticationError,
    /// موجودی ناکافی.
    #[error("InsufficientFunds")]
    InsufficientFunds,
    /// سفارشِ نامعتبر.
    #[error("InvalidOrder")]
    InvalidOrder,
    /// سفارش یافت نشد.
    #[error("OrderNotFound")]
    OrderNotFound,
    /// نمادِ ناشناخته.
    #[error("BadSymbol")]
    BadSymbol,
    /// حالتِ حاشیه از قبل ست شده.
    #[error("MarginModeAlreadySet")]
    MarginModeAlreadySet,
}

/// رده‌بندیِ خطای سیسی‌اکس‌تا — قابلِ تلاشِ دوباره در برابرِ قطعی.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CcxtErrorKind {
    /// خطای شبکه — retry/backoff.
    #[error("NetworkError: {0}")]
    NetworkError(#[from] NetworkErrorKind),
    /// خطای صرافی — retry بی‌معنا.
    #[error("ExchangeError: {0}")]
    ExchangeError(#[from] ExchangeErrorKind),
}

impl CcxtErrorKind {
    /// آیا تلاشِ دوباره معنا دارد؟ — فقط `NetworkError`.
    #[must_use]
    pub const fn is_retryable(&self) -> bool {
        matches!(self, Self::NetworkError(_))
    }

    /// backoff نماییِ پیشنهادی (ms) — پایهٔ هر واریانت، سقفِ ۶۰ ثانیه.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn recommended_backoff_ms(&self, attempt: u32) -> u64 {
        const CAP: f64 = 60_000.0;
        let base: f64 = match self {
            Self::NetworkError(NetworkErrorKind::RateLimitExceeded) => 2_000.0,
            Self::NetworkError(NetworkErrorKind::DdosProtection) => 1_500.0,
            Self::NetworkError(NetworkErrorKind::RequestTimeout) => 500.0,
            Self::NetworkError(NetworkErrorKind::ExchangeNotAvailable) => 1_000.0,
            Self::NetworkError(NetworkErrorKind::InvalidNonce) => 250.0,
            Self::ExchangeError(_) => 0.0,
        };
        let exp = 2f64.powi(i32::try_from(attempt).unwrap_or(30));
        (base * exp).min(CAP) as u64
    }
}

// ── Keyless market matrix ───────────────────────────────────────────────────

/// یک نقطهٔ پایانیِ عمودی — مسیر/متد/وزنِ واقعی.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketEndpoint {
    /// هدفِ کاربردی (`ticker24h`, `depth`, …).
    pub purpose: &'static str,
    /// متدِ HTTP.
    pub method: &'static str,
    /// مسیر یا بدنهٔ نمونه (`POST` های Hyperliquid مسیرِ واحد با بدنه).
    pub path: &'static str,
    /// وزنِ نرخ‌سنجیِ اعلامیِ صرافی (۰ = اعلام‌نشده).
    pub weight: u32,
}

/// مکانِ عمودیِ صفر-کلید — فقط دادهٔ عمومی، صفر KYC، صفر API key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeylessMarketVenue {
    /// شناسهٔ پایدار (`binance-spot`, …).
    pub id: &'static str,
    /// نام نمایشی.
    pub display_name: &'static str,
    /// گونهٔ مکان (`cex-spot`, `cex-linear-perp`, `dex-perp`, `dex-amm`).
    pub venue_kind: &'static str,
    /// پایهٔ URL.
    pub base_url: &'static str,
    /// احرازِ لازم — همیشه `"none"`.
    pub auth: &'static str,
    /// نقاطِ پایانی.
    pub endpoints: Vec<MarketEndpoint>,
    /// یادداشتِ کوتاه.
    pub note: &'static str,
}

/// تعدادِ مکان‌هایِ ماتریس — گیتِ آزمونِ سیم.
pub const KEYLESS_VENUE_COUNT: usize = 8;

/// ماتریسِ کاملِ بازارِ عمومیِ بی‌کلید — ۸ مکانِ تأییدشدهٔ Arena.
///
/// همهٔ مسیرها واقعی و در دسترسِ عموم؛ Hyperliquid با وزن‌هایِ اعلامیِ
/// `info` (2/2/4) و Base با زنجیرهٔ 8453 — هیچ کلیدی در هیچ مسیری.
#[allow(clippy::too_many_lines)] // کاتالوگِ داده‌ایِ ۸ مکان — طولش ماهیتِ داده است
#[must_use]
pub fn keyless_public_market_matrix() -> Vec<KeylessMarketVenue> {
    vec![
        KeylessMarketVenue {
            id: "binance-spot",
            display_name: "Binance Spot",
            venue_kind: "cex-spot",
            base_url: "https://api.binance.com",
            auth: "none",
            note: "اسپات کلاسیک — عمیق‌ترین نقدشوندگیِ جهانی",
            endpoints: vec![
                MarketEndpoint { purpose: "ticker24h", method: "GET", path: "/api/v3/ticker/24hr", weight: 0 },
                MarketEndpoint { purpose: "depth", method: "GET", path: "/api/v3/depth", weight: 0 },
                MarketEndpoint { purpose: "klines", method: "GET", path: "/api/v3/klines", weight: 0 },
            ],
        },
        KeylessMarketVenue {
            id: "binance-linear-usdtm",
            display_name: "Binance Linear USDT-M",
            venue_kind: "cex-linear-perp",
            base_url: "https://fapi.binance.com",
            auth: "none",
            note: "آینده‌های خطیِ USDT-M — funding/OI عمومی",
            endpoints: vec![
                MarketEndpoint { purpose: "premiumIndex", method: "GET", path: "/fapi/v1/premiumIndex", weight: 0 },
                MarketEndpoint { purpose: "fundingRate", method: "GET", path: "/fapi/v1/fundingRate", weight: 0 },
                MarketEndpoint { purpose: "openInterest", method: "GET", path: "/fapi/v1/openInterest", weight: 0 },
            ],
        },
        KeylessMarketVenue {
            id: "bybit-v5",
            display_name: "Bybit v5",
            venue_kind: "cex-linear-perp",
            base_url: "https://api.bybit.com",
            auth: "none",
            note: "v5 عمومی — تیکر/کتاب/کندل/تاریخچهٔ funding",
            endpoints: vec![
                MarketEndpoint { purpose: "tickers", method: "GET", path: "/v5/market/tickers", weight: 0 },
                MarketEndpoint { purpose: "orderbook", method: "GET", path: "/v5/market/orderbook", weight: 0 },
                MarketEndpoint { purpose: "kline", method: "GET", path: "/v5/market/kline", weight: 0 },
                MarketEndpoint { purpose: "funding-history", method: "GET", path: "/v5/market/funding/history", weight: 0 },
            ],
        },
        KeylessMarketVenue {
            id: "okx-v5",
            display_name: "OKX v5",
            venue_kind: "cex-linear-perp",
            base_url: "https://www.okx.com",
            auth: "none",
            note: "v5 عمومی — تیکر/کتاب/کندل/funding",
            endpoints: vec![
                MarketEndpoint { purpose: "tickers", method: "GET", path: "/api/v5/market/tickers", weight: 0 },
                MarketEndpoint { purpose: "books", method: "GET", path: "/api/v5/market/books", weight: 0 },
                MarketEndpoint { purpose: "candles", method: "GET", path: "/api/v5/market/candles", weight: 0 },
                MarketEndpoint { purpose: "funding-rate", method: "GET", path: "/api/v5/public/funding-rate", weight: 0 },
            ],
        },
        KeylessMarketVenue {
            id: "kraken-spot",
            display_name: "Kraken Spot",
            venue_kind: "cex-spot",
            base_url: "https://api.kraken.com",
            auth: "none",
            note: "REST عمومیِ کلاسیک — Ticker/Depth/OHLC",
            endpoints: vec![
                MarketEndpoint { purpose: "ticker", method: "GET", path: "/0/public/Ticker", weight: 0 },
                MarketEndpoint { purpose: "depth", method: "GET", path: "/0/public/Depth", weight: 0 },
                MarketEndpoint { purpose: "ohlc", method: "GET", path: "/0/public/OHLC", weight: 0 },
            ],
        },
        KeylessMarketVenue {
            id: "coinbase-exchange",
            display_name: "Coinbase Exchange",
            venue_kind: "cex-spot",
            base_url: "https://api.exchange.coinbase.com",
            auth: "none",
            note: "پارامترِ {id} در مسیر — نمادِ محصول",
            endpoints: vec![
                MarketEndpoint { purpose: "ticker", method: "GET", path: "/products/{id}/ticker", weight: 0 },
                MarketEndpoint { purpose: "book-l2", method: "GET", path: "/products/{id}/book?level=2", weight: 0 },
                MarketEndpoint { purpose: "candles", method: "GET", path: "/products/{id}/candles", weight: 0 },
            ],
        },
        KeylessMarketVenue {
            id: "hyperliquid-dex",
            display_name: "Hyperliquid DEX",
            venue_kind: "dex-perp",
            base_url: "https://api.hyperliquid.xyz",
            auth: "none",
            note: "POST /info با بدنهٔ تایپ‌دار — وزن‌هایِ اعلامیِ 2/2/4",
            endpoints: vec![
                MarketEndpoint { purpose: "allMids", method: "POST", path: "/info {\"type\":\"allMids\"}", weight: 2 },
                MarketEndpoint { purpose: "l2Book", method: "POST", path: "/info {\"type\":\"l2Book\",\"coin\":…}", weight: 2 },
                MarketEndpoint { purpose: "candleSnapshot", method: "POST", path: "/info {\"type\":\"candleSnapshot\",…}", weight: 4 },
                MarketEndpoint { purpose: "metaAndAssetCtxs", method: "POST", path: "/info {\"type\":\"metaAndAssetCtxs\"} (funding/OI)", weight: 0 },
            ],
        },
        KeylessMarketVenue {
            id: "base-mainnet-dex",
            display_name: "Base Mainnet DEX",
            venue_kind: "dex-amm",
            base_url: "https://mainnet.base.org",
            auth: "none",
            note: "Ethereum Base — Chain ID 8453 — RPC کلید-کمتر + کوتیشنِ استخر",
            endpoints: vec![
                MarketEndpoint { purpose: "rpc-chainid", method: "POST", path: "/ (eth_chainId → 0x2105 = 8453)", weight: 0 },
                MarketEndpoint { purpose: "pool-quote", method: "POST", path: "/ (eth_call روی قراردادِ استخرِ V2/V3)", weight: 0 },
            ],
        },
    ]
}

/// امضای کورِ سیسی‌اکس‌تا — گیتِ آزمونِ سیم.
pub const CCXT_CORE_SIGNATURE: &str = "ccxt-core-rs-v1";

// ── tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> Precise {
        Precise::parse(s).unwrap()
    }

    #[test]
    fn pow10_table_is_exact() {
        assert_eq!(POW10[0], 1);
        assert_eq!(POW10[18], 1_000_000_000_000_000_000);
        assert_eq!(POW10[37], 10_000_000_000_000_000_000_000_000_000_000_000_000);
        assert!(pow10_checked(38).is_err());
        assert!(pow10_checked(-1).is_err());
    }

    #[test]
    fn precise_parses_and_normalizes_trailing_zeros() {
        assert_eq!(p("0.00012300").to_string(), "0.000123");
        assert_eq!(p("-42.5").to_string(), "-42.5");
        assert_eq!(p("1e-6").to_string(), "0.000001");
        assert_eq!(p("1500e-2").to_string(), "15");
        assert_eq!(p("2.5E+3").to_string(), "2500");
        assert_eq!(p("0").to_string(), "0");
        assert_eq!(p("-0").to_string(), "0");
        assert_eq!(p(".5").to_string(), "0.5");
        assert_eq!(p("1.").to_string(), "1");
        assert!(Precise::parse("1.2.3").is_err());
        assert!(Precise::parse("abc").is_err());
        assert!(Precise::parse("").is_err());
        assert!(Precise::parse("1e").is_err());
        assert!(Precise::parse("1e-40").is_err(), "اعشارِ > ۳۷ fail-closed");
    }

    #[test]
    fn precise_ops_are_float_free_exact() {
        // کلاسیکِ IEEE-754: 0.1 + 0.2 = 0.30000000000000004 — اینجا 0.3
        assert_eq!(p("0.1").add(&p("0.2")).unwrap().to_string(), "0.3");
        assert_eq!(p("0.3").sub(&p("0.1")).unwrap().to_string(), "0.2");
        assert_eq!(p("0.000001").mul(&p("1000")).unwrap().to_string(), "0.001");
        assert_eq!(p("-42.5").abs().to_string(), "42.5");
        assert_eq!(p("7").neg().to_string(), "-7");
        // تقسیم با دقتِ ۱۸ + گردِ نیم-دور-از-صفر
        assert_eq!(p("1").div(&p("3"), 18).unwrap().to_string(), "0.333333333333333333");
        assert_eq!(p("2").div(&p("3"), 6).unwrap().to_string(), "0.666667");
        assert_eq!(p("-2").div(&p("3"), 6).unwrap().to_string(), "-0.666667");
        assert_eq!(p("10").div(&p("4"), 18).unwrap().to_string(), "2.5");
        assert!(p("1").div(&p("0"), 18).is_err());
        assert_eq!(p("7.5").mod_op(&p("2")).unwrap().to_string(), "1.5");
        assert_eq!(p("-7.5").mod_op(&p("2")).unwrap().to_string(), "-1.5");
        // مقایسه‌ها
        assert!(p("0.10").eq_val(&p("0.1")).unwrap());
        assert!(p("1.00001").gt(&p("1")).unwrap());
        assert!(p("1").le(&p("1.0")).unwrap());
        assert!(p("2").lt(&p("3")).unwrap());
        assert!(p("3").ge(&p("3")).unwrap());
        assert_eq!(p("2.5").max(&p("3")).unwrap().to_string(), "3");
        assert_eq!(p("2.5").min(&p("3")).unwrap().to_string(), "2.5");
    }

    #[test]
    fn decimal_to_precision_matches_reference_shapes() {
        use CountingMode::{DecimalPlaces as DP, SignificantDigits as SD, TickSize as TS};
        use PaddingMode::{NoPadding as NP, PadWithZero as PZ};
        use RoundingMode::{Round, Truncate};
        let dtp = decimal_to_precision;
        // DecimalPlaces
        assert_eq!(dtp("12.5465", Round, 2, DP, NP).unwrap(), "12.55");
        assert_eq!(dtp("12.5465", Truncate, 2, DP, NP).unwrap(), "12.54");
        assert_eq!(dtp("12.5465", Round, 0, DP, NP).unwrap(), "13");
        assert_eq!(dtp("-12.5465", Round, 2, DP, NP).unwrap(), "-12.55");
        // دقتِ منفی — ده‌گان
        assert_eq!(dtp("1234.567", Round, -2, DP, NP).unwrap(), "1200");
        assert_eq!(dtp("1234.567", Round, -2, DP, PZ).unwrap(), "1200");
        // SignificantDigits
        assert_eq!(dtp("0.000123456", Round, 3, SD, NP).unwrap(), "0.000123");
        assert_eq!(dtp("123.456", Round, 3, SD, NP).unwrap(), "123");
        assert_eq!(dtp("9.99", Round, 2, SD, NP).unwrap(), "10");
        assert_eq!(dtp("0.999", Round, 1, SD, NP).unwrap(), "1");
        assert_eq!(dtp("123.456", Truncate, 3, SD, PZ).unwrap(), "123");
        assert_eq!(dtp("0.000123456", Truncate, 3, SD, PZ).unwrap(), "0.000123");
        assert_eq!(dtp("0", Round, 5, SD, NP).unwrap(), "0");
        // TickSize واحدِ 10^−p
        assert_eq!(dtp("1.23456", Round, 4, TS, NP).unwrap(), "1.2346");
        assert_eq!(dtp("1.23456", Truncate, 4, TS, NP).unwrap(), "1.2345");
        // پدینگ
        assert_eq!(dtp("12.5", Round, 4, DP, PZ).unwrap(), "12.5000");
        assert_eq!(dtp("12.5", Round, 4, DP, NP).unwrap(), "12.5");
    }

    #[test]
    fn forensic_fix_1_tick_size_round_is_pure_precise_no_float_drift() {
        use RoundingMode::{Round, Truncate};
        // فال‌بکِ f64 % مرجع روی 0.0001 ریزش می‌دهد؛ اینجا نتایجِ دقیقِ
        // محاسبه‌شدهٔ صحیح اند:
        assert_eq!(to_precision_tick_size("0.2857", "0.0001", Round).unwrap(), "0.2857");
        assert_eq!(to_precision_tick_size("123.456", "0.0001", Round).unwrap(), "123.456");
        // نیم‌گردها
        assert_eq!(to_precision_tick_size("0.125", "0.05", Round).unwrap(), "0.15");
        assert_eq!(to_precision_tick_size("0.125", "0.05", Truncate).unwrap(), "0.1");
        assert_eq!(to_precision_tick_size("1.0049", "0.05", Round).unwrap(), "1");
        // تیکِ منفی/صفر ممنوع
        assert!(to_precision_tick_size("1", "0", Round).is_err());
        assert!(to_precision_tick_size("1", "-0.01", Round).is_err());
    }

    #[test]
    fn tick_selftest_rows_are_deterministic_and_exact() {
        let rows = precise_tick_selftest_rows();
        assert_eq!(rows.len(), 5);
        assert_eq!(rows[0].output, "0.2857");
        assert_eq!(rows[1].output, "0.15");
        assert_eq!(rows[2].output, "0.15");
        assert_eq!(rows[3].output, "1");
        assert_eq!(rows[4].output, "123.456");
        let again = precise_tick_selftest_rows();
        assert_eq!(rows, again, "خودآزمون باید دترمینیستیک باشد");
    }

    #[test]
    fn leaky_bucket_delays_are_exact() {
        let mut t = CcxtThrottler::new_leaky_bucket(10.0, 1.0, 6.0);
        assert_eq!(t.wait_ms(0, 6.0), 0, "ظرفیتِ کامل — بلافاصله");
        assert_eq!(t.wait_ms(0, 6.0), 2, "۴ توکن مانده → ceil(2/1)=۲ms");
        assert_eq!(t.wait_ms(1_000, 1.0), 0, "۱ ثانیه بعد — پر شده");
    }

    #[test]
    fn rolling_window_is_o1_weighted() {
        let mut t = CcxtThrottler::new_rolling_window(60_000, 100);
        assert_eq!(t.wait_ms(0, 60.0), 0);
        assert_eq!(t.wait_ms(1, 60.0), 59_999, "تا خروجِ اولی از پنجره صبر می‌کند");
        // درخواستِ دوم در لحظهٔ انقضایِ اولی زمان‌بندی شد — یعنی اولی بیرون
        // است و فقط تازه‌مانده: entries=1، وزن=۶۰ (نه ۱۲۰).
        assert_eq!(t.rolling_entries(), 1);
        assert_eq!(t.rolling_total(), 60);
        // پاک‌سازیِ کاملِ پنجره در زمانِ آینده
        assert_eq!(t.wait_ms(70_000, 1.0), 0);
        assert_eq!(t.rolling_entries(), 2, "قدیمی هنوز تا ۱۲۰٬۰۰۰ زنده است + تازه");
    }

    #[test]
    fn orderbook_side_stores_sorts_and_removes() {
        let mut bids = OrderBookSide::new(SideKind::Bids, None);
        bids.store(100_000, 5);
        bids.store(101_000, 7);
        bids.store(99_000, 2);
        assert_eq!(bids.best_level(), Some((101_000, 7)), "بالاترین bid سرِ صف");
        assert_eq!(bids.top_n(2).len(), 2);
        bids.store(101_000, 0);
        assert_eq!(bids.best_level(), Some((100_000, 5)), "size=0 حذف می‌کند");
        // بازنویسیِ $O(1)$ همان قیمت
        bids.store(100_000, 9);
        assert_eq!(bids.best_level(), Some((100_000, 9)));

        let mut asks = OrderBookSide::new(SideKind::Asks, None);
        asks.store(103_000, 4);
        asks.store(102_000, 3);
        assert_eq!(asks.best_level(), Some((102_000, 3)), "پایین‌ترین ask سرِ صف");
        assert!(bids.spread_bps(&asks).unwrap() > 0.0, "اسپردِ مثبت");
        assert_eq!(bids.mid_price_micros(&asks), Some(101_000));
    }

    #[test]
    fn orderbook_side_depth_cap_and_vwap() {
        let mut asks = OrderBookSide::new(SideKind::Asks, Some(2));
        asks.store(100, 1);
        asks.store(101, 1);
        asks.store(102, 1);
        assert_eq!(asks.len(), 2, "سقفِ عمقِ ۲");
        assert_eq!(asks.best_level(), Some((100, 1)));
        assert_eq!(asks.top_n(5).len(), 2);

        let mut bids = OrderBookSide::new(SideKind::Bids, None);
        bids.store(99, 100);
        bids.store(98, 300);
        // ناتینالِ ۲۰٬۰۰۰: 99×100=9900 → 98×300=29400 → cumulative 39300 ≥ 20000
        // vwap = 39300/400 = 98.25
        let vwap = bids.vwap_for_notional(20_000).unwrap();
        assert!((vwap - 98.25).abs() < 1e-9);
        assert!(bids.vwap_for_notional(10_000_000).is_none(), "عمق ناکافی");
        assert!(bids.vwap_for_notional(0).is_none());
    }

    #[test]
    fn forensic_fix_2_sequence_gap_is_detected_not_applied() {
        let mut book = OrderBook::new(64);
        // اولین اسنپ‌شاتِ پیوسته
        let out = book.apply_delta(7, &[(100_000, 5_000)], &[(101_000, 4_000)]);
        assert_eq!(out, OrderBookSyncOutcome::Applied { nonce: 7 });
        // کهنه — نادیده
        assert_eq!(book.apply_delta(5, &[], &[]), OrderBookSyncOutcome::IgnoredStale);
        assert_eq!(book.nonce, 7);
        // گپ — بدونِ جهشِ لِیول‌ها
        let gap = book.apply_delta(9, &[(102_000, 9_999)], &[]);
        assert_eq!(
            gap,
            OrderBookSyncOutcome::SequenceGapDetected { expected: 8, received: 9 }
        );
        assert_eq!(book.nonce, 7, "nonce ثابت ماند");
        assert!(book.asks.top_n(9).iter().all(|&(p, _)| p != 102_000), "لِیولِ گپ‌دار هرگز وارد نشد");
        // پیوستهٔ بعد از گپ — همچنان رد می‌شود تا resync
        assert_eq!(
            book.apply_delta(8, &[], &[]),
            OrderBookSyncOutcome::Applied { nonce: 8 }
        );
        // resync با اسنپ‌شات
        book.reset(20, &[(100_000, 1)], &[(101_000, 1)]);
        assert_eq!(book.nonce, 20);
        assert_eq!(book.apply_delta(23, &[], &[]).clone(),
            OrderBookSyncOutcome::SequenceGapDetected { expected: 21, received: 23 });
    }

    #[test]
    fn gap_guard_probe_labels_are_stable() {
        assert_eq!(orderbook_gap_guard_probe(), ["Applied", "IgnoredStale", "SequenceGapDetected"]);
    }

    #[test]
    fn safe_extractor_walks_nested_paths_defensively() {
        let v: serde_json::Value = serde_json::json!({
            "info": { "lastPrice": "0.000123", "size": "42", "active": true },
            "last": 1234.5
        });
        assert_eq!(v.safe_string(&["info", "lastPrice"]).as_deref(), Some("0.000123"));
        assert_eq!(v.safe_string(&["missing", "x"]), None);
        assert_eq!(v.safe_precise(&["info", "lastPrice"]).unwrap().to_string(), "0.000123");
        assert_eq!(v.safe_i64(&["info", "size"]), Some(42));
        assert_eq!(v.safe_i64(&["nope"]), None);
        assert_eq!(v.safe_bool(&["info", "active"]), Some(true));
        assert_eq!(v.safe_bool(&["info", "size"]), None);
    }

    #[test]
    fn omit_zero_filters_all_zero_shapes() {
        assert_eq!(omit_zero(""), None);
        assert_eq!(omit_zero("0"), None);
        assert_eq!(omit_zero("0.0"), None);
        assert_eq!(omit_zero("-0"), None);
        assert_eq!(omit_zero(" 0 "), None, "حاشیهٔ سفید هم صفر است");
        assert_eq!(omit_zero("0.0001"), Some("0.0001"));
        assert_eq!(omit_zero("12.5"), Some("12.5"));
    }

    #[test]
    fn error_taxonomy_retryability_and_backoff() {
        use NetworkErrorKind as N;
        use ExchangeErrorKind as E;
        let retryable = [
            CcxtErrorKind::NetworkError(N::RateLimitExceeded),
            CcxtErrorKind::NetworkError(N::DdosProtection),
            CcxtErrorKind::NetworkError(N::RequestTimeout),
            CcxtErrorKind::NetworkError(N::ExchangeNotAvailable),
            CcxtErrorKind::NetworkError(N::InvalidNonce),
        ];
        for e in &retryable {
            assert!(e.is_retryable());
            assert!(e.recommended_backoff_ms(0) > 0);
        }
        let fatal = [
            CcxtErrorKind::ExchangeError(E::AuthenticationError),
            CcxtErrorKind::ExchangeError(E::InsufficientFunds),
            CcxtErrorKind::ExchangeError(E::InvalidOrder),
            CcxtErrorKind::ExchangeError(E::OrderNotFound),
            CcxtErrorKind::ExchangeError(E::BadSymbol),
            CcxtErrorKind::ExchangeError(E::MarginModeAlreadySet),
        ];
        for e in &fatal {
            assert!(!e.is_retryable());
            assert_eq!(e.recommended_backoff_ms(0), 0);
        }
        // نمایی با سقفِ ۶۰ ثانیه
        assert_eq!(CcxtErrorKind::NetworkError(N::RequestTimeout).recommended_backoff_ms(0), 500);
        assert_eq!(CcxtErrorKind::NetworkError(N::RequestTimeout).recommended_backoff_ms(1), 1_000);
        assert_eq!(CcxtErrorKind::NetworkError(N::RateLimitExceeded).recommended_backoff_ms(10), 60_000);
    }

    #[test]
    fn keyless_matrix_has_exactly_eight_venues() {
        let m = keyless_public_market_matrix();
        assert_eq!(m.len(), KEYLESS_VENUE_COUNT);
        let ids: Vec<&str> = m.iter().map(|v| v.id).collect();
        assert_eq!(ids, [
            "binance-spot",
            "binance-linear-usdtm",
            "bybit-v5",
            "okx-v5",
            "kraken-spot",
            "coinbase-exchange",
            "hyperliquid-dex",
            "base-mainnet-dex",
        ]);
        for v in &m {
            assert_eq!(v.auth, "none", "همهٔ مکان‌ها بی‌کلید");
            assert!(!v.endpoints.is_empty(), "هر مکان باید نقطهٔ پایانی داشته باشد");
        }
        let hl = m.iter().find(|v| v.id == "hyperliquid-dex").unwrap();
        assert_eq!(hl.endpoints[0].weight, 2);
        assert_eq!(hl.endpoints[2].weight, 4);
        let base = m.iter().find(|v| v.id == "base-mainnet-dex").unwrap();
        assert!(base.note.contains("8453"));
    }
}

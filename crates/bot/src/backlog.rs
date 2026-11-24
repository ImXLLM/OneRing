//! تخلیهٔ عقب‌ماندگی آپدیت‌های تلگرام — پورت `ShopifyHermes/bot.py::drain_backlog`
//! (پوشهٔ ۰۳ گنجینه).
//!
//! الگوی هرمس: در بوت، یک فراخوان `getUpdates` با `offset = -1` و
//! `timeout = 0` می‌گیرد؛ تلگرام «آخرین» آپدیتِ معلق را می‌دهد و
//! `offset جدید = آخرین update_id + 1` یعنی «همهٔ عقب‌ماندگی را بدون پردازش
//! تأیید کن». نتیجه: بعد از ری‌استارت، بات به پیام‌های کهنه پاسخ دوباره
//! نمی‌دهد (ضد double-reply).
//!
//! توابع خالص‌اند (بدون شبکه) — پوستهٔ واقعی (teloxide / وبهوک) آن‌ها را
//! روی خروجی خام API اعمال می‌کند؛ با این شکل، منطق قطعی و آفلاین تست می‌شود.
#![forbid(unsafe_code)]

/// خروجی تخلیهٔ عقب‌ماندگی.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrainOutcome {
    /// عقب‌ماندگی تخلیه شد — پردازش را از این offset شروع کن (`last + 1`).
    Drained(i64),
    /// آپدیت معلقی نبود — همان offset پایدارِ قبلی بماند (پورت `ST["offset"]`).
    KeepCurrent,
}

impl DrainOutcome {
    /// offset اجرایی بعد از تخلیه — برای لاگ و فراخوان بعدی.
    #[must_use]
    pub const fn offset(self, current: i64) -> i64 {
        match self {
            Self::Drained(o) => o,
            Self::KeepCurrent => current,
        }
    }

    /// آیا این آپدیت کهنه است و باید رد شود؟ (پورت `if uid < offset: continue`)
    #[must_use]
    pub const fn is_stale(self, current: i64, update_id: i64) -> bool {
        update_id < self.offset(current)
    }
}

/// محاسبهٔ offset تخلیه از خروجی خام `getUpdates {"offset": -1, "timeout": 0}`.
///
/// ورودی: شناسهٔ آپدیت‌های برگشتی به ترتیب API (کوچک به بزرگ). عین پایتون:
/// اگر ردیفی بود `rs[-1]["update_id"] + 1`؛ اگر خالی/خطا بود → `KeepCurrent`.
#[must_use]
pub fn drain_backlog_offset(pending_update_ids: &[i64]) -> DrainOutcome {
    match pending_update_ids.iter().last() {
        Some(&last) if last >= 0 => DrainOutcome::Drained(last.saturating_add(1)),
        _ => DrainOutcome::KeepCurrent,
    }
}

/// offset بعدی پس از پردازش موفق یک آپدیت — عین `offset = uid + 1` پایتون.
#[must_use]
pub const fn next_offset(update_id: i64) -> i64 {
    update_id.saturating_add(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// الگوی هرمس — آخرین update_id معلق + ۱؛ خالی → نگه‌داشتن جاری.
    #[test]
    fn drain_uses_last_pending_plus_one() {
        assert_eq!(drain_backlog_offset(&[10, 11, 12]), DrainOutcome::Drained(13));
        assert_eq!(drain_backlog_offset(&[41]), DrainOutcome::Drained(42));
        assert_eq!(drain_backlog_offset(&[]), DrainOutcome::KeepCurrent);
        // شناسهٔ منفی (پاسخ خطا) → صادقانه جاری می‌ماند
        assert_eq!(drain_backlog_offset(&[-1]), DrainOutcome::KeepCurrent);
    }

    /// offset اجرایی و کهنگی — پیام‌های قبل از تخلیه دیگر پردازش نمی‌شوند.
    #[test]
    fn stale_detection_prevents_double_replies() {
        let drained = drain_backlog_offset(&[100]);
        assert_eq!(drained.offset(0), 101);
        assert!(drained.is_stale(0, 100), "آپدیت تخلیه‌شده کهنه است");
        assert!(drained.is_stale(0, 42), "پایین‌تر از تخلیه، کهنه است");
        assert!(!drained.is_stale(0, 101), "آپدیت تازه پردازش می‌شود");
        let kept = drain_backlog_offset(&[]);
        assert_eq!(kept.offset(77), 77);
        assert!(kept.is_stale(77, 76));
        assert!(!kept.is_stale(77, 77));
    }

    /// زنجیرهٔ پردازش — بعد از هر آپدیت، offset جلو می‌رود (بدون پاسخ دوباره).
    #[test]
    fn processing_chain_advances_offset() {
        let mut offset = drain_backlog_offset(&[5, 9]).offset(0);
        assert_eq!(offset, 10);
        for uid in [10, 11, 12] {
            assert!(!drain_backlog_offset(&[]).is_stale(offset, uid));
            offset = next_offset(uid);
        }
        assert_eq!(offset, 13);
        assert!(drain_backlog_offset(&[]).is_stale(offset, 12), "تکرار uid=12 رد می‌شود");
    }
}

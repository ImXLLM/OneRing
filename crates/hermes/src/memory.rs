//! حافظهٔ ۷لایهٔ هرمس — طبقه‌بندی ثابت یادداشت‌ها.
//!
//! هر یادداشت فقط در یک لایه زندگی می‌کند؛ لایه‌ها enum بستهٔ کامپایل‌زمان
//! هستند تا رشتهٔ آزادِ قابل‌خراش وجود نداشته باشد.

/// لایه‌های حافظه — همان ۷ لایهٔ منشور مغز.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryLayer {
    /// کاری — جمله‌های کوتاه همین نشست
    Working,
    /// رویدادی — «کاربر چه پرسید/چه گرفت»
    Episodic,
    /// مفهومی — واقعیت‌های پایدار دربارهٔ کاربر
    Semantic,
    /// رویه‌ای — الگوی کارِ تکرارشونده
    Procedural,
    /// هویتی — امتیاز اعتبار و تغییرات آن
    Identity,
    /// مالی — رویدادهای پولی خوانده‌شده (هرگز مبلغ روی خودش نگه نمی‌دارد)
    Financial,
    /// اجتماعی — روابط و ارجاع‌ها
    Social,
}

impl MemoryLayer {
    /// رشتهٔ ذخیره در دیتابیس — `snake_case` ثابت.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            MemoryLayer::Working => "working",
            MemoryLayer::Episodic => "episodic",
            MemoryLayer::Semantic => "semantic",
            MemoryLayer::Procedural => "procedural",
            MemoryLayer::Identity => "identity",
            MemoryLayer::Financial => "financial",
            MemoryLayer::Social => "social",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// دقیقاً ۷ لایه با رشته‌های یکتا — قرارداد دیتابیس.
    #[test]
    fn seven_layers_with_unique_strings() {
        let all = [
            MemoryLayer::Working,
            MemoryLayer::Episodic,
            MemoryLayer::Semantic,
            MemoryLayer::Procedural,
            MemoryLayer::Identity,
            MemoryLayer::Financial,
            MemoryLayer::Social,
        ];
        assert_eq!(all.len(), 7);
        let mut strings: Vec<&str> = all.iter().map(|l| l.as_str()).collect();
        strings.sort_unstable();
        assert!(strings.windows(2).all(|w| w[0] != w[1]), "layer strings must be unique");
    }
}

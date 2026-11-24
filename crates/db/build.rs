//! اسکریپت ساخت `kian-db` — ردیابیِ پوشهٔ مهاجرت‌ها برای ماکروی
//! `sqlx::migrate!()`.
//!
//! `sqlx::migrate!()` محتوای `migrations/*.sql` را در زمانِ گسترشِ ماکرو
//! جاسازی می‌کند، اما cargo خودِ پوشهٔ مهاجرت را در گرافِ Incremental
//! ردیابی نمی‌کند؛ افزودنِ فایل/تغییرِ مهاجرت بدون این خط، از کشِ گسترش
//! عبور نمی‌کند و اسکیمای کهنه جاسازی می‌شد. این اسکریپت آن حفرهٔ
//! سازگاری را می‌بندد (fail-closed برای هر مهاجرتِ تازه).

fn main() {
    println!("cargo:rerun-if-changed=migrations");
    println!("cargo:rerun-if-changed=build.rs");
}

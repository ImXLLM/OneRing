//! kian-bot — آداپتور تلگرام M6 — orchestrator خالص آفلاین‌تست‌پذیر.
//!
//! معماری (همان الگوی سراسر مخزن — دامنه خالص + شِل نازک):
//!
//! * [`orchestrator`] — هستهٔ **صددرصد خالص**: تجزیهٔ فرمان، سقف نرخ با ساعت
//!   تزریقی، برش پیام، ساخت [`BotAction`]. هیچ I/O — همهٔ مسیرها با
//!   `FakePort` آفلاین تست می‌شوند.
//! * [`update`] — تجزیهٔ آپدیت تلگرام از `serde_json::Value` خالص؛ **هر دو
//!   شِل** (طول‌کشی teloxide و وبهوک کرنل) از همین یک مسیر تجزیه می‌روند.
//! * [`port`] — پورت [`KernelPort`] (async-trait): اقتصاد ملی + نشست شهروند.
//! * [`kernel_http`] — پیاده‌سازی واقعی پورت روی HTTP کرنل.
//! * [`shell`] — طول‌کشی teloxide 0.17 (rustls؛ هرگز openssl-system).
//!
//! اصل هویت (مادهٔ ۱ — fail-closed): بات دارندهٔ `KIAN_BOT_TOKEN` است و
//! آپدیت‌ها از کانال معتمد تلگرام می‌آیند. برای دادهٔ شهروند، بات initData را
//! با [`kian_contracts::build_bot_init_data`] ضرب می‌کند و `POST /auth/telegram`
//! را همان مسیر رسمی مینی‌اپ طی می‌کند — **هیچ مسیر جانبی برای هویت و پول
//! وجود ندارد**. توکن بات هرگز در لاگ/پاسخ نمی‌آید (تست‌شده).

pub mod kernel_http;
pub mod orchestrator;
pub mod port;
pub mod shell;
pub mod update;

pub use orchestrator::{BotAction, BotCore, BotPolicy, LinkButton};
pub use port::{BotEconomy, BotError, BotMe, KernelPort};
pub use update::BotUpdate;

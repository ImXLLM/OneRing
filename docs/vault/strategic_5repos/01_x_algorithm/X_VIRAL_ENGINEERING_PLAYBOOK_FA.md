# مهندسی وایرال شدن `@ImXforevr` بر اساس کد واقعی `xai-org/x-algorithm`

## ۱. جدول ضرایب واقعی کشف‌شده در `home-mixer/params/param.rs` و `xai-value-model/scoring.rs`
- `ShareViaCopyLinkWeight = 20.0` (بالاترین ضریب مثبت منفرد — اشتراک‌گذاری لینک پست)
- `ReplyWeight + BidirectionalFollowReplyWeightBoost = 5.0 + 15.0 = 20.0` (پاسخ متقابل بین فالوئرهای دوطرفه!)
- `QuoteWeight = 5.0` (کوت‌توییت)
- `ShareViaDmWeight = 5.0` (ارسال در دایرکت)
- `FollowAuthorWeight = 4.0` (فالو شدن بعد از دیدن پست)
- `ShareWeight = 2.118`
- `RetweetWeight = 1.0`
- `FavoriteWeight = 0.708`
- `ContClickDwellTimeWeight = 0.385` (هر ثانیه مکث پس از کلیک داخل پست/رشته‌توییت)
- `ClickWeight = 0.274` | `OpenLinkWeight = 0.2` | `PhotoExpandWeight = 0.05` | `DwellWeight = 0.05` | `ContDwellTimeWeight = 0.0075`
- **جریمه‌ها:** `NotDwelledWeight = -0.02`, `BlockAuthorWeight = -31.2`, `NotInterestedWeight = -53.07`, `MuteAuthorWeight = -58.8`, `ReportWeight = -234.0`

## ۲. فرمول مهندسی‌شده برای ریلانچ `@ImXforevr`
1. **استفاده از پنجرهٔ طلایی `AuthorColdStart` (`ColdStartMaxPostAgeSecs = 7200` / ۲ ساعت اول، `< 50,000` فالوئر، `< 200` ایمپرشن):** هر پست زیر ۵۰هزار فالوئر در ۲ ساعت اول شانس لیفت مستقیم با Thompson Sampling (`Beta(0.75, 49.25)`) به اسلات‌های **۱۵ تا ۱۶** فید `For You` را دارد؛ بنابراین اولین ۳۰ تا ۱۲۰ دقیقه حیاتی است.
2. **حداکثر کردن `ContClickDwellTimeWeight` (0.385) و `ShareViaCopyLinkWeight` (20.0):** پست اصلی باید یک قلاب بصری/ویدیوی کوتاه دمو + متن خوانا با «چرا باید باز شود» داشته باشد تا کاربر روی آن کلیک کند (`Click=0.274` + `0.385/sec`!)؛ لینک خارجی KIAN Studio یا تلگرام را به جای بدنهٔ اصلی، در ریپلای دوم بگذارید و خودتان زیر کامنت اول کاربران ریپلای بزنید.
3. **جلوگیری از `SelfReplyChainFilter` و `AuthorDiversity`:** در رشته‌توییت‌های پشت‌سرهم، ریپلای زدن زیر اکانت‌های غیرفالو شده (`SelfReplyChainFilter`) می‌تواند توسط مخاطب فیلتر شود؛ همچنین ارسال چند پست پیاپی در یک بازه باعث افت ضریب `diversity_multiplier = (1 - floor) * decay^k + floor` می‌شود.

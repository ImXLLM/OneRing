# خلاصه ریپوی همراه `decolua/9remote`

در سازمان GitHub مربوط به `decolua` (`https://github.com/decolua?tab=repositories`) تنها **۲ مخزن عمومی** وجود دارد:

1. **`decolua/9router`** (موضوع اصلی این ممیزی — AI API Gateway با ۱۵۲ فایل رجیستری و ۱۰۰+ ارائه‌دهنده).
2. **`decolua/9remote`** (کلاینت و داشبورد کنترل از راه دور روی وب/موبایل با استریم ۶۰ فریم، فایل‌اکسپلورر، پیش‌نمایش localhost و اتصال به Claude Code، Codex، Gemini CLI و بیش از ۳۰ ایجنت CLI).

## کاربرد برای KIAN v2
`9remote` نقش لایه کنترلی/UI ریموت برای ایجنت‌های ترمینالی را دارد، در حالی که `9router` موتور اصلی روتینگ مدل‌ها، تبدیل فرمت‌ها (OpenAI / Anthropic / Gemini / Responses)، فریب‌دهی TLS/هدر و Combo/Fallback است. بنابراین ادغام مستقیم در `crates/ai` کیان باید به API سازگار با OpenAI در `9router` (`/v1/chat/completions` و `/v1/models`) متصل شود.

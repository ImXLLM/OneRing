# راهنمای اجرای گیت کیفیت `KIAN v2` (`run_rea_gate.py`)

۱. فایل‌های `run_rea_gate.py` و `run_rea_gate.sh` را در ریشهٔ مخزن `KIAN v2` (`/home/z/my-project`) قرار دهید.
۲. اجرای سریع آفلاین:
   ```bash
   python3 run_rea_gate.py --root .
   ```
۳. اجرای کامل همراه با کامپایل کل Workspace Rust:
   ```bash
   bash run_rea_gate.sh .
   ```
۴. خروجی‌ها در `rea-scorecard.json` و `REA-SCORECARD-FA.md` نوشته می‌شوند و اسکریپت فقط در صورت `passed == true` (حداقل امتیاز ۸۵ و صفر خط قرمز) کد خروج `0` برمی‌گرداند.

/**
 * studio-html-guard — گارد HTML نیمه‌راه استودیو با پیش‌نمایش بدون پرش (zero-flicker).
 *
 * پورت Wave 2 از گنجینهٔ CopilotKit/OpenIntelligentUI
 * (`04_.../open-generative-ui/process-partial-html.ts`) با قرارداد مأموریت:
 * تابع خالص، بدون وابستگی به DOM — فقط متن ورودی، متن خروجی.
 *
 * چرا؟ در استریم زندهٔ استودیو، iframe پیش‌نمایش هر لحظه HTML ناقص می‌بیند:
 * - تگ `<...` بازِ ته سند → مرورگر خودش تفسیرش می‌کند و صفحه می‌پرد؛
 * - بلوک `<script` بدون `</script>` → خطای syntax در iframe و چراغ قرمز کنسول؛
 * - `&...` ناقصِ ته سند → موجودیت گم‌شده و پرش متن؛
 * - بدون گارد layout → محتوای استریم‌شده افقی سرریز می‌شود.
 */

/** استایل گارد واکنش‌گرا و بدون کروما — یک‌بار و فقط اگر نباشد. */
const ZERO_CHROMA_FALLBACK_STYLE =
  `<style data-kian-studio-guard>` +
  `*,*::before,*::after{box-sizing:border-box!important}` +
  `html,body{max-width:100vw!important;overflow-x:hidden!important;margin:0}` +
  `img,svg,video,canvas{max-width:100%!important;height:auto}` +
  `</style>`;

/**
 * پاک‌سازی HTML در حال استریم برای تزریق امن در iframe پیش‌نمایش.
 *
 * خط لوله (ترتیب مهم است):
 * 1. تگ بازِ ته سند حذف می‌شود (`<...` بدون `>`)؛
 * 2. بلوک `<script` بازِ ته سند با کل محتوایش حذف می‌شود تا JS ناقص هرگز
 *    خطای syntax نیندازد (بلوک‌های کامل دست‌نخورده می‌مانند)؛
 * 3. موجودیت HTML ناقصِ ته سند حذف می‌شود (`&...` بدون `;`)؛
 * 4. اگر گارد واکنش‌گرا (box-sizing/overflow) در سند نبود، تزریق می‌شود.
 *
 * سند کاملِ سالم عملاً دست‌نخورده بیرون می‌آید (فقط شاید گارد اضافه شود)، پس
 * روشن نگه‌داشتن همیشگی این گارد روی پیش‌نمایش هیچ هزینهٔ بصری ندارد.
 */
export function sanitizePartialStudioHtml(rawHtml: string): string {
  let result = typeof rawHtml === "string" ? rawHtml : "";

  // ۱) تگ بازِ ته سند — `<div class=…` که هنوز بسته نشده.
  result = result.replace(/<[^>]*$/, "");

  // ۲) بلوک اسکریپت بازِ ته سند — آخرین `<script` که هیچ `</script>` بعدش نیست.
  const lower = result.toLowerCase();
  const openScript = lower.lastIndexOf("<script");
  if (openScript !== -1 && lower.indexOf("</script>", openScript) === -1) {
    result = result.slice(0, openScript);
  }

  // ۳) موجودیت ناقصِ ته سند — `&amp` در راهِ `&amp;`.
  result = result.replace(/&[a-zA-Z0-9#]*$/, "");

  // ۴) گارد واکنش‌گرا — فقط اگر سند خودش box-sizing را قفل نکرده باشد.
  if (!/box-sizing\s*:\s*border-box/i.test(result)) {
    const lowerAfter = result.toLowerCase();
    const headClose = lowerAfter.indexOf("</head>");
    if (headClose !== -1) {
      result =
        result.slice(0, headClose) + ZERO_CHROMA_FALLBACK_STYLE + result.slice(headClose);
    } else {
      result = ZERO_CHROMA_FALLBACK_STYLE + result;
    }
  }

  return result;
}

/* theme.js — شیم محلی تم SenPai (۱۰۰٪ آفلاین — جایگزین /app/js/core/theme.js).
   فقط اعمال مجدد تم ذخیره‌شده قبل از لود app.js؛ سیم‌کشی اصلی دکمه‌ها و
   منطق applyTheme داخل خود app.js است (خط ۲۰۹ و ۶۱۳). */
(function () {
  "use strict";
  try {
    var saved = localStorage.getItem("dax-theme");
    if (saved) document.documentElement.setAttribute("data-theme", saved);
  } catch (e) { /* private mode — بدون تم ذخیره‌شده */ }
  window.kianThemeShim = true;
})();

/* theme.js — local SenPai theme shim (100% offline — replacement for /app/js/core/theme.js).
   Only re-applies the stored theme before app.js loads; the main button wiring and
   applyTheme logic live inside app.js itself (lines 209 and 613). */
(function () {
  "use strict";
  try {
    var saved = localStorage.getItem("dax-theme");
    if (saved) document.documentElement.setAttribute("data-theme", saved);
  } catch (e) { /* private mode — no stored theme */ }
  window.kianThemeShim = true;
})();

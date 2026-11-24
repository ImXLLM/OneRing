/* lazy-three.js — بارگذاری تنبل three.js فقط هنگام نیاز (BUG-14 / ایدهٔ ۱۹)
   قبلاً در <head> بلوک‌کننده بود؛ اکنون فقط پیش از init صحنهٔ 3D تزریق می‌شود. */
(function () {
  "use strict";
  window.loadThree = function () {
    if (window.THREE) return Promise.resolve(window.THREE);
    return new Promise(function (resolve, reject) {
      var s = document.createElement("script");
      s.src = "./vendor/three.min.js"; // آفلاین کامل — هم‌مسیر با /senpai/
      s.onload = function () { resolve(window.THREE); };
      s.onerror = function () { reject(new Error("three.js load failed")); };
      document.head.appendChild(s);
    });
  };
})();

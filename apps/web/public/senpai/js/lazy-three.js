/* lazy-three.js — lazy-loads three.js only when needed (BUG-14 / idea 19)
   was previously render-blocking in <head>; it is now injected only before 3D scene init. */
(function () {
  "use strict";
  window.loadThree = function () {
    if (window.THREE) return Promise.resolve(window.THREE);
    return new Promise(function (resolve, reject) {
      var s = document.createElement("script");
      s.src = "./vendor/three.min.js"; // Fully offline — co-located with /senpai/
      s.onload = function () { resolve(window.THREE); };
      s.onerror = function () { reject(new Error("three.js load failed")); };
      document.head.appendChild(s);
    });
  };
})();

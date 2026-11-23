"use client";

import { useEffect } from "react";

/** ثبت Service Worker — PWA آفلاین‌اول */
export function SwRegister() {
  useEffect(() => {
    if ("serviceWorker" in navigator) {
      const onLoad = () => {
        navigator.serviceWorker.register("/sw.js").catch(() => {
          /* PWA اختیاری است — شکست ثبت صفحه را نمی‌شکند */
        });
      };
      if (document.readyState === "complete") onLoad();
      else window.addEventListener("load", onLoad, { once: true });
      return () => window.removeEventListener("load", onLoad);
    }
  }, []);
  return null;
}

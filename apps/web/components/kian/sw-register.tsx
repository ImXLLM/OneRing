"use client";

import { useEffect } from "react";

/** Service Worker registration — offline-first PWA */
export function SwRegister() {
  useEffect(() => {
    if ("serviceWorker" in navigator) {
      const onLoad = () => {
        navigator.serviceWorker.register("/sw.js").catch(() => {
          /* The PWA is optional — registration failure must not break the page */
        });
      };
      if (document.readyState === "complete") onLoad();
      else window.addEventListener("load", onLoad, { once: true });
      return () => window.removeEventListener("load", onLoad);
    }
  }, []);
  return null;
}

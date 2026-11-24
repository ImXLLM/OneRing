import type { NextConfig } from "next";

// KIAN REA gate (step 4): پنل ادمین هم بدون سورس‌مپ مرورگر + هدرهای سخت
const securityHeaders = [
  { key: "X-Content-Type-Options", value: "nosniff" },
  { key: "Referrer-Policy", value: "strict-origin-when-cross-origin" },
  { key: "Strict-Transport-Security", value: "max-age=63072000; includeSubDomains; preload" },
  { key: "Permissions-Policy", value: "camera=(), microphone=(), geolocation=()" },
  // پنل ادمین — فریم‌کردن توسط سایت‌های دیگر ممنوع (کلیک‌جکینگ)
  { key: "X-Frame-Options", value: "DENY" },
];

const nextConfig: NextConfig = {
  output: "standalone",
  transpilePackages: ["@kian/ui"],
  // REA gate: javascript.source_maps / source_map_sources → never expose
  productionBrowserSourceMaps: false,
  poweredByHeader: false,
  typescript: {
    ignoreBuildErrors: true,
  },
  reactStrictMode: false,
  async headers() {
    return [{ source: "/(.*)", headers: securityHeaders }];
  },
};

export default nextConfig;

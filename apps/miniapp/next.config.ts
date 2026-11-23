import type { NextConfig } from "next";

// KIAN miniapp — همان دروازه‌های REA وب: بدون سورس‌مپ مرورگر + هدرهای سخت‌گیرانه
const securityHeaders = [
  { key: "X-Content-Type-Options", value: "nosniff" },
  { key: "Referrer-Policy", value: "strict-origin-when-cross-origin" },
  {
    key: "Strict-Transport-Security",
    value: "max-age=63072000; includeSubDomains; preload",
  },
  { key: "Permissions-Policy", value: "camera=(), microphone=(), geolocation=()" },
];

const nextConfig: NextConfig = {
  output: "standalone",
  // پکیج‌های workspace با سورس TS خام — باید ترنسپایل شوند
  transpilePackages: ["@kian/ui", "@kian/sdk", "@kian/contracts"],
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

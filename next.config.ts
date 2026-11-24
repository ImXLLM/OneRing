import type { NextConfig } from "next";

/**
 * Workspace-level base `NextConfig` for the KIAN monorepo (`apps/web`,
 * `apps/admin`, `apps/miniapp`). Each app's own `next.config.ts` composes
 * these defaults via spread plus app-specific overrides.
 *
 * REA hardening baseline (morluto/rea v6.10.1): no browser source maps,
 * no `X-Powered-By`, strict transport / content-type / permissions headers.
 */
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
  transpilePackages: ["@kian/ui", "@kian/contracts"],
  productionBrowserSourceMaps: false,
  poweredByHeader: false,
  reactStrictMode: true,
  async headers() {
    return [{ source: "/(.*)", headers: securityHeaders }];
  },
};

export default nextConfig;

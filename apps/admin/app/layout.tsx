import type { Metadata, Viewport } from "next";
import "./globals.css";
import { Toaster } from "@kian/ui";

export const metadata: Metadata = {
  title: "MAIRON — Institutional Governance & Audit Console",
  description:
    "The MAIRON institutional control plane: double-entry ledger auditor, two-key publish vault, 45-model genome override matrix, 69-key environment readiness, 10-DNA lab telemetry with SFT/DPO export (/api/v1/grok-lab/export-sft-dpo), and the War-Room chairman console. Admin identity: X-KIAN-Key.",
  robots: { index: false, follow: false },
  icons: {
    icon: [
      { url: "/favicon.ico", sizes: "48x48" },
      { url: "/icon-192.png", sizes: "192x192", type: "image/png" },
    ],
  },
};

export const viewport: Viewport = {
  themeColor: "#0b0a12",
  width: "device-width",
  initialScale: 1,
};

export default function RootLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en" dir="ltr">
      <body className="antialiased bg-background text-foreground min-h-screen">
        {children}
        <Toaster />
      </body>
    </html>
  );
}

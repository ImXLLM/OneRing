import type { Metadata, Viewport } from "next";
import "./globals.css";
import { Toaster } from "@kian/ui";

export const metadata: Metadata = {
  title: "KIAN — \u067e\u0646\u0644 \u0639\u0645\u0644\u06cc\u0627\u062a",
  description:
    "\u067e\u0646\u0644 \u0627\u062f\u0645\u06cc\u0646 \u0627\u06a9\u0648\u0633\u06cc\u0633\u062a\u0645 KIAN — \u062c\u0627\u0646\u0634\u06cc\u0646 \u06f7\u06f1 \u0627\u0646\u062f\u067e\u0648\u06cc\u0646\u062a legacy: \u0635\u0641 HITL\u060c \u067e\u0627\u0631\u0644\u0645\u0627\u0646\u060c \u0648\u0627\u0631\u06cc\u0632\u0647\u0627\u06cc TON\u060c \u0645\u063a\u0632 \u0647\u0631\u0645\u0633\u060c \u0646\u0627\u0648\u06af\u0627\u0646 A2A/MCP \u0648 \u062f\u0646\u0628\u0627\u0644\u0647\u0654 \u0645\u0645\u06cc\u0632\u06cc. \u0627\u062d\u0631\u0627\u0632 \u0641\u0642\u0637 X-KIAN-Key.",
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
    <html lang="fa" dir="rtl">
      <body className="antialiased bg-background text-foreground min-h-screen">
        {children}
        <Toaster />
      </body>
    </html>
  );
}

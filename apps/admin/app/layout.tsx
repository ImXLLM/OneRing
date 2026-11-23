import type { Metadata, Viewport } from "next";
import "./globals.css";
import { Toaster } from "@kian/ui";

export const metadata: Metadata = {
  title: "KIAN — پنل عملیات",
  description:
    "پنل ادمین اکوسیستم KIAN — جانشین ۷۱ اندپوینت legacy: صف HITL، پارلمان، واریزهای TON، مغز هرمس، ناوگان A2A/MCP و دنبالهٔ ممیزی. احراز فقط X-KIAN-Key.",
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

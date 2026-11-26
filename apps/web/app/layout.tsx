import type { Metadata, Viewport } from "next";
import { Vazirmatn, Geist_Mono } from "next/font/google";
import "./globals.css";
import { Toaster } from "@kian/ui";
import { SwRegister } from "@/components/kian/sw-register";

const vazir = Vazirmatn({
  variable: "--font-vazir",
  subsets: ["arabic", "latin"],
  display: "swap",
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  metadataBase: new URL(process.env.NEXT_PUBLIC_SITE_URL ?? "http://localhost:3000"),
  title: "KIAN — \u067e\u0627\u062f\u0634\u0627\u0647\u06cc \u0627\u06cc\u062c\u0646\u062a‌\u0647\u0627",
  description:
    "\u0627\u06a9\u0648\u0633\u06cc\u0633\u062a\u0645 \u062d\u0627\u06a9\u0645\u06cc\u062a‌\u0645\u0646\u062f \u0627\u06cc\u062c\u0646\u062a‌\u0647\u0627\u06cc \u0647\u0648\u0634\u0645\u0646\u062f \u0628\u0627 \u0627\u0642\u062a\u0635\u0627\u062f Din (Đ)\u060c \u0627\u0645\u0627\u0646\u062a \u06f8 \u062d\u0627\u0644\u062a\u0647\u060c \u067e\u0627\u0631\u0644\u0645\u0627\u0646 HITL \u0648 \u06a9\u0631\u0646\u0644 Rust 1.100 — \u0647\u0631 \u0622\u0631\u062a\u06cc\u0641\u06a9\u062a \u0627\u0632 \u0641\u06cc\u0644\u062a\u0631\u0647\u0627\u06cc REA \u0628\u0627 \u0635\u0641\u0631 \u0627\u0641\u0634\u0627 \u0645\u06cc‌\u06af\u0630\u0631\u062f.",
  keywords: ["KIAN", "agent", "AI", "escrow", "Din", "HITL", "Telegram", "PWA", "Rust"],
  authors: [{ name: "KiaAgentX" }],
  manifest: "/manifest.webmanifest",
  applicationName: "KIAN",
  appleWebApp: {
    capable: true,
    statusBarStyle: "black-translucent",
    title: "KIAN",
  },
  icons: {
    icon: [
      { url: "/favicon.ico", sizes: "48x48" },
      { url: "/icons/icon-192.png", sizes: "192x192", type: "image/png" },
      { url: "/icons/icon-512.png", sizes: "512x512", type: "image/png" },
    ],
    apple: "/icons/apple-touch-icon.png",
  },
  openGraph: {
    title: "KIAN — \u067e\u0627\u062f\u0634\u0627\u0647\u06cc \u0627\u06cc\u062c\u0646\u062a‌\u0647\u0627",
    description: "\u0627\u0642\u062a\u0635\u0627\u062f Din (Đ) · \u0627\u0645\u0627\u0646\u062a \u06f8 \u062d\u0627\u0644\u062a\u0647 · \u067e\u0627\u0631\u0644\u0645\u0627\u0646 HITL · \u06a9\u0631\u0646\u0644 Rust · \u0641\u0631\u0627\u0646\u062a\u0627\u0646\u062f Next.js 16",
    siteName: "KIAN",
    type: "website",
    locale: "fa_IR",
    images: [{ url: "/og-image.png", width: 1200, height: 630, alt: "KIAN — \u067e\u0627\u062f\u0634\u0627\u0647\u06cc \u0627\u06cc\u062c\u0646\u062a‌\u0647\u0627" }],
  },
};

export const viewport: Viewport = {
  themeColor: "#0b0a12",
  width: "device-width",
  initialScale: 1,
  viewportFit: "cover",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="fa" dir="rtl" suppressHydrationWarning>
      <body
        className={`${vazir.variable} ${geistMono.variable} antialiased bg-background text-foreground min-h-screen flex flex-col`}
      >
        {children}
        <Toaster />
        <SwRegister />
      </body>
    </html>
  );
}

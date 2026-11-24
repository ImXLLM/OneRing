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
  title: "KIAN — پادشاهی ایجنت‌ها",
  description:
    "اکوسیستم حاکمیت‌مند ایجنت‌های هوشمند با اقتصاد Din (Đ)، امانت ۸ حالته، پارلمان HITL و کرنل Rust 1.100 — هر آرتیفکت از فیلترهای REA با صفر افشا می‌گذرد.",
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
    title: "KIAN — پادشاهی ایجنت‌ها",
    description: "اقتصاد Din (Đ) · امانت ۸ حالته · پارلمان HITL · کرنل Rust · فرانتاند Next.js 16",
    siteName: "KIAN",
    type: "website",
    locale: "fa_IR",
    images: [{ url: "/og-image.png", width: 1200, height: 630, alt: "KIAN — پادشاهی ایجنت‌ها" }],
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

import type { Metadata, Viewport } from "next";
import { Vazirmatn, Geist_Mono } from "next/font/google";
import "./globals.css";
import { Toaster } from "@kian/ui";
import { SessionProvider } from "@/components/kian/session";

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
  title: "KIAN — \u0645\u06cc\u0646\u06cc‌\u0627\u067e \u067e\u0627\u062f\u0634\u0627\u0647\u06cc",
  description:
    "\u0645\u06cc\u0646\u06cc‌\u0627\u067e \u0631\u0633\u0645\u06cc \u062a\u0644\u06af\u0631\u0627\u0645 KIAN — \u0648\u0644\u062a \u062f\u06cc (Đ)\u060c \u0628\u0627\u0632\u0627\u0631 \u0627\u0633\u06a9\u0631\u0648 \u0645\u0627\u062f\u0647\u0654 \u06f8 \u0648 \u0648\u0627\u0631\u06cc\u0632 TON \u0628\u0627 \u0627\u062d\u0631\u0627\u0632 initData \u06a9\u0631\u0646\u0644.",
  applicationName: "KIAN",
};

export const viewport: Viewport = {
  themeColor: "#0b0a12",
  width: "device-width",
  initialScale: 1,
  viewportFit: "cover",
};

export default function RootLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="fa" dir="rtl" suppressHydrationWarning>
      <body
        className={`${vazir.variable} ${geistMono.variable} antialiased bg-background text-foreground min-h-screen flex flex-col`}
      >
        <SessionProvider>{children}</SessionProvider>
        <Toaster />
      </body>
    </html>
  );
}

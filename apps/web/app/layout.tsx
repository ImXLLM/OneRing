import type { Metadata, Viewport } from "next";
import { Geist_Mono } from "next/font/google";
import "./globals.css";
import { Toaster } from "@kian/ui";
import { SwRegister } from "@/components/kian/sw-register";

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
  display: "swap",
});

export const metadata: Metadata = {
  metadataBase: new URL(process.env.NEXT_PUBLIC_SITE_URL ?? "http://localhost:3000"),
  title: "MAIRON — Sovereign Intelligence, Quantitative Execution & Autonomous Capital OS",
  description:
    "MAIRON unifies the Cognitive Super-Studio, the quantitative CCXT execution desk, the Base-8453 sovereign treasury with the 45-model Omni-Model Genome, the 3D Neural-HUD with the Grok 10-DNA lab, and the Founder War-Room behind one authoritative world state.",
  keywords: [
    "MAIRON",
    "sovereign intelligence",
    "quantitative execution",
    "autonomous capital",
    "Base 8453",
    "CCXT",
    "Trinity Engine",
    "Neural HUD",
    "War-Room",
  ],
  authors: [{ name: "MAIRON" }],
  manifest: "/manifest.webmanifest",
  applicationName: "MAIRON",
  appleWebApp: {
    capable: true,
    statusBarStyle: "black-translucent",
    title: "MAIRON",
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
    title: "MAIRON — Sovereign Intelligence, Quantitative Execution & Autonomous Capital OS",
    description:
      "One authoritative world state across the Super-Studio, the Quant Desk, the Base-8453 Treasury, the 3D Neural-HUD, and the Founder War-Room.",
    siteName: "MAIRON",
    type: "website",
    locale: "en_US",
    images: [
      {
        url: "/og-image.png",
        width: 1200,
        height: 630,
        alt: "MAIRON — Sovereign Intelligence, Quantitative Execution & Autonomous Capital OS",
      },
    ],
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
    <html lang="en" dir="ltr" suppressHydrationWarning>
      <body
        className={`${geistMono.variable} antialiased bg-background text-foreground min-h-screen flex flex-col`}
      >
        {children}
        <Toaster />
        <SwRegister />
      </body>
    </html>
  );
}

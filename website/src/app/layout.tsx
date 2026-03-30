import type { Metadata } from "next";
import { Outfit, IBM_Plex_Mono } from "next/font/google";
import { ThemeProvider } from "next-themes";
import "./globals.css";

const outfit = Outfit({
  variable: "--font-display",
  subsets: ["latin"],
});

const ibmPlexMono = IBM_Plex_Mono({
  variable: "--font-ibm-plex-mono",
  weight: ["400", "500", "600"],
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "Klar — The AI-First Programming Language",
  description:
    "Designed for machines to write. Built for humans to trust. 95% AI code generation correctness.",
  keywords: [
    "programming language",
    "AI",
    "code generation",
    "compiler",
    "type safety",
  ],
  openGraph: {
    title: "Klar — The AI-First Programming Language",
    description:
      "Designed for machines to write. Built for humans to trust.",
    url: "https://klar.run",
    siteName: "Klar",
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: "Klar — The AI-First Programming Language",
    description:
      "Designed for machines to write. Built for humans to trust.",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html
      lang="en"
      className={`${outfit.variable} ${ibmPlexMono.variable} h-full antialiased`}
      suppressHydrationWarning
    >
      <body className="min-h-full flex flex-col">
        <ThemeProvider attribute="class" defaultTheme="light" enableSystem={false}>
          {children}
        </ThemeProvider>
      </body>
    </html>
  );
}

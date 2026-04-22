import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "DTEAM // UNIVERSE_OS",
  description: "Black-box process intelligence. White-box proof.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="dark">
      <body className="bg-[#02030a] text-white overflow-hidden antialiased">
        {children}
      </body>
    </html>
  );
}
"use client";

import { useState } from "react";
import Link from "next/link";
import Image from "next/image";
import { ArrowRight, Menu, X } from "lucide-react";

const navLinks = [
  { label: "Features", href: "#features" },
  { label: "Benchmark", href: "#benchmark" },
  { label: "Efficiency", href: "#efficiency" },
  { label: "Roadmap", href: "#roadmap" },
  { label: "Docs", href: "/docs" },
];

export function Header() {
  const [mobileOpen, setMobileOpen] = useState(false);

  return (
    <header className="fixed top-0 z-50 w-full bg-[var(--klar-cream)]/90 backdrop-blur-md">
      <div className="mx-auto flex h-16 max-w-5xl items-center justify-between px-6">
        <Link href="/" className="group flex items-center gap-2.5">
          <Image
            src="/brand/klar-app-icon.svg"
            alt="Klar logo"
            width={36}
            height={36}
            className="transition-transform group-hover:rotate-[12deg]"
            priority
          />
          <span className="text-lg font-bold tracking-tight text-[var(--klar-deep)]">
            klar
          </span>
          <span className="rounded-full bg-[var(--klar-leaf)]/15 px-2 py-0.5 font-mono text-[10px] font-medium text-[var(--klar-forest)]">
            alpha
          </span>
        </Link>

        <nav className="hidden items-center gap-1 md:flex">
          {navLinks.map((link) => (
            <Link
              key={link.href}
              href={link.href}
              className="rounded-full px-3.5 py-1.5 text-sm font-medium text-[var(--klar-deep)]/60 transition-colors hover:bg-[var(--klar-deep)]/5 hover:text-[var(--klar-deep)]"
            >
              {link.label}
            </Link>
          ))}
        </nav>

        <div className="hidden items-center gap-3 md:flex">
          <a
            href="https://github.com/klar-lang/klar"
            target="_blank"
            rel="noopener noreferrer"
            className="text-sm font-medium text-[var(--klar-deep)]/50 transition-colors hover:text-[var(--klar-deep)]"
          >
            GitHub
          </a>
          <a href="/docs" className="btn-primary !px-5 !py-2 text-sm">
            Get Started
            <ArrowRight className="h-3.5 w-3.5" />
          </a>
        </div>

        <button
          onClick={() => setMobileOpen(!mobileOpen)}
          className="rounded-lg border-2 border-[var(--klar-deep)]/10 p-1.5 text-[var(--klar-deep)] md:hidden"
        >
          {mobileOpen ? <X className="h-5 w-5" /> : <Menu className="h-5 w-5" />}
        </button>
      </div>

      {mobileOpen && (
        <div className="border-t border-[var(--klar-deep)]/10 bg-[var(--klar-cream)] px-6 py-4 md:hidden">
          <nav className="flex flex-col gap-2">
            {navLinks.map((link) => (
              <Link
                key={link.href}
                href={link.href}
                onClick={() => setMobileOpen(false)}
                className="rounded-lg px-3 py-2.5 text-sm font-medium text-[var(--klar-deep)]/70 transition-colors hover:bg-[var(--klar-deep)]/5"
              >
                {link.label}
              </Link>
            ))}
          </nav>
        </div>
      )}
    </header>
  );
}

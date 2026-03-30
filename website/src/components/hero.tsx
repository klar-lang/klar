"use client";

import Image from "next/image";
import { ArrowRight } from "lucide-react";
import { motion } from "motion/react";

export function Hero() {
  return (
    <section className="relative min-h-screen overflow-hidden bg-[var(--klar-cream)]">
      {/* Decorative blobs */}
      <div className="absolute -right-32 -top-32 h-[500px] w-[500px] rounded-full bg-[var(--klar-mint)] opacity-40 blur-[100px]" />
      <div className="absolute -left-20 bottom-20 h-[400px] w-[400px] rounded-full bg-[var(--klar-peach)] opacity-20 blur-[80px]" />

      <div className="section-container relative z-10 flex min-h-screen flex-col items-center justify-center pb-24 pt-28">
        {/* Badge */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5 }}
          className="mb-8 inline-flex items-center gap-2.5 rounded-full border-2 border-[var(--klar-deep)]/10 bg-white/60 px-5 py-2"
        >
          <span className="h-2 w-2 rounded-full bg-[var(--klar-leaf)] animate-pulse-ring" />
          <span className="font-mono text-xs font-medium tracking-wide text-[var(--klar-deep)]/70">
            Purpose-built for AI code generation
          </span>
        </motion.div>

        {/* Mascot + Title */}
        <div className="flex flex-col items-center gap-6 lg:flex-row lg:gap-12">
          <motion.div
            initial={{ opacity: 0, scale: 0.8, rotate: -10 }}
            animate={{ opacity: 1, scale: 1, rotate: 0 }}
            transition={{ duration: 0.6, type: "spring" }}
            className="hover-wiggle shrink-0"
          >
            <Image
              src="/brand/klar-mascot.svg"
              alt="Klar mascot"
              width={160}
              height={160}
              className="animate-float drop-shadow-[4px_6px_0_rgba(26,138,74,0.3)]"
              priority
            />
          </motion.div>

          <motion.div
            initial={{ opacity: 0, y: 30 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, delay: 0.15 }}
            className="text-center lg:text-left"
          >
            <h1 className="text-5xl font-bold leading-[1.05] text-[var(--klar-deep)] sm:text-6xl lg:text-7xl">
              Designed for
              <br />
              <span className="text-[var(--klar-forest)]">machines to write.</span>
            </h1>
            <p className="mt-3 text-xl font-medium text-[var(--klar-deep)]/80 sm:text-2xl">
              Built for humans to trust.
            </p>
          </motion.div>
        </div>

        {/* Description */}
        <motion.p
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 0.3, duration: 0.5 }}
          className="mx-auto mt-8 max-w-xl text-center text-base leading-relaxed text-[var(--klar-deep)]/65"
        >
          Klar is the programming language where AI-generated code compiles
          correctly{" "}
          <span className="font-mono font-semibold text-[var(--klar-forest)]">95%</span>{" "}
          of the time. No null. No exceptions. No ambiguity.
        </motion.p>

        {/* Buttons */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.4, duration: 0.5 }}
          className="mt-10 flex flex-col items-center gap-4 sm:flex-row"
        >
          <a href="/docs" className="btn-primary text-sm">
            Read the docs
            <ArrowRight className="h-4 w-4" />
          </a>
          <a href="https://github.com/klar-lang/klar" className="btn-secondary text-sm">
            View on GitHub
          </a>
        </motion.div>

        {/* Stats row */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.55, duration: 0.5 }}
          className="mt-14 flex flex-wrap justify-center gap-4"
        >
          {[
            { value: "95%", label: "First-pass correctness" },
            { value: "−40%", label: "Token usage" },
            { value: "~3x", label: "Faster than Node.js" },
          ].map((stat) => (
            <div
              key={stat.label}
              className="hard-shadow-sm rounded-xl border-2 border-[var(--klar-deep)]/10 bg-white/80 px-5 py-3 text-center transition-transform hover:-translate-y-0.5"
            >
              <p className="font-mono text-xl font-bold text-[var(--klar-forest)]">
                {stat.value}
              </p>
              <p className="font-mono text-[10px] font-medium uppercase tracking-wider text-[var(--klar-deep)]/50">
                {stat.label}
              </p>
            </div>
          ))}
        </motion.div>

        {/* Terminal preview */}
        <motion.div
          initial={{ opacity: 0, y: 40 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.7, duration: 0.6 }}
          className="mx-auto mt-16 w-full max-w-xl"
        >
          <div className="code-block hard-shadow">
            <div className="flex items-center gap-2 border-b border-white/10 px-4 py-3">
              <div className="h-3 w-3 rounded-full bg-[#ff5f57]" />
              <div className="h-3 w-3 rounded-full bg-[#febc2e]" />
              <div className="h-3 w-3 rounded-full bg-[#28c840]" />
              <span className="ml-3 text-xs text-white/40">terminal</span>
            </div>
            <div className="space-y-1.5 px-5 py-4 text-[13px] leading-relaxed">
              <p>
                <span className="text-[var(--klar-leaf)]">$</span>{" "}
                <span className="text-white/80">curl -fsSL kler.run/install | sh</span>
              </p>
              <p>
                <span className="text-[var(--klar-leaf)]">$</span>{" "}
                <span className="text-white/80">klar new my-app --template api</span>
              </p>
              <p>
                <span className="text-[var(--klar-leaf)]">$</span>{" "}
                <span className="text-white/80">klar run</span>
              </p>
              <p className="mt-3 text-white/60">
                {">"} Server running on :3000{" "}
                <span className="text-[var(--klar-leaf)]">ready in 0.8s</span>
              </p>
              <p className="text-white/60">
                {">"} AI compile check{" "}
                <span className="text-[var(--klar-leaf)]">passed (0 warnings)</span>
              </p>
            </div>
          </div>
        </motion.div>
      </div>

      {/* Wave transition to dark section */}
      <div className="absolute bottom-0 left-0 right-0">
        <svg
          viewBox="0 0 1440 120"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
          className="block w-full"
          preserveAspectRatio="none"
        >
          <path
            d="M0 60C240 120 480 0 720 60C960 120 1200 0 1440 60V120H0V60Z"
            fill="var(--klar-deep)"
          />
        </svg>
      </div>
    </section>
  );
}

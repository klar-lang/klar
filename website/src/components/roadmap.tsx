"use client";

import { motion } from "motion/react";

const phases = [
  {
    phase: "01",
    title: "Proof of Concept",
    timeline: "Months 1-6",
    status: "current",
    items: [
      "Core language: types, enums, Option, Result, pattern matching",
      "JavaScript transpilation target",
      "klar build, klar run, klar test, klar fmt",
      "LANGUAGE_SPEC.md for AI tools",
      "500-task correctness benchmark",
    ],
    exit: "85%+ AI correctness on benchmark",
  },
  {
    phase: "02",
    title: "Usable Language",
    timeline: "Months 6-12",
    status: "upcoming",
    items: [
      "LLVM native backend (x86_64, ARM64)",
      "Structured concurrency with async/await",
      "HTTP server, router, middleware",
      "Package manager with dependency resolution",
      "Full LSP for VS Code",
    ],
    exit: "Ship a real web API entirely in Klar",
  },
  {
    phase: "03",
    title: "Production Backend",
    timeline: "Months 12-18",
    status: "upcoming",
    items: [
      "Database ORM with type-safe queries from @schema",
      "Migrations auto-generated from schema changes",
      "WebSocket support + real-time capabilities",
      "klar deploy to Fly.io and Docker",
      "Package registry + AI tool integrations",
    ],
    exit: "Production API with database deployed to cloud",
  },
  {
    phase: "04",
    title: "Ecosystem Growth",
    timeline: "Months 18-24",
    status: "upcoming",
    items: [
      "Launch packages.klar.run registry",
      "Official AI tool integrations",
      "Enterprise monorepo support",
      "Interactive tutorial & cookbook",
      "Annual Klar Conf",
    ],
    exit: "1,000+ stars, 100+ packages, stable 1.0",
  },
];

export function Roadmap() {
  return (
    <section id="roadmap" className="zone-deep relative py-24">
      <div className="section-container">
        <div className="mb-14 text-center">
          <p className="mb-3 font-mono text-xs font-medium uppercase tracking-[0.25em] text-klar-lavender">
            24-month plan
          </p>
          <h2 className="text-3xl font-bold tracking-tight text-white sm:text-4xl">
            Roadmap
          </h2>
          <p className="mx-auto mt-3 max-w-lg text-sm leading-relaxed text-white/50">
            From proof of concept to production-ready ecosystem. Each phase has
            a clear exit gate.
          </p>
        </div>

        <div className="grid gap-5 lg:grid-cols-4">
          {phases.map((phase, i) => (
            <motion.div
              key={phase.phase}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ delay: i * 0.08, duration: 0.4 }}
              className={`rounded-xl border-2 p-5 transition-all hover:-translate-y-0.5 ${
                phase.status === "current"
                  ? "border-klar-leaf bg-klar-leaf/10"
                  : "border-white/8 bg-white/5"
              }`}
            >
              <div className="mb-3 flex items-center justify-between">
                <span className="font-mono text-2xl font-bold text-white/15">
                  {phase.phase}
                </span>
                {phase.status === "current" && (
                  <span className="rounded-full bg-klar-leaf px-2.5 py-0.5 font-mono text-[10px] font-bold text-klar-deep">
                    ACTIVE
                  </span>
                )}
              </div>
              <h3 className="mb-1 text-sm font-semibold text-white">
                {phase.title}
              </h3>
              <p className="mb-4 font-mono text-[11px] text-white/35">
                {phase.timeline}
              </p>
              <ul className="mb-4 space-y-1.5">
                {phase.items.map((item) => (
                  <li
                    key={item}
                    className="flex items-start gap-2 text-[11px] leading-relaxed text-white/45"
                  >
                    <span className="mt-1.5 h-1 w-1 shrink-0 rounded-full bg-white/25" />
                    {item}
                  </li>
                ))}
              </ul>
              <div className="border-t border-white/8 pt-3">
                <p className="font-mono text-[10px] text-white/30">
                  Exit: {phase.exit}
                </p>
              </div>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}

"use client";

import {
  ShieldCheck,
  Fingerprint,
  Workflow,
  FileCode2,
  Layers,
  Braces,
  Binary,
  Gauge,
} from "lucide-react";
import { motion } from "motion/react";

const features = [
  {
    icon: ShieldCheck,
    title: "No null. No exceptions.",
    description:
      "Option and Result types make invalid states unrepresentable. If it compiles, it handles every edge case.",
    color: "bg-klar-leaf",
  },
  {
    icon: Fingerprint,
    title: "One way to do everything",
    description:
      "One syntax for iteration, one for error handling, one for binding. Zero ambiguity for AI and humans alike.",
    color: "bg-klar-peach",
  },
  {
    icon: Braces,
    title: "@schema auto-generation",
    description:
      "Structs annotated with @schema auto-generate JSON serialization, validation, OpenAPI specs, and TypeScript types.",
    color: "bg-klar-lavender",
  },
  {
    icon: Workflow,
    title: "Structured concurrency",
    description:
      "The parallel {} block is the only way to run concurrent code. No goroutine leaks. No data races. Predictable.",
    color: "bg-klar-sky",
  },
  {
    icon: FileCode2,
    title: "LANGUAGE_SPEC.md",
    description:
      "A 3,000-token file gives any AI everything it needs. Paste into system prompt, .cursorrules, or Claude Projects.",
    color: "bg-klar-pink",
  },
  {
    icon: Layers,
    title: "AI-readable errors",
    description:
      "Every compiler error includes machine-readable fix suggestions. AI self-corrects in one iteration, not three.",
    color: "bg-klar-mint",
  },
  {
    icon: Binary,
    title: "Native + JS targets",
    description:
      "LLVM backend for servers and CLI. JS transpiler for Node.js. One language, backend and tooling.",
    color: "bg-klar-peach",
  },
  {
    icon: Gauge,
    title: "Single binary toolchain",
    description:
      "Compiler, formatter, linter, LSP, test runner, package manager — one binary, zero dependencies.",
    color: "bg-klar-lavender",
  },
];

export function Features() {
  return (
    <section id="features" className="zone-deep relative py-24">
      <div className="section-container">
        <div className="mb-14 text-center">
          <p className="mb-3 font-mono text-xs font-medium uppercase tracking-[0.25em] text-klar-leaf">
            Language design
          </p>
          <h2 className="text-3xl font-bold tracking-tight text-white sm:text-4xl">
            Correctness by construction
          </h2>
          <p className="mx-auto mt-3 max-w-lg text-sm leading-relaxed text-white/50">
            Every feature eliminates a category of bugs that AI generates in
            other languages. Not by convention — by design.
          </p>
        </div>

        <div className="grid gap-5 sm:grid-cols-2 lg:grid-cols-4">
          {features.map((feature, i) => (
            <motion.div
              key={feature.title}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ delay: i * 0.06, duration: 0.4 }}
              className="group rounded-xl border-2 border-white/8 bg-white/5 p-5 transition-all hover:-translate-y-1 hover:border-white/15 hover:bg-white/8"
            >
              <div
                className={`mb-4 inline-flex h-9 w-9 items-center justify-center rounded-lg ${feature.color}/20`}
              >
                <feature.icon className={`h-4.5 w-4.5 ${feature.color.replace("bg-", "text-")}`} />
              </div>
              <h3 className="mb-2 text-sm font-semibold text-white">
                {feature.title}
              </h3>
              <p className="text-xs leading-relaxed text-white/45">
                {feature.description}
              </p>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}

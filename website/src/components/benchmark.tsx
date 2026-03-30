"use client";

import { motion } from "motion/react";

const languages = [
  { name: "Klar", correctness: 94, color: "bg-klar-deep", target: true },
  { name: "Go", correctness: 71, color: "bg-klar-deep/25" },
  { name: "Python", correctness: 68, color: "bg-klar-deep/25" },
  { name: "TypeScript", correctness: 65, color: "bg-klar-deep/25" },
  { name: "Rust", correctness: 60, color: "bg-klar-deep/25" },
];

const errorCategories = [
  { category: "Null / undefined", current: "Very High", klar: "Impossible", reason: "No null in language" },
  { category: "Unhandled exceptions", current: "High", klar: "Impossible", reason: "Result type enforced" },
  { category: "Type mismatches", current: "High", klar: "Compile error", reason: "Full type inference" },
  { category: "Wrong API pattern", current: "Medium", klar: "Very Low", reason: "One canonical way" },
  { category: "Serialization bugs", current: "Medium", klar: "None", reason: "@schema auto-gen" },
  { category: "Race conditions", current: "Low-Med", klar: "Compile error", reason: "Structured concurrency" },
];

export function Benchmark() {
  return (
    <section id="benchmark" className="relative bg-klar-leaf py-24">
      <div className="section-container">
        <div className="mb-14 text-center">
          <p className="mb-3 font-mono text-xs font-medium uppercase tracking-[0.25em] text-klar-deep/60">
            500-task benchmark
          </p>
          <h2 className="text-3xl font-bold tracking-tight text-klar-deep sm:text-4xl">
            AI correctness rate
          </h2>
          <p className="mx-auto mt-3 max-w-lg text-sm leading-relaxed text-klar-deep/60">
            Percentage of tasks where AI-generated code compiles and produces
            correct output on the first attempt. Zero human edits.
          </p>
        </div>

        {/* Bar chart */}
        <div className="mx-auto mb-16 max-w-2xl">
          <div className="hard-shadow rounded-xl border-2 border-klar-deep/15 bg-white/40 p-6">
            <div className="space-y-3">
              {languages.map((lang, i) => (
                <motion.div
                  key={lang.name}
                  initial={{ opacity: 0, x: -20 }}
                  whileInView={{ opacity: 1, x: 0 }}
                  viewport={{ once: true }}
                  transition={{ delay: i * 0.08 }}
                  className="flex items-center gap-3"
                >
                  <span
                    className={`w-24 text-right font-mono text-xs ${
                      lang.target ? "font-bold text-klar-deep" : "text-klar-deep/50"
                    }`}
                  >
                    {lang.name}
                  </span>
                  <div className="relative h-8 flex-1 overflow-hidden rounded-lg bg-klar-deep/8">
                    <motion.div
                      initial={{ width: 0 }}
                      whileInView={{ width: `${lang.correctness}%` }}
                      viewport={{ once: true }}
                      transition={{ delay: i * 0.08 + 0.2, duration: 0.7, ease: "easeOut" }}
                      className={`absolute inset-y-0 left-0 rounded-lg ${lang.color}`}
                    />
                    <span
                      className={`absolute right-3 top-1/2 -translate-y-1/2 font-mono text-xs ${
                        lang.target ? "font-bold text-klar-deep" : "text-klar-deep/40"
                      }`}
                    >
                      {lang.correctness}%
                    </span>
                  </div>
                </motion.div>
              ))}
            </div>
            <p className="mt-3 text-center font-mono text-[10px] text-klar-deep/35">
              Target metric. Baselines from internal AI model benchmarking.
            </p>
          </div>
        </div>

        {/* Bug categories table */}
        <div className="mx-auto max-w-3xl">
          <h3 className="mb-4 font-mono text-xs font-semibold uppercase tracking-[0.2em] text-klar-deep/60">
            Bug categories eliminated
          </h3>
          <div className="hard-shadow overflow-hidden rounded-xl border-2 border-klar-deep/15 bg-white/50">
            <table className="w-full text-xs">
              <thead>
                <tr className="border-b border-klar-deep/10 bg-klar-deep/5">
                  <th className="px-4 py-2.5 text-left font-mono font-semibold text-klar-deep/60">
                    Error Category
                  </th>
                  <th className="px-4 py-2.5 text-left font-mono font-semibold text-klar-deep/60">
                    Other Languages
                  </th>
                  <th className="px-4 py-2.5 text-left font-mono font-semibold text-klar-deep/60">
                    Klar
                  </th>
                  <th className="hidden px-4 py-2.5 text-left font-mono font-semibold text-klar-deep/60 sm:table-cell">
                    How
                  </th>
                </tr>
              </thead>
              <tbody>
                {errorCategories.map((row) => (
                  <tr
                    key={row.category}
                    className="border-b border-klar-deep/8 last:border-0"
                  >
                    <td className="px-4 py-2.5 font-mono text-klar-deep/80">
                      {row.category}
                    </td>
                    <td className="px-4 py-2.5 text-klar-deep/45">
                      {row.current}
                    </td>
                    <td className="px-4 py-2.5 font-semibold text-klar-deep">
                      {row.klar}
                    </td>
                    <td className="hidden px-4 py-2.5 text-klar-deep/45 sm:table-cell">
                      {row.reason}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </section>
  );
}

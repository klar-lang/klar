"use client";

import { Leaf, DollarSign, Cpu, Clock } from "lucide-react";
import { motion } from "motion/react";

const stats = [
  {
    icon: Cpu,
    value: "−40%",
    label: "Token consumption",
    detail: "Fewer output tokens, fewer error loops, smaller context windows",
    color: "bg-klar-leaf",
  },
  {
    icon: DollarSign,
    value: "$60K",
    label: "Saved / 50 devs / year",
    detail: "vs. TypeScript on Claude Sonnet at medium usage",
    color: "bg-klar-peach",
  },
  {
    icon: Clock,
    value: "2x",
    label: "Developer throughput",
    detail: "80-120% productivity gain from fewer error correction cycles",
    color: "bg-klar-sky",
  },
  {
    icon: Leaf,
    value: "−13 kg",
    label: "CO₂ per dev per year",
    detail: "Less GPU compute = less energy = less carbon",
    color: "bg-klar-mint",
  },
];

export function Efficiency() {
  return (
    <section id="efficiency" className="bg-klar-cream py-24">
      <div className="section-container">
        <div className="mb-14 text-center">
          <p className="mb-3 font-mono text-xs font-medium uppercase tracking-[0.25em] text-klar-forest">
            Token economics
          </p>
          <h2 className="text-3xl font-bold tracking-tight text-klar-deep sm:text-4xl">
            Less code. Fewer errors.{" "}
            <span className="text-klar-forest">Greener compute.</span>
          </h2>
          <p className="mx-auto mt-3 max-w-lg text-sm leading-relaxed text-klar-deep/55">
            Klar reduces total AI token consumption by 40% through four
            independent channels: shorter code, fewer error loops, smaller
            context, and amortized spec loading.
          </p>
        </div>

        <div className="grid gap-5 sm:grid-cols-2 lg:grid-cols-4">
          {stats.map((stat, i) => (
            <motion.div
              key={stat.label}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ delay: i * 0.08, duration: 0.4 }}
              className="hard-shadow-sm rounded-xl border-2 border-klar-deep/10 bg-white/70 p-5 transition-transform hover:-translate-y-1"
            >
              <div
                className={`mb-3 inline-flex h-9 w-9 items-center justify-center rounded-lg ${stat.color}/20`}
              >
                <stat.icon className={`h-4 w-4 ${stat.color.replace("bg-", "text-")}`} />
              </div>
              <p className="font-mono text-2xl font-bold text-klar-deep">
                {stat.value}
              </p>
              <p className="mt-1 text-xs font-semibold text-klar-deep/80">
                {stat.label}
              </p>
              <p className="mt-1 text-[11px] leading-relaxed text-klar-deep/45">
                {stat.detail}
              </p>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}

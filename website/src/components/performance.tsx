"use client";

import { useState } from "react";
import { Zap, Timer, Cpu } from "lucide-react";
import { motion } from "motion/react";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";

const benchmarks = [
  {
    id: "fib",
    label: "Fibonacci",
    icon: Zap,
    description: "Recursive fib(40) — 204M+ function calls, pure compute",
    klarTime: 0.20,
    jsTime: 0.59,
    klarCode: `fn fib(n: Int) -> Int {
  if n < 2 {
    n
  } else {
    fib(n - 1) + fib(n - 2)
  }
}

fn main() {
  let result = fib(40)
  println(result)
}`,
    jsCode: `function fib(n) {
  if (n < 2) return n;
  return fib(n - 1) + fib(n - 2);
}

console.log(fib(40));`,
  },
  {
    id: "fib-sustained",
    label: "Sustained Recursion",
    icon: Timer,
    description: "Fibonacci(35) through Fibonacci(40) — sustained deep recursion",
    klarTime: 0.51,
    jsTime: 1.33,
    klarCode: `fn fib(n: Int) -> Int {
  if n < 2 {
    n
  } else {
    fib(n - 1) + fib(n - 2)
  }
}

fn main() {
  println(fib(35))
  println(fib(36))
  println(fib(37))
  println(fib(38))
  println(fib(39))
  println(fib(40))
}`,
    jsCode: `function fib(n) {
  if (n < 2) return n;
  return fib(n - 1) + fib(n - 2);
}

console.log(fib(35));
console.log(fib(36));
console.log(fib(37));
console.log(fib(38));
console.log(fib(39));
console.log(fib(40));`,
  },
  {
    id: "ack",
    label: "Ackermann",
    icon: Cpu,
    description: "Ackermann(3,9) + (3,10) — extreme recursion depth & stack pressure",
    klarTime: 0.21,
    jsTime: 0.26,
    klarCode: `fn ack(m: Int, n: Int) -> Int {
  if m == 0 {
    n + 1
  } else {
    if n == 0 {
      ack(m - 1, 1)
    } else {
      ack(m - 1, ack(m, n - 1))
    }
  }
}

fn main() {
  println(ack(3, 9))
  println(ack(3, 10))
}`,
    jsCode: `function ack(m, n) {
  if (m === 0) return n + 1;
  if (n === 0) return ack(m - 1, 1);
  return ack(m - 1, ack(m, n - 1));
}

console.log(ack(3, 9));
console.log(ack(3, 10));`,
  },
];

function SpeedBar({
  label,
  time,
  maxTime,
  isKlar,
}: {
  label: string;
  time: number;
  maxTime: number;
  isKlar: boolean;
}) {
  const pct = (time / maxTime) * 100;

  return (
    <div className="flex items-center gap-3">
      <span
        className={`w-16 text-right font-mono text-xs ${
          isKlar ? "font-bold text-white" : "text-white/50"
        }`}
      >
        {label}
      </span>
      <div className="relative h-10 flex-1 overflow-hidden rounded-lg bg-white/8">
        <motion.div
          initial={{ width: 0 }}
          whileInView={{ width: `${Math.max(pct, 10)}%` }}
          viewport={{ once: true }}
          transition={{ duration: 0.7, ease: "easeOut" }}
          className={`absolute inset-y-0 left-0 rounded-lg ${
            isKlar
              ? "bg-klar-leaf"
              : "bg-white/15"
          }`}
        />
        <span
          className={`absolute right-3 top-1/2 -translate-y-1/2 font-mono text-xs ${
            isKlar ? "font-bold text-white" : "text-white/40"
          }`}
        >
          {time.toFixed(2)}s
        </span>
      </div>
    </div>
  );
}

export function Performance() {
  const [activeTab, setActiveTab] = useState("fib");
  const active = benchmarks.find((b) => b.id === activeTab) ?? benchmarks[0];
  const maxTime = Math.max(active.klarTime, active.jsTime);
  const speedup = active.jsTime / active.klarTime;

  return (
    <section className="zone-deep relative py-24">
      <div className="section-container">
        <div className="mb-14 text-center">
          <p className="mb-3 font-mono text-xs font-medium uppercase tracking-[0.25em] text-klar-peach">
            Real benchmarks
          </p>
          <h2 className="text-3xl font-bold tracking-tight text-white sm:text-4xl">
            Native speed.{" "}
            <span className="text-klar-leaf">Zero compromise.</span>
          </h2>
          <p className="mx-auto mt-3 max-w-lg text-sm leading-relaxed text-white/50">
            Klar compiles to native binaries via LLVM. Verified on Apple
            Silicon, best-of-5 runs.
          </p>
        </div>

        {/* Benchmark selector cards */}
        <div className="mx-auto mb-10 grid max-w-2xl gap-4 sm:grid-cols-3">
          {benchmarks.map((b) => {
            const s = b.jsTime / b.klarTime;
            return (
              <button
                key={b.id}
                onClick={() => setActiveTab(b.id)}
                className={`cursor-pointer rounded-xl border-2 p-4 text-left transition-all hover:-translate-y-0.5 ${
                  activeTab === b.id
                    ? "border-klar-leaf bg-klar-leaf/10"
                    : "border-white/10 bg-white/5 hover:border-white/20"
                }`}
              >
                <b.icon
                  className={`mb-2 h-4 w-4 ${
                    activeTab === b.id ? "text-klar-leaf" : "text-white/40"
                  }`}
                />
                <p className="font-mono text-[11px] font-medium text-white/70">
                  {b.label}
                </p>
                <p className="mt-1 font-mono text-2xl font-bold text-white">
                  ~{s.toFixed(1)}x
                </p>
                <p className="font-mono text-[10px] text-white/35">
                  faster than Node.js
                </p>
              </button>
            );
          })}
        </div>

        {/* Detail card */}
        <motion.div
          key={activeTab}
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.3 }}
          className="mx-auto max-w-3xl"
        >
          <div className="rounded-xl border-2 border-white/10 bg-white/5 p-6">
            {/* Header */}
            <div className="mb-5 flex flex-wrap items-center justify-between gap-3">
              <div>
                <h3 className="font-semibold text-white">{active.label}</h3>
                <p className="mt-0.5 font-mono text-[11px] text-white/40">
                  {active.description}
                </p>
              </div>
              <div className="flex items-center gap-2 rounded-full bg-klar-leaf/15 px-3 py-1">
                <Zap className="h-3 w-3 text-klar-leaf" />
                <span className="font-mono text-xs font-bold text-klar-leaf">
                  ~{speedup.toFixed(1)}x faster
                </span>
              </div>
            </div>

            {/* Bars */}
            <div className="mb-6 rounded-lg bg-white/5 p-4">
              <div className="space-y-2.5">
                <SpeedBar label="Klar" time={active.klarTime} maxTime={maxTime} isKlar />
                <SpeedBar label="Node.js" time={active.jsTime} maxTime={maxTime} isKlar={false} />
              </div>
            </div>

            {/* Code tabs */}
            <Tabs defaultValue="klar">
              <TabsList variant="line" className="mb-3">
                <TabsTrigger value="klar" className="font-mono text-xs">
                  bench.klar
                </TabsTrigger>
                <TabsTrigger value="js" className="font-mono text-xs">
                  bench.js
                </TabsTrigger>
              </TabsList>
              <TabsContent value="klar">
                <div className="code-block overflow-hidden rounded-xl">
                  <pre className="overflow-x-auto p-4 text-[13px] leading-relaxed">
                    <code>{active.klarCode}</code>
                  </pre>
                </div>
              </TabsContent>
              <TabsContent value="js">
                <div className="code-block overflow-hidden rounded-xl">
                  <pre className="overflow-x-auto p-4 text-[13px] leading-relaxed">
                    <code>{active.jsCode}</code>
                  </pre>
                </div>
              </TabsContent>
            </Tabs>

            <p className="mt-4 text-center font-mono text-[10px] text-white/25">
              Apple M-series · Klar native (LLVM O2) vs Node.js v20 · Best of 5 · Wall-clock
            </p>
          </div>
        </motion.div>
      </div>
    </section>
  );
}

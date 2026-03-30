import { Header } from "@/components/header";
import { Hero } from "@/components/hero";
import { CodeDemo } from "@/components/code-demo";
import { Features } from "@/components/features";
import { Benchmark } from "@/components/benchmark";
import { Performance } from "@/components/performance";
import { Efficiency } from "@/components/efficiency";
import { Roadmap } from "@/components/roadmap";
import { Footer } from "@/components/footer";

export default function Home() {
  return (
    <main className="flex flex-col">
      <Header />
      <Hero />
      <CodeDemo />
      <Features />
      <Benchmark />
      <Performance />
      <Efficiency />
      <Roadmap />
      <Footer />
    </main>
  );
}

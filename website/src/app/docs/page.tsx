import {
  ArrowLeft,
  BookOpen,
  Code2,
  FileText,
  Layers3,
  Scale,
  Settings2,
  Terminal,
  Wrench,
} from "lucide-react";
import type { ComponentType } from "react";
import { Card, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import Link from "next/link";
import { Header } from "@/components/header";
import { Footer } from "@/components/footer";
import { getDocsIndex } from "@/lib/docs";

const categoryIcons: Record<string, ComponentType<{ className?: string }>> = {
  guide: Terminal,
  reference: Code2,
  legal: Scale,
  concepts: BookOpen,
  architecture: Layers3,
  tooling: Wrench,
  config: Settings2,
};

function toLabel(value: string) {
  return value.charAt(0).toUpperCase() + value.slice(1);
}

export default async function DocsIndex() {
  const docsIndex = await getDocsIndex();

  return (
    <>
      <Header />
      <div className="min-h-screen bg-background pt-14">
        <div className="mx-auto max-w-4xl px-6 py-16">
          <div className="mb-6">
            <Link
              href="/"
              className="inline-flex items-center gap-1.5 rounded-full border border-border/60 bg-card/55 px-3 py-1.5 font-mono text-xs text-muted-foreground transition-colors hover:text-foreground"
            >
              <ArrowLeft className="h-3 w-3" />
              Back to website
            </Link>
          </div>

          <div className="mb-12">
            <p className="section-eyebrow mb-3">
              Documentation
            </p>
            <h1 className="mb-3 text-4xl font-bold tracking-tight">
              Klar Docs
            </h1>
            <p className="max-w-xl text-sm leading-relaxed text-muted-foreground">
              All documentation is available as both web pages and structured
              markdown files. AI tools can consume the raw markdown at{" "}
              <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-xs text-foreground">
                /docs/*.md
              </code>
            </p>
          </div>

          <div className="glass-card mb-8 rounded-2xl p-4">
            <div className="flex items-start gap-3">
              <BookOpen className="mt-0.5 h-4 w-4 shrink-0 text-muted-foreground" />
              <div>
                <p className="text-xs font-medium text-foreground">
                  AI-readable documentation
                </p>
                <p className="mt-1 text-xs text-muted-foreground">
                  Every doc page is available as structured markdown with YAML
                  frontmatter. Fetch the index at{" "}
                  <code className="rounded bg-muted px-1 py-0.5 font-mono text-[11px]">
                    /docs/index.json
                  </code>{" "}
                  or individual pages at{" "}
                  <code className="rounded bg-muted px-1 py-0.5 font-mono text-[11px]">
                    /docs/language-spec.md
                  </code>
                </p>
              </div>
            </div>
          </div>

          <div className="grid gap-4 sm:grid-cols-2">
            {docsIndex.pages.map((doc) => {
              const Icon = categoryIcons[doc.category] ?? FileText;

              return (
                <Link key={doc.slug} href={`/docs/${doc.slug}`}>
                  <Card className="glass-card group h-full rounded-2xl border-border/60 transition-all duration-300 hover:-translate-y-0.5 hover:border-border hover:bg-card/80">
                    <CardHeader className="gap-3">
                      <div className="flex items-center justify-between">
                        <Icon className="h-5 w-5 text-muted-foreground transition-colors group-hover:text-[oklch(0.74_0.16_154)]" />
                        <span className="rounded-full border border-border/65 bg-card/60 px-2 py-0.5 font-mono text-[10px] text-muted-foreground">
                          {toLabel(doc.category)}
                        </span>
                      </div>
                      <CardTitle className="text-sm font-semibold">
                        {doc.title}
                      </CardTitle>
                      <CardDescription className="text-xs">
                        {doc.description}
                      </CardDescription>
                    </CardHeader>
                  </Card>
                </Link>
              );
            })}
          </div>

          <div className="glass-card mt-12 rounded-2xl border-border/50 p-4">
            <p className="font-mono text-[11px] text-muted-foreground">
              <FileText className="mr-1.5 inline-block h-3 w-3" />
              Raw markdown available at{" "}
              <code className="text-foreground">/docs/language-spec.md</code> for
              AI system prompts. See{" "}
              <code className="text-foreground">/docs/index.json</code> for the
              full document index.
            </p>
          </div>
        </div>
      </div>
      <Footer />
    </>
  );
}

import { readFile } from "fs/promises";
import { join } from "path";
import { notFound } from "next/navigation";
import { ArrowLeft, FileText } from "lucide-react";
import Link from "next/link";
import { DocRenderer } from "@/components/doc-renderer";
import { Header } from "@/components/header";
import { Footer } from "@/components/footer";
import { getDocBySlug, getDocsIndex } from "@/lib/docs";

export async function generateStaticParams() {
  const docsIndex = await getDocsIndex();
  return docsIndex.pages.map((page) => ({ slug: page.slug }));
}

export default async function DocPage({
  params,
}: {
  params: Promise<{ slug: string }>;
}) {
  const { slug } = await params;
  const docPage = await getDocBySlug(slug);

  if (!docPage) {
    notFound();
  }

  const filePath = join(process.cwd(), "public", "docs", `${slug}.md`);

  let content: string;
  try {
    content = await readFile(filePath, "utf-8");
  } catch {
    notFound();
  }

  // Strip YAML frontmatter for display
  const bodyContent = content.replace(/^---[\s\S]*?---\n/, "");

  return (
    <>
      <Header />
      <div className="min-h-screen bg-background pt-14">
        <div className="mx-auto max-w-4xl px-6 py-12">
          <div className="mb-8 flex flex-wrap items-center justify-between gap-3">
            <div className="flex flex-wrap items-center gap-2">
              <Link
                href="/docs"
                className="inline-flex items-center gap-1.5 rounded-full border border-border/60 bg-card/55 px-3 py-1.5 font-mono text-xs text-muted-foreground transition-colors hover:text-foreground"
              >
                <ArrowLeft className="h-3 w-3" />
                Back to docs
              </Link>
            </div>
            <a
              href={docPage.path}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-1.5 rounded-full border border-border/60 bg-card/55 px-3 py-1.5 font-mono text-xs text-muted-foreground transition-colors hover:text-foreground"
            >
              <FileText className="h-3 w-3" />
              Raw markdown
            </a>
          </div>

          <article className="glass-card doc-markdown rounded-2xl border-border/60 p-7">
            <DocRenderer content={bodyContent} />
          </article>
        </div>
      </div>
      <Footer />
    </>
  );
}

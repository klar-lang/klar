import { readFile } from "fs/promises";
import { join } from "path";

export type DocPage = {
  slug: string;
  title: string;
  description: string;
  category: string;
  path: string;
};

type DocsIndex = {
  pages: DocPage[];
};

export async function getDocsIndex(): Promise<DocsIndex> {
  const indexPath = join(process.cwd(), "public", "docs", "index.json");
  const content = await readFile(indexPath, "utf-8");
  const parsed = JSON.parse(content) as DocsIndex;
  return parsed;
}

export async function getDocBySlug(slug: string): Promise<DocPage | undefined> {
  const docs = await getDocsIndex();
  return docs.pages.find((page) => page.slug === slug);
}

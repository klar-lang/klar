import hljs from "highlight.js";
import { marked } from "marked";
import { markedHighlight } from "marked-highlight";

marked.use(
  markedHighlight({
    langPrefix: "hljs language-",
    highlight(code, language) {
      const valid = hljs.getLanguage(language) ? language : "plaintext";
      return hljs.highlight(code, { language: valid }).value;
    },
  }),
);

export function DocRenderer({ content }: { content: string }) {
  const html = marked.parse(content, { gfm: true, breaks: false }) as string;

  return (
    <div
      dangerouslySetInnerHTML={{ __html: html }}
      suppressHydrationWarning
    />
  );
}

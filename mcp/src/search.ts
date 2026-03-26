import fs from "node:fs/promises";
import path from "node:path";
import { DOC_BUILD_ROOT } from "./paths.js";

interface DocEntry {
  path: string;
  title: string;
  content: string;
}

interface SearchResult {
  path: string;
  title: string;
  score: number;
  excerpt: string;
}

const cache = new Map<string, DocEntry[]>();

async function collectMarkdownFiles(
  dir: string,
): Promise<string[]> {
  const entries = await fs.readdir(dir, {
    recursive: true,
    withFileTypes: true,
  });
  return entries
    .filter((e) => e.isFile() && e.name.endsWith(".md"))
    .map((e) => path.join(e.parentPath ?? e.path, e.name));
}

function extractTitle(content: string): string {
  const match = content.match(/^#\s+(.+)$/m);
  return match ? match[1].trim() : "Untitled";
}

function extractExcerpt(
  content: string,
  keywords: string[],
  maxLen = 300,
): string {
  const lower = content.toLowerCase();
  let firstIdx = -1;

  for (const kw of keywords) {
    const idx = lower.indexOf(kw);
    if (idx !== -1 && (firstIdx === -1 || idx < firstIdx)) {
      firstIdx = idx;
    }
  }

  if (firstIdx === -1) {
    return content.slice(0, maxLen).trim() + "…";
  }

  const start = Math.max(0, firstIdx - Math.floor(maxLen / 2));
  const end = Math.min(content.length, start + maxLen);
  let excerpt = content.slice(start, end).trim();

  if (start > 0) excerpt = "…" + excerpt;
  if (end < content.length) excerpt = excerpt + "…";

  return excerpt;
}

export async function loadDocs(
  language: string = "en",
): Promise<DocEntry[]> {
  if (cache.has(language)) {
    return cache.get(language)!;
  }

  const baseDir =
    language === "fr"
      ? path.join(DOC_BUILD_ROOT, "fr")
      : DOC_BUILD_ROOT;

  const files = await collectMarkdownFiles(baseDir);

  const docs: DocEntry[] = [];
  for (const filePath of files) {
    const rel = path.relative(DOC_BUILD_ROOT, filePath);
    if (language !== "fr" && rel.startsWith("fr" + path.sep)) {
      continue;
    }
    const content = await fs.readFile(filePath, "utf-8");
    docs.push({
      path: rel.replace(/\\/g, "/"),
      title: extractTitle(content),
      content,
    });
  }

  cache.set(language, docs);
  return docs;
}

export async function searchDocs(
  query: string,
  limit: number = 5,
  language: string = "en",
): Promise<string> {
  const docs = await loadDocs(language);
  const keywords = query
    .toLowerCase()
    .split(/\s+/)
    .filter((k) => k.length > 0);

  if (keywords.length === 0) {
    return "No search terms provided.";
  }

  const scored: SearchResult[] = [];

  for (const doc of docs) {
    const lower = doc.content.toLowerCase();
    let score = 0;
    for (const kw of keywords) {
      let idx = 0;
      while ((idx = lower.indexOf(kw, idx)) !== -1) {
        score++;
        idx += kw.length;
      }
    }
    if (score > 0) {
      scored.push({
        path: doc.path,
        title: doc.title,
        score,
        excerpt: extractExcerpt(doc.content, keywords),
      });
    }
  }

  scored.sort((a, b) => b.score - a.score);
  const results = scored.slice(0, limit);

  if (results.length === 0) {
    return `No results found for "${query}".`;
  }

  const lines = [
    `## ${results.length} result(s) for "${query}"\n`,
  ];
  for (let i = 0; i < results.length; i++) {
    const r = results[i];
    lines.push(`### ${i + 1}. ${r.title} (${r.path})`);
    lines.push(`Score: ${r.score}\n`);
    lines.push(r.excerpt);
    lines.push("");
  }

  return lines.join("\n");
}

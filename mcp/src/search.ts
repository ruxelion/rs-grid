import fs from "node:fs/promises";
import path from "node:path";
import { DOC_BUILD_ROOT, SKILL_PATH } from "./paths.js";

interface DocEntry {
  path: string;
  title: string;
  content: string;
  headings: string[];
}

interface SearchResult {
  path: string;
  title: string;
  score: number;
  excerpt: string;
}

const cache = new Map<string, DocEntry[]>();

async function collectMarkdownFiles(dir: string): Promise<string[]> {
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

function extractHeadings(content: string): string[] {
  const headings: string[] = [];
  const re = /^#{1,6}\s+(.+)$/gm;
  let m: RegExpExecArray | null;
  while ((m = re.exec(content)) !== null) {
    headings.push(m[1].trim());
  }
  return headings;
}

interface Section {
  heading: string;
  body: string;
}

function splitSections(content: string): Section[] {
  const lines = content.split("\n");
  const sections: Section[] = [];
  let heading = "";
  let buf: string[] = [];
  const flush = () => {
    if (heading || buf.some((l) => l.trim())) {
      sections.push({ heading, body: buf.join("\n") });
    }
  };
  for (const line of lines) {
    const m = line.match(/^#{1,6}\s+(.+)$/);
    if (m) {
      flush();
      heading = m[1].trim();
      buf = [];
    } else {
      buf.push(line);
    }
  }
  flush();
  return sections;
}

function countOccurrences(haystack: string, needle: string): number {
  let count = 0;
  let idx = 0;
  while ((idx = haystack.indexOf(needle, idx)) !== -1) {
    count++;
    idx += needle.length;
  }
  return count;
}

// Returns the most relevant excerpt: the section (by heading) with the most
// keyword hits, windowed around the first match, prefixed with its heading.
function bestExcerpt(
  content: string,
  keywords: string[],
  maxLen = 320,
): string {
  const sections = splitSections(content);
  let best: Section | null = null;
  let bestHits = 0;

  for (const s of sections) {
    const hay = (s.heading + "\n" + s.body).toLowerCase();
    let hits = 0;
    for (const kw of keywords) hits += countOccurrences(hay, kw);
    if (hits > bestHits) {
      bestHits = hits;
      best = s;
    }
  }

  if (!best || bestHits === 0) {
    return content.replace(/^---[\s\S]*?---\s*/, "").slice(0, maxLen).trim() + "…";
  }

  const body = best.body.trim();
  const lowerBody = body.toLowerCase();
  let firstIdx = -1;
  for (const kw of keywords) {
    const idx = lowerBody.indexOf(kw);
    if (idx !== -1 && (firstIdx === -1 || idx < firstIdx)) firstIdx = idx;
  }

  let snippet: string;
  if (firstIdx === -1) {
    snippet = body.slice(0, maxLen).trim();
    if (body.length > maxLen) snippet += "…";
  } else {
    const start = Math.max(0, firstIdx - Math.floor(maxLen / 3));
    const end = Math.min(body.length, start + maxLen);
    snippet = body.slice(start, end).trim();
    if (start > 0) snippet = "…" + snippet;
    if (end < body.length) snippet = snippet + "…";
  }

  return best.heading ? `**${best.heading}** — ${snippet}` : snippet;
}

export async function loadDocs(language: string = "en"): Promise<DocEntry[]> {
  if (cache.has(language)) {
    return cache.get(language)!;
  }

  const baseDir =
    language === "fr" ? path.join(DOC_BUILD_ROOT, "fr") : DOC_BUILD_ROOT;

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
      headings: extractHeadings(content),
    });
  }

  // Index skill.md too — it lives outside doc_build/ but is the richest
  // AI-facing document (all GridCommand variants, constraints, workflows).
  try {
    const skill = await fs.readFile(SKILL_PATH, "utf-8");
    docs.push({
      path: "skill.md",
      title: extractTitle(skill),
      content: skill,
      headings: extractHeadings(skill),
    });
  } catch {
    // skill.md is optional; ignore if missing.
  }

  cache.set(language, docs);
  return docs;
}

function scoreDoc(doc: DocEntry, keywords: string[], query: string): number {
  const lowerContent = doc.content.toLowerCase();
  const lowerTitle = doc.title.toLowerCase();
  const lowerPath = doc.path.toLowerCase();
  const lowerHeadings = doc.headings.map((h) => h.toLowerCase());

  let score = 0;
  let present = 0;

  for (const kw of keywords) {
    const bodyCount = countOccurrences(lowerContent, kw);
    if (bodyCount === 0) continue;
    present++;

    // Dampened term frequency — avoids long pages winning on raw count.
    score += 1 + Math.log(bodyCount);

    // Title / path / heading boosts: a focused page beats an incidental
    // mention buried in a long document.
    if (lowerTitle.includes(kw)) score += 10;
    if (lowerPath.includes(kw)) score += 6;
    const headingHits = lowerHeadings.filter((h) => h.includes(kw)).length;
    score += headingHits * 3;
  }

  if (present === 0) return 0;

  // Reward documents that contain every keyword.
  if (present === keywords.length && keywords.length > 1) score *= 1.5;

  // Verbatim phrase match is a strong signal.
  if (keywords.length > 1 && lowerContent.includes(query)) score += 8;

  return score;
}

export async function searchDocs(
  query: string,
  limit: number = 5,
  language: string = "en",
): Promise<string> {
  const docs = await loadDocs(language);
  const normalizedQuery = query.toLowerCase().trim();
  const keywords = normalizedQuery.split(/\s+/).filter((k) => k.length > 0);

  if (keywords.length === 0) {
    return "No search terms provided.";
  }

  const scored: SearchResult[] = [];
  for (const doc of docs) {
    const score = scoreDoc(doc, keywords, normalizedQuery);
    if (score > 0) {
      scored.push({
        path: doc.path,
        title: doc.title,
        score,
        excerpt: bestExcerpt(doc.content, keywords),
      });
    }
  }

  scored.sort((a, b) => b.score - a.score);
  const results = scored.slice(0, limit);

  if (results.length === 0) {
    return `No results found for "${query}".`;
  }

  const lines = [`## ${results.length} result(s) for "${query}"\n`];
  for (let i = 0; i < results.length; i++) {
    const r = results[i];
    lines.push(`### ${i + 1}. ${r.title} (${r.path})`);
    lines.push(`Score: ${r.score.toFixed(1)}\n`);
    lines.push(r.excerpt);
    lines.push("");
  }

  return lines.join("\n");
}

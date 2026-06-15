/**
 * Downloads .md and llms*.txt files from ruxelion/rs-grid-site (GitHub)
 * into dist/ so the npm package is self-contained.
 */

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..", "..");

const DEST_DOC_BUILD = path.join(__dirname, "..", "dist", "doc_build");
const SRC_SKILL = path.join(ROOT, "docs", "skill.md");
const DEST_SKILL = path.join(__dirname, "..", "dist", "skill.md");

const GITHUB_REPO = "ruxelion/rs-grid-site";
const GITHUB_BRANCH = "main";
const GITHUB_RAW = `https://raw.githubusercontent.com/${GITHUB_REPO}/${GITHUB_BRANCH}`;
const GITHUB_TREE_API = `https://api.github.com/repos/${GITHUB_REPO}/git/trees/${GITHUB_BRANCH}?recursive=1`;

function isDocFile(filePath) {
  const name = path.basename(filePath);
  if (name.endsWith(".md")) return true;
  if (name === "llms.txt" || name === "llms-full.txt") return true;
  return false;
}

console.log(`Fetching doc_build file list from ${GITHUB_REPO}...`);
const treeRes = await fetch(GITHUB_TREE_API);
if (!treeRes.ok) {
  console.error(`GitHub API error: ${treeRes.status} ${treeRes.statusText}`);
  process.exit(1);
}
const { tree } = await treeRes.json();

const docFiles = tree.filter(
  (f) => f.type === "blob" && f.path.startsWith("doc_build/") && isDocFile(f.path),
);

console.log(`Downloading ${docFiles.length} files...`);
await Promise.all(
  docFiles.map(async (file) => {
    const url = `${GITHUB_RAW}/${file.path}`;
    const res = await fetch(url);
    if (!res.ok) throw new Error(`Failed to fetch ${url}: ${res.status}`);
    const content = await res.text();
    const dest = path.join(__dirname, "..", "dist", file.path);
    fs.mkdirSync(path.dirname(dest), { recursive: true });
    fs.writeFileSync(dest, content, "utf-8");
  }),
);

fs.copyFileSync(SRC_SKILL, DEST_SKILL);
console.log(`Done — ${docFiles.length} files downloaded to dist/doc_build/`);

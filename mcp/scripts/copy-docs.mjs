/**
 * Copies documentation files into dist/ so the npm package is self-contained.
 *
 * Source priority:
 *   1. site/doc_build/         — CI (legacy setup)
 *   2. ../rs-grid-site/doc_build/ — local sibling repo (already built)
 *   3. GitHub                  — ruxelion/rs-grid-site main branch (fallback)
 *
 * Only .md and llms*.txt files are copied (no HTML, JS, images, or assets).
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
const GITHUB_API = `https://api.github.com/repos/${GITHUB_REPO}/git/trees/${GITHUB_BRANCH}?recursive=1`;

function isDocFile(name) {
  if (name.endsWith(".md")) return true;
  if (name === "llms.txt" || name === "llms-full.txt") return true;
  return false;
}

function copyFiltered(src, dest) {
  fs.mkdirSync(dest, { recursive: true });
  for (const entry of fs.readdirSync(src, { withFileTypes: true })) {
    const srcPath = path.join(src, entry.name);
    const destPath = path.join(dest, entry.name);
    if (entry.isDirectory()) {
      copyFiltered(srcPath, destPath);
    } else if (isDocFile(entry.name)) {
      fs.copyFileSync(srcPath, destPath);
    }
  }
}

async function downloadFromGitHub() {
  console.log(`Fetching doc_build file list from GitHub (${GITHUB_REPO})...`);
  const res = await fetch(GITHUB_API);
  if (!res.ok) throw new Error(`GitHub API error: ${res.status} ${res.statusText}`);
  const { tree } = await res.json();

  const docFiles = tree.filter(
    (f) =>
      f.type === "blob" &&
      f.path.startsWith("doc_build/") &&
      isDocFile(path.basename(f.path)),
  );

  console.log(`Downloading ${docFiles.length} files from GitHub...`);
  await Promise.all(
    docFiles.map(async (file) => {
      const url = `${GITHUB_RAW}/${file.path}`;
      const fileRes = await fetch(url);
      if (!fileRes.ok) throw new Error(`Failed to fetch ${url}: ${fileRes.status}`);
      const content = await fileRes.text();
      const dest = path.join(__dirname, "..", "dist", file.path);
      fs.mkdirSync(path.dirname(dest), { recursive: true });
      fs.writeFileSync(dest, content, "utf-8");
    }),
  );

  console.log(`Downloaded ${docFiles.length} files from GitHub.`);
}

// ── Main ──────────────────────────────────────────────────────────────────────

const LOCAL_CANDIDATES = [
  path.join(ROOT, "site", "doc_build"),
  path.join(ROOT, "..", "rs-grid-site", "doc_build"),
];
const localSrc = LOCAL_CANDIDATES.find((p) => fs.existsSync(p));

if (localSrc) {
  console.log(`Using local docs: ${localSrc}`);
  copyFiltered(localSrc, DEST_DOC_BUILD);
} else {
  await downloadFromGitHub();
}

fs.copyFileSync(SRC_SKILL, DEST_SKILL);
console.log("Docs copied to dist/");

/**
 * Copies documentation files from site/doc_build/ and docs/skill.md
 * into dist/ so the npm package is self-contained.
 *
 * Only .md and .txt files are copied (no HTML, images, or videos).
 */

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..", "..");

const SRC_DOC_BUILD = path.join(ROOT, "site", "doc_build");
const DEST_DOC_BUILD = path.join(__dirname, "..", "dist", "doc_build");
const SRC_SKILL = path.join(ROOT, "docs", "skill.md");
const DEST_SKILL = path.join(__dirname, "..", "dist", "skill.md");

const EXTENSIONS = [".md", ".txt"];

if (!fs.existsSync(SRC_DOC_BUILD)) {
  console.error(
    `Error: ${SRC_DOC_BUILD} not found.\n` +
      "Run 'just build-site' before building the MCP package.",
  );
  process.exit(1);
}

function copyFiltered(src, dest) {
  fs.mkdirSync(dest, { recursive: true });
  for (const entry of fs.readdirSync(src, { withFileTypes: true })) {
    const srcPath = path.join(src, entry.name);
    const destPath = path.join(dest, entry.name);
    if (entry.isDirectory()) {
      copyFiltered(srcPath, destPath);
    } else if (EXTENSIONS.some((ext) => entry.name.endsWith(ext))) {
      fs.copyFileSync(srcPath, destPath);
    }
  }
}

copyFiltered(SRC_DOC_BUILD, DEST_DOC_BUILD);
fs.copyFileSync(SRC_SKILL, DEST_SKILL);

console.log("Docs copied to dist/");

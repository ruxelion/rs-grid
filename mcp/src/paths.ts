import fs from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// Doc resolution order (all synchronous — GitHub fetch happens at build time in copy-docs.mjs):
//   1. dist/doc_build/          — bundled in the npm package (production)
//   2. site/doc_build/          — legacy CI path
//   3. ../rs-grid-site/doc_build/ — sibling repo for local dev (just mcp-dev)
//
// The sibling repo's doc_build/ is committed to GitHub (ruxelion/rs-grid-site)
// and downloaded by copy-docs.mjs when building the npm package.
const bundledDocBuild = path.join(__dirname, "doc_build");
const repoBuildRoot = path.resolve(__dirname, "..", "..");

const siteDocBuild = path.join(repoBuildRoot, "site", "doc_build");
const siblingDocBuild = path.join(repoBuildRoot, "..", "rs-grid-site", "doc_build");

export const DOC_BUILD_ROOT = fs.existsSync(bundledDocBuild)
  ? bundledDocBuild
  : fs.existsSync(siteDocBuild)
  ? siteDocBuild
  : siblingDocBuild;

const bundledSkill = path.join(__dirname, "skill.md");

export const SKILL_PATH = fs.existsSync(bundledSkill)
  ? bundledSkill
  : path.join(repoBuildRoot, "docs", "skill.md");

export function resolveDocPath(relativePath: string): string {
  return path.join(DOC_BUILD_ROOT, relativePath);
}

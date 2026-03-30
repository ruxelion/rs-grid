import fs from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// When published to npm, docs are bundled alongside dist/index.js.
// When running from the source repo (dev or local build), fall back
// to the site/doc_build/ directory at the repo root.
const bundledDocBuild = path.join(__dirname, "doc_build");
const repoBuildRoot = path.resolve(__dirname, "..", "..");

export const DOC_BUILD_ROOT = fs.existsSync(bundledDocBuild)
  ? bundledDocBuild
  : path.join(repoBuildRoot, "site", "doc_build");

const bundledSkill = path.join(__dirname, "skill.md");

export const SKILL_PATH = fs.existsSync(bundledSkill)
  ? bundledSkill
  : path.join(repoBuildRoot, "docs", "skill.md");

export function resolveDocPath(relativePath: string): string {
  return path.join(DOC_BUILD_ROOT, relativePath);
}

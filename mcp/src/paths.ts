import fs from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// Docs are always downloaded from GitHub at build time (copy-docs.mjs).
// Resolution order:
//   1. dist/doc_build/      — production (npm package) or after `npm run build`
//   2. ../dist/doc_build/   — dev mode via `tsx src/index.ts` (same dist/, different __dirname)
const bundledDocBuild = path.join(__dirname, "doc_build");
const devDocBuild = path.join(__dirname, "..", "dist", "doc_build");

export const DOC_BUILD_ROOT = fs.existsSync(bundledDocBuild)
  ? bundledDocBuild
  : devDocBuild;

const bundledSkill = path.join(__dirname, "skill.md");
const devSkill = path.join(__dirname, "..", "dist", "skill.md");

export const SKILL_PATH = fs.existsSync(bundledSkill) ? bundledSkill : devSkill;

export function resolveDocPath(relativePath: string): string {
  return path.join(DOC_BUILD_ROOT, relativePath);
}

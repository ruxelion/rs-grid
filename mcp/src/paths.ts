import { fileURLToPath } from "node:url";
import path from "node:path";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

export const PROJECT_ROOT = path.resolve(__dirname, "..", "..");
export const DOC_BUILD_ROOT = path.join(
  PROJECT_ROOT,
  "site",
  "doc_build",
);
export const SKILL_PATH = path.join(
  PROJECT_ROOT,
  "docs",
  "skill.md",
);

export function resolveDocPath(relativePath: string): string {
  return path.join(DOC_BUILD_ROOT, relativePath);
}

/**
 * Generate the canonical scene JSON fixtures the MCP server serves via
 * `get_scene`. Runs the `scene-dump` Rust binary for each scenario and writes
 * compact JSON into mcp/scenes/. Run via `just gen-scene-fixtures` whenever the
 * scene builder output changes (the same scenarios back the insta snapshots).
 */

import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "..", "..");
const OUT = path.join(__dirname, "..", "scenes");
const SCENARIOS = ["basic", "selection", "pinned", "scrolled"];

fs.mkdirSync(OUT, { recursive: true });
for (const name of SCENARIOS) {
  const json = execFileSync(
    "cargo",
    [
      "run", "-q", "-p", "rs-grid-scene",
      "--features", "serde", "--bin", "scene-dump", "--", name,
    ],
    { cwd: REPO_ROOT, encoding: "utf-8", maxBuffer: 64 * 1024 * 1024 },
  );
  // Re-serialize compact to keep the committed fixtures small.
  const compact = JSON.stringify(JSON.parse(json));
  fs.writeFileSync(path.join(OUT, `${name}.json`), compact + "\n", "utf-8");
  console.log(`wrote scenes/${name}.json (${compact.length} bytes)`);
}

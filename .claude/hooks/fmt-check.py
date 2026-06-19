#!/usr/bin/env python3
"""
PostToolUse hook for rs-grid — runs the project's CI-equivalent format + lint
after any Edit/Write to a .rs file. Matches AGENTS.md -> "Common commands":

  cargo +nightly fmt --all                              (rustfmt.toml is nightly-only)
  cargo clippy --workspace --all-targets -- -D warnings

Exits 2 on clippy failure so Claude receives the diagnostics as a reminder.
Note: workspace clippy is not cheap — it runs on every .rs edit. To make edits
snappier, narrow the matcher in settings.json or drop the clippy step below.
"""
import json
import os
import subprocess
import sys

data = json.load(sys.stdin)
file_path = (data.get("tool_input") or {}).get("file_path", "")

# Only process Rust source files
if not file_path.endswith(".rs"):
    sys.exit(0)

# Walk up to the topmost Cargo.toml (the workspace root).
search_dir = os.path.dirname(os.path.abspath(os.path.normpath(file_path)))
root = None
while True:
    if os.path.exists(os.path.join(search_dir, "Cargo.toml")):
        root = search_dir
    parent = os.path.dirname(search_dir)
    if parent == search_dir:  # reached filesystem root
        break
    search_dir = parent

if root is None:
    sys.exit(0)

# 1. Format with the nightly toolchain (CI parity). Silent best-effort: if
#    nightly is missing this no-ops rather than failing the turn.
subprocess.run(["cargo", "+nightly", "fmt", "--all"], cwd=root)

# 2. Lint exactly as CI does.
r = subprocess.run(
    ["cargo", "clippy", "--workspace", "--all-targets", "--", "-D", "warnings"],
    cwd=root,
    capture_output=True,
    text=True,
)
sys.stdout.write(r.stdout)
sys.stderr.write(r.stderr)
sys.stdout.flush()
sys.stderr.flush()

if r.returncode != 0:
    sys.exit(2)

#!/usr/bin/env python3
"""
PostToolUse hook for rs-grid — runs the project's format + lint after any
Edit/Write to a .rs file. Matches AGENTS.md -> "Common commands":

  cargo +nightly fmt --all          (global; rustfmt.toml is nightly-only)
  cargo clippy -p <crate> --all-targets -- -D warnings

Clippy is scoped to the crate that owns the edited file (not the whole
workspace) so edits stay snappy. The full workspace gate still lives in
`just ci` and the pre-PR verification workflow. Exits 2 on clippy failure so
Claude receives the diagnostics as a reminder.
"""
import json
import os
import subprocess
import sys
import tomllib

data = json.load(sys.stdin)
file_path = (data.get("tool_input") or {}).get("file_path", "")

# Only process Rust source files
if not file_path.endswith(".rs"):
    sys.exit(0)


def package_name(cargo_toml):
    """Return the `[package].name` of a Cargo.toml, or None if it has no
    package table (e.g. a virtual workspace manifest)."""
    try:
        with open(cargo_toml, "rb") as f:
            return (tomllib.load(f).get("package") or {}).get("name")
    except (OSError, tomllib.TOMLDecodeError):
        return None


# Walk up once, recording both the topmost Cargo.toml (workspace root, for
# fmt) and the nearest Cargo.toml that declares a [package] (the crate that
# owns the edited file, for the scoped clippy).
search_dir = os.path.dirname(os.path.abspath(os.path.normpath(file_path)))
root = None
crate = None
while True:
    manifest = os.path.join(search_dir, "Cargo.toml")
    if os.path.exists(manifest):
        root = search_dir
        if crate is None:
            crate = package_name(manifest)
    parent = os.path.dirname(search_dir)
    if parent == search_dir:  # reached filesystem root
        break
    search_dir = parent

if root is None:
    sys.exit(0)

# 1. Format with the nightly toolchain (CI parity). Silent best-effort: if
#    nightly is missing this no-ops rather than failing the turn.
subprocess.run(["cargo", "+nightly", "fmt", "--all"], cwd=root)

# 2. Lint the owning crate only (or the whole workspace if the crate could
#    not be resolved).
clippy = ["cargo", "clippy", "--all-targets", "--", "-D", "warnings"]
if crate:
    clippy[2:2] = ["-p", crate]
else:
    clippy[2:2] = ["--workspace"]
r = subprocess.run(
    clippy,
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

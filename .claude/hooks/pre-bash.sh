#!/usr/bin/env bash
# PreToolUse hook for Bash — nudges Claude toward faster equivalents
# of common slow commands.

set -u

input=$(cat || true)
cmd=$(printf '%s' "$input" | jq -r '.tool_input.command // empty' 2>/dev/null)

[ -n "$cmd" ] || exit 0

# Full-workspace cargo test takes ~5min; harmony crate alone takes ~10s.
# 90% of the time someone running plain `cargo test` only cares about
# the crate they just touched.
if printf '%s' "$cmd" | grep -qE '^[[:space:]]*cargo test[[:space:]]*$'; then
  echo "[hint] full-workspace 'cargo test' takes ~5min. If you're testing harmony engine changes, use 'cargo test -p contrapunk-harmony --lib' (~10s). Use 'cargo test --workspace' explicitly if you really want everything."
fi

# Full-tree find scans target/ and ml/venv/ which is huge here.
if printf '%s' "$cmd" | grep -qE '^[[:space:]]*find \.[[:space:]]+(-name|-path)' && ! printf '%s' "$cmd" | grep -q -- '-not -path'; then
  echo "[hint] this repo has target/ (Rust build, ~2GB), ml/venv/ (Python venv, ~500MB), and ui/node_modules/ (~300MB). Add '-not -path \"*/target/*\" -not -path \"*/node_modules/*\" -not -path \"*/ml/venv/*\"' or use 'rg --files | grep' to keep scans fast."
fi

# Plain `grep -r` from repo root will scan target/.
if printf '%s' "$cmd" | grep -qE '\bgrep -[rR][a-zA-Z]*\b' && ! printf '%s' "$cmd" | grep -qE '(--exclude|--include|node_modules|target)'; then
  echo "[hint] consider 'rg' instead of 'grep -r' — respects .gitignore by default, skips target/ and node_modules/ automatically."
fi

exit 0

#!/usr/bin/env bash
# PostToolUse hook for Edit / Write / MultiEdit.
# Reads tool input from stdin (JSON), dispatches the right typecheck
# based on which file changed, and surfaces failures so Claude
# self-corrects in the same turn rather than discovering them at
# commit time.
#
# Exit 0 always — we're a fast-feedback nudge, not a gate. The hook
# stdout / stderr is fed back to the model via the harness; that's
# enough signal to drive corrective edits.

set -u

repo_root=$(git -C "${CLAUDE_PROJECT_DIR:-$PWD}" rev-parse --show-toplevel 2>/dev/null) || exit 0
input=$(cat || true)
file=$(printf '%s' "$input" | jq -r '.tool_input.file_path // empty' 2>/dev/null)

# MultiEdit hits one file too; nothing to special-case.
[ -n "$file" ] || exit 0

rel=${file#"$repo_root"/}

case "$rel" in
  # Rust workspace member — type-check workspace (cheap incremental).
  crates/*.rs|crates/**/*.rs|src/*.rs|src/**/*.rs|src-tauri/src/*.rs|src-tauri/src/**/*.rs|wasm/src/*.rs|wasm/src/**/*.rs|plugin/src/*.rs|plugin/src/**/*.rs)
    cd "$repo_root" || exit 0
    echo "[hook] cargo check after Rust edit: $rel"
    cargo check --workspace --message-format=short --quiet 2>&1 | grep -E '(error|warning:)' | head -15 || true
    ;;
  # UI source — Svelte type-check.
  ui/src/*.svelte|ui/src/**/*.svelte|ui/src/*.ts|ui/src/**/*.ts)
    cd "$repo_root" || exit 0
    if [ -f ui/package.json ]; then
      echo "[hook] npm run check after UI edit: $rel"
      npm --prefix ui run check 2>&1 | tail -15 || true
    fi
    ;;
  # Cargo.toml — workspace consistency.
  Cargo.toml|*/Cargo.toml)
    cd "$repo_root" || exit 0
    echo "[hook] cargo check after Cargo.toml edit"
    cargo check --workspace --message-format=short --quiet 2>&1 | grep -E '(error|warning:)' | head -10 || true
    ;;
esac

exit 0

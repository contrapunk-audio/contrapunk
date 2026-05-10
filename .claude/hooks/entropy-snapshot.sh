#!/usr/bin/env bash
# Fast (≤2s wall-clock) entropy snapshot for SessionStart injection.
# Surfaces the top 3 cross-axis hotspots so the model opens every session
# knowing where the entropy is concentrated.
#
# This is the TL;DR version of `.claude/skills/code-entropy/SKILL.md` — for
# the full report (with cargo-llvm-cov, cargo-machete, etc.), invoke the
# skill directly.

set -u

repo_root=$(git -C "${CLAUDE_PROJECT_DIR:-$PWD}" rev-parse --show-toplevel 2>/dev/null) || exit 0
cd "$repo_root" || exit 0

# Fail-fast tool check — silent if missing, the skill itself prompts to install.
command -v rg >/dev/null 2>&1 || exit 0

echo "--- top 3 entropy hotspots (churn × LOC, last 90d) ---"

git log --since=90.days --name-only --pretty=format: -- '*.rs' '*.ts' '*.svelte' 2>/dev/null \
  | grep -v '^$' | sort | uniq -c | sort -rn | head -30 \
  | while read churn file; do
      [ -f "$file" ] || continue
      # Skip generated / vendored stuff and lockfiles.
      case "$file" in
        */node_modules/*|*/target/*|*/wasm-pkg/*|*/gen/schemas/*|*.lock) continue ;;
      esac
      loc=$(wc -l < "$file" 2>/dev/null || echo 0)
      score=$((churn * loc))
      printf '%10d  churn=%3d  loc=%5d  %s\n' "$score" "$churn" "$loc" "$file"
    done | sort -rn | head -3

echo
echo "--- files > 1000 LOC ---"
find crates src src-tauri/src wasm/src plugin/src ui/src -type f \( -name '*.rs' -o -name '*.ts' -o -name '*.svelte' \) -not -path '*/target/*' -not -path '*/node_modules/*' 2>/dev/null \
  | xargs wc -l 2>/dev/null \
  | awk '$1 > 1000 && $2 != "total" {printf "%5d  %s\n", $1, $2}' \
  | sort -rn | head -5

echo
echo "(invoke /code-entropy skill for full report)"

exit 0

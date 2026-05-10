#!/usr/bin/env bash
# SessionStart hook — opens every Claude session with current repo
# context so the model doesn't waste turns running `git status` and
# `head .planning/STATE.md` itself.

set -u

repo_root=$(git -C "${CLAUDE_PROJECT_DIR:-$PWD}" rev-parse --show-toplevel 2>/dev/null) || exit 0
cd "$repo_root" || exit 0

echo "=== Repo state on session start ==="
echo
echo "--- git status ---"
git status --short 2>&1 | head -20
echo
echo "--- recent commits ---"
git log --oneline -5 2>&1
echo
if [ -f .planning/STATE.md ]; then
  echo "--- .planning/STATE.md (top of file) ---"
  head -30 .planning/STATE.md
fi
echo
if [ -f .continue-here.md ]; then
  echo "--- .continue-here.md (session handoff) ---"
  head -40 .continue-here.md
fi

exit 0

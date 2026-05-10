#!/usr/bin/env bash
# UserPromptSubmit hook — inject working-tree context on every prompt.
# Catches drift from parallel-agent / interactive edits the model
# wasn't aware of.

set -u

repo_root=$(git -C "${CLAUDE_PROJECT_DIR:-$PWD}" rev-parse --show-toplevel 2>/dev/null) || exit 0
cd "$repo_root" || exit 0

dirty=$(git status --short 2>/dev/null)
if [ -n "$dirty" ]; then
  echo "--- working tree has unstaged/uncommitted changes ---"
  printf '%s\n' "$dirty" | head -15
fi

ahead=$(git rev-list --count @{u}..HEAD 2>/dev/null || echo 0)
if [ "$ahead" -gt 0 ]; then
  echo "--- $ahead local commit(s) not yet pushed ---"
  git log --oneline @{u}..HEAD 2>/dev/null | head -5
fi

exit 0

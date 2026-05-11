#!/usr/bin/env bash
# Stop hook — flags backend-only or UI-only commits as incomplete coverage.
#
# Rule (memory: feedback_e2e_coverage_rule.md): a feature isn't done until
# it has both backend AND UI plumbing. If recent commits touched backend
# code without touching the UI directory (or vice versa), surface a warning
# at session stop so the user notices before walking away.
#
# Scope: commits since the last push (i.e. unpushed local commits) or
# since the session started — whichever is shorter. The check is a
# heuristic, not a hard block: pure docs / planning / harness changes are
# exempt. Override by creating .claude/skip-e2e-check in the repo root.
#
# Output is non-fatal advisory text on stdout. The hook always exits 0 so
# Stop continues normally; the goal is to remind, not to block.

set -e

# Override switch — drop a sentinel file to silence this check.
if [[ -f .claude/skip-e2e-check ]]; then
  exit 0
fi

# Compare local main to origin/main. If origin is missing (fresh clone,
# offline) fall back to the last 10 commits.
if git rev-parse --verify origin/main >/dev/null 2>&1; then
  RANGE="origin/main..HEAD"
else
  RANGE="HEAD~10..HEAD"
fi

# Bail early if no unpushed commits — nothing to check.
if [[ -z "$(git log "$RANGE" --oneline 2>/dev/null)" ]]; then
  exit 0
fi

# Collect the union of paths touched across all unpushed commits.
PATHS=$(git diff --name-only "$RANGE" 2>/dev/null)
if [[ -z "$PATHS" ]]; then
  exit 0
fi

# Classify by directory prefix.
BACKEND_HIT=0
UI_HIT=0
DOC_ONLY=1

while IFS= read -r path; do
  case "$path" in
    crates/*|src-tauri/src/*|wasm/src/*|src/*|plugin/src/*)
      BACKEND_HIT=1
      DOC_ONLY=0
      ;;
    ui/src/lib/*|ui/src/routes/*|ui/src/app.html|ui/src/app.css|ui/src/lib/*.svelte)
      UI_HIT=1
      DOC_ONLY=0
      ;;
    .planning/*|*.md|.claude/*|.github/*|*.json|*.yaml|*.yml|*.toml|*.lock|*.gitignore|README*|LICENSE*|CHANGELOG*)
      # Docs / planning / config — neither counts as backend or UI work.
      ;;
    *)
      # Unclassified — treat as backend by default to avoid false greens.
      BACKEND_HIT=1
      DOC_ONLY=0
      ;;
  esac
done <<< "$PATHS"

# Pure docs/planning — exit clean.
if [[ "$DOC_ONLY" -eq 1 ]]; then
  exit 0
fi

# Both surfaces touched — coverage looks plausible. Exit clean.
if [[ "$BACKEND_HIT" -eq 1 && "$UI_HIT" -eq 1 ]]; then
  exit 0
fi

# One side missing — emit advisory.
echo ""
echo "⚠ E2E coverage check: unpushed commits touched ONE side only."
echo ""
if [[ "$BACKEND_HIT" -eq 1 && "$UI_HIT" -eq 0 ]]; then
  echo "  Backend (crates / src-tauri / wasm) modified — no UI changes."
  echo "  Per feedback_e2e_coverage_rule.md, every backend feature should"
  echo "  have a matching UI control before the session ends."
  echo ""
  echo "  Files changed:"
  git diff --name-only "$RANGE" 2>/dev/null | grep -E '^(crates/|src-tauri/src/|wasm/src/|src/|plugin/src/)' | head -10 | sed 's/^/    /'
elif [[ "$BACKEND_HIT" -eq 0 && "$UI_HIT" -eq 1 ]]; then
  echo "  UI (ui/src/lib) modified — no backend changes."
  echo "  This is fine for pure-UI work (layout, theming, components)."
  echo "  Verify the UI isn't expecting a backend hook that doesn't exist yet."
  echo ""
  echo "  Files changed:"
  git diff --name-only "$RANGE" 2>/dev/null | grep -E '^ui/' | head -10 | sed 's/^/    /'
fi
echo ""
echo "  To silence this check: touch .claude/skip-e2e-check"

exit 0

#!/usr/bin/env bash
# Stop hook — autonomous continuation while context budget allows.
#
# Reads token usage from the session transcript, compares against the
# 1M-token budget of Opus 4.7 (1M context), and blocks the Stop if
# we're below 90% of budget AND there's still active work in flight.
# Otherwise allows normal exit.
#
# Override: create .claude/stop-loop.disabled in the repo root to force
# normal stops regardless of context budget — useful when you want to
# end a session even though there's more work to do.
#
# Loop safety: this hook does NOT special-case stop_hook_active. The
# context-budget check is the real loop bound — at 1M tokens divided
# by ~10k tokens per turn, you get ~100 autonomous turns before the
# budget hits 90% and the hook stops blocking. That's a deliberate
# choice: the user asked for "autonomous until ~90%", not "one extra
# turn then stop".

set -u

# 1. Parse hook input from stdin.
input=$(cat || true)
session_id=$(printf '%s' "$input" | jq -r '.session_id // empty' 2>/dev/null)
transcript=$(printf '%s' "$input" | jq -r '.transcript_path // empty' 2>/dev/null)

# 2. Resolve repo root for override file + roadmap lookup.
repo_root=$(git -C "${CLAUDE_PROJECT_DIR:-$PWD}" rev-parse --show-toplevel 2>/dev/null) || exit 0

# 3. Explicit override — user kill switch.
if [ -f "$repo_root/.claude/stop-loop.disabled" ]; then
  exit 0
fi

# 4. Compute token usage. The Claude Code transcript is JSONL; each
# assistant turn carries a `message.usage` block with input_tokens,
# output_tokens, cache_creation_input_tokens, cache_read_input_tokens.
# The model's *context window* on the next turn is the sum of the
# most recent message's input-side fields plus its output. If we can't
# read the transcript or parse it, default to 0 (which forces the
# safest behavior: allow stop, don't block).
total_tokens=0
if [ -n "$transcript" ] && [ -f "$transcript" ]; then
  total_tokens=$(jq -r -s '
    map(select(.message.usage != null))
    | last
    | .message.usage
    | (.input_tokens // 0)
      + (.output_tokens // 0)
      + (.cache_creation_input_tokens // 0)
      + (.cache_read_input_tokens // 0)
  ' "$transcript" 2>/dev/null)
fi
[ -z "$total_tokens" ] || [ "$total_tokens" = "null" ] && total_tokens=0

context_limit=1000000
percent=$(( total_tokens * 100 / context_limit ))

# 5. Decide whether there's pending work to keep working on. We look
# for the most-recent roadmap file in .planning/research/ROADMAP-*.md
# (or the canonical .planning/ROADMAP.md). If one exists and references
# "Phase" markers, assume work is in flight.
has_pending="false"
roadmap=""
for candidate in \
  "$repo_root/.planning/ROADMAP.md" \
  "$repo_root/.planning/research/ROADMAP-v1.2.x.md" \
  "$repo_root/.planning/research/ROADMAP.md"
do
  if [ -f "$candidate" ]; then
    roadmap="$candidate"
    if grep -qE '^## Phase [0-9]' "$candidate" 2>/dev/null; then
      has_pending="true"
      break
    fi
  fi
done

# 6. Decision.
#    - At/above 90% budget OR no pending work signal → allow stop.
#    - Otherwise → block with a continue instruction.
if [ "$percent" -ge 90 ] || [ "$has_pending" != "true" ]; then
  # Allow normal exit. Log to stderr for the model's awareness (stderr
  # goes back to the model via the harness on exit code 0 too, but only
  # when there's specific signal — keep it terse).
  if [ "$percent" -ge 90 ]; then
    echo "[stop-autoloop] context at ${percent}% of 1M budget — allowing stop" >&2
  fi
  exit 0
fi

# 7. Block the stop with a continue instruction. JSON on stdout with
# decision=block sends the `reason` back to the model as new context;
# the model continues its turn.
roadmap_rel=${roadmap#"$repo_root"/}
cat <<EOF
{"decision": "block", "reason": "Context at ${percent}% of 1M budget — continuing autonomous execution per the user's stop-hook directive. Active roadmap: ${roadmap_rel}. Pick the next concrete task from the active Phase, execute it with the TDD workflow (.claude/skills/tdd-workflow/), commit it atomically. If you find no concrete tasks remaining, say so plainly and let this turn end — the hook will allow normal exit on subsequent stops when no Phase markers remain. Do not invent work. Do not loop on already-done tasks; check git log to confirm what's already shipped this session."}
EOF
exit 0

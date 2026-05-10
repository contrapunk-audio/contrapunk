---
name: code-entropy
description: Measure and surface code entropy across the Contrapunk codebase. Use when the user asks "where should I refactor?", "what's the highest-entropy area?", "show codebase health", "what are the next steps?", or before a release. Also called automatically by the SessionStart hook for a fast snapshot. Produces a structured report with hotspots prioritized for action.
---

# code-entropy

A codebase tends toward higher entropy unless you spend energy reducing it. This skill measures that entropy across four axes and surfaces the highest-entropy areas as the next-step priority list.

Four entropy axes Contrapunk cares about:

1. **Structural entropy** — file size, function length, nesting depth. *Where the code is hard to read.*
2. **Behavioral entropy** — churn × complexity hotspots (Adam Tornhill's hotspot analysis). *Where bugs concentrate.*
3. **Test entropy** — uncovered code, untested new public functions, slow / flaky tests. *Where bugs hide.*
4. **Dependency entropy** — total deps, transitive depth, binary / bundle size, unused deps. *Where the surface area for breakage lives.*

A high score on any axis is signal. A high score on **two or more axes** in the same area is the next-step priority.

## Steps

### 1. Structural entropy snapshot

```bash
# Per-language LOC and file counts. Install `tokei` if missing.
tokei crates/ src/ src-tauri/src/ wasm/src/ plugin/src/ ui/src/ --type Rust,TypeScript,Svelte,JSON 2>/dev/null \
  || (cargo install tokei && tokei crates/ src/ src-tauri/src/ wasm/src/ plugin/src/ ui/src/)

# Top 10 longest Rust files (LOC).
find crates src src-tauri/src wasm/src plugin/src -name '*.rs' -not -path '*/target/*' \
  | xargs wc -l 2>/dev/null | sort -rn | head -11 | tail -10

# Top 10 longest functions across the workspace.
rg -n '^(pub |async |unsafe )*fn [a-zA-Z_]+' --type rust crates/ src/ src-tauri/src/ wasm/src/ plugin/src/ \
  | awk -F: '{print $1 ":" $2}' \
  | while read loc; do
      file=$(echo "$loc" | cut -d: -f1)
      line=$(echo "$loc" | cut -d: -f2)
      # Approximate function length = lines until next top-level fn or end of file
      end=$(awk -v start="$line" 'NR > start && /^(pub |async |unsafe )*fn / {print NR-1; exit} END {print NR}' "$file")
      len=$((end - line))
      printf '%5d  %s:%s\n' "$len" "$file" "$line"
    done | sort -rn | head -10
```

Flag anything > 80 lines as a refactor candidate. Anything > 200 is structural smell.

### 2. Behavioral entropy — churn × complexity

The Tornhill hotspot: files touched often AND that are already complex are the highest-risk places to work.

```bash
# Top 20 most-churned Rust files in the last 90 days.
git log --since=90.days --name-only --pretty=format: -- '*.rs' \
  | grep -v '^$' | sort | uniq -c | sort -rn | head -20

# Cross-reference with LOC — multiply each file's churn by its line count to get
# a rough "behavioral entropy score".
git log --since=90.days --name-only --pretty=format: -- '*.rs' \
  | grep -v '^$' | sort | uniq -c | sort -rn | head -30 \
  | while read churn file; do
      [ -f "$file" ] || continue
      loc=$(wc -l < "$file")
      score=$((churn * loc))
      printf '%10d  churn=%3d  loc=%5d  %s\n' "$score" "$churn" "$loc" "$file"
    done | sort -rn | head -10
```

The top of this list is where bugs concentrate. `crates/contrapunk-harmony/src/engine.rs` (2400+ LOC, recent commits) is almost certainly #1.

### 3. Test entropy — coverage + TDD discipline

```bash
# Per-crate test count.
for d in crates/*/src src/ src-tauri/src/ wasm/src/ plugin/src/; do
  [ -d "$d" ] || continue
  count=$(rg -c '^\s*#\[(tokio::)?test\]' "$d" 2>/dev/null | awk -F: '{sum += $2} END {print sum}')
  echo "$d: ${count:-0} tests"
done

# Untested public fns (heuristic: pub fns in src that have no matching test name).
# Coarse — produces noise — use as a directional signal only.
rg -n '^\s*pub fn ([a-z_][a-zA-Z0-9_]+)' --type rust crates/ src/ 2>/dev/null \
  -r '$1' --no-filename | sort -u | head -40 \
  | while read fn; do
      grep_count=$(rg -l "fn .*${fn}|test_.*${fn}|${fn}_test" --type rust 2>/dev/null | wc -l | tr -d ' ')
      [ "$grep_count" -le 1 ] && echo "  no obvious test: $fn"
    done | head -20

# Run cargo-llvm-cov if installed for actual coverage. If not, recommend installing.
if command -v cargo-llvm-cov >/dev/null 2>&1; then
  cargo llvm-cov --workspace --summary-only 2>&1 | tail -10
else
  echo "cargo-llvm-cov not installed — install with:"
  echo "  cargo install cargo-llvm-cov"
  echo "  rustup component add llvm-tools-preview"
fi
```

A new `pub fn` in a commit with no corresponding test is **TDD discipline entropy** — flag it.

### 4. Dependency entropy

```bash
# Cargo deps — total + transitive.
cargo tree --workspace --depth 0 2>/dev/null | wc -l
cargo tree --workspace --duplicates 2>/dev/null | head -20

# WASM bundle size (if built).
if [ -f ui/src/lib/wasm-pkg/contrapunk_wasm_bg.wasm ]; then
  ls -lh ui/src/lib/wasm-pkg/contrapunk_wasm_bg.wasm | awk '{print "wasm bundle: " $5}'
fi

# UI bundle size (if built).
if [ -d ui/build ]; then
  du -sh ui/build/_app/immutable/chunks/ 2>/dev/null | awk '{print "ui chunks: " $1}'
fi

# Cargo binary size of the release build (if available).
if [ -f target/release/contrapunk ]; then
  ls -lh target/release/contrapunk | awk '{print "release bin: " $5}'
fi

# Unused deps (requires cargo-machete).
if command -v cargo-machete >/dev/null 2>&1; then
  echo "--- unused deps ---"
  cargo machete 2>&1 | head -30
else
  echo "cargo-machete not installed — install with: cargo install cargo-machete"
fi

# npm bundle composition.
if [ -f ui/package.json ]; then
  echo "--- npm prod deps count ---"
  jq '.dependencies | length' ui/package.json 2>/dev/null
fi
```

### 5. Report

Emit a structured summary. The TOP of the report is the "next-step priority list" — the cross-axis hotspots.

```
## Code entropy snapshot — <ISO timestamp> (commit: <SHA>)

### Cross-axis hotspots (next-step priority)
[Files that are HIGH on 2+ axes. Sorted by combined entropy. Each row: file, axes, action.]

| File | LOC | 90d churn | Tests touching it | Hotspot score | Suggested action |
|---|---|---|---|---|---|
| ... | ... | ... | ... | ... | ... |

### Structural entropy
- Longest files (top 5):
- Longest functions (top 5):
- Files > 1000 LOC:

### Behavioral entropy
- Churn × complexity top 5:

### Test entropy
- Total tests by crate:
- New pub fns since HEAD~10 with no test:
- Coverage: <% from cargo-llvm-cov or 'not measured'>

### Dependency entropy
- Cargo deps total / duplicate:
- Unused deps (cargo machete):
- WASM bundle size + delta from HEAD~1:
- UI bundle size + delta from HEAD~1:
- Top-level npm deps count:

### Recommendations
1. [Highest-priority action — usually "refactor file X because it scored high on structural + behavioral entropy"]
2. ...
```

## When to invoke

- **Before each release** — perf-check covers the runtime perf; this covers the structural / maintainability angle.
- **When the user asks "what's next?"** — entropy hotspots are usually the right answer when no urgent feature/bug dominates.
- **After a large refactor** — measure the delta.
- **Automatically by SessionStart hook** — a short summary is injected so Claude opens every session knowing where the entropy concentrates.

## What this skill does NOT do

- Doesn't auto-fix anything. Outputs a priority list; the human decides.
- Doesn't fail / block. Pure information.
- Doesn't make absolute calls — entropy is relative. A 600-line file is fine in some contexts, a problem in others. Surface the data; let the operator decide.

## Tool dependencies

| Tool | Required? | Install | Why |
|---|---|---|---|
| `tokei` | yes | `cargo install tokei` | LOC counts |
| `rg` | yes | `brew install ripgrep` | Fast grep |
| `jq` | yes | already in shell PATH | JSON parsing |
| `cargo-llvm-cov` | optional | `cargo install cargo-llvm-cov && rustup component add llvm-tools-preview` | Actual test coverage |
| `cargo-machete` | optional | `cargo install cargo-machete` | Unused dep detection |

If a required tool is missing, install it on first run (one-time cost) — don't degrade silently.

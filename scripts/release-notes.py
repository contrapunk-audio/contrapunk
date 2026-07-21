#!/usr/bin/env python3
"""Generate categorized release notes from git log.

Reads commits since the last release tag and categorizes them
by conventional commit prefix (feat, fix, chore, ci, docs, etc.).
Outputs Markdown suitable for a GitHub Release body.

Environment variables:
    LAST_TAG   — previous release tag (empty = first release)
    VERSION    — version string for this release
    CODENAME   — release codename (e.g. "Holdsworth HAL")
"""

import os
import subprocess
import re
import sys

VERSION = os.environ.get("VERSION", "unreleased")
CODENAME = os.environ.get("CODENAME", "")
LAST_TAG = os.environ.get("LAST_TAG", "")

# Category mapping: prefix → (emoji, display name, priority)
CATEGORIES = {
    "feat":     ("🎸", "Features", 1),
    "fix":      ("🔧", "Bug Fixes", 2),
    "perf":     ("⚡", "Performance", 3),
    "refactor": ("♻️", "Refactoring", 4),
    "docs":     ("📖", "Documentation", 6),
    "test":     ("🧪", "Tests", 7),
    "ci":       ("🏗️", "CI / Build", 8),
    "chore":    ("🧹", "Chores", 9),
    "wip":      ("🚧", "Work in Progress", 10),
}


def get_commits():
    """Get commit log since last tag, enriched with PR release notes when available."""
    if LAST_TAG:
        cmd = ["git", "log", f"{LAST_TAG}..HEAD", "--pretty=format:%H|%s|%an"]
    else:
        cmd = ["git", "log", "--pretty=format:%H|%s|%an", "-50"]

    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        return []

    # Try to get merged PRs with their release notes and labels
    pr_data = get_pr_data()

    commits = []
    for line in result.stdout.strip().splitlines():
        parts = line.split("|", 2)
        if len(parts) == 3:
            full_hash = parts[0]
            commit = {
                "hash": full_hash[:7],
                "full_hash": full_hash,
                "message": parts[1],
                "author": parts[2],
                "release_note": "",
                "pr_labels": [],
                "pr_number": None,
            }
            # Match commit to a PR
            if full_hash in pr_data:
                pr = pr_data[full_hash]
                commit["release_note"] = pr.get("release_note", "")
                commit["pr_labels"] = pr.get("labels", [])
                commit["pr_number"] = pr.get("number")
                commit["author_login"] = pr.get("author_login", "")
            commits.append(commit)
    return commits


def get_pr_data():
    """Fetch merged PR data (release notes, labels) keyed by merge commit SHA."""
    try:
        cmd = [
            "gh", "pr", "list",
            "--repo", "contrapunk-audio/contrapunk",
            "--state", "merged",
            "--limit", "100",
            "--json", "number,title,body,labels,mergeCommit,author",
        ]
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=15)
        if result.returncode != 0:
            return {}

        import json
        prs = json.loads(result.stdout)
        data = {}
        release_note_re = re.compile(
            r"```release-note\s*\n(.*?)```", re.DOTALL
        )
        for pr in prs:
            merge_sha = pr.get("mergeCommit", {}).get("oid", "")
            if not merge_sha:
                continue

            # Extract release note from PR body
            body = pr.get("body", "") or ""
            match = release_note_re.search(body)
            note = ""
            if match:
                note = match.group(1).strip()
                if note.upper() == "NONE":
                    note = ""

            labels = [l["name"] for l in pr.get("labels", [])]

            data[merge_sha] = {
                "number": pr["number"],
                "release_note": note,
                "labels": labels,
                "author_login": (pr.get("author") or {}).get("login", "") or "",
            }
        return data
    except (FileNotFoundError, subprocess.TimeoutExpired, Exception):
        return {}


LABEL_TO_CATEGORY = {
    "kind/feature": "feat",
    "kind/bug": "fix",
    "kind/cleanup": "refactor",
    "kind/docs": "docs",
    "kind/ci": "ci",
    "kind/perf": "perf",
}


def categorize(commits):
    """Sort commits by PR labels first, then conventional commit prefix."""
    categorized = {}
    uncategorized = []

    prefix_re = re.compile(r"^(feat|fix|perf|refactor|docs|test|ci|chore|wip)(\(.+?\))?:\s*(.+)")

    for commit in commits:
        # Try PR labels first
        category = None
        for label in commit.get("pr_labels", []):
            if label in LABEL_TO_CATEGORY:
                category = LABEL_TO_CATEGORY[label]
                break

        # Fall back to commit prefix
        match = prefix_re.match(commit["message"])
        if category:
            description = match.group(3) if match else commit["message"]
            scope = match.group(2).strip("()") if match and match.group(2) else ""
        elif match:
            category = match.group(1)
            scope = match.group(2).strip("()") if match.group(2) else ""
            description = match.group(3)
        else:
            uncategorized.append(commit)
            continue

        if category not in categorized:
            categorized[category] = []
        categorized[category].append({
            **commit,
            "scope": scope,
            "description": description,
        })

    return categorized, uncategorized


def get_contributors(commits):
    """Get unique GitHub handles from PR authors, Tekton-style.

    Sources logins from PR metadata (`gh pr list ... --json author`),
    skips any AI / Claude attributions, and rewrites `name[bot]`
    accounts to `app/name` per Tekton convention.
    """
    handles = set()
    for c in commits:
        login = c.get("author_login", "")
        if not login:
            continue
        # Filter AI co-author / Claude attributions entirely.
        lower = login.lower()
        if "claude" in lower or "anthropic" in lower:
            continue
        # Tekton convention: GitHub Apps render as "app/<name>".
        if login.endswith("[bot]"):
            handles.add(f"app/{login[:-5]}")
        else:
            handles.add(login)
    return sorted(handles)


def get_stats():
    """Get diffstat since last tag."""
    if LAST_TAG:
        cmd = ["git", "diff", "--shortstat", f"{LAST_TAG}..HEAD"]
    else:
        cmd = ["git", "diff", "--shortstat", "HEAD~20..HEAD"]

    result = subprocess.run(cmd, capture_output=True, text=True)
    return result.stdout.strip() if result.returncode == 0 else ""


def render(categorized, uncategorized, commits):
    """Render Markdown release notes."""
    lines = []

    # Header
    if CODENAME:
        lines.append(f"# Contrapunk {VERSION} — {CODENAME}")
    else:
        lines.append(f"# Contrapunk {VERSION}")
    lines.append("")

    # Optional curated header sections (env-driven so weekly auto-release
    # stays terse but tagged releases can prepend a pitch / TL;DR / try-it
    # block matching the Tekton-style hand-curated release format).
    pitch = os.environ.get("PITCH", "").strip()
    tldr = os.environ.get("TLDR", "").strip()  # newline-separated bullets
    try_it = os.environ.get("TRY_IT", "").strip()
    breaking = os.environ.get("BREAKING", "").strip()

    if pitch:
        lines.append(f"**{pitch}**")
        lines.append("")
    if try_it:
        lines.append(f"Try it now: **{try_it}**")
        lines.append("")
    if tldr:
        lines.append("---")
        lines.append("")
        lines.append("## TL;DR")
        lines.append("")
        for bullet in tldr.splitlines():
            bullet = bullet.strip()
            if bullet:
                lines.append(f"- {bullet}")
        lines.append("")
    if breaking:
        lines.append("---")
        lines.append("")
        lines.append("## ⚠️ Breaking changes")
        lines.append("")
        for line in breaking.splitlines():
            lines.append(line)
        lines.append("")
    if pitch or tldr or try_it or breaking:
        lines.append("---")
        lines.append("")

    # Categorized sections (sorted by priority)
    sorted_cats = sorted(
        categorized.items(),
        key=lambda kv: CATEGORIES.get(kv[0], ("", "", 50))[2]
    )

    # Dedupe a trailing "(#N)" already present in commit/PR title so we
    # don't render "(#88) (#88)" — common when the commit message included
    # the PR number AND we append it from PR metadata.
    pr_suffix_re = re.compile(r"\s*\(#\d+\)\s*$")

    for prefix, items in sorted_cats:
        emoji, display_name, _ = CATEGORIES.get(prefix, ("📦", prefix.title(), 50))
        lines.append(f"## {emoji} {display_name}")
        lines.append("")
        seen = set()  # dedupe by (description, pr_number) so split commits don't double-list
        for item in items:
            scope_str = f"**{item['scope']}**: " if item.get("scope") else ""
            description = pr_suffix_re.sub("", item["description"])
            pr_ref = f" (#{item['pr_number']})" if item.get("pr_number") else f" ({item['hash']})"
            key = (description.strip().lower(), item.get("pr_number"))
            if key in seen:
                continue
            seen.add(key)
            lines.append(f"- {scope_str}{description}{pr_ref}")
            # Include PR release note if present
            if item.get("release_note"):
                for note_line in item["release_note"].splitlines():
                    lines.append(f"  {note_line}")
        lines.append("")

    # Uncategorized
    if uncategorized:
        lines.append("## 📦 Other Changes")
        lines.append("")
        for item in uncategorized:
            lines.append(f"- {item['message']} ({item['hash']})")
        lines.append("")

    # Thanks (Tekton-style)
    contributors = get_contributors(commits)
    if contributors:
        ver = VERSION if VERSION.startswith("v") else f"v{VERSION}"
        lines.append("## Thanks")
        lines.append("")
        lines.append(f"Thanks to these contributors who contributed to {ver}!")
        for handle in contributors:
            lines.append(f"* :heart: @{handle}")
        lines.append("")

    # Stats
    stats = get_stats()
    if stats:
        lines.append("## 📊 Stats")
        lines.append("")
        lines.append(f"`{stats}`")
        lines.append("")

    # Compare link
    if LAST_TAG:
        lines.append("## 📋 Full changelog")
        lines.append("")
        lines.append(
            f"https://github.com/contrapunk-audio/contrapunk/compare/{LAST_TAG}...v{VERSION}"
        )
        lines.append("")

    # Footer — only the auto-release banner if no curated header was given
    if not (pitch or tldr or try_it or breaking):
        lines.append("---")
        lines.append("*Automated weekly release. Only created when source changes are detected.*")

    return "\n".join(lines)


if __name__ == "__main__":
    commits = get_commits()
    if not commits:
        print(f"# Contrapunk {VERSION}\n\nNo changes detected.")
        sys.exit(0)

    categorized, uncategorized = categorize(commits)
    print(render(categorized, uncategorized, commits))

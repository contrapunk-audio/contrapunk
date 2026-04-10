#!/usr/bin/env python3
"""Generate categorized release notes from git log.

Reads commits since the last release tag and categorizes them
by conventional commit prefix (feat, fix, chore, ci, docs, etc.).
Outputs Markdown suitable for a GitHub Release body.

Environment variables:
    LAST_TAG   — previous release tag (empty = first release)
    VERSION    — version string for this release
    CODENAME   — release codename (e.g. "Hendrix HAL")
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
    """Get commit log since last tag."""
    if LAST_TAG:
        cmd = ["git", "log", f"{LAST_TAG}..HEAD", "--pretty=format:%H|%s|%an"]
    else:
        cmd = ["git", "log", "--pretty=format:%H|%s|%an", "-50"]

    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        return []

    commits = []
    for line in result.stdout.strip().splitlines():
        parts = line.split("|", 2)
        if len(parts) == 3:
            commits.append({
                "hash": parts[0][:7],
                "message": parts[1],
                "author": parts[2],
            })
    return commits


def categorize(commits):
    """Sort commits into categories based on conventional commit prefix."""
    categorized = {}
    uncategorized = []

    prefix_re = re.compile(r"^(feat|fix|perf|refactor|docs|test|ci|chore|wip)(\(.+?\))?:\s*(.+)")

    for commit in commits:
        match = prefix_re.match(commit["message"])
        if match:
            prefix = match.group(1)
            scope = match.group(2) or ""
            description = match.group(3)
            cat = CATEGORIES.get(prefix, ("📦", prefix.title(), 50))
            key = prefix
            if key not in categorized:
                categorized[key] = []
            categorized[key].append({
                **commit,
                "scope": scope.strip("()"),
                "description": description,
            })
        else:
            uncategorized.append(commit)

    return categorized, uncategorized


def get_contributors(commits):
    """Get unique contributor names."""
    authors = set()
    for c in commits:
        name = c["author"]
        # Skip bot authors
        if "bot" in name.lower() or "noreply" in name.lower():
            continue
        authors.add(name)
    return sorted(authors)


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

    # Categorized sections (sorted by priority)
    sorted_cats = sorted(
        categorized.items(),
        key=lambda kv: CATEGORIES.get(kv[0], ("", "", 50))[2]
    )

    for prefix, items in sorted_cats:
        emoji, display_name, _ = CATEGORIES.get(prefix, ("📦", prefix.title(), 50))
        lines.append(f"## {emoji} {display_name}")
        lines.append("")
        for item in items:
            scope_str = f"**{item['scope']}**: " if item.get("scope") else ""
            lines.append(f"- {scope_str}{item['description']} ({item['hash']})")
        lines.append("")

    # Uncategorized
    if uncategorized:
        lines.append("## 📦 Other Changes")
        lines.append("")
        for item in uncategorized:
            lines.append(f"- {item['message']} ({item['hash']})")
        lines.append("")

    # Contributors
    contributors = get_contributors(commits)
    if contributors:
        lines.append("## 🙏 Contributors")
        lines.append("")
        lines.append(", ".join(contributors))
        lines.append("")

    # Stats
    stats = get_stats()
    if stats:
        lines.append("## 📊 Stats")
        lines.append("")
        lines.append(f"`{stats}`")
        lines.append("")

    # Footer
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

#!/usr/bin/env python3
"""
Parse ml/CONCEPTS.md into ui/static/training/concepts.json
for the diary's inline concept explanations.
"""

import json
import re
import os

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.abspath(os.path.join(SCRIPT_DIR, "..", ".."))
INPUT_PATH = os.path.join(PROJECT_ROOT, "ml", "CONCEPTS.md")
OUTPUT_PATH = os.path.join(PROJECT_ROOT, "ui", "static", "training", "concepts.json")


def slugify(text: str) -> str:
    """Convert a title string to kebab-case id."""
    # Remove markdown formatting and special chars, keep alphanumeric and spaces/hyphens
    text = re.sub(r"[()']", "", text)
    text = text.lower().strip()
    text = re.sub(r"[^a-z0-9\s-]", "", text)
    text = re.sub(r"[\s]+", "-", text)
    text = re.sub(r"-+", "-", text)
    return text.strip("-")


def extract_term(title: str) -> str:
    """Extract the short keyword/term from a concept title.

    For titles like "What is a DI (Direct Input) Signal?" -> "DI Signal"
    For titles like "What is RMS?" -> "RMS"
    For titles like "Why Same Note Sounds Different..." -> "Same Note Different Strings"
    For titles like "What Makes a Hollow Body Different?" -> "Hollow Body"
    """
    # "What is a/an X?" pattern
    m = re.match(r"What is (?:a |an )?(.+?)[\?]?$", title, re.IGNORECASE)
    if m:
        term = m.group(1).strip().rstrip("?")
        # Remove parenthetical explanations: "DI (Direct Input) Signal" -> "DI Signal"
        term = re.sub(r"\s*\([^)]*\)\s*", " ", term).strip()
        return term

    # "What are X?" pattern
    m = re.match(r"What (?:are|is) (.+?)[\?]?$", title, re.IGNORECASE)
    if m:
        term = m.group(1).strip().rstrip("?")
        term = re.sub(r"\s*\([^)]*\)\s*", " ", term).strip()
        return term

    # "What Makes a X Different?" pattern
    m = re.match(r"What Makes (?:a |an )?(.+?)[\?]?$", title, re.IGNORECASE)
    if m:
        term = m.group(1).strip().rstrip("?")
        # Remove trailing "Different"
        term = re.sub(r"\s+Different$", "", term, flags=re.IGNORECASE)
        return term

    # "Why X" pattern — extract a concise term
    m = re.match(r"Why (.+?)[\?]?$", title, re.IGNORECASE)
    if m:
        raw = m.group(1).strip().rstrip("?")
        # "Why ML is a Correction Layer, Not a Trigger" -> "ML Correction Layer"
        # "Why Same Note Sounds Different on Different Strings" -> keep short
        # "Why O(1) Ring Buffers Matter on Audio Threads" -> "Ring Buffers"
        words = raw.split()
        if len(words) > 4:
            return " ".join(words[:4])
        return raw

    # Fallback: use the title as-is
    return title.strip().rstrip("?")


def extract_term_id(title: str) -> str:
    """Generate a concept id from the title, focusing on the key term."""
    term = extract_term(title)
    return slugify(term)


def first_sentence(text: str) -> str:
    """Extract the first sentence from text for the summary.

    Handles cases like "RMS = Root Mean Square." and multi-line first sentences.
    """
    # Strip leading whitespace
    text = text.strip()
    if not text:
        return ""

    # Collect text until we hit a sentence-ending pattern
    # We need to handle code blocks, bullet points, etc.
    lines = text.split("\n")
    collected = []

    for line in lines:
        stripped = line.strip()
        # Stop at empty lines, bullet points, code blocks
        if not stripped:
            break
        if stripped.startswith("- ") or stripped.startswith("* "):
            if collected:
                break
            # If the first line IS a bullet, take it
            collected.append(stripped.lstrip("- *"))
            break
        if stripped.startswith("```"):
            break
        collected.append(stripped)

        # Check if we have a sentence ending
        full = " ".join(collected)
        # Find first sentence break (period followed by space or end)
        # But not inside abbreviations like "e.g." or numbers like "0.1"
        # Simple approach: find ". " or ".\n" or period at end
        sentences = re.split(r'(?<=[.!?])\s+(?=[A-Z])', full)
        if len(sentences) > 1:
            return sentences[0].strip()
        if full.endswith((".","!","?")):
            return full.strip()

    result = " ".join(collected).strip()
    # If there's a sentence ending inside, extract just the first sentence
    m = re.match(r'^(.+?[.!?])(?:\s|$)', result)
    if m:
        return m.group(1).strip()
    # Fallback: return what we have, adding period if needed
    if result and not result.endswith((".", "!", "?")):
        result += "."
    # Clean up awkward ":." endings
    result = result.replace(":.", ".")
    return result


def parse_concepts_md(filepath: str) -> dict:
    """Parse the CONCEPTS.md file into structured JSON data."""
    with open(filepath, "r") as f:
        content = f.read()

    lines = content.split("\n")
    sections = []
    current_section = None
    current_concept = None
    concept_lines = []

    def flush_concept():
        nonlocal current_concept, concept_lines
        if current_concept and current_section is not None:
            body = "\n".join(concept_lines).strip()
            summary = first_sentence(body)
            current_concept["summary"] = summary
            current_concept["content"] = body
            current_section["concepts"].append(current_concept)
        current_concept = None
        concept_lines = []

    for line in lines:
        # Section header: ## Title
        if line.startswith("## ") and not line.startswith("### "):
            flush_concept()
            title = line[3:].strip()
            current_section = {
                "id": slugify(title),
                "title": title,
                "concepts": [],
            }
            sections.append(current_section)
            continue

        # Concept header: ### Title
        if line.startswith("### "):
            flush_concept()
            title = line[4:].strip()
            current_concept = {
                "id": extract_term_id(title),
                "term": extract_term(title),
                "title": title,
                "summary": "",
                "content": "",
            }
            continue

        # Skip horizontal rules (section dividers)
        if line.strip() == "---":
            continue

        # Content line (only collect if inside a concept)
        if current_concept is not None:
            concept_lines.append(line)

    # Flush last concept
    flush_concept()

    return {"sections": sections}


def main():
    data = parse_concepts_md(INPUT_PATH)

    # Ensure output directory exists
    os.makedirs(os.path.dirname(OUTPUT_PATH), exist_ok=True)

    with open(OUTPUT_PATH, "w") as f:
        json.dump(data, f, indent=2, ensure_ascii=False)
        f.write("\n")

    # Print summary
    total_concepts = 0
    print("=" * 50)
    print("Concepts export summary")
    print("=" * 50)
    for section in data["sections"]:
        count = len(section["concepts"])
        total_concepts += count
        print(f"  {section['title']}: {count} concepts")
        for concept in section["concepts"]:
            print(f"    - {concept['term']} ({concept['id']})")

    file_size = os.path.getsize(OUTPUT_PATH)
    print("-" * 50)
    print(f"Sections:    {len(data['sections'])}")
    print(f"Concepts:    {total_concepts}")
    print(f"Output:      {OUTPUT_PATH}")
    print(f"File size:   {file_size:,} bytes ({file_size / 1024:.1f} KB)")
    print("=" * 50)


if __name__ == "__main__":
    main()

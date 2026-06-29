#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import sys
import uuid
from pathlib import Path


def normalize_version(value: str) -> str:
    value = value.strip()
    if value.startswith("[") and "]" in value:
        value = value[1 : value.index("]")]
    else:
        value = value.split(None, 1)[0] if value else value
    return value.removeprefix("v")


def is_h2(line: str) -> bool:
    return line.startswith("## ") and not line.startswith("### ")


def extract_notes(changelog: str, tag: str) -> str:
    target = normalize_version(tag)
    lines = changelog.splitlines()
    start = None

    for index, line in enumerate(lines):
        if is_h2(line) and normalize_version(line[3:]) == target:
            start = index + 1
            break

    if start is None:
        raise ValueError(f"CHANGELOG section for {tag!r} was not found")

    end = len(lines)
    for index in range(start, len(lines)):
        if is_h2(lines[index]):
            end = index
            break

    notes = "\n".join(lines[start:end]).strip()
    if not notes:
        raise ValueError(f"CHANGELOG section for {tag!r} is empty")
    return notes


def write_github_output(name: str, value: str) -> None:
    output_path = os.environ.get("GITHUB_OUTPUT")
    if not output_path:
        raise ValueError("GITHUB_OUTPUT is not set")

    delimiter = f"EOF_{uuid.uuid4().hex}"
    with open(output_path, "a", encoding="utf-8") as output:
        output.write(f"{name}<<{delimiter}\n{value}\n{delimiter}\n")


def main() -> int:
    parser = argparse.ArgumentParser(description="Extract release notes from CHANGELOG.md")
    parser.add_argument("--changelog", default="CHANGELOG.md")
    parser.add_argument("--tag", required=True)
    parser.add_argument("--github-output", help="Write notes to this GitHub Actions output name")
    args = parser.parse_args()

    changelog = Path(args.changelog).read_text(encoding="utf-8")
    notes = extract_notes(changelog, args.tag)

    if args.github_output:
        write_github_output(args.github_output, notes)
    else:
        print(notes)

    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:
        print(f"error: {exc}", file=sys.stderr)
        raise SystemExit(1)

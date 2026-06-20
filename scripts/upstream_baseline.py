#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import tomllib
from dataclasses import dataclass
from pathlib import Path


ROOT_DIR = Path(__file__).resolve().parent.parent
MANIFEST_PATH = ROOT_DIR / "spine-upstream.toml"
UPSTREAM_MARKERS = ("spine-cpp", "spine-c", "examples")


@dataclass(frozen=True)
class UpstreamBaseline:
    repo: str
    ref_kind: str
    rev: str
    commit: str
    description: str


def load_baseline() -> UpstreamBaseline:
    raw = tomllib.loads(MANIFEST_PATH.read_text(encoding="utf-8"))["spine_runtimes"]
    return UpstreamBaseline(
        repo=str(raw["repo"]),
        ref_kind=str(raw["ref_kind"]),
        rev=str(raw["rev"]),
        commit=str(raw["commit"]),
        description=str(raw.get("description", "")),
    )


def read_source_commit(source_file: Path) -> str | None:
    if not source_file.is_file():
        return None
    for line in source_file.read_text(encoding="utf-8").splitlines():
        if line.startswith("Commit:"):
            return line.split(":", 1)[1].strip()
    return None


def git_head(repo_dir: Path) -> str | None:
    if not (repo_dir / ".git").exists():
        return None
    proc = subprocess.run(
        ["git", "-C", str(repo_dir), "rev-parse", "HEAD"],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if proc.returncode == 0:
        return proc.stdout.strip()
    return None


def git_head_for_examples(examples_root: Path) -> str | None:
    examples_root = examples_root.resolve()
    for path in [examples_root, *examples_root.parents]:
        if (path / ".git").exists():
            if path == ROOT_DIR:
                return None
            if not all((path / marker).exists() for marker in UPSTREAM_MARKERS):
                return None
            return git_head(path)
    return None


def commit_for_examples(examples_root: Path, baseline: UpstreamBaseline | None = None) -> str:
    baseline = baseline or load_baseline()
    return (
        read_source_commit(examples_root.parent / "SOURCE.txt")
        or read_source_commit(examples_root.parent.parent / "SOURCE.txt")
        or git_head_for_examples(examples_root)
        or baseline.commit
    )

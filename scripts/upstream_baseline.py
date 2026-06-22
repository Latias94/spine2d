#!/usr/bin/env python3
from __future__ import annotations

import json
import subprocess
import tomllib
from dataclasses import dataclass
from pathlib import Path


ROOT_DIR = Path(__file__).resolve().parent.parent
MANIFEST_PATH = ROOT_DIR / "spine-upstream.toml"
BASELINE_PATH = Path(__file__).resolve().with_name("upstream_baseline.json")
UPSTREAM_MARKERS = ("spine-cpp", "spine-c", "examples")
REFERENCE_RUNTIME_PATHS = ("spine-cpp",)


@dataclass(frozen=True)
class UpstreamBaseline:
    repo: str
    ref_kind: str
    rev: str
    commit: str
    description: str
    repo_url: str
    target_version: str
    upstream_ref: str
    target_commit: str
    comparison_branch: str
    runtime_paths: tuple[str, ...]


def load_baseline() -> UpstreamBaseline:
    raw = tomllib.loads(MANIFEST_PATH.read_text(encoding="utf-8"))["spine_runtimes"]
    commit = str(raw["commit"])
    rev = str(raw["rev"])
    repo = str(raw["repo"])
    return UpstreamBaseline(
        repo=repo,
        ref_kind=str(raw["ref_kind"]),
        rev=rev,
        commit=commit,
        description=str(raw.get("description", "")),
        repo_url=repo,
        target_version=rev,
        upstream_ref=rev,
        target_commit=commit,
        comparison_branch="4.3",
        runtime_paths=REFERENCE_RUNTIME_PATHS,
    )


def load_upstream_baseline(path: Path = BASELINE_PATH) -> UpstreamBaseline:
    data = json.loads(path.read_text(encoding="utf-8"))
    repo_url = str(data["repo_url"])
    target_version = str(data["target_version"])
    upstream_ref = str(data["upstream_ref"])
    target_commit = str(data["target_commit"])
    runtime_paths = tuple(str(p) for p in data["runtime_paths"])
    return UpstreamBaseline(
        repo=repo_url,
        ref_kind="tag",
        rev=upstream_ref,
        commit=target_commit,
        description="",
        repo_url=repo_url,
        target_version=target_version,
        upstream_ref=upstream_ref,
        target_commit=target_commit,
        comparison_branch=str(data["comparison_branch"]),
        runtime_paths=runtime_paths,
    )


def read_source_commit(source_file: Path) -> str | None:
    if not source_file.is_file():
        return None
    for line in source_file.read_text(encoding="utf-8").splitlines():
        if line.startswith("Commit:") or line.startswith("TargetCommit:"):
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

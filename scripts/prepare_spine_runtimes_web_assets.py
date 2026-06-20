#!/usr/bin/env python3
# -*- coding: utf-8 -*-

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

from upstream_baseline import load_baseline

UPSTREAM_BASELINE = load_baseline()
UPSTREAM_DEFAULT_REV = UPSTREAM_BASELINE.rev
UPSTREAM_BASELINE_COMMIT = UPSTREAM_BASELINE.commit


@dataclass(frozen=True)
class ExampleEntry:
    name: str
    skeleton: str
    atlas: str


def repo_root() -> Path:
    return Path(__file__).resolve().parents[1]


def run(argv: list[str], *, cwd: Path) -> None:
    p = subprocess.run(argv, cwd=str(cwd), check=False)
    if p.returncode != 0:
        raise RuntimeError(f"Command failed ({p.returncode}): {' '.join(argv)}")


def choose_file(export_dir: Path, candidates: list[str]) -> Path | None:
    for rel in candidates:
        p = export_dir / rel
        if p.is_file():
            return p
    return None


def pick_skeleton_json(export_dir: Path, example_name: str) -> Path:
    preferred = [
        f"{example_name}-pro.json",
        f"{example_name}-ess.json",
        f"{example_name}.json",
    ]
    p = choose_file(export_dir, preferred)
    if p is not None:
        return p

    jsons = sorted(export_dir.glob("*.json"))
    if not jsons:
        raise RuntimeError(f"No .json skeleton found in: {export_dir}")
    return jsons[0]


def pick_atlas(export_dir: Path, example_name: str) -> Path:
    preferred = [
        f"{example_name}-pma.atlas",
        f"{example_name}.atlas",
    ]
    p = choose_file(export_dir, preferred)
    if p is not None:
        return p

    atlases = sorted(export_dir.glob("*.atlas"))
    if not atlases:
        raise RuntimeError(f"No .atlas found in: {export_dir}")
    return atlases[0]


def generate_manifest(spine_runtimes_dir: Path) -> list[ExampleEntry]:
    examples_dir = spine_runtimes_dir / "examples"
    if not examples_dir.is_dir():
        raise RuntimeError(f"Missing examples dir: {examples_dir}")

    out: list[ExampleEntry] = []
    for ex_dir in sorted(examples_dir.iterdir()):
        if not ex_dir.is_dir():
            continue
        export_dir = ex_dir / "export"
        if not export_dir.is_dir():
            continue

        name = ex_dir.name
        try:
            skeleton = pick_skeleton_json(export_dir, name)
            atlas = pick_atlas(export_dir, name)
        except Exception as e:
            print(f"skip {name}: {e}", file=sys.stderr)
            continue

        out.append(
            ExampleEntry(
                name=name,
                skeleton=str(skeleton.relative_to(spine_runtimes_dir)).replace("\\", "/"),
                atlas=str(atlas.relative_to(spine_runtimes_dir)).replace("\\", "/"),
            )
        )

    if not out:
        raise RuntimeError(f"No examples found in: {examples_dir}")
    return out


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(
        description="Fetch official spine-runtimes example exports and generate a web manifest for spine2d-web (no assets are committed)."
    )
    ap.add_argument(
        "--dest",
        default="assets/spine-runtimes",
        help="Destination directory for imported spine-runtimes examples (same as fetch script).",
    )
    ap.add_argument(
        "--scope",
        choices=["tests", "all"],
        default="tests",
        help="How many upstream examples to import (tests is much smaller).",
    )
    ap.add_argument(
        "--mode",
        choices=["export"],
        default="export",
        help="Import mode (web demo needs export: json/skel/atlas/png).",
    )
    ap.add_argument(
        "--rev",
        default=UPSTREAM_DEFAULT_REV,
        help=(
            "Upstream commit/tag/branch to checkout "
            f"(default: {UPSTREAM_DEFAULT_REV}; current baseline commit: {UPSTREAM_BASELINE_COMMIT})."
        ),
    )
    ap.add_argument("--repo-url", default="", help="Override upstream repo URL (optional).")
    ap.add_argument("--depth", type=int, default=1)
    args = ap.parse_args(argv)

    root = repo_root()
    dest = (root / args.dest).resolve()

    fetch = root / "scripts" / "fetch_spine_runtimes_examples.py"
    cmd = [
        sys.executable,
        str(fetch),
        "--mode",
        args.mode,
        "--scope",
        args.scope,
        "--dest",
        str(dest),
        "--depth",
        str(args.depth),
    ]
    if args.rev.strip():
        cmd += ["--rev", args.rev.strip()]
    if args.repo_url.strip():
        cmd += ["--repo-url", args.repo_url.strip()]

    run(cmd, cwd=root)

    entries = generate_manifest(dest)
    manifest_path = dest / "web_manifest.json"
    manifest_path.write_text(
        json.dumps(
            {
                "version": 1,
                "base": "assets/spine-runtimes",
                "examples": [e.__dict__ for e in entries],
            },
            ensure_ascii=False,
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )

    print(f"Wrote web manifest: {manifest_path}")
    print("Next:")
    print("  cd spine2d-web && trunk serve")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

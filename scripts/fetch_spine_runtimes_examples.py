#!/usr/bin/env python3
# -*- coding: utf-8 -*-

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path

from upstream_baseline import load_baseline

UPSTREAM_BASELINE = load_baseline()
UPSTREAM_DEFAULT_REPO = UPSTREAM_BASELINE.repo
UPSTREAM_DEFAULT_REV = UPSTREAM_BASELINE.rev
UPSTREAM_BASELINE_COMMIT = UPSTREAM_BASELINE.commit
SPARSE_CHECKOUT_DIRS = [
    "examples",
    "spine-c",
    "spine-cpp",
]

# Keep in sync with `spine2d/src/runtime/examples_smoke_tests.rs` + `vine_smoke_tests.rs`.
TEST_JSON_FILES = [
    "alien/export/alien-ess.json",
    "alien/export/alien-pro.json",
    "diamond/export/diamond-pro.json",
    "dragon/export/dragon-ess.json",
    "hero/export/hero-ess.json",
    "hero/export/hero-pro.json",
    "owl/export/owl-pro.json",
    "raptor/export/raptor-pro.json",
    "spinosaurus/export/spinosaurus-ess.json",
    "stretchyman/export/stretchyman-pro.json",
    "speedy/export/speedy-ess.json",
    "windmill/export/windmill-ess.json",
    "celestial-circus/export/celestial-circus-pro.json",
    "chibi-stickers/export/chibi-stickers.json",
    "cloud-pot/export/cloud-pot.json",
    "coin/export/coin-pro.json",
    "6-arcs/export/6-arcs-pro.json",
    "8-follow-through/export/8-follow-through-pro-ball.json",
    "food-app/export/food-app-search-ess.json",
    "goblins/export/goblins-pro.json",
    "powerup/export/powerup-pro.json",
    "snowglobe/export/snowglobe-pro.json",
    "mix-and-match/export/mix-and-match-pro.json",
    "spineboy/export/spineboy-ess.json",
    "spineboy/export/spineboy-pro.json",
    "tank/export/tank-pro.json",
    "vine/export/vine-pro.json",
]


@dataclass(frozen=True)
class GitInfo:
    remote: str
    commit: str


def run(
    argv: list[str], *, cwd: Path | None = None, stream_output: bool = False
) -> str:
    if stream_output:
        p = subprocess.run(
            argv,
            cwd=str(cwd) if cwd is not None else None,
            text=True,
            check=False,
        )
    else:
        p = subprocess.run(
            argv,
            cwd=str(cwd) if cwd is not None else None,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            check=False,
        )
    if p.returncode != 0:
        if stream_output:
            raise RuntimeError(
                "Command failed:\n"
                f"  cwd: {cwd}\n"
                f"  cmd: {' '.join(argv)}\n"
                f"  code: {p.returncode}\n"
            )
        raise RuntimeError(
            "Command failed:\n"
            f"  cwd: {cwd}\n"
            f"  cmd: {' '.join(argv)}\n"
            f"  code: {p.returncode}\n"
            f"  stdout:\n{p.stdout}\n"
            f"  stderr:\n{p.stderr}\n"
        )
    return p.stdout.strip() if p.stdout is not None else ""


def ensure_dir(path: Path) -> None:
    path.mkdir(parents=True, exist_ok=True)


def write_text(path: Path, text: str) -> None:
    ensure_dir(path.parent)
    path.write_text(text, encoding="utf-8")


def copy_file(src: Path, dst: Path) -> None:
    ensure_dir(dst.parent)
    shutil.copy2(src, dst)


def git_info(repo_dir: Path) -> GitInfo:
    commit = run(["git", "-C", str(repo_dir), "rev-parse", "HEAD"])
    remote = "unknown"
    try:
        remote = run(["git", "-C", str(repo_dir), "remote", "get-url", "origin"])
    except Exception:
        pass
    return GitInfo(remote=remote, commit=commit)


def clone_or_update_repo(repo_url: str, dest: Path, *, rev: str | None, depth: int) -> GitInfo:
    fetched_requested_rev = False
    if dest.exists() and (dest / ".git").is_dir():
        print(f"Updating upstream cache: {dest}", flush=True)
        fetch = [
            "git",
            "-C",
            str(dest),
            "fetch",
            "--progress",
            "--depth",
            str(depth),
            "origin",
        ]
        if rev is not None and rev.strip():
            fetch.append(rev.strip())
            fetched_requested_rev = True
        run(fetch, stream_output=True)
    elif dest.exists():
        raise RuntimeError(f"Destination exists but is not a git repo: {dest}")
    else:
        print(f"Cloning upstream repository into: {dest}", flush=True)
        ensure_dir(dest.parent)
        # Use sparse checkout to avoid pulling the full repository content.
        run(
            [
                "git",
                "clone",
                "--progress",
                "--depth",
                str(depth),
                "--sparse",
                repo_url,
                str(dest),
            ],
            stream_output=True,
        )

    # Ensure we only checkout what we need.
    print("Configuring sparse checkout...", flush=True)
    run(["git", "-C", str(dest), "sparse-checkout", "set", *SPARSE_CHECKOUT_DIRS])

    if rev is not None and rev.strip():
        # Ensure the requested rev exists in a shallow clone (clone --depth only fetches the default branch).
        # Note: `git fetch origin <rev>` may only update FETCH_HEAD (not create a named ref), so we detach at FETCH_HEAD.
        if not fetched_requested_rev:
            print(f"Fetching requested revision: {rev.strip()}", flush=True)
            run(
                [
                    "git",
                    "-C",
                    str(dest),
                    "fetch",
                    "--progress",
                    "--depth",
                    str(depth),
                    "origin",
                    rev.strip(),
                ],
                stream_output=True,
            )
        print(f"Checking out revision: {rev.strip()}", flush=True)
        run(["git", "-C", str(dest), "checkout", "--detach", "FETCH_HEAD"])
    else:
        # Make sure HEAD is checked out (clone --sparse can leave things minimal).
        run(["git", "-C", str(dest), "checkout", "--detach", "HEAD"])

    return git_info(dest)


def write_source_metadata(import_dest: Path, info: GitInfo, *, mode: str, scope: str) -> None:
    ts = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    write_text(
        import_dest / "SOURCE.txt",
        "\n".join(
            [
                f"Source: {info.remote}",
                f"Commit: {info.commit}",
                f"ImportedAtUTC: {ts}",
                f"Mode: {mode}",
                f"Scope: {scope}",
                "",
            ]
        ),
    )


def import_examples(
    *,
    repo_dir: Path,
    import_dest: Path,
    mode: str,
    scope: str,
) -> int:
    src_examples = repo_dir / "examples"
    if not src_examples.is_dir():
        raise RuntimeError(f"Invalid spine-runtimes checkout: missing {src_examples}")

    ensure_dir(import_dest / "examples")

    files_to_copy: list[tuple[Path, Path]] = []
    if scope == "tests":
        if mode == "json":
            for rel in TEST_JSON_FILES:
                src = src_examples / rel
                if not src.is_file():
                    raise RuntimeError(f"Missing upstream file: {src}")
                dst = import_dest / "examples" / rel
                files_to_copy.append((src, dst))
        elif mode == "export":
            examples = sorted({rel.split("/", 1)[0] for rel in TEST_JSON_FILES})
            for ex in examples:
                src_dir = src_examples / ex / "export"
                if not src_dir.is_dir():
                    raise RuntimeError(f"Missing upstream directory: {src_dir}")
                for file in sorted(src_dir.rglob("*")):
                    if not file.is_file():
                        continue
                    rel = file.relative_to(repo_dir)
                    dst = import_dest / rel
                    files_to_copy.append((file, dst))
        else:
            raise RuntimeError(f"Invalid mode: {mode}")
    elif scope == "all":
        if mode == "json":
            exts = {".json"}
        elif mode == "export":
            exts = {".json", ".skel", ".atlas", ".png"}
        else:
            raise RuntimeError(f"Invalid mode: {mode}")
        for file in sorted(src_examples.rglob("*")):
            if not file.is_file():
                continue
            if "export" not in file.parts:
                continue
            if file.suffix.lower() not in exts:
                continue
            rel = file.relative_to(repo_dir)
            dst = import_dest / rel
            files_to_copy.append((file, dst))
    else:
        raise RuntimeError(f"Invalid scope: {scope}")

    total = len(files_to_copy)
    print(f"Importing {total} example files...", flush=True)
    progress_interval = max(1, total // 10)
    for copied, (src, dst) in enumerate(files_to_copy, start=1):
        copy_file(src, dst)
        if copied == 1 or copied == total or copied % progress_interval == 0:
            print(f"  imported {copied}/{total} files", flush=True)

    return total


def try_import_license(repo_dir: Path, import_dest: Path) -> None:
    lic_path = repo_dir / "LICENSE"
    if lic_path.is_file():
        copy_file(lic_path, import_dest / "LICENSE.spine-runtimes.txt")
        return

    try:
        text = run(["git", "-C", str(repo_dir), "show", "HEAD:LICENSE"])
    except Exception:
        return
    write_text(import_dest / "LICENSE.spine-runtimes.txt", text + "\n")


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(
        description="Fetch upstream spine-runtimes and import example exports into third_party/ (not committed by default)."
    )
    ap.add_argument("--repo-url", default=UPSTREAM_DEFAULT_REPO)
    ap.add_argument(
        "--rev",
        default=UPSTREAM_DEFAULT_REV,
        help=(
            "Commit/tag/branch to checkout "
            f"(default: {UPSTREAM_DEFAULT_REV}; current baseline commit: {UPSTREAM_BASELINE_COMMIT})."
        ),
    )
    ap.add_argument("--cache", default=".cache/spine-runtimes", help="Local git checkout directory.")
    ap.add_argument(
        "--dest",
        default="assets/spine-runtimes",
        help="Import destination directory (contains examples/ and SOURCE.txt).",
    )
    ap.add_argument("--mode", choices=["json", "export"], default="json")
    ap.add_argument("--scope", choices=["tests", "all"], default="tests")
    ap.add_argument("--depth", type=int, default=1)
    args = ap.parse_args(argv)

    repo_dir = Path(args.cache).resolve()
    import_dest = Path(args.dest).resolve()

    rev = args.rev.strip() or None
    try:
        info = clone_or_update_repo(args.repo_url, repo_dir, rev=rev, depth=args.depth)
        copied = import_examples(repo_dir=repo_dir, import_dest=import_dest, mode=args.mode, scope=args.scope)
        try_import_license(repo_dir, import_dest)
        write_source_metadata(import_dest, info, mode=args.mode, scope=args.scope)
    except Exception as e:
        print(str(e), file=sys.stderr)
        return 1

    print("Imported upstream spine-runtimes examples:")
    print(f"  repo:   {info.remote}")
    print(f"  commit: {info.commit}")
    print(f"  cache:  {repo_dir}")
    print(f"  dest:   {import_dest}")
    print(f"  mode:   {args.mode}")
    print(f"  scope:  {args.scope}")
    print(f"  files:  {copied}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

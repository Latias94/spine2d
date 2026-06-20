#!/usr/bin/env python3
from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

from upstream_baseline import MANIFEST_PATH, git_head, load_baseline, read_source_commit

ROOT_DIR = Path(__file__).resolve().parent.parent
BASELINE = load_baseline()
UPSTREAM_REPO = BASELINE.repo
EXPECTED_REV = BASELINE.rev
EXPECTED_COMMIT = BASELINE.commit


def ref_label() -> str:
    if BASELINE.ref_kind == "tag":
        return f"tag `{EXPECTED_REV}`"
    if BASELINE.ref_kind == "branch":
        return f"branch `{EXPECTED_REV}`"
    return f"ref `{EXPECTED_REV}`"


REQUIRED_TEXT = {
    "spine-upstream.toml": [
        f'ref_kind = "{BASELINE.ref_kind}"',
        f'rev = "{EXPECTED_REV}"',
        f'commit = "{EXPECTED_COMMIT}"',
    ],
    "scripts/fetch_spine_runtimes_examples.py": [
        "from upstream_baseline import load_baseline",
        "UPSTREAM_DEFAULT_REV = UPSTREAM_BASELINE.rev",
        "UPSTREAM_BASELINE_COMMIT = UPSTREAM_BASELINE.commit",
    ],
    "scripts/prepare_spine_runtimes_web_assets.py": [
        "from upstream_baseline import load_baseline",
        "UPSTREAM_DEFAULT_REV = UPSTREAM_BASELINE.rev",
        "UPSTREAM_BASELINE_COMMIT = UPSTREAM_BASELINE.commit",
    ],
    "scripts/record_oracle_goldens.py": [
        "from upstream_baseline import commit_for_examples, load_baseline",
        "UPSTREAM_DEFAULT_REV = UPSTREAM_BASELINE.rev",
        "UPSTREAM_BASELINE_COMMIT = UPSTREAM_BASELINE.commit",
    ],
    "scripts/record_oracle_render_goldens.py": [
        "from upstream_baseline import commit_for_examples, load_baseline",
        "UPSTREAM_DEFAULT_REV = UPSTREAM_BASELINE.rev",
        "UPSTREAM_BASELINE_COMMIT = UPSTREAM_BASELINE.commit",
    ],
    "docs/parity.md": [
        ref_label(),
        f"commit `{EXPECTED_COMMIT}`",
    ],
    "docs/parity-4.3-beta.md": [
        f"Pinned upstream commit for the current refresh: `{EXPECTED_COMMIT}`",
        f"Current refresh reference: {ref_label()}",
    ],
    "docs/upstream-audit-4.3-beta.md": [
        f"Pinned commit for the current refresh: `{EXPECTED_COMMIT}`",
        f"Reference {ref_label()}",
    ],
    "docs/decisions.md": [
        f"upstream {ref_label()} at `{EXPECTED_COMMIT}`",
    ],
    "docs/upstream-tests.md": [
        ref_label(),
        f"commit `{EXPECTED_COMMIT}`",
    ],
}


def read_repo_file(rel: str) -> str:
    path = ROOT_DIR / rel
    if not path.is_file():
        raise FileNotFoundError(rel)
    return path.read_text(encoding="utf-8")


def check_text() -> list[str]:
    errors: list[str] = []
    for rel, needles in REQUIRED_TEXT.items():
        try:
            text = read_repo_file(rel)
        except FileNotFoundError:
            errors.append(f"missing file: {rel}")
            continue

        for needle in needles:
            if needle not in text:
                errors.append(f"{rel}: missing {needle!r}")
    return errors


def check_local_inputs() -> list[str]:
    errors: list[str] = []
    assets_commit = read_source_commit(ROOT_DIR / "assets" / "spine-runtimes" / "SOURCE.txt")
    if assets_commit and assets_commit != EXPECTED_COMMIT:
        errors.append(
            "assets/spine-runtimes/SOURCE.txt: "
            f"expected {EXPECTED_COMMIT}, got {assets_commit}"
        )

    cache_head = git_head(ROOT_DIR / ".cache" / "spine-runtimes")
    if cache_head and cache_head != EXPECTED_COMMIT:
        errors.append(
            ".cache/spine-runtimes: "
            f"expected HEAD {EXPECTED_COMMIT}, got {cache_head}"
        )
    return errors


def resolve_remote_ref() -> str:
    if BASELINE.ref_kind == "branch":
        ref = f"refs/heads/{EXPECTED_REV}"
    elif BASELINE.ref_kind == "tag":
        ref = f"refs/tags/{EXPECTED_REV}"
    else:
        ref = EXPECTED_REV

    proc = subprocess.run(
        ["git", "ls-remote", UPSTREAM_REPO, ref],
        cwd=str(ROOT_DIR),
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if proc.returncode != 0:
        raise RuntimeError(proc.stderr.strip() or "git ls-remote failed")
    line = proc.stdout.strip()
    if not line:
        raise RuntimeError(f"missing remote ref: {EXPECTED_REV}")
    return line.split()[0]


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(
        description="Check that spine2d docs and tooling agree on the Spine 4.3 upstream baseline."
    )
    parser.add_argument(
        "--verify-remote",
        action="store_true",
        help="Also verify that the upstream baseline ref resolves to the expected commit.",
    )
    parser.add_argument(
        "--verify-local",
        action="store_true",
        help="Also verify local assets/cache checkouts match the expected commit when present.",
    )
    args = parser.parse_args(argv)

    errors = check_text()
    if args.verify_local:
        errors.extend(check_local_inputs())

    if args.verify_remote:
        try:
            actual = resolve_remote_ref()
        except Exception as exc:
            errors.append(f"remote check failed: {exc}")
        else:
            if actual != EXPECTED_COMMIT:
                errors.append(
                    f"remote {EXPECTED_REV} moved: expected {EXPECTED_COMMIT}, got {actual}"
                )

    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1

    manifest = MANIFEST_PATH.relative_to(ROOT_DIR)
    print(f"Spine baseline OK: {BASELINE.ref_kind} {EXPECTED_REV} @ {EXPECTED_COMMIT} ({manifest})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

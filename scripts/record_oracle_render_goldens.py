#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import List, Optional

from upstream_baseline import commit_for_examples, load_baseline


ROOT_DIR = Path(__file__).resolve().parent.parent
ORACLE_RUNNER = ROOT_DIR / "scripts" / "run_spine_cpp_lite_render_oracle.zsh"
UPSTREAM_BASELINE = load_baseline()
UPSTREAM_DEFAULT_REV = UPSTREAM_BASELINE.rev
UPSTREAM_BASELINE_COMMIT = UPSTREAM_BASELINE.commit
DEFAULT_ORACLE_TIMEOUT_SECONDS = 30


def find_examples_root() -> Path:
    env = os.environ.get("SPINE2D_UPSTREAM_EXAMPLES_DIR", "").strip()
    if env:
        p = Path(env)
        if p.is_dir():
            return p

    candidates = [
        ROOT_DIR / "assets" / "spine-runtimes" / "examples",
        ROOT_DIR / "third_party" / "spine-runtimes" / "examples",
        ROOT_DIR / ".cache" / "spine-runtimes" / "examples",
    ]
    for p in candidates:
        if p.is_dir():
            return p
    raise SystemExit(
        "Missing upstream Spine examples. Run `python3 ./scripts/prepare_spine_runtimes_web_assets.py --scope tests` "
        "or set SPINE2D_UPSTREAM_EXAMPLES_DIR."
    )


def read_pinned_commit(examples_root: Path) -> str:
    return commit_for_examples(examples_root, UPSTREAM_BASELINE)


def _run_oracle(args: List[str], *, timeout_seconds: int) -> str:
    cmd = [str(ORACLE_RUNNER), *args]
    try:
        return subprocess.check_output(
            cmd,
            cwd=str(ROOT_DIR),
            text=True,
            timeout=timeout_seconds,
            stderr=subprocess.STDOUT,
        )
    except subprocess.TimeoutExpired as e:
        out = e.output or ""
        raise RuntimeError(
            f"oracle timed out after {timeout_seconds}s: {' '.join(cmd)}\n{out}"
        ) from e
    except subprocess.CalledProcessError as e:
        out = e.output or ""
        raise RuntimeError(f"oracle failed: {' '.join(cmd)}\n{out}") from e


@dataclass(frozen=True)
class RenderCase:
    name: str
    atlas: str
    skeleton: str
    anim: str
    time: str
    looped: str = "1"
    skin: Optional[str] = None
    physics: str = "none"

    def golden_name(self) -> str:
        anim = self.anim.replace("/", "__")
        return f"{self.name}_{anim}_t{self.time}.json"

@dataclass(frozen=True)
class RenderScenarioCase:
    name: str
    atlas: str
    skeleton: str
    commands: List[str]

    def golden_name(self) -> str:
        return f"{self.name}.json"


def cases_json() -> List[RenderCase]:
    return [
        RenderCase(
            name="coin",
            atlas="coin/export/coin-pma.atlas",
            skeleton="coin/export/coin-pro.json",
            anim="animation",
            time="0_3",
        ),
        RenderCase(
            name="coin_nonpma",
            atlas="coin/export/coin.atlas",
            skeleton="coin/export/coin-pro.json",
            anim="animation",
            time="0_3",
        ),
        RenderCase(
            name="spineboy",
            atlas="spineboy/export/spineboy-pma.atlas",
            skeleton="spineboy/export/spineboy-pro.json",
            anim="run",
            time="0_2",
        ),
        RenderCase(
            name="spineboy_nonpma",
            atlas="spineboy/export/spineboy.atlas",
            skeleton="spineboy/export/spineboy-pro.json",
            anim="run",
            time="0_2",
        ),
        RenderCase(
            name="alien",
            atlas="alien/export/alien-pma.atlas",
            skeleton="alien/export/alien-pro.json",
            anim="run",
            time="0_3",
        ),
        RenderCase(
            name="dragon",
            atlas="dragon/export/dragon-pma.atlas",
            skeleton="dragon/export/dragon-ess.json",
            anim="flying",
            time="0_25",
        ),
        RenderCase(
            name="goblins",
            atlas="goblins/export/goblins-pma.atlas",
            skeleton="goblins/export/goblins-pro.json",
            anim="walk",
            time="0_3",
        ),
        RenderCase(
            name="hero",
            atlas="hero/export/hero-pma.atlas",
            skeleton="hero/export/hero-pro.json",
            anim="idle",
            time="0_55",
        ),
        RenderCase(
            name="hero_nonpma",
            atlas="hero/export/hero.atlas",
            skeleton="hero/export/hero-pro.json",
            anim="idle",
            time="0_55",
        ),
        RenderCase(
            name="mix_and_match_boy_pma",
            atlas="mix-and-match/export/mix-and-match-pma.atlas",
            skeleton="mix-and-match/export/mix-and-match-pro.json",
            anim="walk",
            time="0_1667",
            skin="full-skins/boy",
        ),
        RenderCase(
            name="mix_and_match_girl_pma",
            atlas="mix-and-match/export/mix-and-match-pma.atlas",
            skeleton="mix-and-match/export/mix-and-match-pro.json",
            anim="walk",
            time="0_1667",
            skin="full-skins/girl",
        ),
        RenderCase(
            name="mix_and_match_boy_nonpma",
            atlas="mix-and-match/export/mix-and-match.atlas",
            skeleton="mix-and-match/export/mix-and-match-pro.json",
            anim="walk",
            time="0_1667",
            skin="full-skins/boy",
        ),
        RenderCase(
            name="mix_and_match_girl_nonpma",
            atlas="mix-and-match/export/mix-and-match.atlas",
            skeleton="mix-and-match/export/mix-and-match-pro.json",
            anim="walk",
            time="0_1667",
            skin="full-skins/girl",
        ),
        RenderCase(
            name="vine",
            atlas="vine/export/vine-pma.atlas",
            skeleton="vine/export/vine-pro.json",
            anim="grow",
            time="0_5",
        ),
        RenderCase(
            name="tank",
            atlas="tank/export/tank-pma.atlas",
            skeleton="tank/export/tank-pro.json",
            anim="shoot",
            time="0_3",
        ),
        RenderCase(
            name="chibi",
            atlas="chibi-stickers/export/chibi-stickers-pma.atlas",
            skeleton="chibi-stickers/export/chibi-stickers.json",
            anim="movement/idle-front",
            time="0_3",
        ),
        RenderCase(
            name="chibi_davide_pma",
            atlas="chibi-stickers/export/chibi-stickers-pma.atlas",
            skeleton="chibi-stickers/export/chibi-stickers.json",
            anim="movement/idle-front",
            time="0_3",
            skin="davide",
        ),
        RenderCase(
            name="chibi_davide_nonpma",
            atlas="chibi-stickers/export/chibi-stickers.atlas",
            skeleton="chibi-stickers/export/chibi-stickers.json",
            anim="movement/idle-front",
            time="0_3",
            skin="davide",
        ),
    ]

def scenario_cases_json() -> List[RenderScenarioCase]:
    return [
        RenderScenarioCase(
            name="tank_scn_drive_to_shoot_midmix",
            atlas="tank/export/tank-pma.atlas",
            skeleton="tank/export/tank-pro.json",
            commands=[
                "--mix",
                "drive",
                "shoot",
                "0.2",
                "--set",
                "0",
                "drive",
                "1",
                "--step",
                "0.1",
                "--set",
                "0",
                "shoot",
                "0",
                "--step",
                "0.1",
            ],
        ),
        RenderScenarioCase(
            name="spineboy_scn_idle_to_shoot_midmix",
            atlas="spineboy/export/spineboy-pma.atlas",
            skeleton="spineboy/export/spineboy-pro.json",
            commands=[
                "--mix",
                "idle",
                "shoot",
                "0.2",
                "--set",
                "0",
                "idle",
                "1",
                "--step",
                "0.1",
                "--set",
                "0",
                "shoot",
                "0",
                "--step",
                "0.1",
            ],
        ),
        RenderScenarioCase(
            name="tank_scn_drive_plus_shoot_add_alpha0_5_t0_4",
            atlas="tank/export/tank-pma.atlas",
            skeleton="tank/export/tank-pro.json",
            commands=[
                "--set",
                "0",
                "drive",
                "1",
                "--step",
                "0.1",
                "--set",
                "1",
                "shoot",
                "0",
                "--entry-additive",
                "1",
                "--entry-alpha",
                "0.5",
                "--step",
                "0.3",
            ],
        ),
    ]


def cases_skel() -> List[RenderCase]:
    out = []
    for c in cases_json():
        if c.skeleton.endswith(".json"):
            skel = c.skeleton[:-5] + ".skel"
        else:
            skel = c.skeleton.replace(".json", ".skel")
        out.append(
            RenderCase(
                name=c.name,
                atlas=c.atlas,
                skeleton=skel,
                anim=c.anim,
                time=c.time,
                looped=c.looped,
                skin=c.skin,
                physics=c.physics,
            )
        )
    return out

def scenario_cases_skel() -> List[RenderScenarioCase]:
    out = []
    for c in scenario_cases_json():
        if c.skeleton.endswith(".json"):
            skel = c.skeleton[:-5] + ".skel"
        else:
            skel = c.skeleton.replace(".json", ".skel")
        out.append(
            RenderScenarioCase(
                name=c.name,
                atlas=c.atlas,
                skeleton=skel,
                commands=c.commands,
            )
        )
    return out


def record_one(
    examples_root: Path,
    out_dir: Path,
    case: RenderCase,
    *,
    timeout_seconds: int,
) -> None:
    atlas = examples_root / case.atlas
    skeleton = examples_root / case.skeleton
    if not atlas.is_file():
        raise FileNotFoundError(f"missing atlas: {atlas}")
    if not skeleton.is_file():
        raise FileNotFoundError(f"missing skeleton: {skeleton}")

    time_cli = case.time.replace("_", ".")
    args = [
        str(atlas),
        str(skeleton),
        "--anim",
        case.anim,
        "--time",
        time_cli,
        "--loop",
        case.looped,
        "--physics",
        case.physics,
    ]
    if case.skin is not None:
        args.extend(["--skin", case.skin])

    out = _run_oracle(args, timeout_seconds=timeout_seconds)
    out_path = out_dir / case.golden_name()
    out_path.write_text(out, encoding="utf-8")

def record_one_scenario(
    examples_root: Path,
    out_dir: Path,
    case: RenderScenarioCase,
    *,
    timeout_seconds: int,
) -> None:
    atlas = examples_root / case.atlas
    skeleton = examples_root / case.skeleton
    if not atlas.is_file():
        raise FileNotFoundError(f"missing atlas: {atlas}")
    if not skeleton.is_file():
        raise FileNotFoundError(f"missing skeleton: {skeleton}")

    args = [str(atlas), str(skeleton), *case.commands]
    out = _run_oracle(args, timeout_seconds=timeout_seconds)
    out_path = out_dir / case.golden_name()
    out_path.write_text(out, encoding="utf-8")


def write_source(out_dir: Path, *, commit: str, fmt: str, status: str) -> None:
    now = datetime.now(timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z")
    ref_prefix = "Tag" if UPSTREAM_BASELINE.ref_kind == "tag" else "Branch"
    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / "SOURCE.txt").write_text(
        "\n".join(
            [
                "Source: https://github.com/EsotericSoftware/spine-runtimes",
                f"{ref_prefix}: {UPSTREAM_DEFAULT_REV}",
                f"TargetCommit: {commit}",
                f"RecordedAtUTC: {now}",
                f"Format: {fmt}",
                f"Status: {status}",
                "Notes:",
                "  These files are render-dump goldens produced by the C++ render oracle.",
                "  Both legacy (--anim/--time) and scenario (--set/--step) cases may be present.",
                "  Re-record when the upstream baseline commit changes.",
                "",
            ]
        ),
        encoding="utf-8",
    )


def main() -> int:
    ap = argparse.ArgumentParser(description="Record C++ render oracle goldens for spine2d.")
    ap.add_argument("--formats", choices=["json", "skel", "all"], default="all")
    ap.add_argument("--only", type=str, default="", help="Regex filter on golden filename")
    ap.add_argument("--keep-going", action="store_true")
    ap.add_argument(
        "--timeout-seconds",
        type=int,
        default=DEFAULT_ORACLE_TIMEOUT_SECONDS,
        help=f"Per oracle scenario timeout (default: {DEFAULT_ORACLE_TIMEOUT_SECONDS})",
    )
    args = ap.parse_args()
    only = re.compile(args.only) if args.only else None

    examples_root = find_examples_root()
    commit = read_pinned_commit(examples_root)

    out_json = ROOT_DIR / "spine2d" / "tests" / "golden" / "render_oracle_scenarios"
    out_skel = ROOT_DIR / "spine2d" / "tests" / "golden" / "render_oracle_scenarios_skel"

    failures = 0

    if args.formats in ("json", "all"):
        out_json.mkdir(parents=True, exist_ok=True)
        json_failures = 0
        for c in cases_json():
            if only and not only.search(c.golden_name()):
                continue
            try:
                record_one(
                    examples_root,
                    out_json,
                    c,
                    timeout_seconds=args.timeout_seconds,
                )
                print(f"Wrote {out_json / c.golden_name()}")
            except Exception as e:
                failures += 1
                json_failures += 1
                print(f"FAIL {c.name} (json): {e}", file=sys.stderr)
                if not args.keep_going:
                    write_source(out_json, commit=commit, fmt="json", status="FAILED")
                    return 1
        for c in scenario_cases_json():
            if only and not only.search(c.golden_name()):
                continue
            try:
                record_one_scenario(
                    examples_root,
                    out_json,
                    c,
                    timeout_seconds=args.timeout_seconds,
                )
                print(f"Wrote {out_json / c.golden_name()}")
            except Exception as e:
                failures += 1
                json_failures += 1
                print(f"FAIL {c.name} (json scenario): {e}", file=sys.stderr)
                if not args.keep_going:
                    write_source(out_json, commit=commit, fmt="json", status="FAILED")
                    return 1
        write_source(
            out_json,
            commit=commit,
            fmt="json",
            status="OK" if json_failures == 0 else f"FAILED ({json_failures} failures)",
        )

    if args.formats in ("skel", "all"):
        out_skel.mkdir(parents=True, exist_ok=True)
        skel_failures = 0
        for c in cases_skel():
            if only and not only.search(c.golden_name()):
                continue
            try:
                record_one(
                    examples_root,
                    out_skel,
                    c,
                    timeout_seconds=args.timeout_seconds,
                )
                print(f"Wrote {out_skel / c.golden_name()}")
            except Exception as e:
                failures += 1
                skel_failures += 1
                print(f"FAIL {c.name} (skel): {e}", file=sys.stderr)
                if not args.keep_going:
                    write_source(out_skel, commit=commit, fmt="skel", status="FAILED")
                    return 1
        for c in scenario_cases_skel():
            if only and not only.search(c.golden_name()):
                continue
            try:
                record_one_scenario(
                    examples_root,
                    out_skel,
                    c,
                    timeout_seconds=args.timeout_seconds,
                )
                print(f"Wrote {out_skel / c.golden_name()}")
            except Exception as e:
                failures += 1
                skel_failures += 1
                print(f"FAIL {c.name} (skel scenario): {e}", file=sys.stderr)
                if not args.keep_going:
                    write_source(out_skel, commit=commit, fmt="skel", status="FAILED")
                    return 1
        write_source(
            out_skel,
            commit=commit,
            fmt="skel",
            status="OK" if skel_failures == 0 else f"FAILED ({skel_failures} failures)",
        )

    if failures:
        print(f"Completed with failures: {failures}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

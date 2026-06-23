#!/usr/bin/env python3
from __future__ import annotations

import argparse
import ast
import json
import os
import re
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Optional, Tuple

from upstream_baseline import commit_for_examples, load_baseline


ROOT_DIR = Path(__file__).resolve().parent.parent
TESTS_RS = ROOT_DIR / "spine2d" / "src" / "runtime" / "oracle_scenario_parity_tests.rs"
ORACLE_RUNNER = ROOT_DIR / "scripts" / "run_spine_cpp_lite_oracle.zsh"
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
        "Missing upstream Spine examples. Run "
        f"`python3 ./scripts/prepare_spine_runtimes_web_assets.py --scope tests --rev {UPSTREAM_DEFAULT_REV}` "
        "or set SPINE2D_UPSTREAM_EXAMPLES_DIR."
    )


def find_examples_root_for_scenario(skeleton_rel: str) -> Path | None:
    env = os.environ.get("SPINE2D_UPSTREAM_EXAMPLES_DIR", "").strip()
    if env:
        p = Path(env)
        if (p / skeleton_rel).is_file():
            return p

    candidates = [
        ROOT_DIR / "assets" / "spine-runtimes" / "examples",
        ROOT_DIR / "third_party" / "spine-runtimes" / "examples",
        ROOT_DIR / ".cache" / "spine-runtimes" / "examples",
    ]
    for p in candidates:
        if (p / skeleton_rel).is_file():
            return p
    return None


def strip_rust_strings_for_brace_count(s: str) -> str:
    out = []
    in_str = False
    escape = False
    for ch in s:
        if in_str:
            if escape:
                escape = False
                continue
            if ch == "\\":
                escape = True
                continue
            if ch == '"':
                in_str = False
            continue
        else:
            if ch == '"':
                in_str = True
                continue
            out.append(ch)
    return "".join(out)


def find_matching_brace(text: str, open_index: int) -> int:
    depth = 0
    i = open_index
    while i < len(text):
        chunk = strip_rust_strings_for_brace_count(text[i : i + 4096])
        for j, ch in enumerate(chunk):
            if ch == "{":
                depth += 1
            elif ch == "}":
                depth -= 1
                if depth == 0:
                    # Map j back to original string index approximation.
                    # We used a chunk with strings stripped, so indices differ. Fall back to a slow path.
                    return find_matching_brace_slow(text, open_index)
        i += 4096
    raise ValueError("unbalanced braces")


def find_matching_brace_slow(text: str, open_index: int) -> int:
    depth = 0
    in_str = False
    escape = False
    for i in range(open_index, len(text)):
        ch = text[i]
        if in_str:
            if escape:
                escape = False
                continue
            if ch == "\\":
                escape = True
                continue
            if ch == '"':
                in_str = False
            continue
        else:
            if ch == '"':
                in_str = True
                continue
            if ch == "{":
                depth += 1
            elif ch == "}":
                depth -= 1
                if depth == 0:
                    return i
    raise ValueError("unbalanced braces (slow)")


def safe_eval_f32(expr: str) -> float:
    expr = expr.replace("_", "").strip()
    node = ast.parse(expr, mode="eval")

    def eval_node(n: ast.AST) -> float:
        if isinstance(n, ast.Expression):
            return eval_node(n.body)
        if isinstance(n, ast.Constant) and isinstance(n.value, (int, float)):
            return float(n.value)
        if isinstance(n, ast.UnaryOp) and isinstance(n.op, (ast.UAdd, ast.USub)):
            v = eval_node(n.operand)
            return v if isinstance(n.op, ast.UAdd) else -v
        if isinstance(n, ast.BinOp) and isinstance(
            n.op, (ast.Add, ast.Sub, ast.Mult, ast.Div)
        ):
            a = eval_node(n.left)
            b = eval_node(n.right)
            if isinstance(n.op, ast.Add):
                return a + b
            if isinstance(n.op, ast.Sub):
                return a - b
            if isinstance(n.op, ast.Mult):
                return a * b
            if isinstance(n.op, ast.Div):
                return a / b
        raise ValueError(f"unsupported expr: {expr!r}")

    return float(eval_node(node))


def extract_const_env(body: str) -> Dict[str, float]:
    env: Dict[str, float] = {}
    # Very small evaluator for patterns like: `let dt = 1.0 / 60.0;`
    for m in re.finditer(r"\blet\s+([A-Za-z_][A-Za-z0-9_]*)\s*=\s*([^;]+);", body):
        name = m.group(1)
        expr = m.group(2).strip()
        try:
            env[name] = safe_eval_f32(expr)
        except Exception:
            continue
    return env


def value_to_string(v: float) -> str:
    # Keep the exact literal as much as possible; the oracle prints max_digits10 anyway.
    # For our command line, a reasonable repr is enough.
    return repr(float(v))


def parse_value(token: str, env: Dict[str, float]) -> float:
    token = token.strip()
    if re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", token):
        if token not in env:
            raise ValueError(f"unknown variable: {token}")
        return float(env[token])
    return safe_eval_f32(token)


def parse_bool(token: str) -> str:
    token = token.strip()
    if token == "true":
        return "1"
    if token == "false":
        return "0"
    raise ValueError(f"invalid bool: {token}")


def parse_physics_mode(variant: str) -> str:
    v = variant.strip()
    mapping = {"None": "none", "Reset": "reset", "Update": "update", "Pose": "pose"}
    if v not in mapping:
        raise ValueError(f"invalid Physics: {v}")
    return mapping[v]


def parse_mix_blend_variant(variant: str) -> str:
    v = variant.strip()
    mapping = {
        "Setup": "setup",
        "First": "first",
        "Replace": "replace",
        "Add": "add",
    }
    if v not in mapping:
        raise ValueError(f"invalid MixBlend: {v}")
    return mapping[v]


def parse_commands_no_loops(body: str, env: Dict[str, float]) -> List[str]:
    patterns: List[Tuple[re.Pattern, callable]] = []

    def add(pattern: str, fn):
        patterns.append((re.compile(pattern, re.S), fn))

    add(
        r"(?:state\s*\.\s*data_mut\s*\(\s*\)|[A-Za-z_][A-Za-z0-9_]*)\s*\.\s*set_mix\s*\(\s*\"([^\"]+)\"\s*,\s*\"([^\"]+)\"\s*,\s*([^\)]+?)\s*\)",
        lambda m: ["--mix", m.group(1), m.group(2), value_to_string(parse_value(m.group(3), env))],
    )
    add(
        r"state\s*\.\s*set_animation\s*\(\s*([0-9]+)\s*,\s*\"([^\"]+)\"\s*,\s*(true|false)\s*\)",
        lambda m: ["--set", m.group(1), m.group(2), parse_bool(m.group(3))],
    )
    add(
        r"state\s*\.\s*add_animation\s*\(\s*([0-9]+)\s*,\s*\"([^\"]+)\"\s*,\s*(true|false)\s*,\s*([^\)]+?)\s*\)",
        lambda m: [
            "--add",
            m.group(1),
            m.group(2),
            parse_bool(m.group(3)),
            value_to_string(parse_value(m.group(4), env)),
        ],
    )
    add(
        r"state\s*\.\s*set_empty_animation\s*\(\s*([0-9]+)\s*,\s*([^\)]+?)\s*\)",
        lambda m: ["--set-empty", m.group(1), value_to_string(parse_value(m.group(2), env))],
    )
    add(
        r"state\s*\.\s*add_empty_animation\s*\(\s*([0-9]+)\s*,\s*([^\),]+?)\s*,\s*([^\)]+?)\s*\)",
        lambda m: [
            "--add-empty",
            m.group(1),
            value_to_string(parse_value(m.group(2), env)),
            value_to_string(parse_value(m.group(3), env)),
        ],
    )
    add(
        r"skeleton\s*\.\s*set_skin\s*\(\s*Some\s*\(\s*\"([^\"]+)\"\s*\)\s*\)",
        lambda m: ["--set-skin", m.group(1)],
    )
    add(
        r"skeleton\s*\.\s*set_skin\s*\(\s*None\s*\)",
        lambda m: ["--set-skin", "none"],
    )

    # TrackEntry / last-entry mutations.
    add(
        r"\.\s*set_mix_blend\s*\(\s*&mut\s+state\s*,\s*(?:crate::)?MixBlend::([A-Za-z]+)\s*\)",
        lambda m: ["--entry-mix-blend", parse_mix_blend_variant(m.group(1))],
    )
    add(
        r"\.\s*set_hold_previous\s*\(\s*&mut\s+state\s*,\s*(true|false)\s*\)",
        lambda m: ["--entry-hold-previous", parse_bool(m.group(1))],
    )
    add(
        r"\.\s*set_alpha\s*\(\s*&mut\s+state\s*,\s*([^\)]+?)\s*\)",
        lambda m: ["--entry-alpha", value_to_string(parse_value(m.group(1), env))],
    )
    add(
        r"\.\s*set_event_threshold\s*\(\s*&mut\s+state\s*,\s*([^\)]+?)\s*\)",
        lambda m: ["--entry-event-threshold", value_to_string(parse_value(m.group(1), env))],
    )
    add(
        r"\.\s*set_alpha_attachment_threshold\s*\(\s*&mut\s+state\s*,\s*([^\)]+?)\s*\)",
        lambda m: [
            "--entry-alpha-attachment-threshold",
            value_to_string(parse_value(m.group(1), env)),
        ],
    )
    add(
        r"\.\s*set_mix_attachment_threshold\s*\(\s*&mut\s+state\s*,\s*([^\)]+?)\s*\)",
        lambda m: [
            "--entry-mix-attachment-threshold",
            value_to_string(parse_value(m.group(1), env)),
        ],
    )
    add(
        r"\.\s*set_mix_draw_order_threshold\s*\(\s*&mut\s+state\s*,\s*([^\)]+?)\s*\)",
        lambda m: [
            "--entry-mix-draw-order-threshold",
            value_to_string(parse_value(m.group(1), env)),
        ],
    )
    add(
        r"\.\s*set_reverse\s*\(\s*&mut\s+state\s*,\s*(true|false)\s*\)",
        lambda m: ["--entry-reverse", parse_bool(m.group(1))],
    )
    add(
        r"\.\s*set_shortest_rotation\s*\(\s*&mut\s+state\s*,\s*(true|false)\s*\)",
        lambda m: ["--entry-shortest-rotation", parse_bool(m.group(1))],
    )
    add(
        r"\.\s*reset_rotation_directions\s*\(\s*&mut\s+state\s*\)",
        lambda m: ["--entry-reset-rotation-directions"],
    )

    # Step helpers.
    add(
        r"\bstep\s*\(\s*&mut\s+state\s*,\s*&mut\s+skeleton\s*,\s*([^\)]+?)\s*\)",
        lambda m: ["--physics", "none", "--step", value_to_string(parse_value(m.group(1), env))],
    )
    add(
        r"\bstep_physics\s*\(\s*&mut\s+state\s*,\s*&mut\s+skeleton\s*,\s*([^\)]+?)\s*\)",
        lambda m: [
            "--physics",
            "update",
            "--step",
            value_to_string(parse_value(m.group(1), env)),
        ],
    )
    add(
        r"\bstep_with_physics\s*\(\s*&mut\s+state\s*,\s*&mut\s+skeleton\s*,\s*([^\),]+?)\s*,\s*crate::Physics::([A-Za-z]+)\s*\)",
        lambda m: [
            "--physics",
            parse_physics_mode(m.group(2)),
            "--step",
            value_to_string(parse_value(m.group(1), env)),
        ],
    )
    # Some scenario tests spell out the step sequence explicitly instead of calling `step(...)`.
    # Treat each `state.update(dt)` as an oracle step with physics mode `none`.
    add(
        r"\bstate\s*\.\s*update\s*\(\s*([^\)]+?)\s*\)\s*;",
        lambda m: ["--physics", "none", "--step", value_to_string(parse_value(m.group(1), env))],
    )

    matches: List[Tuple[int, List[str]]] = []
    for pat, fn in patterns:
        for m in pat.finditer(body):
            try:
                tokens = fn(m)
            except Exception as e:
                raise ValueError(f"failed to parse at {m.start()}: {m.group(0)!r}: {e}") from e
            matches.append((m.start(), tokens))

    matches.sort(key=lambda x: x[0])
    flat: List[str] = []
    for _, tokens in matches:
        flat.extend(tokens)
    return flat


def parse_commands(body: str, env: Optional[Dict[str, float]] = None) -> List[str]:
    if env is None:
        env = extract_const_env(body)

    loop_re = re.compile(r"for\s+_\s+in\s+0\.\.([A-Za-z0-9_]+)\s*\{", re.S)
    m = loop_re.search(body)
    if not m:
        return parse_commands_no_loops(body, env)

    start = m.start()
    open_brace = body.find("{", m.end() - 1)
    close_brace = find_matching_brace_slow(body, open_brace)
    before = body[:start]
    inside = body[open_brace + 1 : close_brace]
    after = body[close_brace + 1 :]

    repeat_token = m.group(1)
    repeat = int(parse_value(repeat_token, env))
    if repeat < 0:
        raise ValueError(f"invalid loop repeat: {repeat}")

    cmds: List[str] = []
    cmds.extend(parse_commands(before, env))
    inner_cmds = parse_commands(inside, env)
    for _ in range(repeat):
        cmds.extend(inner_cmds)
    cmds.extend(parse_commands(after, env))
    return cmds


@dataclass(frozen=True)
class Scenario:
    name: str
    skeleton_rel: str
    golden_file: str
    golden_is_skel: bool
    commands: List[str]
    debug_slot: Optional[str]


def extract_scenarios(rs: Path) -> List[Scenario]:
    text = rs.read_text(encoding="utf-8")
    scenarios: List[Scenario] = []

    fn_re = re.compile(r"#\s*\[\s*test\s*\]\s*[\s\S]*?\bfn\s+(oracle_[A-Za-z0-9_]+)\s*\(", re.S)
    for m in fn_re.finditer(text):
        fn_start = m.start()
        name = m.group(1)

        brace_open = text.find("{", m.end())
        if brace_open < 0:
            continue
        brace_close = find_matching_brace_slow(text, brace_open)
        body = text[brace_open + 1 : brace_close]

        skel_match = re.search(
            r'example_json_path\s*\(\s*\"([^\"]+)\"\s*(?:,\s*)?\)', body
        )
        if not skel_match:
            # Some tests build the path first; fallback to full function slice.
            skel_match = re.search(
                r'example_json_path\s*\(\s*\"([^\"]+)\"\s*(?:,\s*)?\)',
                text[fn_start:brace_close],
            )
        if not skel_match:
            continue
        skeleton_rel = skel_match.group(1)

        golden_is_skel = False
        golden_match = re.search(r'golden_path\s*\(\s*\"([^\"]+)\"\s*(?:,\s*)?\)', body)
        if not golden_match:
            golden_match = re.search(
                r'golden_skel_path\s*\(\s*\"([^\"]+)\"\s*(?:,\s*)?\)', body
            )
            if golden_match:
                golden_is_skel = True
        if not golden_match:
            continue
        golden_file = golden_match.group(1)

        debug_slot = None
        dbg = re.search(r'dump_pose\s*\(\s*&skeleton\s*,\s*[^,]+,\s*Some\s*\(\s*\"([^\"]+)\"\s*\)\s*\)', body)
        if dbg:
            debug_slot = dbg.group(1)

        commands = parse_commands(body)
        if debug_slot:
            commands.extend(["--dump-slot-vertices", debug_slot])

        scenarios.append(
            Scenario(
                name=name,
                skeleton_rel=skeleton_rel,
                golden_file=golden_file,
                golden_is_skel=golden_is_skel,
                commands=commands,
                debug_slot=debug_slot,
            )
        )

    # De-dup by golden path (the Rust file has separate json/skel tests).
    unique: Dict[Tuple[str, bool, str], Scenario] = {}
    for s in scenarios:
        key = (s.golden_file, s.golden_is_skel, s.skeleton_rel)
        unique[key] = s
    return list(unique.values())


def iter_atlas_candidates(export_dir: Path) -> List[Path]:
    return sorted(export_dir.glob("*.atlas"))


def run_oracle(
    atlas: Path,
    skeleton: Path,
    commands: List[str],
    *,
    timeout_seconds: int,
) -> str:
    argv = [str(ORACLE_RUNNER), str(atlas), str(skeleton), *commands]
    try:
        proc = subprocess.run(
            argv,
            cwd=str(ROOT_DIR),
            capture_output=True,
            text=True,
            timeout=timeout_seconds,
        )
    except subprocess.TimeoutExpired as e:
        stdout = e.stdout or ""
        stderr = e.stderr or ""
        raise RuntimeError(
            f"oracle timed out after {timeout_seconds}s\n"
            f"atlas: {atlas}\n"
            f"skeleton: {skeleton}\n"
            f"argv: {argv}\n"
            f"stdout:\n{stdout}\n"
            f"stderr:\n{stderr}"
        ) from e
    if proc.returncode != 0:
        raise RuntimeError(
            f"oracle failed (code {proc.returncode})\n"
            f"atlas: {atlas}\n"
            f"skeleton: {skeleton}\n"
            f"argv: {argv}\n"
            f"stdout:\n{proc.stdout}\n"
            f"stderr:\n{proc.stderr}"
        )
    out = proc.stdout.strip()
    json.loads(out)
    return out + "\n"


def load_upstream_commit(examples_root: Path) -> str:
    return commit_for_examples(examples_root, UPSTREAM_BASELINE)


def update_golden_source(status: str, commit: Optional[str]) -> None:
    p = ROOT_DIR / "spine2d" / "tests" / "golden" / "SOURCE.txt"
    now = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    lines = []
    if p.is_file():
        lines = p.read_text(encoding="utf-8").splitlines()
    else:
        lines = [
            "Source: https://github.com/EsotericSoftware/spine-runtimes",
        ]

    def upsert(prefix: str, value: str):
        nonlocal lines
        for i, line in enumerate(lines):
            if line.startswith(prefix):
                lines[i] = f"{prefix}{value}"
                return
        lines.append(f"{prefix}{value}")

    ref_prefix = "Tag: " if UPSTREAM_BASELINE.ref_kind == "tag" else "Branch: "
    stale_ref_prefix = "Branch: " if ref_prefix == "Tag: " else "Tag: "
    lines = [line for line in lines if not line.startswith(stale_ref_prefix)]
    upsert(ref_prefix, UPSTREAM_DEFAULT_REV)
    if commit:
        upsert("TargetCommit: ", commit)
    upsert("RecordedAtUTC: ", now)
    upsert("Status: ", status)
    p.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--only", type=str, default="", help="Regex filter on golden filename")
    ap.add_argument("--formats", choices=["json", "skel", "all"], default="all")
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--keep-going", action="store_true")
    ap.add_argument(
        "--timeout-seconds",
        type=int,
        default=DEFAULT_ORACLE_TIMEOUT_SECONDS,
        help=f"Per oracle scenario timeout (default: {DEFAULT_ORACLE_TIMEOUT_SECONDS})",
    )
    args = ap.parse_args()

    if not TESTS_RS.is_file():
        raise SystemExit(f"Missing: {TESTS_RS}")
    if not ORACLE_RUNNER.is_file():
        raise SystemExit(f"Missing: {ORACLE_RUNNER}")

    examples_root = find_examples_root()
    scenarios = extract_scenarios(TESTS_RS)
    scenarios.sort(key=lambda s: (s.golden_is_skel, s.golden_file, s.skeleton_rel))

    pat = re.compile(args.only) if args.only else None
    selected: List[Scenario] = []
    for s in scenarios:
        if args.formats == "json" and s.golden_is_skel:
            continue
        if args.formats == "skel" and not s.golden_is_skel:
            continue
        if pat and not pat.search(s.golden_file):
            continue
        selected.append(s)
        if args.limit and len(selected) >= args.limit:
            break

    if not selected:
        print("No scenarios selected.")
        return 0

    if not args.dry_run:
        update_golden_source("STALE (recording in progress)", load_upstream_commit(examples_root))

    ok = 0
    failed = 0
    for s in selected:
        scenario_root = find_examples_root_for_scenario(s.skeleton_rel) or examples_root
        skel_path = scenario_root / s.skeleton_rel
        export_dir = skel_path.parent

        golden_dir = (
            ROOT_DIR / "spine2d" / "tests" / "golden" / "oracle_scenarios_skel"
            if s.golden_is_skel
            else ROOT_DIR / "spine2d" / "tests" / "golden" / "oracle_scenarios"
        )
        out_path = golden_dir / s.golden_file

        if not skel_path.is_file():
            print(f"skip (missing skeleton): {s.skeleton_rel}")
            continue

        atlas_candidates = iter_atlas_candidates(export_dir)
        if not atlas_candidates:
            print(f"skip (no atlas): {export_dir}")
            continue

        if args.dry_run:
            print(f"[dry] {out_path.relative_to(ROOT_DIR)} <- {s.skeleton_rel}")
            ok += 1
            continue

        out_path.parent.mkdir(parents=True, exist_ok=True)

        last_err: Optional[Exception] = None
        for atlas in atlas_candidates:
            try:
                payload = run_oracle(
                    atlas,
                    skel_path,
                    s.commands,
                    timeout_seconds=args.timeout_seconds,
                )
                out_path.write_text(payload, encoding="utf-8")
                ok += 1
                last_err = None
                break
            except Exception as e:
                last_err = e
                continue

        if last_err is not None:
            failed += 1
            print(f"FAIL {s.golden_file} ({s.skeleton_rel})\n{last_err}\n")
            if not args.keep_going:
                update_golden_source(
                    "STALE (recording failed)", load_upstream_commit(examples_root)
                )
                return 2

    status = "OK" if failed == 0 else "STALE (recording failed)"
    if not args.dry_run:
        update_golden_source(status, load_upstream_commit(examples_root))

    print(f"done: ok={ok}, failed={failed}, total={len(selected)}")
    return 0 if failed == 0 else 2


if __name__ == "__main__":
    raise SystemExit(main())

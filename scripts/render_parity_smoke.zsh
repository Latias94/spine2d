#!/usr/bin/env zsh
set -euo pipefail

EPS_POS="1e-3"
EPS_UV="1e-5"

while (( $# > 0 )); do
  case "$1" in
    --eps-pos)
      EPS_POS="${2:-}"; shift 2;;
    --eps-uv)
      EPS_UV="${2:-}"; shift 2;;
    -h|--help)
      cat <<'EOF'
Usage:
  scripts/render_parity_smoke.zsh [--eps-pos <float>] [--eps-uv <float>]

Runs a small set of renderer-agnostic render oracle comparisons:
- C++ oracle: spine-cpp SkeletonRenderer
- Rust: spine2d/examples/render_dump (DrawList)
- Compare: scripts/compare_render.py (triangle stream)

Defaults:
  --eps-pos 1e-3
  --eps-uv  1e-5
EOF
      exit 0;;
    *)
      echo "Unknown arg: $1" >&2
      exit 2;;
  esac
done

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/spine2d_render_oracle.XXXXXX")"
trap 'rm -rf "$TMP_DIR"' EXIT

cd "$ROOT_DIR"

echo "Building Rust render dump..."
cargo build -p spine2d --example render_dump --features json >/dev/null
RENDER_DUMP_BIN="$ROOT_DIR/target/debug/examples/render_dump"
RENDER_ORACLE_BIN="$ROOT_DIR/scripts/run_spine_cpp_lite_render_oracle.zsh"

typeset -a scenarios
scenarios=(
  'coin|assets/spine-runtimes/examples/coin/export/coin-pma.atlas|assets/spine-runtimes/examples/coin/export/coin-pro.json|animation|0.3'
  'spineboy|assets/spine-runtimes/examples/spineboy/export/spineboy-pma.atlas|assets/spine-runtimes/examples/spineboy/export/spineboy-pro.json|run|0.2'
  'alien|assets/spine-runtimes/examples/alien/export/alien-pma.atlas|assets/spine-runtimes/examples/alien/export/alien-pro.json|run|0.3'
  'dragon|assets/spine-runtimes/examples/dragon/export/dragon-pma.atlas|assets/spine-runtimes/examples/dragon/export/dragon-ess.json|flying|0.25'
  'goblins|assets/spine-runtimes/examples/goblins/export/goblins-pma.atlas|assets/spine-runtimes/examples/goblins/export/goblins-pro.json|walk|0.3'
  'hero|assets/spine-runtimes/examples/hero/export/hero-pma.atlas|assets/spine-runtimes/examples/hero/export/hero-pro.json|idle|0.55'
  'mix_and_match|assets/spine-runtimes/examples/mix-and-match/export/mix-and-match-pma.atlas|assets/spine-runtimes/examples/mix-and-match/export/mix-and-match-pro.json|walk|0.1667'
  'vine|assets/spine-runtimes/examples/vine/export/vine-pma.atlas|assets/spine-runtimes/examples/vine/export/vine-pro.json|grow|0.5'
  'tank|assets/spine-runtimes/examples/tank/export/tank-pma.atlas|assets/spine-runtimes/examples/tank/export/tank-pro.json|shoot|0.3'
  'chibi|assets/spine-runtimes/examples/chibi-stickers/export/chibi-stickers-pma.atlas|assets/spine-runtimes/examples/chibi-stickers/export/chibi-stickers.json|movement/idle-front|0.3'
)

fail=0
for spec in "${scenarios[@]}"; do
  IFS='|' read -r name atlas skel anim time <<<"$spec"
  echo "== $name (anim=$anim t=$time) =="

  "$RENDER_ORACLE_BIN" \
    "$atlas" "$skel" --set 0 "$anim" 1 --step "$time" > "$TMP_DIR/cpp_${name}.json"

  "$RENDER_DUMP_BIN" \
    "$atlas" "$skel" --set 0 "$anim" 1 --step "$time" > "$TMP_DIR/rust_${name}.json"

  if python3 "$ROOT_DIR/scripts/compare_render.py" \
    "$TMP_DIR/cpp_${name}.json" "$TMP_DIR/rust_${name}.json" \
    --eps-pos "$EPS_POS" --eps-uv "$EPS_UV" \
    --check-colors --check-dark-colors --eps-color 1
  then
    echo "PASS $name"
  else
    echo "FAIL $name" >&2
    fail=1
  fi
  echo
done

exit "$fail"

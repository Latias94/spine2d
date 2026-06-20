#!/usr/bin/env zsh
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  scripts/import_spine_runtimes_examples.zsh [--src <spine-runtimes-path>] [--mode json|export] [--scope tests|all] [--dest <dir>]

Defaults:
  --src  \$SPINE2D_UPSTREAM_RUNTIMES_DIR (if set), else .cache/spine-runtimes
  --mode json
  --scope tests
  --dest assets/spine-runtimes

Modes:
  json    Copy only exported *.json from examples/*/export/.
  export  Copy *.json, *.skel, *.atlas, *.png from examples/*/export/.

Notes:
  - Imported files are NOT committed by default (see .gitignore).
  - Enable smoke tests with:
      cargo test -p spine2d --features json,upstream-smoke
EOF
}

SCRIPT_DIR="${0:a:h}"
ROOT_DIR="${SCRIPT_DIR:h}"

SRC=""
MODE="json"
SCOPE="tests"
DEST="${ROOT_DIR}/assets/spine-runtimes"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --src)
      SRC="${2:-}"
      shift 2
      ;;
    --mode)
      MODE="${2:-}"
      shift 2
      ;;
    --dest)
      DEST="${2:-}"
      shift 2
      ;;
    --scope)
      SCOPE="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ -z "${SRC}" ]]; then
  if [[ -n "${SPINE2D_UPSTREAM_RUNTIMES_DIR:-}" && -d "${SPINE2D_UPSTREAM_RUNTIMES_DIR}/examples" ]]; then
    SRC="${SPINE2D_UPSTREAM_RUNTIMES_DIR}"
  elif [[ -d "${ROOT_DIR}/.cache/spine-runtimes" ]]; then
    SRC="${ROOT_DIR}/.cache/spine-runtimes"
  else
    echo "No --src provided and no default spine-runtimes checkout found." >&2
    echo "Please clone https://github.com/EsotericSoftware/spine-runtimes and pass --src <path>." >&2
    exit 1
  fi
fi

SRC="${SRC%/}"
DEST="${DEST%/}"

if [[ ! -d "${SRC}/examples" ]]; then
  echo "Invalid --src: expected ${SRC}/examples to exist." >&2
  exit 1
fi

mkdir -p "${DEST}"
mkdir -p "${DEST}/examples"

COMMIT="unknown"
REMOTE="unknown"
if [[ -d "${SRC}/.git" ]]; then
  COMMIT="$(git -C "${SRC}" rev-parse HEAD 2>/dev/null || echo unknown)"
  REMOTE="$(git -C "${SRC}" remote get-url origin 2>/dev/null || echo unknown)"
fi

{
  echo "Source: ${REMOTE}"
  echo "Commit: ${COMMIT}"
  echo "ImportedAtUTC: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "Mode: ${MODE}"
} > "${DEST}/SOURCE.txt"

if [[ -f "${SRC}/LICENSE" ]]; then
  cp -f "${SRC}/LICENSE" "${DEST}/LICENSE.spine-runtimes.txt"
fi

typeset -a patterns
case "${MODE}" in
  json)
    patterns=("*.json")
    ;;
  export)
    patterns=("*.json" "*.skel" "*.atlas" "*.png")
    ;;
  *)
    echo "Invalid --mode: ${MODE} (expected json|export)" >&2
    exit 2
    ;;
esac

echo "Importing Spine runtimes examples..."
echo "  src:  ${SRC}"
echo "  dest: ${DEST}"
echo "  mode: ${MODE}"
echo "  scope: ${SCOPE}"

copied=0

typeset -a test_json_files
test_json_files=(
  "alien/export/alien-ess.json"
  "alien/export/alien-pro.json"
  "dragon/export/dragon-ess.json"
  "hero/export/hero-ess.json"
  "hero/export/hero-pro.json"
  "owl/export/owl-pro.json"
  "raptor/export/raptor-pro.json"
  "spinosaurus/export/spinosaurus-ess.json"
  "stretchyman/export/stretchyman-pro.json"
  "speedy/export/speedy-ess.json"
  "windmill/export/windmill-ess.json"
  "celestial-circus/export/celestial-circus-pro.json"
  "chibi-stickers/export/chibi-stickers.json"
  "cloud-pot/export/cloud-pot.json"
  "coin/export/coin-pro.json"
  "6-arcs/export/6-arcs-pro.json"
  "8-follow-through/export/8-follow-through-pro-ball.json"
  "food-app/export/food-app-search-ess.json"
  "goblins/export/goblins-pro.json"
  "snowglobe/export/snowglobe-pro.json"
  "mix-and-match/export/mix-and-match-pro.json"
  "spineboy/export/spineboy-ess.json"
  "spineboy/export/spineboy-pro.json"
  "tank/export/tank-pro.json"
  "vine/export/vine-pro.json"
)

case "${SCOPE}" in
  tests)
    if [[ "${MODE}" == "json" ]]; then
      for rel in "${test_json_files[@]}"; do
        src_file="${SRC}/examples/${rel}"
        if [[ ! -f "${src_file}" ]]; then
          echo "Missing upstream file: ${src_file}" >&2
          exit 1
        fi
        out="${DEST}/examples/${rel}"
        mkdir -p "${out:h}"
        cp -f "${src_file}" "${out}"
        copied=$((copied + 1))
      done
    else
      # In export mode, copy the entire export/ directory for each example referenced by tests.
      typeset -A seen_examples
      for rel in "${test_json_files[@]}"; do
        example="${rel%%/*}"
        seen_examples[${example}]=1
      done
      for example in ${(k)seen_examples}; do
        src_dir="${SRC}/examples/${example}/export"
        if [[ ! -d "${src_dir}" ]]; then
          echo "Missing upstream directory: ${src_dir}" >&2
          exit 1
        fi
        while IFS= read -r -d '' file; do
          rel_path="${file#${SRC}/}"
          out="${DEST}/${rel_path}"
          mkdir -p "${out:h}"
          cp -f "${file}" "${out}"
          copied=$((copied + 1))
        done < <(find "${src_dir}" -type f -print0)
      done
    fi
    ;;
  all)
    for pat in "${patterns[@]}"; do
      while IFS= read -r -d '' file; do
        rel="${file#${SRC}/}"
        out="${DEST}/${rel}"
        mkdir -p "${out:h}"
        cp -f "${file}" "${out}"
        copied=$((copied + 1))
      done < <(find "${SRC}/examples" -type f -path "*/export/*" -name "${pat}" -print0)
    done
    ;;
  *)
    echo "Invalid --scope: ${SCOPE} (expected tests|all)" >&2
    exit 2
    ;;
esac

echo "Done. Copied ${copied} files."

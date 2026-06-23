#!/usr/bin/env zsh
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
EXPECTED_UPSTREAM_HEAD="$(sed -nE 's/^commit = "([^"]+)"/\1/p' "${ROOT_DIR}/spine-upstream.toml" | head -n 1)"

RUNTIMES_DIR="${SPINE2D_UPSTREAM_RUNTIMES_DIR:-}"
if [[ -z "${RUNTIMES_DIR}" ]]; then
  FIRST_EXISTING_RUNTIMES_DIR=""
  for cand in \
    "${ROOT_DIR}/repo-ref/spine-runtimes" \
    "${ROOT_DIR}/.cache/spine-runtimes" \
    "${ROOT_DIR}/third_party/spine-runtimes" \
  ; do
    if [[ -d "${cand}" ]]; then
      if [[ -z "${FIRST_EXISTING_RUNTIMES_DIR}" ]]; then
        FIRST_EXISTING_RUNTIMES_DIR="${cand}"
      fi
      cand_head="$(git -C "${cand}" rev-parse HEAD 2>/dev/null || echo "unknown")"
      if [[ -n "${EXPECTED_UPSTREAM_HEAD}" && "${cand_head}" == "${EXPECTED_UPSTREAM_HEAD}" ]]; then
        RUNTIMES_DIR="${cand}"
        break
      fi
    fi
  done
  if [[ -z "${RUNTIMES_DIR}" ]]; then
    RUNTIMES_DIR="${FIRST_EXISTING_RUNTIMES_DIR}"
  fi
fi

if [[ ! -d "${RUNTIMES_DIR}" ]]; then
  echo "Missing upstream runtimes dir. Set SPINE2D_UPSTREAM_RUNTIMES_DIR to a spine-runtimes checkout." >&2
  exit 2
fi

SPINE_C_INCLUDE="${RUNTIMES_DIR}/spine-c/include"
SPINE_C_SRC="${RUNTIMES_DIR}/spine-c/src"
if [[ ! -f "${SPINE_C_INCLUDE}/spine-c.h" || ! -f "${SPINE_C_SRC}/extensions.cpp" ]]; then
  echo "Missing spine-c sources under: ${RUNTIMES_DIR}/spine-c" >&2
  exit 2
fi

# spine-c is a thin wrapper around spine-cpp; the directory layout differs between upstream branches.
SPINE_CPP_INCLUDE=""
SPINE_CPP_SRC=""
if [[ -d "${RUNTIMES_DIR}/spine-cpp/include" && -d "${RUNTIMES_DIR}/spine-cpp/src/spine" ]]; then
  SPINE_CPP_INCLUDE="${RUNTIMES_DIR}/spine-cpp/include"
  SPINE_CPP_SRC="${RUNTIMES_DIR}/spine-cpp/src/spine"
elif [[ -d "${RUNTIMES_DIR}/spine-cpp/spine-cpp/include" && -d "${RUNTIMES_DIR}/spine-cpp/spine-cpp/src/spine" ]]; then
  SPINE_CPP_INCLUDE="${RUNTIMES_DIR}/spine-cpp/spine-cpp/include"
  SPINE_CPP_SRC="${RUNTIMES_DIR}/spine-cpp/spine-cpp/src/spine"
else
  echo "Missing spine-cpp sources under: ${RUNTIMES_DIR}/spine-cpp" >&2
  exit 2
fi

BUILD_DIR="${ROOT_DIR}/.cache/spine2d-oracle"
mkdir -p "${BUILD_DIR}"
OUT="${BUILD_DIR}/spine_cpp_lite_oracle"
BUILD_KEY_FILE="${OUT}.build-key"

PATCHED_DIR="${BUILD_DIR}/patched-spine-cpp"
PATCHED_SLIDER_CPP="${PATCHED_DIR}/Slider.cpp"

python3 "${ROOT_DIR}/scripts/patch_spine_runtimes_oracle.py" \
  --in "${SPINE_CPP_SRC}/Slider.cpp" \
  --out "${PATCHED_SLIDER_CPP}"

SPINE_CPP_SOURCES=("${SPINE_CPP_SRC}/"*.cpp)
SPINE_CPP_SOURCES=(${SPINE_CPP_SOURCES:#${SPINE_CPP_SRC}/Slider.cpp})

ORACLE_CXXFLAGS=(-std=c++11 -O2 -fno-exceptions -fno-rtti)
ORACLE_LDFLAGS=()
if [[ "${SPINE2D_ORACLE_DEBUG:-0}" == "1" ]]; then
  ORACLE_CXXFLAGS=(-std=c++11 -O0 -g -fno-omit-frame-pointer -fno-exceptions -fno-rtti)
fi
if [[ "${SPINE2D_ORACLE_ASAN:-0}" == "1" ]]; then
  ORACLE_CXXFLAGS+=(-fsanitize=address -fno-omit-frame-pointer)
  ORACLE_LDFLAGS+=(-fsanitize=address)
fi

UPSTREAM_HEAD="$(git -C "${RUNTIMES_DIR}" rev-parse HEAD 2>/dev/null || echo "unknown")"
if [[ -n "${EXPECTED_UPSTREAM_HEAD}" && "${UPSTREAM_HEAD}" != "${EXPECTED_UPSTREAM_HEAD}" ]]; then
  if [[ "${SPINE2D_ORACLE_ALLOW_BASELINE_MISMATCH:-0}" != "1" ]]; then
    echo "spine-runtimes checkout mismatch: expected ${EXPECTED_UPSTREAM_HEAD}, got ${UPSTREAM_HEAD}" >&2
    echo "Run scripts/fetch_spine_runtimes_examples.py for the active spine-upstream.toml baseline." >&2
    echo "Set SPINE2D_ORACLE_ALLOW_BASELINE_MISMATCH=1 only when intentionally using a local C++ checkout as the source reference." >&2
    exit 2
  fi
  echo "warning: using spine-runtimes checkout ${UPSTREAM_HEAD}; spine-upstream.toml pins ${EXPECTED_UPSTREAM_HEAD}" >&2
fi
BUILD_KEY="upstream=${UPSTREAM_HEAD};debug=${SPINE2D_ORACLE_DEBUG:-0};asan=${SPINE2D_ORACLE_ASAN:-0}"
NEEDS_BUILD=0
if [[ ! -x "${OUT}" || "${ROOT_DIR}/scripts/spine_cpp_lite_oracle.cpp" -nt "${OUT}" ]]; then
  NEEDS_BUILD=1
elif [[ "${SPINE2D_ORACLE_FORCE_REBUILD:-0}" == "1" ]]; then
  NEEDS_BUILD=1
elif [[ "${SPINE2D_ORACLE_REBUILD:-0}" == "1" ]]; then
  if [[ ! -f "${BUILD_KEY_FILE}" || "$(cat "${BUILD_KEY_FILE}")" != "${BUILD_KEY}" ]]; then
    NEEDS_BUILD=1
  fi
fi

if [[ "${NEEDS_BUILD}" == "1" ]]; then
  clang++ "${ORACLE_CXXFLAGS[@]}" \
    -I"${SPINE_C_INCLUDE}" \
    -I"${SPINE_C_SRC}" \
    -I"${SPINE_CPP_INCLUDE}" \
    "${ROOT_DIR}/scripts/spine_cpp_lite_oracle.cpp" \
    "${SPINE_C_SRC}/extensions.cpp" \
    "${SPINE_C_SRC}/generated/"*.cpp \
    "${SPINE_CPP_SOURCES[@]}" \
    "${PATCHED_SLIDER_CPP}" \
    "${ORACLE_LDFLAGS[@]}" \
    -o "${OUT}"
  print -r -- "${BUILD_KEY}" > "${BUILD_KEY_FILE}"
fi

exec "${OUT}" "$@"

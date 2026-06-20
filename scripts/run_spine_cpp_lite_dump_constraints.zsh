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
OUT="${BUILD_DIR}/spine_cpp_lite_dump_constraints"
BUILD_KEY_FILE="${OUT}.build-key"

UPSTREAM_HEAD="$(git -C "${RUNTIMES_DIR}" rev-parse HEAD 2>/dev/null || echo "unknown")"
if [[ -n "${EXPECTED_UPSTREAM_HEAD}" && "${UPSTREAM_HEAD}" != "${EXPECTED_UPSTREAM_HEAD}" ]]; then
  echo "spine-runtimes checkout mismatch: expected ${EXPECTED_UPSTREAM_HEAD}, got ${UPSTREAM_HEAD}" >&2
  echo "Run scripts/fetch_spine_runtimes_examples.py for the active spine-upstream.toml baseline." >&2
  exit 2
fi

BUILD_KEY="upstream=${UPSTREAM_HEAD}"
NEEDS_BUILD=0
if [[ ! -x "${OUT}" || "${ROOT_DIR}/scripts/spine_cpp_lite_dump_constraints.cpp" -nt "${OUT}" ]]; then
  NEEDS_BUILD=1
elif [[ "${SPINE2D_ORACLE_FORCE_REBUILD:-0}" == "1" ]]; then
  NEEDS_BUILD=1
elif [[ ! -f "${BUILD_KEY_FILE}" || "$(cat "${BUILD_KEY_FILE}")" != "${BUILD_KEY}" ]]; then
  NEEDS_BUILD=1
fi

if [[ "${NEEDS_BUILD}" == "1" ]]; then
  clang++ -std=c++17 -O2 \
    -I"${SPINE_C_INCLUDE}" \
    -I"${SPINE_C_SRC}" \
    -I"${SPINE_CPP_INCLUDE}" \
    "${ROOT_DIR}/scripts/spine_cpp_lite_dump_constraints.cpp" \
    "${SPINE_C_SRC}/extensions.cpp" \
    "${SPINE_C_SRC}/generated/"*.cpp \
    "${SPINE_CPP_SRC}/"*.cpp \
    -o "${OUT}"
  print -r -- "${BUILD_KEY}" > "${BUILD_KEY_FILE}"
fi

exec "${OUT}" "$@"

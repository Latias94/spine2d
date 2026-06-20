---
title: "fix: Align 4.3 branch baseline and convex clipping"
type: "fix"
date: "2026-06-18"
---

# fix: Align 4.3 branch baseline and convex clipping

## Summary

Pin the active upstream reference to the latest verified `4.3` branch commit and align renderer clipping startup with `spine-cpp` for convex clipping polygons. This keeps parity scripts honest and removes the render-order drift seen in the `coin` render oracle case.

---

## Problem Frame

The repository still contains manifest and documentation text that treats `spine-flutter-4.3.4` as the current baseline, even though the selected core baseline is branch `4.3` at `7fffd822fa17d924276d8727caa87fb98ccf015e`. In render output, `SkeletonClipper::clip_start` always triangulates and decomposes clipping polygons, while upstream `SkeletonClipping::clipStart` keeps convex polygons as a single closed polygon.

---

## Requirements

- R1. `spine-upstream.toml` and `scripts/check_spine_baseline.py` must identify branch `4.3` at `7fffd822fa17d924276d8727caa87fb98ccf015e` as the current upstream baseline.
- R2. Current baseline docs must describe runtime-specific 4.3 tags as auxiliary markers, not the canonical Rust runtime parity baseline.
- R3. `SkeletonClipper::clip_start` must preserve convex clipping polygons as one closed polygon, matching upstream `spine-cpp`.
- R4. Existing render and geometry tests must keep passing, and the `coin` render oracle mismatch must be rechecked.

---

## Key Technical Decisions

- **Pin branch commit instead of runtime-specific tag:** The official 4.3 tags are runtime-package tags, while the pure Rust runtime tracks shared core behavior from `spine-cpp` and `spine-c`.
- **Model convex clipping before decomposition:** Upstream calls `makeClockwise`, uses the convex result to skip triangulation/decomposition, and only decomposes non-convex non-inverse polygons.
- **Defer full inverse clipping parity:** The current fix passes through the `inverse` flag to the clipping seam and records the missing behavior instead of pretending inverse clipping is complete.

---

## Implementation Units

### U1. Baseline Manifest And Docs

- **Goal:** Make the active baseline manifest, checker, and current docs agree on branch `4.3` commit `7fffd822fa17d924276d8727caa87fb98ccf015e`.
- **Requirements:** R1, R2
- **Dependencies:** None
- **Files:** `spine-upstream.toml`, `scripts/check_spine_baseline.py`, `docs/parity.md`, `docs/decisions.md`, `docs/upstream-tests.md`, `docs/parity-4.3-beta.md`, `docs/upstream-audit-4.3-beta.md`
- **Approach:** Replace latest-tag language with branch-commit language while leaving golden `SOURCE.txt` files as generation evidence until they are re-recorded.
- **Patterns to follow:** Existing `scripts/upstream_baseline.py` manifest loader and `docs/plans/2026-06-18-002-chore-refresh-43-baseline-plan.md`.
- **Test scenarios:** Verify local manifest text with `scripts/check_spine_baseline.py`; verify the remote branch still resolves to the pinned commit with `--verify-remote`.
- **Verification:** Baseline checker passes locally and with remote verification.

### U2. Convex Clipping Startup

- **Goal:** Align clipping polygon preparation with upstream for convex clipping attachments.
- **Requirements:** R3, R4
- **Dependencies:** U1
- **Files:** `spine2d/src/geometry.rs`, `spine2d/src/render.rs`, `spine2d/src/geometry_tests.rs`
- **Approach:** Make `make_clockwise` return the upstream convexity result, let `clip_start` accept `convex` and `inverse` flags, and keep convex or flagged-convex polygons as a single closed polygon.
- **Execution note:** Add or update characterization coverage before relying on render oracle parity.
- **Patterns to follow:** `.cache/spine-runtimes/spine-cpp/src/spine/SkeletonClipping.cpp`.
- **Test scenarios:** A convex rectangle clipping polygon should remain a single closed clipping polygon and produce the upstream clipped triangle vertex order; invalid polygons should still be rejected.
- **Verification:** `cargo fmt --check`, geometry/render tests, and the JSON render oracle parity case covering `coin` pass or report only unrelated remaining mismatches.

---

## Risks & Dependencies

- Full `inverse` clipping is still not implemented in Rust and remains a separate parity item.
- The pose golden `SOURCE.txt` still names the older tag until pose goldens are re-recorded; this plan avoids rewriting that evidence without regeneration.

---

## Sources & Research

- `.cache/spine-runtimes/spine-cpp/src/spine/SkeletonClipping.cpp`
- `.cache/spine-runtimes/spine-cpp/src/spine/SkeletonRenderer.cpp`
- `docs/plans/2026-06-18-002-chore-refresh-43-baseline-plan.md`

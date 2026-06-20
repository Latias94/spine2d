---
title: "fix: Align SkeletonClipper polygon clipping output"
type: "fix"
date: "2026-06-18"
execution: "code"
---

# fix: Align SkeletonClipper polygon clipping output

## Summary

Align `SkeletonClipper` triangle clipping with latest Spine 4.3 `SkeletonClipping::clip` output ordering. The current Rust clipper clips the right polygon shape, but its scratch-buffer flow produces a different final vertex order/count for the Spine C reference scenario.

---

## Problem Frame

`cargo nextest run -p spine2d --features json,binary` currently fails two `geometry_tests` cases. Both failures isolate to `SkeletonClipper::clip_triangles`: the rectangle clipping fixture should produce the same vertices, UVs, and indices as upstream Spine C/C++, but Rust returns a shifted/reduced polygon.

---

## Requirements

- R1. `SkeletonClipper::clip_triangle` matches official `SkeletonClipping::clip` scratch/output buffer selection.
- R2. Clipped vertex ordering, UV interpolation, and fan indices match the existing Spine C reference tests.
- R3. Existing triangulator behavior and render clipping tests remain unchanged.

---

## Key Technical Decisions

- **KTD1. Port the buffer-selection semantics, not the public API:** The bug is inside the clipping helper; callers and return shapes stay unchanged.
- **KTD2. Keep tests as the contract:** The failing `geometry_tests` fixtures already encode upstream output, so implementation should make them pass rather than relaxing expectations.

---

## Implementation Units

### U1. Fix clipping helper buffer flow

- **Goal:** Match upstream `SkeletonClipping::clip` initial scratch/output selection and final copy-back behavior.
- **Requirements:** R1, R2.
- **Files:** `spine2d/src/geometry.rs`, `spine2d/src/geometry_tests.rs`.
- **Approach:** Compare Rust `clip_triangle` with upstream `SkeletonClipping::clip`. Preserve the existing helper signature, but select initial input/output buffers based on `clipping_area.len() % 4 >= 2` and copy the final output back into `out` the same way upstream writes `_clipOutput`.
- **Patterns to follow:** `spine-cpp/src/spine/SkeletonClipping.cpp` at `spine-flutter-4.3.4`.
- **Test scenarios:** The existing `skeleton_clipper_clip_triangles_matches_spine_c_unit_test` and `skeleton_clipper_preserves_convex_polygon_order_without_decomposition` must pass. Full `geometry_tests` should pass.
- **Verification:** Focused geometry nextest, `cargo fmt --check`, and the full `spine2d` `json,binary` test suite.

---

## Scope Boundaries

- This plan does not implement inverse clipping.
- This plan does not refresh render oracle goldens.
- This plan does not redesign triangulator decomposition.

---

## Sources / Research

- `spine-cpp/src/spine/SkeletonClipping.cpp` at `spine-flutter-4.3.4`: `clip` chooses initial input/output by clipping polygon length and copies non-original output back into `_clipOutput`.

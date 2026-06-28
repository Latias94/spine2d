---
type: "Work Progress"
title: "TrackEntry mix-percent clamp cleanup"
description: "Work Progress for parity-aware cleanup of TrackEntry mix percentage clamping."
timestamp: 2026-06-28T12:39:42Z
tags: ["spine-cpp", "parity", "core", "refactor"]
source_session: "manual"
---

# Summary

Rechecked `TrackEntry::mix_percent()` against latest-tag local `spine-cpp` and replaced the hand-written post-interpolation bounds branch with `f32::clamp(0.0, 1.0)`.

# Details

- Local reference: `repo-ref/spine-runtimes/spine-cpp/src/spine/AnimationState.cpp` `TrackEntry::mix()` returns `1` when `mix >= 1`, returns the raw mix for linear interpolation, then applies interpolation and clamps values below `0` or above `1`.
- Rust still keeps the pre-interpolation `mix >= 1.0` and linear fast path unchanged.
- The final non-linear interpolation clamp now uses `self.mix_interpolation.apply(mix).clamp(0.0, 1.0)`.
- With constant non-NaN bounds, `f32::clamp(0.0, 1.0)` preserves the same NaN propagation shape as the previous comparison ladder: comparisons are false for NaN and the NaN value is returned.
- Re-running core Clippy shows the previous `manual_clamp` finding is gone.

# Verification

Passed:

- `cargo fmt --all -- --check`
- `cargo check -p spine2d --features json,binary,upstream-smoke`
- `cargo test -p spine2d --features json,binary,upstream-smoke --lib --no-run`
- `git diff --check`

Expected remaining Clippy failures:

- `cargo clippy -p spine2d --features json,binary,upstream-smoke --lib -- -D warnings`
- Remaining errors are only the three `too_many_arguments` apply helpers in `spine2d/src/model.rs` and `spine2d/src/runtime/animation.rs`.

# Next Action

Handle the three remaining `too_many_arguments` findings through a small internal context object or equivalent local refactor that preserves the public `Animation::apply(...)` behavior and the latest-tag C++ timeline application semantics.

# Citations

- `repo-ref/spine-runtimes/spine-cpp/src/spine/AnimationState.cpp`
- `spine2d/src/runtime/animation_state.rs`
- `cargo check -p spine2d --features json,binary,upstream-smoke`
- `cargo test -p spine2d --features json,binary,upstream-smoke --lib --no-run`
- `cargo clippy -p spine2d --features json,binary,upstream-smoke --lib -- -D warnings`

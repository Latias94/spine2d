---
type: "Verification Evidence"
title: "Negative track-time sentinel parity regressions"
description: "Verification Evidence for Negative track-time sentinel parity regressions."
timestamp: 2026-06-23T20:37:05Z
tags: ["spine2d", "parity", "regression"]
source_session: "019eefe2-c109-7152-92fc-1bd3b4a3cbbf"
---

# Verification

# Context

Validated the negative-track-time sentinel regressions after aligning `-1` as the only unapplied sentinel.

# Result

Passed.

# Evidence

- Focused regressions:
  - `negative_track_end_clears_when_track_last_passes_it`
  - `completed_hold_mix_chain_with_negative_track_time_still_detaches`
  - `set_animation_same_animation_with_negative_track_time_still_mixes_from_applied_entry`
- `cargo fmt --all --check`
- `cargo nextest run -p spine2d --features json,binary,upstream-smoke --no-fail-fast --status-level fail` (`617 passed, 10 skipped`)
- `cargo nextest run -p spine2d-bevy --no-fail-fast --status-level fail` (`43 passed, 0 skipped`)
- `cargo check -p spine2d --features json,binary,upstream-smoke`
- `git diff --check`

# Follow-up

Keep the sentinel checks exact for queue activation, `wasApplied`, and mix cleanup.

# Citations

- [Plan](../plans/2026-06-23-001-refactor-spine-cpp-parity-hardening-plan.md)
- `spine2d/src/runtime/animation_state.rs`
- `spine2d/src/runtime/animation_state_tests.rs`

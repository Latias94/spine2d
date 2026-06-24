---
type: "Verification Evidence"
title: "TrackEnd and mixing-from sentinel parity"
description: "Verification Evidence for TrackEnd and mixing-from sentinel parity."
timestamp: 2026-06-23T20:34:22Z
tags: ["spine2d", "parity", "verification"]
source_session: "019eefe2-c109-7152-92fc-1bd3b4a3cbbf"
---

# Verification

# Context

Validated the outer `AnimationState::update` cleanup path after the hold-mix / mixing-from sentinel fixes.

# Result

Passed.

# Evidence

- `cargo nextest run -p spine2d --features json,binary,upstream-smoke --no-fail-fast --status-level fail` (`613 passed, 10 skipped`)
- `cargo nextest run -p spine2d --features json,binary,upstream-smoke runtime::animation_state_mixing_semantics_tests --no-fail-fast --status-level fail` (`14 passed, 609 skipped`)
- `cargo nextest run -p spine2d --features json,binary,upstream-smoke runtime::animation_state_tests --no-fail-fast --status-level fail` (`69 passed, 554 skipped`)
- `cargo nextest run -p spine2d-bevy --no-fail-fast --status-level fail` (`43 passed, 0 skipped`)
- `cargo fmt --all --check`
- `git diff --check`
- Regression: `completed_hold_mix_chain_detaches_current_mixing_from`

# Follow-up

Keep auditing `computeHold` / timeline mode edges and delete any remaining compatibility shells.

# Citations

- [Plan](../plans/2026-06-23-001-refactor-spine-cpp-parity-hardening-plan.md)
- `spine2d/src/runtime/animation_state.rs`
- `spine2d/src/runtime/animation_state_mixing_semantics_tests.rs`

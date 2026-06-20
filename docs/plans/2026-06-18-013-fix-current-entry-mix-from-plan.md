---
title: "fix: Align current entry MixFrom mapping"
type: "fix"
date: "2026-06-18"
execution: "code"
---

# fix: Align current entry MixFrom mapping

## Summary

Use the computed upstream-style `Current`/`Setup`/`First` timeline mode when applying the current entry, not only when applying `mixingFrom`. After plan 012, `compute_hold` now preserves these modes, but current-entry application still collapses `First` to setup and lets `entry.additive` bypass timeline modes.

---

## Problem Frame

Latest spine-cpp applies current-entry timelines with `MixFrom` from `timelineMode` unless the entry is the base track at alpha 1, where it uses setup directly. The Rust runtime still uses a broader `special_case` that includes additive entries, then applies `blend` for `Current` and setup for every other mode. This loses `MixFrom_First` and can perturb add-to-empty, threshold, and HoldMix scenarios.

---

## Requirements

- R1. Current-entry apply must map `TimelineMode::Current` to the caller's current blend, `Setup` to setup, and `First` to first.
- R2. `TrackEntry::additive` must only control additive timeline math, not bypass `timelineMode`.
- R3. Attachment timelines must receive the same computed current-entry blend as other timeline types.
- R4. Core tests must remain green, and upstream-smoke should shrink or keep the same 26-failure set.

---

## Implementation Units

### U1. Use timeline mode during current-entry apply

- **Goal:** Replace the current `special_case || Current` blend shortcut with explicit `timeline_mode_blend`.
- **Requirements:** R1, R2, R3.
- **Files:** `spine2d/src/runtime/animation_state.rs`.
- **Approach:** Limit the full-setup special case to base track alpha-1 application. Otherwise read `TimelineApplyMode.from` and convert it with `timeline_mode_blend`. Pass this blend into attachment, sequence, draw order, and numeric timeline calls.
- **Test scenarios:** Existing upstream-smoke add-to-empty and threshold scenarios should no longer receive setup blending when upstream would use first/current.
- **Verification:** `cargo nextest run -p spine2d --features json,binary --no-fail-fast` passes; upstream-smoke count is recorded.

### U2. Refresh failure classification

- **Goal:** Re-run upstream-smoke and choose the next dominant failure cluster.
- **Requirements:** R4.
- **Files:** `spine2d/src/runtime/oracle_scenario_parity_tests.rs`.
- **Approach:** Compare pass/fail count against the 419 passed / 26 failed baseline from plan 012. If the count improves, use the new remaining set for the next plan. If it regresses, inspect the first regression before proceeding.
- **Test scenarios:** Full upstream-smoke summary is available for the next iteration.
- **Verification:** Final notes include the remaining failure categories.

---

## Sources / Research

- `.cache/spine-runtimes/spine-cpp/src/spine/AnimationState.cpp`: current-entry apply uses `MixFrom mixFrom = (MixFrom)(timelineMode[ii] & Mode)` for all non-base-track-alpha-1 cases.
- `.cache/spine-runtimes/spine-cpp/src/spine/AnimationState.cpp`: `_additive` is passed separately to `timeline->apply`; it does not force `MixFrom_Current`.

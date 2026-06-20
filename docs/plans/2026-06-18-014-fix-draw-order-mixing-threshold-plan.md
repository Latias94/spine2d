---
title: "fix: Align draw order mixing thresholds"
type: "fix"
date: "2026-06-18"
execution: "code"
---

# fix: Align draw order mixing thresholds

## Summary

Align outgoing draw order timelines with latest spine-cpp `applyMixingFrom` semantics. The Rust runtime currently routes threshold decisions through `MixDirection::Out`, which loses the official distinction between `out=true` and `MixFrom_Current`.

---

## Problem Frame

The remaining upstream-smoke failures cluster around tank draw order thresholds and HoldMix chains. In spine-cpp, outgoing draw order timelines are skipped only when the draw-order threshold has expired and the timeline's `MixFrom` is `Current`; otherwise the timeline is applied with an `out` flag that decides whether setup restoration is allowed. Rust currently maps that condition to `MixDirection::Out`, so `MixFrom_First` and `MixFrom_Setup` can be skipped or restored differently from upstream.

---

## Requirements

- R1. Outgoing draw order timelines must preserve `MixFrom_Current`, `MixFrom_Setup`, and `MixFrom_First` behavior independently from threshold expiry.
- R2. `DrawOrderTimeline` must no-op for threshold-expired `Current` mode, reset for threshold-expired `Setup` or `First`, and apply keyed order while threshold-retained.
- R3. `DrawOrderFolderTimeline` must follow the same `out` and `MixFrom_Current` rules as upstream.
- R4. Core tests must remain green, and upstream-smoke must reduce or keep the current 21-failure set without regressions.

---

## Key Technical Decisions

- **KTD1. Model draw order `out` separately from `MixDirection`:** `MixDirection::Out` means mix-out for many timeline types, but spine-cpp draw order has a separate `out` boolean. Passing a boolean into draw order helpers keeps the official branch structure visible.
- **KTD2. Keep the change inside `AnimationState` and draw order helpers:** No public API changes are needed for this slice.
- **KTD3. Verify with oracle scenarios first:** The failing tank threshold scenarios are stronger regression coverage than synthetic unit tests because they compare against the C++ oracle output.

---

## Implementation Units

### U1. Split draw order out semantics

- **Goal:** Add helper-level support for upstream's draw order `out` boolean without changing other timeline application paths.
- **Requirements:** R1, R2, R3.
- **Files:** `spine2d/src/runtime/animation.rs`, `spine2d/src/runtime/animation_state.rs`.
- **Approach:** Let draw order helpers receive whether the timeline is being applied as `out`. For `out=true` or pre-first-frame, reset only when the blend is not current. For `out=false`, apply the keyed order or setup order exactly as before.
- **Test scenarios:** `tank_shoot_to_shoot_mixDrawOrderThreshold_0_t0_4` should stop differing due to draw order restoration when the outgoing timeline is still retained.
- **Verification:** Targeted upstream-smoke draw order threshold tests pass or move to a non-draw-order numeric failure.

### U2. Mirror `applyMixingFrom` skip gate

- **Goal:** Skip outgoing draw order application only for `!drawOrder && MixFrom_Current`, matching spine-cpp.
- **Requirements:** R1, R2, R3.
- **Files:** `spine2d/src/runtime/animation_state.rs`.
- **Approach:** In `apply_mixing_from_pose`, compute `timeline_blend` from `timeline_mode`. For draw order timelines, skip when threshold has expired and the blend maps to current mode; otherwise call the draw order helper with `out = !draw_order || current-mode`.
- **Test scenarios:** Tank `mixDrawOrderThreshold_0` and `mixDrawOrderThreshold_1` JSON/SKEL cases should converge with C++ oracle behavior.
- **Verification:** Targeted threshold cases pass before broader upstream-smoke rerun.

### U3. Reclassify remaining failures

- **Goal:** Measure the post-fix failure set and choose the next smallest parity slice.
- **Requirements:** R4.
- **Files:** `spine2d/src/runtime/oracle_scenario_parity_tests.rs`.
- **Approach:** Run core tests, then upstream-smoke. If draw order failures clear, prioritize remaining attachment threshold or additive-to-empty numeric drift.
- **Test scenarios:** Full upstream-smoke summary is available for the next plan.
- **Verification:** Final notes include pass/fail counts and remaining failure categories.

---

## Sources / Research

- `.cache/spine-runtimes/spine-cpp/src/spine/AnimationState.cpp`: `applyMixingFrom` skips `DrawOrderTimeline` only for `!drawOrder && mixFrom == MixFrom_Current`, then passes `out = !drawOrder || !DrawOrderTimeline || mixFrom == MixFrom_Current`.
- `.cache/spine-runtimes/spine-cpp/src/spine/DrawOrderTimeline.cpp`: `out || time < first` resets only when `from != MixFrom_Current`.
- `.cache/spine-runtimes/spine-cpp/src/spine/DrawOrderFolderTimeline.cpp`: folder draw order uses the same `out || time < first` plus `from != MixFrom_Current` rule.

---
title: "fix: Align TrackEntry additive API"
type: "fix"
date: "2026-06-18"
execution: "code"
status: "superseded"
---

# fix: Align TrackEntry additive API

> Superseded on 2026-06-23 by latest local `repo-ref/spine-runtimes` C++ evidence. The active C++ reference exposes `TrackEntry::getMixBlend/setMixBlend(MixBlend)` and defaults `_mixBlend` to `MixBlend_Replace`; commit `1a432d3` restored the public Rust API to `mix_blend` and removed the Rust-only additive public surface.

## Summary

Replace the stale TrackEntry `mixBlend` public surface with latest upstream `additive` semantics. Latest `spine-flutter-4.3.4` exposes `TrackEntry::additive`, not per-entry `mixBlend`; keeping both causes Rust-only behavior and mismatches in Add, mix-out, HoldMix, deform, and threshold oracle scenarios.

---

## Problem Frame

After removing `holdPrevious`, latest pose/render oracles can be recorded successfully, but `upstream-smoke` still fails across Add/mix-out scenarios. The common signal is that the Rust runtime treats `MixBlend::Add` as a TrackEntry-level mode while upstream now treats TrackEntry as a boolean `additive` flag plus per-timeline `MixFrom` modes computed by `AnimationState::computeHold`.

---

## Requirements

- R1. Public `TrackEntryHandle` should expose latest `set_additive`, not stale `set_mix_blend`.
- R2. CLI/oracle scenario commands should use `--entry-additive <0|1>` instead of `--entry-mix-blend`.
- R3. `AnimationState::animations_changed`, `compute_hold`, `apply`, and `apply_mixing_from_pose` should use upstream additive semantics directly.
- R4. Existing Add oracle scenario names may stay as regression labels, but their implementation must drive `additive=true`.
- R5. Core tests and oracle re-recorders must stay green after the API break.

---

## Key Technical Decisions

- **KTD1. Delete the compatibility API:** `set_mix_blend` is not in latest official `TrackEntry`; keeping it creates a shallow, misleading interface.
- **KTD2. Keep `MixBlend` internal:** timeline functions still need `Setup/First/Replace/Add`-style internal blend choices, so only TrackEntry's public surface is removed.
- **KTD3. Mechanical test migration first:** update tests and scenario extraction to `set_additive(true)` before changing deeper mix logic, so remaining failures point at runtime behavior rather than stale command generation.

---

## Implementation Units

### U1. Remove TrackEntry mixBlend surface

- **Goal:** Delete `TrackEntry::mix_blend` and `TrackEntryHandle::set_mix_blend`; make additive the only public TrackEntry overlay flag.
- **Requirements:** R1, R3.
- **Files:** `spine2d/src/runtime/animation_state.rs`.
- **Approach:** Replace `effective_additive()` with direct `additive`. Current entry apply should choose upstream `MixFrom_First` on track 0 or `MixFrom_Current` for higher tracks via existing internal `MixBlend` mapping. `compute_hold` should mirror upstream `add && timeline.additive` and `to.additive && timeline.additive` checks.
- **Test scenarios:** Existing unit tests compile only through `set_additive`; no public `set_mix_blend` call remains.
- **Verification:** `cargo nextest run -p spine2d --features json,binary` passes.

### U2. Migrate oracle and helper command surfaces to additive

- **Goal:** Stop producing stale `--entry-mix-blend` commands for TrackEntry overlay behavior.
- **Requirements:** R2, R4, R5.
- **Files:** `spine2d/src/runtime/oracle_scenario_parity_tests.rs`, `spine2d/src/render_oracle_parity_tests.rs`, `spine2d/src/runtime/upstream_spine_skel_smoke_tests.rs`, `spine2d/examples/pose_dump_scenario.rs`, `scripts/record_oracle_goldens.py`, `scripts/spine_cpp_lite_oracle.cpp`, `scripts/spine_cpp_lite_render_oracle.cpp`.
- **Approach:** Replace `set_mix_blend(... MixBlend::Add)` with `set_additive(... true)`. Remove parsing of `MixBlend::Setup/First/Replace` as entry commands; support `--entry-additive <0|1>` in Rust and C++ oracle helpers.
- **Test scenarios:** Pose oracle recorder emits `--entry-additive`; render oracle scenarios still execute Add overlays; no `entry-mix-blend` references remain in code.
- **Verification:** Pose/render oracle re-recorders complete and `rg` shows no code-level `set_mix_blend`/`entry-mix-blend`.

### U3. Re-run parity and classify remaining failures

- **Goal:** Measure how much of the 101-failure upstream-smoke cluster disappears after additive alignment.
- **Requirements:** R5.
- **Files:** `spine2d/tests/golden/oracle_scenarios/`, `spine2d/tests/golden/oracle_scenarios_skel/`, `spine2d/tests/golden/render_oracle_scenarios/`, `spine2d/tests/golden/render_oracle_scenarios_skel/`.
- **Approach:** Re-record pose and render oracles after migration, then run upstream-smoke. Remaining failures should be grouped into the next plan by common runtime seam.
- **Test scenarios:** Full pose and render recorders complete; upstream-smoke summary improves or yields a smaller, clearer failure class.
- **Verification:** `upstream-smoke` result and failure grouping are recorded in the final summary.

---

## Scope Boundaries

- This plan does not rename historical golden files whose names contain `add`; those names describe the scenario intent and keep diffs smaller.
- This plan does not remove the internal `MixBlend` enum because timeline application still needs the internal blend modes.
- This plan does not claim full AnimationState parity; it addresses one stale TrackEntry interface and the high-signal Add failure cluster.

---

## Sources / Research

- `.cache/spine-runtimes/spine-cpp/include/spine/AnimationState.h` at `spine-flutter-4.3.4`: `TrackEntry` exposes `getAdditive`/`setAdditive` and no `mixBlend`.
- `.cache/spine-runtimes/spine-c/src/generated/track_entry.h` at `spine-flutter-4.3.4`: generated C API exposes `spine_track_entry_set_additive`.
- `.cache/spine-runtimes/spine-cpp/src/spine/AnimationState.cpp`: `computeHold`, current apply, and `applyMixingFrom` use `_additive` plus per-timeline `MixFrom` modes.

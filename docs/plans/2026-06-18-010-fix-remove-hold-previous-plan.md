---
title: "fix: Remove stale TrackEntry holdPrevious"
type: "fix"
date: "2026-06-18"
execution: "code"
---

# fix: Remove stale TrackEntry holdPrevious

## Summary

Remove the stale `TrackEntry::hold_previous` behavior surface because latest Spine 4.3 tag `spine-flutter-4.3.4` no longer exposes `holdPrevious` in `spine-cpp` or the generated C API.

---

## Problem Frame

Latest tag oracle regeneration fails only for scenarios that call `set_hold_previous`. The official `TrackEntry` now exposes additive, reverse, shortest rotation, thresholds, and mix interpolation, but no holdPrevious getter or setter. Keeping this Rust-only behavior makes parity tests unrecordable and preserves an upstream-incompatible API.

---

## Requirements

- R1. Public Rust `TrackEntryHandle` must not expose `set_hold_previous`.
- R2. `AnimationState::compute_hold` must remove the special `to.hold_previous` branch and rely on latest upstream additive/instant hold decisions.
- R3. Oracle scenario extraction must stop producing `--entry-hold-previous`.
- R4. Tests and oracle scenarios that exist only to cover `holdPrevious` must be deleted or rewritten to latest upstream behavior.
- R5. Pose oracle regeneration must no longer fail on `--entry-hold-previous`.

---

## Key Technical Decisions

- **KTD1. Delete instead of compatibility-shimming:** The project targets latest upstream behavior and accepts breaking changes, so a Rust-only compatibility field should not stay behind a deprecated method.
- **KTD2. Keep additive hold work separate:** Latest upstream still uses additive/instant timeline metadata in `computeHold`; this plan removes only the deleted `holdPrevious` surface.
- **KTD3. Remove unrecordable oracle cases:** Goldens that require a nonexistent upstream control are not valid parity evidence for the latest tag.

---

## Implementation Units

### U1. Remove the runtime holdPrevious surface

- **Goal:** Delete the stale field, setter, and special hold branch from `AnimationState`.
- **Requirements:** R1, R2.
- **Files:** `spine2d/src/runtime/animation_state.rs`, `spine2d/src/runtime/animation_state_mixing_semantics_tests.rs`.
- **Approach:** Remove `hold_previous` from `TrackEntry`, remove `TrackEntryHandle::set_hold_previous`, and delete the early `to_hold_previous` branch from `compute_hold`. Remove focused tests whose only purpose is `holdPrevious`.
- **Test scenarios:** Existing additive, reverse, shortest rotation, threshold, and event tests still compile and pass without the method.
- **Verification:** `cargo nextest run -p spine2d --features json,binary` compiles without `set_hold_previous`.

### U2. Remove holdPrevious oracle scenarios

- **Goal:** Stop generating or testing latest-incompatible oracle cases.
- **Requirements:** R3, R4, R5.
- **Files:** `scripts/record_oracle_goldens.py`, `scripts/spine_cpp_lite_oracle.cpp`, `spine2d/src/runtime/oracle_scenario_parity_tests.rs`, `spine2d/tests/golden/oracle_scenarios/`, `spine2d/tests/golden/oracle_scenarios_skel/`.
- **Approach:** Remove parser support for `set_hold_previous` commands, delete oracle tests that call the removed method, and remove stale golden files for those scenarios.
- **Test scenarios:** Full pose oracle regeneration reports zero `--entry-hold-previous` failures; upstream-smoke no longer tries to run deleted scenarios.
- **Verification:** `SPINE2D_ORACLE_REBUILD=1 python3 scripts/record_oracle_goldens.py --formats all --keep-going` completes without holdPrevious failures.

---

## Scope Boundaries

- This plan does not remove hold modes that still exist internally for latest upstream mixing semantics.
- This plan does not change additive, reverse, shortest rotation, threshold, or mix interpolation APIs.
- This plan does not claim all AnimationState parity is complete; it removes one obsolete behavior surface blocking latest oracle refresh.

---

## Sources / Research

- `.cache/spine-runtimes/spine-cpp/include/spine/AnimationState.h` at `spine-flutter-4.3.4`: `TrackEntry` exposes additive, reverse, shortest rotation, thresholds, and mix interpolation, but no `holdPrevious`.
- `.cache/spine-runtimes/spine-c/src/generated/track_entry.h` at `spine-flutter-4.3.4`: generated C API has no holdPrevious getter or setter.
- Full pose oracle regeneration failed only on scenarios containing `--entry-hold-previous`.

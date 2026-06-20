---
title: "fix: Align attachment retain current mixing"
type: "fix"
date: "2026-06-18"
execution: "code"
---

# fix: Align attachment retain current mixing

## Summary

Fix the next high-signal upstream-smoke failure after the TrackEntry additive API migration: a stale test expected `crosshair` to survive `aim -> shoot` additive mixing, but latest spine-cpp clears it. While verifying this, preserve upstream's three non-hold timeline modes (`Current`, `Setup`, `First`) instead of collapsing them into Rust's older first/subsequent model.

---

## Problem Frame

Latest upstream `AnimationState::applyAttachmentTimeline` treats `retain=false` plus `MixFrom_Current` as a no-op, but also clears setup-mode outgoing attachments at the end of `AnimationState::apply`. A direct C++ oracle run for `run + aim(additive) -> shoot(additive)` reports `crosshair=None` and `muzzle-glow=muzzle-glow`. The Rust test expectation was stale; the runtime should keep official mode distinctions so future attachment/threshold work can reason from the same model as spine-cpp.

---

## Requirements

- R1. Attachment timelines in current-mix mode must no-op when attachments are not retained.
- R2. Internal timeline modes must preserve upstream `Current`, `Setup`, and `First` semantics instead of flattening `Current` into `Subsequent`.
- R3. The targeted `spineboy aim(additive) -> shoot(additive)` upstream-smoke parity test must assert `crosshair=None`, matching C++ oracle output.
- R4. Core non-upstream tests must remain green.

---

## Key Technical Decisions

- **KTD1. Fix both the mode and the helper:** `apply_attachment` is the module seam used by `Animation::apply`, `AnimationState::apply`, and mixing-from paths, but it only works if callers pass a mode that distinguishes upstream `Current` from `First`.
- **KTD2. Treat `MixBlend::Replace` as `MixFrom_Current`:** The Rust internal mapping already uses `MixBlend` as the public representation of upstream `MixFrom`; for attachment timelines, `Replace` is the current-pose mode that should no-op when not retained.
- **KTD3. Keep the slice narrow:** Deform and draw order failures stay for the next plan unless this fix naturally changes their result. This plan targets attachment loss only.

---

## Implementation Units

### U1. Align timeline current/first modes

- **Goal:** Preserve upstream `Current`, `Setup`, and `First` modes inside `AnimationState` so current-entry and mixing-from application can make the same attachment decisions as spine-cpp.
- **Requirements:** R1, R2.
- **Files:** `spine2d/src/runtime/animation_state.rs`.
- **Approach:** Replace the stale `Subsequent` naming with `Current`, keep `First` distinct from `Setup`, and map modes to `MixBlend` explicitly during both current-entry apply and mixing-from apply.
- **Test scenarios:** The `aim(additive) -> shoot(additive)` path computes current-pose mode for the outgoing `crosshair` attachment timeline and does not collapse it to setup.
- **Verification:** The targeted upstream-smoke test no longer clears `crosshair`.

### U2. Align attachment retain/current behavior and stale test

- **Goal:** Make `apply_attachment` return early for non-retained `MixBlend::Replace` attachment timelines, matching upstream `retain=false && MixFrom_Current`, and update the stale test expectation for setup-mode mix-out.
- **Requirements:** R1, R2.
- **Files:** `spine2d/src/runtime/animation.rs`, `spine2d/src/runtime/examples_pose_parity_tests.rs`.
- **Approach:** Add the no-op guard after inactive-bone/empty-frame checks and before setup/keyframe selection. Keep existing setup restoration for `Setup` and `First`. Use the existing upstream-smoke example as the regression test rather than adding an artificial unit test first.
- **Test scenarios:** `example_spineboy_aim_to_shoot_additive_mixing_clears_crosshair_and_keeps_rgba_colors` clears `crosshair`, keeps `muzzle-glow`, and matches the C++ oracle color.
- **Verification:** The targeted test passes and core `cargo nextest run -p spine2d --features json,binary --no-fail-fast` remains green.

### U3. Reclassify remaining upstream-smoke failures

- **Goal:** Measure whether attachment-retain alignment reduces the 97-failure cluster and identify the next dominant module.
- **Requirements:** R3, R4.
- **Files:** `spine2d/src/runtime/oracle_scenario_parity_tests.rs`, `spine2d/tests/golden/oracle_scenarios/`, `spine2d/tests/golden/oracle_scenarios_skel/`.
- **Approach:** Run targeted upstream-smoke first, then core tests. If the targeted fix is stable, rerun upstream-smoke and group remaining failures by scenario names.
- **Test scenarios:** The corrected attachment parity test passes; upstream-smoke summary is recorded for the next plan.
- **Verification:** Final summary includes pass/fail counts and the next recommended slice.

---

## Sources / Research

- `.cache/spine-runtimes/spine-cpp/src/spine/AnimationState.cpp`: `applyAttachmentTimeline` returns immediately when `!retain && attachmentState == AttachRetain`, and returns immediately for setup restoration when `from == MixFrom_Current`.
- `.cache/spine-runtimes/spine-cpp/src/spine/AnimationState.cpp`: `applyMixingFrom` passes `retainAttachments && alpha >= alphaAttachmentThreshold` into attachment timelines, so non-retained current-mode outgoing timelines must not reset the slot.
- Direct C++ oracle command for `run + aim(additive) -> shoot(additive)` at `t=0.4`: `crosshair=None`, `muzzle-glow=muzzle-glow`, `muzzle-glow.color=[1,0.883626044,0.826886117,0.5]`.

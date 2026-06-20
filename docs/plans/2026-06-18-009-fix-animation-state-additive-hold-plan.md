---
title: "fix: Align AnimationState additive hold modes"
type: fix
date: 2026-06-18
---

# fix: Align AnimationState additive hold modes

## Summary

Align `AnimationState::compute_hold` with upstream Spine 4.3 when additive entries participate in a mixing chain. The fix models the upstream per-timeline additive and instant flags inside the local timeline-kind abstraction, then uses them to avoid holding numeric timelines when the next entry is additive.

---

## Problem Frame

`TrackEntry.additive` is now exposed and applied, but hold-mode computation still treats any next entry with the same property as a normal replacement. Upstream `AnimationState::computeHold` does not hold an outgoing timeline when the incoming entry is additive and the outgoing timeline supports additive blending; otherwise a base animation can be faded or held against an additive overlay instead of letting the overlay add on top of the current pose.

---

## Requirements

- R1. `compute_hold` must classify additive-capable timelines separately from instant timelines, matching upstream `Timeline.getAdditive()` and `Timeline.getInstant()`.
- R2. A mixing-out timeline must not enter a hold mode solely because the next entry is additive and keys the same additive-capable property.
- R3. Existing `hold_previous`, attachment, sequence, draw order, and folder draw order behavior must not be treated as additive numeric blending.
- R4. Focused mixing semantics tests and the full `spine2d` `json,binary` suite must remain green.

---

## Key Technical Decisions

- **KTD1. Keep the metadata local to `TimelineKind`:** The Rust model does not expose a common `Timeline` trait, so `AnimationState` should derive additive and instant flags from `TimelineKind` rather than pushing new public fields through every timeline struct.
- **KTD2. Mirror upstream hold decisions, not just apply-time blending:** Additive blending already affects timeline application, but upstream also uses additive capability while deciding hold modes. The hold computation must account for the next entries in the chain.
- **KTD3. Preserve instant timeline semantics:** Attachment, sequence, inherit, draw order, draw-order folder, event, and physics reset timelines are not numeric additive overlays. This plan only changes hold decisions for additive-capable timeline kinds.

---

## Implementation Units

### U1. Add additive-hold regression coverage

- **Goal:** Prove that mixing from a base translate timeline into an additive translate timeline uses normal mix weighting without holding the outgoing timeline.
- **Files:** `spine2d/src/runtime/animation_state_mixing_semantics_tests.rs`.
- **Approach:** Build a two-animation fixture where `base` translates root to `10`, `overlay` translates root by `5`, and a 1-second mix is half complete. Set only the incoming overlay entry to additive and assert the final root x is `7.5`: `base` contributes `10 * 0.5`, then the additive overlay contributes `5 * 0.5`.
- **Verification:** The test fails before the hold-mode fix and passes after it.

### U2. Model upstream timeline additive and instant flags

- **Goal:** Give `compute_hold` the same decision inputs as upstream `Timeline.getAdditive()` and `Timeline.getInstant()`.
- **Files:** `spine2d/src/runtime/animation_state.rs`.
- **Approach:** Add internal helpers on `TimelineKind` that return whether a kind is additive-capable or instant. Treat bone timelines, deform, transform constraint, path position, physics constraint value timelines, and slider mix as additive-capable; treat attachment, sequence, draw order, draw-order folder, and inherit as instant.
- **Verification:** Focused mixing semantics tests pass without changing public model structs.

### U3. Align hold-mode computation with upstream

- **Goal:** Port the upstream `computeHold` additive/instant branch into the existing Rust mode enum.
- **Files:** `spine2d/src/runtime/animation_state.rs`.
- **Approach:** Use the new helpers when inspecting `to` and later `mixing_to` entries. If the next entry is additive for an additive-capable timeline, keep the outgoing timeline in its normal `First` or `Subsequent` mode rather than `HoldFirst` or `HoldMix`.
- **Verification:** Focused mixing semantics tests, full `spine2d` `json,binary` nextest, `cargo fmt --check`, and clippy for `spine2d`.

---

## Scope Boundaries

- This plan does not redesign timeline property IDs beyond the helper functions needed for hold decisions.
- This plan does not implement reverse event-order changes or broader event queue parity.
- This plan does not change the public `TrackEntry` additive API added by the prior plan.

---

## Sources / Research

- `.cache/spine-runtimes` at `spine-flutter-4.3.4`: `spine-cpp/src/spine/AnimationState.cpp` `computeHold` and `from`.
- `.cache/spine-runtimes` at `spine-flutter-4.3.4`: `spine-cpp/include/spine/Timeline.h` exposes `getAdditive()` and `getInstant()`.
- `.cache/spine-runtimes` at `spine-flutter-4.3.4`: `BoneTimeline.cpp`, `DeformTimeline.cpp`, `TransformConstraintTimeline.cpp`, `PathConstraintPositionTimeline.cpp`, `SliderMixTimeline.cpp`, and `PhysicsConstraintTimeline.h` mark additive timeline classes.
- `spine2d/src/runtime/animation_state.rs`: current `TimelineKind`, `timeline_property_ids`, and `compute_hold` implementation.

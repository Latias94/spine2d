---
title: "fix: Apply sequence timelines to inherited timeline slots"
type: "fix"
date: "2026-06-18"
execution: "code"
---

# fix: Apply sequence timelines to inherited timeline slots

## Summary

Align `SequenceTimeline` with latest Spine 4.3 behavior for linked meshes that inherit a parent mesh timeline across slots. The runtime must apply sequence index setup and keyed values to the timeline attachment's `timeline_slots`, not only to the primary timeline slot.

---

## Problem Frame

Latest upstream `SequenceTimeline.apply` uses the same `Attachment.timelineSlots` propagation shape as deform timelines. `spine2d` already parses and uses `MeshAttachmentData.timeline_slots` for deform, but `apply_sequence_timeline` still exits after updating `timeline.slot_index`, so a linked mesh in another slot can render the wrong sequence frame.

---

## Requirements

- R1. A sequence timeline whose target attachment is a mesh must apply to linked meshes whose `timeline_attachment` points to that target and whose slot is listed in the target mesh `timeline_slots`.
- R2. Setup/out and before-first sequence application must reset all matching timeline slots to `sequence_index = -1`, matching upstream setup-pose behavior.
- R3. Non-matching slots, inactive bones, and slots whose current attachment does not share the timeline attachment must remain unchanged.
- R4. Regression coverage must include a cross-slot linked mesh inheriting sequence timelines from a parent mesh.

---

## Key Technical Decisions

- **KTD1. Reuse the existing mesh `timeline_slots` model:** The parser already records upstream `Attachment.timelineSlots`; sequence runtime logic should read the same source as deform instead of duplicating linked-mesh discovery.
- **KTD2. Split sequence matching from sequence index calculation:** A small slot-level helper keeps primary and propagated slot behavior identical while preserving current mode/index math.
- **KTD3. Keep the change runtime-local:** JSON and binary parsers already populate `timeline_slots`; this plan only adds the missing consumer and focused runtime tests.

---

## Implementation Units

### U1. Add cross-slot sequence regression coverage

- **Goal:** Capture the current mismatch with a linked mesh in `slot1` inheriting a sequence timeline from a parent mesh in `slot0`.
- **Requirements:** R1, R2, R3, R4.
- **Files:** `spine2d/src/runtime/sequence_timeline_tests.rs`.
- **Approach:** Extend the existing sequence tests with JSON data that defines a parent mesh with a sequence, a cross-slot linked mesh, and a sequence timeline keyed on the parent mesh.
- **Test scenarios:** Applying the timeline at a keyed time updates both `slot0.sequence_index` and `slot1.sequence_index`; applying before the first key with setup/first semantics resets both matching slots; a non-matching linked slot remains unchanged.
- **Verification:** The new test fails before the runtime propagation change and passes after it.

### U2. Propagate `SequenceTimeline` over mesh `timeline_slots`

- **Goal:** Make `apply_sequence_timeline` apply setup and keyed sequence values to every matching slot in the timeline attachment's `timeline_slots`.
- **Requirements:** R1, R2, R3.
- **Files:** `spine2d/src/runtime/animation.rs`.
- **Approach:** Add sequence helpers parallel to the existing deform helpers: read `timeline_slots` from the timeline attachment, check whether each slot's current attachment shares the target timeline attachment, compute the sequence index once, and apply/reset each matching slot.
- **Patterns to follow:** `apply_deform`, `deform_timeline_slots`, and upstream `SequenceTimeline.apply` at `spine-flutter-4.3.4`.
- **Verification:** Focused sequence tests pass under `cargo nextest`.

---

## Scope Boundaries

- This plan does not change sequence parsing, sequence path formatting, or render draw-list selection.
- This plan does not re-record oracle goldens.
- This plan does not consolidate deform and sequence into a broader `TimelineSemantics` module; that remains a follow-up architecture direction.

---

## Risks & Dependencies

- The timeline slot list is currently populated by parser resolution. A parser regression can hide this runtime behavior, so the regression fixture asserts the parent mesh `timeline_slots` value before applying animation.
- Sequence timelines are instant timelines in upstream runtimes; setup/out handling must preserve current `MixBlend::Setup` and `MixBlend::First` semantics.

---

## Sources / Research

- `spine-ts/spine-core/src/Animation.ts` at `spine-flutter-4.3.4`: `SequenceTimeline.apply`, `setupPose`, and `applyToSlot` use `attachment.timelineSlots`.
- `spine2d/src/runtime/animation.rs`: existing deform timeline propagation over `MeshAttachmentData.timeline_slots`.
- `spine2d/src/json.rs` and `spine2d/src/binary.rs`: linked mesh resolution populates parent mesh `timeline_slots`.

---
title: "feat: Add TrackEntry additive playback"
type: "feat"
date: "2026-06-18"
execution: "code"
status: "superseded"
---

# feat: Add TrackEntry additive playback

> Superseded on 2026-06-23 by the local `repo-ref/spine-runtimes` C++ reference. Latest C++ exposes `TrackEntry::getMixBlend/setMixBlend(MixBlend)` rather than a public additive flag; commit `1a432d3` restored the public Rust API to `mix_blend` and removed `set_additive`/`with_additive`.

## Summary

Expose and apply latest Spine 4.3 `TrackEntry.additive` semantics. Additive playback is an independent track-entry flag in upstream runtimes; `spine2d` currently requires callers to approximate it by setting `MixBlend::Add`, which conflates two concepts and misses upstream API parity.

---

## Problem Frame

Official `AnimationState` computes `add = entry.additive` and passes that to timeline apply. Timelines that support additive then add their values to the current/setup pose while non-additive and instant timelines keep their normal behavior. The Rust runtime has `MixBlend::Add` plumbing, but no `TrackEntry.additive` field or handle setter, so callers cannot express the upstream flag directly.

---

## Requirements

- R1. `TrackEntry` stores an `additive` flag defaulting to `false`.
- R2. `TrackEntryHandle` exposes `set_additive(&mut AnimationState, bool)`.
- R3. Current-entry application treats `additive = true` as additive timeline application without requiring `mix_blend = MixBlend::Add`.
- R4. Mixing-from behavior uses the entry's additive flag when deciding the Add/Out path, matching upstream `from.additive`.
- R5. Existing `set_mix_blend(MixBlend::Add)` behavior remains functional for current tests during the transition.

---

## Key Technical Decisions

- **KTD1. Keep `MixBlend::Add` as a low-level blend mode:** Existing tests and callers use it directly, so this change adds the upstream flag without deleting the current escape hatch.
- **KTD2. Derive effective additive at application time:** Runtime branches should treat `entry.additive || blend == MixBlend::Add` as additive for compatibility, while the new API carries upstream semantics.
- **KTD3. Start with transform-visible coverage:** A simple translate overlay gives a deterministic pose assertion without depending on render assets.

---

## Implementation Units

### U1. Add additive regression coverage

- **Goal:** Prove a track entry with `additive = true` adds a translate timeline over the current pose even when `mix_blend` remains the default `Replace`.
- **Requirements:** R1, R2, R3.
- **Files:** `spine2d/src/runtime/animation_state_mixing_semantics_tests.rs`.
- **Approach:** Reuse the local single-bone test construction: base track translates root to `10`, overlay track translates by `3`, set only `overlay.set_additive(true)`, then assert final root x is `13`.
- **Verification:** The test fails before implementation and passes after it.

### U2. Implement TrackEntry additive

- **Goal:** Add the public handle setter and route apply/mixing through an effective additive decision.
- **Requirements:** R1, R2, R3, R4, R5.
- **Files:** `spine2d/src/runtime/animation_state.rs`.
- **Approach:** Add `additive: bool` to `TrackEntry`, initialize it to `false`, expose `TrackEntryHandle::set_additive`, and use `entry.additive || blend == MixBlend::Add` wherever current code branches on `MixBlend::Add` for additive semantics.
- **Patterns to follow:** Upstream `TrackEntry::setAdditive`, `AnimationState::apply`, `AnimationState::applyMixingFrom`, and existing Rust `set_shortest_rotation` / `set_mix_blend` handle APIs.
- **Verification:** Focused additive test, `animation_state_mixing_semantics` tests, full `json,binary` nextest, clippy, and upstream smoke pass.

---

## Scope Boundaries

- This plan does not delete `set_mix_blend(MixBlend::Add)` or migrate existing oracle scenarios.
- This plan does not redesign timeline property IDs or additive capability modeling.
- This plan does not expose per-timeline additive metadata publicly.

---

## Risks & Dependencies

- Additive and `shortest_rotation` interact in upstream (`shortestRotation = add || entry.shortestRotation`), so the implementation must update the rotation branch consistently.
- Existing oracle tests may rely on `MixBlend::Add`; compatibility is kept to avoid widening this change.

---

## Sources / Research

- `spine-cpp/include/spine/AnimationState.h` at `spine-flutter-4.3.4`: `TrackEntry` has `_additive` and `setAdditive`.
- `spine-cpp/src/spine/AnimationState.cpp` at `spine-flutter-4.3.4`: current and mixing-from paths use `bool add = entry._additive`.
- `spine-ts/spine-core/src/AnimationState.ts` at `spine-flutter-4.3.4`: same `TrackEntry.additive` behavior.

---
title: "fix: Parse official JSON sequence setup field"
type: "fix"
date: "2026-06-18"
execution: "code"
---

# fix: Parse official JSON sequence setup field

## Summary

Align JSON sequence setup-pose parsing with latest Spine 4.3 exports. Official runtimes read the sequence setup frame from `sequence.setup`; `spine2d` currently reads `sequence.setupIndex`, so exported JSON can render the wrong setup texture before sequence timelines run.

---

## Problem Frame

`Sequence::resolveIndex` falls back to `setupIndex` when a slot has `sequence_index = -1`. In latest upstream JSON readers, that value comes from the `setup` key. The Rust parser only maps `setupIndex`, which is not the official 4.3 field, and the existing tests accidentally encode the local field name.

---

## Requirements

- R1. JSON attachments with `sequence.setup` must populate `SequenceDef.setup_index`.
- R2. Existing no-sequence and omitted-setup behavior must remain upstream-compatible: no sequence means no path suffix, and omitted setup defaults to `0`.
- R3. Regression coverage must assert setup-pose render path selection from official JSON syntax.

---

## Key Technical Decisions

- **KTD1. Prefer the official key:** Parse `setup` as the source of truth for current 4.3 JSON exports.
- **KTD2. Keep the change parser-local:** Runtime sequence application and render path construction already consume `SequenceDef.setup_index`; only deserialization needs to move.
- **KTD3. Do not add a broad compatibility layer:** The project is allowed to break stale local conventions, so tests should use official `setup` syntax going forward.

---

## Implementation Units

### U1. Lock official sequence setup parsing

- **Goal:** Prove setup-pose texture selection follows `sequence.setup`.
- **Requirements:** R1, R2, R3.
- **Files:** `spine2d/src/runtime/sequence_timeline_tests.rs`.
- **Approach:** Update sequence fixtures to use `setup` and add a focused assertion that setup pose renders the configured frame before any animation time is applied.
- **Verification:** The test fails before parser changes and passes after them.

### U2. Parse `sequence.setup`

- **Goal:** Map the latest upstream JSON sequence key into `SequenceDef.setup_index`.
- **Requirements:** R1, R2.
- **Files:** `spine2d/src/json.rs`.
- **Approach:** Rename the serde field mapping from `setupIndex` to `setup`, keeping the default `0`.
- **Patterns to follow:** Upstream `SkeletonJson.readSequence` in `spine-cpp`, `spine-ts`, and `spine-libgdx`.
- **Verification:** Sequence tests, full `json,binary` nextest, and clippy pass.

---

## Scope Boundaries

- This plan does not change binary sequence parsing; binary already reads setup index positionally.
- This plan does not add stale `setupIndex` compatibility.
- This plan does not change sequence timeline mode math.

---

## Sources / Research

- `spine-cpp/src/spine/SkeletonJson.cpp` at `spine-flutter-4.3.4`: `readSequence` reads `Json::getInt(item, "setup", 0)`.
- `spine-ts/spine-core/src/SkeletonJson.ts` at `spine-flutter-4.3.4`: `readSequence` reads `getValue(map, "setup", 0)`.
- `spine-libgdx/spine-libgdx/src/com/esotericsoftware/spine/SkeletonJson.java` at `spine-flutter-4.3.4`: same JSON key.

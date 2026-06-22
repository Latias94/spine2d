---
type: "Current State"
title: "Current Engineering State"
description: "Short durable summary of the active engineering state."
tags: ["engineering-memory"]
timestamp: 2026-06-20T00:00:00Z
status: "active"
---

# Current State

- Goal: 对齐 `spine2d` 与官方 `spine-runtimes` latest 4.3 tag 的运行时行为。
- Branch: 当前工作区有大量既有未提交变更；不要回退用户或其他 agent 的改动。
- Baseline: `spine-ts-4.3.8` / commit `8e12b1250ab88c0f890849ea45aab80338cead63`；行为参考只看 `spine-cpp`。
- Last verified:
  - `cargo nextest run -p spine2d --features json,binary,upstream-smoke --no-fail-fast` passed with `546 passed, 10 skipped` on 2026-06-23.
  - `cargo nextest run -p spine2d-bevy --no-fail-fast` passed with `42 passed, 0 skipped` on 2026-06-23.
- Done:
  - Confirmed `4.3.2` is not the latest 4.3 tag; current explicit baseline is `spine-ts-4.3.8`.
  - Confirmed official 4.3.4 IK uses `ScaleYMode/scaleY`, not development HEAD `uniform`.
  - Confirmed `.json` and `.skel` run-to-walk scenarios are green after refreshing stale goldens.
  - Re-recorded stale `sack_*` physics goldens against the pinned official oracle.
  - Added upstream IK demo coverage for both JSON and `.skel`.
  - Locked exact draw batching parity with official renderers via unit tests for merge/split rules and the 16-bit index limit.
  - Deleted disabled Skeleton legacy solver code in commit `fbc85eb`; post-cleanup full parity gate remains green.
  - Centralized runtime timeline dispatch in commit `73edc54`; `AnimationState` now delegates concrete `TimelineKind` application to internal helpers in `animation.rs`.
  - Centralized binary parser timeline-order registration in commit `48518a5`; JSON already uses a local order reconstruction boundary.
  - Centralized track entry settings in commit `e1e827f`; Bevy now aliases the core runtime settings value object, and queued delay/mix-duration handling follows the `spine-cpp` two-argument `setMixDuration` rule.
  - Hid `TrackEntry` fields in commit `fc1c241`; external code now reads entry state through getters instead of broad public fields.
  - Preserved `spine-cpp` delay branch behavior in commit `f36cfa7`; negative delay is special-cased without coercing non-comparable delay values.
  - Started U6 Skeleton extraction in commit `3edaa0b`; path constraint scratch storage and capacity estimation now live in private `skeleton::path`.
  - Continued U6 Skeleton extraction in commit `0dab0fb`; path attachment lookup and path world-position helpers now live in private `skeleton::path`. Generic attachment world-vertex computation intentionally remains in `skeleton.rs` because it is still shared by `Skeleton::world_vertices`.
  - Continued U6 Skeleton extraction in commit `190a119`; update-cache ordering and debug formatting now live in private `skeleton::cache`, mirroring the official C++ `Skeleton::updateCache`/constraint `sort` responsibility boundary while preserving the Rust centralized constraint model.
  - Continued U6 Skeleton extraction in commit `757b2f7`; BonePose-equivalent world/local transform helpers and root/child world-transform math now live in private `skeleton::bone`, while the `Bone` type itself remains in `skeleton.rs` for now.
  - Continued U6 Skeleton extraction in commit `a37abac`; BonePose-equivalent `modifyWorld`, `modifyLocal`, child world-reset, and applied-transform decomposition now live in private `skeleton::bone`.
  - Continued U6 Skeleton extraction in commit `fc3ef3c`; the bone world-transform update entry now delegates to private `skeleton::bone`, completing the low-risk BonePose helper extraction slice.
  - Continued U6 Skeleton extraction in commit `e076419`; IK solver entry and helper routines now live in private `skeleton::ik`.
  - Continued U6 Skeleton extraction in commit `d772a9f`; transform constraint solver entry and helper routines now live in private `skeleton::transform`.
  - Continued U6 Skeleton extraction in commit `6be2f7b`; physics constraint solver entry and helper routines now live in private `skeleton::physics`.
  - Continued U6 Skeleton extraction in commit `6104586`; slider constraint solver entry and helper routines now live in private `skeleton::slider`.
  - Continued U6 Skeleton extraction in commit `5e93794`; path constraint apply entry now lives in private `skeleton::path`, and its path-only helper visibility was narrowed.
  - Continued U6 Skeleton extraction in commit `7f98a3d`; generic attachment world-vertices computation now lives in private `skeleton::vertices`, matching the official `VertexAttachment::computeWorldVertices` responsibility more closely.
  - Continued U6 Skeleton extraction in commit `b712f53`; the `Bone` runtime type now lives in private `skeleton::bone` and is re-exported from `skeleton` so the external type path stays stable.
  - Continued U6 Skeleton extraction in commit `6f56a26`; the `Slot` runtime type now lives in private `skeleton::slot` and is re-exported from `skeleton` so the external type path stays stable.
  - Continued U6 Skeleton extraction in commit `fcf3389`; IK, transform, path, physics, and slider runtime constraint types now live in their matching private modules and are re-exported from `skeleton`.
- In progress:
  - Autonomous spine-cpp parity hardening on local `main`, tracked by `docs/plans/2026-06-23-001-refactor-spine-cpp-parity-hardening-plan.md`.
- Blocked:
  - Not blocked.
- Next action:
  - Continue U6: audit public field/accessor hardening for `Skeleton`, `Bone`, `Slot`, and runtime constraint structs against the official C++ getter/setter shape.

# Citations

- `spine-upstream.toml`
- `docs/parity.md`
- `spine2d/src/runtime/skeleton.rs`
- `spine2d/src/runtime/animation.rs`
- `spine2d/src/runtime/animation_state.rs`
- `spine2d/src/json.rs`
- `spine2d/src/binary.rs`
- `spine2d/src/runtime/upstream_ik_demo_skel_tests.rs`
- `spine2d/tests/golden/oracle_scenarios_skel/spineboy_run_to_walk_mix0_2_t0_4.json`
- `docs/plans/2026-06-23-001-refactor-spine-cpp-parity-hardening-plan.md`

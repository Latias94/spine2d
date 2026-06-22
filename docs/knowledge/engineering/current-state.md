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
- Branch: local `main`; do not revert user or other agent changes if new unrelated edits appear.
- Baseline: `spine-ts-4.3.8` / commit `8e12b1250ab88c0f890849ea45aab80338cead63`；行为参考只看 `spine-cpp`。
- Last verified:
  - `cargo nextest run -p spine2d --features json,binary,upstream-smoke --no-fail-fast --status-level fail` passed with `567 passed, 10 skipped` on 2026-06-23.
  - `cargo nextest run -p spine2d-bevy --no-fail-fast --status-level fail` passed with `42 passed, 0 skipped` on 2026-06-23.
  - `cargo check -p spine2d --examples --features json,binary,upstream-smoke`, `cargo check -p spine2d-bevy`, `cargo check -p spine2d-bevy --examples`, `cargo check -p spine2d-wgpu`, `cargo check -p spine2d-wgpu --examples --features json`, and `cargo check -p spine2d-web` passed on 2026-06-23.
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
  - Continued U6 Skeleton extraction in commit `190a119`; update-cache ordering now lives in private `skeleton::cache`, mirroring the official C++ `Skeleton::updateCache`/constraint `sort` responsibility boundary while preserving the Rust centralized constraint model.
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
  - Continued U6 Skeleton API hardening in commit `047be09`; `Skeleton` container/state fields are now crate-visible with public accessor and setter methods aligned to the official C++ getter/setter shape.
  - Continued U6 Bone API hardening in commit `12218d2`; local pose, applied pose, active state, and world-transform fields are now crate-visible with public accessors/setters matching the official `BoneLocal`/`BonePose` shape.
  - Continued U6 Slot API hardening in commit `2643dd0`; slot pose fields are now crate-visible with public accessors/setters matching the official `SlotPose` shape, including attachment-change deform/sequence reset.
  - Continued U6 constraint API hardening in commit `0c8d8cd`; IK, transform, path, physics, and slider runtime pose fields are now crate-visible with public accessors/setters matching the official constraint pose/control shape. Physics integration internals are crate-visible only.
  - Added official-style physics movement controls in commit `ecdf83f`; `Skeleton::physics_translate`, `Skeleton::physics_rotate`, `PhysicsConstraint::translate`, and `PhysicsConstraint::rotate` now match the spine-cpp entry points and formulas.
  - Added official-style named lookup and attachment helpers in commit `9bae119`; `Skeleton` now exposes root/find bone/slot helpers, slot-name attachment lookup, and no-op-on-miss `set_attachment` semantics with source-skin-aware reset behavior.
  - Added no-clipper attachment bounds helper in commit `83df693`; `Skeleton::bounds` now computes official-style AABB over active region and mesh attachments using draw order.
  - Added official-style constraint lookup helpers in commit `fed0975`; `Skeleton` now exposes explicit by-name find/index/mut helpers for IK, transform, path, physics, and slider constraints.
  - Added official-style setup pose split APIs in commit `955cc27`; `Skeleton` now exposes `setup_pose`, `setup_pose_bones`, and `setup_pose_slots`, and empty setup attachments reset `sequenceIndex` like `spine-cpp`.
  - Removed the legacy `Skeleton::set_to_setup_pose` alias in breaking commit `c385349`; internal tests, examples, and backend callers now use `setup_pose` directly.
  - Added clipping-aware bounds in commit `43c5503`; `Skeleton::bounds_with_clipping` now matches the official optional `SkeletonClipping` bounds overload while `bounds()` remains the no-clipper default.
  - Aligned Skeleton world/skin controls in breaking commit `aec70e4`; wind/gravity/time/update now use direct C++ assignment semantics, component-level wind/gravity accessors exist, `set_skin(Some(missing))` no-ops, and `Error::UnknownSkin` was removed.
  - Removed the useless `Skeleton::set_skin` `Result` wrapper in breaking commit `ae1ab99`; callers now use the no-return setter directly.
  - Aligned the current skin accessor in breaking commit `ea3d166`; `Skeleton::skin()` now returns `Option<&SkinData>` like C++ `getSkin()`, while `skin_name()` exposes the stored skin name.
  - Added official-style update-cache inspection in commit `f0903f1`; `Skeleton::update_cache_items()` exposes a read-only typed view over the solver cache, matching C++ `getUpdateCache()` without exposing mutable internals.
  - Added official-style ordered constraint inspection in commit `295c836`; `Skeleton::constraints()` returns typed `ConstraintRef` entries sorted by global constraint order, covering the read-only Rust equivalent of C++ `getConstraints()`.
  - Added Bone/BonePose transform helper surface in commit `c20ab80`; `Bone::child_indices`, parent-space point transforms, local/world rotation transforms, and `rotate_world` now cover the remaining read-only/math helper shape from C++ `Bone`/`BonePose`.
  - Added BonePose update/local helper surface in commit `b2cadd4`; `Skeleton` now exposes single-bone world/local transform update, validation, and local/world modification markers without exposing C++'s raw update counter.
  - Fixed the public BonePose local-update wrapper in commit `e3e96c0`; `Skeleton::update_bone_local_transform` and validation now keep the bone world epoch current like C++ `BonePose::updateLocalTransform`.
  - Added `Bone::is_y_down/set_y_down` in commit `d374ddf`; `Skeleton::scale_y()` and world transforms now honor the C++-style global Y-down switch, while the default stays false to preserve the repo's Y-up oracle baseline.
  - Removed hidden `Skeleton::debug_update_cache` in commit `71ddc60`; debugging callers now format the typed `update_cache_items()` view locally instead of keeping a Rust-only public compatibility helper.
- In progress:
  - Autonomous spine-cpp parity hardening on local `main`, tracked by `docs/plans/2026-06-23-001-refactor-spine-cpp-parity-hardening-plan.md`.
- Blocked:
  - Not blocked.
- Next action:
  - Continue U6: audit the remaining `Skeleton.h` debug/internal public helpers and decide whether any more low-risk surface should be exposed or deleted.

# Citations

- `spine-upstream.toml`
- `docs/parity.md`
- `spine2d/src/runtime/skeleton.rs`
- `spine2d/src/runtime/skeleton_tests.rs`
- `spine2d/src/runtime/animation.rs`
- `spine2d/src/runtime/animation_state.rs`
- `spine2d/src/json.rs`
- `spine2d/src/binary.rs`
- `spine2d/src/runtime/upstream_ik_demo_skel_tests.rs`
- `spine2d/tests/golden/oracle_scenarios_skel/spineboy_run_to_walk_mix0_2_t0_4.json`
- `docs/plans/2026-06-23-001-refactor-spine-cpp-parity-hardening-plan.md`

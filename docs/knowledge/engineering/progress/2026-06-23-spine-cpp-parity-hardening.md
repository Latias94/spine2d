---
type: "Work Progress"
title: "Spine-cpp parity hardening"
description: "Progress record for autonomous spine-cpp parity refactoring."
tags: ["spine-cpp", "parity", "refactor"]
timestamp: 2026-06-23T00:00:00Z
status: "active"
related_plan: "docs/plans/2026-06-23-001-refactor-spine-cpp-parity-hardening-plan.md"
git_branch: "main"
verified_by: "cargo nextest run -p spine2d --features json,binary,upstream-smoke --no-fail-fast"
---

# Summary

Autonomous refactoring is active on local `main`. The behavior reference is `spine-cpp` from the pinned latest tag anchor `spine-ts-4.3.8` (`8e12b1250ab88c0f890849ea45aab80338cead63`).

# Verified State

- Full parity gate passed on 2026-06-23: `544 passed, 10 skipped`.
- U2 cleanup commit `fbc85eb` deleted 634 lines of disabled Skeleton legacy solver code.
- U3 dispatch cleanup commit `73edc54` moved `AnimationState` timeline application onto shared internal dispatch helpers in `animation.rs`.
- U4 parser cleanup commit `48518a5` moved binary animation timeline-order recording onto `TimelineOrderBuilder`; JSON keeps its existing local lookup/order builders.
- U5 settings cleanup commit `e1e827f` moved track entry settings application into the core runtime, replaced the Bevy settings implementation with an alias, and aligned queued delay/mix-duration adjustment with `spine-cpp`.
- U5 field cleanup commit `fc1c241` made `TrackEntry` state private and exposed read-only getters for external tests and Bevy.
- U5 delay cleanup commit `f36cfa7` preserved the `spine-cpp` delay branch shape for handle setters by special-casing negative delay without forcing non-comparable values through the queued-delay formula.
- U6 path scratch commit `3edaa0b` moved path constraint scratch storage and capacity estimation into private `skeleton::path`.
- U6 path world-position commit `0dab0fb` moved path attachment lookup, `compute_path_world_positions`, and private path curve helpers into private `skeleton::path`.
- U6 update-cache commit `190a119` moved cache ordering helpers and debug formatting into private `skeleton::cache`.
- U6 bone transform commit `757b2f7` moved BonePose-equivalent world/local transform helpers and root/child world-transform math into private `skeleton::bone`.
- U6 applied-transform commit `a37abac` moved BonePose-equivalent `modifyWorld`, `modifyLocal`, child world reset, and applied-transform decomposition into private `skeleton::bone`.
- U6 bone world-update commit `fc3ef3c` moved the bone world-transform update entry into private `skeleton::bone`.
- U6 IK commit `e076419` moved the IK solver entry and helper routines into private `skeleton::ik`.
- U6 transform commit `d772a9f` moved the transform constraint solver entry and helper routines into private `skeleton::transform`.
- Post-U2 verification passed:
  - `cargo fmt --all --check`
  - `git diff --check`
  - `cargo nextest run -p spine2d --features json,binary,upstream-smoke --no-fail-fast` (`544 passed, 10 skipped`)
- Post-U3 verification passed:
  - `cargo check -p spine2d --features json,binary,upstream-smoke`
  - `cargo nextest run -p spine2d --features json,binary,upstream-smoke animation_state animation --no-fail-fast` (`76 passed, 478 skipped`)
  - `cargo nextest run -p spine2d --features json,binary,upstream-smoke --no-fail-fast` (`544 passed, 10 skipped`)
- Post-U4 verification passed:
  - `cargo check -p spine2d --features json,binary,upstream-smoke`
  - `cargo nextest run -p spine2d --features json,binary,upstream-smoke timeline_order --no-fail-fast` (`5 passed, 549 skipped`)
  - `cargo nextest run -p spine2d --features json,binary,upstream-smoke --no-fail-fast` (`544 passed, 10 skipped`)
- Post-U5 settings-slice verification passed:
  - `cargo check -p spine2d --features json,binary,upstream-smoke`
  - `cargo nextest run -p spine2d --features json,binary,upstream-smoke animation_state animation --no-fail-fast` (`78 passed, 478 skipped`)
  - `cargo nextest run -p spine2d --features json,binary,upstream-smoke --no-fail-fast` (`546 passed, 10 skipped`)
  - `cargo nextest run -p spine2d-bevy --no-fail-fast` (`42 passed, 0 skipped`)
  - `cargo check -p spine2d-bevy`
- Post-U5 field/delay-slice verification passed:
  - `cargo check -p spine2d --features json,binary,upstream-smoke`
  - `cargo nextest run -p spine2d --features json,binary,upstream-smoke animation_state animation --no-fail-fast` (`78 passed, 478 skipped`)
  - `cargo nextest run -p spine2d --features json,binary,upstream-smoke --no-fail-fast` (`546 passed, 10 skipped`)
  - `cargo nextest run -p spine2d-bevy --no-fail-fast` (`42 passed, 0 skipped`)
  - `cargo check -p spine2d-bevy`
- Post-U6 path-scratch verification passed:
  - `cargo check -p spine2d --features json,binary,upstream-smoke`
  - `cargo nextest run -p spine2d --features json,binary,upstream-smoke skeleton path_constraint transform_constraint ik physics slider --no-fail-fast` (`112 passed, 444 skipped`)
  - `cargo nextest run -p spine2d --features json,binary,upstream-smoke --no-fail-fast` (`546 passed, 10 skipped`)
- Post-U6 path-world-helper verification passed:
  - `cargo fmt --all --check`
  - `git diff --check`
  - `cargo check -p spine2d --features json,binary,upstream-smoke`
  - `cargo nextest run -p spine2d --features json,binary,upstream-smoke skeleton path_constraint transform_constraint ik physics slider --no-fail-fast` (`112 passed, 444 skipped`)
  - `cargo nextest run -p spine2d --features json,binary,upstream-smoke --no-fail-fast` (`546 passed, 10 skipped`)
- Post-U6 update-cache verification passed:
  - `cargo fmt --all --check`
  - `git diff --check`
  - `cargo check -p spine2d --features json,binary,upstream-smoke`
  - `cargo nextest run -p spine2d --features json,binary,upstream-smoke skeleton skin_active path_constraint transform_constraint ik physics slider --no-fail-fast` (`117 passed, 439 skipped`)
  - `cargo nextest run -p spine2d --features json,binary,upstream-smoke --no-fail-fast` (`546 passed, 10 skipped`)
- Post-U6 bone-transform verification passed:
  - `cargo fmt --all --check`
  - `git diff --check`
  - `cargo check -p spine2d --features json,binary,upstream-smoke`
  - `cargo nextest run -p spine2d --features json,binary,upstream-smoke skeleton upstream_ik_demo ik path_constraint transform_constraint physics slider --no-fail-fast` (`112 passed, 444 skipped`)
  - `cargo nextest run -p spine2d --features json,binary,upstream-smoke --no-fail-fast` (`546 passed, 10 skipped`)
- Post-U6 applied-transform verification passed:
  - `cargo fmt --all --check`
  - `git diff --check`
  - `cargo check -p spine2d --features json,binary,upstream-smoke`
  - `cargo nextest run -p spine2d --features json,binary,upstream-smoke skeleton upstream_ik_demo ik path_constraint transform_constraint physics slider --no-fail-fast` (`112 passed, 444 skipped`)
  - `cargo nextest run -p spine2d --features json,binary,upstream-smoke --no-fail-fast` (`546 passed, 10 skipped`)
- Post-U6 bone-world-update verification passed:
  - `cargo fmt --all --check`
  - `git diff --check`
  - `cargo check -p spine2d --features json,binary,upstream-smoke`
  - `cargo nextest run -p spine2d --features json,binary,upstream-smoke skeleton upstream_ik_demo ik path_constraint transform_constraint physics slider --no-fail-fast` (`112 passed, 444 skipped`)
  - `cargo nextest run -p spine2d --features json,binary,upstream-smoke --no-fail-fast` (`546 passed, 10 skipped`)
- Post-U6 IK verification passed:
  - `cargo fmt --all --check`
  - `git diff --check`
  - `cargo check -p spine2d --features json,binary,upstream-smoke`
  - `cargo nextest run -p spine2d --features json,binary,upstream-smoke ik upstream_ik_demo skeleton transform_constraint path_constraint physics slider --no-fail-fast` (`112 passed, 444 skipped`)
  - `cargo nextest run -p spine2d --features json,binary,upstream-smoke --no-fail-fast` (`546 passed, 10 skipped`)
- Post-U6 transform verification passed:
  - `cargo fmt --all --check`
  - `git diff --check`
  - `cargo check -p spine2d --features json,binary,upstream-smoke`
  - `cargo nextest run -p spine2d --features json,binary,upstream-smoke transform_constraint skeleton ik path_constraint physics slider --no-fail-fast --status-level fail` (`112 passed, 444 skipped`)
  - `cargo nextest run -p spine2d --features json,binary,upstream-smoke --no-fail-fast --status-level fail` (`546 passed, 10 skipped`)
- The worktree was clean before creating the hardening plan and memory updates.
- Existing golden metadata is intentionally not rewritten unless assets or oracle outputs are regenerated.

# In Progress

- U1 is complete: the hardening plan and engineering memory baseline are recorded.
- U2 is complete: disabled `#[cfg(any())]` Skeleton legacy code has been removed.
- U3 is complete: timeline dispatch is centralized behind internal runtime/state helpers, while `AnimationState` keeps only policy decisions for alpha, hold, additive, thresholds, and draw-order output.
- U4 is complete: binary parser timeline-order ownership is centralized behind `TimelineOrderBuilder`, and JSON already had explicit local lookup/order builders.
- U5 is complete: the shared `TrackEntrySettings` value object is now owned by the core runtime and used by Bevy, direct `TrackEntry` field exposure has been removed, and delay setter branches now follow the official C++ shape. The final numeric setter audit found no additional guard changes needed because `spine-cpp` setters are intentionally sparse.
- U6 is in progress: path constraint scratch storage, capacity estimation, path attachment lookup, path world-position calculation, and private path curve helpers have moved into `skeleton::path`; update-cache ordering and debug formatting have moved into `skeleton::cache`; BonePose-equivalent world/local transform helpers, root/child world-transform math, `modifyWorld`, `modifyLocal`, child reset, applied-transform decomposition, and the bone world-update entry have moved into `skeleton::bone`; IK and transform constraint solver helpers have moved into `skeleton::ik` and `skeleton::transform`. The generic `compute_attachment_world_vertices` helper intentionally remains in `skeleton.rs` because it is still shared by path solving and `Skeleton::world_vertices`.

# Next Action

Audit the remaining `Skeleton` constraint solver bodies and choose the next low-risk extraction, likely physics or slider helpers before broader public API movement. Keep the same verification shape: focused solver tests first, then the full core parity gate.

# Citations

- [Hardening plan](../../../plans/2026-06-23-001-refactor-spine-cpp-parity-hardening-plan.md)
- [Current state](../current-state.md)
- [Update log](../log.md)

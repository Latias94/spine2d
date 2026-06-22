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
- In progress:
  - Autonomous spine-cpp parity hardening on local `main`, tracked by `docs/plans/2026-06-23-001-refactor-spine-cpp-parity-hardening-plan.md`.
- Blocked:
  - Not blocked.
- Next action:
  - Execute U6 from the hardening plan: extract `Skeleton` pose-solver boundaries incrementally while keeping solver parity gates green.

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

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
- Baseline: `spine-flutter-4.3.4` / commit `80dc680a4345ac09cdc5d4c1a77ec572a3f295d1`。
- Last verified:
  - `cargo fmt --all --check` passed.
  - `cargo nextest run -p spine2d --features json,binary,upstream-smoke --no-fail-fast` passed with `533 passed, 10 skipped`.
- Done:
  - Confirmed `4.3.2` is not the latest 4.3 tag; current explicit baseline is `spine-flutter-4.3.4`.
  - Confirmed official 4.3.4 IK uses `ScaleYMode/scaleY`, not development HEAD `uniform`.
  - Confirmed `.json` and `.skel` run-to-walk scenarios are green after refreshing stale goldens.
  - Re-recorded stale `sack_*` physics goldens against the pinned official oracle.
  - Added upstream IK demo coverage for both JSON and `.skel`.
  - Locked exact draw batching parity with official renderers via unit tests for merge/split rules and the 16-bit index limit.
- In progress:
  - Preparing reviewable git commits for the completed parity work.
- Blocked:
  - Not blocked.
- Next action:
  - Move to the next yellow axis from `docs/parity.md` once there is a concrete behavior gap worth isolating.

# Citations

- `spine-upstream.toml`
- `docs/parity.md`
- `spine2d/src/runtime/skeleton.rs`
- `spine2d/src/binary.rs`
- `spine2d/src/runtime/upstream_ik_demo_skel_tests.rs`
- `spine2d/tests/golden/oracle_scenarios_skel/spineboy_run_to_walk_mix0_2_t0_4.json`

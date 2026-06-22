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
- Post-U2 verification passed:
  - `cargo fmt --all --check`
  - `git diff --check`
  - `cargo nextest run -p spine2d --features json,binary,upstream-smoke --no-fail-fast` (`544 passed, 10 skipped`)
- The worktree was clean before creating the hardening plan and memory updates.
- Existing golden metadata is intentionally not rewritten unless assets or oracle outputs are regenerated.

# In Progress

- U1 is complete: the hardening plan and engineering memory baseline are recorded.
- U2 is complete: disabled `#[cfg(any())]` Skeleton legacy code has been removed.
- U3 is next: compare timeline dispatch against `spine-cpp` `Animation.cpp` / `AnimationState.cpp` and collapse duplicated runtime dispatch only after characterization is clear.

# Next Action

Audit timeline application paths in `spine2d/src/runtime/animation.rs` and `spine2d/src/runtime/animation_state.rs` against `spine-cpp`, then decide whether an internal dispatch adapter or fixture-first characterization slice is the safest next change.

# Citations

- [Hardening plan](../../../plans/2026-06-23-001-refactor-spine-cpp-parity-hardening-plan.md)
- [Current state](../current-state.md)
- [Update log](../log.md)

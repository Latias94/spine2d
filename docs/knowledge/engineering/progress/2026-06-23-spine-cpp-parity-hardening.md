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
- Post-U2 verification passed:
  - `cargo fmt --all --check`
  - `git diff --check`
  - `cargo nextest run -p spine2d --features json,binary,upstream-smoke --no-fail-fast` (`544 passed, 10 skipped`)
- Post-U3 verification passed:
  - `cargo check -p spine2d --features json,binary,upstream-smoke`
  - `cargo nextest run -p spine2d --features json,binary,upstream-smoke animation_state animation --no-fail-fast` (`76 passed, 478 skipped`)
  - `cargo nextest run -p spine2d --features json,binary,upstream-smoke --no-fail-fast` (`544 passed, 10 skipped`)
- The worktree was clean before creating the hardening plan and memory updates.
- Existing golden metadata is intentionally not rewritten unless assets or oracle outputs are regenerated.

# In Progress

- U1 is complete: the hardening plan and engineering memory baseline are recorded.
- U2 is complete: disabled `#[cfg(any())]` Skeleton legacy code has been removed.
- U3 is complete: timeline dispatch is centralized behind internal runtime/state helpers, while `AnimationState` keeps only policy decisions for alpha, hold, additive, thresholds, and draw-order output.
- U4 is next: tighten JSON/binary parser timeline-order ownership with an explicit builder or equivalent local boundary.

# Next Action

Audit `spine2d/src/json.rs`, `spine2d/src/binary.rs`, and `spine2d/src/model.rs` timeline-order push logic against `spine-cpp` loader order. If current coverage is sufficient, introduce a parser-side timeline order builder; otherwise add a focused parser characterization first.

# Citations

- [Hardening plan](../../../plans/2026-06-23-001-refactor-spine-cpp-parity-hardening-plan.md)
- [Current state](../current-state.md)
- [Update log](../log.md)

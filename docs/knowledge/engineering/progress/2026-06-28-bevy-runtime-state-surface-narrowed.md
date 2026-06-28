---
type: "Work Progress"
title: "Bevy runtime state surface narrowed"
description: "Work Progress for Bevy Spine runtime snapshot surface narrowing."
timestamp: 2026-06-28T11:33:46Z
tags: ["spine-cpp", "parity", "bevy", "refactor"]
source_session: "manual"
---

# Summary

`SpineRuntimeState` and `SpineTrackState` no longer expose broad public field bags. They now use explicit constructors plus read-only getter methods, and the Bevy systems, examples, and tests read the runtime snapshot through that narrower surface.

# Details

- `SpineRuntimeState` now stores readiness, track snapshots, skeleton time, physics, wind, gravity, and bounds privately.
- `SpineTrackState` now stores per-track snapshot data privately and exposes getter methods for UI/debug consumers.
- `spine2d-bevy/src/systems.rs` now builds runtime snapshots through `SpineRuntimeState::new(...)` and `SpineTrackState::new(...)`.
- `mixing.rs`, `mixing_inspector.rs`, and `runtime_controls.rs` now render runtime state through getters instead of direct field access.
- Bevy runtime snapshot tests now assert through the getter surface.

# Verification

- `cargo fmt --all -- --check`
- `cargo nextest run -p spine2d-bevy --no-fail-fast --status-level fail` (`43 passed, 0 skipped`)
- `cargo test -p spine2d-bevy --example viewer --no-run`
- `cargo test -p spine2d-bevy --example basic --no-run`
- `git diff --check`

# Next Action

继续收紧 Bevy 内部剩余的字段壳，优先检查 `spine2d-bevy/src/spine_world.rs` 的 `SpineInstance` / `SpineInstanceParts` 是否还能用 getter/setter 或构造辅助替代直接字段读写。

# Citations

- `spine2d-bevy/src/components.rs`
- `spine2d-bevy/src/systems.rs`
- `spine2d-bevy/examples/mixing.rs`
- `spine2d-bevy/examples/mixing_inspector.rs`
- `spine2d-bevy/examples/runtime_controls.rs`
- `cargo fmt --all -- --check`
- `cargo nextest run -p spine2d-bevy --no-fail-fast --status-level fail`
- `cargo test -p spine2d-bevy --example viewer --no-run`
- `cargo test -p spine2d-bevy --example basic --no-run`
- `git diff --check`

---
type: "Work Progress"
title: "Bevy spine instance surface narrowed"
description: "Work Progress for Bevy SpineInstance and SpineInstanceParts surface narrowing."
timestamp: 2026-06-28T11:47:58Z
tags: ["spine-cpp", "parity", "bevy", "refactor"]
source_session: "manual"
---

# Summary

`SpineInstance` and `SpineInstanceParts` no longer expose their runtime state as a broad internal field bag. The Bevy world/runtime layer now builds instances through explicit constructors and builder helpers, reads mutable state through getter/setter methods, and rebuilds pose/draw data through a dedicated instance method instead of open-coded field mutation.

# Details

- `SpineInstanceParts` now uses `new(...)` plus `with_*` builder helpers for animation name, loop flag, time scale, skin name, flip Y, and skeleton control.
- `SpineInstance` now keeps its skeleton, animation state, draw list, atlas metadata, animation selection, playback settings, skin state, flip flag, and skeleton control private behind explicit accessors.
- `SpineInstance::rebuild_pose(delta)` now owns the update/apply/world-transform/draw-list rebuild sequence that had previously been open-coded in the systems layer.
- `spine2d-bevy/src/systems.rs` now constructs instances through the builder surface and updates runtime state through getter/setter methods instead of direct field access.
- `spine2d-bevy/src/systems/render.rs` now reads draw extraction inputs through accessors instead of touching instance fields directly.
- Bevy systems tests and helper constructors were updated to read `SpineInstance` state through getters and to build fixture instances through `SpineInstanceParts::new(...)`.

# Verification

- `rg -n "instance\.(animation_state|skeleton|draw_list|atlas|atlas_directory|animation_name|loop_animation|time_scale|skin_name|flip_y|skeleton_control)" spine2d-bevy/src/systems.rs spine2d-bevy/src/systems/render.rs spine2d-bevy/src/spine_world.rs`
- `cargo fmt --all -- --check`
- `cargo nextest run -p spine2d-bevy --no-fail-fast --status-level fail` (`43 passed, 0 skipped`)
- `cargo test -p spine2d-bevy --example viewer --no-run`
- `cargo test -p spine2d-bevy --example basic --no-run`
- `git diff --check`

# Next Action

继续检查 `spine2d-bevy/src/components.rs` 中剩余的公共值对象/命令壳类型，区分哪些应保留为简单数据面，哪些仍值得继续收口以贴近 latest-tag `spine-cpp` 的公开形状。

# Citations

- `spine2d-bevy/src/spine_world.rs`
- `spine2d-bevy/src/systems.rs`
- `spine2d-bevy/src/systems/render.rs`
- `cargo fmt --all -- --check`
- `cargo nextest run -p spine2d-bevy --no-fail-fast --status-level fail`
- `cargo test -p spine2d-bevy --example viewer --no-run`
- `cargo test -p spine2d-bevy --example basic --no-run`
- `git diff --check`

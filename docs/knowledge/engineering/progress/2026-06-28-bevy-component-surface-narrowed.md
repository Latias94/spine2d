---
type: "Work Progress"
title: "Bevy component surface narrowed"
description: "Work Progress for Bevy Spine component surface narrowing."
timestamp: 2026-06-28T10:44:08Z
tags: ["spine-cpp", "parity", "bevy", "refactor"]
source_session: "manual"
---

# Summary

`Spine`, `SpineAnimation`, and `SpineSkin` were narrowed to private fields with explicit getter/setter helpers. The Bevy systems and viewer example now read and write through those helpers, and the component-facing tests were updated to construct the components through small local helpers instead of struct literals.

# Details

- `Spine` now exposes getter/setter pairs for skeleton, atlas, animation, loop flag, time scale, and skin name.
- `SpineAnimation` now exposes getters/setters plus small `with_*` helpers for tests and local construction.
- `SpineSkin` now exposes getters/setters plus a small `with_name` helper.
- `spine2d-bevy/src/systems.rs` now constructs component test fixtures through helpers instead of public field literals.
- `spine2d-bevy/examples/viewer.rs` now syncs the selected example through component setters and getter-based comparisons.

# Next Action

继续收紧 Bevy 侧剩余的公开字段面，优先看 `spine2d-bevy/src/spine_world.rs` 和其它示例里是否还有同类组件字段直读。

# Citations

- `spine2d-bevy/src/components.rs`
- `spine2d-bevy/src/systems.rs`
- `spine2d-bevy/examples/viewer.rs`
- `cargo fmt --all -- --check`
- `cargo nextest run -p spine2d-bevy --no-fail-fast --status-level fail`
- `cargo test -p spine2d-bevy --example viewer --no-run`
- `cargo test -p spine2d-bevy --example basic --no-run`

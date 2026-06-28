---
type: "Work Progress"
title: "Bevy asset wrapper surface narrowed"
description: "Work Progress for Bevy Spine asset wrapper surface narrowing."
timestamp: 2026-06-28T11:12:42Z
tags: ["spine-cpp", "parity", "bevy", "refactor"]
source_session: "manual"
---

# Summary

`SpineSkeletonAsset` and `SpineAtlasAsset` were narrowed from public field bags to private storage with explicit constructors and getter methods. The Bevy runtime, viewer example, and test fixtures now consume those accessors, and the wrappers no longer expose their internals directly.

# Details

- `SpineSkeletonAsset` now stores `Arc<SkeletonData>` privately and exposes `new(...)` plus `get_data()`.
- `SpineAtlasAsset` now stores `Atlas` and its directory privately and exposes `new(...)`, `get_atlas()`, and `get_directory()`.
- The loader code now returns the private constructors directly, without public struct literals.
- Bevy systems and viewer code now read skeleton and atlas handles through the new accessors.

# Verification

- `cargo fmt --all -- --check`
- `cargo nextest run -p spine2d-bevy --no-fail-fast --status-level fail` (`43 passed, 0 skipped`)
- `cargo test -p spine2d-bevy --example viewer --no-run`
- `cargo test -p spine2d-bevy --example basic --no-run`
- `git diff --check`

# Next Action

继续收紧 Bevy 侧剩余的公开字段面，优先检查 `spine2d-bevy/src/spine_world.rs` 和其它示例是否还有同类组件字段直读。

# Citations

- `spine2d-bevy/src/asset_loader.rs`
- `spine2d-bevy/src/systems.rs`
- `spine2d-bevy/examples/viewer.rs`
- `cargo fmt --all -- --check`
- `cargo nextest run -p spine2d-bevy --no-fail-fast --status-level fail`
- `cargo test -p spine2d-bevy --example viewer --no-run`
- `cargo test -p spine2d-bevy --example basic --no-run`
- `git diff --check`

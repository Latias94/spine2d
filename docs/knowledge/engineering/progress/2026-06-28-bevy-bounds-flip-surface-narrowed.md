---
type: "Work Progress"
title: "Bevy bounds and flip surface narrowed"
description: "Work Progress for Bevy SpineBounds and SpineFlipY surface narrowing."
timestamp: 2026-06-28T11:52:59Z
tags: ["spine-cpp", "parity", "bevy", "refactor"]
source_session: "manual"
---

# Summary

`SpineBounds` and `SpineFlipY` no longer expose their state through public fields. The Bevy runtime now reads them through small getters, which trims another piece of runtime-facing state surface without changing how bounds visualization or flip-Y behavior works.

# Details

- `SpineFlipY` is no longer a public tuple-field wrapper; callers now use `SpineFlipY::new(...)`, `SpineFlipY::flipped()`, and `get_flip_y()`.
- `SpineBounds` now keeps `min` and `max` private and exposes `get_min()`, `get_max()`, `center()`, and `size()`.
- `spine2d-bevy/src/systems.rs` now reads the optional flip-Y component through `SpineFlipY::get_flip_y()` and uses `SpineFlipY::flipped()` in tests instead of tuple construction.
- Existing Bevy examples already consumed bounds through `center()` / `size()`, so no example-side API churn was needed for `SpineBounds`.
- Event and command message payload structs in `components.rs` were intentionally left field-public for now because they act as Bevy message data carriers rather than long-lived runtime state holders.

# Verification

- `rg -n "SpineFlipY\\(|flip_y\\.0|bounds\\.(min|max)" spine2d-bevy/src spine2d-bevy/examples`
- `cargo fmt --all -- --check`
- `cargo nextest run -p spine2d-bevy --no-fail-fast --status-level fail` (`43 passed, 0 skipped`)
- `cargo test -p spine2d-bevy --example viewer --no-run`
- `cargo test -p spine2d-bevy --example basic --no-run`
- `git diff --check`

# Next Action

继续筛分 `spine2d-bevy/src/components.rs` 里剩余的公开类型，优先找“长期状态值对象”与“消息载荷”之间的边界，只收口前者，避免为 Bevy 事件/命令调用面引入无谓摩擦。

# Citations

- `spine2d-bevy/src/components.rs`
- `spine2d-bevy/src/systems.rs`
- `spine2d-bevy/examples/bounds.rs`
- `spine2d-bevy/examples/viewer.rs`
- `cargo fmt --all -- --check`
- `cargo nextest run -p spine2d-bevy --no-fail-fast --status-level fail`
- `cargo test -p spine2d-bevy --example viewer --no-run`
- `cargo test -p spine2d-bevy --example basic --no-run`
- `git diff --check`

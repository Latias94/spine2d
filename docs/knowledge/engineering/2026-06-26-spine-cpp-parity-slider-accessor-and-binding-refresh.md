---
type: "Progress"
title: "spine cpp parity slider accessor and binding refresh"
description: "Progress for spine cpp parity slider accessor and binding refresh."
timestamp: 2026-06-26T10:48:00Z
tags: ["spine-rs", "parity", "slider", "progress"]
source_session: "current"
---

# Summary

The slider accessor and binding slice is now consistent end-to-end: `SliderConstraintData::get_animation` reads through `SkeletonData`, loaders populate both the index and the animation name, `set_animation(&Animation)` clears the stale index for manual use, and runtime slider initialization/apply share the same getter path.

# Details

- The manual slider setter now prefers the name path and clears stale index state.
- JSON and binary loaders now write both animation name and index so the public getter and the runtime cache agree.
- The regression test now covers name-only binding plus draw-order output.

# Next Action

Move the parity audit to another high-value C++ surface unless a new slider oracle delta appears.

# Citations

- `spine2d/src/model.rs`
- `spine2d/src/json.rs`
- `spine2d/src/binary.rs`
- `spine2d/src/runtime/skeleton.rs`
- `spine2d/src/runtime/skeleton/slider.rs`
- `spine2d/src/runtime/slider_timeline_tests.rs`
- `cargo fmt --all -- --check`
- `cargo check -p spine2d --features json,binary,upstream-smoke`
- `CARGO_TARGET_DIR=/tmp/spine-rs-target-test cargo nextest run -p spine2d --features json,binary,upstream-smoke --no-fail-fast --status-level fail`

---
type: "Work Progress"
title: "Bevy loop wrapper names aligned"
description: "Work Progress for aligning Bevy animation loop wrapper terminology with core TrackEntry and spine-cpp."
timestamp: 2026-06-28T14:13:33Z
tags: ["spine-cpp", "parity", "bevy", "refactor"]
source_session: "manual"
---

# Summary

Renamed the remaining Bevy animation loop wrapper surface from `loop_animation` terminology to `looped` storage plus `get_loop` / `set_loop` / `with_loop` methods, matching core `TrackEntry` and latest-tag C++ `TrackEntry::getLoop` / `setLoop`.

# Details

- `Spine`, `SpineAnimation`, `SpineTrackState`, and `SpineTrackStateParts` now store the boolean as `looped`.
- Public Bevy component/snapshot accessors now use `get_loop` / `set_loop` / `with_loop` instead of `get_loop_animation` / `set_loop_animation` / `with_loop_animation`.
- `SpineInstance` and `SpineInstanceParts` now use the same `looped` naming internally, so systems no longer translate between Bevy-only and core loop terms.
- Bevy systems, tests, and the viewer example were migrated to the new names.

# Verification

Passed:

- `cargo check -p spine2d-bevy --examples`
- `cargo nextest run -p spine2d-bevy --no-fail-fast --status-level fail` (`43 passed, 0 skipped`)
- `cargo clippy -p spine2d-bevy --tests --examples -- -D warnings`

# Next Action

Continue scanning Bevy command/config wrappers for names that duplicate core runtime concepts with different terminology. Keep Bevy-specific names where they describe ECS lifecycle or scheduling rather than Spine runtime API.

# Citations

- `spine2d-bevy/src/components.rs`
- `spine2d-bevy/src/spine_world.rs`
- `spine2d-bevy/src/systems.rs`
- `spine2d-bevy/examples/viewer.rs`
- `repo-ref/spine-runtimes/spine-cpp/include/spine/AnimationState.h`

---
type: "Work Progress"
title: "Bevy TrackEntry loop builder renamed"
description: "Work Progress for aligning the Bevy track-entry loop settings builder with core and spine-cpp naming."
timestamp: 2026-06-28T14:01:12Z
tags: ["spine-cpp", "parity", "bevy", "refactor"]
source_session: "manual"
---

# Summary

Renamed the Bevy-only `SpineTrackEntrySettings::with_looped(...)` builder to `with_loop(...)` so the command batching surface follows core `TrackEntryHandle::set_loop` and latest-tag C++ `TrackEntry::setLoop`.

# Details

- `spine2d-bevy/src/components.rs` now exposes `with_loop(bool)` on `SpineTrackEntrySettings`.
- The Bevy command settings regression now uses `with_loop(false)` and still verifies the applied current-entry `get_loop()` value.
- Historical plan and memory references were updated so future audit passes do not preserve the old Bevy-only spelling as if it were intentional parity surface.

# Verification

Passed:

- `rg -n "with_looped" spine2d-bevy docs spine2d spine2d-wgpu spine2d-web` (no matches)
- `cargo fmt --all -- --check`
- `cargo nextest run -p spine2d-bevy set_animation_command_applies_track_entry_settings_to_current_entry --no-fail-fast --status-level fail` (`1 passed, 42 skipped`)
- `cargo clippy -p spine2d-bevy --tests --examples -- -D warnings`

# Next Action

Continue Bevy-only API cleanup by looking for duplicated wrapper names that drift from core or latest-tag `spine-cpp` terminology, but keep ECS command/config value objects when they solve Bevy scheduling or batching rather than pretending they are core runtime API.

# Citations

- `spine2d-bevy/src/components.rs`
- `spine2d-bevy/src/systems.rs`
- `repo-ref/spine-runtimes/spine-cpp/include/spine/AnimationState.h`

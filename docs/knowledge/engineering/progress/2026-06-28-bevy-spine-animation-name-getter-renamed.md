---
type: "Work Progress"
title: "Bevy Spine animation-name getter renamed"
description: "Work Progress for tightening the Bevy Spine component's initial-animation getter name."
timestamp: 2026-06-28T14:19:39Z
tags: ["spine-cpp", "parity", "bevy", "refactor"]
source_session: "manual"
---

# Summary

Renamed `Spine::get_animation()` to `Spine::get_animation_name()` because the Bevy root component stores an optional initial animation name, not a resolved core `Animation` object.

# Details

- `Spine::get_animation_name()` now matches `Spine::set_animation_name(...)` and the internal `SpineInstance::get_animation_name()` surface.
- The instance creation system now reads the initial animation through the explicit name getter.
- Core `TrackEntry::get_animation()` call sites were left unchanged because they return real animation objects.

# Verification

Passed:

- `cargo fmt --all -- --check`
- `cargo check -p spine2d-bevy --examples`
- `cargo nextest run -p spine2d-bevy --no-fail-fast --status-level fail` (`43 passed, 0 skipped`)
- `cargo clippy -p spine2d-bevy --tests --examples -- -D warnings`

# Next Action

Continue scanning command/config wrappers where names may be ambiguous between ECS convenience and core Spine runtime terminology.

# Citations

- `spine2d-bevy/src/components.rs`
- `spine2d-bevy/src/systems.rs`

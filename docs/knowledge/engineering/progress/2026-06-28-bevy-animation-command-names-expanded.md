---
type: "Work Progress"
title: "Bevy animation command names expanded"
description: "Work Progress for aligning Bevy animation command constructors and variants with core AnimationState method names."
timestamp: 2026-06-28T15:49:14Z
tags: ["spine-cpp", "parity", "bevy", "refactor"]
source_session: "manual"
---

# Summary

Renamed the short Bevy animation command constructors and variants to match core `AnimationState` method names more directly.

# Details

- `SpineAnimationCommand::set(...)` / `add(...)` are now `set_animation(...)` / `add_animation(...)`.
- `SpineAnimationCommand::set_empty(...)` / `add_empty(...)` are now `set_empty_animation(...)` / `add_empty_animation(...)`.
- The matching command variants are now `SetAnimation`, `AddAnimation`, `SetEmptyAnimation`, and `AddEmptyAnimation`.
- Existing `set_empty_animations`, `clear_track`, `clear_tracks`, `set_default_mix`, `set_mix`, and `clear_mix_data` names already describe their core target behavior and were kept.
- Bevy examples and runtime tests were migrated to the expanded constructor names.

# Verification

Passed:

- `cargo fmt --all -- --check`
- `cargo check -p spine2d-bevy --examples`
- `cargo nextest run -p spine2d-bevy --no-fail-fast --status-level fail` (`43 passed, 0 skipped`)
- `cargo clippy -p spine2d-bevy --tests --examples -- -D warnings`

# Next Action

Re-scan for remaining wrappers where short names obscure core method parity.

# Citations

- `spine2d-bevy/src/components.rs`
- `spine2d-bevy/src/systems.rs`
- `spine2d-bevy/examples`
- `spine2d/src/runtime/animation_state.rs`

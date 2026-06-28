---
type: "Work Progress"
title: "Bevy Spine name builders renamed"
description: "Work Progress for aligning the Bevy Spine root component's name-based builders with its getters and setters."
timestamp: 2026-06-28T14:41:53Z
tags: ["spine-cpp", "parity", "bevy", "refactor"]
source_session: "manual"
---

# Summary

Renamed the Bevy root `Spine` component builders from `with_animation(...)` / `with_skin(...)` to `with_animation_name(...)` / `with_skin_name(...)`.

# Details

- The root `Spine` component stores initial animation and skin values as names, matching `get_animation_name` / `set_animation_name` and `get_skin_name` / `set_skin_name`.
- Bevy examples and runtime tests now use the explicit name-based builders.
- Internal `SpineInstanceParts::with_animation_name(...)` and `with_skin_name(...)` already used the aligned naming and did not need behavior changes.

# Verification

Passed:

- `cargo fmt --all -- --check`
- `cargo check -p spine2d-bevy --examples`
- `cargo nextest run -p spine2d-bevy --no-fail-fast --status-level fail` (`43 passed, 0 skipped`)
- `cargo clippy -p spine2d-bevy --tests --examples -- -D warnings`

# Next Action

Stop unless a remaining wrapper name has the same clear value-vs-object mismatch.

# Citations

- `spine2d-bevy/src/components.rs`
- `spine2d-bevy/src/systems.rs`
- `spine2d-bevy/examples`

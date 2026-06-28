---
type: "Work Progress"
title: "Bevy animation clear-mix-data command renamed"
description: "Work Progress for aligning the Bevy animation mix-data clear command with core AnimationStateData behavior."
timestamp: 2026-06-28T15:28:50Z
tags: ["spine-cpp", "parity", "bevy", "refactor"]
source_session: "manual"
---

# Summary

Renamed `SpineAnimationCommand::clear_mixes(...)` and `SpineAnimationCommandKind::ClearMixes` to `clear_mix_data(...)` / `ClearMixData`.

# Details

- Core `AnimationStateData::clear()` removes named mix durations and resets `default_mix` to `0.0`, matching latest-tag C++ `AnimationStateData::clear()`.
- The Bevy command name now describes the full mix-data reset instead of implying only named mix pairs are removed.
- The focused Bevy runtime test name was updated to document that clearing mix data also resets default mix.

# Verification

Passed:

- `cargo fmt --all -- --check`
- `cargo check -p spine2d-bevy --examples`
- `cargo nextest run -p spine2d-bevy --no-fail-fast --status-level fail` (`43 passed, 0 skipped`)
- `cargo clippy -p spine2d-bevy --tests --examples -- -D warnings`

# Next Action

Continue only if another wrapper name has comparably clear semantic drift.

# Citations

- `spine2d-bevy/src/components.rs`
- `spine2d-bevy/src/systems.rs`
- `spine2d/src/runtime/animation_state.rs`
- `repo-ref/spine-runtimes/spine-cpp/include/spine/AnimationStateData.h`

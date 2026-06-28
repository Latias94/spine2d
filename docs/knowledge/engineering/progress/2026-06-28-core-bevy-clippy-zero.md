---
type: "Work Progress"
title: "Core and Bevy Clippy zero-warning pass"
description: "Work Progress for clearing current core and Bevy clippy findings."
timestamp: 2026-06-28T13:14:25Z
tags: ["spine-cpp", "parity", "core", "bevy", "clippy", "refactor"]
source_session: "manual"
---

# Summary

Cleared the remaining visible `spine2d` core Clippy warnings and then re-ran the previously blocked Bevy Clippy gate. Core and Bevy lint gates are now green for the checked scopes.

# Details

- `Animation::apply(...)` keeps its public C++-mirroring signature and now carries an explicit `#[expect(clippy::too_many_arguments)]` reason.
- Internal public-animation timeline dispatch now uses `PublicAnimationApply` and `PublicTimelineApply` context structs instead of passing a long parameter list through helper functions.
- `TrackEntry::mix_percent()` now uses `clamp(0.0, 1.0)` after parity-checking the final non-linear interpolation bounds behavior against local `spine-cpp` `TrackEntry::mix()`.
- Bevy atlas loading now parses via `atlas_text.parse::<Atlas>()`, removing the redundant borrow after the `Atlas::parse` wrapper was removed.
- `SpineTrackState::new(...)` now takes a named `SpineTrackStateParts` value, so the runtime snapshot construction no longer relies on a 12-argument constructor.

# Verification

Passed:

- `cargo fmt --all -- --check`
- `cargo check -p spine2d --features json,binary,upstream-smoke`
- `cargo test -p spine2d --features json,binary,upstream-smoke --lib --no-run`
- `cargo clippy -p spine2d --features json,binary,upstream-smoke --lib -- -D warnings`
- `cargo nextest run -p spine2d --features json,binary,upstream-smoke --lib runtime::animation_tests --no-fail-fast --status-level fail` (`23 passed, 654 skipped`)
- `cargo nextest run -p spine2d --features json,binary,upstream-smoke --lib runtime::animation_state_mixing_semantics_tests --no-fail-fast --status-level fail` (`16 passed, 661 skipped`)
- `cargo nextest run -p spine2d-bevy --no-fail-fast --status-level fail` (`43 passed, 0 skipped`)
- `cargo clippy -p spine2d-bevy --tests --examples -- -D warnings`
- `cargo check -p spine2d-bevy --examples`
- `cargo check -p spine2d-wgpu -p spine2d-web` (passed with the existing `block v0.1.6` future-incompatibility warning)
- `git diff --check`

# Next Action

Continue with small parity-backed cleanup slices. If committing, stage only files intentionally touched by the current logical unit; the working tree still contains many other pre-existing modified and untracked files from earlier slices.

# Citations

- `spine2d/src/model.rs`
- `spine2d/src/runtime/animation.rs`
- `spine2d/src/runtime/animation_state.rs`
- `spine2d-bevy/src/asset_loader.rs`
- `spine2d-bevy/src/components.rs`
- `spine2d-bevy/src/systems.rs`
- `cargo clippy -p spine2d --features json,binary,upstream-smoke --lib -- -D warnings`
- `cargo clippy -p spine2d-bevy --tests --examples -- -D warnings`

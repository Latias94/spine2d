---
type: "Work Progress"
title: "Bevy Spine skin-name getter renamed"
description: "Work Progress for tightening the Bevy Spine component's initial-skin getter name."
timestamp: 2026-06-28T14:30:26Z
tags: ["spine-cpp", "parity", "bevy", "refactor"]
source_session: "manual"
---

# Summary

Renamed `Spine::get_skin()` to `Spine::get_skin_name()` because the Bevy root component stores an optional initial skin name, not a resolved core skin object.

# Details

- `Spine::get_skin_name()` now matches `Spine::set_skin_name(...)` and the internal `SpineInstance::get_skin_name()` surface.
- The instance creation system now reads the initial skin through the explicit name getter.
- The follow-up builder rename slice completes this naming alignment with `Spine::with_skin_name(...)`.

# Verification

Passed:

- `cargo fmt --all -- --check`
- `cargo check -p spine2d-bevy --examples`
- `cargo nextest run -p spine2d-bevy --no-fail-fast --status-level fail` (`43 passed, 0 skipped`)
- `cargo clippy -p spine2d-bevy --tests --examples -- -D warnings`

# Next Action

Continue scanning wrapper names for the same kind of value-vs-object ambiguity.

# Citations

- `spine2d-bevy/src/components.rs`
- `spine2d-bevy/src/systems.rs`

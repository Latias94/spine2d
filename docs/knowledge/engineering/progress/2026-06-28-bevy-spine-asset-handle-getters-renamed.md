---
type: "Work Progress"
title: "Bevy Spine asset-handle getters renamed"
description: "Work Progress for disambiguating Bevy Spine root component asset handles from runtime skeleton and atlas objects."
timestamp: 2026-06-28T15:44:15Z
tags: ["spine-cpp", "parity", "bevy", "refactor"]
source_session: "manual"
---

# Summary

Renamed root `Spine` component asset accessors from `get_skeleton` / `set_skeleton` and `get_atlas` / `set_atlas` to `get_skeleton_handle` / `set_skeleton_handle` and `get_atlas_handle` / `set_atlas_handle`.

# Details

- The root Bevy component stores `Handle<SpineSkeletonAsset>` and `Handle<SpineAtlasAsset>`, not resolved runtime objects.
- Internal runtime `SpineInstance::get_skeleton()` remains unchanged because it returns the actual `spine2d::Skeleton`.
- `SpineAtlasAsset::get_atlas()` remains unchanged because it returns the resolved atlas object stored inside the asset wrapper.
- Bevy systems and the viewer example now use the explicit handle getters/setters.

# Verification

Passed:

- `cargo fmt --all -- --check`
- `cargo check -p spine2d-bevy --examples`
- `cargo nextest run -p spine2d-bevy --no-fail-fast --status-level fail` (`43 passed, 0 skipped`)
- `cargo clippy -p spine2d-bevy --tests --examples -- -D warnings`

# Next Action

Continue scanning only for similarly clear handle/object or value/object ambiguities.

# Citations

- `spine2d-bevy/src/components.rs`
- `spine2d-bevy/src/systems.rs`
- `spine2d-bevy/examples/viewer.rs`

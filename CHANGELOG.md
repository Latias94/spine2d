# Changelog

This project follows a pragmatic changelog format during early development.
Version numbers follow SemVer, but the public API is expected to change rapidly until `1.0`.

## Unreleased

TBD

## v0.4.0

### Migration notes

- The core Rust API now follows the official Spine C++ runtime more closely. Runtime, model, event, and atlas types favor explicit getters/setters, typed handles, named lookup methods, and narrower field visibility; migrate code that accessed fields or Rust-only convenience helpers to the new accessor surface.
- `AnimationState` and `TrackEntry` removed Rust-only mix removal, snapshot, convenience, index, and time helpers. Use direct mix accessors, handle-bound entry reads, current/queued entry accessors, and the C++-aligned track-entry getters/setters instead.
- `SkeletonData` and atlas lookups now follow C++ ordering and naming more closely. Skins/events stay ordered, named lookups scan by name, atlas regions are found with `find_region`, and C++ page/region metadata is exposed through getters.
- Bevy animation-state configuration now uses `SpineAnimationStateConfig` instead of the temporary `SpineAnimationMixes`; mix data is configured through `SpineAnimationStateConfig` or `SpineAnimationCommand::{set_default_mix,set_mix,clear_mix_data}`, and the Rust-only `remove_mix` command was removed.
- Per-entry Bevy playback options now use `SpineTrackEntrySettings` through settings-bearing animation commands, so gameplay code no longer needs to store raw runtime track-entry handles.
- Bevy wrapper names now make names, handles, and runtime objects explicit:

| Old API | New API |
| --- | --- |
| `Spine::with_animation(...)` | `Spine::with_animation_name(...)` |
| `Spine::with_skin(...)` | `Spine::with_skin_name(...)` |
| `Spine::get_animation()` | `Spine::get_animation_name()` |
| `Spine::get_skin()` | `Spine::get_skin_name()` |
| `Spine::get_skeleton()` / `set_skeleton(...)` | `Spine::get_skeleton_handle()` / `set_skeleton_handle(...)` |
| `Spine::get_atlas()` / `set_atlas(...)` | `Spine::get_atlas_handle()` / `set_atlas_handle(...)` |
| `SpineAnimationCommand::set(...)` / `add(...)` | `SpineAnimationCommand::set_animation(...)` / `add_animation(...)` |
| `SpineAnimationCommand::set_empty(...)` / `add_empty(...)` | `SpineAnimationCommand::set_empty_animation(...)` / `add_empty_animation(...)` |
| `SpineAnimationCommand::clear_mixes(...)` | `SpineAnimationCommand::clear_mix_data(...)` |
| `SpineSkeletonControl::with_time(...)` | `SpineSkeletonControl::with_time_override(...)` |
| `SpineTrackEntrySettings::with_looped(...)` | `SpineTrackEntrySettings::with_loop(...)` |

### Added

- Runtime: add C++-aligned animation-state and track-entry controls for mix configuration, queued/current entry inspection, entry mutation, event queue control, listener cleanup, manual entry disposal, and track slot inspection.
- Runtime: add skeleton-facing APIs for physics movement, named lookups, clipping-aware bounds, attachment world vertices, update-cache and constraint inspection, split setup-pose operations, skin slot queries, mesh metadata, and skeleton color mutation.
- Model and atlas: preserve and expose more source data, including skeleton headers, default skins, ordered skins/events, constraints, affected bones, animation and attachment colors, attachment metadata, bone-slot relationships, and C++ page/region atlas metadata.
- Bevy: add `SpineSkeletonControl`, `SpineSkeletonCommand`, `SpineRuntimeState`, `SpineReady`, `SpineLifecycleEvent`, `SpineBounds`, `SpineFlipY`, animation events, animation-state config components, and animation/skeleton command messages so gameplay code can control runtime state without accessing internal handles.
- Examples: add `runtime_controls`, `mixing`, and `mixing_inspector` for skeleton controls, gameplay-style animation mixing, queued recovery, empty-animation fades, and live mix tuning.

### Fixed

- Improve AnimationState parity with Spine C++ for track-entry delays, empty-track timing/defaults, event queue behavior, reverse playback, completion boundaries, animation-time math, mix completion, hold-previous behavior, and nonpositive timeline alpha.
- Improve skeleton and animation application parity for applied draw order, applied slot pose, shared-attachment deform, bone world epochs, IK mix propagation, point attachment world rotation, event audio defaults, explicit physics world transforms, and attachment timeline resolution.

### Parity and tooling

- Refresh the upstream Spine runtime baseline to the latest verified Spine 4.3 tag, `spine-ts-4.3.8`, and keep the reproducibility anchor in `spine-upstream.toml` and `scripts/upstream_baseline.json`.
- Expand the C++ oracle and render-oracle tooling, regenerate affected goldens, add model lookup coverage, add version parsing tests, and prune stale parity plans so the release tracks the current Spine C++ behavior more directly.

## 0.3.0

- Bevy: upgrade `spine2d-bevy` to `bevy 0.19.0` and make it publish alongside the core crates. The backend itself landed in PR #2 from @Iraeis.
- Runtime: refresh the Spine 4.3 baseline and merge the latest parity fixes.
- Tooling: raise the workspace MSRV to `1.95`, fix the remaining `clippy -D warnings` issues, and upgrade GitHub Actions.

## 0.2.0

- Parity: fix constraint pose timelines to apply even when the constraint is not in the update cache (Spine 4.3 `PosedActive` vs `Constraint::_active` semantics), locked by a new C++ oracle scenario.
- Render: add a regression test for clipping endSlot when the end slot bone is inactive (prevents “clipping leaks” to subsequent slots).
- Render oracle: add scenario-mode command stream support (`--set/--add/--mix/--entry-*/--step`) to lock down multi-track mixing + clipping geometry parity against the upstream C++ runtime.
- Tests: add render-oracle scenario parity cases (JSON + `.skel`) and record corresponding new goldens.
- Packaging: silence a `dead_code` warning in default (no-feature) builds by gating JSON-only helpers.
- Docs: clarify render oracle workflow and scenario coverage in `docs/parity.md` and `docs/roadmap.md`.

## 0.1.0

Initial experimental release.

Highlights:
- Pure Rust Spine 4.3 runtime core (`spine2d`) with JSON parsing and renderer-agnostic draw output.
- Native wgpu integration crate (`spine2d-wgpu`) with a runnable viewer example.
- wasm32 demo crate (`spine2d-web`, not published) for `wasm32-unknown-unknown` validation.
- Oracle-driven parity workflow against upstream `spine-runtimes` (pinned by commit) to avoid “approximate” behaviour.

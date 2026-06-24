# Changelog

This project follows a pragmatic changelog format during early development.
Version numbers follow SemVer, but the public API is expected to change rapidly until `1.0`.

## Unreleased

TBD

## 0.4.0

- Breaking Bevy API: replace the temporary `SpineAnimationMixes` surface with `SpineAnimationStateConfig`, add settings-bearing `SpineAnimationCommand` paths, and expose `SpineTrackEntrySettings` for per-entry playback controls.
- Bevy: add `SpineSkeletonControl`, `SpineSkeletonCommand`, and `SpineRuntimeState` so gameplay code can configure physics/wind/gravity/time and observe active tracks without accessing internal runtime handles.
- Runtime: make `AnimationStateData` mix configuration a validated API, add `set_empty_animations`, expose queued/current track snapshots, and add skeleton wind/gravity getters.
- Examples: add `spine2d-bevy --example runtime_controls`, a gameplay-style `spine2d-bevy --example mixing` demo for locomotion, one-shot actions, queued recovery, and empty-animation fades, plus an egui-powered `spine2d-bevy --example mixing_inspector` for live mix tuning.

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

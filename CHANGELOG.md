# Changelog

This project follows a pragmatic changelog format during early development.
Version numbers follow SemVer, but the public API is expected to change rapidly until `1.0`.

## Unreleased

- TBD

## 0.3.0

- Bevy: upgrade `spine2d-bevy` to `bevy 0.19.0` and make it publish alongside the core crates.
- Runtime: refresh the Spine 4.3 baseline and merge the latest parity fixes.
- Tooling: raise the workspace MSRV to `1.95`, fix the remaining `clippy -D warnings` issues, and upgrade GitHub Actions.

## 0.2.0

- Parity: fix constraint pose timelines to apply even when the constraint is not in the update cache (Spine 4.3 `PosedActive` vs `Constraint::_active` semantics), locked by a new C++ oracle scenario.
- Render: add a regression test for clipping endSlot when the end slot bone is inactive (prevents “clipping leaks” to subsequent slots).
- Render oracle: add scenario-mode command stream support (`--set/--add/--mix/--entry-*/--step`) to lock down multi-track mixing + clipping geometry parity against the upstream C++ runtime.
- Tests: add render-oracle scenario parity cases (JSON + `.skel`) and record corresponding new goldens.
- Packaging: silence a `dead_code` warning in default (no-feature) builds by gating JSON-only helpers.
- Docs: clarify render oracle workflow and scenario coverage in `docs/parity-4.3-beta.md` and `docs/roadmap.md`.

## 0.1.0

Initial experimental release.

Highlights:
- Pure Rust Spine 4.3 runtime core (`spine2d`) with JSON parsing and renderer-agnostic draw output.
- Native wgpu integration crate (`spine2d-wgpu`) with a runnable viewer example.
- wasm32 demo crate (`spine2d-web`, not published) for `wasm32-unknown-unknown` validation.
- Oracle-driven parity workflow against upstream `spine-runtimes` (pinned by commit) to avoid “approximate” behaviour.

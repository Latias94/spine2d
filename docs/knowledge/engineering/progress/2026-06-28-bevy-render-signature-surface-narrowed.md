---
type: "Work Progress"
title: "Bevy render signature surface narrowed"
description: "Work Progress for Bevy internal render signature cache surface narrowing."
timestamp: 2026-06-28T12:04:53Z
tags: ["spine-cpp", "parity", "bevy", "refactor"]
source_session: "manual"
---

# Summary

The Bevy internal render signature cache no longer exposes mutable fields to the systems layer. Render systems and tests now update signature state through small crate-internal methods, while mesh child components expose their mesh handle through an accessor instead of a public field.

# Details

- `SpineDrawSignatureCache` now keeps its signature private and exposes `get_signature()`, `set_signature(...)`, and `set_render_layers(...)`.
- `SpineRenderSignature` now keeps draw signatures and render layers private behind `new(...)`, `get_draws()`, `get_render_layers()`, and `set_render_layers(...)`.
- `SpineDrawSignature` now keeps texture path, blend mode, and premultiplied-alpha state private and continues to build from runtime draw data through `from_draw(...)`.
- `SpineMeshChild` now keeps its mesh handle private behind `new(...)` and `get_mesh()`.
- `spine2d-bevy/src/systems/render.rs` and render-focused tests now use the accessor surface instead of direct field reads/writes.
- `cargo clippy -p spine2d-bevy --tests --examples -- -D warnings` was attempted but failed in the core `spine2d` crate on pre-existing Clippy items (`too_many_arguments`, `manual_clamp`, `derivable_impls`, etc.); no Bevy-specific warning remained after removing a test-only `SpineDrawSignature::new(...)` helper.

# Verification

- `rg -n "signature_cache\\.signature|\\.signature\\.(draws|render_layers)|mesh_child\\.mesh|SpineDrawSignature \\{|SpineRenderSignature \\{|SpineDrawSignatureCache \\{" spine2d-bevy/src/systems.rs spine2d-bevy/src/systems/render.rs spine2d-bevy/src/components.rs`
- `cargo fmt --all -- --check`
- `cargo nextest run -p spine2d-bevy --no-fail-fast --status-level fail` (`43 passed, 0 skipped`)
- `cargo test -p spine2d-bevy --example viewer --no-run`
- `cargo test -p spine2d-bevy --example basic --no-run`
- `git diff --check`
- `python /Users/frankorz/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`

# Next Action

Continue Bevy cleanup by looking for internal-only field bags first. Keep public Bevy message/event payloads field-public unless there is a concrete compatibility or maintenance reason to wrap them.

# Citations

- `spine2d-bevy/src/components.rs`
- `spine2d-bevy/src/systems.rs`
- `spine2d-bevy/src/systems/render.rs`
- `cargo nextest run -p spine2d-bevy --no-fail-fast --status-level fail`
- `cargo test -p spine2d-bevy --example viewer --no-run`
- `cargo test -p spine2d-bevy --example basic --no-run`
- `cargo clippy -p spine2d-bevy --tests --examples -- -D warnings`

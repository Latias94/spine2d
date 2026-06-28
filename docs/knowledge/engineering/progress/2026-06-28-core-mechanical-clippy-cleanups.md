---
type: "Work Progress"
title: "Core mechanical Clippy cleanups"
description: "Work Progress for low-risk core runtime/parser mechanical cleanup."
timestamp: 2026-06-28T12:21:49Z
tags: ["spine-cpp", "parity", "core", "refactor"]
source_session: "manual"
---

# Summary

Cleaned up a small set of core `spine2d` parser/runtime implementation details that were flagged during a focused Clippy-style audit and do not change the intended latest-tag `spine-cpp` behavior. Riskier suggestions, especially `manual_clamp` in animation mixing, were intentionally left untouched because they can affect NaN semantics.

# Details

- `spine2d/src/atlas.rs` now branches on `current_page` with `if let Some(page_index)` instead of checking `is_none()` and later calling `expect(...)`.
- `spine2d/src/json.rs` and `spine2d/src/binary.rs` now pass `Arc::unwrap_or_clone` directly to `map(...)` instead of wrapping it in a redundant closure.
- `spine2d/src/runtime/skeleton.rs` no longer explicitly dereferences constraint data references in `ConstraintRefMut::get_data()`.
- `spine2d/src/runtime/animation.rs` derives `Default` for `MixInterpolation` with `Linear` marked as the default variant.
- `spine2d/src/runtime/animation.rs` also removed a redundant `Option::as_deref_mut()` call for `Option<&mut Vec<Event>>`.
- The Clippy `manual_clamp` suggestion in `TrackEntry::mix_percentage()` was not applied because `f32::clamp` has different NaN/panic semantics than the existing C++-style direct comparisons.

# Verification

Passed:

- `rg -n "current_page\\.expect|\\.map\\(\\|event\\| Arc::unwrap_or_clone\\(event\\)\\)|impl Default for MixInterpolation|as_deref_mut\\(\\)|ConstraintDataRef::(Ik|Transform|Path|Physics|Slider)\\([^\\n]*\\*data" spine2d/src`
- `cargo fmt --all -- --check`
- `cargo check -p spine2d --features json,binary,upstream-smoke`
- `cargo test -p spine2d --features json,binary,upstream-smoke --lib --no-run`
- `git diff --check`

Attempted but interrupted because the test binary hung:

- `cargo nextest run -p spine2d --features json,binary,upstream-smoke --lib atlas --no-fail-fast --status-level fail`
- `cargo nextest run -p spine2d --features json,binary,upstream-smoke --lib model_lookup_tests runtime::skeleton_tests --no-fail-fast --status-level fail`
- `cargo test -p spine2d --features json,binary,upstream-smoke parse_minimal_atlas_one_page_one_region -- --exact --nocapture`

# Next Action

Continue parity cleanup with small, evidence-backed slices. Avoid applying generic Clippy suggestions that alter runtime floating-point or public API semantics unless they are first checked against local `repo-ref/spine-runtimes/spine-cpp`.

# Citations

- `spine2d/src/atlas.rs`
- `spine2d/src/json.rs`
- `spine2d/src/binary.rs`
- `spine2d/src/runtime/skeleton.rs`
- `spine2d/src/runtime/animation.rs`
- `spine2d/src/runtime/animation_state.rs`
- `cargo check -p spine2d --features json,binary,upstream-smoke`
- `cargo test -p spine2d --features json,binary,upstream-smoke --lib --no-run`

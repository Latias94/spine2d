---
type: "Decision"
title: "slider constraint keeps runtime bone binding as optional index for now"
description: "Decision for slider constraint keeps runtime bone binding as optional index for now."
timestamp: 2026-06-26T09:56:25Z
tags: ["spine-rs", "spine-cpp", "slider", "decision"]
source_session: "current"
---

# Decision

# Context

Latest-tag `spine-cpp` stores slider animation and bone bindings as raw pointers inside `SliderData`, but Rust cannot safely keep self-referential references into the same `SkeletonData` object graph. The internal storage therefore remains an optional index, while the public getter resolves to `Option<&Animation>` through `SkeletonData` so callers see object-level access instead of layout leakage.

# Alternatives

1. Expose the raw `Option<usize>` publicly. This is easy but leaks Rust internals and diverges from the C++ object-level surface.
2. Store `&Animation` directly. This does not fit Rust ownership without unsafe self-references or a separate arena.
3. Keep the internal index and resolve it on read. This preserves soundness and keeps the public API closer to C++.

# Consequences

The model layer stays ownership-safe and the public API no longer advertises parser indexes. Internal parser/runtime code still binds slider animations by index, but callers now ask for the animation through `SkeletonData`, which matches the rest of the lookup-style API.

# Citations

- `repo-ref/spine-runtimes/spine-cpp/include/spine/SliderData.h`
- `repo-ref/spine-runtimes/spine-cpp/src/spine/SkeletonData.cpp`
- `spine2d/src/model.rs`
- `spine2d/src/model_lookup_tests.rs`
- Verification: `cargo fmt --all -- --check`
- Verification: `CARGO_TARGET_DIR=/tmp/spine-rs-target-test cargo nextest run -p spine2d --features json,binary,upstream-smoke --no-fail-fast --status-level fail`

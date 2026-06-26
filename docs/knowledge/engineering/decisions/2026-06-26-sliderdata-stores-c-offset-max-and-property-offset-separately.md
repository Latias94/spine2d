---
type: "Decision"
title: "SliderData stores C++ offset max and property offset separately"
description: "Decision for SliderData stores C++ offset max and property offset separately."
timestamp: 2026-06-26T10:29:47Z
tags: ["spine-cpp", "slider", "parity"]
git_branch: "refactor-slot-attachment-surface"
verified_by: "cargo fmt --all -- --check; CARGO_TARGET_DIR=/tmp/spine-rs-target-test cargo nextest run -p spine2d --features json,binary,upstream-smoke --no-fail-fast --status-level fail"
---

# Decision

`SliderConstraintData` now stores the three C++ slider values separately:

- `property_offset`: Rust storage for the selected `FromProperty` offset loaded from JSON `from` or binary `from`.
- `offset`: Rust storage for C++ `SliderData::_offset`, loaded from JSON `to` or the binary slider offset field.
- `max`: Rust storage for C++ `SliderData::_max`, loaded from JSON `max` or the binary nonessential `flags & 8` value when a bone-driven slider is present.

# Context

Latest-tag `spine-cpp` keeps slider setup time in `SliderPose::_time`, while bone-driven slider metadata lives on `SliderData` as `_property->_offset`, `_offset`, `_scale`, `_max`, and `_local`. The old Rust model collapsed part of that shape into `property_from` and `to`, making `get_offset()` return the C++ property offset instead of C++ `SliderData::getOffset()` semantics and leaving generated `max` metadata unrepresented.

# Alternatives

Keeping the old field names would preserve less churn but continue to hide a real semantic mismatch in the public getter surface. Adding only `max` without splitting `offset` and property offset would still leave `get_offset()` incorrect relative to C++.

# Consequences

JSON and binary parsers now follow the C++ split: slider `time` is setup-pose time only for sliders without a driving bone; bone-driven sliders load `from` into property offset, `to` into slider offset, and `max` into slider max. Runtime slider application now uses `offset + (property_value - property_offset) * scale`, matching C++ `_offset + (value - property->_offset) * _scale`.

The model accessor test now covers `get_property_offset()`, `get_offset()`, and `get_max()`. A slider timeline test fixture was corrected to remove `bone` when asserting setup-pose `time`, because C++ ignores JSON `time` for bone-driven sliders.

# Citations

- `repo-ref/spine-runtimes/spine-cpp/include/spine/SliderData.h`
- `repo-ref/spine-runtimes/spine-cpp/src/spine/SkeletonJson.cpp`
- `repo-ref/spine-runtimes/spine-cpp/src/spine/SkeletonBinary.cpp`
- `spine2d/src/model.rs`
- `spine2d/src/json.rs`
- `spine2d/src/binary.rs`
- `spine2d/src/runtime/skeleton/slider.rs`
- Verification: `cargo fmt --all -- --check`
- Verification: `CARGO_TARGET_DIR=/tmp/spine-rs-target-test cargo nextest run -p spine2d --features json,binary,upstream-smoke --no-fail-fast --status-level fail`

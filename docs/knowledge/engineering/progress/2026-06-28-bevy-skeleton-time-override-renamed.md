---
type: "Work Progress"
title: "Bevy skeleton time override renamed"
description: "Work Progress for disambiguating the Bevy skeleton-control optional time override from core Skeleton time access."
timestamp: 2026-06-28T15:39:24Z
tags: ["spine-cpp", "parity", "bevy", "refactor"]
source_session: "manual"
---

# Summary

Renamed the optional time field on `SpineSkeletonControl` from `time` to `time_override`, with matching `get_time_override` / `set_time_override` / `with_time_override` methods.

# Details

- `SpineSkeletonControl` stores an optional component-level override; when absent, it does not overwrite the runtime skeleton's current time.
- Core `Skeleton::get_time` / `set_time` and Bevy `SpineSkeletonCommand::set_time(...)` remain unchanged because those APIs directly read or write skeleton time.
- The spawn-control test now uses `with_time_override(...)`, while assertions continue to read the resolved skeleton time through `Skeleton::get_time()`.

# Verification

Passed:

- `cargo fmt --all -- --check`
- `cargo check -p spine2d-bevy --examples`
- `cargo nextest run -p spine2d-bevy --no-fail-fast --status-level fail` (`43 passed, 0 skipped`)
- `cargo clippy -p spine2d-bevy --tests --examples -- -D warnings`

# Next Action

Re-scan the remaining skeleton command wrapper names and keep `set_time` unchanged where it directly maps to core `Skeleton::set_time`.

# Citations

- `spine2d-bevy/src/components.rs`
- `spine2d-bevy/src/systems.rs`
- `spine2d/src/runtime/skeleton.rs`

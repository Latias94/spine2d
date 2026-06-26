---
type: "Session Handoff"
title: "spine cpp parity slider accessor refresh"
description: "Session Handoff for spine cpp parity slider accessor refresh."
timestamp: 2026-06-26T10:48:00Z
tags: ["spine-rs", "spine-cpp", "slider", "parity"]
source_session: "current"
---

# Summary

继续收敛 `SliderConstraintData` 的公开 surface，并把 loader/runtime 的绑定路径收口为一致语义：公开 getter 仍通过 `SkeletonData` 返回 `Option<&Animation>`，但内部现在采用 index-first + name fallback，JSON/Binary 也同步写入索引和名字，避免手写 API 与解析路径分裂。

# Verified State

- `cargo fmt --all -- --check` passed.
- `cargo check -p spine2d --features json,binary,upstream-smoke` passed.
- `CARGO_TARGET_DIR=/tmp/spine-rs-target-test cargo nextest run -p spine2d --features json,binary,upstream-smoke --no-fail-fast --status-level fail` passed with `658 passed, 2 skipped`.

# Open Threads

- Continue the parity audit by checking whether any remaining model getters still leak parser/storage indices when C++ returns object references or pointers.
- If slider binding comes up again, inspect whether the latest-tag C++ loader/runtime split expects any further distinction beyond name/index duplication.

# Next Action

继续审查模型层其余约束/附件 getter，优先清掉仍然泄露内部索引或 Rust-only 布局的公开接口，暂时跳过已经收口的 slider 绑定路径。

# Citations

- `spine2d/src/model.rs`
- `spine2d/src/model_lookup_tests.rs`
- `docs/knowledge/engineering/current-state.md`

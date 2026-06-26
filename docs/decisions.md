# Decisions (spine2d)

This document captures the current project decisions so we can iterate without re-litigating fundamentals.

## What we are building

- A **pure Rust** Spine runtime targeting **Spine 4.3** exported data.
- A **safe, renderer-agnostic** core crate (`spine2d`) that can run on **native** and **`wasm32-unknown-unknown`**.
- Optional renderer integrations as separate crates (starting with `spine2d-wgpu`).

## What we are not building (initially)

- Not a “safe binding” to the official `spine-c`/`spine-cpp` runtimes.
- Not a bindgen-based sys crate.
- Not a `c2rust` transpilation of the official C runtime.

Rationale: `wasm32-unknown-unknown` should not require Emscripten/emsdk or a C++ toolchain, and we want full control over the Rust API and memory model.

## Target version and compatibility

- Spine exported data: **4.3.x**.
- We will prioritize correctness for 4.3 before considering compatibility layers for older exports.
- We pin our upstream reference by commit:
  - Example exports used by smoke tests and demos live under `assets/spine-runtimes`.
  - The exact upstream commit is recorded in `assets/spine-runtimes/SOURCE.txt`.
  - Current refresh target: upstream tag `spine-ts-4.3.8` at `8e12b1250ab88c0f890849ea45aab80338cead63`.
  - `spine-cpp` is the sole behaviour reference for runtime parity; runtime-specific tags are only release/reproducibility markers.

## Workspace / crate layout

- `spine2d`
  - Parsing: atlas text, skeleton JSON/Binary (4.3).
  - Runtime: animation sampling, constraints, pose computation, world transforms.
  - Output: renderer-agnostic render data (e.g. batches/meshes/commands) that can be consumed by backends.
- `spine2d-wgpu`
  - Translating `spine2d` render output into wgpu buffers/pipelines.
  - Keeping wgpu-specific types out of `spine2d`.

## WASM stance

- First-class goal: `wasm32-unknown-unknown`.
- No emsdk requirement.
- APIs should accept in-memory assets (`&str`, `&[u8]`) instead of doing filesystem I/O inside the core runtime.

## Testing approach (planned)

- Start with deterministic unit tests around parsing and math.
- Add “behaviour” tests on exported demo data to validate animation sampling and transforms.
- Where feasible, compare against known-good reference outputs (golden files) rather than linking to official runtimes.
- When porting upstream tests, only port test cases that already exist in the official runtimes (e.g. `spine-csharp` `AnimationStateTests.cs`). For areas without upstream tests, prefer the C++ oracle parity workflow to lock semantics.

## Event semantics baseline

- Event ordering and edge cases are aligned to the upstream C# runtime tests (`AnimationStateTests.cs`).
- Event dispatch is **re-entrant** (calling `set_animation`/`add_animation` inside callbacks is supported).
- `apply()` follows the C# "next vs current" time model (`animationLast/trackLast` vs `nextAnimationLast/nextTrackLast`), which means calling `apply()` multiple times without an intervening `update()` can re-emit events (matching the upstream test harness behavior).

## TrackEntry ownership model

- Internals use an **arena + generational `EntryId`** instead of `Rc<RefCell<_>>` to avoid runtime borrow panics and to make disposal semantics explicit.
- `TrackEntryHandle` is an ID handle; mutating a track entry is done via `handle.*(&mut state, ...)` so mutation always flows through a single `&mut AnimationState`.

## Licensing / distribution notes

Spine is licensed by Esoteric Software. Even though this is a pure Rust implementation, users of software integrating Spine runtimes or using Spine data may need their own Spine Editor license depending on distribution and use-case. We will keep clear notices in the repository and crate docs.

## Runtime scope notes (current)

- Bone timelines are currently implemented for `rotate/translate/scale/shear` and are interpreted relative to the setup pose (matching upstream runtime semantics).
- Skins/active are aligned to upstream semantics for `skinRequired` and no-skin startup:
  - `Skeleton` starts with no skin; the `"default"` skin is used only as an attachment fallback.
  - `Skeleton::new` initializes slots to the setup pose (including resolving setup attachments) and builds the internal update cache.
  - `Skeleton::update_cache` computes bone/constraint `active` and timelines gate on `active` (inactive items are not mutated).
  - JSON `AttachmentDef.name` is treated as the attachment’s internal name (used in dumps and for default `path` when `path` is omitted).
- Events carry full payload:
  - `EventData` supports `int/float/string/audioPath/volume/balance`.
  - JSON `animations.events` keys default `int/float/string/volume/balance` from `EventData` setup values when an audio path exists; no-audio events keep C++ `Event` constructor defaults for volume/balance (`0/0`).
- Rotation mixing follows the upstream `spine-cpp` semantics:
  - Uses shortest-path angle normalization (`wrapDegrees`) for direct blends.
  - When mixing with `alpha != 1`, uses a per-TrackEntry rotation accumulator (cross detection) to avoid flip/jitter across successive applies.
- `AnimationState::apply` is moving toward `spine-cpp` parity:
  - Maintains `unkeyedState` and uses slot attachment-state (`Setup=1`, `Current=2`) to restore setup attachments for unkeyed slots.
  - Implements per-timeline property gating (`computeHold`) and per-entry thresholds (`mixAttachmentThreshold`, `mixDrawOrderThreshold`, `alphaAttachmentThreshold`).
- Curve sampling supports `linear`, `stepped`, and Spine's cubic Bezier curves (in **(time,value)** space, matching upstream runtimes). Multi-value timelines store one curve per value index.
- JSON loader supports an explicit parse scale via `SkeletonData::from_json_str_with_scale(input, scale)` (matching upstream loaders: geometry is scaled, and PathConstraint `position/spacing` are scaled only for `fixed/length` modes).
- Smoke tests use official exported `examples/*/export/*.json` to validate parsing, animation sampling, world transforms, and renderer-agnostic draw list generation without panics/NaNs.
- JSON loader supports `type: "linkedmesh"` by resolving it against its parent mesh during parsing (copying vertices/uvs/triangles), to maximize compatibility with official examples.
- JSON loader supports `type: "boundingbox"` and `type: "clipping"` parsing; the renderer applies `clipping` to `region`/`mesh` geometry (polygon triangulation + per-triangle clipping), while `boundingbox` is still ignored in render output.
- Slots support Spine `blend` modes (`normal/additive/multiply/screen`); atlas page `pma: true` is parsed, and render output tags each draw with `premultiplied_alpha` and premultiplies vertex colors when enabled.
- Slots/skins/attachments are being built incrementally; `region` attachments are parsed first to unblock renderer-agnostic render output.
- Deform timelines are parsed from `animations.*.attachments.*.*.*.deform` and applied to `Slot::deform` (unweighted: vertex positions; weighted: offsets), and the render output uses them when generating mesh vertices.
- Slot timelines currently support `animations.*.slots.*.attachment` (instant switch), `animations.*.slots.*.color` (interpolated RGBA), and `animations.*.drawOrder` (reorders `Skeleton::draw_order`).
- IK constraints are parsed from JSON `ik` and applied in `Skeleton::update_world_transform` (supports 1- and 2-bone IK with `mix`, `softness`, `stretch`, `compress`, `uniform`, and `bendPositive`).
- IK constraint timelines are parsed from `animations.*.ik` and drive `mix`, `softness`, and `bendPositive`.
- Transform constraints are parsed from JSON `transform` and applied in `Skeleton::update_world_transform`, interleaved with IK by the `order` field; all four variants (absolute/relative × local/world) are implemented, and `animations.*.transform` timelines can drive the `mix*` values.
- Path constraints are parsed from JSON `path`, can be driven by `animations.*.path` timelines (`position`, `spacing`, `mix`), and path constraint solving is implemented in `Skeleton::update_world_transform` (with unit tests covering constantSpeed on/off, rotateMode chain/chainScale, percent/proportional/length spacing + percent position, closed path wrap, `mixRotate<1`, and a `vine-pro.json` smoke test).

## Reference-oracle workflow (current)

- We keep a small C++ oracle to compare pose dumps against the official runtime.
- Tools:
  - Scenario sample (single-track, mixing, and multi-track): `spine2d/examples/pose_dump_scenario.rs` + `scripts/spine_cpp_lite_oracle.cpp`
  - Runner + diff: `scripts/run_spine_cpp_lite_oracle.zsh`, `scripts/compare_pose.py`
  - Dump contents: bones + slots (color/attachment) + drawOrder + constraint runtime values (IK/transform/path)

Note: upstream 4.3 no longer ships `spine-cpp-lite`; some scripts are still named `*cpp_lite*` for historical reasons, but the oracle builds against upstream `spine-c` + `spine-cpp`.

## Parity tracking

See `docs/parity.md` for the living “100% parity” checklist (what is done/partial/missing).

# Upstream Tests (Port Status)

This document tracks which test suites from the official Spine runtimes have been ported into `spine2d`.

Reference upstream:
- Example exports used by smoke tests are imported into `assets/spine-runtimes` (see `assets/spine-runtimes/SOURCE.txt` for the exact upstream commit).
- C++ oracle scripts require a local checkout of `spine-runtimes` sources (not committed). For best parity, use the same commit as `assets/spine-runtimes/SOURCE.txt`.
- Current refresh target: upstream tag `spine-ts-4.3.8` commit `8e12b1250ab88c0f890849ea45aab80338cead63`.
- Note: this pinned latest tag commit ships `spine-cpp`, which is the sole behaviour reference used by the parity oracle.
  - The tag currently includes only small headless test programs under `spine-c/tests` and `spine-cpp/tests`.
  - Historical test suites from removed runtimes (eg. `spine-csharp`, `spine-libgdx`) are still useful as behavioural specs, so `spine2d` keeps their ported cases even though they are no longer present on the current upstream tag.

Legend:
- ✅ ported to Rust (kept close to upstream behaviour)
- 🟡 partially ported (subset only, eg. headless/runtime-only parts)
- ⛔ not ported (engine/integration, or out of scope for now)

## Behaviour / unit tests

- ✅ `spine-runtimes/spine-csharp/tests/src/AnimationStateTests.cs`
  - Port: `spine2d/src/runtime/animation_state_tests.rs` (covers the upstream numbered cases).
- ✅ `spine-runtimes/spine-libgdx/spine-libgdx-tests/src/com/esotericsoftware/spine/AnimationStateTests.java`
  - Port: `spine2d/src/runtime/animation_state_tests.rs` (includes the extra upstream cases `#28` and `#29`).
- ✅ `spine-runtimes/spine-libgdx/spine-libgdx-tests/src/com/esotericsoftware/spine/EventTimelineTests.java`
  - Port: `spine2d/src/runtime/upstream_event_timeline_tests.rs`.
- ✅ `spine-runtimes/spine-libgdx/spine-libgdx-tests/src/com/esotericsoftware/spine/AttachmentTimelineTests.java`
  - Port: `spine2d/src/runtime/upstream_attachment_timeline_tests.rs`.
- ✅ `spine-runtimes/spine-libgdx/spine-libgdx-tests/src/com/esotericsoftware/spine/MixAndMatchTest.java`
  - Port: `spine2d/src/runtime/skin_active_semantics_tests.rs` (covers runtime-composed skins via `Skin::addSkin` + skin→skin `attachAll` semantics).
  - Also locks skin attachment insertion order for iteration and merge.
- ✅ Draw order folder mixing semantics
  - Port: `spine2d/src/runtime/animation_state_mixing_semantics_tests.rs` (locks `DrawOrderFolderTimeline` ordering after the plain draw order timeline).
- ✅ `spine-runtimes/spine-libgdx/spine-libgdx-tests/src/com/esotericsoftware/spine/IKTest.java`
  - Port: `spine2d/src/runtime/upstream_ik_demo_tests.rs` (JSON path) and `spine2d/src/runtime/upstream_ik_demo_skel_tests.rs` (`.skel` path).
  - Coverage: headless regression for `Bone.worldToLocal` usage plus `Physics::Pose` vs `Physics::Update` flow.
- ✅ `spine-runtimes/spine-c/tests/headless-test.c`
  - Port intent: `spine2d/src/runtime/upstream_spine_c_smoke_tests.rs` (headless behaviour smoke using upstream exported examples; not a C API conformance test).
- ✅ (4.3) examples “tests scope” headless sampling smoke (JSON + `.skel`)
  - Port: `spine2d/src/runtime/upstream_spine_c_smoke_tests.rs` (JSON) and `spine2d/src/runtime/upstream_spine_skel_smoke_tests.rs` (`.skel`).
  - Includes: per-animation sampling + bounded queue smoke + bounded multi-track overlay smoke (finite world transform asserts).
- ✅ `spine-runtimes/spine-libgdx/spine-libgdx-tests/src/com/esotericsoftware/spine/PhysicsTest2.java` / `PhysicsTest3.java` / `PhysicsTest4.java`
  - Port (headless): `spine2d/src/runtime/upstream_libgdx_physics_demo_tests.rs` (covers `.skel` parsing with non-1.0 scale + typical update/apply/worldTransform flow).
- 🟡 (historical) geometry/unit fixtures from older branches
  - Port: `spine2d/src/geometry_tests.rs` (ported deterministic cases for `triangulator` and `skeletonClipper`).

## Engine / integration tests (tracked, not ported)

- ⛔ `spine-runtimes/spine-godot/**/tests/unit-tests.gd` (Godot runtime API smoke tests)
- ⛔ `spine-runtimes/spine-unity/**/Tests/**` (Unity playmode/editor tests, eg. root motion)
- 🟡 `spine-runtimes/spine-cpp/tests/HeadlessTest.cpp` (upstream headless debug printer; we use a separate C++ oracle for parity instead of porting this directly)

## How to run

- Minimal (no JSON feature): `cargo test -p spine2d`
- With JSON runtime tests: `cargo test -p spine2d --features json`
- With upstream examples + oracle smoke: `cargo test -p spine2d --features json,upstream-smoke` (requires `assets/spine-runtimes/examples` or `SPINE2D_UPSTREAM_EXAMPLES_DIR`)

Current local note:
- On 2026-06-20, `cargo nextest run -p spine2d --features json,binary,upstream-smoke --no-fail-fast` passed with `533 passed, 10 skipped`.
- This run includes the refreshed `spineboy_run_to_walk_mix0_2_t0_4` `.skel` golden, the added `IKTest` `.skel` regression, the `InheritTimeline` keyed-bone regression, the `SkinData::add_skin` idempotence / last-write-wins regression, the `set_skin` skin→skin `attachAll` regression, the new `mix-and-match dance` JSON + `.skel` oracle, the new `coin_animation_t0_3`, `windmill_animation_t3_0`, `powerup_bounce_t0_7`, and `speedy_run_t0_433333` JSON + `.skel` oracles, plus the new `6_arcs_arcs_t5_666667` JSON + `.skel` oracle, `stretchyman_sneak_t1_366667` JSON + `.skel` oracle, `8_follow_through_ball_follow_through_t2_4` JSON + `.skel` oracle, and the new `food_app_search_add_bread_t1_4` and `food_app_search_add_carrot_t0_2` JSON + `.skel` oracles.

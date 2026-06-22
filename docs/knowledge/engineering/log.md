# Engineering Memory Update Log

## 2026-06-18
* **Initialization**: Created engineering wiki memory bundle.
* **Spine 4.3 latest baseline**: Active parity baseline is `spine-flutter-4.3.4` (`80dc680a4345ac09cdc5d4c1a77ec572a3f295d1`), not `4.3.2`.
* **Run-to-walk `.skel` failure isolated**: JSON scenario `oracle_spineboy_run_to_walk_mix0_2_t0_4_matches_cpp` passes. `.skel` scenario at `t=0.4` still fails on `rear-foot applied.rotation` by about `0.0115` degrees. Debug dumps show the mismatch emerges in rear-leg 2-bone IK after near-identical pre-world inputs, not from a simple `rear-foot-ik bendDirection` parse error.

## 2026-06-19
* **Run-to-walk `.skel` parity restored**: Refreshed the stale `spineboy_run_to_walk_mix0_2_t0_4` `.skel` golden to the current `spine-flutter-4.3.4` output. `cargo nextest run -p spine2d --features json,binary,upstream-smoke --no-fail-fast` now passes with `457 passed, 10 skipped`.
* **Draw batching parity locked**: Added direct `render::append_indexed` unit tests covering batch merge, color/dark-color split, and the 16-bit index limit. This closes the remaining draw batching checklist item against the official `SkeletonRenderer` batching rules.
* **WASM compile smoke passed**: `cargo check -p spine2d --target wasm32-unknown-unknown --features json,binary` passed, confirming the core crate still builds for the target the roadmap calls out as first-class.

## 2026-06-20
* **Latest upstream parity suite green**: `cargo nextest run -p spine2d --features json,binary,upstream-smoke --no-fail-fast` passed with `533 passed, 10 skipped`.
* **Golden baseline refreshed**: Re-recorded stale `sack_*` physics oracle goldens against `spine-flutter-4.3.4` commit `80dc680a4345ac09cdc5d4c1a77ec572a3f295d1`.
* **Formatting gate green**: `cargo fmt --all --check` passed.

## 2026-06-23
* **Autonomous parity hardening started**: Created `docs/plans/2026-06-23-001-refactor-spine-cpp-parity-hardening-plan.md`; full baseline gate `cargo nextest run -p spine2d --features json,binary,upstream-smoke --no-fail-fast` passed with `544 passed, 10 skipped`.
* **C++ reference narrowed**: Active baseline moved to latest verified 4.3 tag `spine-ts-4.3.8` (`8e12b1250ab88c0f890849ea45aab80338cead63`) as the reproducibility anchor. Runtime parity now treats `spine-cpp` as the sole behaviour reference; other runtime tags are metadata only.
* **Skeleton legacy code deleted**: Commit `fbc85eb` removed 634 lines of permanently disabled `#[cfg(any())]` Skeleton solver code. Post-cleanup verification stayed green: `cargo fmt --all --check`, `git diff --check`, and `cargo nextest run -p spine2d --features json,binary,upstream-smoke --no-fail-fast` (`544 passed, 10 skipped`).
* **Timeline dispatch centralized**: Commit `73edc54` moved plain/applied animation dispatch and `AnimationState` current/mixing-from timeline application onto shared internal helpers in `spine2d/src/runtime/animation.rs`. Focused nextest (`animation_state animation`) passed with `76 passed, 478 skipped`; full parity gate stayed green with `544 passed, 10 skipped`.
* **Binary timeline order centralized**: Commit `48518a5` replaced scattered binary parser `TimelineKind` pushes with `TimelineOrderBuilder`, including a debug assertion for registration-before-push mistakes. Focused `timeline_order` tests passed with `5 passed, 549 skipped`; full parity gate stayed green with `544 passed, 10 skipped`.
* **TrackEntry settings centralized**: Commit `e1e827f` moved entry settings application into `spine2d::TrackEntrySettings`, replaced Bevy's duplicate settings implementation with a type alias, deleted unused TrackEntry completion flags, and added coverage for negative delay plus `setMixDuration(mixDuration, delay)` queued-delay adjustment. Full core parity gate passed with `546 passed, 10 skipped`; Bevy nextest passed with `42 passed, 0 skipped`.
* **TrackEntry fields hidden**: Commit `fc1c241` made TrackEntry state private and exposed read-only getters for external use, matching the official C++ getter/setter shape more closely. Focused animation-state tests, Bevy nextest, Bevy check, and the full core parity gate all passed.
* **TrackEntry delay branch aligned**: Commit `f36cfa7` adjusted the new delay setter path to match `spine-cpp` branch behavior: negative delay is ignored or adjusted, while non-comparable values are not coerced through the queued-delay formula. The same focused and full gates stayed green.
* **TrackEntry control surface audit complete**: Final U5 setter audit found no additional guard changes needed; official `spine-cpp` TrackEntry setters are intentionally minimal except for negative delay handling.
* **Skeleton path scratch boundary started**: Commit `3edaa0b` moved path constraint scratch storage and capacity estimation into `skeleton::path`. Focused solver tests passed with `112 passed, 444 skipped`; full core parity gate passed with `546 passed, 10 skipped`.

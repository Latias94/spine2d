# Spine 4.3 Parity (Baseline + Module Matrix)

This document is the compact “what’s aligned / what’s not” tracker for the Spine 4.3 upstream baseline.
It is intentionally terse so it can be updated frequently.

For the detailed module-by-module upstream mapping, see `docs/upstream-audit-4.3-beta.md`.

## Baseline

- Upstream repository: `https://github.com/EsotericSoftware/spine-runtimes`
- Current refresh branch: `4.3`
- Pinned upstream commit for the current refresh: `80dc680a4345ac09cdc5d4c1a77ec572a3f295d1`
- Current refresh reference: tag `spine-flutter-4.3.4`
- Previous beta commit (examples + oracle reference): `d050ae66829ed5e46bb38690c83f792ffc2b3d8b`
  - Imported examples: `assets/spine-runtimes/SOURCE.txt`
  - Golden dumps (C++ oracle outputs): `spine2d/tests/golden/SOURCE.txt` (Status: OK)
- Historical note: this file records the 4.3-beta migration context; current work uses the pinned latest tag commit above.

## Current parity status

- Unit tests: `cargo test -p spine2d --features json` ✅
- Upstream smoke / oracle parity: `cargo test -p spine2d --features json,binary,upstream-smoke` ✅
- Local verification on 2026-06-18: `cargo nextest run -p spine2d --features json,binary --lib` passed.
- The refresh must re-record oracle goldens before this status is considered current for `spine-flutter-4.3.4 @ 80dc680a4345ac09cdc5d4c1a77ec572a3f295d1`.

## Module matrix (4.3)

Legend:
- ✅ verified (has a regression signal *for this baseline*)
- 🟡 implemented, verification pending (usually needs golden re-record / more tests)
- ⛔ missing

| Area | Module | Status | Regression signal |
|---|---|---:|---|
| Parsing | JSON 4.3 schema compat (`constraints: [...]`, `source`/`slot` aliases) | ✅ | unit tests + upstream examples smoke |
| Parsing | Binary `.skel` | ✅ | unit tests + oracle parity |
| Runtime | `Skeleton` init/setup/cache semantics | ✅ | unit tests + oracle parity |
| Runtime | `AnimationState` (queue/mix/thresholds/etc) | ✅ | ported upstream behaviour tests + oracle parity |
| Runtime | Constraints (IK/Transform/Path ordering + solve) | ✅ | unit tests + oracle parity |
| Runtime | Physics constraints (update/pose/reset + long-run/jitter) | ✅ | oracle parity |
| Runtime | Slider constraint (4.3) | ✅ | oracle parity (diamond) |
| Render | Color chain `skeleton * slot * attachment` + alpha==0 skip | ✅ | unit tests + render oracle goldens |
| Render | Atlas UV/trim/rotate + batching/blend/PMA | ✅ | unit tests + render oracle goldens |
| Performance | Constraint update scratch + path-constraint scratch (no per-frame allocs) | ✅ | code review (scratch reuse) |

## Notes (recent alignment)

- `.skel` mesh `triangles` and `edges` indices are encoded as varints (not `u16` arrays).
- Spine 4.3 animation streams include a `slider timelines` section even when empty; we must read the 0-count to keep the cursor aligned.
- C++ oracle pose dumps must use `*_get_applied_pose()` values (constraints update uses applied values, not “raw pose” fields).
- Spine 4.3 constraints have two independent “active” flags (`PosedActive` vs `Constraint::_active`): timelines update constraint poses even when the constraint is not in the update cache (locked by `mix_and_match_walk_plus_dress_up_add_t0_4`).
- Sequence timelines under multi-track `MixBlend::Add` are locked by `diamond_idle_rotating_plus_rotation_add_t0_5`.
- Slider timelines + sequence timelines under multi-track `MixBlend::Add` are locked by `diamond_idle_rotating_plus_idle_still_add_t0_5` (and the mix-out semantics by `diamond_idle_rotating_plus_idle_still_add_to_empty_mix0_2_t0_1`).
- IK: do not clamp `mix` to `[0,1]` (Add blending can legitimately push it beyond 1).
- `.skel` PathConstraint: `positionMode` flags must be decoded as `(flags >> 1) & 2` (matches spine-cpp).
- Linked mesh: do not overwrite the linked mesh `path` with the parent mesh `path` during resolution (linked meshes can share vertices but use a different region).
- C++ oracle note: upstream `spine-cpp/src/spine/Slider.cpp` uses an invalid `Timeline* -> SlotTimeline*` cast; `scripts/run_spine_cpp_lite_{oracle,render_oracle}.zsh` compile a patched copy for oracle-only.
- `AnimationState::setAnimation`: when switching to a different animation, mixingFrom is set even if the previous entry was never applied yet (`_nextTrackLast == -1`), and this affects pose parity during immediate mix-out scenarios.
- This edge is locked by `tank_drive_plus_shoot_add_to_empty_immediate_mix0_2_t0_1` (track1 Add -> empty without ever applying the Add entry).
- Attachment + drawOrder gating during “unapplied mixingFrom” is locked by `spineboy_ess_run_to_empty_immediate_mix0_2_mixAttachmentThreshold_1_mixDrawOrderThreshold_1_t0_1` (`mixAttachmentThreshold=1`, `mixDrawOrderThreshold=1`).

## Coverage gaps (next)

- Render parity is enforced by golden tests (JSON + `.skel`) under `--features upstream-smoke`.
- The C++ render oracle supports both legacy single-anim dumps and scenario-mode command streams (`--set/--mix/--step/...`) to lock multi-track mixing + clipping geometry.
- Next: expand render scenarios when a new visual delta is discovered (eg. additional skins, endSlot clipping edge cases, MixBlend::Add long-tail).

## Re-record checklist (baseline refresh)

When bumping the upstream commit (or switching tags), do this once:

1. Re-import examples (writes commit pin):
   - `python3 ./scripts/prepare_spine_runtimes_web_assets.py --scope tests --mode export --rev spine-flutter-4.3.4`
2. Re-record C++ oracle pose goldens (writes commit pin):
   - `python3 ./scripts/record_oracle_goldens.py --keep-going`
3. Re-record C++ oracle render goldens:
   - `python3 ./scripts/record_oracle_render_goldens.py --formats all`
4. Re-run parity:
   - `cargo nextest run -p spine2d --features json,binary,upstream-smoke`

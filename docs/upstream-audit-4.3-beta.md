# Spine 4.3 Upstream Audit Map (Module-by-Module)

This document is the **systematic, per-module parity checklist** for achieving **100% behavioural parity**
with the official Spine runtimes.

It complements:
- `docs/parity-4.3-beta.md` (baseline pin + high-level matrix)
- `docs/parity.md` (living feature checklist + oracle scenario inventory)

## Baseline

- Upstream repo: `https://github.com/EsotericSoftware/spine-runtimes`
- Reference tag `spine-ts-4.3.8`
- Pinned commit for the current refresh: `8e12b1250ab88c0f890849ea45aab80338cead63`
- Previous beta / interim refresh commits: `d050ae66829ed5e46bb38690c83f792ffc2b3d8b`, `80dc680a4345ac09cdc5d4c1a77ec572a3f295d1`
- Historical note: this file keeps the 4.3-beta audit name, but current work uses `spine-cpp` from the pinned latest tag commit above.

## Reference ladder (when runtimes differ)

We must avoid “accidental parity” where we match one runtime but break another.

1. **Primary reference:** `spine-cpp` (Spine 4.3)
2. **Secondary reference:** `spine-ts/spine-core` (when `spine-cpp` is ambiguous or known to differ)
3. **Renderer-specific reference:** `spine-ts/spine-webgl` (for renderer state-machine semantics such as clipping edge-cases)

If we intentionally follow a non-`spine-cpp` behaviour for a specific case, it must be recorded in `docs/decisions.md`.

## Audit workflow (per module)

For each module below:

1. **Pin the upstream reference** (file + function/symbol).
2. **Add/confirm a regression signal**:
   - Prefer: oracle scenario (`--features upstream-smoke`) comparing dumps to upstream.
   - Otherwise: port an upstream unit test if it exists.
   - Otherwise: add a minimal deterministic unit test.
3. Implement until the signal is green.
4. Update the status cell and add a short note if the behaviour is non-obvious.

Legend:
- ✅ aligned (verified by tests/oracle for this baseline)
- 🟡 implemented but not fully verified / known gaps
- 🔴 known mismatch (has failing signal)
- ⛔ missing

---

## Parsing (JSON)

| Area | Upstream reference | spine2d reference | Status | Signal |
|---|---|---|---:|---|
| Skeleton JSON root + scale | `spine-cpp/src/spine/SkeletonJson.cpp` | `spine2d/src/json.rs` | ✅ | unit + upstream smoke |
| Bones/Slots core schema | `SkeletonJson::readSkeletonData` | `SkeletonData::from_json_*` | ✅ | unit + oracle |
| Skins/Attachments schema | `SkeletonJson::readSkin` + `readAttachment` | `spine2d/src/json.rs` | ✅ | upstream examples smoke + render goldens |
| Events: defaults + key overrides | `SkeletonJson::readEvents` + animation event keys | `spine2d/src/json.rs` | ✅ | unit tests |
| Animation timelines (JSON) | `SkeletonJson::readAnimation` | `spine2d/src/json.rs` + runtime apply | ✅ | upstream tests + oracle |

## Parsing (Binary `.skel`)

| Area | Upstream reference | spine2d reference | Status | Signal |
|---|---|---|---:|---|
| Header + strings table | `spine-cpp/src/spine/SkeletonBinary.cpp` | `spine2d/src/binary.rs` | ✅ | unit + `.skel` smoke |
| Bones | `SkeletonBinary::readSkeletonData` | `spine2d/src/binary.rs` | ✅ | unit + oracle |
| Slots | `SkeletonBinary::readSkeletonData` | `spine2d/src/binary.rs` | ✅ | unit + oracle |
| Constraints ordered list (4.3) | `SkeletonBinary::readSkeletonData` | `spine2d/src/binary.rs` | ✅ | unit + oracle |
| Skins (default + named) | `SkeletonBinary::readSkin` | `spine2d/src/binary.rs` | ✅ | unit + oracle |
| Attachments: region/bbox/mesh/linkedmesh/path/point/clipping | `SkeletonBinary::readAttachment` | `read_attachment` | ✅ | smoke + oracle + render goldens |
| Mesh indices encoding | `SkeletonBinary::readUnsignedShortArray` (varint indices) | `read_attachment(mesh)` | ✅ | `.skel` smoke |
| Animations stream layout | `SkeletonBinary::readAnimation` | `read_animation` | ✅ | `.skel` smoke + oracle |
| Slider timelines section (4.3) | `SkeletonBinary::readAnimation` (“Slider timelines.”) | `read_animation` | ✅ | `.skel` smoke |
| Slider constraints + timelines | `spine-cpp/src/spine/Slider*` | model + runtime + timelines | ✅ | oracle (diamond) |

---

## Runtime core

| Area | Upstream reference | spine2d reference | Status | Signal |
|---|---|------:|---:|---|
| Skeleton init: setup pose + cache | `spine-cpp/src/spine/Skeleton.cpp` | `spine2d/src/runtime/skeleton.rs` | ✅ | unit + oracle |
| Bone transforms + inherit | `spine-cpp/src/spine/Bone.cpp` | `spine2d/src/runtime/bone.rs` | ✅ | unit + oracle |
| Slot state + attachment set/reset | `spine-cpp/src/spine/Slot.cpp` | `spine2d/src/runtime/slot.rs` | ✅ | unit + oracle |
| Skins: default skin fallback | `spine-cpp/src/spine/Skin.cpp` | `spine2d/src/runtime/skin.rs` | ✅ | oracle |
| Linked mesh resolution / timeline attachment | `spine-cpp/src/spine/MeshAttachment.cpp` | model + runtime mesh | ✅ | oracle scenarios + render goldens |

## AnimationState / Mixing

| Area | Upstream reference | spine2d reference | Status | Signal |
|---|---|---|---:|---|
| Event queue ordering + re-entrancy | `spine-csharp` / `spine-libgdx` tests | `spine2d/src/runtime/animation_state*` | ✅ | ported tests |
| Mixing thresholds + unkeyed restore | `spine-cpp/src/spine/AnimationState.cpp` | `AnimationState::apply` | ✅ | unit tests + oracle scenarios |
| Timeline property gating (`computeHold`) | `AnimationState::computeHold` | `spine2d/src/runtime/animation_state.rs` | ✅ | unit + oracle |

## Constraints

| Area | Upstream reference | spine2d reference | Status | Signal |
|---|---|---|---:|---|
| IK constraint solve | `spine-cpp/src/spine/IkConstraint.cpp` | `spine2d/src/runtime/ik_constraint.rs` | ✅ | unit + oracle |
| Transform constraint solve | `spine-cpp/src/spine/TransformConstraint.cpp` | `spine2d/src/runtime/transform_constraint.rs` | ✅ | unit + oracle |
| Path constraint solve | `spine-cpp/src/spine/PathConstraint.cpp` | `spine2d/src/runtime/skeleton.rs` (`apply_path_constraint` + `compute_path_world_positions`) | ✅ | unit + oracle |
| Physics constraint solve | `spine-cpp/src/spine/PhysicsConstraint.cpp` | `spine2d/src/runtime/skeleton.rs` (`apply_physics_constraint`) | ✅ | oracle (incl. jitter/long-run) |
| Slider constraint | `spine-cpp/src/spine/Slider*` | `spine2d/src/runtime/skeleton.rs` (`apply_slider_constraint`) | ✅ | oracle (diamond) |

---

## Renderer-agnostic output (`spine2d::render`)

| Area | Upstream reference | spine2d reference | Status | Signal |
|---|---|---|---:|---|
| Slot visibility gating (`bone.active`) | `spine-cpp` / `spine-webgl` | `spine2d/src/render.rs` | ✅ | unit tests + render oracle goldens |
| Tint chain (`skeleton * slot * attachment`) | `spine-cpp/src/spine/SkeletonRenderer.cpp` | `spine2d/src/render.rs` | ✅ | render goldens |
| Clipping state-machine | `spine-webgl/src/SkeletonRenderer.ts` | `spine2d/src/render.rs` | ✅ | unit tests + render oracle goldens |
| Mesh world vertices (weighted/unweighted + deform) | `spine-cpp/src/spine/VertexAttachment.cpp` | render + runtime vertices | ✅ | unit tests + render oracle goldens |

## Atlas mapping (UV/trim/rotate)

| Area | Upstream reference | spine2d reference | Status | Signal |
|---|---|---|---:|---|
| `.atlas` parsing | `spine-ts` atlas parsing (behaviour) | `spine2d/src/atlas.rs` | ✅ | unit + render goldens |
| Region UV mapping | `spine-cpp` / `spine-ts` | `spine2d::render` | ✅ | render goldens + web smoke |
| Mesh UV mapping + rotate degrees | `spine-ts` | `spine2d::render` | ✅ | render goldens |

---

## Notes / guardrails

- Physics JSON defaults must match `spine-cpp` (`inertia=0.5`, `damping=0.85`, `fps=60` → `step=1/60`).
- Avoid “best effort” behaviour: if upstream behaviour is quirky, we replicate it and lock it with a test.
- Prefer adding *one* high-signal oracle scenario per risky axis (mixing, constraints ordering, clipping, deform, linkedmesh, physics).
- When changing baseline commit, re-record goldens (`scripts/record_oracle_goldens.py`) before judging failures.

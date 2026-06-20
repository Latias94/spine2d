# Parity Checklist (Spine 4.3)

This is the living checklist for “100% parity” with the official Spine 4.3 runtimes.
Items are only considered *done* when we have a regression signal (unit tests and/or a C++ pose oracle comparison).

Baseline note:
- Current Spine 4.3 parity work targets upstream `spine-runtimes` tag `spine-flutter-4.3.4` pinned at commit `80dc680a4345ac09cdc5d4c1a77ec572a3f295d1`.
- Runtime-specific tags such as `spine-libgdx-4.3.2` remain auxiliary release markers; this project keeps the latest tag explicit so C++ oracle sources and imported assets stay reproducible.
- Historical `4.3-beta` parity work is tracked in `docs/parity-4.3-beta.md`.
- The scenario golden dumps under `spine2d/tests/golden/` must be re-recorded when the baseline commit changes; until then, `--features upstream-smoke` will report expected failures.
- For the currently pinned tag, `cargo nextest run -p spine2d --features json,binary,upstream-smoke` is expected to be green.

Legend:
- ✅ done (has tests/oracle coverage)
- 🟡 partial (works for common cases; gaps remain)
- ⛔ missing (not implemented)

## Pose / Math

- ✅ Bone local timelines: `rotate`, `translate`, `scale`, `shear`
- ✅ Rotate mixing uses a `spine-cpp`-style per-entry accumulator (cross detection) + shortest-path normalization (has unit test coverage + oracle snapshots)
- ✅ Rotate timeline “pure apply” (alpha==1) matches `spine-cpp`’s relative value semantics (no extra wrap/shortest-path adjustments)
- ✅ Curves: `linear`, `stepped`, `bezier` in **(time,value)** space (multi-value timelines have per-value curves)
- ✅ Bone inherit modes (`normal`, `onlyTranslation`, `noRotationOrReflection`, `noScale`, `noScaleOrReflection`)
- ✅ Constraint ordering (`order`) across IK/transform/path (recomputes world transforms after each constraint)

Oracle status:
- ✅ `spineboy-pro.json` `run` pose matches the official C++ runtime (oracle) (eps=1e-3) at multiple sampled times.
- ✅ `spineboy-pro.json` `run` has baked regression snapshots against the official C++ runtime (oracle) (feature `upstream-smoke`)
- ✅ Scenario oracle snapshots (feature `upstream-smoke`):
  - `spineboy_run_plus_aim_add_t0_2.json` (multi-track + `MixBlend::Add`)
  - `spineboy_run_plus_aim_add_alpha0_5_t0_2.json` (multi-track + `MixBlend::Add` + entry `alpha=0.5`)
  - `spineboy_run_plus_portal_add_to_empty_mix0_2_t0_6.json` (shear/scale/translate under `MixBlend::Add` + mix-out to empty)
  - `spineboy_run_plus_portal_add_to_empty_mix0_2_jitter_dt_t0_6.json` (same scenario; mixed dt)
  - `spineboy_run_plus_portal_add_reverse_t0_35.json` (multi-track `MixBlend::Add` + `reverse=true` on the Add entry)
  - `spineboy_run_plus_portal_add_reverse_to_empty_immediate_mix0_2_t0_1.json` (reverse Add entry is mixed out before first apply; locks “unapplied mixingFrom” under `reverse=true`)
  - `spineboy_portal_reverse_t0_5.json` (single-track `reverse=true` sampling semantics)
  - `spineboy_portal_alpha0_5_shortestRotation_true_t2_0.json` (single-track `alpha=0.5` + `shortestRotation=true` rotation accumulator semantics)
  - `spineboy_portal_reverse_to_shoot_mix0_2_t0_1.json` (portal -> shoot transition; locks `reverse=true` behaviour during `applyMixingFrom`)
  - `spineboy_portal_shortestRotation_true_to_shoot_mix0_2_t0_1.json` (portal -> shoot transition; locks `shortestRotation=true` during `applyMixingFrom`)
  - `spineboy_portal_alpha0_5_reset_rotation_directions_t0_4.json` (rotation accumulator reset via `resetRotationDirections`)
  - `spineboy_portal_add_reverse_to_shoot_replace_mix0_2_t0_1.json` (from(Add+reverse) -> to(Replace) behaviour during `applyMixingFrom`)
  - `spineboy_run_to_portal_reverse_mix0_2_t0_1.json` (single-track transition with `reverse=true` on the target entry)
  - `spineboy_run_to_portal_mix0_2_shortestRotation_true_t0_1.json` (single-track transition with `shortestRotation=true` on the target entry)
	  - `spineboy_aim_to_shoot_to_portal_holdMix_t0_2.json` (locks `TimelineMode::HoldMix` alpha scaling semantics over a 3-entry chain)
  - `spineboy_aim_to_shoot_interrupt_to_portal_mix0_2_t0_2.json` (interrupt during mix; locks `interruptAlpha` carry-over)
  - `spineboy_run_t1_aim_add_alpha0_5_t2_shoot_replace_alpha0_5_t0_3.json` (3-track stack; locks cross-track property gating (`Subsequent` vs `First`) when track1 is `Add` and track2 is `Replace`)
  - `spineboy_run_t1_aim_add_alpha0_5_t2_shoot_add_alpha0_5_t0_3.json` (3-track stack; locks Add+Add overlay accumulation with per-entry `alpha=0.5`)
  - `spineboy_run_t1_aim_add_alpha0_5_t2_aim_to_shoot_mix0_2_mixAttachmentThreshold_0_mixDrawOrderThreshold_0_interrupt_to_portal_t0_3.json` (3-track + interrupted mix + thresholds; locks `interruptAlpha` and attachment/drawOrder gating under cross-track overlay)
  - `spineboy_run_t1_aim_add_alpha0_5_t2_aim_to_shoot_mix0_2_mixAttachmentThreshold_1_mixDrawOrderThreshold_1_interrupt_to_portal_t0_3.json` (same scenario with thresholds=1; locks “always apply attachments/drawOrder during mixingFrom”)
  - `spineboy_run_t1_shoot_add_alpha0_5_t2_aim_to_shoot_mix0_2_mixAttachmentThreshold_0_mixDrawOrderThreshold_0_interrupt_to_portal_t0_3.json` (variant with track1=`shoot(Add)`; increases sensitivity for slot RGBA/attachments under cross-track overlay)
  - `spineboy_run_t1_shoot_add_alpha0_5_t2_aim_to_shoot_mix0_2_mixAttachmentThreshold_1_mixDrawOrderThreshold_1_interrupt_to_portal_t0_3.json` (same variant with thresholds=1)
  - `spineboy_run_plus_shoot_add_to_empty_mix0_2_t0_6.json` (slot RGBA under `MixBlend::Add` + mix-out to empty)
  - `spineboy_run_plus_shoot_add_to_empty_mix0_2_jitter_dt_t0_6.json` (same scenario; mixed dt)
  - `spineboy_run_plus_shoot_add_alpha0_5_t0_4.json` (entry `alpha=0.5` under `MixBlend::Add`, locks RGBA/attachment scaling)
  - `spineboy_run_plus_shoot_add_alpha0_5_jitter_dt_t0_4.json` (same scenario; mixed dt)
  - `spineboy_aim_to_shoot_add_t0_4.json` (`MixBlend::Add` mixingFrom/out branch; locks attachment + slot color parity)
	  - `spineboy_aim_add_to_shoot_replace_t0_4.json` (from Add -> to Replace; locks `applyMixingFrom` blend selection)
	  - `spineboy_aim_replace_to_shoot_add_t0_4.json` (from Replace -> to Add; locks `applyMixingFrom` non-Add path + to(Add) apply)
	  - `spineboy_run_plus_aim_add_to_empty_immediate_mix0_2_t0_1.json` (track1 Add is mixed out before first apply; locks “unapplied mixingFrom” for IK + transform constraint timelines in a real asset)
	  - `spineboy_shoot_alphaAttachmentThreshold_0_6_alpha_0_5_t0_1.json` (entry `alphaAttachmentThreshold` gates attachment application)
  - `spineboy_shoot_to_empty_mixAttachmentThreshold_0_mixDrawOrderThreshold_0_t0_2.json` (entry `mixAttachmentThreshold`/`mixDrawOrderThreshold` edge case, locks unkeyed/threshold behaviour)
  - `spineboy_ess_run_to_empty_immediate_mix0_2_mixAttachmentThreshold_1_mixDrawOrderThreshold_1_t0_1.json` (entry was never applied before mixing out; `mixAttachmentThreshold=1` + `mixDrawOrderThreshold=1` locks attachment/drawOrder gating during “unapplied mixingFrom”)
  - `spineboy_run_to_walk_mix0_2_t0_4.json` (single-track transition, mix=0.2)
  - `spineboy_run_to_walk_mix0_2_t0_55.json` (single-track transition, mix=0.2)
  - `spineboy_run_to_walk_headless_t0_25.json` / `spineboy_run_to_walk_headless_t0_333333.json` / `spineboy_run_to_walk_headless_t0_416667.json` / `spineboy_run_to_walk_headless_t0_5.json` (official headless `run -> walk` transition sampling after the switch; JSON + `.skel` versions)
  - `alien_run_plus_death_add_to_empty_immediate_mix0_2_t0_1.json` (track1 Add is mixed out before first apply; locks shear timeline (`bones.eye.shear`) + drawOrder/event timelines under “unapplied mixingFrom”)
  - `diamond_idle_rotating_plus_rotation_add_t0_5.json` (multi-track + `MixBlend::Add`; locks attachment sequence timelines under Add)
  - `diamond_idle_rotating_plus_idle_still_add_t0_5.json` (multi-track + `MixBlend::Add`; locks slider timelines + attachment sequence timelines under Add)
  - `diamond_idle_rotating_plus_idle_still_add_to_empty_mix0_2_t0_1.json` (track1 Add mix-out to empty; locks `MixDirection::Out` for slider + sequence timelines)
  - `diamond_size_changing_rotation_t3_0.json` (diamond example's size-changing-rotation animation, with slider constraint coverage)
  - `tank_shoot_clipping_deform_t0_3.json` (clipping + deform + tint)
  - `tank_drive_plus_shoot_add_t0_4.json` (multi-track + `MixBlend::Add` + path constraint + clipping/deform)
  - `tank_drive_plus_shoot_add_smoke_glow_deform_t0_25.json` (`smoke-glow` deform vertices under `MixBlend::Add`, locks DeformTimeline sampling)
  - `tank_drive_plus_shoot_add_smoke_glow_deform_jitter_dt_t0_25.json` (same scenario; mixed dt)
  - `tank_drive_plus_shoot_add_alpha0_5_t0_4.json` (multi-track + `MixBlend::Add` + entry `alpha=0.5`, locks two-color tint scaling)
  - `tank_drive_plus_shoot_add_alpha0_5_jitter_dt_t0_4.json` (same scenario; mixed dt)
  - `tank_drive_plus_shoot_add_to_empty_t0_35.json` (multi-track `MixBlend::Add` mix-out to empty; locks mixingFrom/out semantics for attachments/drawOrder/clipping/constraints)
  - `tank_drive_plus_shoot_add_to_empty_smoke_glow_deform_t0_35.json` (`smoke-glow` deform vertices during mix-out to empty, locks deform blend-out)
  - `tank_drive_plus_shoot_add_to_empty_smoke_glow_deform_jitter_dt_t0_35.json` (same scenario; mixed dt)
  - `tank_drive_plus_shoot_add_to_empty_immediate_mix0_2_t0_1.json` (track1 Add is mixed out before first apply; locks “unapplied mixingFrom” semantics across slot tint + deform)
  - `tank_shoot_plus_drive_add_to_empty_immediate_mix0_2_t0_1.json` (track1 Add is mixed out before first apply; locks “unapplied mixingFrom” semantics for path constraint timelines under Add)
  - `tank_drive_plus_shoot_add_alpha0_5_to_empty_t0_35.json` (same scenario with entry `alpha=0.5`, locks mix-out tint scaling)
  - `tank_drive_plus_shoot_add_alpha0_5_to_empty_jitter_dt_t0_35.json` (same scenario; mixed dt)
  - `tank_drive_plus_shoot_add_to_empty_mixDrawOrderThreshold_1_t0_55.json` (`mixDrawOrderThreshold=1` under `MixBlend::Add` + mix-out; locks drawOrder mixingFrom behaviour)
  - `tank_drive_plus_shoot_add_to_empty_mixDrawOrderThreshold_1_jitter_dt_t0_55.json` (same scenario; mixed dt)
  - `tank_drive_plus_shoot_add_to_empty_mixDrawOrderThreshold_0_t0_55.json` (`mixDrawOrderThreshold=0` under `MixBlend::Add` + mix-out; drawOrder should NOT apply during mixingFrom)
  - `tank_drive_plus_shoot_add_to_empty_mixDrawOrderThreshold_0_jitter_dt_t0_55.json` (same scenario; mixed dt)
  - `tank_shoot_to_shoot_mixDrawOrderThreshold_0_t0_4.json` (`mixDrawOrderThreshold=0` edge case; drawOrder should not be applied during mixingFrom)
  - `tank_shoot_to_shoot_mixDrawOrderThreshold_1_t0_4.json` (`mixDrawOrderThreshold=1` forces drawOrder to apply during mixingFrom)
  - `tank_shoot_to_shoot_to_drive_holdMix_smoke_glow_t0_2.json` (locks deform `HoldMix` semantics in a 3-entry chain)
  - `tank_drive_t1_shoot_add_alpha0_5_t2_shoot_replace_alpha0_5_t0_3_smoke_glow.json` (3-track stack; locks Add+Replace overlay interaction for deform world vertices)
  - `tank_drive_t2_shoot_add_alpha0_5_t1_shoot_to_shoot_mixDrawOrderThreshold_0_smoke_glow_t0_4.json` (3-track stack + mixingFrom; locks drawOrder threshold=0 under cross-track Add overlay, also verifies deform vertices)
  - `tank_drive_t2_shoot_add_alpha0_5_t1_shoot_to_shoot_mixDrawOrderThreshold_1_smoke_glow_t0_4.json` (same scenario with drawOrder threshold=1)
  - `tank_drive_t2_shoot_add_alpha0_5_t1_shoot_to_shoot_mixAttachmentThreshold_0_smoke_glow_t0_4.json` (same 3-track stack; locks attachment threshold=0 under cross-track Add overlay)
  - `tank_drive_t2_shoot_add_alpha0_5_t1_shoot_to_shoot_mixAttachmentThreshold_1_smoke_glow_t0_4.json` (same scenario with attachment threshold=1)
  - `goblins_walk_dagger_t0_3.json` (attachment switching during animation)
  - `goblins_walk_skin_goblin_left_foot_deform_t0_3.json` (mesh deform sampling)
  - `goblins_walk_skin_goblin_left_foot_deform_jitter_dt_t0_3.json` (same scenario; mixed dt)
  - `goblins_walk_skin_goblingirl_left_foot_deform_t0_3.json` (linkedmesh inherits parent mesh deform via `timelineAttachment`)
  - `goblins_walk_skin_goblingirl_left_foot_deform_jitter_dt_t0_3.json` (same scenario; mixed dt)
  - `hero_idle_head_deform_t0_55.json` (weighted mesh deform sampling)
  - `hero_idle_head_deform_jitter_dt_t0_55.json` (same scenario; mixed dt)
  - `hero_idle_plus_run_add_head_deform_t0_55.json` (weighted mesh deform under `MixBlend::Add`)
  - `hero_idle_plus_run_add_head_deform_jitter_dt_t0_55.json` (same scenario; mixed dt)
  - `hero_idle_plus_run_add_to_empty_mix0_2_head_deform_t0_55.json` (weighted mesh deform during mix-out to empty under `MixBlend::Add`)
  - `hero_idle_plus_run_add_to_empty_mix0_2_head_deform_jitter_dt_t0_55.json` (same scenario; mixed dt)
  - `owl_up_head_base_deform_t0_55.json` (weighted mesh deform; multiple deform timelines in a single animation)
  - `owl_up_head_base_deform_jitter_dt_t0_55.json` (same scenario; mixed dt)
  - `owl_up_plus_left_add_head_base_deform_t0_55.json` (weighted mesh deform under multi-track `MixBlend::Add`)
  - `owl_up_plus_left_add_head_base_deform_jitter_dt_t0_55.json` (same scenario; mixed dt)
  - `owl_up_plus_left_add_to_empty_mix0_2_head_base_deform_t0_55.json` (weighted mesh deform during mix-out to empty under `MixBlend::Add`)
  - `owl_up_plus_left_add_to_empty_mix0_2_head_base_deform_jitter_dt_t0_55.json` (same scenario; mixed dt)
  - `owl_up_l_wing_deform_t0_55.json` (small weighted mesh deform target)
  - `owl_up_l_wing_deform_jitter_dt_t0_55.json` (same scenario; mixed dt)
  - `owl_up_plus_left_add_l_wing_deform_t0_55.json` (small weighted mesh deform under multi-track `MixBlend::Add`)
  - `owl_up_plus_left_add_l_wing_deform_jitter_dt_t0_55.json` (same scenario; mixed dt)
  - `owl_up_plus_left_add_to_empty_mix0_2_l_wing_deform_t0_55.json` (small weighted mesh deform during mix-out to empty under `MixBlend::Add`)
  - `owl_up_plus_left_add_to_empty_mix0_2_l_wing_deform_jitter_dt_t0_55.json` (same scenario; mixed dt)
  - `owl_up_plus_blink_l_wing_t0_5.json` (attachment timeline switching on track 1, while track 0 continues driving deform)
  - `owl_up_plus_blink_l_wing_jitter_dt_t0_5.json` (same scenario; mixed dt)
  - `owl_up_plus_blink_to_empty_mix0_2_l_wing_t0_55.json` (attachment switching + mix-out to empty on track 1)
  - `owl_up_plus_blink_to_empty_mix0_2_l_wing_jitter_dt_t0_55.json` (same scenario; mixed dt)
  - `owl_up_r_wing_deform_t0_55.json` (small weighted mesh deform target)
  - `owl_up_r_wing_deform_jitter_dt_t0_55.json` (same scenario; mixed dt)
  - `owl_up_plus_left_add_r_wing_deform_t0_55.json` (small weighted mesh deform under multi-track `MixBlend::Add`)
  - `owl_up_plus_left_add_r_wing_deform_jitter_dt_t0_55.json` (same scenario; mixed dt)
  - `owl_up_plus_left_add_to_empty_mix0_2_r_wing_deform_t0_55.json` (small weighted mesh deform during mix-out to empty under `MixBlend::Add`)
  - `owl_up_plus_left_add_to_empty_mix0_2_r_wing_deform_jitter_dt_t0_55.json` (same scenario; mixed dt)
  - `mix_and_match_skin_switch_boy_to_girl.json` (skin switching)
  - `mix_and_match_skin_switch_backpack_to_hat.json` (skin switching + attachment source)
  - `mix_and_match_skin_switch_hat_aware_t0_1667.json` (skin switching + timeline sampling)
  - `chibi_stickers_movement__trot-front_t0_3.json` (chibi-stickers `movement/trot-front`; locks bones/slots/drawOrder + IK parity)
  - `chibi_stickers_emotes__excited_t0_35.json` (chibi-stickers `emotes/excited`; locks bones/slots/IK/drawOrder parity)
  - `chibi_stickers_emotes__dramatic-stare_t0_5.json` (chibi-stickers `emotes/dramatic-stare`; locks bones/slots/IK/drawOrder parity)
  - `chibi_stickers_interactive__password__hooray_t0_25.json` (chibi-stickers `interactive/password/hooray`; locks bones/slots/IK parity)
  - `vine_grow_t0_5.json` (official vine example; locks path constraint `chainScale` pose under animated control bones)
  - `raptor_walk_t0_5.json` (official raptor example; locks complex IK pose, including soft IK, during `walk`)
  - `dragon_flying_sequence_t0_25.json` (sequence timeline / frame index parity, early segment)
  - `dragon_flying_sequence_t0_65.json` (sequence timeline / frame index parity, post-0.6 key)
  - `dragon_flying_sequence_t0_76.json` (sequence timeline / frame index parity, index=1 key)
  - `dragon_flying_sequence_t0_85.json` (sequence timeline / frame index parity, index=2+delay change)
  - `dragon_flying_sequence_t0_98.json` (sequence timeline / frame index parity, index=7 key)
  - `dragon_flying_to_empty_t0_35.json` (sequence timeline mix-out reset parity)
  - `coin_animation_t0_3.json` (`SlotRgba2` + attachment + drawOrder parity; locks two-color tint and drawOrder sampling on the `animation` track)
  - `windmill_animation_t3_0.json` (shear-heavy pose parity on a long-running loop sample)
  - `powerup_bounce_t0_7.json` (slot RGBA + token deform parity on the `bounce` track)
  - `speedy_run_t0_433333.json` (dense curve-sampling parity on the `run` track)
  - `6_arcs_arcs_t5_666667.json` (arc-tracker attachment + late-loop pose parity on the `arcs` track)
  - `stretchyman_sneak_t1_366667.json` (IK + path attachment parity on the `sneak` track)
  - `8_follow_through_ball_follow_through_t2_4.json` (transform-constraint pose parity on the `follow-through` track)
  - `food_app_search_add_bread_t1_4.json` (drawOrder + slot/bone pose parity on the `add-bread` track)
  - `food_app_search_add_carrot_t0_2.json` (drawOrder + slot/bone pose parity on the `add-carrot` track)
- Note: the C++ oracle prints floats with `max_digits10` to avoid precision artifacts in diffs.

## AnimationState / Mixing

- ✅ Time update semantics + event queue baseline (ported tests exist in `spine2d`)
- ✅ Event payload parity: `EventData`/`Event` include `int/float/string/audioPath/volume/balance`; JSON defaults match upstream and binary `stringValue==null → EventData.stringValue` fallback is covered by a dedicated unit test (broader binary event coverage can be added as we hit real deltas).
- ✅ Upstream test suites ported:
  - `spine-csharp` `AnimationStateTests`
  - `spine-libgdx` `AnimationStateTests` (including `#28`/`#29`)
  - `spine-libgdx` `EventTimelineTests`
  - `spine-libgdx` `AttachmentTimelineTests`
- ✅ `add_empty_animation` (delay adjustment matches upstream: empty mix ends with previous entry end) (has unit test coverage)
- ✅ Per-timeline property gating (`computeHold`): `first/subsequent/holdFirst/holdSubsequent/holdMix` (has unit tests)
- ✅ Per-entry `mixBlend` (Replace/Add) plumbed through `AnimationState::apply` (has unit tests)
- ✅ Attachment and drawOrder mix thresholds (`mixAttachmentThreshold`, `mixDrawOrderThreshold`, `alphaAttachmentThreshold`) + unkeyed attachment state tracking (has unit tests)
- ✅ “property id” mapping matches `spine-cpp` (including Deform/Sequence `slotIndex<<16 | id` packing)
- ✅ Constraint pose timelines apply even when the constraint is not in the update cache (`PosedActive` vs `Constraint::_active` semantics; locked by an oracle scenario)
- ✅ Rotation accumulator parity detail: fixes `sign(0)=0` edge so first-frame rotation mixing doesn't spuriously add 360° (locked by upstream scenario pose oracle)
- ✅ TrackEntry flags: `reverse` (pose sampling uses `duration - animationTime`, events disabled) and `shortestRotation` (disables the rotation accumulator; uses shortest-path each frame) (has unit test coverage)
- ✅ Draw order folder timelines: `DrawOrderFolderTimeline` applies after the plain draw order timeline and is locked by unit coverage
- ✅ `set_animation` mixes from an entry that was never applied yet (except the special-case “same animation set twice”; matches spine-cpp `_nextTrackLast == -1` logic) (locked by `diamond_idle_rotating_plus_idle_still_add_to_empty_mix0_2_t0_1` + `spineboy_ess_run_to_empty_immediate_mix0_2_mixAttachmentThreshold_1_mixDrawOrderThreshold_1_t0_1`)
- 🟡 Still incomplete:
  - Remaining upstream edge-case tests from official runtimes (mostly rare multi-track overlay combinations)

## Performance parity (non-semantic)

- ✅ Path constraint solver allocations: per-constraint scratch buffers are reused for `spaces/lengths/positions/world/curves` (semantics unchanged) and the solver no longer clones `PathConstraint` each update.

## Constraints

- ✅ IK constraint solve (1-bone and 2-bone) + timeline driving `mix/softness/bendPositive` (including `bendPositive` defaulting to `true` when omitted)
- ✅ Transform constraint solve + timeline driving `mix*`
- ✅ Path constraint solve + timelines (`position`, `spacing`, `mix`)
  - ✅ Oracle: `tank-pro.json` `drive` tread chain at `t=0.3` (eps=1e-3)
  - ✅ Oracle: `mix-and-match-pro.json` + skin `accessories/backpack` + `aware` at `t=0.1667` (eps=1e-3)
- ✅ Physics constraints (Spine 4.3): implemented + sentinel oracle coverage
  - ✅ JSON/Binary parsing of physics constraints + timelines
  - ✅ Runtime update/apply (`Skeleton::update_world_transform_with_physics`)
  - ✅ Oracle snapshots (includes `physicsConstraints` internal state):
    - `cloud_pot_playing_in_the_rain_physics_t0_5.json` (30×`dt=1/60`)
    - `cloud_pot_playing_in_the_rain_physics_t1_0.json` (60×`dt=1/60`, exercises reset timelines over longer duration)
    - `cloud_pot_playing_in_the_rain_physics_t10_0.json` (600×`dt=1/60`, long-run drift coverage)
    - `cloud_pot_playing_in_the_rain_physics_update_to_pose_t1_0.json` (Update→Pose mode switch at `t=0.5`)
    - `cloud_pot_playing_in_the_rain_physics_update_reset_update_t1_0.json` (Update→Reset→Update at `t=0.5`)
    - `cloud_pot_playing_in_the_rain_physics_jitter_dt_t1_0.json` (mixed dt sequence; remaining/step edge coverage)
    - `cloud_pot_playing_in_the_rain_physics_jitter_dt_t10_0.json` (10s long-run, mixed dt sequence)
    - `sack_walk_physics_t0_5.json` (30×`dt=1/60`)
    - `sack_walk_physics_jitter_dt_t1_0.json` (mixed dt sequence)
    - `sack_walk_physics_update_pose_update_t1_0.json` (Update→Pose→Update roundtrip)
    - `sack_walk_to_hello_mix0_2_physics_t0_4.json` (AnimationState mix under physics)
    - `sack_walk_plus_hello_add_physics_t0_4.json` (multi-track overlay, `MixBlend::Add`, under physics)
    - `sack_walk_plus_hello_add_to_empty_mix0_2_physics_t0_6.json` (track1 Add mix-out to empty under physics)
    - `celestial_circus_wind_idle_physics_t0_5.json` (30×`dt=1/60`)
    - `snowglobe_idle_physics_t0_5.json` (30×`dt=1/60`)
    - `snowglobe_idle_physics_jitter_dt_t1_0.json` (mixed dt sequence)
    - `snowglobe_idle_physics_jitter_dt_t10_0.json` (10s long-run, mixed dt sequence)
    - `snowglobe_idle_physics_update_pose_update_t1_0.json` (Update→Pose→Update roundtrip)
    - `snowglobe_idle_physics_update_reset_update_t1_0.json` (Update→Reset→Update roundtrip)
    - `snowglobe_idle_physics_t10_0.json` (10s long-run, 600×`dt=1/60`)
    - `snowglobe_idle_plus_shake_add_to_empty_mix0_2_physics_t0_6.json` (track1 Add mix-out to empty under physics)
    - `snowglobe_idle_plus_shake_add_to_empty_mix0_2_physics_jitter_dt_t0_6.json` (track1 Add mix-out to empty under physics; mixed dt)
  - Note: we still add targeted oracle cases when bugs/behavioural deltas are discovered.
- ✅ Slider constraint (Spine 4.3): implemented + oracle-locked (diamond scenarios)

## Data Formats

- ✅ JSON loader for common fields used by official examples (incremental coverage)
- ✅ Binary `.skel` loader:
  - Can parse official Spine example exports and run animations (feature `binary`).
  - Has oracle-locked parity coverage via the C++ oracle (feature `upstream-smoke`):
    - `spineboy-pro.skel`:
      - `run + aim(add)` at `t=0.2`
      - `run -> walk` (mix=0.2) at `t=0.4` / `t=0.55`
    - `tank-pro.skel`:
      - `shoot` at `t=0.3` (clipping + deform world vertices)
      - `drive + shoot(add)` at `t=0.4`
      - `shoot -> shoot` (mix=0.2) + `mixDrawOrderThreshold=0/1` at `t=0.4`
    - `dragon-ess.skel`:
      - `flying` sequence timeline at `t=0.25/0.65/0.76/0.85/0.98`
      - `flying -> empty` (mix=0.2) at `t=0.35`
    - `mix-and-match-pro.skel`:
      - skin switch `full-skins/boy -> full-skins/girl`
      - skin switch `accessories/backpack -> accessories/hat-red-yellow`
      - skin switch + `aware` at `t=0.1667`
      - `dance` at `t=0.25` (dense mixed-bone / transform-constraint pose parity)
    - `coin-pro.skel`:
      - `animation` at `t=0.3` (`SlotRgba2` + attachment + drawOrder parity)
    - `windmill-ess.skel`:
      - `animation` at `t=3.0` (shear-heavy pose parity on a long-running loop sample)
    - `powerup-pro.skel`:
      - `bounce` at `t=0.7` (slot RGBA + token deform parity)
    - `speedy-ess.skel`:
      - `run` at `t=0.43333334` (dense curve-sampling parity)
    - `goblins-pro.skel`:
      - skin `goblin` + `walk` at `t=0.3` (`right-hand-item` world vertices)
      - skin `goblin/goblingirl` + `walk` at `t=0.3` (`left-foot` world vertices, includes mixed dt)
    - `hero-pro.skel`:
      - `idle` at `t=0.55` (`head` world vertices, weighted mesh deform)
      - `idle(track0) + run(track1, MixBlend::Add)` at `t=0.55` (`head` world vertices, includes mixed dt)
      - `idle(track0) + run(Add) -> empty(mix=0.2)` at `t=0.55` (`head` world vertices, includes mixed dt)
    - `owl-pro.skel`:
      - `up` at `t=0.55` (`head-base` world vertices, weighted mesh deform, includes mixed dt)
      - `up(track0) + left(track1, MixBlend::Add)` at `t=0.55` (`head-base` world vertices, includes mixed dt)
      - `up(track0) + left(Add) -> empty(mix=0.2)` at `t=0.55` (`head-base` world vertices, includes mixed dt)
  - ✅ Physics constraints `.skel` oracle snapshots:
    - `cloud_pot_playing_in_the_rain_physics_t0_5.json` (30×`dt=1/60`)
    - `cloud_pot_playing_in_the_rain_physics_t1_0.json` (60×`dt=1/60`)
    - `cloud_pot_playing_in_the_rain_physics_t10_0.json` (600×`dt=1/60`)
    - `cloud_pot_playing_in_the_rain_physics_update_to_pose_t1_0.json` (Update→Pose mode switch at `t=0.5`)
    - `cloud_pot_playing_in_the_rain_physics_update_reset_update_t1_0.json` (Update→Reset→Update at `t=0.5`)
    - `cloud_pot_playing_in_the_rain_physics_jitter_dt_t1_0.json` (mixed dt sequence)
    - `cloud_pot_playing_in_the_rain_physics_jitter_dt_t10_0.json` (10s long-run, mixed dt sequence)
    - `sack_walk_physics_t0_5.json` (30×`dt=1/60`)
    - `sack_walk_physics_jitter_dt_t1_0.json` (mixed dt sequence)
    - `sack_walk_physics_update_pose_update_t1_0.json` (Update→Pose→Update roundtrip)
    - `sack_walk_to_hello_mix0_2_physics_t0_4.json` (AnimationState mix under physics)
    - `sack_walk_plus_hello_add_physics_t0_4.json` (multi-track overlay, `MixBlend::Add`)
    - `sack_walk_plus_hello_add_to_empty_mix0_2_physics_t0_6.json` (track1 Add mix-out to empty)
    - `celestial_circus_wind_idle_physics_t0_5.json` (30×`dt=1/60`)
    - `snowglobe_idle_physics_t0_5.json` (30×`dt=1/60`)
    - `snowglobe_idle_physics_jitter_dt_t1_0.json` (mixed dt sequence)
    - `snowglobe_idle_physics_jitter_dt_t10_0.json` (10s long-run, mixed dt sequence)
    - `snowglobe_idle_physics_update_pose_update_t1_0.json` (Update→Pose→Update roundtrip)
    - `snowglobe_idle_physics_update_reset_update_t1_0.json` (Update→Reset→Update roundtrip)
    - `snowglobe_idle_physics_t10_0.json` (10s long-run, 600×`dt=1/60`)
    - `snowglobe_idle_plus_shake_add_to_empty_mix0_2_physics_t0_6.json` (track1 Add mix-out to empty)
    - `snowglobe_idle_plus_shake_add_to_empty_mix0_2_physics_jitter_dt_t0_6.json` (track1 Add mix-out to empty; mixed dt)

## Skins / Attachments

- ✅ Skins: basic lookup + `default` fallback
- ✅ Skin switching semantics + “active”:
  - ✅ JSON: parses `skinRequired` on bones/IK/transform/path constraints, plus per-skin `bones/ik/transform/path` lists (Spine 4.3 `skins: [...]` format)
  - ✅ Runtime: `Skeleton::update_cache` computes bone/constraint `active` (and respects per-skin membership when `skinRequired=true`)
  - ✅ Timeline apply gates by `active` (inactive bones/slots/constraints are not mutated)
- ✅ `Skeleton` starts with no skin (matches upstream); switching from no-skin → skin applies setup attachment names from the new skin
- ✅ Skin→skin `attachAll` semantics (tracks attachment source skin; oracle-locked by mix-and-match scenarios)
- ✅ Skin attachment order is preserved during iteration and merge (`Skin::addSkin` / `attachAll` use insertion order, matching upstream `OrderedSet` behavior)

Attachment types:
- ✅ `region`
- ✅ `mesh` (weighted/unweighted)
- ✅ `linkedmesh` resolution
- ✅ Deform timelines match `timelineAttachment` (linkedmesh inherits parent mesh deform, oracle-locked by `goblins_walk_skin_goblingirl_left_foot_deform_*.json`)
- ✅ Slot attachment switching clears deform only when `timelineAttachment` changes (matches spine-cpp `Slot::setAttachment`)
- ✅ Attachment sequences: parses `sequence` on `region`/`mesh`, applies `SequenceTimeline`, and resolves the effective render path using `Slot.sequence_index`
- ✅ `path`
- ✅ `boundingbox` (parsed; not rendered)
- ✅ `clipping` (parsed + applied during render output)
- ✅ `point` attachment (parsed; provides world position/rotation helpers)

## Slot Timelines / Colors

- ✅ Slot `attachment`
- ✅ Slot `color` (RGBA) (supports upstream JSON keys `color` and `rgba`)
- ✅ Two-color tint (`dark` setup color + `rgba2`/`rgb2` timelines) (has oracle-locked coverage for `tank-pro.json` `shoot` at `t=0.3`)

## Rendering Output (renderer-agnostic)

- ✅ `DrawList` generation for region/mesh + atlas UV mapping
- ✅ Render oracle for geometry/UV/light+dark color parity (triangle stream compare vs the official C++ runtime `SkeletonRenderer`):
  - Note: the C++ oracle applies the same PMA + two-color tint packing conventions used by `spine-ts/spine-webgl` so we can compare packed `colors`/`dark_colors` directly.
  - `spineboy-pro.json` `run` @ `t=0.2`
  - `alien-pro.json` `run` @ `t=0.3`
  - `dragon-ess.json` `flying` @ `t=0.25` (multi-page atlas)
  - `hero-pro.json` `idle` @ `t=0.55` (weighted mesh)
  - `coin-pro.json` `animation` @ `t=0.3` (clipping end slot = self)
  - `tank-pro.json` `shoot` @ `t=0.3` (clipping + deform + tint, typical eps-pos `1e-3`)
- ✅ Atlas `.atlas` parsing supports multiple pages (region → page index is correct)
- ✅ Clipping applied to geometry (including `end` slot semantics and “inactive clip slot does not start clipping”)
- ✅ Triangulation + clipping geometry helpers match the upstream `spine-c` unit tests (`Triangulator`, `SkeletonClipping::clipTriangles` ported as pure Rust tests)
- ✅ Blend modes + PMA tagging
- ✅ Smoke runner: `scripts/render_parity_smoke.zsh` (runs a small set of render oracle comparisons)
- ✅ Exact draw batching parity vs official renderers (batch merge rules are locked by unit tests)
- 🟡 Pixel parity is not yet locked (filtering/bleeding/gamma are renderer-dependent and need a separate oracle)

## WASM

- ✅ Core crate is IO-free and compiles for `wasm32-unknown-unknown` (`cargo check -p spine2d --target wasm32-unknown-unknown --features json,binary`)
- ✅ Web demo crate exists (`spine2d-web` via Trunk) and compiles for `wasm32-unknown-unknown`

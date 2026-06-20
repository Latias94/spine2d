mod animation;
mod animation_state;
mod skeleton;

pub use animation::*;
pub use animation_state::*;
pub use skeleton::*;

pub(crate) fn finalize_animation(mut animation: crate::Animation) -> crate::Animation {
    if animation.timeline_order.is_empty() {
        animation.timeline_order = crate::model::timeline_order_for_animation(&animation);
    }
    animation
}

#[cfg(all(test, feature = "json"))]
mod animation_state_tests;

#[cfg(test)]
mod upstream_event_timeline_tests;

#[cfg(test)]
mod upstream_attachment_timeline_tests;

#[cfg(test)]
mod skeleton_tests;

#[cfg(test)]
mod animation_tests;

#[cfg(test)]
mod pose_integration_tests;

#[cfg(all(test, feature = "json"))]
mod animation_state_mixing_semantics_tests;

#[cfg(all(test, feature = "json"))]
mod slots_tests;

#[cfg(all(test, feature = "json"))]
mod deform_tests;

#[cfg(all(test, feature = "json"))]
mod slot_timeline_tests;

#[cfg(all(test, feature = "json"))]
mod color_timeline_tests;

#[cfg(all(test, feature = "json"))]
mod point_attachment_tests;

#[cfg(all(test, feature = "json"))]
mod sequence_timeline_tests;

#[cfg(all(test, feature = "json"))]
mod ik_tests;

#[cfg(all(test, feature = "json"))]
mod ik_timeline_tests;

#[cfg(all(test, feature = "json"))]
mod transform_constraint_tests;

#[cfg(all(test, feature = "json"))]
mod transform_constraint_timeline_tests;

#[cfg(all(test, feature = "json"))]
mod path_constraint_timeline_tests;

#[cfg(all(test, feature = "json"))]
mod path_constraint_solve_tests;

#[cfg(all(test, feature = "json", feature = "upstream-smoke"))]
mod vine_smoke_tests;

#[cfg(all(test, feature = "json", feature = "upstream-smoke"))]
mod examples_smoke_tests;

#[cfg(all(test, feature = "json", feature = "upstream-smoke"))]
mod examples_pose_parity_tests;

#[cfg(all(test, feature = "json", feature = "upstream-smoke"))]
mod skin_active_semantics_tests;

#[cfg(all(test, feature = "json", feature = "upstream-smoke"))]
mod oracle_scenario_parity_tests;

#[cfg(all(test, feature = "json", feature = "upstream-smoke"))]
mod upstream_spine_c_smoke_tests;

#[cfg(all(test, feature = "binary", feature = "upstream-smoke"))]
mod upstream_spine_skel_smoke_tests;

#[cfg(all(test, feature = "binary", feature = "upstream-smoke"))]
mod upstream_libgdx_physics_demo_tests;

#[cfg(all(test, feature = "json", feature = "upstream-smoke"))]
mod upstream_ik_demo_tests;

#[cfg(all(test, feature = "binary", feature = "upstream-smoke"))]
mod upstream_ik_demo_skel_tests;

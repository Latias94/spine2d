//! Pure Rust runtime for Spine 4.3 exported data (unofficial).
//!
//! This crate is renderer-agnostic. Rendering integrations live in separate crates
//! (e.g. `spine2d-wgpu`).

#![forbid(unsafe_code)]

mod atlas;
mod error;
#[cfg(any(feature = "json", feature = "binary"))]
mod export_version;
mod geometry;
#[cfg(any(feature = "json", feature = "binary"))]
mod ids;
mod model;
mod render;
mod runtime;

#[cfg(feature = "json")]
pub mod json;

#[cfg(feature = "binary")]
pub mod binary;

pub use atlas::*;
pub use error::*;
pub use model::*;
pub use render::*;
pub use runtime::*;

#[cfg(test)]
mod geometry_tests;

#[cfg(test)]
mod model_lookup_tests;

#[cfg(all(test, feature = "json"))]
mod render_tests;

#[cfg(all(test, feature = "json"))]
mod json_scale_tests;

#[cfg(all(test, feature = "binary"))]
mod binary_tests;

#[cfg(all(test, any(feature = "json", feature = "binary")))]
mod version_tests;

#[cfg(all(test, feature = "json"))]
mod json_event_tests;

#[cfg(all(test, feature = "json"))]
mod json_physics_defaults_tests;

#[cfg(all(test, feature = "json"))]
mod json_transform_constraint_tests;

#[cfg(all(test, feature = "json"))]
mod json_timeline_order_tests;

#[cfg(all(test, feature = "json", feature = "upstream-smoke"))]
mod render_oracle_parity_tests;

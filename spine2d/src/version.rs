//! Target Spine export version.

/// Target Spine major version for exported data.
pub const SPINE_EXPORT_MAJOR: u32 = 4;

/// Target Spine minor version for exported data.
pub const SPINE_EXPORT_MINOR: u32 = 3;

/// Required Spine export version prefix, matching the official C++ runtime.
pub(crate) const SPINE_EXPORT_VERSION_PREFIX: &str = "4.3";

pub(crate) fn spine_version_matches_runtime(value: &str) -> bool {
    value.starts_with(SPINE_EXPORT_VERSION_PREFIX)
}

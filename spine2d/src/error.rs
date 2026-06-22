use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unknown animation: {name}")]
    UnknownAnimation { name: String },

    #[error("invalid track index: {index}")]
    InvalidTrackIndex { index: usize },

    #[error("invalid value: {message}")]
    InvalidValue { message: String },

    #[cfg(feature = "json")]
    #[error("failed to parse Spine JSON: {message}")]
    JsonParse { message: String },

    #[cfg(feature = "json")]
    #[error("invalid color '{value}' for {context}")]
    JsonInvalidColor { context: String, value: String },

    #[cfg(feature = "json")]
    #[error("invalid curve for {context}: {message}")]
    JsonInvalidCurve { context: String, message: String },

    #[cfg(feature = "json")]
    #[error("unsupported or invalid Spine version string: {value}")]
    JsonSpineVersion { value: String },

    #[cfg(feature = "binary")]
    #[error("failed to parse Spine binary: {message}")]
    BinaryParse { message: String },

    #[cfg(feature = "binary")]
    #[error("unsupported or invalid Spine version string: {value}")]
    BinarySpineVersion { value: String },

    #[error("failed to parse Spine atlas: {message}")]
    AtlasParse { message: String },

    #[cfg(feature = "json")]
    #[error("unknown parent bone '{parent}' for bone '{bone}'")]
    JsonUnknownBoneParent { bone: String, parent: String },

    #[cfg(feature = "json")]
    #[error("unknown bone '{bone}' referenced by animation '{animation}'")]
    JsonUnknownAnimationBone { animation: String, bone: String },

    #[cfg(feature = "json")]
    #[error("unknown bone '{bone}' referenced by slot '{slot}'")]
    JsonUnknownSlotBone { slot: String, bone: String },

    #[cfg(feature = "json")]
    #[error("unsupported blend mode '{value}' for slot '{slot}'")]
    JsonUnsupportedBlendMode { slot: String, value: String },

    #[cfg(feature = "json")]
    #[error("unknown slot '{slot}' referenced by skin '{skin}'")]
    JsonUnknownSkinSlot { skin: String, slot: String },

    #[cfg(feature = "json")]
    #[error("unknown bone '{bone}' referenced by skin '{skin}'")]
    JsonUnknownSkinBone { skin: String, bone: String },

    #[cfg(feature = "json")]
    #[error("unknown {kind} constraint '{constraint}' referenced by skin '{skin}'")]
    JsonUnknownSkinConstraint {
        skin: String,
        kind: String,
        constraint: String,
    },

    #[cfg(feature = "json")]
    #[error(
        "unsupported attachment type '{attachment_type}' for skin '{skin}', slot '{slot}', attachment '{attachment}'"
    )]
    JsonUnsupportedAttachmentType {
        skin: String,
        slot: String,
        attachment: String,
        attachment_type: String,
    },

    #[cfg(feature = "json")]
    #[error(
        "unsupported weighted mesh for skin '{skin}', slot '{slot}', attachment '{attachment}'"
    )]
    JsonUnsupportedWeightedMesh {
        skin: String,
        slot: String,
        attachment: String,
    },

    #[cfg(feature = "json")]
    #[error(
        "invalid mesh data for skin '{skin}', slot '{slot}', attachment '{attachment}': {message}"
    )]
    JsonInvalidMeshData {
        skin: String,
        slot: String,
        attachment: String,
        message: String,
    },

    #[cfg(feature = "json")]
    #[error("unknown skin '{skin}' referenced by deform timeline in animation '{animation}'")]
    JsonUnknownDeformSkin { animation: String, skin: String },

    #[cfg(feature = "json")]
    #[error(
        "unknown slot '{slot}' referenced by deform timeline in animation '{animation}', skin '{skin}'"
    )]
    JsonUnknownDeformSlot {
        animation: String,
        skin: String,
        slot: String,
    },

    #[cfg(feature = "json")]
    #[error(
        "unknown attachment '{attachment}' referenced by deform timeline in animation '{animation}', skin '{skin}', slot '{slot}'"
    )]
    JsonUnknownDeformAttachment {
        animation: String,
        skin: String,
        slot: String,
        attachment: String,
    },

    #[cfg(feature = "json")]
    #[error(
        "unsupported deform timeline attachment type for animation '{animation}', skin '{skin}', slot '{slot}', attachment '{attachment}'"
    )]
    JsonUnsupportedDeformAttachment {
        animation: String,
        skin: String,
        slot: String,
        attachment: String,
    },

    #[cfg(feature = "json")]
    #[error(
        "invalid deform data for animation '{animation}', skin '{skin}', slot '{slot}', attachment '{attachment}': {message}"
    )]
    JsonInvalidDeformData {
        animation: String,
        skin: String,
        slot: String,
        attachment: String,
        message: String,
    },

    #[cfg(feature = "json")]
    #[error("unknown skin '{skin}' referenced by sequence timeline in animation '{animation}'")]
    JsonUnknownSequenceSkin { animation: String, skin: String },

    #[cfg(feature = "json")]
    #[error(
        "unknown slot '{slot}' referenced by sequence timeline in animation '{animation}', skin '{skin}'"
    )]
    JsonUnknownSequenceSlot {
        animation: String,
        skin: String,
        slot: String,
    },

    #[cfg(feature = "json")]
    #[error(
        "unknown attachment '{attachment}' referenced by sequence timeline in animation '{animation}', skin '{skin}', slot '{slot}'"
    )]
    JsonUnknownSequenceAttachment {
        animation: String,
        skin: String,
        slot: String,
        attachment: String,
    },

    #[cfg(feature = "json")]
    #[error(
        "unsupported sequence timeline attachment type for animation '{animation}', skin '{skin}', slot '{slot}', attachment '{attachment}'"
    )]
    JsonUnsupportedSequenceAttachment {
        animation: String,
        skin: String,
        slot: String,
        attachment: String,
    },

    #[cfg(feature = "json")]
    #[error("unknown slot '{slot}' referenced by slot timeline in animation '{animation}'")]
    JsonUnknownSlotTimelineSlot { animation: String, slot: String },

    #[cfg(feature = "json")]
    #[error("unknown event '{event}' referenced by animation '{animation}'")]
    JsonUnknownEvent { animation: String, event: String },

    #[cfg(feature = "json")]
    #[error("invalid drawOrder data for animation '{animation}': {message}")]
    JsonInvalidDrawOrder { animation: String, message: String },

    #[cfg(feature = "json")]
    #[error(
        "unknown IK constraint '{constraint}' referenced by IK timeline in animation '{animation}'"
    )]
    JsonUnknownIkConstraintTimeline {
        animation: String,
        constraint: String,
    },

    #[cfg(feature = "json")]
    #[error(
        "unknown transform constraint '{constraint}' referenced by transform timeline in animation '{animation}'"
    )]
    JsonUnknownTransformConstraintTimeline {
        animation: String,
        constraint: String,
    },

    #[cfg(feature = "json")]
    #[error("unknown path constraint bone '{bone}' referenced by path constraint '{constraint}'")]
    JsonUnknownPathConstraintBone { constraint: String, bone: String },

    #[cfg(feature = "json")]
    #[error("unknown target slot '{slot}' referenced by path constraint '{constraint}'")]
    JsonUnknownPathConstraintTargetSlot { constraint: String, slot: String },

    #[cfg(feature = "json")]
    #[error("unknown bone '{bone}' referenced by physics constraint '{constraint}'")]
    JsonUnknownPhysicsConstraintBone { constraint: String, bone: String },

    #[cfg(feature = "json")]
    #[error("unknown bone '{bone}' referenced by slider constraint '{constraint}'")]
    JsonUnknownSliderConstraintBone { constraint: String, bone: String },

    #[cfg(feature = "json")]
    #[error("unsupported path constraint {field} '{value}' for constraint '{constraint}'")]
    JsonUnsupportedPathConstraintMode {
        constraint: String,
        field: String,
        value: String,
    },

    #[cfg(feature = "json")]
    #[error(
        "unknown path constraint '{constraint}' referenced by path timeline in animation '{animation}'"
    )]
    JsonUnknownPathConstraintTimeline {
        animation: String,
        constraint: String,
    },

    #[cfg(feature = "json")]
    #[error(
        "unknown physics constraint '{constraint}' referenced by physics timeline in animation '{animation}'"
    )]
    JsonUnknownPhysicsConstraintTimeline {
        animation: String,
        constraint: String,
    },

    #[cfg(feature = "json")]
    #[error(
        "unknown slider constraint '{constraint}' referenced by slider timeline in animation '{animation}'"
    )]
    JsonUnknownSliderConstraintTimeline {
        animation: String,
        constraint: String,
    },

    #[cfg(feature = "json")]
    #[error("unknown animation '{animation}' referenced by slider constraint '{constraint}'")]
    JsonUnknownSliderAnimation {
        constraint: String,
        animation: String,
    },

    #[cfg(feature = "json")]
    #[error(
        "invalid path data for skin '{skin}', slot '{slot}', attachment '{attachment}': {message}"
    )]
    JsonInvalidPathData {
        skin: String,
        slot: String,
        attachment: String,
        message: String,
    },

    #[cfg(feature = "json")]
    #[error(
        "slot '{slot}' referenced by animation '{animation}' has a '{timeline}' timeline but no setup 'dark' color"
    )]
    JsonTwoColorTimelineRequiresDarkSlot {
        animation: String,
        slot: String,
        timeline: String,
    },
}

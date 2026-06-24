use crate::{
    AlphaFrame, AlphaTimeline, Animation, AttachmentData, AttachmentFrame, AttachmentTimeline,
    BoneData, BoneTimeline, BoundingBoxAttachmentData, ClippingAttachmentData, ColorFrame,
    ColorTimeline, Curve, DeformFrame, DeformTimeline, DrawOrderFolderFrame,
    DrawOrderFolderTimeline, DrawOrderFrame, DrawOrderTimeline, Error, Event, EventData,
    EventTimeline, FloatFrame, IkConstraintTimeline, IkFrame, InheritFrame, InheritTimeline,
    MeshAttachmentData, PathAttachmentData, PointAttachmentData, RegionAttachmentData, Rgb2Frame,
    Rgb2Timeline, RgbFrame, RgbTimeline, Rgba2Frame, Rgba2Timeline, RotateFrame, RotateTimeline,
    ScaleTimeline, ScaleXTimeline, ScaleYTimeline, ShearTimeline, ShearXTimeline, ShearYTimeline,
    SkeletonData, SkinData, SliderConstraintData, SliderConstraintTimeline, SlotData, TimelineKind,
    TransformFromProperty, TransformProperty, TransformToProperty, TranslateTimeline,
    TranslateXTimeline, TranslateYTimeline, Vec2Frame,
};
use indexmap::IndexMap;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ConstraintDef {
    #[serde(rename = "ik")]
    Ik(IkConstraintDef),
    #[serde(rename = "transform")]
    Transform(TransformConstraintDef),
    #[serde(rename = "path")]
    Path(PathConstraintDef),
    #[serde(rename = "physics")]
    Physics(PhysicsConstraintDef),
    #[serde(rename = "slider")]
    Slider(SliderConstraintDef),
}

#[derive(Debug, Deserialize)]
struct Root {
    skeleton: Option<SkeletonHeader>,
    bones: Option<Vec<BoneDef>>,
    slots: Option<Vec<SlotDef>>,
    skins: Option<SkinsDef>,
    #[serde(default)]
    events: IndexMap<String, EventDef>,
    #[serde(default)]
    constraints: Option<Vec<ConstraintDef>>,
    ik: Option<Vec<IkConstraintDef>>,
    transform: Option<Vec<TransformConstraintDef>>,
    path: Option<Vec<PathConstraintDef>>,
    physics: Option<Vec<PhysicsConstraintDef>>,
    slider: Option<Vec<SliderConstraintDef>>,
    #[serde(default)]
    animations: IndexMap<String, Value>,
}

fn default_event_volume() -> f32 {
    1.0
}

#[derive(Debug, Deserialize, Default)]
struct EventDef {
    #[serde(default, rename = "int")]
    int_value: i32,
    #[serde(default, rename = "float")]
    float_value: f32,
    #[serde(default, rename = "string")]
    string_value: String,
    #[serde(default, rename = "audio")]
    audio_path: String,
    #[serde(default = "default_event_volume")]
    volume: f32,
    #[serde(default)]
    balance: f32,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum SkinsDef {
    Map(IndexMap<String, IndexMap<String, IndexMap<String, AttachmentDef>>>),
    Array(Vec<SkinDef>),
}

#[derive(Debug, Deserialize)]
struct SkinDef {
    name: String,
    #[serde(default)]
    attachments: IndexMap<String, IndexMap<String, AttachmentDef>>,
    #[serde(default)]
    bones: Vec<String>,
    #[serde(default)]
    ik: Vec<String>,
    #[serde(default)]
    transform: Vec<String>,
    #[serde(default)]
    path: Vec<String>,
    #[serde(default)]
    physics: Vec<String>,
    #[serde(default)]
    slider: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SkeletonHeader {
    spine: Option<String>,
    #[serde(default, rename = "referenceScale")]
    reference_scale: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct BoneDef {
    name: String,
    #[serde(default)]
    parent: Option<String>,
    #[serde(default)]
    length: f32,
    #[serde(default)]
    x: f32,
    #[serde(default)]
    y: f32,
    #[serde(default)]
    rotation: f32,
    #[serde(default = "default_one", rename = "scaleX")]
    scale_x: f32,
    #[serde(default = "default_one", rename = "scaleY")]
    scale_y: f32,
    #[serde(default, rename = "shearX")]
    shear_x: f32,
    #[serde(default, rename = "shearY")]
    shear_y: f32,
    #[serde(default)]
    inherit: Option<String>,
    #[serde(default, rename = "skin")]
    skin_required: bool,
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    icon: Option<String>,
    #[serde(default, rename = "iconSize")]
    icon_size: Option<f32>,
    #[serde(default, rename = "iconRotation")]
    icon_rotation: Option<f32>,
    #[serde(default)]
    visible: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct SlotDef {
    name: String,
    bone: String,
    #[serde(default)]
    attachment: Option<String>,
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    dark: Option<String>,
    #[serde(default)]
    blend: Option<String>,
    #[serde(default)]
    visible: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct IkConstraintDef {
    name: String,
    #[serde(default)]
    order: i32,
    #[serde(default, rename = "skin")]
    skin_required: bool,
    bones: Vec<String>,
    target: String,
    #[serde(default = "default_one")]
    mix: f32,
    #[serde(default)]
    softness: f32,
    #[serde(default)]
    compress: bool,
    #[serde(default)]
    stretch: bool,
    #[serde(default, rename = "scaleY")]
    scale_y: Option<String>,
    #[serde(default = "default_true", rename = "bendPositive")]
    bend_positive: bool,
}

#[derive(Debug, Deserialize)]
struct TransformConstraintDef {
    name: String,
    #[serde(default)]
    order: i32,
    #[serde(default, rename = "skin")]
    skin_required: bool,
    bones: Vec<String>,
    #[serde(default)]
    #[serde(alias = "target")]
    source: String,

    #[serde(default, rename = "localSource")]
    local_source: bool,
    #[serde(default)]
    #[serde(rename = "localTarget")]
    local_target: bool,
    #[serde(default)]
    additive: bool,
    #[serde(default)]
    clamp: bool,

    #[serde(default)]
    x: f32,
    #[serde(default)]
    y: f32,
    #[serde(default)]
    rotation: f32,
    #[serde(default, rename = "scaleX")]
    scale_x: f32,
    #[serde(default, rename = "scaleY")]
    scale_y: f32,
    #[serde(default, rename = "shearY")]
    shear_y: f32,

    #[serde(default)]
    properties: Option<IndexMap<String, TransformFromDef>>,

    #[serde(default, rename = "mixRotate")]
    mix_rotate: Option<f32>,
    #[serde(default, rename = "mixX")]
    mix_x: Option<f32>,
    #[serde(default, rename = "mixY")]
    mix_y: Option<f32>,
    #[serde(default, rename = "mixScaleX")]
    mix_scale_x: Option<f32>,
    #[serde(default, rename = "mixScaleY")]
    mix_scale_y: Option<f32>,
    #[serde(default, rename = "mixShearY")]
    mix_shear_y: Option<f32>,
}

#[derive(Debug, Deserialize, Default)]
struct TransformFromDef {
    #[serde(default)]
    offset: f32,
    #[serde(default)]
    to: Option<IndexMap<String, TransformToDef>>,
}

#[derive(Debug, Deserialize, Default)]
struct TransformToDef {
    #[serde(default)]
    offset: f32,
    #[serde(default = "default_one")]
    max: f32,
    #[serde(default = "default_one")]
    scale: f32,
}

#[derive(Debug, Deserialize)]
struct PathConstraintDef {
    name: String,
    #[serde(default)]
    order: i32,
    #[serde(default, rename = "skin")]
    skin_required: bool,
    bones: Vec<String>,
    #[serde(alias = "slot")]
    target: String,
    #[serde(default, rename = "positionMode")]
    position_mode: Option<String>,
    #[serde(default, rename = "spacingMode")]
    spacing_mode: Option<String>,
    #[serde(default, rename = "rotateMode")]
    rotate_mode: Option<String>,
    #[serde(default, rename = "rotation")]
    offset_rotation: f32,
    #[serde(default)]
    position: f32,
    #[serde(default)]
    spacing: f32,
    #[serde(default = "default_one", rename = "mixRotate")]
    mix_rotate: f32,
    #[serde(default = "default_one", rename = "mixX")]
    mix_x: f32,
    #[serde(default, rename = "mixY")]
    mix_y: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct PhysicsConstraintDef {
    name: String,
    #[serde(default)]
    order: i32,
    #[serde(default, rename = "skin")]
    skin_required: bool,
    bone: String,

    #[serde(default)]
    x: f32,
    #[serde(default)]
    y: f32,
    #[serde(default)]
    rotate: f32,
    #[serde(default, rename = "scaleX")]
    scale_x: f32,
    #[serde(default, rename = "scaleY")]
    scale_y: Option<String>,
    #[serde(default, rename = "shearX")]
    shear_x: f32,
    #[serde(default)]
    limit: Option<f32>,
    #[serde(default)]
    fps: Option<i32>,

    #[serde(default)]
    inertia: Option<f32>,
    #[serde(default)]
    strength: Option<f32>,
    #[serde(default)]
    damping: Option<f32>,
    #[serde(default)]
    mass: Option<f32>,
    #[serde(default)]
    wind: Option<f32>,
    #[serde(default)]
    gravity: Option<f32>,
    #[serde(default)]
    mix: Option<f32>,

    #[serde(default, rename = "inertiaGlobal")]
    inertia_global: bool,
    #[serde(default, rename = "strengthGlobal")]
    strength_global: bool,
    #[serde(default, rename = "dampingGlobal")]
    damping_global: bool,
    #[serde(default, rename = "massGlobal")]
    mass_global: bool,
    #[serde(default, rename = "windGlobal")]
    wind_global: bool,
    #[serde(default, rename = "gravityGlobal")]
    gravity_global: bool,
    #[serde(default, rename = "mixGlobal")]
    mix_global: bool,
}

#[derive(Debug, Deserialize)]
struct SliderConstraintDef {
    name: String,
    #[serde(default)]
    order: i32,
    #[serde(default, rename = "skin")]
    skin_required: bool,

    #[serde(default)]
    additive: bool,
    #[serde(default, rename = "loop")]
    looped: bool,

    #[serde(default)]
    time: f32,
    #[serde(default = "default_one")]
    mix: f32,

    #[serde(default)]
    animation: Option<String>,

    #[serde(default)]
    bone: Option<String>,
    #[serde(default)]
    property: Option<String>,
    #[serde(default)]
    from: f32,
    #[serde(default)]
    to: f32,
    #[serde(default = "default_one")]
    scale: f32,
    #[serde(default)]
    local: bool,
}

#[derive(Debug, Deserialize)]
struct AnimationDef {
    events: Option<Vec<EventKey>>,
    #[serde(default)]
    bones: IndexMap<String, BoneAnimDef>,
    attachments: Option<AttachmentTimelinesBySlot>,
    #[serde(default)]
    slots: IndexMap<String, SlotAnimDef>,
    #[serde(rename = "drawOrder")]
    draw_order: Option<Vec<DrawOrderKey>>,
    #[serde(rename = "drawOrderFolder")]
    #[serde(default)]
    draw_order_folder: IndexMap<String, DrawOrderFolderDef>,
    #[serde(default)]
    ik: IndexMap<String, Vec<IkTimelineKey>>,
    #[serde(default)]
    transform: IndexMap<String, Vec<TransformTimelineKey>>,
    #[serde(default)]
    path: IndexMap<String, IndexMap<String, Vec<PathTimelineKey>>>,
    #[serde(default)]
    physics: IndexMap<String, IndexMap<String, Vec<PhysicsTimelineKey>>>,
    #[serde(default)]
    slider: IndexMap<String, IndexMap<String, Vec<FloatKey>>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
enum JsonTimelineRef {
    Slot {
        slot: String,
        timeline: String,
    },
    Bone {
        bone: String,
        timeline: String,
    },
    Ik {
        constraint: String,
    },
    Transform {
        constraint: String,
    },
    Path {
        constraint: String,
        timeline: String,
    },
    Physics {
        constraint: String,
        timeline: String,
    },
    Slider {
        constraint: String,
        timeline: String,
    },
    Attachment {
        skin: String,
        slot: String,
        attachment: String,
        timeline: String,
    },
    DrawOrder,
    DrawOrderFolder {
        folder: String,
    },
}

type AttachmentTimelinesBySlot =
    IndexMap<String, IndexMap<String, IndexMap<String, AttachmentTimelinesDef>>>;

#[derive(Debug, Deserialize)]
struct PathTimelineKey {
    #[serde(default)]
    time: Option<serde_json::Value>,
    #[serde(default, rename = "value")]
    value: Option<serde_json::Value>,
    #[serde(default)]
    position: Option<serde_json::Value>,
    #[serde(default)]
    spacing: Option<serde_json::Value>,
    #[serde(default, rename = "mixRotate")]
    mix_rotate: Option<serde_json::Value>,
    #[serde(default, rename = "mixX")]
    mix_x: Option<serde_json::Value>,
    #[serde(default, rename = "mixY")]
    mix_y: Option<serde_json::Value>,
    #[serde(default)]
    curve: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct PhysicsTimelineKey {
    #[serde(default)]
    time: Option<f32>,
    #[serde(default)]
    value: Option<f32>,
    #[serde(default)]
    curve: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct EventKey {
    #[serde(default)]
    time: Option<f32>,
    name: String,
    #[serde(default, rename = "int")]
    int_value: Option<i32>,
    #[serde(default, rename = "float")]
    float_value: Option<f32>,
    #[serde(default, rename = "string")]
    string_value: Option<String>,
    #[serde(default)]
    volume: Option<f32>,
    #[serde(default)]
    balance: Option<f32>,
}

#[derive(Debug, Deserialize, Default)]
struct BoneAnimDef {
    #[serde(default)]
    rotate: Option<Vec<RotateKey>>,
    #[serde(default)]
    translate: Option<Vec<Vec2Key>>,
    #[serde(rename = "translatex", alias = "translateX")]
    translate_x: Option<Vec<FloatKey>>,
    #[serde(rename = "translatey", alias = "translateY")]
    translate_y: Option<Vec<FloatKey>>,
    #[serde(default)]
    scale: Option<Vec<Vec2Key>>,
    #[serde(rename = "scalex", alias = "scaleX")]
    scale_x: Option<Vec<FloatKey>>,
    #[serde(rename = "scaley", alias = "scaleY")]
    scale_y: Option<Vec<FloatKey>>,
    #[serde(default)]
    shear: Option<Vec<Vec2Key>>,
    #[serde(rename = "shearx", alias = "shearX")]
    shear_x: Option<Vec<FloatKey>>,
    #[serde(rename = "sheary", alias = "shearY")]
    shear_y: Option<Vec<FloatKey>>,
    #[serde(default)]
    inherit: Option<Vec<InheritKey>>,
}

#[derive(Debug, Deserialize)]
struct RotateKey {
    #[serde(default)]
    time: Option<f32>,
    #[serde(default)]
    angle: Option<f32>,
    #[serde(default)]
    value: Option<f32>,
    #[serde(default)]
    curve: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct FloatKey {
    #[serde(default)]
    time: Option<f32>,
    #[serde(default)]
    value: Option<f32>,
    #[serde(default)]
    curve: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct Vec2Key {
    #[serde(default)]
    time: Option<f32>,
    #[serde(default)]
    x: Option<f32>,
    #[serde(default)]
    y: Option<f32>,
    #[serde(default)]
    curve: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct InheritKey {
    #[serde(default)]
    time: Option<f32>,
    #[serde(default)]
    inherit: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AttachmentDef {
    #[serde(default, rename = "type")]
    attachment_type: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    parent: Option<String>,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    slot: Option<String>,
    #[serde(default)]
    skin: Option<String>,
    #[serde(default)]
    timelines: Option<bool>,
    #[serde(default)]
    end: Option<String>,
    #[serde(default)]
    x: f32,
    #[serde(default)]
    y: f32,
    #[serde(default)]
    rotation: f32,
    #[serde(default = "default_one", rename = "scaleX")]
    scale_x: f32,
    #[serde(default = "default_one", rename = "scaleY")]
    scale_y: f32,
    #[serde(default)]
    width: f32,
    #[serde(default)]
    height: f32,
    #[serde(default)]
    uvs: Option<Vec<f32>>,
    #[serde(default)]
    vertices: Option<Vec<f32>>,
    #[serde(default)]
    triangles: Option<Vec<u32>>,
    #[serde(default)]
    sequence: Option<AttachmentSequenceDef>,
    #[serde(default)]
    color: Option<String>,

    #[serde(default, rename = "closed")]
    closed: bool,
    #[serde(default = "default_true", rename = "constantSpeed")]
    constant_speed: bool,
    #[serde(default, rename = "vertexCount")]
    vertex_count: Option<usize>,
    #[serde(default)]
    lengths: Option<Vec<f32>>,
    #[serde(default)]
    convex: bool,
    #[serde(default)]
    inverse: bool,
}

#[derive(Debug, Deserialize)]
struct AttachmentSequenceDef {
    count: usize,
    #[serde(default = "default_one_i32")]
    start: i32,
    #[serde(default)]
    digits: usize,
    #[serde(default, rename = "setup")]
    setup_index: i32,
}

fn default_one_i32() -> i32 {
    1
}

#[derive(Debug, Deserialize, Default)]
struct SlotAnimDef {
    #[serde(default)]
    attachment: Option<Vec<SlotAttachmentKey>>,
    #[serde(default, rename = "rgba")]
    rgba: Option<Vec<SlotColorKey>>,
    #[serde(default, rename = "rgb")]
    rgb: Option<Vec<SlotColorKey>>,
    #[serde(default, rename = "alpha")]
    alpha: Option<Vec<SlotAlphaKey>>,
    #[serde(default, rename = "rgba2")]
    rgba2: Option<Vec<SlotTwoColorKey>>,
    #[serde(default, rename = "rgb2")]
    rgb2: Option<Vec<SlotTwoColorKey>>,
}

#[derive(Debug, Deserialize)]
struct SlotAttachmentKey {
    #[serde(default)]
    time: Option<f32>,
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SlotColorKey {
    #[serde(default)]
    time: Option<f32>,
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    curve: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct SlotAlphaKey {
    #[serde(default)]
    time: Option<f32>,
    #[serde(default)]
    value: Option<f32>,
    #[serde(default)]
    curve: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct SlotTwoColorKey {
    #[serde(default)]
    time: Option<f32>,
    #[serde(default)]
    light: Option<String>,
    #[serde(default)]
    dark: Option<String>,
    #[serde(default)]
    curve: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct DrawOrderKey {
    #[serde(default)]
    time: Option<f32>,
    #[serde(default)]
    offsets: Option<Vec<DrawOrderOffset>>,
}

#[derive(Debug, Deserialize)]
struct DrawOrderOffset {
    slot: String,
    #[serde(default)]
    offset: i32,
}

#[derive(Debug, Deserialize)]
struct DrawOrderFolderDef {
    #[serde(default)]
    slots: Vec<String>,
    #[serde(default)]
    keys: Option<Vec<DrawOrderKey>>,
}

#[derive(Debug, Deserialize)]
struct IkTimelineKey {
    #[serde(default)]
    time: Option<f32>,
    #[serde(default)]
    mix: Option<f32>,
    #[serde(default)]
    softness: Option<f32>,
    #[serde(default)]
    compress: Option<bool>,
    #[serde(default)]
    stretch: Option<bool>,
    #[serde(default, rename = "bendPositive")]
    bend_positive: Option<bool>,
    #[serde(default)]
    curve: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct TransformTimelineKey {
    #[serde(default)]
    time: Option<f32>,
    #[serde(default, rename = "mixRotate")]
    mix_rotate: Option<f32>,
    #[serde(default, rename = "mixX")]
    mix_x: Option<f32>,
    #[serde(default, rename = "mixY")]
    mix_y: Option<f32>,
    #[serde(default, rename = "mixScaleX")]
    mix_scale_x: Option<f32>,
    #[serde(default, rename = "mixScaleY")]
    mix_scale_y: Option<f32>,
    #[serde(default, rename = "mixShearY")]
    mix_shear_y: Option<f32>,
    #[serde(default)]
    curve: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Default)]
struct AttachmentTimelinesDef {
    #[serde(default)]
    deform: Option<Vec<DeformKey>>,
    #[serde(default)]
    sequence: Option<Vec<SequenceKey>>,
}

#[derive(Debug, Deserialize)]
struct DeformKey {
    #[serde(default)]
    time: Option<f32>,
    #[serde(default)]
    offset: Option<usize>,
    #[serde(default)]
    vertices: Option<Vec<f32>>,
    #[serde(default)]
    curve: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct SequenceKey {
    #[serde(default)]
    time: Option<f32>,
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    index: Option<i32>,
    #[serde(default)]
    delay: Option<f32>,
}

impl SkeletonData {
    pub fn from_json_str(input: &str) -> Result<Arc<Self>, Error> {
        Self::from_json_str_with_scale(input, 1.0)
    }

    pub fn from_json_str_with_scale(input: &str, scale: f32) -> Result<Arc<Self>, Error> {
        let root: Root = serde_json::from_str(input).map_err(|e| Error::JsonParse {
            message: e.to_string(),
        })?;

        let (spine_version, reference_scale_raw) = match root.skeleton {
            Some(s) => {
                let spine_version = s.spine;
                validate_spine_version(spine_version.as_deref().unwrap_or(""))?;
                (spine_version, s.reference_scale.unwrap_or(100.0))
            }
            None => (None, 100.0),
        };

        let scale = if scale.is_finite() { scale } else { 1.0 };
        let reference_scale = reference_scale_raw * scale;

        fn parse_inherit(raw: Option<&str>) -> crate::Inherit {
            match raw.unwrap_or("normal") {
                "normal" => crate::Inherit::Normal,
                "onlyTranslation" => crate::Inherit::OnlyTranslation,
                "noRotationOrReflection" => crate::Inherit::NoRotationOrReflection,
                "noScale" => crate::Inherit::NoScale,
                "noScaleOrReflection" => crate::Inherit::NoScaleOrReflection,
                _ => crate::Inherit::Normal,
            }
        }

        let mut bones = Vec::new();
        let mut bone_index = HashMap::<String, usize>::new();
        for bone in root.bones.unwrap_or_default() {
            let parent = match bone.parent.as_deref() {
                None => None,
                Some(parent_name) => {
                    Some(bone_index.get(parent_name).copied().ok_or_else(|| {
                        Error::JsonUnknownBoneParent {
                            bone: bone.name.clone(),
                            parent: parent_name.to_string(),
                        }
                    })?)
                }
            };

            let index = bones.len();
            bone_index.insert(bone.name.clone(), index);
            bones.push(BoneData {
                name: bone.name,
                parent,
                length: bone.length * scale,
                x: bone.x * scale,
                y: bone.y * scale,
                rotation: bone.rotation,
                scale_x: bone.scale_x,
                scale_y: bone.scale_y,
                shear_x: bone.shear_x,
                shear_y: bone.shear_y,
                inherit: parse_inherit(bone.inherit.as_deref()),
                skin_required: bone.skin_required,
                color: bone
                    .color
                    .as_deref()
                    .map(|s| parse_hex_color_rgba(s, "bone color"))
                    .transpose()?
                    .unwrap_or([0.61, 0.61, 0.61, 1.0]),
                icon: bone.icon.unwrap_or_default(),
                icon_size: bone.icon_size.unwrap_or(1.0),
                icon_rotation: bone.icon_rotation.unwrap_or(0.0),
                visible: bone.visible.unwrap_or(true),
            });
        }

        let mut slots = Vec::new();
        let mut slot_index = HashMap::<String, usize>::new();
        for slot in root.slots.unwrap_or_default() {
            let slot_name = slot.name.clone();
            let bone =
                bone_index
                    .get(&slot.bone)
                    .copied()
                    .ok_or_else(|| Error::JsonUnknownSlotBone {
                        slot: slot_name.clone(),
                        bone: slot.bone.clone(),
                    })?;
            let index = slots.len();
            slot_index.insert(slot_name.clone(), index);
            let dark = slot
                .dark
                .as_deref()
                .map(|s| parse_hex_color_rgba(s, "slot setup dark"))
                .transpose()?
                .map(|c| [c[0], c[1], c[2]]);
            slots.push(SlotData {
                name: slot_name.clone(),
                bone,
                attachment: slot.attachment,
                color: slot
                    .color
                    .as_deref()
                    .map(|s| parse_hex_color_rgba(s, "slot setup color"))
                    .transpose()?
                    .unwrap_or([1.0, 1.0, 1.0, 1.0]),
                has_dark: dark.is_some(),
                dark_color: dark.unwrap_or([0.0, 0.0, 0.0]),
                blend: parse_blend_mode(slot.blend.as_deref(), &slot_name)?,
                visible: slot.visible.unwrap_or(true),
            });
        }

        #[derive(Clone, Debug, Default)]
        struct PendingSkinConstraints {
            ik: Vec<String>,
            transform: Vec<String>,
            path: Vec<String>,
            physics: Vec<String>,
            slider: Vec<String>,
        }

        let mut skins = HashMap::<String, SkinData>::new();
        let mut pending_skin_constraints: HashMap<String, PendingSkinConstraints> = HashMap::new();
        if let Some(skins_def) = root.skins {
            #[derive(Clone, Debug)]
            struct PendingLinkedMesh {
                skin: String,
                slot_index: usize,
                attachment_name: String,
                source: String,
                parent_skin: Option<String>,
                source_slot_index: usize,
                inherit_deform: bool,
            }

            let mut pending_linked_meshes: Vec<PendingLinkedMesh> = Vec::new();

            let mut add_skin = |skin_name: String,
                                skin_slots: IndexMap<String, IndexMap<String, AttachmentDef>>,
                                skin_bones: Vec<String>,
                                skin_ik: Vec<String>,
                                skin_transform: Vec<String>,
                                skin_path: Vec<String>,
                                skin_physics: Vec<String>,
                                skin_slider: Vec<String>|
             -> Result<(), Error> {
                let mut attachments = vec![IndexMap::new(); slots.len()];
                for (slot_name, slot_attachments) in skin_slots {
                    let s_index =
                        *slot_index
                            .get(&slot_name)
                            .ok_or_else(|| Error::JsonUnknownSkinSlot {
                                skin: skin_name.clone(),
                                slot: slot_name.clone(),
                            })?;
                    for (attachment_name, attachment_def) in slot_attachments {
                        let attachment_type = attachment_def
                            .attachment_type
                            .as_deref()
                            .unwrap_or("region");
                        if attachment_type != "region"
                            && attachment_type != "mesh"
                            && attachment_type != "linkedmesh"
                            && attachment_type != "point"
                            && attachment_type != "path"
                            && attachment_type != "boundingbox"
                            && attachment_type != "clipping"
                        {
                            return Err(Error::JsonUnsupportedAttachmentType {
                                skin: skin_name.clone(),
                                slot: slot_name.clone(),
                                attachment: attachment_name.clone(),
                                attachment_type: attachment_type.to_string(),
                            });
                        }

                        let internal_name = attachment_def
                            .name
                            .clone()
                            .unwrap_or_else(|| attachment_name.clone());
                        let path = attachment_def
                            .path
                            .clone()
                            .unwrap_or_else(|| internal_name.clone());
                        let sequence = attachment_def.sequence.as_ref().map(|s| {
                            let id = crate::ids::next_sequence_id();
                            crate::SequenceDef {
                                id,
                                count: s.count,
                                start: s.start,
                                digits: s.digits,
                                setup_index: s.setup_index,
                            }
                        });
                        let attachment_color = attachment_def
                            .color
                            .as_deref()
                            .map(|s| parse_hex_color_rgba(s, "attachment color"))
                            .transpose()?
                            .unwrap_or([1.0, 1.0, 1.0, 1.0]);
                        let attachment = match attachment_type {
                            "region" => AttachmentData::Region(RegionAttachmentData {
                                name: internal_name.clone(),
                                path,
                                sequence: sequence.clone(),
                                color: attachment_color,
                                x: attachment_def.x * scale,
                                y: attachment_def.y * scale,
                                rotation: attachment_def.rotation,
                                scale_x: attachment_def.scale_x,
                                scale_y: attachment_def.scale_y,
                                width: attachment_def.width * scale,
                                height: attachment_def.height * scale,
                            }),
                            "point" => AttachmentData::Point(PointAttachmentData {
                                name: internal_name.clone(),
                                x: attachment_def.x * scale,
                                y: attachment_def.y * scale,
                                rotation: attachment_def.rotation,
                            }),
                            "mesh" => {
                                let uvs = attachment_def.uvs.ok_or_else(|| {
                                    Error::JsonInvalidMeshData {
                                        skin: skin_name.clone(),
                                        slot: slot_name.clone(),
                                        attachment: attachment_name.clone(),
                                        message: "missing 'uvs'".to_string(),
                                    }
                                })?;
                                let vertices = attachment_def.vertices.ok_or_else(|| {
                                    Error::JsonInvalidMeshData {
                                        skin: skin_name.clone(),
                                        slot: slot_name.clone(),
                                        attachment: attachment_name.clone(),
                                        message: "missing 'vertices'".to_string(),
                                    }
                                })?;
                                let triangles = attachment_def.triangles.ok_or_else(|| {
                                    Error::JsonInvalidMeshData {
                                        skin: skin_name.clone(),
                                        slot: slot_name.clone(),
                                        attachment: attachment_name.clone(),
                                        message: "missing 'triangles'".to_string(),
                                    }
                                })?;

                                if uvs.len() % 2 != 0 {
                                    return Err(Error::JsonInvalidMeshData {
                                        skin: skin_name.clone(),
                                        slot: slot_name.clone(),
                                        attachment: attachment_name.clone(),
                                        message: "uvs length must be even".to_string(),
                                    });
                                }
                                let vertex_count = uvs.len() / 2;
                                let packed_vertices = if vertices.len() == vertex_count * 2 {
                                    let mut packed = Vec::with_capacity(vertex_count);
                                    for i in 0..vertex_count {
                                        packed.push([
                                            vertices[i * 2] * scale,
                                            vertices[i * 2 + 1] * scale,
                                        ]);
                                    }
                                    crate::MeshVertices::Unweighted(packed)
                                } else {
                                    let weights = parse_weighted_mesh_vertices(
                                        &vertices,
                                        vertex_count,
                                        bones.len(),
                                        scale,
                                        skin_name.as_str(),
                                        slot_name.as_str(),
                                        attachment_name.as_str(),
                                    )?;
                                    crate::MeshVertices::Weighted(weights)
                                };

                                let mut packed_uvs = Vec::with_capacity(vertex_count);
                                for i in 0..vertex_count {
                                    packed_uvs.push([uvs[i * 2], uvs[i * 2 + 1]]);
                                }

                                if triangles.iter().any(|&i| i as usize >= vertex_count) {
                                    return Err(Error::JsonInvalidMeshData {
                                        skin: skin_name.clone(),
                                        slot: slot_name.clone(),
                                        attachment: attachment_name.clone(),
                                        message: "triangle index out of range".to_string(),
                                    });
                                }

                                AttachmentData::Mesh(MeshAttachmentData {
                                    vertex_id: crate::ids::next_vertex_attachment_id(),
                                    name: internal_name.clone(),
                                    path,
                                    timeline_skin: skin_name.clone(),
                                    timeline_attachment: attachment_name.clone(),
                                    timeline_slots: Vec::new(),
                                    sequence: sequence.clone(),
                                    color: attachment_color,
                                    vertices: packed_vertices,
                                    uvs: packed_uvs,
                                    triangles,
                                })
                            }
                            "path" => {
                                let vertex_count =
                                    attachment_def.vertex_count.ok_or_else(|| {
                                        Error::JsonInvalidPathData {
                                            skin: skin_name.clone(),
                                            slot: slot_name.clone(),
                                            attachment: attachment_name.clone(),
                                            message: "missing 'vertexCount'".to_string(),
                                        }
                                    })?;
                                let vertices = attachment_def.vertices.ok_or_else(|| {
                                    Error::JsonInvalidPathData {
                                        skin: skin_name.clone(),
                                        slot: slot_name.clone(),
                                        attachment: attachment_name.clone(),
                                        message: "missing 'vertices'".to_string(),
                                    }
                                })?;

                                let packed_vertices = if vertices.len() == vertex_count * 2 {
                                    let mut packed = Vec::with_capacity(vertex_count);
                                    for i in 0..vertex_count {
                                        packed.push([
                                            vertices[i * 2] * scale,
                                            vertices[i * 2 + 1] * scale,
                                        ]);
                                    }
                                    crate::MeshVertices::Unweighted(packed)
                                } else {
                                    let weights = parse_weighted_mesh_vertices(
                                        &vertices,
                                        vertex_count,
                                        bones.len(),
                                        scale,
                                        skin_name.as_str(),
                                        slot_name.as_str(),
                                        attachment_name.as_str(),
                                    )?;
                                    crate::MeshVertices::Weighted(weights)
                                };

                                AttachmentData::Path(PathAttachmentData {
                                    vertex_id: crate::ids::next_vertex_attachment_id(),
                                    name: internal_name.clone(),
                                    vertices: packed_vertices,
                                    lengths: attachment_def
                                        .lengths
                                        .unwrap_or_default()
                                        .into_iter()
                                        .map(|v| v * scale)
                                        .collect(),
                                    closed: attachment_def.closed,
                                    constant_speed: attachment_def.constant_speed,
                                })
                            }
                            "boundingbox" => {
                                let vertex_count =
                                    attachment_def.vertex_count.ok_or_else(|| {
                                        Error::JsonInvalidPathData {
                                            skin: skin_name.clone(),
                                            slot: slot_name.clone(),
                                            attachment: attachment_name.clone(),
                                            message: "missing 'vertexCount'".to_string(),
                                        }
                                    })?;
                                let vertices = attachment_def.vertices.ok_or_else(|| {
                                    Error::JsonInvalidPathData {
                                        skin: skin_name.clone(),
                                        slot: slot_name.clone(),
                                        attachment: attachment_name.clone(),
                                        message: "missing 'vertices'".to_string(),
                                    }
                                })?;

                                let packed_vertices = if vertices.len() == vertex_count * 2 {
                                    let mut packed = Vec::with_capacity(vertex_count);
                                    for i in 0..vertex_count {
                                        packed.push([
                                            vertices[i * 2] * scale,
                                            vertices[i * 2 + 1] * scale,
                                        ]);
                                    }
                                    crate::MeshVertices::Unweighted(packed)
                                } else {
                                    let weights = parse_weighted_mesh_vertices(
                                        &vertices,
                                        vertex_count,
                                        bones.len(),
                                        scale,
                                        skin_name.as_str(),
                                        slot_name.as_str(),
                                        attachment_name.as_str(),
                                    )?;
                                    crate::MeshVertices::Weighted(weights)
                                };

                                AttachmentData::BoundingBox(BoundingBoxAttachmentData {
                                    vertex_id: crate::ids::next_vertex_attachment_id(),
                                    name: internal_name.clone(),
                                    vertices: packed_vertices,
                                })
                            }
                            "clipping" => {
                                let vertex_count =
                                    attachment_def.vertex_count.ok_or_else(|| {
                                        Error::JsonInvalidPathData {
                                            skin: skin_name.clone(),
                                            slot: slot_name.clone(),
                                            attachment: attachment_name.clone(),
                                            message: "missing 'vertexCount'".to_string(),
                                        }
                                    })?;
                                let vertices = attachment_def.vertices.ok_or_else(|| {
                                    Error::JsonInvalidPathData {
                                        skin: skin_name.clone(),
                                        slot: slot_name.clone(),
                                        attachment: attachment_name.clone(),
                                        message: "missing 'vertices'".to_string(),
                                    }
                                })?;

                                let end_slot = match attachment_def.end.as_deref() {
                                    None => None,
                                    Some(end_name) => {
                                        Some(*slot_index.get(end_name).ok_or_else(|| {
                                            Error::JsonInvalidPathData {
                                                skin: skin_name.clone(),
                                                slot: slot_name.clone(),
                                                attachment: attachment_name.clone(),
                                                message: format!(
                                                    "unknown clipping end slot: {end_name}"
                                                ),
                                            }
                                        })?)
                                    }
                                };

                                let packed_vertices = if vertices.len() == vertex_count * 2 {
                                    let mut packed = Vec::with_capacity(vertex_count);
                                    for i in 0..vertex_count {
                                        packed.push([
                                            vertices[i * 2] * scale,
                                            vertices[i * 2 + 1] * scale,
                                        ]);
                                    }
                                    crate::MeshVertices::Unweighted(packed)
                                } else {
                                    let weights = parse_weighted_mesh_vertices(
                                        &vertices,
                                        vertex_count,
                                        bones.len(),
                                        scale,
                                        skin_name.as_str(),
                                        slot_name.as_str(),
                                        attachment_name.as_str(),
                                    )?;
                                    crate::MeshVertices::Weighted(weights)
                                };

                                AttachmentData::Clipping(ClippingAttachmentData {
                                    vertex_id: crate::ids::next_vertex_attachment_id(),
                                    name: internal_name.clone(),
                                    vertices: packed_vertices,
                                    end_slot,
                                    convex: attachment_def.convex,
                                    inverse: attachment_def.inverse,
                                })
                            }
                            "linkedmesh" => {
                                let source = attachment_def
                                    .source
                                    .clone()
                                    .or_else(|| attachment_def.parent.clone())
                                    .ok_or_else(|| Error::JsonInvalidMeshData {
                                        skin: skin_name.clone(),
                                        slot: slot_name.clone(),
                                        attachment: attachment_name.clone(),
                                        message: "linkedmesh missing 'source'".to_string(),
                                    })?;
                                let source_slot_index = match attachment_def.slot.as_deref() {
                                        Some(source_slot_name) => *slot_index
                                            .get(source_slot_name)
                                            .ok_or_else(|| Error::JsonInvalidMeshData {
                                                skin: skin_name.clone(),
                                                slot: slot_name.clone(),
                                                attachment: attachment_name.clone(),
                                                message: format!(
                                                    "linkedmesh source slot not found: {source_slot_name}"
                                                ),
                                            })?,
                                        None => s_index,
                                    };
                                if source.is_empty() {
                                    return Err(Error::JsonInvalidMeshData {
                                        skin: skin_name.clone(),
                                        slot: slot_name.clone(),
                                        attachment: attachment_name.clone(),
                                        message: "linkedmesh missing 'source'".to_string(),
                                    });
                                }
                                let parent_skin = attachment_def
                                    .skin
                                    .as_deref()
                                    .filter(|s| !s.is_empty())
                                    .map(|s| s.to_string());
                                let inherit_deform = attachment_def.timelines.unwrap_or(true);
                                pending_linked_meshes.push(PendingLinkedMesh {
                                    skin: skin_name.clone(),
                                    slot_index: s_index,
                                    attachment_name: attachment_name.clone(),
                                    source: source.clone(),
                                    parent_skin: parent_skin.clone(),
                                    source_slot_index,
                                    inherit_deform,
                                });

                                AttachmentData::Mesh(MeshAttachmentData {
                                    vertex_id: crate::ids::next_vertex_attachment_id(),
                                    name: internal_name.clone(),
                                    path,
                                    timeline_skin: if inherit_deform {
                                        parent_skin.unwrap_or_else(|| "default".to_string())
                                    } else {
                                        skin_name.clone()
                                    },
                                    timeline_attachment: if inherit_deform {
                                        source.clone()
                                    } else {
                                        attachment_name.clone()
                                    },
                                    timeline_slots: Vec::new(),
                                    sequence: sequence.clone(),
                                    color: attachment_color,
                                    vertices: crate::MeshVertices::Unweighted(Vec::new()),
                                    uvs: Vec::new(),
                                    triangles: Vec::new(),
                                })
                            }
                            _ => unreachable!(),
                        };

                        attachments[s_index].insert(attachment_name, attachment);
                    }
                }

                let mut bones_in_skin = Vec::with_capacity(skin_bones.len());
                for bone_name in skin_bones {
                    let idx =
                        *bone_index
                            .get(&bone_name)
                            .ok_or_else(|| Error::JsonUnknownSkinBone {
                                skin: skin_name.clone(),
                                bone: bone_name.clone(),
                            })?;
                    bones_in_skin.push(idx);
                }

                let skin_name_key = skin_name.clone();
                skins.insert(
                    skin_name_key.clone(),
                    SkinData {
                        name: skin_name,
                        attachments,
                        bones: bones_in_skin,
                        ik_constraints: Vec::new(),
                        transform_constraints: Vec::new(),
                        path_constraints: Vec::new(),
                        physics_constraints: Vec::new(),
                        slider_constraints: Vec::new(),
                    },
                );
                if !skin_ik.is_empty()
                    || !skin_transform.is_empty()
                    || !skin_path.is_empty()
                    || !skin_physics.is_empty()
                    || !skin_slider.is_empty()
                {
                    pending_skin_constraints.insert(
                        skin_name_key,
                        PendingSkinConstraints {
                            ik: skin_ik,
                            transform: skin_transform,
                            path: skin_path,
                            physics: skin_physics,
                            slider: skin_slider,
                        },
                    );
                }
                Ok(())
            };

            match skins_def {
                SkinsDef::Map(map) => {
                    for (skin_name, skin_slots) in map {
                        add_skin(
                            skin_name,
                            skin_slots,
                            Vec::new(),
                            Vec::new(),
                            Vec::new(),
                            Vec::new(),
                            Vec::new(),
                            Vec::new(),
                        )?;
                    }
                }
                SkinsDef::Array(list) => {
                    for skin in list {
                        add_skin(
                            skin.name,
                            skin.attachments,
                            skin.bones,
                            skin.ik,
                            skin.transform,
                            skin.path,
                            skin.physics,
                            skin.slider,
                        )?;
                    }
                }
            }

            let mut remaining = pending_linked_meshes;
            let mut passes_left = remaining.len().max(1);
            while !remaining.is_empty() && passes_left > 0 {
                passes_left -= 1;
                let mut next = Vec::new();
                let mut resolved_any = false;

                for pending in remaining {
                    let Some((parent_skin_name, parent_attachment_name)) =
                        resolve_linked_mesh_parent(
                            &skins,
                            pending.source_slot_index,
                            pending.parent_skin.as_deref(),
                            pending.source.as_str(),
                        )
                    else {
                        return Err(Error::JsonInvalidMeshData {
                            skin: pending.skin.clone(),
                            slot: slots
                                .get(pending.slot_index)
                                .map(|s| s.name.clone())
                                .unwrap_or_else(|| "<unknown>".to_string()),
                            attachment: pending.attachment_name.clone(),
                            message: format!(
                                "linkedmesh parent attachment not found: {}",
                                pending.source
                            ),
                        });
                    };

                    let Some(parent_skin) = skins.get(&parent_skin_name) else {
                        return Err(Error::JsonInvalidMeshData {
                            skin: pending.skin.clone(),
                            slot: slots
                                .get(pending.slot_index)
                                .map(|s| s.name.clone())
                                .unwrap_or_else(|| "<unknown>".to_string()),
                            attachment: pending.attachment_name.clone(),
                            message: format!(
                                "linkedmesh parent skin not found: {parent_skin_name}"
                            ),
                        });
                    };
                    let Some(parent_attachment) =
                        parent_skin.attachment(pending.source_slot_index, &parent_attachment_name)
                    else {
                        return Err(Error::JsonInvalidMeshData {
                            skin: pending.skin.clone(),
                            slot: slots
                                .get(pending.slot_index)
                                .map(|s| s.name.clone())
                                .unwrap_or_else(|| "<unknown>".to_string()),
                            attachment: pending.attachment_name.clone(),
                            message: format!(
                                "linkedmesh parent attachment not found: {}",
                                parent_attachment_name
                            ),
                        });
                    };
                    let AttachmentData::Mesh(parent_mesh) = parent_attachment else {
                        return Err(Error::JsonInvalidMeshData {
                            skin: pending.skin.clone(),
                            slot: slots
                                .get(pending.slot_index)
                                .map(|s| s.name.clone())
                                .unwrap_or_else(|| "<unknown>".to_string()),
                            attachment: pending.attachment_name.clone(),
                            message: "linkedmesh parent attachment is not a mesh".to_string(),
                        });
                    };

                    if parent_mesh.triangles.is_empty() {
                        next.push(pending);
                        continue;
                    }

                    let parent_vertices = parent_mesh.vertices.clone();
                    let parent_uvs = parent_mesh.uvs.clone();
                    let parent_triangles = parent_mesh.triangles.clone();
                    let timeline_skin = if pending.inherit_deform {
                        parent_skin_name.clone()
                    } else {
                        pending.skin.clone()
                    };
                    let timeline_attachment = if pending.inherit_deform {
                        parent_attachment_name.clone()
                    } else {
                        pending.attachment_name.clone()
                    };

                    {
                        let Some(linked_skin) = skins.get_mut(&pending.skin) else {
                            continue;
                        };
                        let Some(slot_map) = linked_skin.attachments.get_mut(pending.slot_index)
                        else {
                            continue;
                        };
                        let Some(linked_attachment) = slot_map.get_mut(&pending.attachment_name)
                        else {
                            continue;
                        };
                        let AttachmentData::Mesh(linked_mesh) = linked_attachment else {
                            continue;
                        };

                        linked_mesh.vertices = parent_vertices;
                        linked_mesh.uvs = parent_uvs;
                        linked_mesh.triangles = parent_triangles;
                        linked_mesh.timeline_skin = timeline_skin.clone();
                        linked_mesh.timeline_attachment = timeline_attachment.clone();
                    }

                    if pending.inherit_deform && pending.slot_index != pending.source_slot_index {
                        let Some(parent_skin) = skins.get_mut(&timeline_skin) else {
                            continue;
                        };
                        let Some(parent_slot_map) =
                            parent_skin.attachments.get_mut(pending.source_slot_index)
                        else {
                            continue;
                        };
                        let Some(parent_attachment) =
                            parent_slot_map.get_mut(timeline_attachment.as_str())
                        else {
                            continue;
                        };
                        if let AttachmentData::Mesh(parent_mesh) = parent_attachment
                            && !parent_mesh.timeline_slots.contains(&pending.slot_index)
                        {
                            parent_mesh.timeline_slots.push(pending.slot_index);
                        }
                    }
                    resolved_any = true;
                }

                if !resolved_any && !next.is_empty() {
                    let pending = &next[0];
                    let parent_skin_name = pending
                        .parent_skin
                        .as_deref()
                        .filter(|s| !s.is_empty())
                        .unwrap_or("default")
                        .to_string();
                    return Err(Error::JsonInvalidMeshData {
                        skin: pending.skin.clone(),
                        slot: slots
                            .get(pending.slot_index)
                            .map(|s| s.name.clone())
                            .unwrap_or_else(|| "<unknown>".to_string()),
                        attachment: pending.attachment_name.clone(),
                        message: format!(
                            "linkedmesh resolution stalled (parent may be missing/unresolved): skin={parent_skin_name}, parent={}",
                            pending.source
                        ),
                    });
                }

                remaining = next;
            }

            // Resolve skin constraint membership (requires constraint indices).
            // This is deferred because constraints are parsed after skins.
            // The vectors are filled later, after `ik_constraint_index`/etc are built.
            // (see below)
        }

        let events = root
            .events
            .into_iter()
            .map(|(name, def)| {
                let has_audio = !def.audio_path.is_empty();
                let (volume, balance) = if has_audio {
                    (def.volume, def.balance)
                } else {
                    (1.0, 0.0)
                };
                (
                    name.clone(),
                    EventData {
                        name,
                        int_value: def.int_value,
                        float_value: def.float_value,
                        string: def.string_value,
                        audio_path: def.audio_path,
                        volume,
                        balance,
                    },
                )
            })
            .collect::<HashMap<_, _>>();

        let (ik_defs, transform_defs, path_defs, physics_defs, slider_defs) =
            if let Some(constraints) = root.constraints {
                let mut ik = Vec::new();
                let mut transform = Vec::new();
                let mut path = Vec::new();
                let mut physics = Vec::new();
                let mut slider = Vec::new();
                for (order, c) in constraints.into_iter().enumerate() {
                    let order = order as i32;
                    match c {
                        ConstraintDef::Ik(mut d) => {
                            d.order = order;
                            ik.push(d);
                        }
                        ConstraintDef::Transform(mut d) => {
                            d.order = order;
                            transform.push(d);
                        }
                        ConstraintDef::Path(mut d) => {
                            d.order = order;
                            path.push(d);
                        }
                        ConstraintDef::Physics(mut d) => {
                            d.order = order;
                            physics.push(d);
                        }
                        ConstraintDef::Slider(mut d) => {
                            d.order = order;
                            slider.push(d);
                        }
                    }
                }
                (ik, transform, path, physics, slider)
            } else {
                (
                    root.ik.unwrap_or_default(),
                    root.transform.unwrap_or_default(),
                    root.path.unwrap_or_default(),
                    root.physics.unwrap_or_default(),
                    root.slider.unwrap_or_default(),
                )
            };

        let mut ik_constraints = Vec::new();
        for ik in ik_defs {
            let mut bones_indices = Vec::with_capacity(ik.bones.len());
            for bone_name in ik.bones {
                let Some(&idx) = bone_index.get(&bone_name) else {
                    return Err(Error::JsonUnknownAnimationBone {
                        animation: "<ik>".to_string(),
                        bone: bone_name,
                    });
                };
                bones_indices.push(idx);
            }
            let target =
                *bone_index
                    .get(&ik.target)
                    .ok_or_else(|| Error::JsonUnknownAnimationBone {
                        animation: "<ik>".to_string(),
                        bone: ik.target.clone(),
                    })?;
            let bend_direction = if ik.bend_positive { 1 } else { -1 };
            ik_constraints.push(crate::IkConstraintData {
                name: ik.name,
                bones: bones_indices,
                target,
                scale_y_mode: ik
                    .scale_y
                    .as_deref()
                    .map(crate::ScaleYMode::from_name)
                    .unwrap_or_default(),
                mix: ik.mix,
                softness: ik.softness * scale,
                compress: ik.compress,
                stretch: ik.stretch,
                bend_direction,
                order: ik.order,
                skin_required: ik.skin_required,
            });
        }
        let ik_constraint_index = ik_constraints
            .iter()
            .enumerate()
            .map(|(i, c)| (c.name.clone(), i))
            .collect::<HashMap<_, _>>();

        let mut transform_constraints = Vec::new();
        for c in transform_defs {
            let mut bones_indices = Vec::with_capacity(c.bones.len());
            for bone_name in c.bones {
                let Some(&idx) = bone_index.get(&bone_name) else {
                    return Err(Error::JsonUnknownAnimationBone {
                        animation: "<transform>".to_string(),
                        bone: bone_name,
                    });
                };
                bones_indices.push(idx);
            }
            let source =
                *bone_index
                    .get(&c.source)
                    .ok_or_else(|| Error::JsonUnknownAnimationBone {
                        animation: "<transform>".to_string(),
                        bone: c.source.clone(),
                    })?;

            let mut used_rotate = false;
            let mut used_x = false;
            let mut used_y = false;
            let mut used_scale_x = false;
            let mut used_scale_y = false;
            let mut used_shear_y = false;

            let mut properties = Vec::<TransformFromProperty>::new();
            if let Some(props) = c.properties {
                for (from_name, from_def) in props {
                    let Some(from_prop) = TransformProperty::from_json_name(from_name.as_str())
                    else {
                        return Err(Error::JsonParse {
                            message: format!(
                                "invalid transform constraint from property: {from_name}"
                            ),
                        });
                    };
                    let from_scale =
                        if matches!(from_prop, TransformProperty::X | TransformProperty::Y) {
                            scale
                        } else {
                            1.0
                        };
                    let mut to = Vec::<TransformToProperty>::new();
                    if let Some(to_map) = from_def.to {
                        for (to_name, to_def) in to_map {
                            let Some(to_prop) = TransformProperty::from_json_name(to_name.as_str())
                            else {
                                return Err(Error::JsonParse {
                                    message: format!(
                                        "invalid transform constraint to property: {to_name}"
                                    ),
                                });
                            };
                            match to_prop {
                                TransformProperty::Rotate => used_rotate = true,
                                TransformProperty::X => used_x = true,
                                TransformProperty::Y => used_y = true,
                                TransformProperty::ScaleX => used_scale_x = true,
                                TransformProperty::ScaleY => used_scale_y = true,
                                TransformProperty::ShearY => used_shear_y = true,
                            }
                            let to_scale =
                                if matches!(to_prop, TransformProperty::X | TransformProperty::Y) {
                                    scale
                                } else {
                                    1.0
                                };
                            to.push(TransformToProperty {
                                property: to_prop,
                                offset: to_def.offset * to_scale,
                                max: to_def.max * to_scale,
                                scale: to_def.scale * to_scale / from_scale,
                            });
                        }
                    }
                    if !to.is_empty() {
                        properties.push(TransformFromProperty {
                            property: from_prop,
                            offset: from_def.offset * from_scale,
                            to,
                        });
                    }
                }
            }

            let mix_rotate = if used_rotate {
                c.mix_rotate.unwrap_or(1.0)
            } else {
                0.0
            };
            let mix_x = if used_x { c.mix_x.unwrap_or(1.0) } else { 0.0 };
            let mix_y = if used_y {
                c.mix_y.unwrap_or(mix_x)
            } else {
                0.0
            };
            let mix_scale_x = if used_scale_x {
                c.mix_scale_x.unwrap_or(1.0)
            } else {
                0.0
            };
            let mix_scale_y = if used_scale_y {
                c.mix_scale_y.unwrap_or(mix_scale_x)
            } else {
                0.0
            };
            let mix_shear_y = if used_shear_y {
                c.mix_shear_y.unwrap_or(1.0)
            } else {
                0.0
            };

            transform_constraints.push(crate::TransformConstraintData {
                name: c.name,
                order: c.order,
                skin_required: c.skin_required,
                bones: bones_indices,
                source,
                local_source: c.local_source,
                local_target: c.local_target,
                additive: c.additive,
                clamp: c.clamp,
                offsets: [
                    c.rotation,
                    c.x * scale,
                    c.y * scale,
                    c.scale_x,
                    c.scale_y,
                    c.shear_y,
                ],
                properties,
                mix_rotate,
                mix_x,
                mix_y,
                mix_scale_x,
                mix_scale_y,
                mix_shear_y,
            });
        }
        let transform_constraint_index = transform_constraints
            .iter()
            .enumerate()
            .map(|(i, c)| (c.name.clone(), i))
            .collect::<HashMap<_, _>>();

        let mut path_constraints = Vec::new();
        for c in path_defs {
            let mut bones_indices = Vec::with_capacity(c.bones.len());
            for bone_name in c.bones {
                let Some(&idx) = bone_index.get(&bone_name) else {
                    return Err(Error::JsonUnknownPathConstraintBone {
                        constraint: c.name.clone(),
                        bone: bone_name,
                    });
                };
                bones_indices.push(idx);
            }

            let target = *slot_index.get(&c.target).ok_or_else(|| {
                Error::JsonUnknownPathConstraintTargetSlot {
                    constraint: c.name.clone(),
                    slot: c.target.clone(),
                }
            })?;

            let position_mode = match c.position_mode.as_deref().unwrap_or("percent") {
                "fixed" => crate::PositionMode::Fixed,
                "percent" => crate::PositionMode::Percent,
                other => {
                    return Err(Error::JsonUnsupportedPathConstraintMode {
                        constraint: c.name.clone(),
                        field: "positionMode".to_string(),
                        value: other.to_string(),
                    });
                }
            };

            let spacing_mode = match c.spacing_mode.as_deref().unwrap_or("length") {
                "length" => crate::SpacingMode::Length,
                "fixed" => crate::SpacingMode::Fixed,
                "percent" => crate::SpacingMode::Percent,
                "proportional" => crate::SpacingMode::Proportional,
                other => {
                    return Err(Error::JsonUnsupportedPathConstraintMode {
                        constraint: c.name.clone(),
                        field: "spacingMode".to_string(),
                        value: other.to_string(),
                    });
                }
            };

            let rotate_mode = match c.rotate_mode.as_deref().unwrap_or("tangent") {
                "tangent" => crate::RotateMode::Tangent,
                "chain" => crate::RotateMode::Chain,
                "chainScale" => crate::RotateMode::ChainScale,
                other => {
                    return Err(Error::JsonUnsupportedPathConstraintMode {
                        constraint: c.name.clone(),
                        field: "rotateMode".to_string(),
                        value: other.to_string(),
                    });
                }
            };

            let mix_y = c.mix_y.unwrap_or(c.mix_x);
            let position = if position_mode == crate::PositionMode::Fixed {
                c.position * scale
            } else {
                c.position
            };
            let spacing = if spacing_mode == crate::SpacingMode::Length
                || spacing_mode == crate::SpacingMode::Fixed
            {
                c.spacing * scale
            } else {
                c.spacing
            };
            path_constraints.push(crate::PathConstraintData {
                name: c.name,
                order: c.order,
                bones: bones_indices,
                target,
                position_mode,
                spacing_mode,
                rotate_mode,
                offset_rotation: c.offset_rotation,
                position,
                spacing,
                mix_rotate: c.mix_rotate,
                mix_x: c.mix_x,
                mix_y,
                skin_required: c.skin_required,
            });
        }
        let path_constraint_index = path_constraints
            .iter()
            .enumerate()
            .map(|(i, c)| (c.name.clone(), i))
            .collect::<HashMap<_, _>>();

        let mut physics_constraints = Vec::new();
        for c in physics_defs {
            let bone = *bone_index.get(&c.bone).ok_or_else(|| {
                Error::JsonUnknownPhysicsConstraintBone {
                    constraint: c.name.clone(),
                    bone: c.bone.clone(),
                }
            })?;

            let limit = c.limit.unwrap_or(5000.0) * scale;
            let fps = c.fps.unwrap_or(60) as f32;
            let step = 1.0 / fps;

            let inertia = c.inertia.unwrap_or(0.5);
            let strength = c.strength.unwrap_or(100.0);
            let damping = c.damping.unwrap_or(0.85);
            let mass = c.mass.unwrap_or(1.0);
            let mass_inverse = 1.0 / mass;
            let wind = c.wind.unwrap_or(0.0);
            let gravity = c.gravity.unwrap_or(0.0);
            let mix = c.mix.unwrap_or(1.0);

            physics_constraints.push(crate::PhysicsConstraintData {
                name: c.name,
                order: c.order,
                skin_required: c.skin_required,
                bone,
                x: c.x,
                y: c.y,
                rotate: c.rotate,
                scale_x: c.scale_x,
                scale_y_mode: c
                    .scale_y
                    .as_deref()
                    .map(crate::ScaleYMode::from_name)
                    .unwrap_or_default(),
                shear_x: c.shear_x,
                limit,
                step,
                inertia,
                strength,
                damping,
                mass_inverse,
                wind,
                gravity,
                mix,
                inertia_global: c.inertia_global,
                strength_global: c.strength_global,
                damping_global: c.damping_global,
                mass_global: c.mass_global,
                wind_global: c.wind_global,
                gravity_global: c.gravity_global,
                mix_global: c.mix_global,
            });
        }
        let physics_constraint_index = physics_constraints
            .iter()
            .enumerate()
            .map(|(i, c)| (c.name.clone(), i))
            .collect::<HashMap<_, _>>();

        let mut pending_slider_animations: Vec<(usize, String)> = Vec::new();
        let mut slider_constraints = Vec::new();
        for c in slider_defs {
            let bone = match c.bone.as_deref() {
                None => None,
                Some(name) => Some(*bone_index.get(name).ok_or_else(|| {
                    Error::JsonUnknownSliderConstraintBone {
                        constraint: c.name.clone(),
                        bone: name.to_string(),
                    }
                })?),
            };

            let property = c
                .property
                .as_deref()
                .and_then(crate::TransformProperty::from_json_name);
            let property_scale = match property {
                Some(crate::TransformProperty::X | crate::TransformProperty::Y) => scale,
                _ => 1.0,
            };

            let idx = slider_constraints.len();
            if let Some(animation_name) = c.animation.as_deref() {
                pending_slider_animations.push((idx, animation_name.to_string()));
            }
            slider_constraints.push(SliderConstraintData {
                name: c.name,
                order: c.order,
                skin_required: c.skin_required,
                setup_time: c.time,
                setup_mix: c.mix,
                additive: c.additive,
                looped: c.looped,
                bone,
                property,
                property_from: c.from * property_scale,
                to: c.to,
                scale: c.scale / property_scale,
                local: c.local,
                animation: None,
            });
        }
        let slider_constraint_index = slider_constraints
            .iter()
            .enumerate()
            .map(|(i, c)| (c.name.clone(), i))
            .collect::<HashMap<_, _>>();

        for (skin_name, pending) in pending_skin_constraints {
            let Some(skin) = skins.get_mut(&skin_name) else {
                continue;
            };

            for constraint_name in pending.ik {
                let idx = *ik_constraint_index.get(&constraint_name).ok_or_else(|| {
                    Error::JsonUnknownSkinConstraint {
                        skin: skin_name.clone(),
                        constraint: constraint_name.clone(),
                        kind: "ik".to_string(),
                    }
                })?;
                skin.ik_constraints.push(idx);
            }
            for constraint_name in pending.transform {
                let idx = *transform_constraint_index
                    .get(&constraint_name)
                    .ok_or_else(|| Error::JsonUnknownSkinConstraint {
                        skin: skin_name.clone(),
                        constraint: constraint_name.clone(),
                        kind: "transform".to_string(),
                    })?;
                skin.transform_constraints.push(idx);
            }
            for constraint_name in pending.path {
                let idx = *path_constraint_index.get(&constraint_name).ok_or_else(|| {
                    Error::JsonUnknownSkinConstraint {
                        skin: skin_name.clone(),
                        constraint: constraint_name.clone(),
                        kind: "path".to_string(),
                    }
                })?;
                skin.path_constraints.push(idx);
            }
            for constraint_name in pending.physics {
                let idx = *physics_constraint_index
                    .get(&constraint_name)
                    .ok_or_else(|| Error::JsonUnknownSkinConstraint {
                        skin: skin_name.clone(),
                        constraint: constraint_name.clone(),
                        kind: "physics".to_string(),
                    })?;
                skin.physics_constraints.push(idx);
            }
            for constraint_name in pending.slider {
                let idx = *slider_constraint_index
                    .get(&constraint_name)
                    .ok_or_else(|| Error::JsonUnknownSkinConstraint {
                        skin: skin_name.clone(),
                        constraint: constraint_name.clone(),
                        kind: "slider".to_string(),
                    })?;
                skin.slider_constraints.push(idx);
            }
        }

        let mut animations = Vec::new();
        let mut animation_index = HashMap::new();
        for (name, def_value) in root.animations {
            let def: AnimationDef = from_json_value(&def_value, &format!("animation '{name}'"))?;
            let events_def = def.events;
            let bones_def = def.bones;
            let attachments_def = def.attachments;
            let slots_def = def.slots;
            let draw_order_def = def.draw_order;
            let draw_order_folder_def = def.draw_order_folder;
            let ik_def = def.ik;
            let transform_def = def.transform;
            let path_def = def.path;
            let physics_def = def.physics;
            let slider_def = def.slider;

            let mut duration: f32 = 0.0;

            let timeline = if let Some(keys) = events_def {
                // Preserve file order for events that share the same `time` (matches upstream runtimes).
                let mut event_frames = Vec::with_capacity(keys.len());
                for (source_index, k) in keys.into_iter().enumerate() {
                    let event_data =
                        events
                            .get(k.name.as_str())
                            .ok_or_else(|| Error::JsonUnknownEvent {
                                animation: name.clone(),
                                event: k.name.clone(),
                            })?;
                    let has_audio = !event_data.audio_path.is_empty();
                    let time = k.time.unwrap_or(0.0);
                    duration = duration.max(time);
                    event_frames.push((
                        source_index,
                        Event {
                            time,
                            name: k.name,
                            int_value: k.int_value.unwrap_or(event_data.int_value),
                            float_value: k.float_value.unwrap_or(event_data.float_value),
                            string: k.string_value.unwrap_or_else(|| event_data.string.clone()),
                            audio_path: event_data.audio_path.clone(),
                            volume: if has_audio {
                                k.volume.unwrap_or(1.0)
                            } else {
                                1.0
                            },
                            balance: if has_audio {
                                k.balance.unwrap_or(0.0)
                            } else {
                                0.0
                            },
                        },
                    ));
                }
                event_frames.sort_by(|(ia, a), (ib, b)| a.time.total_cmp(&b.time).then(ia.cmp(ib)));
                Some(EventTimeline {
                    events: event_frames.into_iter().map(|(_, e)| e).collect(),
                })
            } else {
                None
            };

            let mut bone_timelines = Vec::new();
            for (bone_name, bone_anim) in bones_def {
                let bone_data_index = bone_index.get(&bone_name).copied().ok_or_else(|| {
                    Error::JsonUnknownAnimationBone {
                        animation: name.clone(),
                        bone: bone_name.clone(),
                    }
                })?;

                if let Some(keys) = bone_anim.rotate {
                    let curve_context =
                        format!("animation '{name}'.bones.{bone_name}.rotate curve");
                    let mut frames = Vec::with_capacity(keys.len());
                    for k in keys {
                        frames.push(RotateFrame {
                            time: k.time.unwrap_or(0.0),
                            angle: k.angle.or(k.value).unwrap_or(0.0),
                            curve: parse_curve_1(k.curve.as_ref(), 1.0, &curve_context)?,
                        });
                    }
                    frames.sort_by(|a, b| a.time.total_cmp(&b.time));
                    if let Some(last) = frames.last() {
                        duration = duration.max(last.time);
                    }
                    bone_timelines.push(BoneTimeline::Rotate(RotateTimeline {
                        bone_index: bone_data_index,
                        frames,
                    }));
                }

                if let Some(keys) = bone_anim.translate {
                    let curve_context =
                        format!("animation '{name}'.bones.{bone_name}.translate curve");
                    let mut frames = Vec::with_capacity(keys.len());
                    for k in keys {
                        frames.push(Vec2Frame {
                            time: k.time.unwrap_or(0.0),
                            x: k.x.unwrap_or(0.0) * scale,
                            y: k.y.unwrap_or(0.0) * scale,
                            curve: parse_curve_n(k.curve.as_ref(), [scale, scale], &curve_context)?,
                        });
                    }
                    frames.sort_by(|a, b| a.time.total_cmp(&b.time));
                    if let Some(last) = frames.last() {
                        duration = duration.max(last.time);
                    }
                    bone_timelines.push(BoneTimeline::Translate(TranslateTimeline {
                        bone_index: bone_data_index,
                        frames,
                    }));
                }

                if let Some(keys) = bone_anim.translate_x {
                    let curve_context =
                        format!("animation '{name}'.bones.{bone_name}.translatex curve");
                    let mut frames = Vec::with_capacity(keys.len());
                    for k in keys {
                        frames.push(FloatFrame {
                            time: k.time.unwrap_or(0.0),
                            value: k.value.unwrap_or(0.0) * scale,
                            curve: parse_curve_1(k.curve.as_ref(), scale, &curve_context)?,
                        });
                    }
                    frames.sort_by(|a, b| a.time.total_cmp(&b.time));
                    if let Some(last) = frames.last() {
                        duration = duration.max(last.time);
                    }
                    bone_timelines.push(BoneTimeline::TranslateX(TranslateXTimeline {
                        bone_index: bone_data_index,
                        frames,
                    }));
                }

                if let Some(keys) = bone_anim.translate_y {
                    let curve_context =
                        format!("animation '{name}'.bones.{bone_name}.translatey curve");
                    let mut frames = Vec::with_capacity(keys.len());
                    for k in keys {
                        frames.push(FloatFrame {
                            time: k.time.unwrap_or(0.0),
                            value: k.value.unwrap_or(0.0) * scale,
                            curve: parse_curve_1(k.curve.as_ref(), scale, &curve_context)?,
                        });
                    }
                    frames.sort_by(|a, b| a.time.total_cmp(&b.time));
                    if let Some(last) = frames.last() {
                        duration = duration.max(last.time);
                    }
                    bone_timelines.push(BoneTimeline::TranslateY(TranslateYTimeline {
                        bone_index: bone_data_index,
                        frames,
                    }));
                }

                if let Some(keys) = bone_anim.scale {
                    let curve_context = format!("animation '{name}'.bones.{bone_name}.scale curve");
                    let mut frames = Vec::with_capacity(keys.len());
                    for k in keys {
                        frames.push(Vec2Frame {
                            time: k.time.unwrap_or(0.0),
                            x: k.x.unwrap_or(1.0),
                            y: k.y.unwrap_or(1.0),
                            curve: parse_curve_n(k.curve.as_ref(), [1.0, 1.0], &curve_context)?,
                        });
                    }
                    frames.sort_by(|a, b| a.time.total_cmp(&b.time));
                    if let Some(last) = frames.last() {
                        duration = duration.max(last.time);
                    }
                    bone_timelines.push(BoneTimeline::Scale(ScaleTimeline {
                        bone_index: bone_data_index,
                        frames,
                    }));
                }

                if let Some(keys) = bone_anim.scale_x {
                    let curve_context =
                        format!("animation '{name}'.bones.{bone_name}.scalex curve");
                    let mut frames = Vec::with_capacity(keys.len());
                    for k in keys {
                        frames.push(FloatFrame {
                            time: k.time.unwrap_or(0.0),
                            value: k.value.unwrap_or(1.0),
                            curve: parse_curve_1(k.curve.as_ref(), 1.0, &curve_context)?,
                        });
                    }
                    frames.sort_by(|a, b| a.time.total_cmp(&b.time));
                    if let Some(last) = frames.last() {
                        duration = duration.max(last.time);
                    }
                    bone_timelines.push(BoneTimeline::ScaleX(ScaleXTimeline {
                        bone_index: bone_data_index,
                        frames,
                    }));
                }

                if let Some(keys) = bone_anim.scale_y {
                    let curve_context =
                        format!("animation '{name}'.bones.{bone_name}.scaley curve");
                    let mut frames = Vec::with_capacity(keys.len());
                    for k in keys {
                        frames.push(FloatFrame {
                            time: k.time.unwrap_or(0.0),
                            value: k.value.unwrap_or(1.0),
                            curve: parse_curve_1(k.curve.as_ref(), 1.0, &curve_context)?,
                        });
                    }
                    frames.sort_by(|a, b| a.time.total_cmp(&b.time));
                    if let Some(last) = frames.last() {
                        duration = duration.max(last.time);
                    }
                    bone_timelines.push(BoneTimeline::ScaleY(ScaleYTimeline {
                        bone_index: bone_data_index,
                        frames,
                    }));
                }

                if let Some(keys) = bone_anim.shear {
                    let curve_context = format!("animation '{name}'.bones.{bone_name}.shear curve");
                    let mut frames = Vec::with_capacity(keys.len());
                    for k in keys {
                        frames.push(Vec2Frame {
                            time: k.time.unwrap_or(0.0),
                            x: k.x.unwrap_or(0.0),
                            y: k.y.unwrap_or(0.0),
                            curve: parse_curve_n(k.curve.as_ref(), [1.0, 1.0], &curve_context)?,
                        });
                    }
                    frames.sort_by(|a, b| a.time.total_cmp(&b.time));
                    if let Some(last) = frames.last() {
                        duration = duration.max(last.time);
                    }
                    bone_timelines.push(BoneTimeline::Shear(ShearTimeline {
                        bone_index: bone_data_index,
                        frames,
                    }));
                }

                if let Some(keys) = bone_anim.shear_x {
                    let curve_context =
                        format!("animation '{name}'.bones.{bone_name}.shearx curve");
                    let mut frames = Vec::with_capacity(keys.len());
                    for k in keys {
                        frames.push(FloatFrame {
                            time: k.time.unwrap_or(0.0),
                            value: k.value.unwrap_or(0.0),
                            curve: parse_curve_1(k.curve.as_ref(), 1.0, &curve_context)?,
                        });
                    }
                    frames.sort_by(|a, b| a.time.total_cmp(&b.time));
                    if let Some(last) = frames.last() {
                        duration = duration.max(last.time);
                    }
                    bone_timelines.push(BoneTimeline::ShearX(ShearXTimeline {
                        bone_index: bone_data_index,
                        frames,
                    }));
                }

                if let Some(keys) = bone_anim.shear_y {
                    let curve_context =
                        format!("animation '{name}'.bones.{bone_name}.sheary curve");
                    let mut frames = Vec::with_capacity(keys.len());
                    for k in keys {
                        frames.push(FloatFrame {
                            time: k.time.unwrap_or(0.0),
                            value: k.value.unwrap_or(0.0),
                            curve: parse_curve_1(k.curve.as_ref(), 1.0, &curve_context)?,
                        });
                    }
                    frames.sort_by(|a, b| a.time.total_cmp(&b.time));
                    if let Some(last) = frames.last() {
                        duration = duration.max(last.time);
                    }
                    bone_timelines.push(BoneTimeline::ShearY(ShearYTimeline {
                        bone_index: bone_data_index,
                        frames,
                    }));
                }

                if let Some(keys) = bone_anim.inherit {
                    let mut frames = Vec::with_capacity(keys.len());
                    for k in keys {
                        frames.push(InheritFrame {
                            time: k.time.unwrap_or(0.0),
                            inherit: parse_inherit(k.inherit.as_deref()),
                        });
                    }
                    frames.sort_by(|a, b| a.time.total_cmp(&b.time));
                    if let Some(last) = frames.last() {
                        duration = duration.max(last.time);
                    }
                    bone_timelines.push(BoneTimeline::Inherit(InheritTimeline {
                        bone_index: bone_data_index,
                        frames,
                    }));
                }
            }

            let mut deform_timelines = Vec::new();
            let mut sequence_timelines = Vec::new();
            if let Some(attachments) = attachments_def {
                for (skin_name, skin_map) in attachments {
                    let skin = skins.get(&skin_name).ok_or_else(|| {
                        if skin_map
                            .values()
                            .any(|slot_map| slot_map.values().any(|t| t.sequence.is_some()))
                        {
                            Error::JsonUnknownSequenceSkin {
                                animation: name.clone(),
                                skin: skin_name.clone(),
                            }
                        } else {
                            Error::JsonUnknownDeformSkin {
                                animation: name.clone(),
                                skin: skin_name.clone(),
                            }
                        }
                    })?;

                    for (slot_name, slot_map) in skin_map {
                        let s_index = slot_index.get(&slot_name).copied().ok_or_else(|| {
                            if slot_map.values().any(|t| t.sequence.is_some()) {
                                Error::JsonUnknownSequenceSlot {
                                    animation: name.clone(),
                                    skin: skin_name.clone(),
                                    slot: slot_name.clone(),
                                }
                            } else {
                                Error::JsonUnknownDeformSlot {
                                    animation: name.clone(),
                                    skin: skin_name.clone(),
                                    slot: slot_name.clone(),
                                }
                            }
                        })?;

                        for (attachment_name, timelines) in slot_map {
                            let has_deform = timelines.deform.is_some();
                            let has_sequence = timelines.sequence.is_some();
                            if !has_deform && !has_sequence {
                                continue;
                            }

                            let Some(attachment) = skin.attachment(s_index, &attachment_name)
                            else {
                                return Err(if has_sequence {
                                    Error::JsonUnknownSequenceAttachment {
                                        animation: name.clone(),
                                        skin: skin_name.clone(),
                                        slot: slot_name.clone(),
                                        attachment: attachment_name.clone(),
                                    }
                                } else {
                                    Error::JsonUnknownDeformAttachment {
                                        animation: name.clone(),
                                        skin: skin_name.clone(),
                                        slot: slot_name.clone(),
                                        attachment: attachment_name.clone(),
                                    }
                                });
                            };

                            if let Some(keys) = timelines.deform {
                                let attachment_vertices = match attachment {
                                    AttachmentData::Mesh(mesh) => &mesh.vertices,
                                    AttachmentData::Path(path) => &path.vertices,
                                    AttachmentData::BoundingBox(bb) => &bb.vertices,
                                    AttachmentData::Clipping(clip) => &clip.vertices,
                                    _ => {
                                        return Err(Error::JsonUnsupportedDeformAttachment {
                                            animation: name.clone(),
                                            skin: skin_name.clone(),
                                            slot: slot_name.clone(),
                                            attachment: attachment_name.clone(),
                                        });
                                    }
                                };

                                let (vertex_count, setup_vertices) =
                                    vertex_attachment_deform_setup(attachment_vertices);

                                let curve_context = format!(
                                    "animation '{name}'.attachments.{skin_name}.{slot_name}.{attachment_name}.deform curve"
                                );
                                let mut frames = Vec::with_capacity(keys.len());
                                for key in keys {
                                    let time = key.time.unwrap_or(0.0);
                                    duration = duration.max(time);

                                    let curve =
                                        parse_curve_1(key.curve.as_ref(), 1.0, &curve_context)?;
                                    let vertices = build_deform_vertices(
                                        attachment_vertices,
                                        setup_vertices.as_deref(),
                                        vertex_count,
                                        key.offset.unwrap_or(0),
                                        key.vertices.as_deref(),
                                        scale,
                                        name.as_str(),
                                        skin_name.as_str(),
                                        slot_name.as_str(),
                                        attachment_name.as_str(),
                                    )?;
                                    frames.push(DeformFrame {
                                        time,
                                        vertices,
                                        curve,
                                    });
                                }
                                frames.sort_by(|a, b| a.time.total_cmp(&b.time));

                                deform_timelines.push(DeformTimeline {
                                    skin: skin_name.clone(),
                                    slot_index: s_index,
                                    attachment: attachment_name.clone(),
                                    vertex_count,
                                    setup_vertices,
                                    frames,
                                });
                            }

                            if let Some(keys) = timelines.sequence {
                                match attachment {
                                    AttachmentData::Region(_) | AttachmentData::Mesh(_) => {}
                                    _ => {
                                        return Err(Error::JsonUnsupportedSequenceAttachment {
                                            animation: name.clone(),
                                            skin: skin_name.clone(),
                                            slot: slot_name.clone(),
                                            attachment: attachment_name.clone(),
                                        });
                                    }
                                }

                                let mut frames = Vec::with_capacity(keys.len());
                                let mut last_delay = 0.0f32;
                                for key in keys {
                                    let time = key.time.unwrap_or(0.0);
                                    duration = duration.max(time);
                                    let delay = key.delay.unwrap_or(last_delay);
                                    last_delay = delay;

                                    let mode = match key.mode.as_deref().unwrap_or("hold") {
                                        "hold" => crate::SequenceMode::Hold,
                                        "once" => crate::SequenceMode::Once,
                                        "loop" => crate::SequenceMode::Loop,
                                        "pingpong" => crate::SequenceMode::PingPong,
                                        "onceReverse" => crate::SequenceMode::OnceReverse,
                                        "loopReverse" => crate::SequenceMode::LoopReverse,
                                        "pingpongReverse" => crate::SequenceMode::PingPongReverse,
                                        other => {
                                            return Err(Error::InvalidValue {
                                                message: format!(
                                                    "invalid sequence mode '{other}' for animation '{name}' attachment timeline {skin_name}.{slot_name}.{attachment_name}"
                                                ),
                                            });
                                        }
                                    };
                                    frames.push(crate::SequenceFrame {
                                        time,
                                        mode,
                                        index: key.index.unwrap_or(0),
                                        delay,
                                    });
                                }
                                frames.sort_by(|a, b| a.time.total_cmp(&b.time));

                                sequence_timelines.push(crate::SequenceTimeline {
                                    skin: skin_name.clone(),
                                    slot_index: s_index,
                                    attachment: attachment_name,
                                    frames,
                                });
                            }
                        }
                    }
                }
            }

            let mut slot_attachment_timelines = Vec::new();
            let mut slot_color_timelines = Vec::new();
            let mut slot_rgb_timelines: Vec<RgbTimeline> = Vec::new();
            let mut slot_alpha_timelines: Vec<AlphaTimeline> = Vec::new();
            let mut slot_rgba2_timelines = Vec::new();
            let mut slot_rgb2_timelines = Vec::new();
            for (slot_name, slot_anim) in slots_def {
                let s_index = slot_index.get(&slot_name).copied().ok_or_else(|| {
                    Error::JsonUnknownSlotTimelineSlot {
                        animation: name.clone(),
                        slot: slot_name.clone(),
                    }
                })?;

                if let Some(keys) = slot_anim.attachment {
                    let mut frames = keys
                        .into_iter()
                        .map(|k| AttachmentFrame {
                            time: k.time.unwrap_or(0.0),
                            name: k.name,
                        })
                        .collect::<Vec<_>>();
                    frames.sort_by(|a, b| a.time.total_cmp(&b.time));
                    if let Some(last) = frames.last() {
                        duration = duration.max(last.time);
                    }
                    slot_attachment_timelines.push(AttachmentTimeline {
                        slot_index: s_index,
                        frames,
                    });
                }

                if let Some(keys) = slot_anim.rgba {
                    let curve_context = format!("animation '{name}'.slots.{slot_name}.color curve");
                    let mut frames = Vec::with_capacity(keys.len());
                    for key in keys {
                        let time = key.time.unwrap_or(0.0);
                        duration = duration.max(time);
                        let color_str = key.color.unwrap_or_else(|| "FFFFFFFF".to_string());
                        let color = parse_hex_color_rgba(
                            &color_str,
                            &format!("slot color timeline '{}.slots.{}.color'", name, slot_name),
                        )?;
                        frames.push(ColorFrame {
                            time,
                            color,
                            curve: parse_curve_n(key.curve.as_ref(), [1.0; 4], &curve_context)?,
                        });
                    }
                    frames.sort_by(|a, b| a.time.total_cmp(&b.time));
                    slot_color_timelines.push(ColorTimeline {
                        slot_index: s_index,
                        frames,
                    });
                }

                if let Some(keys) = slot_anim.rgb {
                    let curve_context = format!("animation '{name}'.slots.{slot_name}.rgb curve");
                    let mut frames = Vec::with_capacity(keys.len());
                    for key in keys {
                        let time = key.time.unwrap_or(0.0);
                        duration = duration.max(time);
                        let color_str = key.color.unwrap_or_else(|| "FFFFFF".to_string());
                        let color = parse_hex_color_rgb(
                            &color_str,
                            &format!("slot rgb timeline '{}.slots.{}.rgb'", name, slot_name),
                        )?;
                        frames.push(RgbFrame {
                            time,
                            color,
                            curve: parse_curve_n(key.curve.as_ref(), [1.0; 3], &curve_context)?,
                        });
                    }
                    frames.sort_by(|a, b| a.time.total_cmp(&b.time));
                    slot_rgb_timelines.push(RgbTimeline {
                        slot_index: s_index,
                        frames,
                    });
                }

                if let Some(keys) = slot_anim.alpha {
                    let curve_context = format!("animation '{name}'.slots.{slot_name}.alpha curve");
                    let mut frames = Vec::with_capacity(keys.len());
                    for key in keys {
                        let time = key.time.unwrap_or(0.0);
                        duration = duration.max(time);
                        frames.push(AlphaFrame {
                            time,
                            alpha: key.value.unwrap_or(1.0),
                            curve: parse_curve_1(key.curve.as_ref(), 1.0, &curve_context)?,
                        });
                    }
                    frames.sort_by(|a, b| a.time.total_cmp(&b.time));
                    slot_alpha_timelines.push(AlphaTimeline {
                        slot_index: s_index,
                        frames,
                    });
                }

                if let Some(keys) = slot_anim.rgba2 {
                    if !slots.get(s_index).map(|s| s.has_dark).unwrap_or(false) {
                        return Err(Error::JsonTwoColorTimelineRequiresDarkSlot {
                            animation: name.clone(),
                            slot: slot_name.clone(),
                            timeline: "rgba2".to_string(),
                        });
                    }

                    let curve_context = format!("animation '{name}'.slots.{slot_name}.rgba2 curve");
                    let mut frames = Vec::with_capacity(keys.len());
                    for key in keys {
                        let time = key.time.unwrap_or(0.0);
                        duration = duration.max(time);
                        let light_str = key.light.unwrap_or_else(|| "FFFFFFFF".to_string());
                        let dark_str = key.dark.unwrap_or_else(|| "000000".to_string());
                        let light = parse_hex_color_rgba(
                            &light_str,
                            &format!(
                                "slot rgba2 timeline '{}.slots.{}.rgba2.light'",
                                name, slot_name
                            ),
                        )?;
                        let dark = parse_hex_color_rgb(
                            &dark_str,
                            &format!(
                                "slot rgba2 timeline '{}.slots.{}.rgba2.dark'",
                                name, slot_name
                            ),
                        )?;
                        frames.push(Rgba2Frame {
                            time,
                            light,
                            dark,
                            curve: parse_curve_n(key.curve.as_ref(), [1.0; 7], &curve_context)?,
                        });
                    }
                    frames.sort_by(|a, b| a.time.total_cmp(&b.time));
                    slot_rgba2_timelines.push(Rgba2Timeline {
                        slot_index: s_index,
                        frames,
                    });
                }

                if let Some(keys) = slot_anim.rgb2 {
                    if !slots.get(s_index).map(|s| s.has_dark).unwrap_or(false) {
                        return Err(Error::JsonTwoColorTimelineRequiresDarkSlot {
                            animation: name.clone(),
                            slot: slot_name.clone(),
                            timeline: "rgb2".to_string(),
                        });
                    }

                    let curve_context = format!("animation '{name}'.slots.{slot_name}.rgb2 curve");
                    let mut frames = Vec::with_capacity(keys.len());
                    for key in keys {
                        let time = key.time.unwrap_or(0.0);
                        duration = duration.max(time);
                        let light_str = key.light.unwrap_or_else(|| "FFFFFF".to_string());
                        let dark_str = key.dark.unwrap_or_else(|| "000000".to_string());
                        let light = parse_hex_color_rgb(
                            &light_str,
                            &format!(
                                "slot rgb2 timeline '{}.slots.{}.rgb2.light'",
                                name, slot_name
                            ),
                        )?;
                        let dark = parse_hex_color_rgb(
                            &dark_str,
                            &format!(
                                "slot rgb2 timeline '{}.slots.{}.rgb2.dark'",
                                name, slot_name
                            ),
                        )?;
                        frames.push(Rgb2Frame {
                            time,
                            light,
                            dark,
                            curve: parse_curve_n(key.curve.as_ref(), [1.0; 6], &curve_context)?,
                        });
                    }
                    frames.sort_by(|a, b| a.time.total_cmp(&b.time));
                    slot_rgb2_timelines.push(Rgb2Timeline {
                        slot_index: s_index,
                        frames,
                    });
                }
            }

            let draw_order_timeline = if let Some(keys) = draw_order_def {
                let mut frames = Vec::with_capacity(keys.len());
                for key in keys {
                    let time = key.time.unwrap_or(0.0);
                    duration = duration.max(time);

                    let draw_order_to_setup_index = match key.offsets.as_deref() {
                        Some(offsets) => Some(build_draw_order_to_setup_index(
                            offsets,
                            slots.len(),
                            &slot_index,
                            name.as_str(),
                        )?),
                        None => None,
                    };
                    frames.push(DrawOrderFrame {
                        time,
                        draw_order_to_setup_index,
                    });
                }
                frames.sort_by(|a, b| a.time.total_cmp(&b.time));
                Some(DrawOrderTimeline { frames })
            } else {
                None
            };

            let mut draw_order_folder_timelines = Vec::new();
            let mut draw_order_folder_names = Vec::new();
            for (folder_name, folder) in draw_order_folder_def {
                let mut folder_slots = Vec::with_capacity(folder.slots.len());
                for slot_name in folder.slots {
                    let slot = slot_index.get(&slot_name).copied().ok_or_else(|| {
                        Error::JsonInvalidDrawOrder {
                            animation: name.clone(),
                            message: format!(
                                "unknown slot '{}' in drawOrderFolder slots",
                                slot_name
                            ),
                        }
                    })?;
                    folder_slots.push(slot);
                }

                let mut folder_slot_index = HashMap::with_capacity(folder_slots.len());
                for (folder_index, &setup_slot_index) in folder_slots.iter().enumerate() {
                    let slot_name = slots
                        .get(setup_slot_index)
                        .map(|s| s.name.clone())
                        .unwrap_or_default();
                    folder_slot_index.insert(slot_name, folder_index);
                }

                let keys = folder.keys.unwrap_or_default();
                let mut frames = Vec::with_capacity(keys.len());
                for key in keys {
                    let time = key.time.unwrap_or(0.0);
                    duration = duration.max(time);
                    let folder_draw_order = match key.offsets.as_deref() {
                        Some(offsets) => Some(build_draw_order_to_setup_index(
                            offsets,
                            folder_slots.len(),
                            &folder_slot_index,
                            name.as_str(),
                        )?),
                        None => None,
                    };
                    frames.push(DrawOrderFolderFrame {
                        time,
                        folder_draw_order,
                    });
                }
                frames.sort_by(|a, b| a.time.total_cmp(&b.time));
                draw_order_folder_names.push(folder_name);
                draw_order_folder_timelines.push(DrawOrderFolderTimeline {
                    slots: folder_slots,
                    frames,
                });
            }

            let mut ik_constraint_timelines = Vec::new();
            for (constraint_name, keys) in ik_def {
                let constraint_index =
                    *ik_constraint_index.get(&constraint_name).ok_or_else(|| {
                        Error::JsonUnknownIkConstraintTimeline {
                            animation: name.clone(),
                            constraint: constraint_name.clone(),
                        }
                    })?;

                let curve_context = format!("animation '{name}'.ik.{constraint_name} curve");
                let mut frames = Vec::with_capacity(keys.len());
                for k in keys {
                    frames.push(IkFrame {
                        time: k.time.unwrap_or(0.0),
                        mix: k.mix.unwrap_or(1.0),
                        softness: k.softness.unwrap_or(0.0) * scale,
                        bend_direction: if k.bend_positive.unwrap_or(true) {
                            1
                        } else {
                            -1
                        },
                        compress: k.compress.unwrap_or(false),
                        stretch: k.stretch.unwrap_or(false),
                        curve: parse_curve_n(k.curve.as_ref(), [1.0, scale], &curve_context)?,
                    });
                }
                frames.sort_by(|a, b| a.time.total_cmp(&b.time));
                if let Some(last) = frames.last() {
                    duration = duration.max(last.time);
                }
                ik_constraint_timelines.push(IkConstraintTimeline {
                    constraint_index,
                    frames,
                });
            }

            let mut transform_constraint_timelines = Vec::new();
            for (constraint_name, keys) in transform_def {
                let constraint_index = *transform_constraint_index
                    .get(&constraint_name)
                    .ok_or_else(|| Error::JsonUnknownTransformConstraintTimeline {
                        animation: name.clone(),
                        constraint: constraint_name.clone(),
                    })?;

                let mut frames = Vec::with_capacity(keys.len());
                let curve_context = format!("animation '{name}'.transform.{constraint_name} curve");
                for k in keys {
                    let time = k.time.unwrap_or(0.0);
                    duration = duration.max(time);

                    let mix_rotate = k.mix_rotate.unwrap_or(1.0);
                    let mix_x = k.mix_x.unwrap_or(1.0);
                    let mix_y = k.mix_y.unwrap_or(mix_x);
                    let mix_scale_x = k.mix_scale_x.unwrap_or(1.0);
                    let mix_scale_y = k.mix_scale_y.unwrap_or(mix_scale_x);
                    let mix_shear_y = k.mix_shear_y.unwrap_or(1.0);

                    frames.push(crate::TransformFrame {
                        time,
                        mix_rotate,
                        mix_x,
                        mix_y,
                        mix_scale_x,
                        mix_scale_y,
                        mix_shear_y,
                        curve: parse_curve_n(k.curve.as_ref(), [1.0; 6], &curve_context)?,
                    });
                }
                frames.sort_by(|a, b| a.time.total_cmp(&b.time));
                if let Some(last) = frames.last() {
                    duration = duration.max(last.time);
                }
                transform_constraint_timelines.push(crate::TransformConstraintTimeline {
                    constraint_index,
                    frames,
                });
            }

            let mut path_constraint_timelines = Vec::new();
            for (constraint_name, timelines) in path_def {
                let constraint_index =
                    *path_constraint_index.get(&constraint_name).ok_or_else(|| {
                        Error::JsonUnknownPathConstraintTimeline {
                            animation: name.clone(),
                            constraint: constraint_name.clone(),
                        }
                    })?;

                let constraint = path_constraints.get(constraint_index).ok_or_else(|| {
                    Error::JsonUnknownPathConstraintTimeline {
                        animation: name.clone(),
                        constraint: constraint_name.clone(),
                    }
                })?;
                let position_scale = if constraint.position_mode == crate::PositionMode::Fixed {
                    scale
                } else {
                    1.0
                };
                let spacing_scale = if constraint.spacing_mode == crate::SpacingMode::Length
                    || constraint.spacing_mode == crate::SpacingMode::Fixed
                {
                    scale
                } else {
                    1.0
                };

                for (timeline_name, keys) in timelines {
                    if keys.is_empty() {
                        continue;
                    }
                    match timeline_name.as_str() {
                        "position" => {
                            let curve_context =
                                format!("animation '{name}'.path.{constraint_name}.position curve");
                            let mut frames = Vec::with_capacity(keys.len());
                            for k in keys {
                                let time = k.time.as_ref().and_then(curve_number).unwrap_or(0.0);
                                let value = k
                                    .value
                                    .as_ref()
                                    .or(k.position.as_ref())
                                    .and_then(curve_number)
                                    .unwrap_or(0.0)
                                    * position_scale;
                                frames.push(crate::FloatFrame {
                                    time,
                                    value,
                                    curve: parse_curve_1(
                                        k.curve.as_ref(),
                                        position_scale,
                                        &curve_context,
                                    )?,
                                });
                            }
                            frames.sort_by(|a, b| a.time.total_cmp(&b.time));
                            if let Some(last) = frames.last() {
                                duration = duration.max(last.time);
                            }
                            path_constraint_timelines.push(
                                crate::PathConstraintTimeline::Position(
                                    crate::PathConstraintPositionTimeline {
                                        constraint_index,
                                        frames,
                                    },
                                ),
                            );
                        }
                        "spacing" => {
                            let curve_context =
                                format!("animation '{name}'.path.{constraint_name}.spacing curve");
                            let mut frames = Vec::with_capacity(keys.len());
                            for k in keys {
                                let time = k.time.as_ref().and_then(curve_number).unwrap_or(0.0);
                                let value = k
                                    .value
                                    .as_ref()
                                    .or(k.spacing.as_ref())
                                    .and_then(curve_number)
                                    .unwrap_or(0.0)
                                    * spacing_scale;
                                frames.push(crate::FloatFrame {
                                    time,
                                    value,
                                    curve: parse_curve_1(
                                        k.curve.as_ref(),
                                        spacing_scale,
                                        &curve_context,
                                    )?,
                                });
                            }
                            frames.sort_by(|a, b| a.time.total_cmp(&b.time));
                            if let Some(last) = frames.last() {
                                duration = duration.max(last.time);
                            }
                            path_constraint_timelines.push(crate::PathConstraintTimeline::Spacing(
                                crate::PathConstraintSpacingTimeline {
                                    constraint_index,
                                    frames,
                                },
                            ));
                        }
                        "mix" => {
                            let curve_context =
                                format!("animation '{name}'.path.{constraint_name}.mix curve");
                            let mut frames = Vec::with_capacity(keys.len());
                            for k in keys {
                                let time = k.time.as_ref().and_then(curve_number).unwrap_or(0.0);
                                duration = duration.max(time);
                                let mix_rotate =
                                    k.mix_rotate.as_ref().and_then(curve_number).unwrap_or(1.0);
                                let mix_x = k.mix_x.as_ref().and_then(curve_number).unwrap_or(1.0);
                                let mix_y =
                                    k.mix_y.as_ref().and_then(curve_number).unwrap_or(mix_x);
                                frames.push(crate::PathMixFrame {
                                    time,
                                    mix_rotate,
                                    mix_x,
                                    mix_y,
                                    curve: parse_curve_n(
                                        k.curve.as_ref(),
                                        [1.0; 3],
                                        &curve_context,
                                    )?,
                                });
                            }
                            frames.sort_by(|a, b| a.time.total_cmp(&b.time));
                            if let Some(last) = frames.last() {
                                duration = duration.max(last.time);
                            }
                            path_constraint_timelines.push(crate::PathConstraintTimeline::Mix(
                                crate::PathConstraintMixTimeline {
                                    constraint_index,
                                    frames,
                                },
                            ));
                        }
                        _ => {}
                    }
                }
            }

            let mut physics_constraint_timelines = Vec::new();
            let mut physics_reset_timelines = Vec::new();
            for (constraint_name, timelines) in physics_def {
                let constraint_index: i32 = if constraint_name.is_empty() {
                    -1
                } else {
                    *physics_constraint_index
                        .get(&constraint_name)
                        .ok_or_else(|| Error::JsonUnknownPhysicsConstraintTimeline {
                            animation: name.clone(),
                            constraint: constraint_name.clone(),
                        })? as i32
                };

                for (timeline_name, keys) in timelines {
                    if keys.is_empty() {
                        continue;
                    }

                    if timeline_name == "reset" {
                        let mut frames = keys
                            .iter()
                            .map(|k| k.time.unwrap_or(0.0))
                            .collect::<Vec<_>>();
                        frames.sort_by(|a, b| a.total_cmp(b));
                        if let Some(last) = frames.last() {
                            duration = duration.max(*last);
                        }
                        physics_reset_timelines.push(crate::PhysicsConstraintResetTimeline {
                            constraint_index,
                            frames,
                        });
                        continue;
                    }

                    let curve_context = format!(
                        "animation '{name}'.physics.{constraint_name}.{timeline_name} curve"
                    );
                    let mut frames = Vec::with_capacity(keys.len());
                    for k in keys {
                        let time = k.time.unwrap_or(0.0);
                        let value = k.value.unwrap_or(0.0);
                        frames.push(crate::FloatFrame {
                            time,
                            value,
                            curve: parse_curve_1(k.curve.as_ref(), 1.0, &curve_context)?,
                        });
                    }
                    frames.sort_by(|a, b| a.time.total_cmp(&b.time));
                    if let Some(last) = frames.last() {
                        duration = duration.max(last.time);
                    }

                    let timeline = crate::PhysicsConstraintFloatTimeline {
                        constraint_index,
                        frames,
                    };
                    let wrapped = match timeline_name.as_str() {
                        "inertia" => crate::PhysicsConstraintTimeline::Inertia(timeline),
                        "strength" => crate::PhysicsConstraintTimeline::Strength(timeline),
                        "damping" => crate::PhysicsConstraintTimeline::Damping(timeline),
                        "mass" => crate::PhysicsConstraintTimeline::Mass(timeline),
                        "wind" => crate::PhysicsConstraintTimeline::Wind(timeline),
                        "gravity" => crate::PhysicsConstraintTimeline::Gravity(timeline),
                        "mix" => crate::PhysicsConstraintTimeline::Mix(timeline),
                        _ => continue,
                    };
                    physics_constraint_timelines.push(wrapped);
                }
            }

            let mut slider_time_timelines = Vec::new();
            let mut slider_mix_timelines = Vec::new();
            for (constraint_name, timelines) in slider_def {
                let constraint_index =
                    *slider_constraint_index
                        .get(&constraint_name)
                        .ok_or_else(|| Error::JsonUnknownSliderConstraintTimeline {
                            animation: name.clone(),
                            constraint: constraint_name.clone(),
                        })?;
                for (timeline_name, keys) in timelines {
                    if keys.is_empty() {
                        continue;
                    }

                    // Match upstream: Slider timelines use `readTimeline(..., defaultValue=1, scale=1)`.
                    let default_value = 1.0f32;
                    let curve_context = format!(
                        "animation '{name}'.slider.{constraint_name}.{timeline_name} curve"
                    );
                    let mut frames = Vec::with_capacity(keys.len());
                    for k in keys {
                        let time = k.time.unwrap_or(0.0);
                        let value = k.value.unwrap_or(default_value);
                        frames.push(crate::FloatFrame {
                            time,
                            value,
                            curve: parse_curve_1(k.curve.as_ref(), 1.0, &curve_context)?,
                        });
                    }
                    frames.sort_by(|a, b| a.time.total_cmp(&b.time));
                    if let Some(last) = frames.last() {
                        duration = duration.max(last.time);
                    }

                    let timeline = SliderConstraintTimeline {
                        constraint_index,
                        frames,
                    };
                    match timeline_name.as_str() {
                        "time" => slider_time_timelines.push(timeline),
                        "mix" => slider_mix_timelines.push(timeline),
                        _ => {}
                    }
                }
            }

            let timeline_lookup = build_json_timeline_lookup(
                &slots,
                &bones,
                &slot_attachment_timelines,
                &slot_color_timelines,
                &slot_rgb_timelines,
                &slot_alpha_timelines,
                &slot_rgba2_timelines,
                &slot_rgb2_timelines,
                &bone_timelines,
                &deform_timelines,
                &sequence_timelines,
                &ik_constraint_timelines,
                &transform_constraint_timelines,
                &path_constraint_timelines,
                &physics_constraint_timelines,
                &physics_reset_timelines,
                &slider_time_timelines,
                &slider_mix_timelines,
                draw_order_timeline.as_ref(),
                &draw_order_folder_timelines,
                &draw_order_folder_names,
                &ik_constraints,
                &transform_constraints,
                &path_constraints,
                &physics_constraints,
                &slider_constraints,
            );
            let timeline_order = build_json_timeline_order(&def_value, &timeline_lookup);

            let index = animations.len();
            let animation = Animation {
                name: name.clone(),
                duration,
                event_timeline: timeline,
                bone_timelines,
                deform_timelines,
                sequence_timelines,
                slot_attachment_timelines,
                slot_color_timelines,
                slot_rgb_timelines,
                slot_alpha_timelines,
                slot_rgba2_timelines,
                slot_rgb2_timelines,
                ik_constraint_timelines,
                transform_constraint_timelines,
                path_constraint_timelines,
                physics_constraint_timelines,
                physics_reset_timelines,
                slider_time_timelines,
                slider_mix_timelines,
                draw_order_timeline,
                draw_order_folder_timelines,
                timeline_order,
            };
            animations.push(crate::runtime::finalize_animation(animation));
            animation_index.insert(name, index);
        }

        for (constraint_index, animation_name) in pending_slider_animations {
            let Some(anim_index) = animation_index.get(&animation_name).copied() else {
                return Err(Error::JsonUnknownSliderAnimation {
                    constraint: slider_constraints
                        .get(constraint_index)
                        .map(|c| c.name.clone())
                        .unwrap_or_else(|| "<unknown>".to_string()),
                    animation: animation_name,
                });
            };
            if let Some(c) = slider_constraints.get_mut(constraint_index) {
                c.animation = Some(anim_index);
            }
        }

        Ok(Arc::new(SkeletonData {
            spine_version,
            reference_scale,
            bones,
            slots,
            skins,
            events,
            animations,
            animation_index,
            ik_constraints,
            transform_constraints,
            path_constraints,
            physics_constraints,
            slider_constraints,
        }))
    }
}

fn from_json_value<T: DeserializeOwned>(
    value: &serde_json::Value,
    context: &str,
) -> Result<T, Error> {
    serde_json::from_value(value.clone()).map_err(|e| Error::JsonParse {
        message: format!("{context}: {e}"),
    })
}

fn normalize_bone_timeline_name(name: &str) -> &str {
    match name {
        "translateX" => "translatex",
        "translateY" => "translatey",
        "scaleX" => "scalex",
        "scaleY" => "scaley",
        "shearX" => "shearx",
        "shearY" => "sheary",
        _ => name,
    }
}

fn push_json_timeline_order(
    out: &mut Vec<TimelineKind>,
    timelines: &HashMap<JsonTimelineRef, TimelineKind>,
    key: JsonTimelineRef,
) {
    if let Some(kind) = timelines.get(&key).copied() {
        out.push(kind);
    }
}

#[allow(clippy::too_many_arguments)]
fn build_json_timeline_lookup(
    slots: &[SlotData],
    bones: &[BoneData],
    slot_attachment_timelines: &[AttachmentTimeline],
    slot_color_timelines: &[ColorTimeline],
    slot_rgb_timelines: &[RgbTimeline],
    slot_alpha_timelines: &[AlphaTimeline],
    slot_rgba2_timelines: &[Rgba2Timeline],
    slot_rgb2_timelines: &[Rgb2Timeline],
    bone_timelines: &[BoneTimeline],
    deform_timelines: &[DeformTimeline],
    sequence_timelines: &[crate::SequenceTimeline],
    ik_constraint_timelines: &[IkConstraintTimeline],
    transform_constraint_timelines: &[crate::TransformConstraintTimeline],
    path_constraint_timelines: &[crate::PathConstraintTimeline],
    physics_constraint_timelines: &[crate::PhysicsConstraintTimeline],
    physics_reset_timelines: &[crate::PhysicsConstraintResetTimeline],
    slider_time_timelines: &[SliderConstraintTimeline],
    slider_mix_timelines: &[SliderConstraintTimeline],
    draw_order_timeline: Option<&DrawOrderTimeline>,
    draw_order_folder_timelines: &[DrawOrderFolderTimeline],
    draw_order_folder_names: &[String],
    ik_constraints: &[crate::IkConstraintData],
    transform_constraints: &[crate::TransformConstraintData],
    path_constraints: &[crate::PathConstraintData],
    physics_constraints: &[crate::PhysicsConstraintData],
    slider_constraints: &[crate::SliderConstraintData],
) -> HashMap<JsonTimelineRef, TimelineKind> {
    let mut out = HashMap::new();
    for (i, timeline) in slot_attachment_timelines.iter().enumerate() {
        if let Some(slot) = slots.get(timeline.slot_index) {
            out.insert(
                JsonTimelineRef::Slot {
                    slot: slot.name.clone(),
                    timeline: "attachment".to_string(),
                },
                TimelineKind::SlotAttachment(i),
            );
        }
    }
    for (i, timeline) in slot_color_timelines.iter().enumerate() {
        if let Some(slot) = slots.get(timeline.slot_index) {
            out.insert(
                JsonTimelineRef::Slot {
                    slot: slot.name.clone(),
                    timeline: "rgba".to_string(),
                },
                TimelineKind::SlotColor(i),
            );
        }
    }
    for (i, timeline) in slot_rgb_timelines.iter().enumerate() {
        if let Some(slot) = slots.get(timeline.slot_index) {
            out.insert(
                JsonTimelineRef::Slot {
                    slot: slot.name.clone(),
                    timeline: "rgb".to_string(),
                },
                TimelineKind::SlotRgb(i),
            );
        }
    }
    for (i, timeline) in slot_alpha_timelines.iter().enumerate() {
        if let Some(slot) = slots.get(timeline.slot_index) {
            out.insert(
                JsonTimelineRef::Slot {
                    slot: slot.name.clone(),
                    timeline: "alpha".to_string(),
                },
                TimelineKind::SlotAlpha(i),
            );
        }
    }
    for (i, timeline) in slot_rgba2_timelines.iter().enumerate() {
        if let Some(slot) = slots.get(timeline.slot_index) {
            out.insert(
                JsonTimelineRef::Slot {
                    slot: slot.name.clone(),
                    timeline: "rgba2".to_string(),
                },
                TimelineKind::SlotRgba2(i),
            );
        }
    }
    for (i, timeline) in slot_rgb2_timelines.iter().enumerate() {
        if let Some(slot) = slots.get(timeline.slot_index) {
            out.insert(
                JsonTimelineRef::Slot {
                    slot: slot.name.clone(),
                    timeline: "rgb2".to_string(),
                },
                TimelineKind::SlotRgb2(i),
            );
        }
    }

    for (i, timeline) in bone_timelines.iter().enumerate() {
        let (bone_index, timeline_name) = match timeline {
            BoneTimeline::Rotate(t) => (t.bone_index, "rotate"),
            BoneTimeline::Translate(t) => (t.bone_index, "translate"),
            BoneTimeline::TranslateX(t) => (t.bone_index, "translatex"),
            BoneTimeline::TranslateY(t) => (t.bone_index, "translatey"),
            BoneTimeline::Scale(t) => (t.bone_index, "scale"),
            BoneTimeline::ScaleX(t) => (t.bone_index, "scalex"),
            BoneTimeline::ScaleY(t) => (t.bone_index, "scaley"),
            BoneTimeline::Shear(t) => (t.bone_index, "shear"),
            BoneTimeline::ShearX(t) => (t.bone_index, "shearx"),
            BoneTimeline::ShearY(t) => (t.bone_index, "sheary"),
            BoneTimeline::Inherit(t) => (t.bone_index, "inherit"),
        };
        if let Some(bone) = bones.get(bone_index) {
            out.insert(
                JsonTimelineRef::Bone {
                    bone: bone.name.clone(),
                    timeline: timeline_name.to_string(),
                },
                TimelineKind::Bone(i),
            );
        }
    }

    for (i, timeline) in ik_constraint_timelines.iter().enumerate() {
        if let Some(constraint) = ik_constraints.get(timeline.constraint_index) {
            out.insert(
                JsonTimelineRef::Ik {
                    constraint: constraint.name.clone(),
                },
                TimelineKind::IkConstraint(i),
            );
        }
    }
    for (i, timeline) in transform_constraint_timelines.iter().enumerate() {
        if let Some(constraint) = transform_constraints.get(timeline.constraint_index) {
            out.insert(
                JsonTimelineRef::Transform {
                    constraint: constraint.name.clone(),
                },
                TimelineKind::TransformConstraint(i),
            );
        }
    }
    for (i, timeline) in path_constraint_timelines.iter().enumerate() {
        let (constraint_index, timeline_name) = match timeline {
            crate::PathConstraintTimeline::Position(t) => (t.constraint_index, "position"),
            crate::PathConstraintTimeline::Spacing(t) => (t.constraint_index, "spacing"),
            crate::PathConstraintTimeline::Mix(t) => (t.constraint_index, "mix"),
        };
        if let Some(constraint) = path_constraints.get(constraint_index) {
            out.insert(
                JsonTimelineRef::Path {
                    constraint: constraint.name.clone(),
                    timeline: timeline_name.to_string(),
                },
                TimelineKind::PathConstraint(i),
            );
        }
    }
    for (i, timeline) in physics_constraint_timelines.iter().enumerate() {
        let (constraint_index, timeline_name) = match timeline {
            crate::PhysicsConstraintTimeline::Inertia(t) => (t.constraint_index, "inertia"),
            crate::PhysicsConstraintTimeline::Strength(t) => (t.constraint_index, "strength"),
            crate::PhysicsConstraintTimeline::Damping(t) => (t.constraint_index, "damping"),
            crate::PhysicsConstraintTimeline::Mass(t) => (t.constraint_index, "mass"),
            crate::PhysicsConstraintTimeline::Wind(t) => (t.constraint_index, "wind"),
            crate::PhysicsConstraintTimeline::Gravity(t) => (t.constraint_index, "gravity"),
            crate::PhysicsConstraintTimeline::Mix(t) => (t.constraint_index, "mix"),
        };
        let constraint = if constraint_index < 0 {
            Some("")
        } else {
            physics_constraints
                .get(constraint_index as usize)
                .map(|c| c.name.as_str())
        };
        if let Some(constraint) = constraint {
            out.insert(
                JsonTimelineRef::Physics {
                    constraint: constraint.to_string(),
                    timeline: timeline_name.to_string(),
                },
                TimelineKind::PhysicsConstraint(i),
            );
        }
    }
    for (i, timeline) in physics_reset_timelines.iter().enumerate() {
        let constraint = if timeline.constraint_index < 0 {
            Some("")
        } else {
            physics_constraints
                .get(timeline.constraint_index as usize)
                .map(|c| c.name.as_str())
        };
        if let Some(constraint) = constraint {
            out.insert(
                JsonTimelineRef::Physics {
                    constraint: constraint.to_string(),
                    timeline: "reset".to_string(),
                },
                TimelineKind::PhysicsReset(i),
            );
        }
    }
    for (i, timeline) in slider_time_timelines.iter().enumerate() {
        if let Some(constraint) = slider_constraints.get(timeline.constraint_index) {
            out.insert(
                JsonTimelineRef::Slider {
                    constraint: constraint.name.clone(),
                    timeline: "time".to_string(),
                },
                TimelineKind::SliderTime(i),
            );
        }
    }
    for (i, timeline) in slider_mix_timelines.iter().enumerate() {
        if let Some(constraint) = slider_constraints.get(timeline.constraint_index) {
            out.insert(
                JsonTimelineRef::Slider {
                    constraint: constraint.name.clone(),
                    timeline: "mix".to_string(),
                },
                TimelineKind::SliderMix(i),
            );
        }
    }

    for (i, timeline) in deform_timelines.iter().enumerate() {
        if let Some(slot) = slots.get(timeline.slot_index) {
            out.insert(
                JsonTimelineRef::Attachment {
                    skin: timeline.skin.clone(),
                    slot: slot.name.clone(),
                    attachment: timeline.attachment.clone(),
                    timeline: "deform".to_string(),
                },
                TimelineKind::Deform(i),
            );
        }
    }
    for (i, timeline) in sequence_timelines.iter().enumerate() {
        if let Some(slot) = slots.get(timeline.slot_index) {
            out.insert(
                JsonTimelineRef::Attachment {
                    skin: timeline.skin.clone(),
                    slot: slot.name.clone(),
                    attachment: timeline.attachment.clone(),
                    timeline: "sequence".to_string(),
                },
                TimelineKind::Sequence(i),
            );
        }
    }

    if draw_order_timeline.is_some() {
        out.insert(JsonTimelineRef::DrawOrder, TimelineKind::DrawOrder);
    }
    for (i, folder_name) in draw_order_folder_names.iter().enumerate() {
        if i < draw_order_folder_timelines.len() {
            out.insert(
                JsonTimelineRef::DrawOrderFolder {
                    folder: folder_name.clone(),
                },
                TimelineKind::DrawOrderFolder(i),
            );
        }
    }

    out
}

fn build_json_timeline_order(
    animation: &serde_json::Value,
    timelines: &HashMap<JsonTimelineRef, TimelineKind>,
) -> Vec<TimelineKind> {
    let Some(animation) = animation.as_object() else {
        return Vec::new();
    };
    let mut out = Vec::with_capacity(timelines.len());

    if let Some(slots) = animation.get("slots").and_then(|v| v.as_object()) {
        for (slot, slot_map) in slots {
            let Some(slot_map) = slot_map.as_object() else {
                continue;
            };
            for (timeline, _) in slot_map {
                push_json_timeline_order(
                    &mut out,
                    timelines,
                    JsonTimelineRef::Slot {
                        slot: slot.clone(),
                        timeline: timeline.clone(),
                    },
                );
            }
        }
    }

    if let Some(bones) = animation.get("bones").and_then(|v| v.as_object()) {
        for (bone, bone_map) in bones {
            let Some(bone_map) = bone_map.as_object() else {
                continue;
            };
            for (timeline, _) in bone_map {
                push_json_timeline_order(
                    &mut out,
                    timelines,
                    JsonTimelineRef::Bone {
                        bone: bone.clone(),
                        timeline: normalize_bone_timeline_name(timeline).to_string(),
                    },
                );
            }
        }
    }

    if let Some(ik) = animation.get("ik").and_then(|v| v.as_object()) {
        for (constraint, _) in ik {
            push_json_timeline_order(
                &mut out,
                timelines,
                JsonTimelineRef::Ik {
                    constraint: constraint.clone(),
                },
            );
        }
    }

    if let Some(transform) = animation.get("transform").and_then(|v| v.as_object()) {
        for (constraint, _) in transform {
            push_json_timeline_order(
                &mut out,
                timelines,
                JsonTimelineRef::Transform {
                    constraint: constraint.clone(),
                },
            );
        }
    }

    if let Some(path) = animation.get("path").and_then(|v| v.as_object()) {
        for (constraint, path_map) in path {
            let Some(path_map) = path_map.as_object() else {
                continue;
            };
            for (timeline, _) in path_map {
                push_json_timeline_order(
                    &mut out,
                    timelines,
                    JsonTimelineRef::Path {
                        constraint: constraint.clone(),
                        timeline: timeline.clone(),
                    },
                );
            }
        }
    }

    if let Some(physics) = animation.get("physics").and_then(|v| v.as_object()) {
        for (constraint, physics_map) in physics {
            let Some(physics_map) = physics_map.as_object() else {
                continue;
            };
            for (timeline, _) in physics_map {
                push_json_timeline_order(
                    &mut out,
                    timelines,
                    JsonTimelineRef::Physics {
                        constraint: constraint.clone(),
                        timeline: timeline.clone(),
                    },
                );
            }
        }
    }

    if let Some(slider) = animation.get("slider").and_then(|v| v.as_object()) {
        for (constraint, slider_map) in slider {
            let Some(slider_map) = slider_map.as_object() else {
                continue;
            };
            for (timeline, _) in slider_map {
                push_json_timeline_order(
                    &mut out,
                    timelines,
                    JsonTimelineRef::Slider {
                        constraint: constraint.clone(),
                        timeline: timeline.clone(),
                    },
                );
            }
        }
    }

    if let Some(attachments) = animation.get("attachments").and_then(|v| v.as_object()) {
        for (skin, skin_map) in attachments {
            let Some(skin_map) = skin_map.as_object() else {
                continue;
            };
            for (slot, slot_map) in skin_map {
                let Some(slot_map) = slot_map.as_object() else {
                    continue;
                };
                for (attachment, attachment_map) in slot_map {
                    let Some(attachment_map) = attachment_map.as_object() else {
                        continue;
                    };
                    for (timeline, _) in attachment_map {
                        push_json_timeline_order(
                            &mut out,
                            timelines,
                            JsonTimelineRef::Attachment {
                                skin: skin.clone(),
                                slot: slot.clone(),
                                attachment: attachment.clone(),
                                timeline: timeline.clone(),
                            },
                        );
                    }
                }
            }
        }
    }

    if animation.contains_key("drawOrder") {
        push_json_timeline_order(&mut out, timelines, JsonTimelineRef::DrawOrder);
    }

    if let Some(folders) = animation.get("drawOrderFolder").and_then(|v| v.as_object()) {
        for (folder, _) in folders {
            push_json_timeline_order(
                &mut out,
                timelines,
                JsonTimelineRef::DrawOrderFolder {
                    folder: folder.clone(),
                },
            );
        }
    }

    out
}

fn build_draw_order_to_setup_index(
    offsets: &[DrawOrderOffset],
    slot_count: usize,
    slot_index: &HashMap<String, usize>,
    animation: &str,
) -> Result<Vec<usize>, Error> {
    let mut draw_order = vec![usize::MAX; slot_count];
    let mut unchanged = Vec::with_capacity(slot_count.saturating_sub(offsets.len()));
    let mut original_index = 0usize;

    for offset in offsets {
        let slot_idx =
            slot_index
                .get(&offset.slot)
                .copied()
                .ok_or_else(|| Error::JsonInvalidDrawOrder {
                    animation: animation.to_string(),
                    message: format!("unknown slot '{}' in drawOrder offsets", offset.slot),
                })?;

        while original_index != slot_idx {
            unchanged.push(original_index);
            original_index += 1;
            if original_index > slot_count {
                break;
            }
        }

        let target_i64 = original_index as i64 + offset.offset as i64;
        if target_i64 < 0 || target_i64 >= slot_count as i64 {
            return Err(Error::JsonInvalidDrawOrder {
                animation: animation.to_string(),
                message: format!(
                    "drawOrder offset out of range for slot '{}' (offset {})",
                    offset.slot, offset.offset
                ),
            });
        }
        let target = target_i64 as usize;
        if draw_order[target] != usize::MAX {
            return Err(Error::JsonInvalidDrawOrder {
                animation: animation.to_string(),
                message: "drawOrder produced duplicate target indices".to_string(),
            });
        }
        draw_order[target] = original_index;
        original_index += 1;
    }

    while original_index < slot_count {
        unchanged.push(original_index);
        original_index += 1;
    }

    for i in (0..slot_count).rev() {
        if draw_order[i] == usize::MAX {
            let Some(v) = unchanged.pop() else {
                return Err(Error::JsonInvalidDrawOrder {
                    animation: animation.to_string(),
                    message: "drawOrder failed to fill unchanged indices".to_string(),
                });
            };
            draw_order[i] = v;
        }
    }

    Ok(draw_order)
}

fn vertex_attachment_deform_setup(vertices: &crate::MeshVertices) -> (usize, Option<Vec<f32>>) {
    match vertices {
        crate::MeshVertices::Unweighted(vertices) => {
            let mut setup = Vec::with_capacity(vertices.len() * 2);
            for [x, y] in vertices {
                setup.push(*x);
                setup.push(*y);
            }
            (setup.len(), Some(setup))
        }
        crate::MeshVertices::Weighted(vertices) => {
            let weight_count = vertices.iter().map(|v| v.len()).sum::<usize>();
            (weight_count * 2, None)
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn build_deform_vertices(
    attachment_vertices: &crate::MeshVertices,
    setup_vertices: Option<&[f32]>,
    vertex_count: usize,
    offset: usize,
    vertices: Option<&[f32]>,
    scale: f32,
    animation: &str,
    skin: &str,
    slot: &str,
    attachment: &str,
) -> Result<Vec<f32>, Error> {
    let mut out = match vertices {
        None => match attachment_vertices {
            crate::MeshVertices::Unweighted(_) => setup_vertices.unwrap_or(&[]).to_vec(),
            crate::MeshVertices::Weighted(_) => vec![0.0; vertex_count],
        },
        Some(values) => {
            let mut out = vec![0.0f32; vertex_count];
            for (i, v) in values.iter().copied().enumerate() {
                let index = offset + i;
                if index >= vertex_count {
                    return Err(Error::JsonInvalidDeformData {
                        animation: animation.to_string(),
                        skin: skin.to_string(),
                        slot: slot.to_string(),
                        attachment: attachment.to_string(),
                        message: format!(
                            "deform vertices out of range (offset {offset}, len {}, max {vertex_count})",
                            values.len()
                        ),
                    });
                }
                out[index] = v * scale;
            }

            if matches!(attachment_vertices, crate::MeshVertices::Unweighted(_)) {
                let Some(setup) = setup_vertices else {
                    return Err(Error::JsonInvalidDeformData {
                        animation: animation.to_string(),
                        skin: skin.to_string(),
                        slot: slot.to_string(),
                        attachment: attachment.to_string(),
                        message: "missing setup vertices for unweighted deform timeline"
                            .to_string(),
                    });
                };
                for (i, o) in out.iter_mut().enumerate().take(vertex_count) {
                    *o += setup.get(i).copied().unwrap_or(0.0);
                }
            }
            out
        }
    };

    if out.len() != vertex_count {
        out.resize(vertex_count, 0.0);
    }
    Ok(out)
}

fn default_one() -> f32 {
    1.0
}

fn default_true() -> bool {
    true
}

fn parse_hex_color_rgba(input: &str, context: &str) -> Result<[f32; 4], Error> {
    fn hex(b: u8) -> Option<u8> {
        match b {
            b'0'..=b'9' => Some(b - b'0'),
            b'a'..=b'f' => Some(b - b'a' + 10),
            b'A'..=b'F' => Some(b - b'A' + 10),
            _ => None,
        }
    }

    let bytes = input.as_bytes();
    let (r, g, b, a) = match bytes.len() {
        6 => {
            let r = hex(bytes[0]).zip(hex(bytes[1])).map(|(h, l)| (h << 4) | l);
            let g = hex(bytes[2]).zip(hex(bytes[3])).map(|(h, l)| (h << 4) | l);
            let b = hex(bytes[4]).zip(hex(bytes[5])).map(|(h, l)| (h << 4) | l);
            (r, g, b, Some(255))
        }
        8 => {
            let r = hex(bytes[0]).zip(hex(bytes[1])).map(|(h, l)| (h << 4) | l);
            let g = hex(bytes[2]).zip(hex(bytes[3])).map(|(h, l)| (h << 4) | l);
            let b = hex(bytes[4]).zip(hex(bytes[5])).map(|(h, l)| (h << 4) | l);
            let a = hex(bytes[6]).zip(hex(bytes[7])).map(|(h, l)| (h << 4) | l);
            (r, g, b, a)
        }
        _ => {
            return Err(Error::JsonInvalidColor {
                context: context.to_string(),
                value: input.to_string(),
            });
        }
    };

    let (Some(r), Some(g), Some(b), Some(a)) = (r, g, b, a) else {
        return Err(Error::JsonInvalidColor {
            context: context.to_string(),
            value: input.to_string(),
        });
    };

    Ok([
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    ])
}

fn parse_hex_color_rgb(input: &str, context: &str) -> Result<[f32; 3], Error> {
    let rgba = parse_hex_color_rgba(input, context)?;
    Ok([rgba[0], rgba[1], rgba[2]])
}

fn parse_blend_mode(value: Option<&str>, slot_name: &str) -> Result<crate::BlendMode, Error> {
    let Some(value) = value else {
        return Ok(crate::BlendMode::Normal);
    };
    match value {
        "normal" => Ok(crate::BlendMode::Normal),
        "additive" => Ok(crate::BlendMode::Additive),
        "multiply" => Ok(crate::BlendMode::Multiply),
        "screen" => Ok(crate::BlendMode::Screen),
        other => Err(Error::JsonUnsupportedBlendMode {
            slot: slot_name.to_string(),
            value: other.to_string(),
        }),
    }
}

fn curve_number(value: &serde_json::Value) -> Option<f32> {
    if !value.is_number() {
        return None;
    }
    // `serde_json` parses JSON numbers as f64 internally. Some Spine runtimes (notably spine-cpp)
    // parse numbers differently (custom parser to double, then cast to float). For better parity
    // on curve-heavy timelines, re-parse the number from its string form using a matching algorithm.
    let s = value.to_string();
    if s.is_empty() {
        return None;
    }
    Some(parse_spine_cpp_style_number_to_f32(&s))
}

fn parse_spine_cpp_style_number_to_f32(input: &str) -> f32 {
    // Port of `spine-cpp` Json.cpp `parseNumber` behavior:
    // - parse into a `double`-like accumulator (here `f64`)
    // - apply fractional digits using `pow(10, n)`
    // - apply exponent using `pow(10, exponent)`
    // - cast to `float` (here `f32`)
    //
    // This is intentionally *not* a correctly-rounded float parser.
    let bytes = input.as_bytes();
    if bytes.is_empty() {
        return 0.0;
    }

    let mut i = 0usize;
    let mut negative = false;
    if bytes[i] == b'-' {
        negative = true;
        i += 1;
    }

    let mut result = 0.0f64;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        result = result * 10.0 + (bytes[i] - b'0') as f64;
        i += 1;
    }

    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        let mut fraction = 0.0f64;
        let mut n = 0i32;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            fraction = fraction * 10.0 + (bytes[i] - b'0') as f64;
            i += 1;
            n += 1;
        }
        if n > 0 {
            result += fraction / 10.0f64.powi(n);
        }
    }

    if negative {
        result = -result;
    }

    if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
        i += 1;
        let mut exp_negative = false;
        if i < bytes.len() && bytes[i] == b'-' {
            exp_negative = true;
            i += 1;
        } else if i < bytes.len() && bytes[i] == b'+' {
            i += 1;
        }

        let mut exponent = 0.0f64;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            exponent = exponent * 10.0 + (bytes[i] - b'0') as f64;
            i += 1;
        }

        if exp_negative {
            result /= 10.0f64.powf(exponent);
        } else {
            result *= 10.0f64.powf(exponent);
        }
    }

    result as f32
}

fn parse_curve_1(
    value: Option<&serde_json::Value>,
    scale: f32,
    context: &str,
) -> Result<Curve, Error> {
    let Some(value) = value else {
        return Ok(Curve::Linear);
    };

    if let Some(s) = value.as_str() {
        return Ok(match s {
            "stepped" => Curve::Stepped,
            _ => Curve::Linear,
        });
    }

    let Some(arr) = value.as_array() else {
        return Ok(Curve::Linear);
    };

    if arr.len() != 4 {
        return Err(Error::JsonInvalidCurve {
            context: context.to_string(),
            message: format!("expected 4 numbers, got {}", arr.len()),
        });
    }

    let cx1 = curve_number(&arr[0]).ok_or_else(|| Error::JsonInvalidCurve {
        context: context.to_string(),
        message: "curve[0] must be a number".to_string(),
    })?;
    let cy1 = curve_number(&arr[1]).ok_or_else(|| Error::JsonInvalidCurve {
        context: context.to_string(),
        message: "curve[1] must be a number".to_string(),
    })? * scale;
    let cx2 = curve_number(&arr[2]).ok_or_else(|| Error::JsonInvalidCurve {
        context: context.to_string(),
        message: "curve[2] must be a number".to_string(),
    })?;
    let cy2 = curve_number(&arr[3]).ok_or_else(|| Error::JsonInvalidCurve {
        context: context.to_string(),
        message: "curve[3] must be a number".to_string(),
    })? * scale;

    Ok(Curve::Bezier { cx1, cy1, cx2, cy2 })
}

fn parse_curve_n<const N: usize>(
    value: Option<&serde_json::Value>,
    scales: [f32; N],
    context: &str,
) -> Result<[Curve; N], Error> {
    let Some(value) = value else {
        return Ok([Curve::Linear; N]);
    };

    if let Some(s) = value.as_str() {
        return Ok(match s {
            "stepped" => [Curve::Stepped; N],
            _ => [Curve::Linear; N],
        });
    }

    let Some(arr) = value.as_array() else {
        return Ok([Curve::Linear; N]);
    };

    let expected = 4 * N;
    if arr.len() != expected {
        return Err(Error::JsonInvalidCurve {
            context: context.to_string(),
            message: format!("expected {expected} numbers, got {}", arr.len()),
        });
    }

    let mut out = [Curve::Linear; N];
    for value_index in 0..N {
        let base = value_index * 4;
        let cx1 = curve_number(&arr[base]).ok_or_else(|| Error::JsonInvalidCurve {
            context: context.to_string(),
            message: format!("curve[{base}] must be a number"),
        })?;
        let cy1 = curve_number(&arr[base + 1]).ok_or_else(|| Error::JsonInvalidCurve {
            context: context.to_string(),
            message: format!("curve[{}] must be a number", base + 1),
        })? * scales[value_index];
        let cx2 = curve_number(&arr[base + 2]).ok_or_else(|| Error::JsonInvalidCurve {
            context: context.to_string(),
            message: format!("curve[{}] must be a number", base + 2),
        })?;
        let cy2 = curve_number(&arr[base + 3]).ok_or_else(|| Error::JsonInvalidCurve {
            context: context.to_string(),
            message: format!("curve[{}] must be a number", base + 3),
        })? * scales[value_index];
        out[value_index] = Curve::Bezier { cx1, cy1, cx2, cy2 };
    }

    Ok(out)
}

fn parse_weighted_mesh_vertices(
    raw: &[f32],
    vertex_count: usize,
    bone_count: usize,
    scale: f32,
    skin: &str,
    slot: &str,
    attachment: &str,
) -> Result<Vec<Vec<crate::VertexWeight>>, Error> {
    fn expect_int(value: f32) -> Option<usize> {
        if !value.is_finite() {
            return None;
        }
        let rounded = value.round();
        if (value - rounded).abs() > 1.0e-4 {
            return None;
        }
        if rounded < 0.0 {
            return None;
        }
        Some(rounded as usize)
    }

    let mut cursor = 0usize;
    let mut out = Vec::with_capacity(vertex_count);
    for _ in 0..vertex_count {
        let Some(&n_raw) = raw.get(cursor) else {
            return Err(Error::JsonInvalidMeshData {
                skin: skin.to_string(),
                slot: slot.to_string(),
                attachment: attachment.to_string(),
                message: "unexpected end of weighted vertices".to_string(),
            });
        };
        cursor += 1;
        let n = expect_int(n_raw).ok_or_else(|| Error::JsonInvalidMeshData {
            skin: skin.to_string(),
            slot: slot.to_string(),
            attachment: attachment.to_string(),
            message: "invalid bone count in weighted vertices".to_string(),
        })?;
        if n == 0 {
            out.push(Vec::new());
            continue;
        }

        let mut weights = Vec::with_capacity(n);
        for _ in 0..n {
            let slice = raw
                .get(cursor..cursor + 4)
                .ok_or_else(|| Error::JsonInvalidMeshData {
                    skin: skin.to_string(),
                    slot: slot.to_string(),
                    attachment: attachment.to_string(),
                    message: "unexpected end of weighted vertices".to_string(),
                })?;
            cursor += 4;

            let bone = expect_int(slice[0]).ok_or_else(|| Error::JsonInvalidMeshData {
                skin: skin.to_string(),
                slot: slot.to_string(),
                attachment: attachment.to_string(),
                message: "invalid bone index in weighted vertices".to_string(),
            })?;
            if bone >= bone_count {
                return Err(Error::JsonInvalidMeshData {
                    skin: skin.to_string(),
                    slot: slot.to_string(),
                    attachment: attachment.to_string(),
                    message: "bone index out of range in weighted vertices".to_string(),
                });
            }
            weights.push(crate::VertexWeight {
                bone,
                x: slice[1] * scale,
                y: slice[2] * scale,
                weight: slice[3],
            });
        }
        out.push(weights);
    }

    if cursor != raw.len() {
        // Accept extra fields? For now treat as an error to keep parsing strict.
        return Err(Error::JsonInvalidMeshData {
            skin: skin.to_string(),
            slot: slot.to_string(),
            attachment: attachment.to_string(),
            message: "unexpected extra data in weighted vertices".to_string(),
        });
    }

    Ok(out)
}

fn resolve_linked_mesh_parent(
    skins: &HashMap<String, SkinData>,
    source_slot_index: usize,
    explicit_skin: Option<&str>,
    source: &str,
) -> Option<(String, String)> {
    if let Some(parent_skin_name) = explicit_skin.filter(|skin| !skin.is_empty()) {
        return skins
            .get(parent_skin_name)
            .and_then(|skin| {
                find_linked_mesh_attachment_name(skin, source_slot_index, source).map(
                    |(attachment_name, _)| (parent_skin_name.to_string(), attachment_name.clone()),
                )
            })
            .or_else(|| Some((parent_skin_name.to_string(), source.to_string())));
    }

    if let Some(default_skin) = skins.get("default")
        && default_skin.attachment(source_slot_index, source).is_some()
    {
        return Some(("default".to_string(), source.to_string()));
    }

    let mut prefix_matches = skins
        .keys()
        .filter_map(|skin_name| {
            source
                .strip_prefix(skin_name)
                .and_then(|rest| rest.strip_prefix('/'))
                .map(|attachment_name| (skin_name.as_str(), attachment_name))
        })
        .collect::<Vec<_>>();
    prefix_matches.sort_unstable_by(|a, b| b.0.len().cmp(&a.0.len()).then_with(|| a.0.cmp(b.0)));
    for (skin_name, attachment_name) in prefix_matches {
        if skins
            .get(skin_name)
            .and_then(|skin| skin.attachment(source_slot_index, attachment_name))
            .is_some()
        {
            return Some((skin_name.to_string(), attachment_name.to_string()));
        }
    }

    let mut name_matches = skins
        .iter()
        .filter_map(|(skin_name, skin)| {
            find_linked_mesh_attachment_name(skin, source_slot_index, source)
                .map(|(attachment_name, _)| (skin_name.as_str(), attachment_name.as_str()))
        })
        .collect::<Vec<_>>();
    name_matches.sort_unstable();
    name_matches.first().map(|(skin_name, attachment_name)| {
        ((*skin_name).to_string(), (*attachment_name).to_string())
    })
}

fn find_linked_mesh_attachment_name<'a>(
    skin: &'a SkinData,
    slot_index: usize,
    source: &str,
) -> Option<(&'a String, &'a AttachmentData)> {
    skin.attachments.get(slot_index).and_then(|slot_map| {
        slot_map.get_key_value(source).or_else(|| {
            slot_map
                .iter()
                .find(|(_, attachment)| attachment.name() == source)
        })
    })
}

fn validate_spine_version(value: &str) -> Result<(), Error> {
    if !crate::version::spine_version_matches_runtime(value) {
        return Err(Error::JsonSpineVersion {
            value: value.to_string(),
        });
    }
    Ok(())
}

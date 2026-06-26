use indexmap::IndexMap;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct BoneData {
    pub(crate) index: usize,
    pub(crate) name: String,
    pub(crate) parent: Option<usize>,
    pub(crate) length: f32,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) rotation: f32,
    pub(crate) scale_x: f32,
    pub(crate) scale_y: f32,
    pub(crate) shear_x: f32,
    pub(crate) shear_y: f32,
    pub(crate) inherit: Inherit,
    pub(crate) skin_required: bool,
    pub(crate) color: [f32; 4],
    pub(crate) icon: String,
    pub(crate) icon_size: f32,
    pub(crate) icon_rotation: f32,
    pub(crate) visible: bool,
}

impl BoneData {
    pub fn get_index(&self) -> usize {
        self.index
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_length(&self) -> f32 {
        self.length
    }

    pub fn set_length(&mut self, length: f32) {
        self.length = length;
    }

    pub fn get_skin_required(&self) -> bool {
        self.skin_required
    }

    pub fn set_skin_required(&mut self, skin_required: bool) {
        self.skin_required = skin_required;
    }

    pub fn get_color(&self) -> [f32; 4] {
        self.color
    }

    pub fn get_icon(&self) -> &str {
        self.icon.as_str()
    }

    pub fn set_icon(&mut self, icon: impl Into<String>) {
        self.icon = icon.into();
    }

    pub fn get_icon_size(&self) -> f32 {
        self.icon_size
    }

    pub fn set_icon_size(&mut self, icon_size: f32) {
        self.icon_size = icon_size;
    }

    pub fn get_icon_rotation(&self) -> f32 {
        self.icon_rotation
    }

    pub fn set_icon_rotation(&mut self, icon_rotation: f32) {
        self.icon_rotation = icon_rotation;
    }

    pub fn get_visible(&self) -> bool {
        self.visible
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}

impl Default for BoneData {
    fn default() -> Self {
        Self {
            index: 0,
            name: String::new(),
            parent: None,
            length: 0.0,
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            shear_x: 0.0,
            shear_y: 0.0,
            inherit: Inherit::Normal,
            skin_required: false,
            color: [0.61, 0.61, 0.61, 1.0],
            icon: String::new(),
            icon_size: 1.0,
            icon_rotation: 0.0,
            visible: true,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
pub enum Inherit {
    #[default]
    Normal,
    OnlyTranslation,
    NoRotationOrReflection,
    NoScale,
    NoScaleOrReflection,
}

#[derive(Clone, Debug)]
pub struct SlotData {
    pub(crate) index: usize,
    pub(crate) name: String,
    pub(crate) bone: usize,
    pub(crate) attachment: Option<String>,
    pub(crate) setup_pose: SlotSetupPose,
    pub(crate) blend: BlendMode,
    pub(crate) visible: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct SlotSetupPose {
    pub(crate) color: [f32; 4],
    pub(crate) has_dark: bool,
    pub(crate) dark_color: [f32; 3],
    pub(crate) sequence_index: i32,
}

impl Default for SlotSetupPose {
    fn default() -> Self {
        Self {
            color: [1.0, 1.0, 1.0, 1.0],
            has_dark: false,
            dark_color: [0.0, 0.0, 0.0],
            sequence_index: 0,
        }
    }
}

impl SlotData {
    pub fn get_index(&self) -> usize {
        self.index
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_attachment_name(&self) -> &str {
        self.attachment.as_deref().unwrap_or("")
    }

    pub fn set_attachment_name(&mut self, attachment_name: impl Into<String>) {
        let attachment_name = attachment_name.into();
        self.attachment = if attachment_name.is_empty() {
            None
        } else {
            Some(attachment_name)
        };
    }

    pub fn get_blend_mode(&self) -> BlendMode {
        self.blend
    }

    pub fn set_blend_mode(&mut self, blend: BlendMode) {
        self.blend = blend;
    }

    pub fn get_visible(&self) -> bool {
        self.visible
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}

impl Default for SlotData {
    fn default() -> Self {
        Self {
            index: 0,
            name: String::new(),
            bone: 0,
            attachment: None,
            setup_pose: SlotSetupPose::default(),
            blend: BlendMode::Normal,
            visible: true,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub enum BlendMode {
    #[default]
    Normal,
    Additive,
    Multiply,
    Screen,
}

#[derive(Clone, Debug)]
pub struct IkConstraintData {
    pub(crate) name: String,
    pub(crate) order: i32,
    pub(crate) skin_required: bool,
    pub(crate) bones: Vec<usize>,
    pub(crate) target: usize,
    pub(crate) scale_y_mode: ScaleYMode,
    pub(crate) mix: f32,
    pub(crate) softness: f32,
    pub(crate) compress: bool,
    pub(crate) stretch: bool,
    pub(crate) bend_direction: i32,
}

impl IkConstraintData {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            order: 0,
            skin_required: false,
            bones: Vec::new(),
            target: 0,
            scale_y_mode: ScaleYMode::None,
            mix: 0.0,
            softness: 0.0,
            compress: false,
            stretch: false,
            bend_direction: 0,
        }
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_skin_required(&self) -> bool {
        self.skin_required
    }

    pub fn set_skin_required(&mut self, skin_required: bool) {
        self.skin_required = skin_required;
    }

    pub fn get_bones(&self) -> &[usize] {
        &self.bones
    }

    pub fn get_bones_mut(&mut self) -> &mut Vec<usize> {
        &mut self.bones
    }

    pub fn get_target(&self) -> usize {
        self.target
    }

    pub fn set_target(&mut self, target: usize) {
        self.target = target;
    }

    pub fn get_scale_y_mode(&self) -> ScaleYMode {
        self.scale_y_mode
    }

    pub fn set_scale_y_mode(&mut self, scale_y_mode: ScaleYMode) {
        self.scale_y_mode = scale_y_mode;
    }

    pub fn get_mix(&self) -> f32 {
        self.mix
    }

    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix;
    }

    pub fn get_softness(&self) -> f32 {
        self.softness
    }

    pub fn set_softness(&mut self, softness: f32) {
        self.softness = softness;
    }

    pub fn get_bend_direction(&self) -> i32 {
        self.bend_direction
    }

    pub fn set_bend_direction(&mut self, bend_direction: i32) {
        self.bend_direction = bend_direction;
    }

    pub fn get_compress(&self) -> bool {
        self.compress
    }

    pub fn set_compress(&mut self, compress: bool) {
        self.compress = compress;
    }

    pub fn get_stretch(&self) -> bool {
        self.stretch
    }

    pub fn set_stretch(&mut self, stretch: bool) {
        self.stretch = stretch;
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ScaleYMode {
    #[default]
    None,
    Uniform,
    Volume,
}

impl ScaleYMode {
    #[cfg(feature = "json")]
    pub(crate) fn from_name(name: &str) -> Self {
        match name {
            "uniform" => Self::Uniform,
            "volume" => Self::Volume,
            _ => Self::None,
        }
    }

    #[cfg(feature = "binary")]
    pub(crate) fn from_binary(value: u8) -> Self {
        match value {
            1 => Self::Uniform,
            2 => Self::Volume,
            _ => Self::None,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum TransformProperty {
    Rotate,
    X,
    Y,
    ScaleX,
    ScaleY,
    ShearY,
}

impl TransformProperty {
    pub(crate) fn index(self) -> usize {
        match self {
            Self::Rotate => 0,
            Self::X => 1,
            Self::Y => 2,
            Self::ScaleX => 3,
            Self::ScaleY => 4,
            Self::ShearY => 5,
        }
    }

    #[cfg(feature = "json")]
    pub(crate) fn from_json_name(name: &str) -> Option<Self> {
        match name {
            "rotate" => Some(Self::Rotate),
            "x" => Some(Self::X),
            "y" => Some(Self::Y),
            "scaleX" => Some(Self::ScaleX),
            "scaleY" => Some(Self::ScaleY),
            "shearY" => Some(Self::ShearY),
            _ => None,
        }
    }

    #[cfg(feature = "binary")]
    pub(crate) fn from_binary_kind(kind: u8) -> Option<Self> {
        match kind {
            0 => Some(Self::Rotate),
            1 => Some(Self::X),
            2 => Some(Self::Y),
            3 => Some(Self::ScaleX),
            4 => Some(Self::ScaleY),
            5 => Some(Self::ShearY),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TransformToProperty {
    pub property: TransformProperty,
    pub offset: f32,
    pub max: f32,
    pub scale: f32,
}

#[derive(Clone, Debug)]
pub struct TransformFromProperty {
    pub property: TransformProperty,
    pub offset: f32,
    pub to: Vec<TransformToProperty>,
}

#[derive(Clone, Debug)]
pub struct TransformConstraintData {
    pub(crate) name: String,
    pub(crate) order: i32,
    pub(crate) skin_required: bool,
    pub(crate) bones: Vec<usize>,
    pub(crate) source: usize,
    pub(crate) local_source: bool,
    pub(crate) local_target: bool,
    pub(crate) additive: bool,
    pub(crate) clamp: bool,

    /// [rotate, x, y, scaleX, scaleY, shearY]
    pub(crate) offsets: [f32; 6],
    pub(crate) properties: Vec<TransformFromProperty>,

    pub(crate) mix_rotate: f32,
    pub(crate) mix_x: f32,
    pub(crate) mix_y: f32,
    pub(crate) mix_scale_x: f32,
    pub(crate) mix_scale_y: f32,
    pub(crate) mix_shear_y: f32,
}

impl TransformConstraintData {
    pub const ROTATION: usize = 0;
    pub const X: usize = 1;
    pub const Y: usize = 2;
    pub const SCALE_X: usize = 3;
    pub const SCALE_Y: usize = 4;
    pub const SHEAR_Y: usize = 5;

    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            order: 0,
            skin_required: false,
            bones: Vec::new(),
            source: 0,
            local_source: false,
            local_target: false,
            additive: false,
            clamp: false,
            offsets: [0.0; 6],
            properties: Vec::new(),
            mix_rotate: 0.0,
            mix_x: 0.0,
            mix_y: 0.0,
            mix_scale_x: 0.0,
            mix_scale_y: 0.0,
            mix_shear_y: 0.0,
        }
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_skin_required(&self) -> bool {
        self.skin_required
    }

    pub fn set_skin_required(&mut self, skin_required: bool) {
        self.skin_required = skin_required;
    }

    pub fn get_bones(&self) -> &[usize] {
        &self.bones
    }

    pub fn get_bones_mut(&mut self) -> &mut Vec<usize> {
        &mut self.bones
    }

    pub fn get_source(&self) -> usize {
        self.source
    }

    pub fn set_source(&mut self, source: usize) {
        self.source = source;
    }

    pub fn get_offset_rotation(&self) -> f32 {
        self.offsets[Self::ROTATION]
    }

    pub fn set_offset_rotation(&mut self, offset_rotation: f32) {
        self.offsets[Self::ROTATION] = offset_rotation;
    }

    pub fn get_offset_x(&self) -> f32 {
        self.offsets[Self::X]
    }

    pub fn set_offset_x(&mut self, offset_x: f32) {
        self.offsets[Self::X] = offset_x;
    }

    pub fn get_offset_y(&self) -> f32 {
        self.offsets[Self::Y]
    }

    pub fn set_offset_y(&mut self, offset_y: f32) {
        self.offsets[Self::Y] = offset_y;
    }

    pub fn get_offset_scale_x(&self) -> f32 {
        self.offsets[Self::SCALE_X]
    }

    pub fn set_offset_scale_x(&mut self, offset_scale_x: f32) {
        self.offsets[Self::SCALE_X] = offset_scale_x;
    }

    pub fn get_offset_scale_y(&self) -> f32 {
        self.offsets[Self::SCALE_Y]
    }

    pub fn set_offset_scale_y(&mut self, offset_scale_y: f32) {
        self.offsets[Self::SCALE_Y] = offset_scale_y;
    }

    pub fn get_offset_shear_y(&self) -> f32 {
        self.offsets[Self::SHEAR_Y]
    }

    pub fn set_offset_shear_y(&mut self, offset_shear_y: f32) {
        self.offsets[Self::SHEAR_Y] = offset_shear_y;
    }

    pub fn get_local_source(&self) -> bool {
        self.local_source
    }

    pub fn set_local_source(&mut self, local_source: bool) {
        self.local_source = local_source;
    }

    pub fn get_local_target(&self) -> bool {
        self.local_target
    }

    pub fn set_local_target(&mut self, local_target: bool) {
        self.local_target = local_target;
    }

    pub fn get_additive(&self) -> bool {
        self.additive
    }

    pub fn set_additive(&mut self, additive: bool) {
        self.additive = additive;
    }

    pub fn get_clamp(&self) -> bool {
        self.clamp
    }

    pub fn set_clamp(&mut self, clamp: bool) {
        self.clamp = clamp;
    }

    pub fn get_properties(&self) -> &[TransformFromProperty] {
        &self.properties
    }

    pub fn get_properties_mut(&mut self) -> &mut Vec<TransformFromProperty> {
        &mut self.properties
    }

    pub fn get_mix_rotate(&self) -> f32 {
        self.mix_rotate
    }

    pub fn set_mix_rotate(&mut self, mix_rotate: f32) {
        self.mix_rotate = mix_rotate;
    }

    pub fn get_mix_x(&self) -> f32 {
        self.mix_x
    }

    pub fn set_mix_x(&mut self, mix_x: f32) {
        self.mix_x = mix_x;
    }

    pub fn get_mix_y(&self) -> f32 {
        self.mix_y
    }

    pub fn set_mix_y(&mut self, mix_y: f32) {
        self.mix_y = mix_y;
    }

    pub fn get_mix_scale_x(&self) -> f32 {
        self.mix_scale_x
    }

    pub fn set_mix_scale_x(&mut self, mix_scale_x: f32) {
        self.mix_scale_x = mix_scale_x;
    }

    pub fn get_mix_scale_y(&self) -> f32 {
        self.mix_scale_y
    }

    pub fn set_mix_scale_y(&mut self, mix_scale_y: f32) {
        self.mix_scale_y = mix_scale_y;
    }

    pub fn get_mix_shear_y(&self) -> f32 {
        self.mix_shear_y
    }

    pub fn set_mix_shear_y(&mut self, mix_shear_y: f32) {
        self.mix_shear_y = mix_shear_y;
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PositionMode {
    Fixed,
    Percent,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SpacingMode {
    Length,
    Fixed,
    Percent,
    Proportional,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RotateMode {
    Tangent,
    Chain,
    ChainScale,
}

#[derive(Clone, Debug)]
pub struct PathConstraintData {
    pub(crate) name: String,
    pub(crate) order: i32,
    pub(crate) bones: Vec<usize>,
    pub(crate) target: usize,
    pub(crate) position_mode: PositionMode,
    pub(crate) spacing_mode: SpacingMode,
    pub(crate) rotate_mode: RotateMode,
    pub(crate) offset_rotation: f32,
    pub(crate) position: f32,
    pub(crate) spacing: f32,
    pub(crate) mix_rotate: f32,
    pub(crate) mix_x: f32,
    pub(crate) mix_y: f32,
    pub(crate) skin_required: bool,
}

impl PathConstraintData {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            order: 0,
            bones: Vec::new(),
            target: 0,
            position_mode: PositionMode::Fixed,
            spacing_mode: SpacingMode::Length,
            rotate_mode: RotateMode::Tangent,
            offset_rotation: 0.0,
            position: 0.0,
            spacing: 0.0,
            mix_rotate: 0.0,
            mix_x: 0.0,
            mix_y: 0.0,
            skin_required: false,
        }
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_skin_required(&self) -> bool {
        self.skin_required
    }

    pub fn set_skin_required(&mut self, skin_required: bool) {
        self.skin_required = skin_required;
    }

    pub fn get_bones(&self) -> &[usize] {
        &self.bones
    }

    pub fn get_bones_mut(&mut self) -> &mut Vec<usize> {
        &mut self.bones
    }

    pub fn get_slot(&self) -> usize {
        self.target
    }

    pub fn set_slot(&mut self, slot: usize) {
        self.target = slot;
    }

    pub fn get_position_mode(&self) -> PositionMode {
        self.position_mode
    }

    pub fn set_position_mode(&mut self, position_mode: PositionMode) {
        self.position_mode = position_mode;
    }

    pub fn get_spacing_mode(&self) -> SpacingMode {
        self.spacing_mode
    }

    pub fn set_spacing_mode(&mut self, spacing_mode: SpacingMode) {
        self.spacing_mode = spacing_mode;
    }

    pub fn get_rotate_mode(&self) -> RotateMode {
        self.rotate_mode
    }

    pub fn set_rotate_mode(&mut self, rotate_mode: RotateMode) {
        self.rotate_mode = rotate_mode;
    }

    pub fn get_offset_rotation(&self) -> f32 {
        self.offset_rotation
    }

    pub fn set_offset_rotation(&mut self, offset_rotation: f32) {
        self.offset_rotation = offset_rotation;
    }

    pub fn get_position(&self) -> f32 {
        self.position
    }

    pub fn set_position(&mut self, position: f32) {
        self.position = position;
    }

    pub fn get_spacing(&self) -> f32 {
        self.spacing
    }

    pub fn set_spacing(&mut self, spacing: f32) {
        self.spacing = spacing;
    }

    pub fn get_mix_rotate(&self) -> f32 {
        self.mix_rotate
    }

    pub fn set_mix_rotate(&mut self, mix_rotate: f32) {
        self.mix_rotate = mix_rotate;
    }

    pub fn get_mix_x(&self) -> f32 {
        self.mix_x
    }

    pub fn set_mix_x(&mut self, mix_x: f32) {
        self.mix_x = mix_x;
    }

    pub fn get_mix_y(&self) -> f32 {
        self.mix_y
    }

    pub fn set_mix_y(&mut self, mix_y: f32) {
        self.mix_y = mix_y;
    }
}

#[derive(Clone, Debug)]
pub struct PhysicsConstraintData {
    pub(crate) name: String,
    pub(crate) order: i32,
    pub(crate) skin_required: bool,
    pub(crate) bone: usize,

    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) rotate: f32,
    pub(crate) scale_x: f32,
    pub(crate) scale_y_mode: ScaleYMode,
    pub(crate) shear_x: f32,
    pub(crate) limit: f32,
    pub(crate) step: f32,

    pub(crate) inertia: f32,
    pub(crate) strength: f32,
    pub(crate) damping: f32,
    pub(crate) mass_inverse: f32,
    pub(crate) wind: f32,
    pub(crate) gravity: f32,
    pub(crate) mix: f32,

    pub(crate) inertia_global: bool,
    pub(crate) strength_global: bool,
    pub(crate) damping_global: bool,
    pub(crate) mass_global: bool,
    pub(crate) wind_global: bool,
    pub(crate) gravity_global: bool,
    pub(crate) mix_global: bool,
}

impl PhysicsConstraintData {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            order: 0,
            skin_required: false,
            bone: 0,
            x: 0.0,
            y: 0.0,
            rotate: 0.0,
            scale_x: 0.0,
            scale_y_mode: ScaleYMode::None,
            shear_x: 0.0,
            limit: 0.0,
            step: 0.0,
            inertia: 0.0,
            strength: 0.0,
            damping: 0.0,
            mass_inverse: 0.0,
            wind: 0.0,
            gravity: 0.0,
            mix: 0.0,
            inertia_global: false,
            strength_global: false,
            damping_global: false,
            mass_global: false,
            wind_global: false,
            gravity_global: false,
            mix_global: false,
        }
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_skin_required(&self) -> bool {
        self.skin_required
    }

    pub fn set_skin_required(&mut self, skin_required: bool) {
        self.skin_required = skin_required;
    }

    pub fn get_bone(&self) -> usize {
        self.bone
    }

    pub fn set_bone(&mut self, bone: usize) {
        self.bone = bone;
    }

    pub fn get_step(&self) -> f32 {
        self.step
    }

    pub fn set_step(&mut self, step: f32) {
        self.step = step;
    }

    pub fn get_x(&self) -> f32 {
        self.x
    }

    pub fn set_x(&mut self, x: f32) {
        self.x = x;
    }

    pub fn get_y(&self) -> f32 {
        self.y
    }

    pub fn set_y(&mut self, y: f32) {
        self.y = y;
    }

    pub fn get_rotate(&self) -> f32 {
        self.rotate
    }

    pub fn set_rotate(&mut self, rotate: f32) {
        self.rotate = rotate;
    }

    pub fn get_scale_x(&self) -> f32 {
        self.scale_x
    }

    pub fn set_scale_x(&mut self, scale_x: f32) {
        self.scale_x = scale_x;
    }

    pub fn get_scale_y_mode(&self) -> ScaleYMode {
        self.scale_y_mode
    }

    pub fn set_scale_y_mode(&mut self, scale_y_mode: ScaleYMode) {
        self.scale_y_mode = scale_y_mode;
    }

    pub fn get_shear_x(&self) -> f32 {
        self.shear_x
    }

    pub fn set_shear_x(&mut self, shear_x: f32) {
        self.shear_x = shear_x;
    }

    pub fn get_limit(&self) -> f32 {
        self.limit
    }

    pub fn set_limit(&mut self, limit: f32) {
        self.limit = limit;
    }

    pub fn get_inertia(&self) -> f32 {
        self.inertia
    }

    pub fn set_inertia(&mut self, inertia: f32) {
        self.inertia = inertia;
    }

    pub fn get_strength(&self) -> f32 {
        self.strength
    }

    pub fn set_strength(&mut self, strength: f32) {
        self.strength = strength;
    }

    pub fn get_damping(&self) -> f32 {
        self.damping
    }

    pub fn set_damping(&mut self, damping: f32) {
        self.damping = damping;
    }

    pub fn get_mass_inverse(&self) -> f32 {
        self.mass_inverse
    }

    pub fn set_mass_inverse(&mut self, mass_inverse: f32) {
        self.mass_inverse = mass_inverse;
    }

    pub fn get_wind(&self) -> f32 {
        self.wind
    }

    pub fn set_wind(&mut self, wind: f32) {
        self.wind = wind;
    }

    pub fn get_gravity(&self) -> f32 {
        self.gravity
    }

    pub fn set_gravity(&mut self, gravity: f32) {
        self.gravity = gravity;
    }

    pub fn get_mix(&self) -> f32 {
        self.mix
    }

    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix;
    }

    pub fn get_inertia_global(&self) -> bool {
        self.inertia_global
    }

    pub fn set_inertia_global(&mut self, inertia_global: bool) {
        self.inertia_global = inertia_global;
    }

    pub fn get_strength_global(&self) -> bool {
        self.strength_global
    }

    pub fn set_strength_global(&mut self, strength_global: bool) {
        self.strength_global = strength_global;
    }

    pub fn get_damping_global(&self) -> bool {
        self.damping_global
    }

    pub fn set_damping_global(&mut self, damping_global: bool) {
        self.damping_global = damping_global;
    }

    pub fn get_mass_global(&self) -> bool {
        self.mass_global
    }

    pub fn set_mass_global(&mut self, mass_global: bool) {
        self.mass_global = mass_global;
    }

    pub fn get_wind_global(&self) -> bool {
        self.wind_global
    }

    pub fn set_wind_global(&mut self, wind_global: bool) {
        self.wind_global = wind_global;
    }

    pub fn get_gravity_global(&self) -> bool {
        self.gravity_global
    }

    pub fn set_gravity_global(&mut self, gravity_global: bool) {
        self.gravity_global = gravity_global;
    }

    pub fn get_mix_global(&self) -> bool {
        self.mix_global
    }

    pub fn set_mix_global(&mut self, mix_global: bool) {
        self.mix_global = mix_global;
    }
}

#[derive(Clone, Debug)]
pub struct SliderConstraintData {
    pub(crate) name: String,
    pub(crate) order: i32,
    pub(crate) skin_required: bool,

    pub(crate) setup_time: f32,
    pub(crate) setup_mix: f32,

    pub(crate) additive: bool,
    pub(crate) looped: bool,

    pub(crate) bone: Option<usize>,
    pub(crate) property: Option<TransformProperty>,
    pub(crate) property_from: f32,
    pub(crate) to: f32,
    pub(crate) scale: f32,
    pub(crate) local: bool,

    pub(crate) animation: Option<usize>,
}

impl SliderConstraintData {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            order: 0,
            skin_required: false,
            setup_time: 0.0,
            setup_mix: 0.0,
            additive: false,
            looped: false,
            bone: None,
            property: None,
            property_from: 0.0,
            to: 0.0,
            scale: 0.0,
            local: false,
            animation: None,
        }
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_skin_required(&self) -> bool {
        self.skin_required
    }

    pub fn set_skin_required(&mut self, skin_required: bool) {
        self.skin_required = skin_required;
    }

    pub fn get_animation(&self) -> Option<usize> {
        self.animation
    }

    pub fn set_animation(&mut self, animation: usize) {
        self.animation = Some(animation);
    }

    pub fn clear_animation(&mut self) {
        self.animation = None;
    }

    pub fn get_additive(&self) -> bool {
        self.additive
    }

    pub fn set_additive(&mut self, additive: bool) {
        self.additive = additive;
    }

    pub fn get_loop(&self) -> bool {
        self.looped
    }

    pub fn set_loop(&mut self, looped: bool) {
        self.looped = looped;
    }

    pub fn get_bone(&self) -> Option<usize> {
        self.bone
    }

    pub fn set_bone(&mut self, bone: Option<usize>) {
        self.bone = bone;
    }

    pub fn get_property(&self) -> Option<TransformProperty> {
        self.property
    }

    pub fn set_property(&mut self, property: Option<TransformProperty>) {
        self.property = property;
    }

    pub fn get_offset(&self) -> f32 {
        self.property_from
    }

    pub fn set_offset(&mut self, offset: f32) {
        self.property_from = offset;
    }

    pub fn get_scale(&self) -> f32 {
        self.scale
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }

    pub fn get_max(&self) -> f32 {
        self.to
    }

    pub fn set_max(&mut self, max: f32) {
        self.to = max;
    }

    pub fn get_local(&self) -> bool {
        self.local
    }

    pub fn set_local(&mut self, local: bool) {
        self.local = local;
    }

    pub fn get_time(&self) -> f32 {
        self.setup_time
    }

    pub fn set_time(&mut self, time: f32) {
        self.setup_time = time;
    }

    pub fn get_mix(&self) -> f32 {
        self.setup_mix
    }

    pub fn set_mix(&mut self, mix: f32) {
        self.setup_mix = mix;
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ConstraintDataRef<'a> {
    Ik(&'a IkConstraintData),
    Transform(&'a TransformConstraintData),
    Path(&'a PathConstraintData),
    Physics(&'a PhysicsConstraintData),
    Slider(&'a SliderConstraintData),
}

impl ConstraintDataRef<'_> {
    pub fn get_name(&self) -> &str {
        match self {
            ConstraintDataRef::Ik(data) => data.name.as_str(),
            ConstraintDataRef::Transform(data) => data.name.as_str(),
            ConstraintDataRef::Path(data) => data.name.as_str(),
            ConstraintDataRef::Physics(data) => data.name.as_str(),
            ConstraintDataRef::Slider(data) => data.name.as_str(),
        }
    }

    pub fn get_skin_required(&self) -> bool {
        match self {
            ConstraintDataRef::Ik(data) => data.skin_required,
            ConstraintDataRef::Transform(data) => data.skin_required,
            ConstraintDataRef::Path(data) => data.skin_required,
            ConstraintDataRef::Physics(data) => data.skin_required,
            ConstraintDataRef::Slider(data) => data.skin_required,
        }
    }

    pub fn get_order(&self) -> i32 {
        match self {
            ConstraintDataRef::Ik(data) => data.order,
            ConstraintDataRef::Transform(data) => data.order,
            ConstraintDataRef::Path(data) => data.order,
            ConstraintDataRef::Physics(data) => data.order,
            ConstraintDataRef::Slider(data) => data.order,
        }
    }
}

#[derive(Clone, Debug)]
pub struct IkFrame {
    pub time: f32,
    pub mix: f32,
    pub softness: f32,
    pub bend_direction: i32,
    pub compress: bool,
    pub stretch: bool,
    pub curve: [Curve; 2],
}

#[derive(Clone, Debug)]
pub struct IkConstraintTimeline {
    pub constraint_index: usize,
    pub frames: Vec<IkFrame>,
}

#[derive(Clone, Debug)]
pub struct TransformFrame {
    pub time: f32,
    pub mix_rotate: f32,
    pub mix_x: f32,
    pub mix_y: f32,
    pub mix_scale_x: f32,
    pub mix_scale_y: f32,
    pub mix_shear_y: f32,
    pub curve: [Curve; 6],
}

#[derive(Clone, Debug)]
pub struct TransformConstraintTimeline {
    pub constraint_index: usize,
    pub frames: Vec<TransformFrame>,
}

#[derive(Clone, Debug)]
pub struct FloatFrame {
    pub time: f32,
    pub value: f32,
    pub curve: Curve,
}

#[derive(Clone, Debug)]
pub struct PhysicsConstraintResetTimeline {
    /// -1 means apply to all constraints (per upstream semantics).
    pub constraint_index: i32,
    pub frames: Vec<f32>,
}

#[derive(Clone, Debug)]
pub struct PhysicsConstraintFloatTimeline {
    /// -1 means apply to all constraints (per upstream semantics).
    pub constraint_index: i32,
    pub frames: Vec<FloatFrame>,
}

#[derive(Clone, Debug)]
pub enum PhysicsConstraintTimeline {
    Inertia(PhysicsConstraintFloatTimeline),
    Strength(PhysicsConstraintFloatTimeline),
    Damping(PhysicsConstraintFloatTimeline),
    Mass(PhysicsConstraintFloatTimeline),
    Wind(PhysicsConstraintFloatTimeline),
    Gravity(PhysicsConstraintFloatTimeline),
    Mix(PhysicsConstraintFloatTimeline),
}

#[derive(Clone, Debug)]
pub struct PathConstraintPositionTimeline {
    pub constraint_index: usize,
    pub frames: Vec<FloatFrame>,
}

#[derive(Clone, Debug)]
pub struct PathConstraintSpacingTimeline {
    pub constraint_index: usize,
    pub frames: Vec<FloatFrame>,
}

#[derive(Clone, Debug)]
pub struct PathMixFrame {
    pub time: f32,
    pub mix_rotate: f32,
    pub mix_x: f32,
    pub mix_y: f32,
    pub curve: [Curve; 3],
}

#[derive(Clone, Debug)]
pub struct PathConstraintMixTimeline {
    pub constraint_index: usize,
    pub frames: Vec<PathMixFrame>,
}

#[derive(Clone, Debug)]
pub enum PathConstraintTimeline {
    Position(PathConstraintPositionTimeline),
    Spacing(PathConstraintSpacingTimeline),
    Mix(PathConstraintMixTimeline),
}

#[derive(Clone, Debug)]
pub struct SliderConstraintTimeline {
    pub constraint_index: usize,
    pub frames: Vec<FloatFrame>,
}

#[derive(Clone, Debug)]
pub struct RegionAttachmentData {
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) sequence: Option<SequenceDef>,
    pub(crate) color: [f32; 4],
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) rotation: f32,
    pub(crate) scale_x: f32,
    pub(crate) scale_y: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
}

impl RegionAttachmentData {
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_path(&self) -> &str {
        self.path.as_str()
    }

    pub fn set_path(&mut self, path: impl Into<String>) {
        self.path = path.into();
    }

    pub fn get_color(&self) -> [f32; 4] {
        self.color
    }

    pub fn get_color_mut(&mut self) -> &mut [f32; 4] {
        &mut self.color
    }

    pub fn get_x(&self) -> f32 {
        self.x
    }

    pub fn set_x(&mut self, x: f32) {
        self.x = x;
    }

    pub fn get_y(&self) -> f32 {
        self.y
    }

    pub fn set_y(&mut self, y: f32) {
        self.y = y;
    }

    pub fn get_scale_x(&self) -> f32 {
        self.scale_x
    }

    pub fn set_scale_x(&mut self, scale_x: f32) {
        self.scale_x = scale_x;
    }

    pub fn get_scale_y(&self) -> f32 {
        self.scale_y
    }

    pub fn set_scale_y(&mut self, scale_y: f32) {
        self.scale_y = scale_y;
    }

    pub fn get_rotation(&self) -> f32 {
        self.rotation
    }

    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation;
    }

    pub fn get_width(&self) -> f32 {
        self.width
    }

    pub fn set_width(&mut self, width: f32) {
        self.width = width;
    }

    pub fn get_height(&self) -> f32 {
        self.height
    }

    pub fn set_height(&mut self, height: f32) {
        self.height = height;
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SequenceDef {
    pub(crate) id: u32,
    pub(crate) count: usize,
    pub(crate) start: i32,
    pub(crate) digits: usize,
    pub(crate) setup_index: i32,
}

#[derive(Clone, Debug)]
pub struct MeshAttachmentData {
    pub(crate) vertex_id: u32,
    pub(crate) name: String,
    pub(crate) path: String,
    /// For deform timelines, Spine runtimes match on `timelineAttachment` (linked meshes may inherit from a parent mesh).
    /// This stores the `(skin, attachmentKey)` of the mesh used as the deform timeline target.
    pub(crate) timeline_skin: String,
    pub(crate) timeline_attachment: String,
    pub(crate) timeline_slots: Vec<usize>,
    pub(crate) sequence: Option<SequenceDef>,
    pub(crate) color: [f32; 4],
    pub(crate) vertices: MeshVertices,
    pub(crate) uvs: Vec<[f32; 2]>,
    pub(crate) triangles: Vec<u32>,
    pub(crate) hull_length: usize,
    pub(crate) edges: Vec<u32>,
    pub(crate) width: f32,
    pub(crate) height: f32,
}

impl MeshAttachmentData {
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_region_uvs(&self) -> &[[f32; 2]] {
        &self.uvs
    }

    pub fn get_region_uvs_mut(&mut self) -> &mut Vec<[f32; 2]> {
        &mut self.uvs
    }

    pub fn get_triangles(&self) -> &[u32] {
        &self.triangles
    }

    pub fn get_triangles_mut(&mut self) -> &mut Vec<u32> {
        &mut self.triangles
    }

    pub fn get_hull_length(&self) -> usize {
        self.hull_length
    }

    pub fn set_hull_length(&mut self, hull_length: usize) {
        self.hull_length = hull_length;
    }

    pub fn get_path(&self) -> &str {
        self.path.as_str()
    }

    pub fn set_path(&mut self, path: impl Into<String>) {
        self.path = path.into();
    }

    pub fn get_color(&self) -> [f32; 4] {
        self.color
    }

    pub fn get_color_mut(&mut self) -> &mut [f32; 4] {
        &mut self.color
    }

    pub fn get_edges(&self) -> &[u32] {
        &self.edges
    }

    pub fn get_edges_mut(&mut self) -> &mut Vec<u32> {
        &mut self.edges
    }

    pub fn get_width(&self) -> f32 {
        self.width
    }

    pub fn set_width(&mut self, width: f32) {
        self.width = width;
    }

    pub fn get_height(&self) -> f32 {
        self.height
    }

    pub fn set_height(&mut self, height: f32) {
        self.height = height;
    }
}

#[derive(Clone, Debug)]
pub(crate) struct VertexWeight {
    pub(crate) bone: usize,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) weight: f32,
}

#[derive(Clone, Debug)]
pub(crate) enum MeshVertices {
    Unweighted(Vec<[f32; 2]>),
    Weighted(Vec<Vec<VertexWeight>>),
}

#[derive(Clone, Debug)]
pub enum AttachmentData {
    Region(RegionAttachmentData),
    Mesh(MeshAttachmentData),
    Point(PointAttachmentData),
    Path(PathAttachmentData),
    BoundingBox(BoundingBoxAttachmentData),
    Clipping(ClippingAttachmentData),
}

impl AttachmentData {
    pub fn get_name(&self) -> &str {
        match self {
            AttachmentData::Region(a) => a.name.as_str(),
            AttachmentData::Mesh(a) => a.name.as_str(),
            AttachmentData::Point(a) => a.name.as_str(),
            AttachmentData::Path(a) => a.name.as_str(),
            AttachmentData::BoundingBox(a) => a.name.as_str(),
            AttachmentData::Clipping(a) => a.name.as_str(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PointAttachmentData {
    pub(crate) name: String,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) rotation: f32,
    pub(crate) color: [f32; 4],
}

impl PointAttachmentData {
    pub const DEFAULT_COLOR: [f32; 4] = [0.9451, 0.9451, 0.0, 1.0];

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_x(&self) -> f32 {
        self.x
    }

    pub fn set_x(&mut self, x: f32) {
        self.x = x;
    }

    pub fn get_y(&self) -> f32 {
        self.y
    }

    pub fn set_y(&mut self, y: f32) {
        self.y = y;
    }

    pub fn get_rotation(&self) -> f32 {
        self.rotation
    }

    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation;
    }

    pub fn get_color(&self) -> [f32; 4] {
        self.color
    }

    pub fn get_color_mut(&mut self) -> &mut [f32; 4] {
        &mut self.color
    }
}

#[derive(Clone, Debug)]
pub struct PathAttachmentData {
    pub(crate) vertex_id: u32,
    pub(crate) name: String,
    pub(crate) color: [f32; 4],
    pub(crate) vertices: MeshVertices,
    pub(crate) lengths: Vec<f32>,
    pub(crate) closed: bool,
    pub(crate) constant_speed: bool,
}

impl PathAttachmentData {
    pub const DEFAULT_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 0.0];

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_lengths(&self) -> &[f32] {
        &self.lengths
    }

    pub fn get_lengths_mut(&mut self) -> &mut Vec<f32> {
        &mut self.lengths
    }

    pub fn get_closed(&self) -> bool {
        self.closed
    }

    pub fn set_closed(&mut self, closed: bool) {
        self.closed = closed;
    }

    pub fn get_constant_speed(&self) -> bool {
        self.constant_speed
    }

    pub fn set_constant_speed(&mut self, constant_speed: bool) {
        self.constant_speed = constant_speed;
    }

    pub fn get_color(&self) -> [f32; 4] {
        self.color
    }

    pub fn get_color_mut(&mut self) -> &mut [f32; 4] {
        &mut self.color
    }
}

#[derive(Clone, Debug)]
pub struct BoundingBoxAttachmentData {
    pub(crate) vertex_id: u32,
    pub(crate) name: String,
    pub(crate) color: [f32; 4],
    pub(crate) vertices: MeshVertices,
}

impl BoundingBoxAttachmentData {
    pub const DEFAULT_COLOR: [f32; 4] = [0.38, 0.94, 0.0, 1.0];

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_color(&self) -> [f32; 4] {
        self.color
    }

    pub fn get_color_mut(&mut self) -> &mut [f32; 4] {
        &mut self.color
    }
}

#[derive(Clone, Debug)]
pub struct ClippingAttachmentData {
    pub(crate) vertex_id: u32,
    pub(crate) name: String,
    pub(crate) color: [f32; 4],
    pub(crate) vertices: MeshVertices,
    pub(crate) end_slot: Option<usize>,
    pub(crate) convex: bool,
    pub(crate) inverse: bool,
}

impl ClippingAttachmentData {
    pub const DEFAULT_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 0.0];

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_end_slot(&self) -> Option<usize> {
        self.end_slot
    }

    pub fn set_end_slot(&mut self, end_slot: Option<usize>) {
        self.end_slot = end_slot;
    }

    pub fn get_convex(&self) -> bool {
        self.convex
    }

    pub fn set_convex(&mut self, convex: bool) {
        self.convex = convex;
    }

    pub fn get_inverse(&self) -> bool {
        self.inverse
    }

    pub fn set_inverse(&mut self, inverse: bool) {
        self.inverse = inverse;
    }

    pub fn get_color(&self) -> [f32; 4] {
        self.color
    }

    pub fn get_color_mut(&mut self) -> &mut [f32; 4] {
        &mut self.color
    }
}

#[derive(Clone, Debug)]
pub struct SkinData {
    pub(crate) name: String,
    pub(crate) color: [f32; 4],
    pub(crate) attachments: Vec<IndexMap<String, AttachmentData>>,
    pub(crate) bones: Vec<usize>,
    pub(crate) ik_constraints: Vec<usize>,
    pub(crate) transform_constraints: Vec<usize>,
    pub(crate) path_constraints: Vec<usize>,
    pub(crate) physics_constraints: Vec<usize>,
    pub(crate) slider_constraints: Vec<usize>,
}

impl SkinData {
    pub const DEFAULT_COLOR: [f32; 4] = [0.99607843, 0.61960787, 0.30980393, 1.0];

    /// Creates an empty skin.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            color: Self::DEFAULT_COLOR,
            attachments: Vec::new(),
            bones: Vec::new(),
            ik_constraints: Vec::new(),
            transform_constraints: Vec::new(),
            path_constraints: Vec::new(),
            physics_constraints: Vec::new(),
            slider_constraints: Vec::new(),
        }
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_color(&self) -> [f32; 4] {
        self.color
    }

    pub fn get_attachments(&self) -> &[IndexMap<String, AttachmentData>] {
        &self.attachments
    }

    pub fn get_bones(&self) -> &[usize] {
        &self.bones
    }

    /// Merges `other` into `self` (union of bones/constraints + last-write-wins attachments).
    ///
    /// Mirrors the behaviour of the official runtimes' `Skin::addSkin`.
    pub fn add_skin(&mut self, other: &SkinData) {
        fn push_unique(target: &mut Vec<usize>, values: &[usize]) {
            for &v in values {
                if !target.contains(&v) {
                    target.push(v);
                }
            }
        }

        push_unique(&mut self.bones, &other.bones);
        push_unique(&mut self.ik_constraints, &other.ik_constraints);
        push_unique(
            &mut self.transform_constraints,
            &other.transform_constraints,
        );
        push_unique(&mut self.path_constraints, &other.path_constraints);
        push_unique(&mut self.physics_constraints, &other.physics_constraints);
        push_unique(&mut self.slider_constraints, &other.slider_constraints);

        if self.attachments.len() < other.attachments.len() {
            self.attachments.extend(
                (0..(other.attachments.len() - self.attachments.len())).map(|_| IndexMap::new()),
            );
        }
        for (slot_index, slot_map) in other.attachments.iter().enumerate() {
            for (key, attachment) in slot_map {
                self.set_attachment(slot_index, key.clone(), attachment.clone());
            }
        }
    }

    /// Copies all attachments, bones, and constraints from `other` into this skin.
    ///
    /// Mirrors `Skin::copySkin`. Attachments are owned data in this Rust model, so this uses
    /// the same merge path as `add_skin`.
    pub fn copy_skin(&mut self, other: &SkinData) {
        self.add_skin(other);
    }

    /// Adds or replaces an attachment for the specified slot index and name.
    ///
    /// Mirrors the official runtimes' `Skin::setAttachment`, including growing the internal
    /// slot bucket storage when the slot index is beyond the current skin capacity.
    pub fn set_attachment(
        &mut self,
        slot_index: usize,
        attachment_name: impl Into<String>,
        attachment: AttachmentData,
    ) {
        if self.attachments.len() <= slot_index {
            self.attachments.resize_with(slot_index + 1, IndexMap::new);
        }
        self.attachments[slot_index].insert(attachment_name.into(), attachment);
    }

    pub fn get_attachment(
        &self,
        slot_index: usize,
        attachment_name: &str,
    ) -> Option<&AttachmentData> {
        self.attachments
            .get(slot_index)
            .and_then(|slot_map| slot_map.get(attachment_name))
    }

    pub fn find_names_for_slot(&self, slot_index: usize, names: &mut Vec<String>) {
        if let Some(slot_map) = self.attachments.get(slot_index) {
            names.extend(slot_map.keys().cloned());
        }
    }

    pub fn find_attachments_for_slot<'a>(
        &'a self,
        slot_index: usize,
        attachments: &mut Vec<&'a AttachmentData>,
    ) {
        if let Some(slot_map) = self.attachments.get(slot_index) {
            attachments.extend(slot_map.values());
        }
    }

    /// Removes an attachment from the skin. Missing slots or names are no-ops, matching C++.
    pub fn remove_attachment(&mut self, slot_index: usize, attachment_name: &str) {
        if let Some(slot_map) = self.attachments.get_mut(slot_index) {
            slot_map.shift_remove(attachment_name);
        }
    }
}

#[derive(Clone, Debug)]
pub struct EventData {
    name: String,
    audio_path: String,
    setup_pose: EventSetupPose,
}

impl EventData {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            audio_path: String::new(),
            setup_pose: EventSetupPose::default(),
        }
    }

    pub(crate) fn with_setup_pose(
        name: impl Into<String>,
        int_value: i32,
        float_value: f32,
        string: impl Into<String>,
        audio_path: impl Into<String>,
        volume: f32,
        balance: f32,
    ) -> Self {
        Self {
            name: name.into(),
            audio_path: audio_path.into(),
            setup_pose: EventSetupPose {
                int_value,
                float_value,
                string: string.into(),
                volume,
                balance,
            },
        }
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_setup_pose(&self) -> &EventSetupPose {
        &self.setup_pose
    }

    pub fn get_setup_pose_mut(&mut self) -> &mut EventSetupPose {
        &mut self.setup_pose
    }

    pub fn get_audio_path(&self) -> &str {
        self.audio_path.as_str()
    }

    pub fn set_audio_path(&mut self, audio_path: impl Into<String>) {
        self.audio_path = audio_path.into();
    }
}

#[derive(Clone, Debug, Default)]
pub struct EventSetupPose {
    int_value: i32,
    float_value: f32,
    string: String,
    volume: f32,
    balance: f32,
}

impl EventSetupPose {
    pub fn get_int(&self) -> i32 {
        self.int_value
    }

    pub fn set_int(&mut self, int_value: i32) {
        self.int_value = int_value;
    }

    pub fn get_float(&self) -> f32 {
        self.float_value
    }

    pub fn set_float(&mut self, float_value: f32) {
        self.float_value = float_value;
    }

    pub fn get_string(&self) -> &str {
        self.string.as_str()
    }

    pub fn set_string(&mut self, string: impl Into<String>) {
        self.string = string.into();
    }

    pub fn get_volume(&self) -> f32 {
        self.volume
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }

    pub fn get_balance(&self) -> f32 {
        self.balance
    }

    pub fn set_balance(&mut self, balance: f32) {
        self.balance = balance;
    }
}

#[derive(Clone, Debug)]
pub struct Event {
    data: Arc<EventData>,
    time: f32,
    int_value: i32,
    float_value: f32,
    string: String,
    volume: f32,
    balance: f32,
}

impl Event {
    pub fn new(time: f32, data: Arc<EventData>) -> Self {
        Self {
            data,
            time,
            int_value: 0,
            float_value: 0.0,
            string: String::new(),
            volume: 0.0,
            balance: 0.0,
        }
    }

    pub fn get_data(&self) -> &EventData {
        &self.data
    }

    pub fn get_time(&self) -> f32 {
        self.time
    }

    pub fn get_int(&self) -> i32 {
        self.int_value
    }

    pub fn set_int(&mut self, int_value: i32) {
        self.int_value = int_value;
    }

    pub fn get_float(&self) -> f32 {
        self.float_value
    }

    pub fn set_float(&mut self, float_value: f32) {
        self.float_value = float_value;
    }

    pub fn get_string(&self) -> &str {
        self.string.as_str()
    }

    pub fn set_string(&mut self, string: impl Into<String>) {
        self.string = string.into();
    }

    pub fn get_volume(&self) -> f32 {
        self.volume
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }

    pub fn get_balance(&self) -> f32 {
        self.balance
    }

    pub fn set_balance(&mut self, balance: f32) {
        self.balance = balance;
    }
}

#[derive(Clone, Debug)]
pub struct EventTimeline {
    events: Vec<Event>,
}

impl EventTimeline {
    pub fn from_events(events: Vec<Event>) -> Self {
        Self { events }
    }

    pub fn get_frame_count(&self) -> usize {
        self.events.len()
    }

    pub fn get_events(&self) -> &[Event] {
        &self.events
    }

    pub fn set_frame(&mut self, frame: usize, event: Event) {
        if frame == self.events.len() {
            self.events.push(event);
        } else {
            self.events[frame] = event;
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Curve {
    Linear,
    Stepped,
    Bezier {
        cx1: f32,
        cy1: f32,
        cx2: f32,
        cy2: f32,
    },
}

#[derive(Clone, Debug)]
pub struct DeformFrame {
    pub time: f32,
    pub vertices: Vec<f32>,
    pub curve: Curve,
}

#[derive(Clone, Debug)]
pub struct DeformTimeline {
    pub skin: String,
    pub slot_index: usize,
    pub attachment: String,
    pub vertex_count: usize,
    pub setup_vertices: Option<Vec<f32>>,
    pub frames: Vec<DeformFrame>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SequenceMode {
    Hold,
    Once,
    Loop,
    PingPong,
    OnceReverse,
    LoopReverse,
    PingPongReverse,
}

#[derive(Clone, Debug)]
pub struct SequenceFrame {
    pub time: f32,
    pub mode: SequenceMode,
    pub index: i32,
    pub delay: f32,
}

#[derive(Clone, Debug)]
pub struct SequenceTimeline {
    pub skin: String,
    pub slot_index: usize,
    pub attachment: String,
    pub frames: Vec<SequenceFrame>,
}

#[derive(Clone, Debug)]
pub struct AttachmentFrame {
    pub time: f32,
    pub name: Option<String>,
}

#[derive(Clone, Debug)]
pub struct AttachmentTimeline {
    pub slot_index: usize,
    pub frames: Vec<AttachmentFrame>,
}

#[derive(Clone, Debug)]
pub struct DrawOrderFrame {
    pub time: f32,
    pub draw_order_to_setup_index: Option<Vec<usize>>,
}

#[derive(Clone, Debug)]
pub struct DrawOrderTimeline {
    pub frames: Vec<DrawOrderFrame>,
}

#[derive(Clone, Debug)]
pub struct DrawOrderFolderFrame {
    pub time: f32,
    pub folder_draw_order: Option<Vec<usize>>,
}

#[derive(Clone, Debug)]
pub struct DrawOrderFolderTimeline {
    pub slots: Vec<usize>,
    pub frames: Vec<DrawOrderFolderFrame>,
}

#[derive(Clone, Debug)]
pub struct ColorFrame {
    pub time: f32,
    pub color: [f32; 4],
    pub curve: [Curve; 4],
}

#[derive(Clone, Debug)]
pub struct ColorTimeline {
    pub slot_index: usize,
    pub frames: Vec<ColorFrame>,
}

#[derive(Clone, Debug)]
pub struct RgbFrame {
    pub time: f32,
    pub color: [f32; 3],
    pub curve: [Curve; 3],
}

#[derive(Clone, Debug)]
pub struct RgbTimeline {
    pub slot_index: usize,
    pub frames: Vec<RgbFrame>,
}

#[derive(Clone, Debug)]
pub struct AlphaFrame {
    pub time: f32,
    pub alpha: f32,
    pub curve: Curve,
}

#[derive(Clone, Debug)]
pub struct AlphaTimeline {
    pub slot_index: usize,
    pub frames: Vec<AlphaFrame>,
}

#[derive(Clone, Debug)]
pub struct Rgba2Frame {
    pub time: f32,
    pub light: [f32; 4],
    pub dark: [f32; 3],
    pub curve: [Curve; 7],
}

#[derive(Clone, Debug)]
pub struct Rgba2Timeline {
    pub slot_index: usize,
    pub frames: Vec<Rgba2Frame>,
}

#[derive(Clone, Debug)]
pub struct Rgb2Frame {
    pub time: f32,
    pub light: [f32; 3],
    pub dark: [f32; 3],
    pub curve: [Curve; 6],
}

#[derive(Clone, Debug)]
pub struct Rgb2Timeline {
    pub slot_index: usize,
    pub frames: Vec<Rgb2Frame>,
}

#[derive(Clone, Debug)]
pub struct RotateFrame {
    pub time: f32,
    pub angle: f32,
    pub curve: Curve,
}

#[derive(Clone, Debug)]
pub struct RotateTimeline {
    pub bone_index: usize,
    pub frames: Vec<RotateFrame>,
}

#[derive(Clone, Debug)]
pub struct Vec2Frame {
    pub time: f32,
    pub x: f32,
    pub y: f32,
    pub curve: [Curve; 2],
}

#[derive(Clone, Debug)]
pub struct TranslateTimeline {
    pub bone_index: usize,
    pub frames: Vec<Vec2Frame>,
}

#[derive(Clone, Debug)]
pub struct TranslateXTimeline {
    pub bone_index: usize,
    pub frames: Vec<FloatFrame>,
}

#[derive(Clone, Debug)]
pub struct TranslateYTimeline {
    pub bone_index: usize,
    pub frames: Vec<FloatFrame>,
}

#[derive(Clone, Debug)]
pub struct ScaleTimeline {
    pub bone_index: usize,
    pub frames: Vec<Vec2Frame>,
}

#[derive(Clone, Debug)]
pub struct ScaleXTimeline {
    pub bone_index: usize,
    pub frames: Vec<FloatFrame>,
}

#[derive(Clone, Debug)]
pub struct ScaleYTimeline {
    pub bone_index: usize,
    pub frames: Vec<FloatFrame>,
}

#[derive(Clone, Debug)]
pub struct ShearTimeline {
    pub bone_index: usize,
    pub frames: Vec<Vec2Frame>,
}

#[derive(Clone, Debug)]
pub struct ShearXTimeline {
    pub bone_index: usize,
    pub frames: Vec<FloatFrame>,
}

#[derive(Clone, Debug)]
pub struct ShearYTimeline {
    pub bone_index: usize,
    pub frames: Vec<FloatFrame>,
}

#[derive(Clone, Debug)]
pub struct InheritFrame {
    pub time: f32,
    pub inherit: Inherit,
}

#[derive(Clone, Debug)]
pub struct InheritTimeline {
    pub bone_index: usize,
    pub frames: Vec<InheritFrame>,
}

#[derive(Clone, Debug)]
pub enum BoneTimeline {
    Rotate(RotateTimeline),
    Translate(TranslateTimeline),
    TranslateX(TranslateXTimeline),
    TranslateY(TranslateYTimeline),
    Scale(ScaleTimeline),
    ScaleX(ScaleXTimeline),
    ScaleY(ScaleYTimeline),
    Shear(ShearTimeline),
    ShearX(ShearXTimeline),
    ShearY(ShearYTimeline),
    Inherit(InheritTimeline),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TimelineKind {
    SlotAttachment(usize),
    Deform(usize),
    Sequence(usize),
    Bone(usize),
    SlotColor(usize),
    SlotRgb(usize),
    SlotAlpha(usize),
    SlotRgba2(usize),
    SlotRgb2(usize),
    IkConstraint(usize),
    TransformConstraint(usize),
    PathConstraint(usize),
    PhysicsConstraint(usize),
    PhysicsReset(usize),
    SliderTime(usize),
    SliderMix(usize),
    DrawOrder,
    DrawOrderFolder(usize),
}

#[derive(Clone, Copy, Debug)]
pub enum TimelineRef<'a> {
    Event {
        timeline: &'a EventTimeline,
    },
    SlotAttachment {
        index: usize,
        timeline: &'a AttachmentTimeline,
    },
    Deform {
        index: usize,
        timeline: &'a DeformTimeline,
    },
    Sequence {
        index: usize,
        timeline: &'a SequenceTimeline,
    },
    Bone {
        index: usize,
        timeline: &'a BoneTimeline,
    },
    SlotColor {
        index: usize,
        timeline: &'a ColorTimeline,
    },
    SlotRgb {
        index: usize,
        timeline: &'a RgbTimeline,
    },
    SlotAlpha {
        index: usize,
        timeline: &'a AlphaTimeline,
    },
    SlotRgba2 {
        index: usize,
        timeline: &'a Rgba2Timeline,
    },
    SlotRgb2 {
        index: usize,
        timeline: &'a Rgb2Timeline,
    },
    IkConstraint {
        index: usize,
        timeline: &'a IkConstraintTimeline,
    },
    TransformConstraint {
        index: usize,
        timeline: &'a TransformConstraintTimeline,
    },
    PathConstraint {
        index: usize,
        timeline: &'a PathConstraintTimeline,
    },
    PhysicsConstraint {
        index: usize,
        timeline: &'a PhysicsConstraintTimeline,
    },
    PhysicsReset {
        index: usize,
        timeline: &'a PhysicsConstraintResetTimeline,
    },
    SliderTime {
        index: usize,
        timeline: &'a SliderConstraintTimeline,
    },
    SliderMix {
        index: usize,
        timeline: &'a SliderConstraintTimeline,
    },
    DrawOrder {
        timeline: &'a DrawOrderTimeline,
    },
    DrawOrderFolder {
        index: usize,
        timeline: &'a DrawOrderFolderTimeline,
    },
}

#[derive(Clone, Debug)]
pub struct Animation {
    pub(crate) name: String,
    pub(crate) duration: f32,
    pub(crate) color: [f32; 4],
    pub(crate) event_timeline: Option<EventTimeline>,
    pub(crate) bone_timelines: Vec<BoneTimeline>,
    pub(crate) deform_timelines: Vec<DeformTimeline>,
    pub(crate) sequence_timelines: Vec<SequenceTimeline>,
    pub(crate) slot_attachment_timelines: Vec<AttachmentTimeline>,
    pub(crate) slot_color_timelines: Vec<ColorTimeline>,
    pub(crate) slot_rgb_timelines: Vec<RgbTimeline>,
    pub(crate) slot_alpha_timelines: Vec<AlphaTimeline>,
    pub(crate) slot_rgba2_timelines: Vec<Rgba2Timeline>,
    pub(crate) slot_rgb2_timelines: Vec<Rgb2Timeline>,
    pub(crate) ik_constraint_timelines: Vec<IkConstraintTimeline>,
    pub(crate) transform_constraint_timelines: Vec<TransformConstraintTimeline>,
    pub(crate) path_constraint_timelines: Vec<PathConstraintTimeline>,
    pub(crate) physics_constraint_timelines: Vec<PhysicsConstraintTimeline>,
    pub(crate) physics_reset_timelines: Vec<PhysicsConstraintResetTimeline>,
    pub(crate) slider_time_timelines: Vec<SliderConstraintTimeline>,
    pub(crate) slider_mix_timelines: Vec<SliderConstraintTimeline>,
    pub(crate) draw_order_timeline: Option<DrawOrderTimeline>,
    pub(crate) draw_order_folder_timelines: Vec<DrawOrderFolderTimeline>,
    pub(crate) timeline_order: Vec<TimelineKind>,
}

impl Animation {
    pub const DEFAULT_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            duration: 0.0,
            color: Self::DEFAULT_COLOR,
            event_timeline: None,
            bone_timelines: Vec::new(),
            deform_timelines: Vec::new(),
            sequence_timelines: Vec::new(),
            slot_attachment_timelines: Vec::new(),
            slot_color_timelines: Vec::new(),
            slot_rgb_timelines: Vec::new(),
            slot_alpha_timelines: Vec::new(),
            slot_rgba2_timelines: Vec::new(),
            slot_rgb2_timelines: Vec::new(),
            ik_constraint_timelines: Vec::new(),
            transform_constraint_timelines: Vec::new(),
            path_constraint_timelines: Vec::new(),
            physics_constraint_timelines: Vec::new(),
            physics_reset_timelines: Vec::new(),
            slider_time_timelines: Vec::new(),
            slider_mix_timelines: Vec::new(),
            draw_order_timeline: None,
            draw_order_folder_timelines: Vec::new(),
            timeline_order: Vec::new(),
        }
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_duration(&self) -> f32 {
        self.duration
    }

    pub fn set_duration(&mut self, duration: f32) {
        self.duration = duration;
    }

    pub fn get_color(&self) -> [f32; 4] {
        self.color
    }

    pub fn get_color_mut(&mut self) -> &mut [f32; 4] {
        &mut self.color
    }

    /// Returns the `Skeleton::get_bones()` indices affected by this animation.
    ///
    /// This matches C++ `Animation::getBones()` for callers such as Slider constraints. The
    /// returned indices preserve the first-seen bone timeline order.
    pub fn get_bones(&self) -> Vec<usize> {
        let mut bones = Vec::new();
        for timeline in &self.bone_timelines {
            let bone_index = timeline.get_bone_index();
            if !bones.contains(&bone_index) {
                bones.push(bone_index);
            }
        }
        bones
    }

    pub fn get_timelines(&self) -> impl Iterator<Item = TimelineRef<'_>> + '_ {
        self.timeline_order
            .iter()
            .filter_map(|kind| self.timeline_ref(*kind))
            .chain(
                self.event_timeline
                    .as_ref()
                    .map(|timeline| TimelineRef::Event { timeline }),
            )
    }

    pub(crate) fn timeline_order(&self) -> &[TimelineKind] {
        &self.timeline_order
    }

    fn timeline_ref(&self, kind: TimelineKind) -> Option<TimelineRef<'_>> {
        match kind {
            TimelineKind::SlotAttachment(index) => self
                .slot_attachment_timelines
                .get(index)
                .map(|timeline| TimelineRef::SlotAttachment { index, timeline }),
            TimelineKind::Deform(index) => self
                .deform_timelines
                .get(index)
                .map(|timeline| TimelineRef::Deform { index, timeline }),
            TimelineKind::Sequence(index) => self
                .sequence_timelines
                .get(index)
                .map(|timeline| TimelineRef::Sequence { index, timeline }),
            TimelineKind::Bone(index) => self
                .bone_timelines
                .get(index)
                .map(|timeline| TimelineRef::Bone { index, timeline }),
            TimelineKind::SlotColor(index) => self
                .slot_color_timelines
                .get(index)
                .map(|timeline| TimelineRef::SlotColor { index, timeline }),
            TimelineKind::SlotRgb(index) => self
                .slot_rgb_timelines
                .get(index)
                .map(|timeline| TimelineRef::SlotRgb { index, timeline }),
            TimelineKind::SlotAlpha(index) => self
                .slot_alpha_timelines
                .get(index)
                .map(|timeline| TimelineRef::SlotAlpha { index, timeline }),
            TimelineKind::SlotRgba2(index) => self
                .slot_rgba2_timelines
                .get(index)
                .map(|timeline| TimelineRef::SlotRgba2 { index, timeline }),
            TimelineKind::SlotRgb2(index) => self
                .slot_rgb2_timelines
                .get(index)
                .map(|timeline| TimelineRef::SlotRgb2 { index, timeline }),
            TimelineKind::IkConstraint(index) => self
                .ik_constraint_timelines
                .get(index)
                .map(|timeline| TimelineRef::IkConstraint { index, timeline }),
            TimelineKind::TransformConstraint(index) => self
                .transform_constraint_timelines
                .get(index)
                .map(|timeline| TimelineRef::TransformConstraint { index, timeline }),
            TimelineKind::PathConstraint(index) => self
                .path_constraint_timelines
                .get(index)
                .map(|timeline| TimelineRef::PathConstraint { index, timeline }),
            TimelineKind::PhysicsConstraint(index) => self
                .physics_constraint_timelines
                .get(index)
                .map(|timeline| TimelineRef::PhysicsConstraint { index, timeline }),
            TimelineKind::PhysicsReset(index) => self
                .physics_reset_timelines
                .get(index)
                .map(|timeline| TimelineRef::PhysicsReset { index, timeline }),
            TimelineKind::SliderTime(index) => self
                .slider_time_timelines
                .get(index)
                .map(|timeline| TimelineRef::SliderTime { index, timeline }),
            TimelineKind::SliderMix(index) => self
                .slider_mix_timelines
                .get(index)
                .map(|timeline| TimelineRef::SliderMix { index, timeline }),
            TimelineKind::DrawOrder => self
                .draw_order_timeline
                .as_ref()
                .map(|timeline| TimelineRef::DrawOrder { timeline }),
            TimelineKind::DrawOrderFolder(index) => self
                .draw_order_folder_timelines
                .get(index)
                .map(|timeline| TimelineRef::DrawOrderFolder { index, timeline }),
        }
    }
}

impl BoneTimeline {
    /// Returns the `Skeleton::get_bones()` index changed by this timeline.
    pub fn get_bone_index(&self) -> usize {
        match self {
            BoneTimeline::Rotate(timeline) => timeline.bone_index,
            BoneTimeline::Translate(timeline) => timeline.bone_index,
            BoneTimeline::TranslateX(timeline) => timeline.bone_index,
            BoneTimeline::TranslateY(timeline) => timeline.bone_index,
            BoneTimeline::Scale(timeline) => timeline.bone_index,
            BoneTimeline::ScaleX(timeline) => timeline.bone_index,
            BoneTimeline::ScaleY(timeline) => timeline.bone_index,
            BoneTimeline::Shear(timeline) => timeline.bone_index,
            BoneTimeline::ShearX(timeline) => timeline.bone_index,
            BoneTimeline::ShearY(timeline) => timeline.bone_index,
            BoneTimeline::Inherit(timeline) => timeline.bone_index,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SkeletonData {
    pub(crate) name: String,
    pub(crate) spine_version: Option<String>,
    pub(crate) hash: String,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) reference_scale: f32,
    pub(crate) fps: f32,
    pub(crate) images_path: String,
    pub(crate) audio_path: String,
    pub(crate) bones: Vec<BoneData>,
    pub(crate) slots: Vec<SlotData>,
    pub(crate) skins: Vec<SkinData>,
    pub(crate) events: Vec<EventData>,
    pub(crate) animations: Vec<Animation>,
    pub(crate) ik_constraints: Vec<IkConstraintData>,
    pub(crate) transform_constraints: Vec<TransformConstraintData>,
    pub(crate) path_constraints: Vec<PathConstraintData>,
    pub(crate) physics_constraints: Vec<PhysicsConstraintData>,
    pub(crate) slider_constraints: Vec<SliderConstraintData>,
}

impl SkeletonData {
    pub const DEFAULT_SKIN_NAME: &'static str = "default";
    pub const DEFAULT_REFERENCE_SCALE: f32 = 100.0;
    pub const DEFAULT_FPS: f32 = 30.0;

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_version(&self) -> Option<&str> {
        self.spine_version.as_deref()
    }

    pub fn get_hash(&self) -> &str {
        self.hash.as_str()
    }

    pub fn get_x(&self) -> f32 {
        self.x
    }

    pub fn get_y(&self) -> f32 {
        self.y
    }

    pub fn get_width(&self) -> f32 {
        self.width
    }

    pub fn get_height(&self) -> f32 {
        self.height
    }

    pub fn get_reference_scale(&self) -> f32 {
        self.reference_scale
    }

    pub fn get_fps(&self) -> f32 {
        self.fps
    }

    pub fn get_images_path(&self) -> &str {
        self.images_path.as_str()
    }

    pub fn get_audio_path(&self) -> &str {
        self.audio_path.as_str()
    }

    pub fn get_bones(&self) -> &[BoneData] {
        &self.bones
    }

    pub fn get_slots(&self) -> &[SlotData] {
        &self.slots
    }

    pub fn get_skins(&self) -> &[SkinData] {
        &self.skins
    }

    pub fn get_events(&self) -> &[EventData] {
        &self.events
    }

    pub fn get_animations(&self) -> &[Animation] {
        &self.animations
    }

    pub fn get_ik_constraints(&self) -> &[IkConstraintData] {
        &self.ik_constraints
    }

    pub fn get_transform_constraints(&self) -> &[TransformConstraintData] {
        &self.transform_constraints
    }

    pub fn get_path_constraints(&self) -> &[PathConstraintData] {
        &self.path_constraints
    }

    pub fn get_physics_constraints(&self) -> &[PhysicsConstraintData] {
        &self.physics_constraints
    }

    pub fn get_slider_constraints(&self) -> &[SliderConstraintData] {
        &self.slider_constraints
    }

    pub fn find_bone(&self, name: &str) -> Option<&BoneData> {
        self.bones.iter().find(|data| data.name == name)
    }

    pub fn find_slot(&self, name: &str) -> Option<&SlotData> {
        self.slots.iter().find(|data| data.name == name)
    }

    pub fn find_animation(&self, name: &str) -> Option<&Animation> {
        self.animations.iter().find(|data| data.name == name)
    }

    pub fn find_skin(&self, name: &str) -> Option<&SkinData> {
        self.skins.iter().find(|data| data.name == name)
    }

    pub fn find_event(&self, name: &str) -> Option<&EventData> {
        self.events.iter().find(|data| data.get_name() == name)
    }

    pub fn find_ik_constraint(&self, name: &str) -> Option<&IkConstraintData> {
        self.ik_constraints.iter().find(|data| data.name == name)
    }

    pub fn find_transform_constraint(&self, name: &str) -> Option<&TransformConstraintData> {
        self.transform_constraints
            .iter()
            .find(|data| data.name == name)
    }

    pub fn find_path_constraint(&self, name: &str) -> Option<&PathConstraintData> {
        self.path_constraints.iter().find(|data| data.name == name)
    }

    pub fn find_physics_constraint(&self, name: &str) -> Option<&PhysicsConstraintData> {
        self.physics_constraints
            .iter()
            .find(|data| data.name == name)
    }

    pub fn find_slider_constraint(&self, name: &str) -> Option<&SliderConstraintData> {
        self.slider_constraints
            .iter()
            .find(|data| data.name == name)
    }

    /// Returns the C++-style unified constraint data view in update order.
    pub fn get_constraints(&self) -> Vec<ConstraintDataRef<'_>> {
        let mut constraints = Vec::with_capacity(
            self.ik_constraints.len()
                + self.transform_constraints.len()
                + self.path_constraints.len()
                + self.physics_constraints.len()
                + self.slider_constraints.len(),
        );
        constraints.extend(
            self.ik_constraints
                .iter()
                .enumerate()
                .map(|(_, data)| ConstraintDataRef::Ik(data)),
        );
        constraints.extend(
            self.transform_constraints
                .iter()
                .enumerate()
                .map(|(_, data)| ConstraintDataRef::Transform(data)),
        );
        constraints.extend(
            self.path_constraints
                .iter()
                .enumerate()
                .map(|(_, data)| ConstraintDataRef::Path(data)),
        );
        constraints.extend(
            self.physics_constraints
                .iter()
                .enumerate()
                .map(|(_, data)| ConstraintDataRef::Physics(data)),
        );
        constraints.extend(
            self.slider_constraints
                .iter()
                .enumerate()
                .map(|(_, data)| ConstraintDataRef::Slider(data)),
        );
        constraints.sort_by_key(ConstraintDataRef::get_order);
        constraints
    }

    pub fn get_default_skin(&self) -> Option<&SkinData> {
        self.find_skin(Self::DEFAULT_SKIN_NAME)
    }

    pub fn find_slider_animations<'data, 'out>(
        &'data self,
        animations: &'out mut Vec<&'data Animation>,
    ) -> &'out mut Vec<&'data Animation> {
        animations.extend(self.slider_constraints.iter().filter_map(|constraint| {
            constraint
                .animation
                .and_then(|index| self.animations.get(index))
        }));
        animations
    }
}

impl Default for SkeletonData {
    fn default() -> Self {
        Self {
            name: String::new(),
            spine_version: None,
            hash: String::new(),
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            reference_scale: Self::DEFAULT_REFERENCE_SCALE,
            fps: Self::DEFAULT_FPS,
            images_path: String::new(),
            audio_path: String::new(),
            bones: Vec::new(),
            slots: Vec::new(),
            skins: Vec::new(),
            events: Vec::new(),
            animations: Vec::new(),
            ik_constraints: Vec::new(),
            transform_constraints: Vec::new(),
            path_constraints: Vec::new(),
            physics_constraints: Vec::new(),
            slider_constraints: Vec::new(),
        }
    }
}

#[cfg(any(test, feature = "json", feature = "binary"))]
pub(crate) fn timeline_order_for_animation(animation: &Animation) -> Vec<TimelineKind> {
    let mut timeline_order = Vec::new();
    timeline_order
        .extend((0..animation.slot_attachment_timelines.len()).map(TimelineKind::SlotAttachment));
    timeline_order.extend((0..animation.slot_color_timelines.len()).map(TimelineKind::SlotColor));
    timeline_order.extend((0..animation.slot_rgb_timelines.len()).map(TimelineKind::SlotRgb));
    timeline_order.extend((0..animation.slot_rgba2_timelines.len()).map(TimelineKind::SlotRgba2));
    timeline_order.extend((0..animation.slot_rgb2_timelines.len()).map(TimelineKind::SlotRgb2));
    timeline_order.extend((0..animation.slot_alpha_timelines.len()).map(TimelineKind::SlotAlpha));
    timeline_order.extend((0..animation.bone_timelines.len()).map(TimelineKind::Bone));
    timeline_order
        .extend((0..animation.ik_constraint_timelines.len()).map(TimelineKind::IkConstraint));
    timeline_order.extend(
        (0..animation.transform_constraint_timelines.len()).map(TimelineKind::TransformConstraint),
    );
    timeline_order
        .extend((0..animation.path_constraint_timelines.len()).map(TimelineKind::PathConstraint));
    timeline_order.extend(
        (0..animation.physics_constraint_timelines.len()).map(TimelineKind::PhysicsConstraint),
    );
    timeline_order
        .extend((0..animation.physics_reset_timelines.len()).map(TimelineKind::PhysicsReset));
    timeline_order.extend((0..animation.slider_time_timelines.len()).map(TimelineKind::SliderTime));
    timeline_order.extend((0..animation.slider_mix_timelines.len()).map(TimelineKind::SliderMix));
    timeline_order.extend((0..animation.deform_timelines.len()).map(TimelineKind::Deform));
    timeline_order.extend((0..animation.sequence_timelines.len()).map(TimelineKind::Sequence));
    if animation.draw_order_timeline.is_some() {
        timeline_order.push(TimelineKind::DrawOrder);
    }
    timeline_order.extend(
        (0..animation.draw_order_folder_timelines.len()).map(TimelineKind::DrawOrderFolder),
    );
    timeline_order
}

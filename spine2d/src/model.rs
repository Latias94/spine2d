use indexmap::IndexMap;

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
    pub name: String,
    pub order: i32,
    pub skin_required: bool,
    pub bones: Vec<usize>,
    pub target: usize,
    pub scale_y_mode: ScaleYMode,
    pub mix: f32,
    pub softness: f32,
    pub compress: bool,
    pub stretch: bool,
    pub bend_direction: i32,
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
    pub name: String,
    pub order: i32,
    pub skin_required: bool,
    pub bones: Vec<usize>,
    pub source: usize,
    pub local_source: bool,
    pub local_target: bool,
    pub additive: bool,
    pub clamp: bool,

    /// [rotate, x, y, scaleX, scaleY, shearY]
    pub offsets: [f32; 6],
    pub properties: Vec<TransformFromProperty>,

    pub mix_rotate: f32,
    pub mix_x: f32,
    pub mix_y: f32,
    pub mix_scale_x: f32,
    pub mix_scale_y: f32,
    pub mix_shear_y: f32,
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
    pub name: String,
    pub order: i32,
    pub bones: Vec<usize>,
    pub target: usize,
    pub position_mode: PositionMode,
    pub spacing_mode: SpacingMode,
    pub rotate_mode: RotateMode,
    pub offset_rotation: f32,
    pub position: f32,
    pub spacing: f32,
    pub mix_rotate: f32,
    pub mix_x: f32,
    pub mix_y: f32,
    pub skin_required: bool,
}

#[derive(Clone, Debug)]
pub struct PhysicsConstraintData {
    pub name: String,
    pub order: i32,
    pub skin_required: bool,
    pub bone: usize,

    pub x: f32,
    pub y: f32,
    pub rotate: f32,
    pub scale_x: f32,
    pub scale_y_mode: ScaleYMode,
    pub shear_x: f32,
    pub limit: f32,
    pub step: f32,

    pub inertia: f32,
    pub strength: f32,
    pub damping: f32,
    pub mass_inverse: f32,
    pub wind: f32,
    pub gravity: f32,
    pub mix: f32,

    pub inertia_global: bool,
    pub strength_global: bool,
    pub damping_global: bool,
    pub mass_global: bool,
    pub wind_global: bool,
    pub gravity_global: bool,
    pub mix_global: bool,
}

#[derive(Clone, Debug)]
pub struct SliderConstraintData {
    pub name: String,
    pub order: i32,
    pub skin_required: bool,

    pub setup_time: f32,
    pub setup_mix: f32,

    pub additive: bool,
    pub looped: bool,

    pub bone: Option<usize>,
    pub property: Option<TransformProperty>,
    pub property_from: f32,
    pub to: f32,
    pub scale: f32,
    pub local: bool,

    pub(crate) animation: Option<usize>,
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
    pub fn name(&self) -> &str {
        match self {
            ConstraintDataRef::Ik(data) => data.name.as_str(),
            ConstraintDataRef::Transform(data) => data.name.as_str(),
            ConstraintDataRef::Path(data) => data.name.as_str(),
            ConstraintDataRef::Physics(data) => data.name.as_str(),
            ConstraintDataRef::Slider(data) => data.name.as_str(),
        }
    }

    pub fn skin_required(&self) -> bool {
        match self {
            ConstraintDataRef::Ik(data) => data.skin_required,
            ConstraintDataRef::Transform(data) => data.skin_required,
            ConstraintDataRef::Path(data) => data.skin_required,
            ConstraintDataRef::Physics(data) => data.skin_required,
            ConstraintDataRef::Slider(data) => data.skin_required,
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
    pub name: String,
    pub path: String,
    pub sequence: Option<SequenceDef>,
    pub color: [f32; 4],
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Debug)]
pub struct SequenceDef {
    pub id: u32,
    pub count: usize,
    pub start: i32,
    pub digits: usize,
    pub setup_index: i32,
}

#[derive(Clone, Debug)]
pub struct MeshAttachmentData {
    pub vertex_id: u32,
    pub name: String,
    pub path: String,
    /// For deform timelines, Spine runtimes match on `timelineAttachment` (linked meshes may inherit from a parent mesh).
    /// This stores the `(skin, attachmentKey)` of the mesh used as the deform timeline target.
    pub timeline_skin: String,
    pub timeline_attachment: String,
    pub timeline_slots: Vec<usize>,
    pub sequence: Option<SequenceDef>,
    pub color: [f32; 4],
    pub vertices: MeshVertices,
    pub uvs: Vec<[f32; 2]>,
    pub triangles: Vec<u32>,
    pub hull_length: usize,
    pub edges: Vec<u32>,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Debug)]
pub struct VertexWeight {
    pub bone: usize,
    pub x: f32,
    pub y: f32,
    pub weight: f32,
}

#[derive(Clone, Debug)]
pub enum MeshVertices {
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
    pub fn name(&self) -> &str {
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
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub color: [f32; 4],
}

impl PointAttachmentData {
    pub const DEFAULT_COLOR: [f32; 4] = [0.9451, 0.9451, 0.0, 1.0];
}

#[derive(Clone, Debug)]
pub struct PathAttachmentData {
    pub vertex_id: u32,
    pub name: String,
    pub color: [f32; 4],
    pub vertices: MeshVertices,
    pub lengths: Vec<f32>,
    pub closed: bool,
    pub constant_speed: bool,
}

impl PathAttachmentData {
    pub const DEFAULT_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 0.0];
}

#[derive(Clone, Debug)]
pub struct BoundingBoxAttachmentData {
    pub vertex_id: u32,
    pub name: String,
    pub color: [f32; 4],
    pub vertices: MeshVertices,
}

impl BoundingBoxAttachmentData {
    pub const DEFAULT_COLOR: [f32; 4] = [0.38, 0.94, 0.0, 1.0];
}

#[derive(Clone, Debug)]
pub struct ClippingAttachmentData {
    pub vertex_id: u32,
    pub name: String,
    pub color: [f32; 4],
    pub vertices: MeshVertices,
    pub end_slot: Option<usize>,
    pub convex: bool,
    pub inverse: bool,
}

impl ClippingAttachmentData {
    pub const DEFAULT_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 0.0];
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
    pub name: String,
    pub int_value: i32,
    pub float_value: f32,
    pub string: String,
    pub audio_path: String,
    pub volume: f32,
    pub balance: f32,
}

#[derive(Clone, Debug)]
pub struct Event {
    pub time: f32,
    pub name: String,
    pub int_value: i32,
    pub float_value: f32,
    pub string: String,
    pub audio_path: String,
    pub volume: f32,
    pub balance: f32,
}

#[derive(Clone, Debug)]
pub struct EventTimeline {
    pub events: Vec<Event>,
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
pub enum TimelineKind {
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
    pub name: String,
    pub duration: f32,
    pub color: [f32; 4],
    pub event_timeline: Option<EventTimeline>,
    pub bone_timelines: Vec<BoneTimeline>,
    pub deform_timelines: Vec<DeformTimeline>,
    pub sequence_timelines: Vec<SequenceTimeline>,
    pub slot_attachment_timelines: Vec<AttachmentTimeline>,
    pub slot_color_timelines: Vec<ColorTimeline>,
    pub slot_rgb_timelines: Vec<RgbTimeline>,
    pub slot_alpha_timelines: Vec<AlphaTimeline>,
    pub slot_rgba2_timelines: Vec<Rgba2Timeline>,
    pub slot_rgb2_timelines: Vec<Rgb2Timeline>,
    pub ik_constraint_timelines: Vec<IkConstraintTimeline>,
    pub transform_constraint_timelines: Vec<TransformConstraintTimeline>,
    pub path_constraint_timelines: Vec<PathConstraintTimeline>,
    pub physics_constraint_timelines: Vec<PhysicsConstraintTimeline>,
    pub physics_reset_timelines: Vec<PhysicsConstraintResetTimeline>,
    pub slider_time_timelines: Vec<SliderConstraintTimeline>,
    pub slider_mix_timelines: Vec<SliderConstraintTimeline>,
    pub draw_order_timeline: Option<DrawOrderTimeline>,
    pub draw_order_folder_timelines: Vec<DrawOrderFolderTimeline>,
    pub(crate) timeline_order: Vec<TimelineKind>,
}

impl Animation {
    pub const DEFAULT_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

    /// Returns the `Skeleton::bones()` indices affected by this animation.
    ///
    /// This matches C++ `Animation::getBones()` for callers such as Slider constraints. The
    /// returned indices preserve the first-seen bone timeline order.
    pub fn bones(&self) -> Vec<usize> {
        let mut bones = Vec::new();
        for timeline in &self.bone_timelines {
            let bone_index = timeline.bone_index();
            if !bones.contains(&bone_index) {
                bones.push(bone_index);
            }
        }
        bones
    }

    pub fn timelines(&self) -> impl Iterator<Item = TimelineRef<'_>> + '_ {
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
    /// Returns the `Skeleton::bones()` index changed by this timeline.
    pub fn bone_index(&self) -> usize {
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
    pub(crate) skins: IndexMap<String, SkinData>,
    pub(crate) events: IndexMap<String, EventData>,
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

    pub fn get_spine_version(&self) -> Option<&str> {
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

    pub fn get_skins(&self) -> &IndexMap<String, SkinData> {
        &self.skins
    }

    pub fn get_events(&self) -> &IndexMap<String, EventData> {
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
        self.skins.values().find(|data| data.name == name)
    }

    pub fn find_event(&self, name: &str) -> Option<&EventData> {
        self.events.values().find(|data| data.name == name)
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
        constraints.sort_by_key(|constraint| match constraint {
            ConstraintDataRef::Ik(data) => data.order,
            ConstraintDataRef::Transform(data) => data.order,
            ConstraintDataRef::Path(data) => data.order,
            ConstraintDataRef::Physics(data) => data.order,
            ConstraintDataRef::Slider(data) => data.order,
        });
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
            skins: IndexMap::new(),
            events: IndexMap::new(),
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

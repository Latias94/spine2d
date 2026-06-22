#[derive(Clone, Debug)]
pub struct Slot {
    pub(super) data_index: usize,
    pub(crate) bone: usize,
    pub(crate) attachment: Option<String>,
    pub(crate) attachment_skin: Option<String>,
    pub(crate) attachment_state: i32,
    pub(crate) sequence_index: i32,
    pub(crate) deform: Vec<f32>,
    pub(crate) color: [f32; 4],
    pub(crate) has_dark: bool,
    pub(crate) dark_color: [f32; 3],
    pub(crate) blend: crate::BlendMode,
}

impl Slot {
    pub fn data_index(&self) -> usize {
        self.data_index
    }

    pub fn bone_index(&self) -> usize {
        self.bone
    }

    pub fn set_bone_index(&mut self, bone: usize) {
        self.bone = bone;
    }

    pub fn attachment_name(&self) -> Option<&str> {
        self.attachment.as_deref()
    }

    pub fn set_attachment_name(&mut self, attachment: Option<String>) {
        if self.attachment == attachment {
            return;
        }
        self.attachment = attachment;
        self.attachment_skin = None;
        self.deform.clear();
        self.sequence_index = -1;
    }

    pub fn sequence_index(&self) -> i32 {
        self.sequence_index
    }

    pub fn set_sequence_index(&mut self, sequence_index: i32) {
        self.sequence_index = sequence_index;
    }

    pub fn deform(&self) -> &[f32] {
        &self.deform
    }

    pub fn deform_mut(&mut self) -> &mut Vec<f32> {
        &mut self.deform
    }

    pub fn color(&self) -> [f32; 4] {
        self.color
    }

    pub fn set_color(&mut self, color: [f32; 4]) {
        self.color = color;
    }

    pub fn has_dark(&self) -> bool {
        self.has_dark
    }

    pub fn set_has_dark(&mut self, has_dark: bool) {
        self.has_dark = has_dark;
    }

    pub fn dark_color(&self) -> [f32; 3] {
        self.dark_color
    }

    pub fn set_dark_color(&mut self, dark_color: [f32; 3]) {
        self.dark_color = dark_color;
    }

    pub fn blend(&self) -> crate::BlendMode {
        self.blend
    }

    pub fn set_blend(&mut self, blend: crate::BlendMode) {
        self.blend = blend;
    }
}

#[derive(Clone, Debug)]
pub struct Slot {
    pub(super) data_index: usize,
    pub bone: usize,
    pub attachment: Option<String>,
    pub(crate) attachment_skin: Option<String>,
    pub(crate) attachment_state: i32,
    pub sequence_index: i32,
    pub deform: Vec<f32>,
    pub color: [f32; 4],
    pub has_dark: bool,
    pub dark_color: [f32; 3],
    pub blend: crate::BlendMode,
}

impl Slot {
    pub fn data_index(&self) -> usize {
        self.data_index
    }
}

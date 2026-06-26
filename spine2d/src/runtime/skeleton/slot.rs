use super::{Bone, Skeleton};

#[derive(Clone, Debug)]
pub(crate) struct SlotPose {
    pub(crate) attachment: Option<String>,
    pub(crate) attachment_skin: Option<String>,
    pub(crate) sequence_index: i32,
    pub(crate) deform: Vec<f32>,
    pub(crate) color: [f32; 4],
    pub(crate) has_dark: bool,
    pub(crate) dark_color: [f32; 3],
}

impl SlotPose {
    pub(crate) fn from_slot(slot: &Slot) -> Self {
        Self {
            attachment: slot.attachment.clone(),
            attachment_skin: slot.attachment_skin.clone(),
            sequence_index: slot.sequence_index,
            deform: slot.deform.clone(),
            color: slot.color,
            has_dark: slot.has_dark,
            dark_color: slot.dark_color,
        }
    }
}

pub(crate) enum SlotPoseRef<'a> {
    Pose(&'a Slot),
    Applied(&'a SlotPose),
}

impl SlotPoseRef<'_> {
    pub(crate) fn get_attachment_name(&self) -> Option<&str> {
        match self {
            Self::Pose(slot) => slot.attachment.as_deref(),
            Self::Applied(pose) => pose.attachment.as_deref(),
        }
    }

    pub(crate) fn get_attachment_skin(&self) -> Option<&str> {
        match self {
            Self::Pose(slot) => slot.attachment_skin.as_deref(),
            Self::Applied(pose) => pose.attachment_skin.as_deref(),
        }
    }

    pub(crate) fn get_sequence_index(&self) -> i32 {
        match self {
            Self::Pose(slot) => slot.sequence_index,
            Self::Applied(pose) => pose.sequence_index,
        }
    }

    pub(crate) fn get_color(&self) -> [f32; 4] {
        match self {
            Self::Pose(slot) => slot.color,
            Self::Applied(pose) => pose.color,
        }
    }

    pub(crate) fn has_dark_color(&self) -> bool {
        match self {
            Self::Pose(slot) => slot.has_dark,
            Self::Applied(pose) => pose.has_dark,
        }
    }

    pub(crate) fn get_dark_color(&self) -> [f32; 3] {
        match self {
            Self::Pose(slot) => slot.dark_color,
            Self::Applied(pose) => pose.dark_color,
        }
    }
}

pub(crate) enum SlotPoseMut<'a> {
    Pose(&'a mut Slot),
    Applied(&'a mut SlotPose),
}

impl SlotPoseMut<'_> {
    pub(crate) fn get_color(&self) -> [f32; 4] {
        match self {
            Self::Pose(slot) => slot.color,
            Self::Applied(pose) => pose.color,
        }
    }

    pub(crate) fn set_color(&mut self, color: [f32; 4]) {
        match self {
            Self::Pose(slot) => slot.color = color,
            Self::Applied(pose) => pose.color = color,
        }
    }

    pub(crate) fn color_mut(&mut self) -> &mut [f32; 4] {
        match self {
            Self::Pose(slot) => &mut slot.color,
            Self::Applied(pose) => &mut pose.color,
        }
    }

    pub(crate) fn set_has_dark_color(&mut self, has_dark: bool) {
        match self {
            Self::Pose(slot) => slot.has_dark = has_dark,
            Self::Applied(pose) => pose.has_dark = has_dark,
        }
    }

    pub(crate) fn get_dark_color(&self) -> [f32; 3] {
        match self {
            Self::Pose(slot) => slot.dark_color,
            Self::Applied(pose) => pose.dark_color,
        }
    }

    pub(crate) fn set_dark_color(&mut self, dark_color: [f32; 3]) {
        match self {
            Self::Pose(slot) => slot.dark_color = dark_color,
            Self::Applied(pose) => pose.dark_color = dark_color,
        }
    }

    pub(crate) fn set_sequence_index(&mut self, sequence_index: i32) {
        match self {
            Self::Pose(slot) => slot.sequence_index = sequence_index,
            Self::Applied(pose) => pose.sequence_index = sequence_index,
        }
    }

    pub(crate) fn get_deform(&self) -> &[f32] {
        match self {
            Self::Pose(slot) => slot.deform.as_slice(),
            Self::Applied(pose) => pose.deform.as_slice(),
        }
    }

    pub(crate) fn deform_mut(&mut self) -> &mut Vec<f32> {
        match self {
            Self::Pose(slot) => &mut slot.deform,
            Self::Applied(pose) => &mut pose.deform,
        }
    }

    pub(crate) fn set_attachment(
        &mut self,
        attachment: Option<String>,
        attachment_skin: Option<String>,
        clear_deform: bool,
    ) {
        match self {
            Self::Pose(slot) => {
                slot.attachment = attachment;
                slot.attachment_skin = attachment_skin;
                if clear_deform {
                    slot.deform.clear();
                }
                slot.sequence_index = -1;
            }
            Self::Applied(pose) => {
                pose.attachment = attachment;
                pose.attachment_skin = attachment_skin;
                if clear_deform {
                    pose.deform.clear();
                }
                pose.sequence_index = -1;
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Slot {
    pub(crate) data_index: usize,
    pub(crate) bone: usize,
    pub(crate) attachment: Option<String>,
    pub(crate) attachment_skin: Option<String>,
    pub(crate) attachment_state: i32,
    pub(crate) sequence_index: i32,
    pub(crate) deform: Vec<f32>,
    pub(crate) color: [f32; 4],
    pub(crate) has_dark: bool,
    pub(crate) dark_color: [f32; 3],
    pub(crate) applied_pose: SlotPose,
    pub(crate) pose_constrained: bool,
}

impl Slot {
    fn attachment_data<'a>(
        skeleton: &'a Skeleton,
        slot_index: usize,
        attachment_name: Option<&str>,
        attachment_skin: Option<&str>,
    ) -> Option<&'a crate::AttachmentData> {
        let key = attachment_name?;

        if let Some(source_skin) = attachment_skin
            && let Some(skin) = skeleton.data.find_skin(source_skin)
            && let Some(attachment) = skin.get_attachment(slot_index, key)
        {
            return Some(attachment);
        }

        skeleton.get_attachment(slot_index, key)
    }

    pub fn get_attachment_name(&self) -> Option<&str> {
        self.attachment.as_deref()
    }

    pub fn get_applied_attachment_name(&self) -> Option<&str> {
        if self.pose_constrained {
            self.applied_pose.attachment.as_deref()
        } else {
            self.attachment.as_deref()
        }
    }

    pub fn get_bone<'a>(&self, skeleton: &'a Skeleton) -> &'a Bone {
        &skeleton.bones[self.bone]
    }

    pub fn get_attachment<'a>(&self, skeleton: &'a Skeleton) -> Option<&'a crate::AttachmentData> {
        Self::attachment_data(
            skeleton,
            self.data_index,
            self.attachment.as_deref(),
            self.attachment_skin.as_deref(),
        )
    }

    pub fn get_applied_attachment<'a>(
        &self,
        skeleton: &'a Skeleton,
    ) -> Option<&'a crate::AttachmentData> {
        let pose = self.pose_for(true);
        Self::attachment_data(
            skeleton,
            self.data_index,
            pose.get_attachment_name(),
            pose.get_attachment_skin(),
        )
    }

    pub fn get_sequence_index(&self) -> i32 {
        self.sequence_index
    }

    pub fn get_applied_sequence_index(&self) -> i32 {
        self.pose_for(true).get_sequence_index()
    }

    pub fn set_sequence_index(&mut self, sequence_index: i32) {
        self.sequence_index = sequence_index;
    }

    pub fn get_deform(&self) -> &[f32] {
        &self.deform
    }

    pub fn get_applied_deform(&self) -> &[f32] {
        if self.pose_constrained {
            self.applied_pose.deform.as_slice()
        } else {
            self.deform.as_slice()
        }
    }

    pub fn deform_mut(&mut self) -> &mut Vec<f32> {
        &mut self.deform
    }

    pub fn get_color(&self) -> [f32; 4] {
        self.color
    }

    pub fn get_applied_color(&self) -> [f32; 4] {
        self.pose_for(true).get_color()
    }

    pub fn set_color(&mut self, color: [f32; 4]) {
        self.color = color;
    }

    pub fn has_dark_color(&self) -> bool {
        self.has_dark
    }

    pub fn applied_has_dark_color(&self) -> bool {
        self.pose_for(true).has_dark_color()
    }

    pub fn set_has_dark_color(&mut self, has_dark: bool) {
        self.has_dark = has_dark;
    }

    pub fn get_dark_color(&self) -> [f32; 3] {
        self.dark_color
    }

    pub fn get_applied_dark_color(&self) -> [f32; 3] {
        self.pose_for(true).get_dark_color()
    }

    pub fn set_dark_color(&mut self, dark_color: [f32; 3]) {
        self.dark_color = dark_color;
    }

    pub(crate) fn pose_for(&self, applied_pose: bool) -> SlotPoseRef<'_> {
        if applied_pose && self.pose_constrained {
            SlotPoseRef::Applied(&self.applied_pose)
        } else {
            SlotPoseRef::Pose(self)
        }
    }

    pub(crate) fn pose_mut_for(&mut self, applied_pose: bool) -> SlotPoseMut<'_> {
        if applied_pose && self.pose_constrained {
            SlotPoseMut::Applied(&mut self.applied_pose)
        } else {
            SlotPoseMut::Pose(self)
        }
    }

    pub(crate) fn set_pose_constrained(&mut self, constrained: bool) {
        self.pose_constrained = constrained;
        if constrained {
            self.reset_constrained_pose();
        }
    }

    pub(crate) fn reset_constrained_pose(&mut self) {
        if self.pose_constrained {
            self.applied_pose = SlotPose::from_slot(self);
        }
    }
}

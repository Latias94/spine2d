use bevy::prelude::*;
use spine2d::{Skeleton, AnimationState, DrawList, Atlas, SkeletonData};
use std::sync::Arc;
use crate::SpineHandle;

pub struct SpineInstance {
    pub skeleton: Skeleton,
    pub animation_state: AnimationState,
    pub draw_list: DrawList,
    pub atlas: Atlas,
    pub skeleton_data: Arc<SkeletonData>,   // changed from SkeletonData to Arc
}

impl SpineInstance {
    pub fn new(
        skeleton: Skeleton,
        animation_state: AnimationState,
        atlas: Atlas,
        skeleton_data: Arc<SkeletonData>,   // accept Arc
    ) -> Self {
        Self {
            skeleton,
            animation_state,
            draw_list: DrawList::default(),
            atlas,
            skeleton_data,
        }
    }
}

pub struct SpineWorld {
    instances: Vec<Option<SpineInstance>>,
    free_ids: Vec<u32>,
}

impl Default for SpineWorld {
    fn default() -> Self {
        Self::new()
    }
}

impl SpineWorld {
    pub fn new() -> Self {
        Self {
            instances: Vec::new(),
            free_ids: Vec::new(),
        }
    }

    pub fn insert(&mut self, instance: SpineInstance) -> SpineHandle {
        if let Some(id) = self.free_ids.pop() {
            self.instances[id as usize] = Some(instance);
            SpineHandle(id)
        } else {
            let id = self.instances.len() as u32;
            self.instances.push(Some(instance));
            SpineHandle(id)
        }
    }

    pub fn get(&self, handle: SpineHandle) -> &SpineInstance {
        self.instances[handle.0 as usize].as_ref().unwrap()
    }

    pub fn get_mut(&mut self, handle: SpineHandle) -> &mut SpineInstance {
        self.instances[handle.0 as usize].as_mut().unwrap()
    }

    pub fn remove(&mut self, handle: SpineHandle) {
        self.instances[handle.0 as usize] = None;
        self.free_ids.push(handle.0);
    }
}
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use bevy::prelude::*;
use spine2d::{
    AnimationState, AnimationStateEvent, AnimationStateListener, Atlas, DrawList, Skeleton,
    TrackEntryHandle,
};

use crate::{SpineAnimationEventKind, SpineSkeletonControl, components::SpineInstanceId};

pub(crate) struct SpineInstance {
    pub skeleton: Skeleton,
    pub animation_state: AnimationState,
    pub draw_list: DrawList,
    pub atlas: Atlas,
    pub atlas_directory: String,
    pub animation_name: Option<String>,
    pub loop_animation: bool,
    pub time_scale: f32,
    pub skin_name: Option<String>,
    pub flip_y: bool,
    pub skeleton_control: SpineSkeletonControl,
    pending_events: SpineEventBuffer,
}

pub(crate) struct SpineInstanceParts {
    pub skeleton: Skeleton,
    pub animation_state: AnimationState,
    pub atlas: Atlas,
    pub atlas_directory: String,
    pub animation_name: Option<String>,
    pub loop_animation: bool,
    pub time_scale: f32,
    pub skin_name: Option<String>,
    pub flip_y: bool,
    pub skeleton_control: SpineSkeletonControl,
}

impl SpineInstance {
    pub fn new(parts: SpineInstanceParts) -> Self {
        Self {
            skeleton: parts.skeleton,
            animation_state: parts.animation_state,
            draw_list: DrawList::default(),
            atlas: parts.atlas,
            atlas_directory: parts.atlas_directory,
            animation_name: parts.animation_name,
            loop_animation: parts.loop_animation,
            time_scale: parts.time_scale,
            skin_name: parts.skin_name,
            flip_y: parts.flip_y,
            skeleton_control: parts.skeleton_control,
            pending_events: SpineEventBuffer::default(),
        }
    }

    pub fn attach_event_listener(&mut self) {
        self.animation_state
            .set_listener(SpineEventListener::new(self.pending_events.clone()));
    }

    pub fn drain_events(&mut self) -> Vec<PendingSpineAnimationEvent> {
        self.pending_events.drain()
    }
}

#[derive(Clone, Default)]
struct SpineEventBuffer(Rc<RefCell<Vec<PendingSpineAnimationEvent>>>);

impl SpineEventBuffer {
    fn push(&self, event: PendingSpineAnimationEvent) {
        self.0.borrow_mut().push(event);
    }

    fn drain(&self) -> Vec<PendingSpineAnimationEvent> {
        self.0.borrow_mut().drain(..).collect()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PendingSpineAnimationEvent {
    pub track_index: usize,
    pub animation_name: String,
    pub track_time: f32,
    pub kind: SpineAnimationEventKind,
}

struct SpineEventListener {
    buffer: SpineEventBuffer,
}

impl SpineEventListener {
    fn new(buffer: SpineEventBuffer) -> Self {
        Self { buffer }
    }
}

impl AnimationStateListener for SpineEventListener {
    fn on_event(
        &mut self,
        state: &mut AnimationState,
        entry: TrackEntryHandle,
        event: &AnimationStateEvent,
    ) {
        let Some(entry) = entry.entry(state) else {
            return;
        };
        let track_index = entry.track_index();
        let animation_name = entry.animation().name.clone();
        let track_time = entry.track_time();

        self.buffer.push(PendingSpineAnimationEvent {
            track_index,
            animation_name,
            track_time,
            kind: match event {
                AnimationStateEvent::Start => SpineAnimationEventKind::Start,
                AnimationStateEvent::Interrupt => SpineAnimationEventKind::Interrupt,
                AnimationStateEvent::End => SpineAnimationEventKind::End,
                AnimationStateEvent::Dispose => SpineAnimationEventKind::Dispose,
                AnimationStateEvent::Complete => SpineAnimationEventKind::Complete,
                AnimationStateEvent::Event(event) => SpineAnimationEventKind::Event(event.clone()),
            },
        });
    }
}

#[derive(Default)]
struct SpineWorldSlot {
    generation: u32,
    instance: Option<SpineInstance>,
    owner: Option<Entity>,
}

pub(crate) struct SpineWorld {
    slots: Vec<SpineWorldSlot>,
    free: Vec<u32>,
    by_owner: HashMap<Entity, SpineInstanceId>,
}

impl Default for SpineWorld {
    fn default() -> Self {
        Self::new()
    }
}

impl SpineWorld {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            free: Vec::new(),
            by_owner: HashMap::new(),
        }
    }

    pub fn insert(&mut self, owner: Entity, instance: SpineInstance) -> SpineInstanceId {
        self.remove_by_owner(owner);

        let index = self.free.pop().unwrap_or_else(|| {
            let index = self.slots.len() as u32;
            self.slots.push(SpineWorldSlot::default());
            index
        });
        let slot = &mut self.slots[index as usize];
        slot.generation = slot.generation.wrapping_add(1).max(1);
        slot.instance = Some(instance);
        slot.owner = Some(owner);

        let id = SpineInstanceId {
            index,
            generation: slot.generation,
        };
        self.by_owner.insert(owner, id);
        id
    }

    pub fn get(&self, id: SpineInstanceId) -> Option<&SpineInstance> {
        let slot = self.slots.get(id.index as usize)?;
        (slot.generation == id.generation)
            .then_some(slot.instance.as_ref())
            .flatten()
    }

    pub fn get_mut(&mut self, id: SpineInstanceId) -> Option<&mut SpineInstance> {
        let slot = self.slots.get_mut(id.index as usize)?;
        (slot.generation == id.generation)
            .then_some(slot.instance.as_mut())
            .flatten()
    }

    pub fn remove_by_owner(&mut self, owner: Entity) -> Option<SpineInstance> {
        let id = self.by_owner.remove(&owner)?;
        let slot = self.slots.get_mut(id.index as usize)?;
        if slot.generation != id.generation {
            return None;
        }
        slot.owner = None;
        let instance = slot.instance.take()?;
        self.free.push(id.index);
        Some(instance)
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.by_owner.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spine2d::{AnimationStateData, SkeletonData};

    fn demo_instance() -> SpineInstance {
        let data = SkeletonData::from_json_str(include_str!("../../spine2d-web/assets/demo.json"))
            .expect("parse demo skeleton");
        let atlas = Atlas::parse(include_str!("../../spine2d-web/assets/demo.atlas"))
            .expect("parse demo atlas");

        SpineInstance::new(SpineInstanceParts {
            skeleton: Skeleton::new(data.clone()),
            animation_state: AnimationState::new(AnimationStateData::new(data)),
            atlas,
            atlas_directory: String::new(),
            animation_name: Some("spin".to_owned()),
            loop_animation: true,
            time_scale: 1.0,
            skin_name: None,
            flip_y: false,
            skeleton_control: SpineSkeletonControl::default(),
        })
    }

    #[test]
    fn replacing_owner_invalidates_previous_instance_key() {
        let mut ecs_world = World::new();
        let owner = ecs_world.spawn_empty().id();
        let mut spine_world = SpineWorld::new();

        let first = spine_world.insert(owner, demo_instance());
        let second = spine_world.insert(owner, demo_instance());

        assert_eq!(spine_world.len(), 1);
        assert!(spine_world.get(first).is_none());
        assert!(spine_world.get(second).is_some());
    }

    #[test]
    fn removing_owner_reuses_slot_with_new_generation() {
        let mut ecs_world = World::new();
        let owner = ecs_world.spawn_empty().id();
        let mut spine_world = SpineWorld::new();

        let first = spine_world.insert(owner, demo_instance());
        assert!(spine_world.remove_by_owner(owner).is_some());
        assert_eq!(spine_world.len(), 0);
        assert!(spine_world.get(first).is_none());

        let second = spine_world.insert(owner, demo_instance());
        assert_eq!(first.index, second.index);
        assert_ne!(first.generation, second.generation);
        assert!(spine_world.get(second).is_some());
    }
}

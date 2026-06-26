use crate::Skeleton;
use crate::runtime::{
    AnimationState, AnimationStateData, AnimationStateEvent, AnimationStateListener, TrackEntry,
    TrackEntryHandle, TrackEntryListener,
};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

const TEST_JSON: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "events": { "event": {} },
  "animations": {
    "events0": {
      "events": [
        { "name": "event", "string": "0" },
        { "time": 0.4667, "name": "event", "string": "14" },
        { "time": 1.0, "name": "event", "string": "30" }
      ]
    },
    "events1": {
      "events": [
        { "name": "event", "string": "0" },
        { "time": 0.4667, "name": "event", "string": "14" },
        { "time": 1.0, "name": "event", "string": "30" }
      ]
    },
    "events2": {
      "events": [
        { "name": "event", "string": "0" },
        { "time": 0.4667, "name": "event", "string": "14" },
        { "time": 1.0, "name": "event", "string": "30" }
      ]
    }
  }
}
"#;

#[derive(Clone, Debug, PartialEq)]
struct ResultRow {
    animation_name: String,
    name: String,
    track_time: f32,
    total_time: f32,
}

#[derive(Clone)]
struct Recording {
    time: Rc<Cell<f32>>,
    enabled: Rc<Cell<bool>>,
    rows: Rc<RefCell<Vec<ResultRow>>>,
}

struct RecordingListener {
    recording: Recording,
}

fn round_tracks(state: &mut AnimationState) {
    fn round_decimals(value: f32, decimals: u32) -> f32 {
        let factor = 10_f32.powi(decimals as i32);
        (value * factor).round() / factor
    }

    let current_ids = state.get_tracks().into_iter().flatten().collect::<Vec<_>>();
    for current in current_ids {
        let track_time = current
            .entry(state)
            .map(|entry| entry.track_time())
            .unwrap();
        let delay = current.entry(state).map(|entry| entry.delay()).unwrap();
        current.set_track_time(state, round_decimals(track_time, 6));
        current.set_delay(state, round_decimals(delay, 3));
        let mut from = current.mixing_from(state);
        while let Some(id) = from {
            let track_time = id.entry(state).map(|entry| entry.track_time()).unwrap();
            from = id.mixing_from(state);
            id.set_track_time(state, round_decimals(track_time, 6));
        }
    }
}

impl AnimationStateListener for RecordingListener {
    fn on_event(
        &mut self,
        state: &mut AnimationState,
        entry: TrackEntryHandle,
        event: &AnimationStateEvent,
    ) {
        if !self.recording.enabled.get() {
            return;
        }
        let name = match event {
            AnimationStateEvent::Start => "start".to_string(),
            AnimationStateEvent::Interrupt => "interrupt".to_string(),
            AnimationStateEvent::End => "end".to_string(),
            AnimationStateEvent::Dispose => "dispose".to_string(),
            AnimationStateEvent::Complete => "complete".to_string(),
            AnimationStateEvent::Event(ev) => format!("event {}", ev.string),
        };

        let Some(entry) = entry.entry(state) else {
            return;
        };
        let animation_name = entry.animation().name.clone();
        let track_time = entry.track_time();

        self.recording.rows.borrow_mut().push(ResultRow {
            animation_name,
            name,
            track_time: round3(track_time),
            total_time: round3(self.recording.time.get()),
        });
    }
}

fn round3(value: f32) -> f32 {
    (value * 1000.0).round() / 1000.0
}

fn setup() -> (AnimationState, Skeleton, Recording) {
    let data = crate::SkeletonData::from_json_str(TEST_JSON).unwrap();
    let state_data = AnimationStateData::new(data.clone());
    let state = AnimationState::new(state_data);
    let skeleton = Skeleton::new(data);
    let recording = Recording {
        time: Rc::new(Cell::new(0.0)),
        enabled: Rc::new(Cell::new(true)),
        rows: Rc::new(RefCell::new(Vec::new())),
    };
    (state, skeleton, recording)
}

fn with_track_entry<R>(
    state: &AnimationState,
    track_index: usize,
    f: impl FnOnce(&TrackEntry) -> R,
) -> Option<R> {
    state.get_current(track_index)?.entry(state).map(f)
}

fn with_queued_track_entry<R>(
    state: &AnimationState,
    track_index: usize,
    queue_index: usize,
    f: impl FnOnce(&TrackEntry) -> R,
) -> Option<R> {
    let mut handle = state.get_current(track_index)?.next(state)?;
    for _ in 0..queue_index {
        handle = handle.next(state)?;
    }
    handle.entry(state).map(f)
}

#[test]
fn animation_state_data_default_mix_is_directly_stored_and_used_as_fallback() {
    let data = crate::SkeletonData::from_json_str(TEST_JSON).unwrap();
    let from = data.find_animation("events0").unwrap();
    let to = data.find_animation("events1").unwrap();
    let mut state_data = AnimationStateData::new(data.clone());

    assert_eq!(state_data.default_mix(), 0.0);
    assert_eq!(state_data.get_mix_animation(from, to), 0.0);

    state_data.set_default_mix(0.25);

    assert_eq!(state_data.default_mix(), 0.25);
    assert_eq!(state_data.get_mix_animation(from, to), 0.25);
    state_data.set_default_mix(-0.1);
    assert_eq!(state_data.default_mix(), -0.1);
    state_data.set_default_mix(f32::NAN);
    assert!(state_data.default_mix().is_nan());
    state_data.set_default_mix(f32::INFINITY);
    assert!(state_data.default_mix().is_infinite());
}

#[test]
fn animation_state_data_named_mix_overrides_default_mix() {
    let data = crate::SkeletonData::from_json_str(TEST_JSON).unwrap();
    let from = data.find_animation("events0").unwrap();
    let to = data.find_animation("events1").unwrap();
    let mut state_data = AnimationStateData::new(data.clone());

    state_data.set_default_mix(0.25);
    state_data.set_mix("events0", "events1", 0.5);

    assert_eq!(state_data.get_mix_animation(from, to), 0.5);
    assert_eq!(state_data.get_mix_animation(to, from), 0.25);

    state_data.set_mix("events0", "events1", 0.75);

    assert_eq!(state_data.get_mix_animation(from, to), 0.75);
}

#[test]
fn animation_state_data_animation_mix_accessors_match_name_mix_storage() {
    let data = crate::SkeletonData::from_json_str(TEST_JSON).unwrap();
    let from = data.find_animation("events0").unwrap();
    let to = data.find_animation("events1").unwrap();
    let mut state_data = AnimationStateData::new(data.clone());

    state_data.set_default_mix(0.25);
    assert_eq!(state_data.get_mix_animation(from, to), 0.25);

    state_data.set_mix_animation(from, to, 0.5);

    assert_eq!(state_data.get_mix_animation(from, to), 0.5);
}

#[test]
fn animation_state_data_clear_resets_default_and_animation_mixes() {
    let data = crate::SkeletonData::from_json_str(TEST_JSON).unwrap();
    let from = data.find_animation("events0").unwrap();
    let to = data.find_animation("events1").unwrap();
    let mut state_data = AnimationStateData::new(data.clone());

    state_data.set_default_mix(0.25);
    state_data.set_mix("events0", "events1", 0.5);

    state_data.clear();

    assert_eq!(state_data.default_mix(), 0.0);
    assert_eq!(state_data.get_mix_animation(from, to), 0.0);
}

#[test]
fn animation_state_data_accessor_matches_mutable_data() {
    let (mut state, _skeleton, _recording) = setup();

    assert_eq!(state.get_data().default_mix(), 0.0);
    state.get_data_mut().set_default_mix(0.3);

    assert_eq!(state.get_data().default_mix(), 0.3);
}

#[test]
fn animation_state_data_skeleton_data_accessor_exposes_bound_skeleton() {
    let data = crate::SkeletonData::from_json_str(TEST_JSON).unwrap();
    let data_ptr = Arc::as_ptr(&data);
    let state_data = AnimationStateData::new(data);

    assert_eq!(
        state_data.skeleton_data() as *const crate::SkeletonData,
        data_ptr
    );
    assert!(
        state_data
            .skeleton_data()
            .find_animation("events0")
            .is_some()
    );
}

#[test]
fn track_entry_handles_are_bound_to_their_animation_state() {
    let data = crate::SkeletonData::from_json_str(TEST_JSON).unwrap();
    let mut first_state = AnimationState::new(AnimationStateData::new(data.clone()));
    let mut second_state = AnimationState::new(AnimationStateData::new(data));

    let first_entry = first_state.set_animation(0, "events0", false);
    let second_entry = second_state.set_animation(0, "events1", false);

    assert!(first_entry.animation_state(&first_state).is_some());
    assert!(first_entry.animation_state(&second_state).is_none());
    assert!(first_entry.entry(&first_state).is_some());
    assert!(first_entry.entry(&second_state).is_none());

    first_entry.set_alpha(&mut second_state, 0.25);
    assert_eq!(
        with_track_entry(&second_state, 0, |entry| entry.alpha()).unwrap(),
        1.0
    );

    second_entry.set_alpha(&mut second_state, 0.75);
    assert_eq!(
        with_track_entry(&second_state, 0, |entry| entry.alpha()).unwrap(),
        0.75
    );
}

#[test]
fn track_entry_handle_reads_current_and_queued_entries() {
    let data = crate::SkeletonData::from_json_str(TEST_JSON).unwrap();
    let mut state = AnimationState::new(AnimationStateData::new(data.clone()));
    let other_state = AnimationState::new(AnimationStateData::new(data));

    let current = state.set_animation(0, "events0", false);
    let queued = state.add_animation(0, "events1", true, 0.4);

    assert_eq!(
        current
            .entry(&state)
            .map(|entry| entry.animation().name.clone()),
        Some("events0".to_string())
    );
    assert_eq!(
        queued.entry(&state).map(|entry| {
            (
                entry.animation().name.clone(),
                entry.looped(),
                entry.delay(),
            )
        }),
        Some(("events1".to_string(), true, 0.4))
    );
    assert_eq!(queued.entry(&other_state).map(|_| ()), None);
}

#[test]
fn animation_state_data_name_mix_panics_on_unknown_animations() {
    let data = crate::SkeletonData::from_json_str(TEST_JSON).unwrap();
    let from = data.find_animation("events0").unwrap();
    let to = data.find_animation("events1").unwrap();
    let mut state_data = AnimationStateData::new(data.clone());

    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            state_data.set_mix("missing", "events1", 0.5);
        }))
        .is_err()
    );
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            state_data.set_mix("events0", "missing", 0.5);
        }))
        .is_err()
    );
    state_data.set_mix("events0", "events1", -0.1);
    state_data.set_mix("events0", "events1", f32::NAN);
    assert!(state_data.get_mix_animation(from, to).is_nan());
    state_data.set_mix("events0", "events1", f32::INFINITY);
    assert!(state_data.get_mix_animation(from, to).is_infinite());
}

#[test]
fn animation_state_data_animation_mix_accepts_animation_references() {
    let data = crate::SkeletonData::from_json_str(TEST_JSON).unwrap();
    let mut from = data.find_animation("events0").unwrap().clone();
    let mut to = data.find_animation("events1").unwrap().clone();
    let mut state_data = AnimationStateData::new(data.clone());

    from.name = "from-reference".to_string();
    to.name = "to-reference".to_string();
    state_data.set_default_mix(0.25);

    assert_eq!(state_data.get_mix_animation(&from, &to), 0.25);
    state_data.set_mix_animation(&from, &to, 0.5);
    assert_eq!(state_data.get_mix_animation(&from, &to), 0.5);
}

const EMPTY_DELAY_JSON: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "animations": {
    "a": {
      "bones": {
        "root": {
          "rotate": [
            { "time": 0.0, "value": 0.0 },
            { "time": 1.0, "value": 0.0 }
          ]
        }
      }
    }
  }
}
"#;

const NEGATIVE_ALPHA_JSON: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "animations": {
    "turn": {
      "bones": {
        "root": {
          "rotate": [
            { "time": 0.0, "value": 10.0 },
            { "time": 1.0, "value": 10.0 }
          ]
        }
      }
    }
  }
}
"#;

const PHYSICS_RESET_JSON: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [
    { "name": "root" },
    { "name": "bone", "parent": "root" }
  ],
  "physics": [
    { "name": "wind", "bone": "bone" }
  ],
  "animations": {
    "run": {
      "physics": {
        "wind": {
          "reset": [
            { "time": 0.5 }
          ]
        }
      }
    }
  }
}
"#;

#[test]
fn animation_state_applies_negative_track_alpha_like_cpp() {
    let data = crate::SkeletonData::from_json_str(NEGATIVE_ALPHA_JSON).unwrap();
    let state_data = AnimationStateData::new(data.clone());
    let mut state = AnimationState::new(state_data);
    let mut skeleton = Skeleton::new(data);

    let entry = state.set_animation(0, "turn", false);
    entry.set_alpha(&mut state, -0.5);

    skeleton.setup_pose();
    state.apply(&mut skeleton);

    assert_eq!(skeleton.bones[0].rotation, -5.0);
}

fn run(
    state: &mut AnimationState,
    skeleton: &mut Skeleton,
    recording: &Recording,
    step: f32,
    end_time: f32,
) {
    run_with_frame(state, skeleton, recording, step, end_time, |_, _| {});
}

fn run_with_frame<F: FnMut(f32, &mut AnimationState)>(
    state: &mut AnimationState,
    skeleton: &mut Skeleton,
    recording: &Recording,
    step: f32,
    end_time: f32,
    mut on_frame: F,
) {
    recording.time.set(0.0);
    recording.enabled.set(true);
    state.apply(skeleton);

    let mut time = 0.0;
    while time < end_time {
        time += step;
        recording.time.set(time);
        state.update(step);
        round_tracks(state);
        // Match the upstream C# tests: apply multiple times per frame to ensure the state doesn't depend on apply side effects.
        recording.enabled.set(true);
        state.apply(skeleton);
        recording.enabled.set(false);
        state.apply(skeleton);
        state.apply(skeleton);
        recording.enabled.set(true);
        on_frame(round3(time), state);
    }
}

#[test]
fn add_empty_animation_delay_is_adjusted_to_end_with_previous_entry() {
    let data = crate::SkeletonData::from_json_str(EMPTY_DELAY_JSON).unwrap();
    let state_data = AnimationStateData::new(data.clone());
    let mut state = AnimationState::new(state_data);
    let mut skeleton = Skeleton::new(data);

    let recording = Recording {
        time: Rc::new(Cell::new(0.0)),
        enabled: Rc::new(Cell::new(true)),
        rows: Rc::new(RefCell::new(Vec::new())),
    };
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    state.set_animation(0, "a", false);
    state.add_empty_animation(0, 0.5, 0.0);
    let delay = state
        .get_current(0)
        .and_then(|entry| entry.next(&state))
        .and_then(|entry| entry.entry(&state).map(|entry| entry.delay()))
        .expect("queued empty animation");
    assert_eq!(round3(delay), 0.5);

    // Smoke-run the state to ensure the queue can actually be consumed without panics.
    run(&mut state, &mut skeleton, &recording, 0.1, 1.2);
}

#[test]
fn add_empty_animation_stores_non_finite_delay_directly() {
    let data = crate::SkeletonData::from_json_str(EMPTY_DELAY_JSON).unwrap();
    let state_data = AnimationStateData::new(data);
    let mut state = AnimationState::new(state_data);

    state.add_empty_animation(0, 0.5, f32::INFINITY);

    let delay = with_track_entry(&state, 0, |entry| entry.delay()).expect("current empty entry");
    assert!(delay.is_infinite());
    assert_eq!(
        with_track_entry(&state, 0, |entry| entry.mix_duration()).expect("current empty entry"),
        0.5
    );
}

#[test]
fn add_empty_animation_on_empty_track_sets_entry_fields_after_start_event_like_cpp() {
    #[derive(Default)]
    struct EntryStateOnStart {
        rows: Rc<RefCell<Vec<(f32, f32, f32)>>>,
    }

    impl AnimationStateListener for EntryStateOnStart {
        fn on_event(
            &mut self,
            state: &mut AnimationState,
            entry: TrackEntryHandle,
            event: &AnimationStateEvent,
        ) {
            if matches!(event, AnimationStateEvent::Start)
                && let Some(entry) = entry.entry(state)
            {
                self.rows.borrow_mut().push((
                    entry.delay(),
                    entry.mix_duration(),
                    entry.track_end(),
                ));
            }
        }
    }

    let data = crate::SkeletonData::from_json_str(TEST_JSON).unwrap();
    let mut state = AnimationState::new(AnimationStateData::new(data));
    let rows = Rc::new(RefCell::new(Vec::new()));
    state.set_listener(EntryStateOnStart { rows: rows.clone() });

    let entry = state.add_empty_animation(0, 0.5, 0.0);

    assert_eq!(*rows.borrow(), vec![(0.0, 0.0, f32::MAX)]);
    assert_eq!(
        entry
            .entry(&state)
            .map(|entry| { (entry.delay(), entry.mix_duration(), entry.track_end(),) }),
        Some((0.0, 0.5, 0.5))
    );
}

#[test]
fn track_entry_set_delay_ignores_negative_values() {
    let (mut state, _skeleton, _recording) = setup();

    let entry = state.set_animation(0, "events0", false);
    entry.set_delay(&mut state, 0.25);
    entry.set_delay(&mut state, -0.5);

    let delay = with_track_entry(&state, 0, |entry| entry.delay()).expect("current entry");
    assert_eq!(round3(delay), 0.25);
}

#[test]
fn track_entry_set_animation_swaps_animation_without_rebuilding_timing() {
    let (mut state, _skeleton, _recording) = setup();

    let entry = state.set_animation(0, "events0", false);
    entry.set_animation_start(&mut state, 0.125);
    entry.set_animation_end(&mut state, 0.5);
    entry.set_track_time(&mut state, 0.25);
    entry.set_delay(&mut state, 0.375);
    let replacement = state
        .get_data()
        .skeleton_data()
        .find_animation("events1")
        .unwrap()
        .clone();
    let mut replacement = replacement;
    replacement.duration += 10.0;

    entry.set_animation(&mut state, &replacement);

    with_track_entry(&state, 0, |entry| {
        assert_eq!(entry.animation().name, "events1");
        assert_eq!(entry.animation().duration, replacement.duration);
        assert_eq!(round3(entry.animation_start()), 0.125);
        assert_eq!(round3(entry.animation_end()), 0.5);
        assert_eq!(round3(entry.track_time()), 0.25);
        assert_eq!(round3(entry.delay()), 0.375);
    });
}

#[test]
fn track_entry_set_animation_preserves_animation_references() {
    let (mut state, _skeleton, _recording) = setup();

    let entry = state.set_animation(0, "events0", false);
    let mut unknown = state
        .get_data()
        .skeleton_data()
        .find_animation("events1")
        .unwrap()
        .clone();
    unknown.name = "missing".into();
    unknown.duration += 10.0;

    entry.set_animation(&mut state, &unknown);
    let animation_name =
        with_track_entry(&state, 0, |entry| entry.animation().name.clone()).expect("current entry");
    assert_eq!(animation_name, "missing");
    let animation_duration =
        with_track_entry(&state, 0, |entry| entry.animation().duration).expect("current entry");
    assert_eq!(animation_duration, unknown.duration);
}

#[test]
fn animation_state_set_and_add_preserve_animation_references() {
    let data = crate::SkeletonData::from_json_str(TEST_JSON).unwrap();
    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_default_mix(0.2);
    let mut state = AnimationState::new(state_data);

    let mut alpha = data.find_animation("events0").unwrap().clone();
    alpha.duration += 5.0;
    let mut beta = data.find_animation("events1").unwrap().clone();
    beta.duration += 7.0;

    let current = state.set_animation_ref(0, &alpha, false);
    assert_eq!(
        current
            .entry(&state)
            .map(|entry| entry.animation().name.clone())
            .unwrap(),
        "events0"
    );
    assert_eq!(
        current
            .entry(&state)
            .map(|entry| entry.animation().duration)
            .unwrap(),
        alpha.duration
    );

    let queued = state.add_animation_ref(0, &beta, true, 0.0);
    assert_eq!(
        queued
            .entry(&state)
            .map(|entry| entry.animation().name.clone())
            .unwrap(),
        "events1"
    );
    assert_eq!(
        queued
            .entry(&state)
            .map(|entry| entry.animation().duration)
            .unwrap(),
        beta.duration
    );
}

#[test]
fn animation_state_apply_returns_whether_any_track_was_applied() {
    let (mut state, mut skeleton, _recording) = setup();

    assert!(!state.apply(&mut skeleton));

    let entry = state.set_animation(0, "events0", false);
    entry.set_delay(&mut state, 0.25);
    assert!(!state.apply(&mut skeleton));

    entry.set_delay(&mut state, 0.0);
    assert!(state.apply(&mut skeleton));
}

#[test]
fn animation_state_current_returns_current_track_handle() {
    let (mut state, _skeleton, _recording) = setup();

    assert_eq!(state.get_current(0), None);
    assert!(state.get_tracks().is_empty());

    let first = state.set_animation(0, "events0", false);
    state.add_animation(0, "events1", false, 0.0);
    let third = state.set_animation(2, "events2", false);

    assert_eq!(state.get_current(0), Some(first));
    assert_eq!(state.get_current(1), None);
    assert_eq!(state.get_current(2), Some(third));
    assert_eq!(state.get_tracks(), vec![Some(first), None, Some(third)]);

    state.clear_track(0);
    assert_eq!(state.get_current(0), None);
    assert_eq!(state.get_tracks(), vec![None, None, Some(third)]);
}

#[test]
fn animation_state_queue_can_be_disabled_until_next_drain_point() {
    let (mut state, _skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    state.disable_queue();
    state.set_animation(0, "events0", false);
    assert!(recording.rows.borrow().is_empty());

    state.enable_queue();
    assert!(recording.rows.borrow().is_empty());

    state.update(0.0);

    assert_eq!(
        *recording.rows.borrow(),
        vec![ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        }]
    );
}

#[test]
fn disable_queue_inside_listener_only_suppresses_reentrant_drain_like_cpp() {
    #[derive(Default)]
    struct DisableOnStart {
        starts: Rc<Cell<usize>>,
    }

    impl AnimationStateListener for DisableOnStart {
        fn on_event(
            &mut self,
            state: &mut AnimationState,
            _entry: TrackEntryHandle,
            event: &AnimationStateEvent,
        ) {
            if matches!(event, AnimationStateEvent::Start) {
                self.starts.set(self.starts.get() + 1);
                state.disable_queue();
            }
        }
    }

    let data = crate::SkeletonData::from_json_str(TEST_JSON).unwrap();
    let mut state = AnimationState::new(AnimationStateData::new(data));
    let starts = Rc::new(Cell::new(0));
    state.set_listener(DisableOnStart {
        starts: starts.clone(),
    });

    state.set_animation(0, "events0", false);
    assert_eq!(starts.get(), 1);

    state.set_animation(1, "events1", false);
    assert_eq!(starts.get(), 2);
}

#[test]
fn animation_state_manual_track_entry_disposal_matches_cpp_lifetime_control() {
    let (mut auto_state, _skeleton, _recording) = setup();
    let auto_entry = auto_state.set_animation(0, "events0", false);
    assert!(auto_entry.entry(&auto_state).is_some());

    auto_state.clear_track(0);
    assert!(auto_entry.entry(&auto_state).is_none());

    let (mut manual_state, _skeleton, _recording) = setup();
    assert!(!manual_state.get_manual_track_entry_disposal());
    manual_state.set_manual_track_entry_disposal(true);
    assert!(manual_state.get_manual_track_entry_disposal());

    let manual_entry = manual_state.set_animation(0, "events0", false);
    manual_state.clear_track(0);
    assert!(manual_entry.entry(&manual_state).is_some());

    manual_state.dispose_track_entry(manual_entry);
    assert!(manual_entry.entry(&manual_state).is_none());
}

#[test]
fn manual_disposal_disposes_a_single_entry_without_chain_cleanup() {
    let (mut state, _skeleton, _recording) = setup();

    state.set_manual_track_entry_disposal(true);
    let first = state.set_animation(0, "events0", false);
    let second = state.add_animation(0, "events1", false, 0.5);
    let third = state.add_animation(0, "events2", false, 0.0);

    state.clear_track(0);

    assert!(second.entry(&state).is_some());
    assert!(third.entry(&state).is_some());

    state.dispose_track_entry(second);
    assert!(second.entry(&state).is_none());
    assert_eq!(second.previous(&state), None);
    assert_eq!(second.next(&state), None);
    assert_eq!(third.previous(&state), None);

    state.dispose_track_entry(third);
    assert!(third.entry(&state).is_none());
    assert_eq!(third.previous(&state), None);
    assert_eq!(third.next(&state), None);

    assert!(first.entry(&state).is_some());
}

#[test]
fn track_entry_set_mix_duration_with_delay_adjusts_queued_delay() {
    let data = crate::SkeletonData::from_json_str(EMPTY_DELAY_JSON).unwrap();
    let state_data = AnimationStateData::new(data);
    let mut state = AnimationState::new(state_data);

    state.set_animation(0, "a", false);
    let queued = state.add_animation(0, "a", false, 0.0);
    queued.set_mix_duration_with_delay(&mut state, 0.4, 0.0);

    let delay = state
        .get_current(0)
        .and_then(|entry| entry.next(&state))
        .and_then(|entry| entry.entry(&state).map(|entry| entry.delay()))
        .expect("queued animation");
    assert_eq!(round3(delay), 0.6);

    let mix_duration =
        with_queued_track_entry(&state, 0, 0, |entry| entry.mix_duration()).expect("queued entry");
    assert_eq!(round3(mix_duration), 0.4);
}

#[test]
fn queued_negative_mix_duration_is_preserved_on_activation() {
    let (mut state, _skeleton, _recording) = setup();
    let mut skeleton = _skeleton;

    state.set_animation(0, "events0", false);
    let queued = state.add_animation(0, "events1", false, 0.0);
    queued.set_mix_duration_with_delay(&mut state, -0.25, 0.0);

    state.apply(&mut skeleton);
    state.update(1.3);
    state.apply(&mut skeleton);
    state.update(0.0);

    assert_eq!(
        with_track_entry(&state, 0, |entry| entry.mix_duration()).unwrap(),
        -0.25
    );
}

#[test]
fn events_0p1_time_step() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    let entry = state.set_animation(0, "events0", false);
    entry.set_track_end(&mut state, 1.0);

    run(&mut state, &mut skeleton, &recording, 0.1, 2.0);

    let expected = vec![
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 0.5,
            total_time: 0.5,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 30".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 1.0,
            total_time: 1.1,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 1.0,
            total_time: 1.1,
        },
    ];

    assert_eq!(*recording.rows.borrow(), expected);
}

#[test]
fn queue_events_uses_cpp_signed_track_last_remainder() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    let entry = state.set_animation(0, "events0", true);
    entry.set_animation_start(&mut state, 0.2);
    entry.set_animation_end(&mut state, 0.8);
    entry.set_track_time(&mut state, 1.0);

    state.apply(&mut skeleton);

    let expected = vec![
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 1.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 1.0,
            total_time: 0.0,
        },
    ];

    assert_eq!(*recording.rows.borrow(), expected);
}

#[test]
fn events_30_time_step() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    let entry = state.set_animation(0, "events0", false);
    entry.set_track_end(&mut state, 1.0);

    recording.time.set(0.0);
    state.apply(&mut skeleton);

    recording.time.set(30.0);
    state.update(30.0);
    state.apply(&mut skeleton);

    recording.time.set(60.0);
    state.update(30.0);
    state.apply(&mut skeleton);

    let expected = vec![
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 30.0,
            total_time: 30.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 30".into(),
            track_time: 30.0,
            total_time: 30.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 30.0,
            total_time: 30.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 30.0,
            total_time: 60.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 30.0,
            total_time: 60.0,
        },
    ];

    assert_eq!(*recording.rows.borrow(), expected);
}

#[test]
fn physics_reset_timeline_uses_previous_animation_time() {
    let data = crate::SkeletonData::from_json_str(PHYSICS_RESET_JSON).unwrap();
    let state_data = AnimationStateData::new(data.clone());
    let mut state = AnimationState::new(state_data);
    let mut skeleton = Skeleton::new(data);

    skeleton.setup_pose();
    state.set_animation(0, "run", false);

    state.update(0.75);
    state.apply(&mut skeleton);
    assert!(skeleton.physics_constraints[0].reset);

    skeleton.update(0.75);
    skeleton.update_world_transform_with_physics(crate::Physics::Update);
    assert!(!skeleton.physics_constraints[0].reset);

    state.update(0.1);
    state.apply(&mut skeleton);
    assert!(!skeleton.physics_constraints[0].reset);
}

#[test]
fn physics_reset_timelines_share_one_property_slot() {
    let mut data = crate::SkeletonData::from_json_str(PHYSICS_RESET_JSON).unwrap();
    let skeleton_data = Arc::get_mut(&mut data).expect("unique skeleton data");
    skeleton_data
        .physics_constraints
        .push(crate::PhysicsConstraintData {
            name: "physics1".to_string(),
            order: 1,
            skin_required: false,
            bone: 0,
            x: 0.0,
            y: 0.0,
            rotate: 0.0,
            scale_x: 1.0,
            scale_y_mode: crate::ScaleYMode::None,
            shear_x: 0.0,
            limit: 5000.0,
            step: 1.0 / 60.0,
            inertia: 0.5,
            strength: 100.0,
            damping: 0.85,
            mass_inverse: 1.0,
            wind: 0.0,
            gravity: 0.0,
            mix: 1.0,
            inertia_global: false,
            strength_global: false,
            damping_global: false,
            mass_global: false,
            wind_global: false,
            gravity_global: false,
            mix_global: false,
        });
    let animation = crate::runtime::finalize_animation(crate::Animation {
        name: "run2".to_string(),
        duration: 1.0,
        color: crate::Animation::DEFAULT_COLOR,
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
        physics_reset_timelines: vec![
            crate::PhysicsConstraintResetTimeline {
                constraint_index: 0,
                frames: vec![0.0],
            },
            crate::PhysicsConstraintResetTimeline {
                constraint_index: 1,
                frames: vec![0.0],
            },
        ],
        slider_time_timelines: Vec::new(),
        slider_mix_timelines: Vec::new(),
        draw_order_timeline: None,
        draw_order_folder_timelines: Vec::new(),
        timeline_order: Vec::new(),
    });
    skeleton_data.animations = vec![animation.clone()];
    let state_data = AnimationStateData::new(data.clone());
    let mut state = AnimationState::new(state_data);
    let mut skeleton = Skeleton::new(data.clone());

    skeleton.setup_pose();
    state.set_animation(0, "run2", false);
    state.update(0.1);
    state.apply(&mut skeleton);
    assert!(skeleton.physics_constraints[0].reset);
}

#[test]
fn events_1_time_step() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    let entry = state.set_animation(0, "events0", false);
    entry.set_track_end(&mut state, 1.0);

    run(&mut state, &mut skeleton, &recording, 1.0, 1.01);

    let expected = vec![
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 30".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 1.0,
            total_time: 2.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 1.0,
            total_time: 2.0,
        },
    ];

    assert_eq!(*recording.rows.borrow(), expected);
}

#[test]
fn dispose_queued_entries_and_run_1_over_60() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    state.set_animation(0, "events0", false);
    state.add_animation(0, "events1", false, 0.0);
    state.add_animation(0, "events0", false, 0.0);
    state.add_animation(0, "events1", false, 0.0);
    let entry = state.set_animation(0, "events0", false);
    entry.set_track_end(&mut state, 1.0);

    run(&mut state, &mut skeleton, &recording, 1.0 / 60.0, 1.2);

    let expected = vec![
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "interrupt".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "dispose".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "dispose".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 0.483,
            total_time: 0.483,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 30".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 1.0,
            total_time: 1.017,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 1.0,
            total_time: 1.017,
        },
    ];

    let rows = recording.rows.borrow();
    assert_eq!(&**rows, expected);
}

#[test]
fn interrupt_chain_delay_0() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    state.set_animation(0, "events0", false);
    state.add_animation(0, "events1", false, 0.0);
    let entry = state.add_animation(0, "events0", false, 0.0);
    entry.set_track_end(&mut state, 1.0);

    run(&mut state, &mut skeleton, &recording, 0.1, 4.0);

    let expected = vec![
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 0.5,
            total_time: 0.5,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 30".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "interrupt".into(),
            track_time: 1.1,
            total_time: 1.1,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "start".into(),
            track_time: 0.1,
            total_time: 1.1,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 0".into(),
            track_time: 0.1,
            total_time: 1.1,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 1.1,
            total_time: 1.2,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 1.1,
            total_time: 1.2,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 14".into(),
            track_time: 0.5,
            total_time: 1.5,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 30".into(),
            track_time: 1.0,
            total_time: 2.0,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "complete".into(),
            track_time: 1.0,
            total_time: 2.0,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "interrupt".into(),
            track_time: 1.1,
            total_time: 2.1,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.1,
            total_time: 2.1,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 0.1,
            total_time: 2.1,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "end".into(),
            track_time: 1.1,
            total_time: 2.2,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "dispose".into(),
            track_time: 1.1,
            total_time: 2.2,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 0.5,
            total_time: 2.5,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 30".into(),
            track_time: 1.0,
            total_time: 3.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 1.0,
            total_time: 3.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 1.0,
            total_time: 3.1,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 1.0,
            total_time: 3.1,
        },
    ];

    assert_eq!(*recording.rows.borrow(), expected);
}

#[test]
fn interrupt_with_delay() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    state.set_animation(0, "events0", false);
    let entry = state.add_animation(0, "events1", false, 0.5);
    entry.set_track_end(&mut state, 1.0);

    run(&mut state, &mut skeleton, &recording, 0.1, 2.0);

    let expected = vec![
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 0.5,
            total_time: 0.5,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "interrupt".into(),
            track_time: 0.6,
            total_time: 0.6,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "start".into(),
            track_time: 0.1,
            total_time: 0.6,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 0".into(),
            track_time: 0.1,
            total_time: 0.6,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 0.6,
            total_time: 0.7,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 0.6,
            total_time: 0.7,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 14".into(),
            track_time: 0.5,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 30".into(),
            track_time: 1.0,
            total_time: 1.5,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "complete".into(),
            track_time: 1.0,
            total_time: 1.5,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "end".into(),
            track_time: 1.0,
            total_time: 1.6,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "dispose".into(),
            track_time: 1.0,
            total_time: 1.6,
        },
    ];

    assert_eq!(*recording.rows.borrow(), expected);
}

#[test]
fn interrupt_with_delay_and_mix_time() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    state.get_data_mut().set_mix("events0", "events1", 0.7);

    state.set_animation(0, "events0", true);
    let entry = state.add_animation(0, "events1", false, 0.9);
    entry.set_track_end(&mut state, 1.0);

    run(&mut state, &mut skeleton, &recording, 0.1, 2.0);

    let expected = vec![
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 0.5,
            total_time: 0.5,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "interrupt".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "start".into(),
            track_time: 0.1,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 0".into(),
            track_time: 0.1,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 14".into(),
            track_time: 0.5,
            total_time: 1.4,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 1.6,
            total_time: 1.7,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 1.6,
            total_time: 1.7,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 30".into(),
            track_time: 1.0,
            total_time: 1.9,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "complete".into(),
            track_time: 1.0,
            total_time: 1.9,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "end".into(),
            track_time: 1.0,
            total_time: 2.0,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "dispose".into(),
            track_time: 1.0,
            total_time: 2.0,
        },
    ];

    assert_eq!(*recording.rows.borrow(), expected);
}

#[test]
fn animation0_events_do_not_fire_during_mix() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    state.get_data_mut().set_default_mix(0.7);

    state.set_animation(0, "events0", false);
    let entry = state.add_animation(0, "events1", false, 0.4);
    entry.set_track_end(&mut state, 1.0);

    run(&mut state, &mut skeleton, &recording, 0.1, 1.5);

    let expected = vec![
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "interrupt".into(),
            track_time: 0.5,
            total_time: 0.5,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "start".into(),
            track_time: 0.1,
            total_time: 0.5,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 0".into(),
            track_time: 0.1,
            total_time: 0.5,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 14".into(),
            track_time: 0.5,
            total_time: 0.9,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 1.1,
            total_time: 1.2,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 1.1,
            total_time: 1.2,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 30".into(),
            track_time: 1.0,
            total_time: 1.4,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "complete".into(),
            track_time: 1.0,
            total_time: 1.4,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "end".into(),
            track_time: 1.0,
            total_time: 1.5,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "dispose".into(),
            track_time: 1.0,
            total_time: 1.5,
        },
    ];

    assert_eq!(*recording.rows.borrow(), expected);
}

#[test]
fn event_threshold_some_animation0_events_fire_during_mix() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    state.get_data_mut().set_mix("events0", "events1", 0.7);

    let entry = state.set_animation(0, "events0", false);
    entry.set_event_threshold(&mut state, 0.5);
    let entry = state.add_animation(0, "events1", false, 0.4);
    entry.set_track_end(&mut state, 1.0);

    run(&mut state, &mut skeleton, &recording, 0.1, 2.0);

    let expected = vec![
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "interrupt".into(),
            track_time: 0.5,
            total_time: 0.5,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "start".into(),
            track_time: 0.1,
            total_time: 0.5,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 0.5,
            total_time: 0.5,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 0".into(),
            track_time: 0.1,
            total_time: 0.5,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 14".into(),
            track_time: 0.5,
            total_time: 0.9,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 1.1,
            total_time: 1.2,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 1.1,
            total_time: 1.2,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 30".into(),
            track_time: 1.0,
            total_time: 1.4,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "complete".into(),
            track_time: 1.0,
            total_time: 1.4,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "end".into(),
            track_time: 1.0,
            total_time: 1.5,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "dispose".into(),
            track_time: 1.0,
            total_time: 1.5,
        },
    ];

    assert_eq!(*recording.rows.borrow(), expected);
}

#[test]
fn reverse_playback_queues_event_timeline_events_like_cpp() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    let entry = state.set_animation(0, "events0", false);
    entry.set_reverse(&mut state, true);
    recording.rows.borrow_mut().clear();

    recording.time.set(0.5);
    state.update(0.5);
    state.apply(&mut skeleton);

    let rows = recording.rows.borrow();
    let actual: Vec<_> = rows.iter().map(|row| row.name.as_str()).collect();
    assert_eq!(
        actual,
        vec!["event 30"],
        "reverse playback queued events: {actual:?}"
    );
}

#[test]
fn event_threshold_all_animation0_events_fire_during_mix() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    let entry = state.set_animation(0, "events0", true);
    entry.set_event_threshold(&mut state, 1.0);
    let entry = state.add_animation(0, "events1", false, 0.8);
    entry.set_mix_duration(&mut state, 0.7);
    entry.set_track_end(&mut state, 1.0);

    run(&mut state, &mut skeleton, &recording, 0.1, 2.0);

    let expected = vec![
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 0.5,
            total_time: 0.5,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "interrupt".into(),
            track_time: 0.9,
            total_time: 0.9,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "start".into(),
            track_time: 0.1,
            total_time: 0.9,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 0".into(),
            track_time: 0.1,
            total_time: 0.9,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 30".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 14".into(),
            track_time: 0.5,
            total_time: 1.3,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 1.5,
            total_time: 1.6,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 1.5,
            total_time: 1.6,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 30".into(),
            track_time: 1.0,
            total_time: 1.8,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "complete".into(),
            track_time: 1.0,
            total_time: 1.8,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "end".into(),
            track_time: 1.0,
            total_time: 1.9,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "dispose".into(),
            track_time: 1.0,
            total_time: 1.9,
        },
    ];

    assert_eq!(*recording.rows.borrow(), expected);
}

#[test]
fn looping() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    state.set_animation(0, "events0", true);

    run(&mut state, &mut skeleton, &recording, 0.1, 4.01);
    state.clear_tracks();
    state.apply(&mut skeleton);

    let expected = vec![
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 0.5,
            total_time: 0.5,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 30".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 1.5,
            total_time: 1.5,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 30".into(),
            track_time: 2.0,
            total_time: 2.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 2.0,
            total_time: 2.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 2.0,
            total_time: 2.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 2.5,
            total_time: 2.5,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 30".into(),
            track_time: 3.0,
            total_time: 3.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 3.0,
            total_time: 3.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 3.0,
            total_time: 3.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 3.5,
            total_time: 3.5,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 30".into(),
            track_time: 4.0,
            total_time: 4.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 4.0,
            total_time: 4.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 4.0,
            total_time: 4.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 4.1,
            total_time: 4.1,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 4.1,
            total_time: 4.1,
        },
    ];

    assert_eq!(*recording.rows.borrow(), expected);
}

#[test]
fn not_looping_track_end_past_animation_duration() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    state.set_animation(0, "events0", false);
    let entry = state.add_animation(0, "events1", false, 2.0);
    entry.set_track_end(&mut state, 1.0);

    run(&mut state, &mut skeleton, &recording, 0.1, 4.0);

    let expected = vec![
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 0.5,
            total_time: 0.5,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 30".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "interrupt".into(),
            track_time: 2.1,
            total_time: 2.1,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "start".into(),
            track_time: 0.1,
            total_time: 2.1,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 0".into(),
            track_time: 0.1,
            total_time: 2.1,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 2.1,
            total_time: 2.2,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 2.1,
            total_time: 2.2,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 14".into(),
            track_time: 0.5,
            total_time: 2.5,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 30".into(),
            track_time: 1.0,
            total_time: 3.0,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "complete".into(),
            track_time: 1.0,
            total_time: 3.0,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "end".into(),
            track_time: 1.0,
            total_time: 3.1,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "dispose".into(),
            track_time: 1.0,
            total_time: 3.1,
        },
    ];

    assert_eq!(*recording.rows.borrow(), expected);
}

#[test]
fn interrupt_animation_after_first_loop_complete() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    state.set_animation(0, "events0", true);

    run_with_frame(
        &mut state,
        &mut skeleton,
        &recording,
        0.1,
        6.0,
        |time, state| {
            if (time - 1.4).abs() < 0.000001 {
                let entry = state.add_animation(0, "events1", false, 0.0);
                entry.set_track_end(state, 1.0);
            }
        },
    );

    let expected = vec![
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 0.5,
            total_time: 0.5,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 30".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 1.5,
            total_time: 1.5,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 30".into(),
            track_time: 2.0,
            total_time: 2.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 2.0,
            total_time: 2.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 2.0,
            total_time: 2.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "interrupt".into(),
            track_time: 2.1,
            total_time: 2.1,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "start".into(),
            track_time: 0.1,
            total_time: 2.1,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 0".into(),
            track_time: 0.1,
            total_time: 2.1,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 2.1,
            total_time: 2.2,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 2.1,
            total_time: 2.2,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 14".into(),
            track_time: 0.5,
            total_time: 2.5,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 30".into(),
            track_time: 1.0,
            total_time: 3.0,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "complete".into(),
            track_time: 1.0,
            total_time: 3.0,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "end".into(),
            track_time: 1.0,
            total_time: 3.1,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "dispose".into(),
            track_time: 1.0,
            total_time: 3.1,
        },
    ];

    assert_eq!(*recording.rows.borrow(), expected);
}

#[test]
fn add_animation_on_empty_track() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    let entry = state.add_animation(0, "events0", false, 0.0);
    entry.set_track_end(&mut state, 1.0);

    run(&mut state, &mut skeleton, &recording, 0.1, 1.9);

    let expected = vec![
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 0.5,
            total_time: 0.5,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 30".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 1.0,
            total_time: 1.1,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 1.0,
            total_time: 1.1,
        },
    ];

    assert_eq!(*recording.rows.borrow(), expected);
}

#[test]
fn negative_track_end_clears_when_track_last_passes_it() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    let entry = state.set_animation(0, "events0", false);
    entry.set_track_end(&mut state, -0.1);

    run(&mut state, &mut skeleton, &recording, 0.1, 0.2);

    let expected = vec![
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 0.0,
            total_time: 0.1,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 0.0,
            total_time: 0.1,
        },
    ];

    assert_eq!(*recording.rows.borrow(), expected);
}

#[test]
fn end_time_beyond_non_looping_animation_duration() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    let entry = state.set_animation(0, "events0", false);
    entry.set_track_end(&mut state, 9.0);

    run(&mut state, &mut skeleton, &recording, 0.1, 10.0);

    let expected = vec![
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 0.5,
            total_time: 0.5,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 30".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 9.0,
            total_time: 9.1,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 9.0,
            total_time: 9.1,
        },
    ];

    assert_eq!(*recording.rows.borrow(), expected);
}

#[test]
fn looping_animation_time_uses_cpp_fmod_semantics() {
    let (mut state, _skeleton, _recording) = setup();

    let entry = state.set_animation(0, "events0", true);
    entry.set_animation_start(&mut state, 0.2);
    entry.set_animation_end(&mut state, 0.8);
    entry.set_track_time(&mut state, 0.6);

    assert_eq!(
        with_track_entry(&state, 0, |entry| round3(entry.animation_time())).unwrap(),
        0.2
    );

    entry.set_track_time(&mut state, -0.1);

    assert_eq!(
        with_track_entry(&state, 0, |entry| round3(entry.animation_time())).unwrap(),
        0.1
    );
}

#[test]
fn looping_track_complete_uses_cpp_integer_truncation() {
    let (mut state, _skeleton, _recording) = setup();

    let entry = state.set_animation(0, "events0", true);
    entry.set_animation_start(&mut state, 0.2);
    entry.set_animation_end(&mut state, 0.8);
    entry.set_track_time(&mut state, -0.1);

    assert_eq!(
        with_track_entry(&state, 0, |entry| round3(entry.track_complete())).unwrap(),
        0.6
    );
}

#[test]
fn non_looping_animation_time_uses_cpp_exact_duration_comparison() {
    let (mut state, _skeleton, _recording) = setup();

    let entry = state.set_animation(0, "events0", false);
    let duration = entry
        .entry(&state)
        .map(|entry| entry.animation().duration)
        .expect("current entry");
    let animation_end = duration - 0.000_000_5;
    entry.set_animation_end(&mut state, animation_end);
    entry.set_track_time(&mut state, duration + 0.1);

    assert_eq!(
        with_track_entry(&state, 0, |entry| entry.animation_time()).unwrap(),
        animation_end
    );
}

#[test]
fn non_looping_animation_time_clamps_to_animation_end_past_duration() {
    let (mut state, _skeleton, _recording) = setup();

    let entry = state.set_animation(0, "events0", false);
    let duration = entry
        .entry(&state)
        .map(|entry| entry.animation().duration)
        .expect("current entry");
    let animation_end = duration + 0.25;
    entry.set_animation_end(&mut state, animation_end);
    entry.set_track_time(&mut state, duration + 0.5);

    assert_eq!(
        with_track_entry(&state, 0, |entry| round3(entry.animation_time())).unwrap(),
        round3(animation_end)
    );
}

#[test]
fn non_looping_complete_uses_cpp_exact_boundary() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    let entry = state.set_animation(0, "events0", false);
    entry.set_track_time(&mut state, 1.0 - 0.000_000_5);
    state.apply(&mut skeleton);

    assert!(
        recording
            .rows
            .borrow()
            .iter()
            .all(|row| row.name != "complete")
    );

    entry.set_track_time(&mut state, 1.0);
    state.apply(&mut skeleton);

    assert!(
        recording
            .rows
            .borrow()
            .iter()
            .any(|row| row.name == "complete")
    );
}

#[test]
fn looping_complete_uses_cpp_exact_zero_duration_check() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    let entry = state.set_animation(0, "events0", true);
    entry.set_animation_start(&mut state, 0.2);
    entry.set_animation_end(&mut state, 0.2 + 0.000_000_5);
    state.apply(&mut skeleton);

    assert!(
        recording
            .rows
            .borrow()
            .iter()
            .all(|row| row.name != "complete")
    );
}

#[test]
fn zero_duration_events_use_cpp_zero_split_semantics() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    let entry = state.set_animation(0, "events0", true);
    entry.set_animation_start(&mut state, 0.2);
    entry.set_animation_end(&mut state, 0.2);
    entry.set_track_time(&mut state, 0.0);
    recording.rows.borrow_mut().clear();

    state.apply(&mut skeleton);

    let rows = recording.rows.borrow();
    let event_names: Vec<_> = rows.iter().map(|row| row.name.as_str()).collect();
    assert_eq!(event_names, vec!["complete"]);
}

#[test]
fn looping_with_animation_start() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    let entry = state.set_animation(0, "events0", true);
    entry.set_animation_last(&mut state, 0.6);
    entry.set_animation_start(&mut state, 0.6);

    run(&mut state, &mut skeleton, &recording, 0.1, 1.4);
    state.clear_tracks();
    state.apply(&mut skeleton);

    let expected = vec![
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 30".into(),
            track_time: 0.4,
            total_time: 0.4,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 0.4,
            total_time: 0.4,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 30".into(),
            track_time: 0.8,
            total_time: 0.8,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 0.8,
            total_time: 0.8,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 30".into(),
            track_time: 1.2,
            total_time: 1.2,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 1.2,
            total_time: 1.2,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 1.4,
            total_time: 1.4,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 1.4,
            total_time: 1.4,
        },
    ];

    assert_eq!(*recording.rows.borrow(), expected);
}

#[test]
fn looping_with_animation_start_and_end() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    let entry = state.set_animation(0, "events0", true);
    entry.set_animation_start(&mut state, 0.2);
    entry.set_animation_last(&mut state, 0.2);
    entry.set_animation_end(&mut state, 0.8);

    run(&mut state, &mut skeleton, &recording, 0.1, 1.8);
    state.clear_tracks();
    state.apply(&mut skeleton);

    let expected = vec![
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 0.3,
            total_time: 0.3,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 0.6,
            total_time: 0.6,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 0.9,
            total_time: 0.9,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 1.2,
            total_time: 1.2,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 1.5,
            total_time: 1.5,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 1.8,
            total_time: 1.8,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 1.8,
            total_time: 1.8,
        },
    ];

    assert_eq!(*recording.rows.borrow(), expected);
}

#[test]
fn non_looping_with_animation_start_and_end() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    let entry = state.set_animation(0, "events0", false);
    entry.set_animation_start(&mut state, 0.2);
    entry.set_animation_last(&mut state, 0.2);
    entry.set_animation_end(&mut state, 0.8);
    entry.set_track_end(&mut state, 1.0);

    run(&mut state, &mut skeleton, &recording, 0.1, 1.8);

    let expected = vec![
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 0.3,
            total_time: 0.3,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 0.6,
            total_time: 0.6,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 1.0,
            total_time: 1.1,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 1.0,
            total_time: 1.1,
        },
    ];

    assert_eq!(*recording.rows.borrow(), expected);
}

#[test]
fn mix_out_looping_with_animation_start_and_end() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    let entry = state.set_animation(0, "events0", true);
    entry.set_animation_start(&mut state, 0.2);
    entry.set_animation_last(&mut state, 0.2);
    entry.set_animation_end(&mut state, 0.8);
    entry.set_event_threshold(&mut state, 1.0);

    let entry = state.add_animation(0, "events1", false, 0.7);
    entry.set_mix_duration(&mut state, 0.7);
    entry.set_track_end(&mut state, 1.0);

    run(&mut state, &mut skeleton, &recording, 0.1, 2.0);

    let expected = vec![
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 0.3,
            total_time: 0.3,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 0.6,
            total_time: 0.6,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "interrupt".into(),
            track_time: 0.8,
            total_time: 0.8,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "start".into(),
            track_time: 0.1,
            total_time: 0.8,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 0".into(),
            track_time: 0.1,
            total_time: 0.8,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 0.9,
            total_time: 0.9,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 1.2,
            total_time: 1.2,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 14".into(),
            track_time: 0.5,
            total_time: 1.2,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 1.4,
            total_time: 1.5,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 1.4,
            total_time: 1.5,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 30".into(),
            track_time: 1.0,
            total_time: 1.7,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "complete".into(),
            track_time: 1.0,
            total_time: 1.7,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "end".into(),
            track_time: 1.0,
            total_time: 1.8,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "dispose".into(),
            track_time: 1.0,
            total_time: 1.8,
        },
    ];

    assert_eq!(*recording.rows.borrow(), expected);
}

#[test]
fn set_animation_with_track_entry_mix() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    state.set_animation(0, "events0", true);

    run_with_frame(
        &mut state,
        &mut skeleton,
        &recording,
        0.1,
        2.1,
        |time, state| {
            if (time - 1.0).abs() < 0.000001 {
                let entry = state.set_animation(0, "events1", false);
                entry.set_mix_duration(state, 0.7);
                entry.set_track_end(state, 1.0);
            }
        },
    );

    let expected = vec![
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 0.5,
            total_time: 0.5,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 30".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "interrupt".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 0".into(),
            track_time: 0.1,
            total_time: 1.1,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 14".into(),
            track_time: 0.5,
            total_time: 1.5,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 1.7,
            total_time: 1.8,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 1.7,
            total_time: 1.8,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 30".into(),
            track_time: 1.0,
            total_time: 2.0,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "complete".into(),
            track_time: 1.0,
            total_time: 2.0,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "end".into(),
            track_time: 1.0,
            total_time: 2.1,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "dispose".into(),
            track_time: 1.0,
            total_time: 2.1,
        },
    ];

    assert_eq!(*recording.rows.borrow(), expected);
}

#[test]
fn set_animation_twice() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    state.set_animation(0, "events0", false); // First should be ignored.
    state.set_animation(0, "events1", false);

    run_with_frame(
        &mut state,
        &mut skeleton,
        &recording,
        0.1,
        1.9,
        |time, state| {
            if (time - 0.8).abs() < 0.000001 {
                state.set_animation(0, "events0", false); // First should be ignored.
                let entry = state.set_animation(0, "events2", false);
                entry.set_track_end(state, 1.0);
            }
        },
    );

    let expected = vec![
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "interrupt".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 0".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 0.0,
            total_time: 0.1,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 0.0,
            total_time: 0.1,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 14".into(),
            track_time: 0.5,
            total_time: 0.5,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "interrupt".into(),
            track_time: 0.8,
            total_time: 0.8,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.8,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "interrupt".into(),
            track_time: 0.0,
            total_time: 0.8,
        },
        ResultRow {
            animation_name: "events2".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.8,
        },
        ResultRow {
            animation_name: "events2".into(),
            name: "event 0".into(),
            track_time: 0.1,
            total_time: 0.9,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "end".into(),
            track_time: 0.9,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "dispose".into(),
            track_time: 0.9,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 0.1,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 0.1,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events2".into(),
            name: "event 14".into(),
            track_time: 0.5,
            total_time: 1.3,
        },
        ResultRow {
            animation_name: "events2".into(),
            name: "event 30".into(),
            track_time: 1.0,
            total_time: 1.8,
        },
        ResultRow {
            animation_name: "events2".into(),
            name: "complete".into(),
            track_time: 1.0,
            total_time: 1.8,
        },
        ResultRow {
            animation_name: "events2".into(),
            name: "end".into(),
            track_time: 1.0,
            total_time: 1.9,
        },
        ResultRow {
            animation_name: "events2".into(),
            name: "dispose".into(),
            track_time: 1.0,
            total_time: 1.9,
        },
    ];

    assert_eq!(*recording.rows.borrow(), expected);
}

#[test]
fn set_animation_twice_with_multiple_mixing() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    state.get_data_mut().set_default_mix(0.6);

    state.set_animation(0, "events0", false); // First should be ignored.
    state.set_animation(0, "events1", false);

    run_with_frame(
        &mut state,
        &mut skeleton,
        &recording,
        0.1,
        1.5,
        |time, state| {
            if (time - 0.2).abs() < 0.000001 {
                state.set_animation(0, "events0", false); // First should be ignored.
                state.set_animation(0, "events2", false);
            }
            if (time - 0.4).abs() < 0.000001 {
                state.set_animation(0, "events1", false); // First should be ignored.
                let entry = state.set_animation(0, "events0", false);
                entry.set_track_end(state, 1.0);
            }
        },
    );

    let expected = vec![
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "interrupt".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 0".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "interrupt".into(),
            track_time: 0.2,
            total_time: 0.2,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.2,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "interrupt".into(),
            track_time: 0.0,
            total_time: 0.2,
        },
        ResultRow {
            animation_name: "events2".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.2,
        },
        ResultRow {
            animation_name: "events2".into(),
            name: "event 0".into(),
            track_time: 0.1,
            total_time: 0.3,
        },
        ResultRow {
            animation_name: "events2".into(),
            name: "interrupt".into(),
            track_time: 0.2,
            total_time: 0.4,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.4,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "interrupt".into(),
            track_time: 0.0,
            total_time: 0.4,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.4,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 0.1,
            total_time: 0.5,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 0.6,
            total_time: 0.7,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 0.6,
            total_time: 0.7,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "end".into(),
            track_time: 0.8,
            total_time: 0.9,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "dispose".into(),
            track_time: 0.8,
            total_time: 0.9,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 0.6,
            total_time: 0.9,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 0.6,
            total_time: 0.9,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 0.5,
            total_time: 0.9,
        },
        ResultRow {
            animation_name: "events2".into(),
            name: "end".into(),
            track_time: 0.8,
            total_time: 1.1,
        },
        ResultRow {
            animation_name: "events2".into(),
            name: "dispose".into(),
            track_time: 0.8,
            total_time: 1.1,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "end".into(),
            track_time: 0.6,
            total_time: 1.1,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "dispose".into(),
            track_time: 0.6,
            total_time: 1.1,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 30".into(),
            track_time: 1.0,
            total_time: 1.4,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 1.0,
            total_time: 1.4,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 1.0,
            total_time: 1.5,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 1.0,
            total_time: 1.5,
        },
    ];

    assert_eq!(*recording.rows.borrow(), expected);
}

#[test]
fn add_animation_with_delay_on_empty_track() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    let entry = state.add_animation(0, "events0", false, 5.0);
    entry.set_track_end(&mut state, 1.0);

    run(&mut state, &mut skeleton, &recording, 0.1, 10.0);

    let expected = vec![
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 0.0,
            total_time: 5.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 0.5,
            total_time: 5.5,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 30".into(),
            track_time: 1.0,
            total_time: 6.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 1.0,
            total_time: 6.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 1.0,
            total_time: 6.1,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 1.0,
            total_time: 6.1,
        },
    ];

    assert_eq!(*recording.rows.borrow(), expected);
}

#[test]
fn add_animation_on_empty_track_sets_delay_after_start_event_like_cpp() {
    #[derive(Default)]
    struct DelayOnStart {
        delays: Rc<RefCell<Vec<f32>>>,
    }

    impl AnimationStateListener for DelayOnStart {
        fn on_event(
            &mut self,
            state: &mut AnimationState,
            entry: TrackEntryHandle,
            event: &AnimationStateEvent,
        ) {
            if matches!(event, AnimationStateEvent::Start)
                && let Some(delay) = entry.entry(state).map(|entry| entry.delay())
            {
                self.delays.borrow_mut().push(delay);
            }
        }
    }

    let data = crate::SkeletonData::from_json_str(TEST_JSON).unwrap();
    let mut state = AnimationState::new(AnimationStateData::new(data));
    let delays = Rc::new(RefCell::new(Vec::new()));
    state.set_listener(DelayOnStart {
        delays: delays.clone(),
    });

    let entry = state.add_animation(0, "events0", false, 5.0);

    assert_eq!(*delays.borrow(), vec![0.0]);
    assert_eq!(entry.entry(&state).map(|entry| entry.delay()), Some(5.0));
}

#[test]
fn state_time_scale_scales_update_and_queue_progression() {
    let (mut state, mut skeleton, _recording) = setup();

    state.set_animation(0, "events0", false);
    state.add_animation(0, "events1", false, 0.0);
    state.set_time_scale(0.5);

    for _ in 0..3 {
        state.update(1.0);
        state.apply(&mut skeleton);
    }

    assert_eq!(
        with_track_entry(&state, 0, |entry| entry.animation().name.clone())
            .expect("track 0 should have advanced to the queued animation"),
        "events1"
    );
    assert_eq!(
        with_track_entry(&state, 0, |entry| round3(entry.track_time()))
            .expect("track 0 should remain active"),
        0.5
    );
}

#[test]
fn update_accepts_negative_delta_like_cpp() {
    let (mut state, _skeleton, _recording) = setup();

    state.set_animation(0, "events0", false);
    state.update(1.0);
    state.update(-0.25);

    assert_eq!(
        with_track_entry(&state, 0, |entry| round3(entry.track_time()))
            .expect("track 0 should remain active"),
        0.75
    );
}

#[test]
fn set_animation_during_animation_state_listener() {
    #[derive(Default)]
    struct Reentrant;

    impl AnimationStateListener for Reentrant {
        fn on_event(
            &mut self,
            state: &mut AnimationState,
            entry: TrackEntryHandle,
            event: &AnimationStateEvent,
        ) {
            let Some(entry) = entry.entry(state) else {
                return;
            };
            let animation_name = entry.animation().name.clone();
            let track_index = entry.track_index();

            match event {
                AnimationStateEvent::Start => {
                    if animation_name == "events0" {
                        state.set_animation(1, "events1", false);
                    }
                }
                AnimationStateEvent::Interrupt => {
                    state.add_animation(3, "events1", false, 0.0);
                }
                AnimationStateEvent::End => {
                    if animation_name == "events0" {
                        state.set_animation(0, "events1", false);
                    }
                }
                AnimationStateEvent::Dispose => {
                    if animation_name == "events0" {
                        state.set_animation(1, "events1", false);
                    }
                }
                AnimationStateEvent::Complete => {
                    if animation_name == "events0" {
                        state.set_animation(1, "events1", false);
                    }
                }
                AnimationStateEvent::Event(_) => {
                    if track_index != 2 {
                        state.set_animation(2, "events1", false);
                    }
                }
            }
        }
    }

    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(Reentrant);

    state.add_animation(0, "events0", false, 0.0);
    state.add_animation(0, "events1", false, 0.0);
    let entry = state.set_animation(1, "events1", false);
    entry.set_track_end(&mut state, 1.0);

    run(&mut state, &mut skeleton, &recording, 0.1, 2.0);
}

#[test]
fn set_animation_discards_queued_entries_before_interrupt_like_cpp() {
    let (mut state, _skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    state.set_animation(0, "events0", false);
    state.add_animation(0, "events1", false, 0.0);
    recording.rows.borrow_mut().clear();

    state.set_animation(0, "events2", false);

    let actual: Vec<_> = recording
        .rows
        .borrow()
        .iter()
        .map(|row| (row.animation_name.clone(), row.name.clone()))
        .collect();
    assert_eq!(
        actual,
        vec![
            ("events1".into(), "dispose".into()),
            ("events0".into(), "interrupt".into()),
            ("events2".into(), "start".into()),
        ]
    );
}

#[test]
fn clear_track() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    let entry = state.add_animation(0, "events0", false, 0.0);
    entry.set_track_end(&mut state, 1.0);

    run_with_frame(
        &mut state,
        &mut skeleton,
        &recording,
        0.1,
        2.0,
        |time, state| {
            if time == 0.7 {
                state.clear_track(0);
            }
        },
    );

    let expected = vec![
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 0.5,
            total_time: 0.5,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 0.7,
            total_time: 0.7,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 0.7,
            total_time: 0.7,
        },
    ];

    assert_eq!(*recording.rows.borrow(), expected);
}

#[test]
fn clear_track_disposes_queued_entries_before_mixing_from_like_cpp() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    let first = state.set_animation(0, "events0", false);
    let second = state.add_animation(0, "events1", false, 0.5);
    second.set_mix_duration(&mut state, 0.5);
    let _third = state.add_animation(0, "events2", false, 0.0);

    state.update(0.6);
    state.apply(&mut skeleton);
    state.update(0.0);
    recording.rows.borrow_mut().clear();

    state.clear_track(0);

    let actual: Vec<_> = recording
        .rows
        .borrow()
        .iter()
        .map(|row| (row.animation_name.clone(), row.name.clone()))
        .collect();
    assert_eq!(
        actual,
        vec![
            ("events1".into(), "end".into()),
            ("events1".into(), "dispose".into()),
            ("events2".into(), "dispose".into()),
            ("events0".into(), "end".into()),
            ("events0".into(), "dispose".into()),
        ],
        "clear_track should drain queued disposals before mixing-from ends"
    );

    assert!(first.entry(&state).is_none());
}

#[test]
fn set_empty_animation() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    let entry = state.add_animation(0, "events0", false, 0.0);
    entry.set_track_end(&mut state, 1.0);

    run_with_frame(
        &mut state,
        &mut skeleton,
        &recording,
        0.1,
        2.0,
        |time, state| {
            if time == 0.7 {
                state.set_empty_animation(0, 0.0);
            }
        },
    );

    let expected = vec![
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 0.5,
            total_time: 0.5,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "interrupt".into(),
            track_time: 0.7,
            total_time: 0.7,
        },
        ResultRow {
            animation_name: "<empty>".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.7,
        },
        ResultRow {
            animation_name: "<empty>".into(),
            name: "complete".into(),
            track_time: 0.1,
            total_time: 0.8,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 0.8,
            total_time: 0.9,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 0.8,
            total_time: 0.9,
        },
        ResultRow {
            animation_name: "<empty>".into(),
            name: "end".into(),
            track_time: 0.2,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "<empty>".into(),
            name: "dispose".into(),
            track_time: 0.2,
            total_time: 1.0,
        },
    ];

    assert_eq!(*recording.rows.borrow(), expected);
}

#[test]
fn track_entry_official_query_helpers_and_loop_setter_follow_cpp_state() {
    let (mut state, mut skeleton, _recording) = setup();

    let entry = state.set_animation(0, "events0", true);
    with_track_entry(&state, 0, |entry| {
        assert!(entry.looped());
        assert!(!entry.is_complete());
        assert!(!entry.was_applied());
        assert!(!entry.is_empty_animation());
    });

    entry.set_loop(&mut state, false);
    with_track_entry(&state, 0, |entry| {
        assert!(!entry.looped());
    });

    entry.set_track_time(&mut state, 0.25);
    entry.set_mix_time(&mut state, 0.125);
    with_track_entry(&state, 0, |entry| {
        assert_eq!(entry.track_time(), 0.25);
        assert_eq!(entry.mix_time(), 0.125);
    });

    state.update(0.5);
    state.apply(&mut skeleton);
    with_track_entry(&state, 0, |entry| {
        assert!(entry.was_applied());
        assert!(!entry.is_complete());
    });

    state.update(0.6);
    state.apply(&mut skeleton);
    with_track_entry(&state, 0, |entry| {
        assert!(entry.is_complete());
    });

    let empty = state.set_empty_animation(0, 0.2);
    with_track_entry(&state, 0, |entry| {
        assert!(entry.is_empty_animation());
        assert_eq!(entry.track_end(), 0.2);
    });
    empty.set_loop(&mut state, true);
    with_track_entry(&state, 0, |entry| {
        assert!(entry.looped());
    });
}

#[test]
fn track_entry_queue_neighbors_follow_cpp_previous_next_chain() {
    let (mut state, mut skeleton, _recording) = setup();

    let first = state.set_animation(0, "events0", false);
    let second = state.add_animation(0, "events1", false, 0.5);
    second.set_mix_duration(&mut state, 0.5);
    let third = state.add_animation(0, "events2", false, 0.0);

    assert_eq!(first.previous(&state), None);
    assert_eq!(first.next(&state), Some(second));
    assert_eq!(second.previous(&state), Some(first));
    assert_eq!(second.next(&state), Some(third));
    assert_eq!(third.previous(&state), Some(second));
    assert_eq!(third.next(&state), None);
    assert!(!first.is_next_ready(&state));

    state.apply(&mut skeleton);
    assert!(!first.is_next_ready(&state));

    state.update(0.6);
    state.apply(&mut skeleton);
    assert!(first.is_next_ready(&state));

    state.update(0.0);
    assert_eq!(first.next(&state), None);
    assert_eq!(second.previous(&state), None);
    assert_eq!(second.next(&state), Some(third));
    assert_eq!(third.previous(&state), Some(second));
    assert_eq!(second.mixing_from(&state), Some(first));
    assert_eq!(first.mixing_to(&state), Some(second));
}

#[test]
fn set_empty_animations_sets_empty_entries_for_active_tracks() {
    let (mut state, _skeleton, _recording) = setup();

    state.set_animation(0, "events0", true);
    state.add_animation(0, "events1", true, 0.0);
    state.set_animation(2, "events2", true);

    state.set_empty_animations(0.4);

    assert!(state.get_current(0).unwrap().next(&state).is_none());
    with_track_entry(&state, 0, |entry| {
        assert_eq!(entry.animation().name, "<empty>");
        assert_eq!(entry.mix_duration(), 0.4);
        assert_eq!(entry.track_end(), 0.4);
    });
    assert!(with_track_entry(&state, 1, |_| ()).is_none());
    with_track_entry(&state, 2, |entry| {
        assert_eq!(entry.animation().name, "<empty>");
        assert_eq!(entry.mix_duration(), 0.4);
        assert_eq!(entry.track_end(), 0.4);
    })
    .unwrap();
}

#[test]
fn set_empty_animations_is_noop_without_active_tracks() {
    let (mut state, _skeleton, _recording) = setup();

    state.set_empty_animations(0.4);

    assert!(state.get_tracks().is_empty());
}

#[test]
fn set_empty_animations_stores_mix_duration_directly() {
    let (mut state, _skeleton, _recording) = setup();

    state.set_animation(0, "events0", true);

    state.set_empty_animations(-0.1);
    assert_eq!(
        with_track_entry(&state, 0, |entry| entry.mix_duration()).unwrap(),
        -0.1
    );

    state.set_empty_animations(f32::INFINITY);
    assert_eq!(
        with_track_entry(&state, 0, |entry| entry.mix_duration()).unwrap(),
        f32::INFINITY
    );
}

#[test]
fn set_animation_same_animation_with_negative_track_time_still_mixes_from_applied_entry() {
    let (mut state, mut skeleton, _recording) = setup();

    let first = state.set_animation(0, "events0", false);
    state.apply(&mut skeleton);

    first.set_track_time(&mut state, -0.1);

    let second = state.set_animation(0, "events0", false);

    assert_eq!(second.mixing_from(&state), Some(first));
    assert!(first.next(&state).is_none());
}

#[test]
fn set_animation_same_unapplied_animation_replaces_without_mixing_like_cpp() {
    let (mut state, _skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    let first = state.set_animation(0, "events0", false);
    recording.rows.borrow_mut().clear();

    let second = state.set_animation(0, "events0", false);

    assert_eq!(second.mixing_from(&state), None);
    assert!(first.entry(&state).is_none());
    let rows = recording.rows.borrow();
    let events = rows.iter().map(|row| row.name.as_str()).collect::<Vec<_>>();
    assert_eq!(events, vec!["interrupt", "end", "dispose", "start"]);
}

#[test]
fn animation_state_clear_listener_restores_noop_listener_like_cpp() {
    struct Counter {
        events: Rc<Cell<usize>>,
    }

    impl AnimationStateListener for Counter {
        fn on_event(
            &mut self,
            _state: &mut AnimationState,
            _entry: TrackEntryHandle,
            _event: &AnimationStateEvent,
        ) {
            self.events.set(self.events.get() + 1);
        }
    }

    let data = crate::SkeletonData::from_json_str(TEST_JSON).unwrap();
    let mut state = AnimationState::new(AnimationStateData::new(data));
    let events = Rc::new(Cell::new(0));

    state.set_listener(Counter {
        events: events.clone(),
    });
    state.clear_listener();
    state.set_animation(0, "events0", false);

    assert_eq!(events.get(), 0);
}

#[test]
fn track_entry_clear_listener_restores_noop_listener_like_cpp() {
    struct Counter {
        events: Rc<Cell<usize>>,
    }

    impl TrackEntryListener for Counter {
        fn on_event(
            &mut self,
            _state: &mut AnimationState,
            _entry: TrackEntryHandle,
            _event: &AnimationStateEvent,
        ) {
            self.events.set(self.events.get() + 1);
        }
    }

    let (mut state, _skeleton, _recording) = setup();
    let events = Rc::new(Cell::new(0));

    let entry = state.set_animation(0, "events0", false);
    entry.set_listener(
        &mut state,
        Counter {
            events: events.clone(),
        },
    );
    entry.clear_listener(&mut state);
    state.clear_track(0);

    assert_eq!(events.get(), 0);
}

#[test]
fn track_entry_listener() {
    #[derive(Clone)]
    struct Bits {
        counter: Rc<Cell<i32>>,
    }

    impl TrackEntryListener for Bits {
        fn on_event(
            &mut self,
            _state: &mut AnimationState,
            _entry: TrackEntryHandle,
            event: &AnimationStateEvent,
        ) {
            let add = match event {
                AnimationStateEvent::Start => 1 << 1,
                AnimationStateEvent::Interrupt => 1 << 5,
                AnimationStateEvent::End => 1 << 9,
                AnimationStateEvent::Dispose => 1 << 13,
                AnimationStateEvent::Complete => 1 << 17,
                AnimationStateEvent::Event(_) => 1 << 21,
            };
            self.counter.set(self.counter.get() + add);
        }
    }

    let (mut state, mut skeleton, recording) = setup();
    let counter = Rc::new(Cell::new(0));

    let entry = state.add_animation(0, "events0", false, 0.0);
    entry.set_listener(
        &mut state,
        Bits {
            counter: counter.clone(),
        },
    );

    state.add_animation(0, "events0", false, 0.0);
    state.add_animation(0, "events1", false, 0.0);
    let entry = state.set_animation(1, "events1", false);
    entry.set_track_end(&mut state, 1.0);

    run(&mut state, &mut skeleton, &recording, 0.1, 10.0);

    assert_eq!(counter.get(), 15082016);
}

#[test]
fn looping_with_track_end_2p6() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    let entry = state.set_animation(0, "events0", true);
    entry.set_track_end(&mut state, 2.6);

    run(&mut state, &mut skeleton, &recording, 0.1, 3.0);

    let expected = vec![
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 0.5,
            total_time: 0.5,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 30".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 1.5,
            total_time: 1.5,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 30".into(),
            track_time: 2.0,
            total_time: 2.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 2.0,
            total_time: 2.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 2.0,
            total_time: 2.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 2.5,
            total_time: 2.5,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 2.6,
            total_time: 2.7,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 2.6,
            total_time: 2.7,
        },
    ];

    assert_eq!(*recording.rows.borrow(), expected);
}

#[test]
fn set_next() {
    let (mut state, mut skeleton, recording) = setup();
    state.set_listener(RecordingListener {
        recording: recording.clone(),
    });

    state.set_animation(0, "events0", false);
    let entry = state.add_animation(0, "events1", false, 0.0);
    entry.set_track_end(&mut state, 1.0);

    run(&mut state, &mut skeleton, &recording, 0.1, 3.0);

    let expected = vec![
        ResultRow {
            animation_name: "events0".into(),
            name: "start".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 0".into(),
            track_time: 0.0,
            total_time: 0.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 14".into(),
            track_time: 0.5,
            total_time: 0.5,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "event 30".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "complete".into(),
            track_time: 1.0,
            total_time: 1.0,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "interrupt".into(),
            track_time: 1.1,
            total_time: 1.1,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "start".into(),
            track_time: 0.1,
            total_time: 1.1,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 0".into(),
            track_time: 0.1,
            total_time: 1.1,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "end".into(),
            track_time: 1.1,
            total_time: 1.2,
        },
        ResultRow {
            animation_name: "events0".into(),
            name: "dispose".into(),
            track_time: 1.1,
            total_time: 1.2,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 14".into(),
            track_time: 0.5,
            total_time: 1.5,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "event 30".into(),
            track_time: 1.0,
            total_time: 2.0,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "complete".into(),
            track_time: 1.0,
            total_time: 2.0,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "end".into(),
            track_time: 1.0,
            total_time: 2.1,
        },
        ResultRow {
            animation_name: "events1".into(),
            name: "dispose".into(),
            track_time: 1.0,
            total_time: 2.1,
        },
    ];

    assert_eq!(*recording.rows.borrow(), expected);
}

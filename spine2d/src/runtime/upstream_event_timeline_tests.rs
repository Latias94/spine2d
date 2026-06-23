use super::animation_state::collect_events_for_tests;
use crate::{Event, EventTimeline};

#[derive(Debug)]
struct Fail(String);

fn make_timeline(frames: &[f32]) -> (EventTimeline, Vec<char>) {
    let mut names = Vec::with_capacity(frames.len());
    let mut events = Vec::with_capacity(frames.len());
    for (i, &time) in frames.iter().enumerate() {
        let ch = (b'a' + (i as u8)) as char;
        names.push(ch);
        events.push(Event {
            time,
            name: "event".to_string(),
            int_value: 0,
            float_value: 0.0,
            string: ch.to_string(),
            audio_path: String::new(),
            volume: 1.0,
            balance: 0.0,
        });
    }
    (EventTimeline { events }, names)
}

fn distinct_count(frames: &[f32]) -> usize {
    let mut distinct = 0usize;
    let mut last = f32::NAN;
    for &t in frames {
        if !(t == last) {
            distinct += 1;
            last = t;
        }
    }
    distinct
}

fn fire(
    frames: &[f32],
    expected_names: &[char],
    time_start: f32,
    time_end: f32,
    mut time_step: f32,
    looped: bool,
) -> Result<(), Fail> {
    if frames.is_empty() {
        return Ok(());
    }
    time_step = time_step.max(0.00001);

    let mut event_index = 0usize;
    while frames[event_index] < time_start {
        event_index += 1;
        if event_index >= frames.len() {
            return Ok(());
        }
    }

    let mut events_count = frames.len();
    while events_count > 0 && frames[events_count - 1] > time_end {
        events_count -= 1;
    }
    if events_count <= event_index {
        return Ok(());
    }
    events_count -= event_index;

    let duration = frames[event_index + events_count - 1];
    if looped && duration > 0.0 {
        while time_step > duration / 2.0 {
            time_step /= 2.0;
        }
    }

    let (timeline, _) = make_timeline(frames);
    let mut fired_events_count = 0usize;
    let mut i = 0i32;
    let mut last_time = time_start - 0.00001;
    loop {
        let time = (time_start + time_step * (i as f32)).min(time_end);
        let mut last_time_looped = last_time;
        let mut time_looped = time;
        if looped && duration != 0.0 {
            // Match upstream libgdx tests: `%` keeps the sign for negative values, so a negative
            // `last_time` stays negative instead of wrapping to `duration - epsilon`.
            last_time_looped %= duration;
            time_looped %= duration;
        }

        let fired = collect_events_for_tests(&timeline, last_time_looped, time_looped);

        for event in fired {
            let fired = event
                .string
                .chars()
                .next()
                .ok_or_else(|| Fail("Event string was empty.".to_string()))?;

            if looped {
                event_index %= expected_names.len();
            } else if fired_events_count >= events_count {
                let _ = collect_events_for_tests(&timeline, last_time_looped, time_looped);
                return Err(Fail(format!(
                    "Too many events fired. frames={frames:?} time_start={time_start} time_end={time_end} time_step={time_step} looped={looped}"
                )));
            }

            if fired != expected_names[event_index] {
                let _ = collect_events_for_tests(&timeline, last_time_looped, time_looped);
                return Err(Fail(format!(
                    "Wrong event fired: got {fired:?}, expected {:?}. frames={frames:?} time_start={time_start} time_end={time_end} time_step={time_step} looped={looped}",
                    expected_names[event_index]
                )));
            }

            event_index += 1;
            fired_events_count += 1;
        }

        if time >= time_end {
            break;
        }
        last_time = time;
        i += 1;
    }

    if fired_events_count < events_count {
        return Err(Fail(format!(
            "Event not fired (expected at least {events_count}, got {}). frames={frames:?} time_start={time_start} time_end={time_end} time_step={time_step} looped={looped}",
            fired_events_count
        )));
    }
    Ok(())
}

fn run(frames: &[f32], time_start: f32, time_end: f32, time_step: f32) {
    let (_, names) = make_timeline(frames);
    let distinct = distinct_count(frames);

    // Matches upstream: run non-looping first, then looping, and if it fails, rerun with a more
    // descriptive error.
    if let Err(err) = fire(frames, &names, time_start, time_end, time_step, false)
        .and_then(|_| fire(frames, &names, time_start, time_end, time_step, true))
    {
        let mut message = err.0;
        message.push_str(&format!(
            " (distinct_frame_times={distinct}, frames={frames:?}, time_start={time_start}, time_end={time_end}, time_step={time_step})"
        ));
        panic!("{message}");
    }
}

fn test_frames(frames: &[f32]) {
    let max_frame = frames.iter().copied().fold(0.0f32, |acc, v| acc.max(v));

    run(frames, 0.0, 99.0, 0.1);
    run(frames, 0.0, max_frame, 0.1);
    run(frames, frames[0], 999.0, 2.0);
    run(frames, frames[0], max_frame, 0.1);
    run(frames, 0.0, max_frame, (max_frame / 100.0).ceil());
    run(frames, 0.0, 99.0, 0.1);
    run(frames, 0.0, 999.0, 100.0);

    if distinct_count(frames) > 1 {
        let epsilon = 0.02;
        run(frames, frames[0], max_frame - epsilon, 0.1);
        run(frames, 0.0, max_frame - epsilon, 0.1);
        run(frames, frames[0] + epsilon, max_frame, 0.1);
        run(frames, frames[0] + epsilon, 99.0, 0.1);
    }
}

#[test]
fn event_timeline_libgdx_upstream_tests() {
    test_frames(&[0.0]);
    test_frames(&[1.0]);
    test_frames(&[1.0, 1.0]);
    test_frames(&[1.0, 2.0]);
    test_frames(&[1.0, 2.0]);
    test_frames(&[1.0, 2.0, 3.0]);
    test_frames(&[1.0, 2.0, 3.0]);
    test_frames(&[0.0, 0.0, 0.0]);
    test_frames(&[0.0, 0.0, 1.0]);
    test_frames(&[0.0, 1.0, 1.0]);
    test_frames(&[1.0, 1.0, 1.0]);
    test_frames(&[1.0, 2.0, 3.0, 4.0]);
    test_frames(&[0.0, 2.0, 3.0, 4.0]);
    test_frames(&[0.0, 2.0, 2.0, 4.0]);
    test_frames(&[0.0, 0.0, 0.0, 0.0]);
    test_frames(&[2.0, 2.0, 2.0, 2.0]);
    test_frames(&[0.1]);
    test_frames(&[0.1, 0.1]);
    test_frames(&[0.1, 50.0]);
    test_frames(&[0.1, 0.2, 0.3, 0.4]);
    test_frames(&[
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 6.0, 7.0, 7.0, 8.0, 9.0, 10.0, 11.0, 11.01, 12.0, 12.0, 12.0,
        12.0,
    ]);
}

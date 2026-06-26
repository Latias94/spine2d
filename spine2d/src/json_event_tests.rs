use crate::SkeletonData;

#[test]
fn json_events_parse_defaults_and_key_overrides() {
    let json = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "events": {
    "e": { "int": 7, "float": 1.5, "string": "setup", "audio": "sound.ogg", "volume": 0.25, "balance": -0.5 },
    "silent": { "int": 1, "volume": 0.5, "balance": 0.9 }
  },
  "animations": {
    "a": {
      "events": [
        { "time": 0.5, "name": "e" },
        { "time": 1.0, "name": "e", "int": 9, "float": 2.0, "string": "key", "volume": 0.8, "balance": -0.2 },
        { "time": 1.5, "name": "silent", "volume": 0.2, "balance": -0.1 }
      ]
    }
  }
}
"#;

    let data = SkeletonData::from_json_str(json).expect("parse");

    let e = data.find_event("e").expect("event data e");
    let setup = e.get_setup_pose();
    assert_eq!(setup.get_int(), 7);
    assert!((setup.get_float() - 1.5).abs() < 1e-6);
    assert_eq!(setup.get_string(), "setup");
    assert_eq!(e.get_audio_path(), "sound.ogg");
    assert!((setup.get_volume() - 0.25).abs() < 1e-6);
    assert!((setup.get_balance() + 0.5).abs() < 1e-6);

    // Match spine-cpp SkeletonJson: volume/balance are only parsed when audioPath is present.
    let silent = data.find_event("silent").expect("event data silent");
    let silent_setup = silent.get_setup_pose();
    assert_eq!(silent.get_audio_path(), "");
    assert!((silent_setup.get_volume() - 0.0).abs() < 1e-6);
    assert!((silent_setup.get_balance() - 0.0).abs() < 1e-6);

    let anim = data.find_animation("a").expect("animation a");
    let timeline = anim.event_timeline.as_ref().expect("event timeline");
    let events = timeline.get_events();
    assert_eq!(events.len(), 3);

    // First key: int/float/string/volume/balance use EventData setup defaults.
    let ev0 = &events[0];
    assert!((ev0.get_time() - 0.5).abs() < 1e-6);
    assert_eq!(ev0.get_data().get_name(), "e");
    assert_eq!(ev0.get_int(), 7);
    assert!((ev0.get_float() - 1.5).abs() < 1e-6);
    assert_eq!(ev0.get_string(), "setup");
    assert_eq!(ev0.get_data().get_audio_path(), "sound.ogg");
    assert!((ev0.get_volume() - 0.25).abs() < 1e-6);
    assert!((ev0.get_balance() + 0.5).abs() < 1e-6);

    // Second key: overrides.
    let ev1 = &events[1];
    assert!((ev1.get_time() - 1.0).abs() < 1e-6);
    assert_eq!(ev1.get_int(), 9);
    assert!((ev1.get_float() - 2.0).abs() < 1e-6);
    assert_eq!(ev1.get_string(), "key");
    assert!((ev1.get_volume() - 0.8).abs() < 1e-6);
    assert!((ev1.get_balance() + 0.2).abs() < 1e-6);

    // No-audio events ignore per-key volume/balance.
    let ev2 = &events[2];
    assert_eq!(ev2.get_data().get_name(), "silent");
    assert_eq!(ev2.get_data().get_audio_path(), "");
    assert!((ev2.get_volume() - 0.0).abs() < 1e-6);
    assert!((ev2.get_balance() - 0.0).abs() < 1e-6);
}

#[test]
fn json_events_keep_file_order_for_same_time() {
    // Upstream runtimes preserve file order for events at the same time.
    // We still sort by time for timeline search, but must not reorder equal-time events.
    let json = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "events": {
    "a": { "string": "A" },
    "b": { "string": "B" }
  },
  "animations": {
    "anim": {
      "events": [
        { "time": 0.5, "name": "b" },
        { "time": 0.5, "name": "a" }
      ]
    }
  }
}
"#;

    let data = SkeletonData::from_json_str(json).expect("parse");
    let anim = data.find_animation("anim").expect("animation");
    let timeline = anim.event_timeline.as_ref().expect("event timeline");
    let events = timeline.get_events();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].get_data().get_name(), "b");
    assert_eq!(events[1].get_data().get_name(), "a");
}

#[test]
fn json_audio_event_default_volume_matches_spine_cpp() {
    let json = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "events": {
    "audio": { "audio": "sound.ogg" }
  },
  "animations": {
    "anim": {
      "events": [
        { "time": 0.25, "name": "audio" }
      ]
    }
  }
}
"#;

    let data = SkeletonData::from_json_str(json).expect("parse");
    let event_data = data.find_event("audio").expect("event data");
    assert_eq!(event_data.get_audio_path(), "sound.ogg");
    assert!((event_data.get_setup_pose().get_volume() - 1.0).abs() < 1e-6);
    assert!((event_data.get_setup_pose().get_balance() - 0.0).abs() < 1e-6);

    let event = &data
        .find_animation("anim")
        .expect("animation")
        .event_timeline
        .as_ref()
        .expect("event timeline")
        .get_events()[0];

    assert!((event.get_volume() - 1.0).abs() < 1e-6);
    assert!((event.get_balance() - 0.0).abs() < 1e-6);
}

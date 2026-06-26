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
    assert_eq!(e.int_value, 7);
    assert!((e.float_value - 1.5).abs() < 1e-6);
    assert_eq!(e.string, "setup");
    assert_eq!(e.audio_path, "sound.ogg");
    assert!((e.volume - 0.25).abs() < 1e-6);
    assert!((e.balance + 0.5).abs() < 1e-6);

    // Match spine-cpp SkeletonJson: volume/balance are only parsed when audioPath is present.
    let silent = data.find_event("silent").expect("event data silent");
    assert_eq!(silent.audio_path, "");
    assert!((silent.volume - 0.0).abs() < 1e-6);
    assert!((silent.balance - 0.0).abs() < 1e-6);

    let anim = data.find_animation("a").expect("animation a");
    let timeline = anim.event_timeline.as_ref().expect("event timeline");
    assert_eq!(timeline.events.len(), 3);

    // First key: int/float/string/volume/balance use EventData setup defaults.
    let ev0 = &timeline.events[0];
    assert!((ev0.time - 0.5).abs() < 1e-6);
    assert_eq!(ev0.name, "e");
    assert_eq!(ev0.int_value, 7);
    assert!((ev0.float_value - 1.5).abs() < 1e-6);
    assert_eq!(ev0.string, "setup");
    assert_eq!(ev0.audio_path, "sound.ogg");
    assert!((ev0.volume - 0.25).abs() < 1e-6);
    assert!((ev0.balance + 0.5).abs() < 1e-6);

    // Second key: overrides.
    let ev1 = &timeline.events[1];
    assert!((ev1.time - 1.0).abs() < 1e-6);
    assert_eq!(ev1.int_value, 9);
    assert!((ev1.float_value - 2.0).abs() < 1e-6);
    assert_eq!(ev1.string, "key");
    assert!((ev1.volume - 0.8).abs() < 1e-6);
    assert!((ev1.balance + 0.2).abs() < 1e-6);

    // No-audio events ignore per-key volume/balance.
    let ev2 = &timeline.events[2];
    assert_eq!(ev2.name, "silent");
    assert_eq!(ev2.audio_path, "");
    assert!((ev2.volume - 0.0).abs() < 1e-6);
    assert!((ev2.balance - 0.0).abs() < 1e-6);
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
    assert_eq!(timeline.events.len(), 2);
    assert_eq!(timeline.events[0].name, "b");
    assert_eq!(timeline.events[1].name, "a");
}

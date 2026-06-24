use crate::{Error, SkeletonData};

#[cfg(feature = "json")]
#[test]
fn json_rejects_non_43_spine_versions_like_cpp() {
    let err = match SkeletonData::from_json_str(r#"{ "skeleton": { "spine": "4.4.00" } }"#) {
        Ok(_) => panic!("expected version rejection"),
        Err(err) => err,
    };

    match &err {
        Error::JsonSpineVersion { value } => assert_eq!(value, "4.4.00"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[cfg(feature = "json")]
#[test]
fn json_rejects_missing_spine_versions_like_cpp() {
    let err = match SkeletonData::from_json_str(r#"{ "skeleton": { } }"#) {
        Ok(_) => panic!("expected version rejection"),
        Err(err) => err,
    };

    match &err {
        Error::JsonSpineVersion { value } => assert_eq!(value, ""),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[cfg(feature = "binary")]
#[test]
fn binary_rejects_non_43_spine_versions_like_cpp() {
    fn push_string(out: &mut Vec<u8>, s: Option<&str>) {
        match s {
            None => out.push(0),
            Some("") => out.push(1),
            Some(s) => {
                out.push((s.len() + 1) as u8);
                out.extend_from_slice(s.as_bytes());
            }
        }
    }

    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0; 8]);
    push_string(&mut bytes, Some("4.4.00"));

    let err = match SkeletonData::from_skel_bytes(&bytes) {
        Ok(_) => panic!("expected version rejection"),
        Err(err) => err,
    };

    match &err {
        Error::BinarySpineVersion { value } => assert_eq!(value, "4.4.00"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[cfg(feature = "binary")]
#[test]
fn binary_rejects_missing_spine_versions_like_cpp() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0; 8]);
    bytes.push(0);

    let err = match SkeletonData::from_skel_bytes(&bytes) {
        Ok(_) => panic!("expected version rejection"),
        Err(err) => err,
    };

    match &err {
        Error::BinarySpineVersion { value } => assert_eq!(value, ""),
        other => panic!("unexpected error: {other:?}"),
    }
}

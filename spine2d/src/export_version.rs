pub(crate) const SPINE_RUNTIME_VERSION_PREFIX: &str = "4.3";

pub(crate) fn validate_spine_version<E>(
    value: &str,
    unsupported: impl FnOnce(String) -> E,
) -> Result<(), E> {
    if value.starts_with(SPINE_RUNTIME_VERSION_PREFIX) {
        Ok(())
    } else {
        Err(unsupported(value.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::validate_spine_version;

    #[test]
    fn accepts_cpp_runtime_version_prefix() {
        assert!(validate_spine_version("4.3", String::from).is_ok());
        assert!(validate_spine_version("4.3.00", String::from).is_ok());
        assert!(validate_spine_version("4.3.8", String::from).is_ok());
    }

    #[test]
    fn rejects_missing_or_other_runtime_version_prefix() {
        assert_eq!(validate_spine_version("", String::from), Err(String::new()));
        assert_eq!(
            validate_spine_version("4.4.00", String::from),
            Err(String::from("4.4.00"))
        );
    }
}

#[test]
fn test_driver_options() {
    use crate::options::{DriverOptions, DriverOptionsArg};
    use std::collections::HashMap;
    use std::time::Duration;
    let driver_options = DriverOptionsArg::Options(HashMap::from_iter([
        (
            DriverOptions::OPT_URL.into(),
            "postgres://user:pass@host/database".into(),
        ),
        (DriverOptions::OPT_MAX_LIFETIME.into(), "1 hour".into()),
        (DriverOptions::OPT_IDLE_TIMEOUT.into(), "2 min".into()),
    ]))
    .parse()
    .unwrap();

    assert_eq!(driver_options.max_lifetime, Some(Duration::from_secs(3600)));
    assert_eq!(driver_options.idle_timeout, Some(Duration::from_secs(120)));
}

#[test]
pub fn test_strip_prefix_word_ignore_ascii_case() {
    use crate::utils::StripPrefixWordIgnoreAsciiCase;
    "NOT IN (:ph)"
        .strip_prefix_word_ignore_ascii_case(&["NOT", "IN"])
        .unwrap();
}

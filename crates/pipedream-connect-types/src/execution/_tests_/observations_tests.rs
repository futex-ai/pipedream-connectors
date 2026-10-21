//! Fractional observation timestamp conversion tests.

use serde_json::json;

use crate::{Error, Observation};

#[test]
fn converts_fractional_milliseconds_to_nanoseconds() {
    for (millis, nanos) in [
        (1.25, 1_250_000),
        (1700000000000.25, 250_000),
        (-0.25, 999_750_000),
    ] {
        let value: Observation =
            serde_json::from_value(json!({"k":"console.log","ts":millis})).unwrap();
        let date = value.timestamp().unwrap();
        assert_eq!(date.timestamp_subsec_nanos(), nanos);
    }
}

#[test]
fn rejects_nonfinite_and_out_of_range_timestamps() {
    for ts in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, f64::MAX, 1e30] {
        let value = Observation {
            ts,
            k: "console.log".into(),
            msg: None,
            err: None,
            extra: Default::default(),
        };
        assert!(matches!(value.timestamp(), Err(Error::InvalidTimestamp)));
    }
}

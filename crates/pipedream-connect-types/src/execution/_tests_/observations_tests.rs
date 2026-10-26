//! Fractional observation timestamp conversion tests.

use serde_json::json;

use crate::Observation;

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
fn rejects_out_of_range_timestamps() {
    for ts in [f64::MAX, 1e30] {
        assert!(
            serde_json::from_value::<Observation>(json!({
                "k": "console.log",
                "ts": ts,
            }))
            .is_err()
        );
    }
}

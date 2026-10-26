//! Timestamp formats, ranges, and crate-private serde adapters.

use serde_json::json;

use crate::Timestamp;

#[test]
fn timestamp_decodes_epoch_and_iso_offsets() {
    let epoch: Timestamp = serde_json::from_value(json!(1700000000000i64)).unwrap();
    assert_eq!(epoch.as_datetime().timestamp_millis(), 1_700_000_000_000);
    for text in [
        "2026-01-01T00:00:00Z",
        "2026-01-01T02:00:00+02:00",
        "2025-12-31T19:00:00-05:00",
    ] {
        let iso: Timestamp = serde_json::from_value(json!(text)).unwrap();
        assert_eq!(iso.as_datetime().timestamp(), 1_767_225_600);
        assert_eq!(
            serde_json::to_value(iso).unwrap(),
            json!("2026-01-01T00:00:00Z")
        );
    }
}

#[test]
fn timestamp_round_trips_epoch_milliseconds_as_an_integer() {
    let value: Timestamp = serde_json::from_value(json!(1_700_000_000_000_i64)).unwrap();
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!(1_700_000_000_000_i64)
    );
}

#[test]
fn invalid_dates_and_out_of_range_milliseconds_fail() {
    for value in [
        json!("bad"),
        json!("2026-02-30T00:00:00Z"),
        json!(true),
        json!(1.5),
        json!(i64::MAX),
        json!(i64::MIN),
        json!(u64::MAX),
    ] {
        assert!(serde_json::from_value::<Timestamp>(value).is_err());
    }
    assert!(Timestamp::from_millis(i64::MAX).is_err());
}

//! Timestamp formats, ranges, and crate-private serde adapters.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{Timestamp, epoch_millis};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct Millis {
    #[serde(with = "epoch_millis")]
    at: DateTime<Utc>,
}

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

#[test]
fn epoch_adapter_round_trips_and_rejects_out_of_range_values() {
    let sample = json!({"at":1700000000000i64});
    let decoded: Millis = serde_json::from_value(sample.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), sample);
    for invalid in [json!(i64::MAX), json!("bad"), json!(0.5)] {
        assert!(serde_json::from_value::<Millis>(json!({"at":invalid})).is_err());
    }
}

//! Pagination boundaries, directions, and termination tests.

use serde_json::json;

use crate::{Cursor, Error, PageInfo, PageRequest, ValidationField};

#[test]
fn limits_and_deserialization_use_the_same_validation() {
    for limit in [0, 1, 200, 201] {
        let valid = (1..=200).contains(&limit);
        assert_eq!(PageRequest::with_limit(limit).is_ok(), valid);
        assert_eq!(
            serde_json::from_value::<PageRequest>(json!({"limit":limit})).is_ok(),
            valid
        );
    }
    assert!(matches!(
        PageRequest::with_limit(0),
        Err(Error::OutOfRange {
            field: ValidationField::Limit,
            ..
        })
    ));
    assert_eq!(
        serde_json::from_value::<PageRequest>(json!({})).unwrap(),
        PageRequest::default()
    );
    assert!(serde_json::from_value::<PageRequest>(json!({"after":"a","before":"b"})).is_err());
    assert!(serde_json::from_value::<PageRequest>(json!({"after":"bad/cursor"})).is_ok());
    assert!(serde_json::from_value::<PageRequest>(json!({"after":" "})).is_err());
}

#[test]
fn next_preserves_limit_and_direction() {
    for backward in [false, true] {
        let request = PageRequest {
            limit: 10,
            after: (!backward).then(|| Cursor::new("previous").unwrap()),
            before: backward.then(|| Cursor::new("previous").unwrap()),
        };
        let page = PageInfo {
            count: Some(1),
            start_cursor: Some(Cursor::new("start").unwrap()),
            end_cursor: Some(Cursor::new("end").unwrap()),
            ..PageInfo::default()
        };
        let next = request.next(&page).unwrap().unwrap();
        assert_eq!(next.limit, 10);
        assert_eq!(
            next.after.as_ref().map(Cursor::as_str),
            (!backward).then_some("end")
        );
        assert_eq!(
            next.before.as_ref().map(Cursor::as_str),
            backward.then_some("start")
        );
        let stuck = PageInfo {
            count: Some(1),
            start_cursor: request.before.clone(),
            end_cursor: request.after.clone(),
            ..PageInfo::default()
        };
        assert!(matches!(request.next(&stuck), Err(Error::StuckCursor)));
    }
}

#[test]
fn missing_count_is_unknown_and_only_zero_or_missing_cursor_stops() {
    let request = PageRequest::default();
    for count in [None, Some(0), Some(1)] {
        let page = PageInfo {
            count,
            end_cursor: Some(Cursor::new("next").unwrap()),
            ..PageInfo::default()
        };
        assert_eq!(request.next(&page).unwrap().is_some(), count != Some(0));
        let missing = PageInfo {
            count,
            ..PageInfo::default()
        };
        assert!(request.next(&missing).unwrap().is_none());
    }
    let page: PageInfo = serde_json::from_value(json!({"end_cursor":"next"})).unwrap();
    assert_eq!(page.count, None);
    assert!(request.next(&page).unwrap().is_some());
    let conflict = PageRequest {
        after: Some(Cursor::new("a").unwrap()),
        before: Some(Cursor::new("b").unwrap()),
        ..request
    };
    assert!(matches!(
        conflict.next(&page),
        Err(Error::ConflictingCursors)
    ));
}

#[test]
fn backward_pages_stop_only_on_zero_count_or_missing_start_cursor() {
    let request = PageRequest {
        before: Some(Cursor::new("previous").unwrap()),
        ..PageRequest::default()
    };
    for count in [None, Some(0), Some(1)] {
        let page = PageInfo {
            count,
            start_cursor: Some(Cursor::new("next").unwrap()),
            ..PageInfo::default()
        };
        assert_eq!(request.next(&page).unwrap().is_some(), count != Some(0));
        assert!(
            request
                .next(&PageInfo {
                    count,
                    ..PageInfo::default()
                })
                .unwrap()
                .is_none()
        );
    }
}

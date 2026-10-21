//! Tests for input sources and configured props decoding.

use std::io;
use std::path::PathBuf;

use pipedream_connect_types::{ConfiguredPropValue, PropName};
use unimock::{MockFn, Unimock, matching};

use crate::argument::Argument;
use crate::error::{Error, JsonProblem};
use crate::inputs::{InputSource, ReadInputMock, read_props};
use crate::report::failure::describe;

#[test]
fn dash_selects_standard_input_and_other_values_select_files() {
    assert_eq!(InputSource::from(PathBuf::from("-")), InputSource::Stdin);
    assert_eq!(
        InputSource::from(PathBuf::from("./-")),
        InputSource::File(PathBuf::from("./-"))
    );
    assert_eq!(
        InputSource::from(PathBuf::from("props.json")),
        InputSource::File(PathBuf::from("props.json"))
    );
}

#[test]
fn props_decode_from_the_selected_source() {
    let inputs = Unimock::new(
        ReadInputMock
            .next_call(
                matching!((source) if **source == InputSource::File(PathBuf::from("props.json"))),
            )
            .returns(Ok(br#"{"calendarId":"primary","maxResults":5}"#.to_vec())),
    );
    let props = read_props(
        &inputs,
        &InputSource::File(PathBuf::from("props.json")),
        Argument::Props,
    )
    .unwrap();
    assert_eq!(
        props.get(&PropName::new("calendarId").unwrap()),
        Some(&ConfiguredPropValue::String("primary".to_owned()))
    );
    assert_eq!(
        props.get(&PropName::new("maxResults").unwrap()),
        Some(&ConfiguredPropValue::Number(5.into()))
    );
}

#[test]
fn unreadable_input_keeps_the_failure_kind() {
    let inputs = Unimock::new(
        ReadInputMock
            .next_call(matching!((source) if **source == InputSource::Stdin))
            .answers(&|_, _| Err(io::Error::from(io::ErrorKind::PermissionDenied))),
    );
    let error = read_props(&inputs, &InputSource::Stdin, Argument::Props).unwrap_err();
    assert!(matches!(
        &error,
        Error::UnreadableInput { argument: Argument::Props, cause }
            if cause.kind() == io::ErrorKind::PermissionDenied
    ));
}

#[test]
fn invalid_json_keeps_only_the_failure_position() {
    let inputs = Unimock::new(
        ReadInputMock
            .next_call(matching!(_))
            .returns(Ok(br#"{"apiKey": "sk_live_hidden", }"#.to_vec())),
    );
    let error = read_props(&inputs, &InputSource::Stdin, Argument::Props).unwrap_err();
    assert!(matches!(
        error,
        Error::InvalidJson {
            argument: Argument::Props,
            line: 1,
            column: 30,
            problem: JsonProblem::Syntax,
        }
    ));
    assert!(!describe(&error).message.contains("sk_live_hidden"));
    assert!(!format!("{error:?}").contains("sk_live_hidden"));
}

#[test]
fn props_that_are_not_an_object_of_prop_names_are_a_shape_problem() {
    for (body, column) in [
        (&br#"["primary"]"#[..], 1),
        (&br#"{"bad/name": "sk_live_hidden"}"#[..], 11),
    ] {
        let inputs = Unimock::new(
            ReadInputMock
                .next_call(matching!(_))
                .returns(Ok(body.to_vec())),
        );
        let error = read_props(&inputs, &InputSource::Stdin, Argument::Props).unwrap_err();
        assert!(
            matches!(
                error,
                Error::InvalidJson {
                    problem: JsonProblem::Shape,
                    line: 1,
                    column: found,
                    ..
                } if found == column
            ),
            "{error:?}"
        );
        assert!(!describe(&error).message.contains("sk_live_hidden"));
    }
}

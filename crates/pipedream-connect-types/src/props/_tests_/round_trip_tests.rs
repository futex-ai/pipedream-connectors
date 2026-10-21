//! Table tests for every custom serde variant and unknown object fields.

use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

use crate::{
    ConfigurableProp, ConfiguredPropValue, Emitter, PropKind, PropOption, PropOptionValue,
    TimerValue,
};

type RoundTrip = fn(Value);

fn round_trip<T: DeserializeOwned + Serialize>(sample: Value) {
    let decoded: T = serde_json::from_value(sample.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), sample);
}

#[test]
fn custom_serde_variants_round_trip_with_unknown_object_keys() {
    let mut cases: Vec<(&str, RoundTrip, Vec<Value>)> = vec![
        (
            "PropOptionValue",
            round_trip::<PropOptionValue>,
            vec![
                json!("C1"),
                json!(1),
                json!(true),
                Value::Null,
                json!({"future":1}),
                json!(1.25),
                json!(u64::MAX),
            ],
        ),
        (
            "PropOption",
            round_trip::<PropOption>,
            vec![
                json!("C1"),
                json!({"label":"Channel","value":"C1","future":1}),
                json!({"__lv":{"label":"Channel","value":"C1","future":1},"future":2}),
                json!({"label":{"en":"Channel"},"value":"C1","future":1}),
            ],
        ),
        (
            "TimerValue",
            round_trip::<TimerValue>,
            vec![
                json!({"intervalSeconds":60,"future":1}),
                json!({"cron":"* * * * *","future":1}),
                json!({"schedule":[],"future":1}),
            ],
        ),
        (
            "ConfiguredPropValue",
            round_trip::<ConfiguredPropValue>,
            vec![
                json!({"authProvisionId":"apn_sample","future":1}),
                json!(true),
                json!(9007199254740993u64),
                json!(u64::MAX),
                json!({"value":"select 1","query":"select ?","params":["1"],"usePreparedStatements":true,"future":1}),
                json!("C1"),
                json!(["C1", "C2"]),
                json!({"object":{},"future":1}),
                Value::Null,
                json!([1, false]),
            ],
        ),
        (
            "ConfigurableProp",
            round_trip::<ConfigurableProp>,
            vec![
                json!({"name":"channel","type":"string","label":"Channel","useQuery":true,"default":"C1","options":[],"future":1}),
                json!({"name":"future","type":"new_kind","label":{},"optional":"yes","app":[],"default":{},"options":42,"future":1}),
            ],
        ),
    ];
    cases.push(("Emitter",round_trip::<Emitter>,vec![
        json!({"type":"DeployedComponent","id":"dc_sample","owner_id":"sample","component_id":"sample","component_key":null,"configurable_props":[],"configured_props":{},"active":true,"created_at":0,"updated_at":0,"name":"Sample","name_slug":"sample","callback_observations":null,"emit_on_deploy":null,"webhook_signing_key":null,"webhook_urls":null,"webhooks":null,"future":1}),
        json!({"type":"HttpInterface","id":"hi_sample","key":"sample","endpoint_url":"https://example.com","custom_response":false,"created_at":0,"updated_at":0,"future":1}),
        json!({"type":"TimerInterface","id":"ti_sample","interval_seconds":60.0,"cron":null,"timezone":"UTC","schedule_changed_at":0,"created_at":0,"updated_at":0,"future":1}),
        json!({"type":"FutureEmitter","future":1}),
    ]));
    for (name, check, samples) in cases {
        for sample in samples {
            let result = std::panic::catch_unwind(|| check(sample.clone()));
            assert!(result.is_ok(), "{name}: {sample}");
        }
    }
}

#[test]
fn known_sql_and_app_values_are_typed() {
    let sql: ConfiguredPropValue = serde_json::from_value(
        json!({"value":"x","query":"?","params":["x"],"usePreparedStatements":true,"future":1}),
    )
    .unwrap();
    assert!(matches!(
        sql,
        ConfiguredPropValue::Sql {
            use_prepared_statements: true,
            ..
        }
    ));
    let app: ConfiguredPropValue =
        serde_json::from_value(json!({"authProvisionId":"apn_sample","future":1})).unwrap();
    assert!(matches!(app, ConfiguredPropValue::App { .. }));
}

#[test]
fn unknown_props_keep_failed_fields_and_use_best_effort_typed_fields() {
    let sample = json!({"name":"future","type":"new_kind","label":{},"description":"kept","optional":"yes","disabled":[],"hidden":{},"readOnly":0,"remoteOptions":"yes","reloadProps":[],"withLabel":{},"app":[],"default":"x","options":42,"useQuery":true});
    let prop: ConfigurableProp = serde_json::from_value(sample.clone()).unwrap();
    assert!(matches!(prop.kind, PropKind::Other { .. }));
    assert_eq!(prop.description.as_deref(), Some("kept"));
    assert_eq!(prop.use_query, Some(true));
    for key in [
        "label",
        "optional",
        "disabled",
        "hidden",
        "readOnly",
        "remoteOptions",
        "reloadProps",
        "withLabel",
        "app",
        "options",
    ] {
        assert_eq!(prop.extra.get(key), sample.get(key));
    }
    assert_eq!(serde_json::to_value(prop).unwrap(), sample);
}

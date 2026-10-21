//! Schema samples that cover unions, maps, nullable fields, and omissions.

use serde_json::{Map, Value, json};

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Mode {
    Full,
    Minimal,
    Nullable,
}

pub fn resolve(schema: &Value, schemas: &Map<String, Value>) -> Value {
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        return resolve(&schemas[reference.rsplit('/').next().unwrap()], schemas);
    }
    if let Some(parts) = schema.get("allOf").and_then(Value::as_array) {
        let mut merged = json!({"type":"object","properties":{},"required":[]});
        for part in parts {
            let resolved = resolve(part, schemas);
            if let Some(properties) = resolved["properties"].as_object() {
                merged["properties"]
                    .as_object_mut()
                    .unwrap()
                    .extend(properties.clone());
            }
            if let Some(required) = resolved["required"].as_array() {
                merged["required"]
                    .as_array_mut()
                    .unwrap()
                    .extend(required.clone());
            }
        }
        if schema["nullable"] == true {
            merged["nullable"] = json!(true);
        }
        return merged;
    }
    schema.clone()
}

pub fn samples(
    schema: &Value,
    schemas: &Map<String, Value>,
    mode: Mode,
    extensions: bool,
) -> Vec<Value> {
    let schema = resolve(schema, schemas);
    if mode == Mode::Nullable && schema["nullable"] == true {
        return vec![Value::Null];
    }
    for union in ["oneOf", "anyOf"] {
        if let Some(branches) = schema[union].as_array() {
            return branches
                .iter()
                .flat_map(|branch| samples(branch, schemas, mode, extensions))
                .collect();
        }
    }
    if let Some(values) = schema["enum"].as_array() {
        return values.clone();
    }
    match schema["type"].as_str() {
        Some("object") => object_samples(&schema, schemas, mode, extensions),
        Some("array") => vec![Value::Array(samples(
            &schema["items"],
            schemas,
            mode,
            extensions,
        ))],
        Some("integer") => vec![schema.get("example").cloned().unwrap_or(json!(1))],
        Some("number") => vec![json!(1.25)],
        Some("boolean") => vec![json!(true), json!(false)],
        Some("string") => vec![json!(sample_string(&schema))],
        _ => vec![json!({"arbitrary":"kept","number":9007199254740993u64})],
    }
}

fn object_samples(
    schema: &Value,
    schemas: &Map<String, Value>,
    mode: Mode,
    extensions: bool,
) -> Vec<Value> {
    let mut object = Map::new();
    let mut variants = Vec::new();
    if let Some(properties) = schema["properties"].as_object() {
        for (key, property) in properties {
            let required = schema["required"]
                .as_array()
                .is_some_and(|fields| fields.contains(&json!(key)));
            if mode == Mode::Minimal && !required {
                continue;
            }
            let mut values = samples(property, schemas, mode, extensions);
            for value in &mut values {
                semantic_identifier(schema, key, value);
            }
            object.insert(key.clone(), values[0].clone());
            variants.push((key.clone(), values));
        }
    }
    if let Some(additional) = schema.get("additionalProperties") {
        if additional == &Value::Bool(true) {
            object.insert("custom_field".into(), json!({"nested":42}));
        } else if additional.is_object() {
            for (index, value) in samples(additional, schemas, mode, extensions)
                .into_iter()
                .enumerate()
            {
                object.insert(format!("prop_{index}"), value);
            }
        }
    }
    if extensions {
        object.insert("__future".into(), json!({"kept":true}));
    }
    let mut result = vec![Value::Object(object.clone())];
    for (key, values) in variants {
        for value in values.into_iter().skip(1) {
            let mut variant = object.clone();
            variant.insert(key.clone(), value);
            result.push(Value::Object(variant));
        }
    }
    result
}

fn semantic_identifier(schema: &Value, key: &str, value: &mut Value) {
    if !value.is_string() {
        return;
    }
    let replacement = match key {
        "project_id" => Some("proj_sample"),
        "oauth_app_id" => Some("oa_sample"),
        "account_id" => Some("apn_sample"),
        "connect_token" => Some("ctok_0123456789abcdef0123456789abcdef"),
        "id" if schema["properties"]["id"]["description"] == "ID of the app category" => {
            Some("appcat_sample")
        }
        _ => None,
    };
    if let Some(replacement) = replacement {
        *value = json!(replacement);
    }
}

fn sample_string(schema: &Value) -> String {
    if let Some(pattern) = schema["pattern"].as_str() {
        for (needle, value) in [
            ("proj_", "proj_sample"),
            ("apn_", "apn_sample"),
            ("dc_", "dc_sample"),
            ("oa_", "oa_sample"),
            ("appcat_", "appcat_sample"),
            ("ctok_", "ctok_0123456789abcdef0123456789abcdef"),
            ("p_", "p_sample"),
        ] {
            if pattern.contains(needle) {
                return value.into();
            }
        }
    }
    match schema["format"].as_str() {
        Some("date-time") => "2026-01-01T00:00:00Z".into(),
        Some("uri" | "uri-reference") => "https://example.com/hook".into(),
        Some("email") => "user@example.com".into(),
        _ => schema
            .get("example")
            .and_then(Value::as_str)
            .unwrap_or("sample")
            .into(),
    }
}

pub fn assert_preserved(sample: &Value, encoded: &Value, path: &str) {
    match sample {
        Value::Object(fields) => {
            let encoded = encoded
                .as_object()
                .unwrap_or_else(|| panic!("{path}: expected object, got {encoded}"));
            for (key, value) in fields {
                assert_preserved(
                    value,
                    encoded
                        .get(key)
                        .unwrap_or_else(|| panic!("{path}.{key}: lost field")),
                    &format!("{path}.{key}"),
                );
            }
            for (key, value) in encoded {
                assert!(
                    fields.contains_key(key),
                    "{path}.{key}: fabricated value {value}"
                );
            }
        }
        Value::Array(values) => {
            let encoded = encoded.as_array().unwrap();
            assert_eq!(values.len(), encoded.len(), "{path}: array length");
            for (index, (expected, actual)) in values.iter().zip(encoded).enumerate() {
                assert_preserved(expected, actual, &format!("{path}[{index}]"));
            }
        }
        Value::Number(number) if number.is_f64() => {
            assert_eq!(number.as_f64(), encoded.as_f64(), "{path}")
        }
        _ => assert_eq!(sample, encoded, "{path}"),
    }
}

/// Builds the explicit null defaults permitted by each response schema.
pub fn expected_response(sample: &Value, schema: &Value, schemas: &Map<String, Value>) -> Value {
    if schema["$ref"] == "#/components/schemas/ConfigurableProp" {
        return sample.clone();
    }
    let schema = resolve(schema, schemas);
    if let Some(field) = schema["discriminator"]["propertyName"].as_str()
        && let Some(kind) = sample[field].as_str()
        && let Some(reference) = schema["discriminator"]["mapping"][kind].as_str()
    {
        return expected_response(sample, &json!({"$ref":reference}), schemas);
    }
    match sample {
        Value::Object(fields) => {
            let mut expected = fields.clone();
            if let Some(properties) = schema["properties"].as_object() {
                for (key, property) in properties {
                    if let Some(value) = fields.get(key) {
                        expected.insert(key.clone(), expected_response(value, property, schemas));
                    } else if resolve(property, schemas)["nullable"] == true {
                        expected.insert(key.clone(), Value::Null);
                    }
                }
            }
            Value::Object(expected)
        }
        Value::Array(values) => Value::Array(
            values
                .iter()
                .map(|value| expected_response(value, &schema["items"], schemas))
                .collect(),
        ),
        _ => sample.clone(),
    }
}

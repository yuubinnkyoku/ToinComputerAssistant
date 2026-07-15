use serde_json::{Map, Value};

const UNSUPPORTED_SCHEMA_KEYS: &[&str] = &[
    "$id",
    "$schema",
    "$defs",
    "additionalProperties",
    "dependentSchemas",
    "else",
    "if",
    "not",
    "nullable",
    "patternProperties",
    "then",
    "unevaluatedProperties",
];

/// Gemini function declarations accept only a selected OpenAPI schema subset.
/// Keep OpenAI-flavored tool schemas unchanged and normalize at the Gemini boundary.
pub fn to_gemini_schema(mut schema: Value) -> Value {
    normalize_schema(&mut schema);
    schema
}

fn normalize_schema(schema: &mut Value) {
    match schema {
        Value::Object(obj) => normalize_object(obj),
        Value::Array(items) => {
            for item in items {
                normalize_schema(item);
            }
        }
        _ => {}
    }
}

fn normalize_object(obj: &mut Map<String, Value>) {
    collapse_schema_combiners(obj);
    normalize_type(obj);

    for key in UNSUPPORTED_SCHEMA_KEYS {
        obj.remove(*key);
    }

    for value in obj.values_mut() {
        normalize_schema(value);
    }
}

fn normalize_type(obj: &mut Map<String, Value>) {
    let Some(type_value) = obj.get_mut("type") else {
        return;
    };

    match type_value {
        Value::Array(types) => {
            let selected = types
                .iter()
                .filter_map(Value::as_str)
                .find(|schema_type| *schema_type != "null")
                .unwrap_or("string");
            *type_value = Value::String(selected.to_string());
        }
        Value::String(schema_type) if schema_type == "null" => {
            *type_value = Value::String("string".to_string());
        }
        _ => {}
    }
}

fn collapse_schema_combiners(obj: &mut Map<String, Value>) {
    for key in ["oneOf", "anyOf"] {
        let Some(Value::Array(options)) = obj.remove(key) else {
            continue;
        };

        if let Some(mut selected) = select_schema_variant(options) {
            normalize_schema(&mut selected);
            merge_schema_object(obj, selected);
        }
    }

    let Some(Value::Array(options)) = obj.remove("allOf") else {
        return;
    };

    let mut merged = Map::new();
    for mut option in options {
        normalize_schema(&mut option);
        if let Value::Object(option_obj) = option {
            for (key, value) in option_obj {
                merged.entry(key).or_insert(value);
            }
        }
    }

    merge_schema_object(obj, Value::Object(merged));
}

fn select_schema_variant(options: Vec<Value>) -> Option<Value> {
    options
        .into_iter()
        .find(|option| !is_null_schema(option))
}

fn is_null_schema(schema: &Value) -> bool {
    let Value::Object(obj) = schema else {
        return false;
    };

    match obj.get("type") {
        Some(Value::String(schema_type)) => schema_type == "null",
        Some(Value::Array(types)) => types
            .iter()
            .filter_map(Value::as_str)
            .all(|schema_type| schema_type == "null"),
        _ => false,
    }
}

fn merge_schema_object(obj: &mut Map<String, Value>, schema: Value) {
    let Value::Object(schema_obj) = schema else {
        return;
    };

    for (key, value) in schema_obj {
        obj.entry(key).or_insert(value);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::to_gemini_schema;

    #[test]
    fn collapses_json_schema_type_arrays() {
        let normalized = to_gemini_schema(json!({
            "type": "object",
            "properties": {
                "speaker": {
                    "type": ["integer", "string", "null"]
                },
                "voice_pan": {
                    "type": ["number", "string", "null"]
                }
            }
        }));

        assert_eq!(normalized["properties"]["speaker"]["type"], "integer");
        assert_eq!(normalized["properties"]["voice_pan"]["type"], "number");
    }

    #[test]
    fn removes_keywords_not_supported_by_gemini_function_declarations() {
        let normalized = to_gemini_schema(json!({
            "type": "object",
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "additionalProperties": false,
            "properties": {
                "value": {
                    "nullable": true,
                    "oneOf": [
                        { "type": "null" },
                        { "type": "string" }
                    ]
                }
            }
        }));

        assert!(normalized.get("$schema").is_none());
        assert!(normalized.get("additionalProperties").is_none());
        assert!(normalized["properties"]["value"].get("nullable").is_none());
        assert_eq!(normalized["properties"]["value"]["type"], "string");
    }
}

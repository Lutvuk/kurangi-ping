use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn fixture_text(relative: &str) -> String {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = base
        .join("tests")
        .join("fixtures")
        .join("contracts")
        .join(relative);
    fs::read_to_string(path).expect("fixture should be readable")
}

fn fixture_json(relative: &str) -> Value {
    serde_json::from_str(&fixture_text(relative)).expect("fixture should be valid json")
}

fn looks_like_rfc3339_utc(value: &str) -> bool {
    value.len() >= 20 && value.contains('T') && value.ends_with('Z')
}

fn validate_relay_manifest_schema(value: &Value) -> Result<(), String> {
    let object = value
        .as_object()
        .ok_or_else(|| "manifest must be an object".to_string())?;

    let version = object
        .get("version")
        .and_then(Value::as_str)
        .ok_or_else(|| "version must be string".to_string())?;
    if version.is_empty() {
        return Err("version must not be empty".to_string());
    }

    let valid_until = object
        .get("valid_until")
        .and_then(Value::as_str)
        .ok_or_else(|| "valid_until must be string".to_string())?;
    if !looks_like_rfc3339_utc(valid_until) {
        return Err("valid_until must be RFC3339 UTC-like string".to_string());
    }

    let signature = object
        .get("signature")
        .and_then(Value::as_str)
        .ok_or_else(|| "signature must be string".to_string())?;
    if signature.is_empty() {
        return Err("signature must not be empty".to_string());
    }

    let relays = object
        .get("relays")
        .and_then(Value::as_array)
        .ok_or_else(|| "relays must be array".to_string())?;
    if relays.is_empty() {
        return Err("relays must not be empty".to_string());
    }

    for relay in relays {
        let relay_object = relay
            .as_object()
            .ok_or_else(|| "relay item must be object".to_string())?;

        relay_object
            .get("relay_id")
            .and_then(Value::as_str)
            .ok_or_else(|| "relay_id must be string".to_string())?;
        relay_object
            .get("region")
            .and_then(Value::as_str)
            .ok_or_else(|| "region must be string".to_string())?;
        relay_object
            .get("hostname")
            .and_then(Value::as_str)
            .ok_or_else(|| "hostname must be string".to_string())?;
        let priority = relay_object
            .get("priority")
            .and_then(Value::as_i64)
            .ok_or_else(|| "priority must be integer".to_string())?;
        if priority < 1 {
            return Err("priority must be >= 1".to_string());
        }
    }

    Ok(())
}

fn validate_relay_health_schema(value: &Value) -> Result<(), String> {
    let object = value
        .as_object()
        .ok_or_else(|| "health response must be object".to_string())?;
    let data = object
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| "data must be array".to_string())?;
    let page = object
        .get("page")
        .and_then(Value::as_object)
        .ok_or_else(|| "page must be object".to_string())?;

    for item in data {
        let item_object = item
            .as_object()
            .ok_or_else(|| "health item must be object".to_string())?;

        item_object
            .get("relay_id")
            .and_then(Value::as_str)
            .ok_or_else(|| "relay_id must be string".to_string())?;

        let status = item_object
            .get("status")
            .and_then(Value::as_str)
            .ok_or_else(|| "status must be string".to_string())?;
        if !matches!(status, "ok" | "warn" | "dead") {
            return Err("status must be one of ok,warn,dead".to_string());
        }

        let latency_ms = item_object
            .get("latency_ms")
            .and_then(Value::as_i64)
            .ok_or_else(|| "latency_ms must be integer".to_string())?;
        if latency_ms < 0 {
            return Err("latency_ms must be >= 0".to_string());
        }

        let updated_at = item_object
            .get("updated_at")
            .and_then(Value::as_str)
            .ok_or_else(|| "updated_at must be string".to_string())?;
        if !looks_like_rfc3339_utc(updated_at) {
            return Err("updated_at must be RFC3339 UTC-like string".to_string());
        }
    }

    let limit = page
        .get("limit")
        .and_then(Value::as_i64)
        .ok_or_else(|| "page.limit must be integer".to_string())?;
    if !(1..=200).contains(&limit) {
        return Err("page.limit must be between 1 and 200".to_string());
    }

    if let Some(next_cursor) = page.get("next_cursor") {
        if !(next_cursor.is_null() || next_cursor.is_string()) {
            return Err("page.next_cursor must be null or string".to_string());
        }
    }

    Ok(())
}

#[test]
fn relay_manifest_input_parsing_matches_openapi_expectations() {
    let valid = fixture_json("relay_manifest_valid.json");
    validate_relay_manifest_schema(&valid).expect("valid manifest fixture should conform");

    let invalid = fixture_json("relay_manifest_invalid.json");
    let err = validate_relay_manifest_schema(&invalid).expect_err("invalid fixture should fail");
    assert!(
        err.contains("priority"),
        "expected priority-related schema failure, got: {err}"
    );
}

#[test]
fn relay_health_input_parsing_matches_openapi_expectations() {
    let valid = fixture_json("relay_health_valid.json");
    validate_relay_health_schema(&valid).expect("valid health fixture should conform");

    let invalid = fixture_json("relay_health_invalid.json");
    let err = validate_relay_health_schema(&invalid).expect_err("invalid fixture should fail");
    assert!(
        err.contains("status") || err.contains("latency_ms"),
        "expected status/latency schema failure, got: {err}"
    );
}

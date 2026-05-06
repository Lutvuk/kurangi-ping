use client_engine::telemetry::delivery_client::{
    TelemetryDeliveryClient, TelemetryDeliveryEvent, TelemetryDeliveryOutcome,
    TelemetryHttpRequest, TelemetryHttpResponse, TelemetryHttpTransport, TelemetryTransportError,
    TELEMETRY_INGEST_PATH,
};
use client_engine::telemetry::scrubber::SensitiveFieldPolicy;
use client_engine::telemetry::validator::{TelemetryValidationErrorCode, UnknownKeyPolicy};
use client_engine::telemetry::{TelemetryEvent, TelemetryPayload, TelemetryService, TelemetryValue};
use serde_json::Value;
use std::collections::BTreeMap;
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

fn parse_fixture_to_delivery_batch(
    fixture: &Value,
) -> Result<(String, Vec<TelemetryDeliveryEvent>), String> {
    let object = fixture
        .as_object()
        .ok_or_else(|| "batch must be object".to_string())?;

    let installation_id_hash = object
        .get("installation_id_hash")
        .and_then(Value::as_str)
        .ok_or_else(|| "installation_id_hash must be string".to_string())?;
    if installation_id_hash.len() < 16 || installation_id_hash.len() > 256 {
        return Err("installation_id_hash length must be 16..=256".to_string());
    }

    let events = object
        .get("events")
        .and_then(Value::as_array)
        .ok_or_else(|| "events must be array".to_string())?;
    if events.is_empty() || events.len() > 200 {
        return Err("events size must be 1..=200".to_string());
    }

    let mut delivery_events = Vec::with_capacity(events.len());
    for item in events {
        let item_object = item
            .as_object()
            .ok_or_else(|| "event item must be object".to_string())?;

        let event_name = item_object
            .get("event_name")
            .and_then(Value::as_str)
            .ok_or_else(|| "event_name must be string".to_string())?;
        let occurred_at = item_object
            .get("occurred_at")
            .and_then(Value::as_str)
            .ok_or_else(|| "occurred_at must be string".to_string())?;
        if !looks_like_rfc3339_utc(occurred_at) {
            return Err("occurred_at must be RFC3339 UTC-like string".to_string());
        }

        let session_id = match item_object.get("session_id") {
            None | Some(Value::Null) => None,
            Some(Value::String(value)) => Some(value.clone()),
            Some(_) => return Err("session_id must be string or null".to_string()),
        };

        let payload = item_object
            .get("payload")
            .and_then(Value::as_object)
            .ok_or_else(|| "payload must be object".to_string())?;

        let telemetry_payload = payload_to_telemetry_payload(payload)
            .map_err(|err| format!("payload conversion failed: {err}"))?;
        delivery_events.push(TelemetryDeliveryEvent {
            event: TelemetryEvent::new(event_name, telemetry_payload),
            occurred_at: occurred_at.to_string(),
            session_id,
        });
    }

    Ok((installation_id_hash.to_string(), delivery_events))
}

fn payload_to_telemetry_payload(
    payload: &serde_json::Map<String, Value>,
) -> Result<TelemetryPayload, String> {
    let mut mapped = BTreeMap::new();
    for (key, value) in payload {
        let telemetry_value = match value {
            Value::String(text) => TelemetryValue::Text(text.clone()),
            Value::Number(number) => {
                if let Some(integer) = number.as_i64() {
                    TelemetryValue::Integer(integer)
                } else if let Some(float) = number.as_f64() {
                    TelemetryValue::Float(float)
                } else {
                    return Err(format!("number field '{key}' is unsupported"));
                }
            }
            _ => {
                return Err(format!(
                    "field '{key}' must be string/number for v1 telemetry payload"
                ))
            }
        };
        mapped.insert(key.clone(), telemetry_value);
    }
    Ok(mapped)
}

fn assert_pipeline_contract_passes(events: &[TelemetryDeliveryEvent]) {
    let mut service = TelemetryService::with_sensitive_field_policy(SensitiveFieldPolicy::reject());
    for event in events {
        service
            .enqueue_validated(
                event.event.name.clone(),
                event.event.payload.clone(),
                UnknownKeyPolicy::Reject,
            )
            .expect("event should pass allowlist + privacy checks");
    }
}

#[derive(Debug, Clone, Default)]
struct ContractAwareTransport {
    requests: Vec<TelemetryHttpRequest>,
}

impl TelemetryHttpTransport for ContractAwareTransport {
    fn post_events_batch(
        &mut self,
        request: &TelemetryHttpRequest,
    ) -> Result<TelemetryHttpResponse, TelemetryTransportError> {
        self.requests.push(request.clone());

        assert_eq!(request.path, TELEMETRY_INGEST_PATH);
        assert!(
            request.idempotency_key.len() >= 8,
            "idempotency key must be present"
        );

        let json: Value = serde_json::from_str(&request.body_json)
            .expect("delivery request body should be valid json");
        let parsed = parse_fixture_to_delivery_batch(&json)
            .expect("delivery request body should conform to batch contract");
        let accepted = u32::try_from(parsed.1.len()).expect("accepted count must fit into u32");

        Ok(TelemetryHttpResponse {
            status_code: 202,
            body_json: format!(r#"{{"accepted":{accepted},"rejected":0,"request_id":"req-ci"}}"#),
            retry_after_seconds: None,
        })
    }
}

#[test]
fn valid_payload_batches_pass_integration_checks() {
    let fixture = fixture_json("telemetry_batch_valid.json");
    let (installation_id_hash, events) =
        parse_fixture_to_delivery_batch(&fixture).expect("valid fixture should parse");

    assert_pipeline_contract_passes(&events);

    let transport = ContractAwareTransport::default();
    let mut client = TelemetryDeliveryClient::new(transport);
    let outcome = client
        .send_batch(&installation_id_hash, &events)
        .expect("valid delivery should succeed");
    match outcome {
        TelemetryDeliveryOutcome::Accepted {
            accepted, rejected, ..
        } => {
            assert_eq!(accepted, 2);
            assert_eq!(rejected, 0);
        }
        _ => panic!("expected accepted outcome"),
    }
}

#[test]
fn invalid_payloads_fail_with_expected_policy_outcomes() {
    let schema_invalid = fixture_json("telemetry_batch_invalid_schema.json");
    let schema_error = parse_fixture_to_delivery_batch(&schema_invalid)
        .expect_err("schema-invalid fixture should fail envelope validation");
    assert!(schema_error.contains("events must be array"));

    let unknown_key = fixture_json("telemetry_batch_invalid_policy_unknown_key.json");
    let (_, unknown_events) =
        parse_fixture_to_delivery_batch(&unknown_key).expect("unknown-key fixture should parse");
    let mut service = TelemetryService::with_sensitive_field_policy(SensitiveFieldPolicy::reject());
    let unknown_error = service
        .enqueue_validated(
            unknown_events[0].event.name.clone(),
            unknown_events[0].event.payload.clone(),
            UnknownKeyPolicy::Reject,
        )
        .expect_err("unknown payload key should fail");
    assert_eq!(
        unknown_error.code,
        TelemetryValidationErrorCode::UnknownPayloadKey
    );

    let sensitive = fixture_json("telemetry_batch_invalid_policy_sensitive.json");
    let (_, sensitive_events) =
        parse_fixture_to_delivery_batch(&sensitive).expect("sensitive fixture should parse");
    let sensitive_error = service
        .enqueue_validated(
            sensitive_events[0].event.name.clone(),
            sensitive_events[0].event.payload.clone(),
            UnknownKeyPolicy::Reject,
        )
        .expect_err("sensitive field should fail under reject policy");
    assert_eq!(
        sensitive_error.code,
        TelemetryValidationErrorCode::SensitiveFieldViolation
    );
}

#[test]
fn batch_size_and_schema_constraints_are_enforced() {
    let valid = fixture_json("telemetry_batch_valid.json");
    let (installation_id_hash, events) =
        parse_fixture_to_delivery_batch(&valid).expect("valid fixture should parse");
    assert!(!installation_id_hash.is_empty());
    assert!(!events.is_empty());

    let mut oversized = valid;
    let seed_event = oversized
        .get("events")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .cloned()
        .expect("valid fixture should contain at least one event");
    let events_array = (0..201).map(|_| seed_event.clone()).collect::<Vec<_>>();
    oversized["events"] = Value::Array(events_array);

    let err = parse_fixture_to_delivery_batch(&oversized)
        .expect_err("batch size above 200 should fail contract check");
    assert!(err.contains("events size must be 1..=200"));
}

#[test]
fn conformance_suite_is_a_ci_release_gate() {
    let fixture = fixture_json("telemetry_batch_valid.json");
    let (_, events) = parse_fixture_to_delivery_batch(&fixture).expect("fixture should parse");
    assert!(
        !events.is_empty(),
        "conformance suite must exercise at least one contract payload"
    );
}

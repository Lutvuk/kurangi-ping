use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};

use super::{TelemetryEvent, TelemetryValue};

pub const TELEMETRY_INGEST_PATH: &str = "/v1/telemetry/events:batch";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelemetryDeliveryEvent {
    pub event: TelemetryEvent,
    pub occurred_at: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelemetryHttpRequest {
    pub path: String,
    pub idempotency_key: String,
    pub body_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelemetryHttpResponse {
    pub status_code: u16,
    pub body_json: String,
    pub retry_after_seconds: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TelemetryTransportErrorCode {
    Timeout,
    ConnectionFailed,
    DnsResolutionFailed,
    Unknown,
}

impl TelemetryTransportErrorCode {
    pub fn as_code(self) -> &'static str {
        match self {
            Self::Timeout => "transport_timeout",
            Self::ConnectionFailed => "transport_connection_failed",
            Self::DnsResolutionFailed => "transport_dns_resolution_failed",
            Self::Unknown => "transport_unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelemetryTransportError {
    pub code: TelemetryTransportErrorCode,
    pub message: String,
}

pub trait TelemetryHttpTransport {
    fn post_events_batch(
        &mut self,
        request: &TelemetryHttpRequest,
    ) -> Result<TelemetryHttpResponse, TelemetryTransportError>;
}

#[derive(Debug, Clone)]
pub struct TelemetryDeliveryClient<T: TelemetryHttpTransport> {
    transport: T,
    diagnostics: Vec<TelemetryDeliveryDiagnostic>,
}

impl<T: TelemetryHttpTransport> TelemetryDeliveryClient<T> {
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            diagnostics: Vec::new(),
        }
    }

    pub fn diagnostics(&self) -> &[TelemetryDeliveryDiagnostic] {
        &self.diagnostics
    }

    pub fn take_diagnostics(&mut self) -> Vec<TelemetryDeliveryDiagnostic> {
        std::mem::take(&mut self.diagnostics)
    }

    pub fn send_batch(
        &mut self,
        installation_id_hash: &str,
        events: &[TelemetryDeliveryEvent],
    ) -> Result<TelemetryDeliveryOutcome, TelemetryDeliveryError> {
        if events.is_empty() {
            return Err(TelemetryDeliveryError::EmptyBatch);
        }

        let request_body = build_request_body(installation_id_hash, events)?;
        let idempotency_key = build_idempotency_key(&request_body);
        let body_json = serde_json::to_string(&request_body).map_err(|_| {
            TelemetryDeliveryError::SerializationFailed {
                reason_code: "request_json_serialize_failed".to_string(),
            }
        })?;
        let request = TelemetryHttpRequest {
            path: TELEMETRY_INGEST_PATH.to_string(),
            idempotency_key: idempotency_key.clone(),
            body_json,
        };

        let response = self
            .transport
            .post_events_batch(&request)
            .map_err(TelemetryDeliveryError::Transport)?;
        let outcome = parse_delivery_outcome(&idempotency_key, response);
        self.diagnostics.push(outcome.to_diagnostic());
        Ok(outcome)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TelemetryDeliveryError {
    EmptyBatch,
    InvalidPayloadFloat {
        event_name: String,
        payload_key: String,
    },
    SerializationFailed {
        reason_code: String,
    },
    Transport(TelemetryTransportError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TelemetryDeliveryOutcome {
    Accepted {
        accepted: u32,
        rejected: u32,
        request_id: String,
        idempotency_key: String,
    },
    PartiallyAccepted {
        accepted: u32,
        rejected: u32,
        request_id: String,
        idempotency_key: String,
    },
    Failed {
        http_status: u16,
        retryable: bool,
        error_code: Option<String>,
        request_id: Option<String>,
        retry_after_seconds: Option<u64>,
        idempotency_key: String,
    },
}

impl TelemetryDeliveryOutcome {
    fn to_diagnostic(&self) -> TelemetryDeliveryDiagnostic {
        match self {
            Self::Accepted {
                request_id,
                idempotency_key,
                ..
            } => TelemetryDeliveryDiagnostic {
                status: "accepted".to_string(),
                http_status: Some(202),
                retryable: false,
                request_id: Some(request_id.clone()),
                error_code: None,
                idempotency_key: idempotency_key.clone(),
            },
            Self::PartiallyAccepted {
                request_id,
                idempotency_key,
                ..
            } => TelemetryDeliveryDiagnostic {
                status: "partial".to_string(),
                http_status: Some(202),
                retryable: false,
                request_id: Some(request_id.clone()),
                error_code: None,
                idempotency_key: idempotency_key.clone(),
            },
            Self::Failed {
                http_status,
                retryable,
                error_code,
                request_id,
                idempotency_key,
                ..
            } => TelemetryDeliveryDiagnostic {
                status: "failed".to_string(),
                http_status: Some(*http_status),
                retryable: *retryable,
                request_id: request_id.clone(),
                error_code: error_code.clone(),
                idempotency_key: idempotency_key.clone(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelemetryDeliveryDiagnostic {
    pub status: String,
    pub http_status: Option<u16>,
    pub retryable: bool,
    pub request_id: Option<String>,
    pub error_code: Option<String>,
    pub idempotency_key: String,
}

pub fn build_idempotency_key(body: &TelemetryBatchRequest) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in canonical_request_bytes(body) {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    format!("kp2-{hash:016x}-{:04}", body.events.len())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetryBatchRequest {
    pub installation_id_hash: String,
    pub events: Vec<TelemetryBatchEvent>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetryBatchEvent {
    pub event_name: String,
    pub occurred_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub payload: JsonMap<String, JsonValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct TelemetryAcceptedResponse {
    accepted: u32,
    rejected: u32,
    request_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ErrorEnvelope {
    error: ErrorObject,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ErrorObject {
    code: String,
    message: String,
    retryable: bool,
    request_id: String,
}

fn build_request_body(
    installation_id_hash: &str,
    events: &[TelemetryDeliveryEvent],
) -> Result<TelemetryBatchRequest, TelemetryDeliveryError> {
    let mut mapped_events = Vec::with_capacity(events.len());
    for item in events {
        mapped_events.push(TelemetryBatchEvent {
            event_name: item.event.name.clone(),
            occurred_at: item.occurred_at.clone(),
            session_id: item.session_id.clone(),
            payload: payload_to_json_map(&item.event.name, &item.event.payload)?,
        });
    }

    Ok(TelemetryBatchRequest {
        installation_id_hash: installation_id_hash.to_string(),
        events: mapped_events,
    })
}

fn payload_to_json_map(
    event_name: &str,
    payload: &BTreeMap<String, TelemetryValue>,
) -> Result<JsonMap<String, JsonValue>, TelemetryDeliveryError> {
    let mut mapped = JsonMap::new();
    for (key, value) in payload {
        mapped.insert(key.clone(), telemetry_value_to_json(event_name, key, value)?);
    }
    Ok(mapped)
}

fn telemetry_value_to_json(
    event_name: &str,
    key: &str,
    value: &TelemetryValue,
) -> Result<JsonValue, TelemetryDeliveryError> {
    match value {
        TelemetryValue::Text(text) => Ok(JsonValue::String(text.clone())),
        TelemetryValue::Integer(number) => Ok(JsonValue::from(*number)),
        TelemetryValue::Float(number) => {
            if !number.is_finite() {
                return Err(TelemetryDeliveryError::InvalidPayloadFloat {
                    event_name: event_name.to_string(),
                    payload_key: key.to_string(),
                });
            }
            Ok(JsonValue::from(*number))
        }
    }
}

fn parse_delivery_outcome(idempotency_key: &str, response: TelemetryHttpResponse) -> TelemetryDeliveryOutcome {
    if response.status_code == 202 {
        let accepted = serde_json::from_str::<TelemetryAcceptedResponse>(&response.body_json);
        if let Ok(parsed) = accepted {
            if parsed.rejected == 0 {
                return TelemetryDeliveryOutcome::Accepted {
                    accepted: parsed.accepted,
                    rejected: parsed.rejected,
                    request_id: parsed.request_id,
                    idempotency_key: idempotency_key.to_string(),
                };
            }
            return TelemetryDeliveryOutcome::PartiallyAccepted {
                accepted: parsed.accepted,
                rejected: parsed.rejected,
                request_id: parsed.request_id,
                idempotency_key: idempotency_key.to_string(),
            };
        }

        return TelemetryDeliveryOutcome::Failed {
            http_status: 202,
            retryable: true,
            error_code: Some("malformed_accepted_response".to_string()),
            request_id: None,
            retry_after_seconds: response.retry_after_seconds,
            idempotency_key: idempotency_key.to_string(),
        };
    }

    let envelope = serde_json::from_str::<ErrorEnvelope>(&response.body_json).ok();
    let (error_code, request_id, retryable_from_body) = match envelope {
        Some(parsed) => (
            Some(parsed.error.code),
            Some(parsed.error.request_id),
            Some(parsed.error.retryable),
        ),
        None => (None, None, None),
    };

    TelemetryDeliveryOutcome::Failed {
        http_status: response.status_code,
        retryable: retryable_from_body.unwrap_or_else(|| is_retryable_status(response.status_code)),
        error_code,
        request_id,
        retry_after_seconds: response.retry_after_seconds,
        idempotency_key: idempotency_key.to_string(),
    }
}

fn canonical_request_bytes(body: &TelemetryBatchRequest) -> Vec<u8> {
    let mut canonical = String::new();
    canonical.push_str(body.installation_id_hash.as_str());
    canonical.push('|');
    canonical.push_str(body.events.len().to_string().as_str());
    canonical.push('|');

    for event in &body.events {
        canonical.push_str(event.event_name.as_str());
        canonical.push('|');
        canonical.push_str(event.occurred_at.as_str());
        canonical.push('|');
        canonical.push_str(event.session_id.as_deref().unwrap_or("none"));
        canonical.push('|');

        let mut keys = event.payload.keys().cloned().collect::<Vec<_>>();
        keys.sort();
        for key in keys {
            canonical.push_str(key.as_str());
            canonical.push('=');
            if let Some(value) = event.payload.get(&key) {
                canonical.push_str(value.to_string().as_str());
            }
            canonical.push(';');
        }
        canonical.push('|');
    }

    canonical.into_bytes()
}

fn is_retryable_status(status_code: u16) -> bool {
    matches!(status_code, 408 | 429 | 500 | 502 | 503 | 504)
}

#[cfg(test)]
mod tests {
    use super::{
        build_idempotency_key, TelemetryBatchEvent, TelemetryBatchRequest, TelemetryDeliveryClient,
        TelemetryDeliveryError, TelemetryDeliveryEvent, TelemetryDeliveryOutcome,
        TelemetryHttpRequest, TelemetryHttpResponse, TelemetryHttpTransport, TelemetryTransportError,
        TelemetryTransportErrorCode, TELEMETRY_INGEST_PATH,
    };
    use crate::telemetry::{TelemetryEvent, TelemetryValue};
    use serde_json::{json, Map as JsonMap};
    use std::collections::{BTreeMap, VecDeque};

    #[derive(Debug, Clone)]
    struct FakeTransport {
        responses: VecDeque<Result<TelemetryHttpResponse, TelemetryTransportError>>,
        requests: Vec<TelemetryHttpRequest>,
    }

    impl FakeTransport {
        fn with_response(response: Result<TelemetryHttpResponse, TelemetryTransportError>) -> Self {
            Self {
                responses: VecDeque::from([response]),
                requests: Vec::new(),
            }
        }
    }

    impl TelemetryHttpTransport for FakeTransport {
        fn post_events_batch(
            &mut self,
            request: &TelemetryHttpRequest,
        ) -> Result<TelemetryHttpResponse, TelemetryTransportError> {
            self.requests.push(request.clone());
            self.responses
                .pop_front()
                .expect("fake response should be configured")
        }
    }

    fn sample_event(name: &str) -> TelemetryDeliveryEvent {
        TelemetryDeliveryEvent {
            event: TelemetryEvent::new(
                name,
                BTreeMap::from([
                    (
                        "reason_code".to_string(),
                        TelemetryValue::Text("safe_generic_issue".to_string()),
                    ),
                    ("attempt_count".to_string(), TelemetryValue::Integer(2)),
                ]),
            ),
            occurred_at: "2026-05-06T09:10:00Z".to_string(),
            session_id: Some("sess-01".to_string()),
        }
    }

    #[test]
    fn idempotency_key_is_deterministic_for_same_payload() {
        let body = TelemetryBatchRequest {
            installation_id_hash: "sha256:test".to_string(),
            events: vec![TelemetryBatchEvent {
                event_name: "relay_failed".to_string(),
                occurred_at: "2026-05-06T09:10:00Z".to_string(),
                session_id: Some("sess-01".to_string()),
                payload: JsonMap::from_iter([
                    ("attempt_count".to_string(), json!(2)),
                    ("reason_code".to_string(), json!("safe_generic_issue")),
                ]),
            }],
        };

        let key1 = build_idempotency_key(&body);
        let key2 = build_idempotency_key(&body);
        assert_eq!(key1, key2);
        assert!(key1.starts_with("kp2-"));
    }

    #[test]
    fn request_includes_idempotency_key_and_contract_path() {
        let transport = FakeTransport::with_response(Ok(TelemetryHttpResponse {
            status_code: 202,
            body_json: r#"{"accepted":1,"rejected":0,"request_id":"req-1"}"#.to_string(),
            retry_after_seconds: None,
        }));
        let mut client = TelemetryDeliveryClient::new(transport);

        let outcome = client
            .send_batch("sha256:device", &[sample_event("routing_enabled")])
            .expect("send should succeed");
        match outcome {
            TelemetryDeliveryOutcome::Accepted {
                accepted,
                rejected,
                request_id,
                ..
            } => {
                assert_eq!(accepted, 1);
                assert_eq!(rejected, 0);
                assert_eq!(request_id, "req-1");
            }
            _ => panic!("expected accepted outcome"),
        }

        let sent = client.transport.requests.first().expect("request should exist");
        assert_eq!(sent.path, TELEMETRY_INGEST_PATH);
        assert!(sent.idempotency_key.len() >= 8);
    }

    #[test]
    fn response_parsing_supports_partial_acceptance_outcome() {
        let transport = FakeTransport::with_response(Ok(TelemetryHttpResponse {
            status_code: 202,
            body_json: r#"{"accepted":2,"rejected":1,"request_id":"req-2"}"#.to_string(),
            retry_after_seconds: None,
        }));
        let mut client = TelemetryDeliveryClient::new(transport);

        let outcome = client
            .send_batch(
                "sha256:device",
                &[sample_event("relay_failed"), sample_event("relay_recovered")],
            )
            .expect("partial response should parse");
        match outcome {
            TelemetryDeliveryOutcome::PartiallyAccepted {
                accepted,
                rejected,
                request_id,
                ..
            } => {
                assert_eq!(accepted, 2);
                assert_eq!(rejected, 1);
                assert_eq!(request_id, "req-2");
            }
            _ => panic!("expected partial outcome"),
        }
    }

    #[test]
    fn response_parsing_supports_failure_outcome() {
        let transport = FakeTransport::with_response(Ok(TelemetryHttpResponse {
            status_code: 429,
            body_json:
                r#"{"error":{"code":"RATE_LIMITED","message":"limited","retryable":true,"request_id":"req-3"}}"#
                    .to_string(),
            retry_after_seconds: Some(30),
        }));
        let mut client = TelemetryDeliveryClient::new(transport);

        let outcome = client
            .send_batch("sha256:device", &[sample_event("routing_disabled")])
            .expect("failure should still return outcome enum");
        match outcome {
            TelemetryDeliveryOutcome::Failed {
                http_status,
                retryable,
                error_code,
                request_id,
                retry_after_seconds,
                ..
            } => {
                assert_eq!(http_status, 429);
                assert!(retryable);
                assert_eq!(error_code.as_deref(), Some("RATE_LIMITED"));
                assert_eq!(request_id.as_deref(), Some("req-3"));
                assert_eq!(retry_after_seconds, Some(30));
            }
            _ => panic!("expected failed outcome"),
        }
    }

    #[test]
    fn transport_errors_propagate_for_retry_state_machine() {
        let transport = FakeTransport::with_response(Err(TelemetryTransportError {
            code: TelemetryTransportErrorCode::Timeout,
            message: "network timeout".to_string(),
        }));
        let mut client = TelemetryDeliveryClient::new(transport);

        let error = client
            .send_batch("sha256:device", &[sample_event("routing_enabled")])
            .expect_err("transport errors should propagate");
        match error {
            TelemetryDeliveryError::Transport(source) => {
                assert_eq!(source.code.as_code(), "transport_timeout");
            }
            _ => panic!("expected transport error"),
        }
    }

    #[test]
    fn diagnostics_do_not_include_payload_values() {
        let transport = FakeTransport::with_response(Ok(TelemetryHttpResponse {
            status_code: 422,
            body_json: r#"{"error":{"code":"TELEMETRY_SCHEMA_VIOLATION","message":"invalid","retryable":false,"request_id":"req-4"}}"#.to_string(),
            retry_after_seconds: None,
        }));
        let mut client = TelemetryDeliveryClient::new(transport);
        let mut event = sample_event("routing_enabled");
        event.event.payload.insert(
            "private_host".to_string(),
            TelemetryValue::Text("secret.internal.example".to_string()),
        );

        let _ = client
            .send_batch("sha256:device", &[event])
            .expect("http failure should still return outcome");
        let diagnostics = client.take_diagnostics();
        assert_eq!(diagnostics.len(), 1);
        let diag = &diagnostics[0];
        let rendered = format!("{diag:?}");
        assert!(!rendered.contains("private_host"));
        assert!(!rendered.contains("secret.internal.example"));
    }
}

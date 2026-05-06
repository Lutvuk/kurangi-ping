use serde_json::Value as JsonValue;

use super::{TelemetryPayload, TelemetryValue};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensitiveFieldMode {
    Reject,
    Sanitize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SensitiveFieldPolicy {
    pub mode: SensitiveFieldMode,
}

impl SensitiveFieldPolicy {
    pub fn reject() -> Self {
        Self {
            mode: SensitiveFieldMode::Reject,
        }
    }

    pub fn sanitize() -> Self {
        Self {
            mode: SensitiveFieldMode::Sanitize,
        }
    }
}

impl Default for SensitiveFieldPolicy {
    fn default() -> Self {
        Self::sanitize()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensitiveFieldViolationCode {
    UsernameField,
    EmailField,
    PhoneField,
    IpAddressField,
    PacketPayloadField,
    DestinationHostListField,
    ProcessCommandLineField,
}

impl SensitiveFieldViolationCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UsernameField => "username_field",
            Self::EmailField => "email_field",
            Self::PhoneField => "phone_field",
            Self::IpAddressField => "ip_address_field",
            Self::PacketPayloadField => "packet_payload_field",
            Self::DestinationHostListField => "destination_host_list_field",
            Self::ProcessCommandLineField => "process_command_line_field",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveFieldViolation {
    pub path: String,
    pub code: SensitiveFieldViolationCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensitiveFieldAction {
    Rejected,
    Sanitized,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveFieldDiagnostic {
    pub event_name: String,
    pub field_path: String,
    pub code: SensitiveFieldViolationCode,
    pub action: SensitiveFieldAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveFieldViolationError {
    pub event_name: String,
    pub violation: SensitiveFieldViolation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrubbedPayload {
    pub payload: TelemetryPayload,
    pub violations: Vec<SensitiveFieldViolation>,
}

pub fn scrub_or_reject_payload(
    event_name: &str,
    payload: &TelemetryPayload,
    policy: SensitiveFieldPolicy,
) -> Result<ScrubbedPayload, SensitiveFieldViolationError> {
    let mut violations = Vec::new();
    let mut sanitized = TelemetryPayload::new();

    for (key, value) in payload {
        if let Some(code) = classify_sensitive_key(key) {
            let violation = SensitiveFieldViolation {
                path: key.clone(),
                code,
            };
            if matches!(policy.mode, SensitiveFieldMode::Reject) {
                return Err(SensitiveFieldViolationError {
                    event_name: event_name.to_string(),
                    violation,
                });
            }
            violations.push(violation);
            continue;
        }

        let scrubbed = scrub_value(event_name, key, value, policy, &mut violations)?;
        sanitized.insert(key.clone(), scrubbed);
    }

    Ok(ScrubbedPayload {
        payload: sanitized,
        violations,
    })
}

fn scrub_value(
    event_name: &str,
    root_path: &str,
    value: &TelemetryValue,
    policy: SensitiveFieldPolicy,
    violations: &mut Vec<SensitiveFieldViolation>,
) -> Result<TelemetryValue, SensitiveFieldViolationError> {
    match value {
        TelemetryValue::Text(text) => {
            if !looks_like_json(text) {
                return Ok(TelemetryValue::Text(text.clone()));
            }

            let mut json = match serde_json::from_str::<JsonValue>(text) {
                Ok(parsed) => parsed,
                Err(_) => return Ok(TelemetryValue::Text(text.clone())),
            };

            scrub_json_value(event_name, root_path, &mut json, policy, violations)?;
            Ok(TelemetryValue::Text(json.to_string()))
        }
        TelemetryValue::Integer(value) => Ok(TelemetryValue::Integer(*value)),
        TelemetryValue::Float(value) => Ok(TelemetryValue::Float(*value)),
    }
}

fn scrub_json_value(
    event_name: &str,
    path: &str,
    value: &mut JsonValue,
    policy: SensitiveFieldPolicy,
    violations: &mut Vec<SensitiveFieldViolation>,
) -> Result<(), SensitiveFieldViolationError> {
    match value {
        JsonValue::Object(map) => {
            let mut keys = map.keys().cloned().collect::<Vec<_>>();
            keys.sort();

            for key in keys {
                let child_path = format!("{path}.{key}");
                if let Some(code) = classify_sensitive_key(&key) {
                    let violation = SensitiveFieldViolation {
                        path: child_path,
                        code,
                    };
                    if matches!(policy.mode, SensitiveFieldMode::Reject) {
                        return Err(SensitiveFieldViolationError {
                            event_name: event_name.to_string(),
                            violation,
                        });
                    }
                    violations.push(violation);
                    map.remove(&key);
                    continue;
                }

                if let Some(child) = map.get_mut(&key) {
                    scrub_json_value(event_name, &child_path, child, policy, violations)?;
                }
            }
        }
        JsonValue::Array(items) => {
            for (index, item) in items.iter_mut().enumerate() {
                let child_path = format!("{path}[{index}]");
                scrub_json_value(event_name, &child_path, item, policy, violations)?;
            }
        }
        JsonValue::Null | JsonValue::Bool(_) | JsonValue::Number(_) | JsonValue::String(_) => {}
    }

    Ok(())
}

fn looks_like_json(raw: &str) -> bool {
    let trimmed = raw.trim();
    (trimmed.starts_with('{') && trimmed.ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']'))
}

fn classify_sensitive_key(key: &str) -> Option<SensitiveFieldViolationCode> {
    let normalized = key
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect::<String>();

    if normalized.contains("username") {
        return Some(SensitiveFieldViolationCode::UsernameField);
    }
    if normalized.contains("email") {
        return Some(SensitiveFieldViolationCode::EmailField);
    }
    if normalized.contains("phone") || normalized.contains("phonenumber") {
        return Some(SensitiveFieldViolationCode::PhoneField);
    }
    if normalized == "ip"
        || normalized.contains("ipaddress")
        || normalized.ends_with("sourceip")
        || normalized.ends_with("clientip")
    {
        return Some(SensitiveFieldViolationCode::IpAddressField);
    }
    if normalized.contains("packetpayload") || normalized.contains("rawpayload") {
        return Some(SensitiveFieldViolationCode::PacketPayloadField);
    }
    if normalized.contains("destinationhostlist")
        || normalized.contains("destinationhosts")
        || normalized.contains("rawdestinationhosts")
    {
        return Some(SensitiveFieldViolationCode::DestinationHostListField);
    }
    if normalized.contains("processcommandline")
        || normalized.contains("fullcommandline")
        || normalized.contains("cmdline")
    {
        return Some(SensitiveFieldViolationCode::ProcessCommandLineField);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{
        scrub_or_reject_payload, SensitiveFieldMode, SensitiveFieldPolicy,
        SensitiveFieldViolationCode,
    };
    use crate::telemetry::{TelemetryPayload, TelemetryValue};

    #[test]
    fn reject_mode_blocks_prohibited_root_fields() {
        let payload = TelemetryPayload::from([
            (
                "username".to_string(),
                TelemetryValue::Text("secret-user".to_string()),
            ),
            (
                "reason_code".to_string(),
                TelemetryValue::Text("safe_generic_issue".to_string()),
            ),
        ]);

        let error =
            scrub_or_reject_payload("relay_failed", &payload, SensitiveFieldPolicy::reject())
                .expect_err("reject mode should block prohibited fields");
        assert_eq!(error.event_name, "relay_failed");
        assert_eq!(error.violation.path, "username");
        assert_eq!(
            error.violation.code,
            SensitiveFieldViolationCode::UsernameField
        );
    }

    #[test]
    fn sanitize_mode_removes_prohibited_root_fields() {
        let payload = TelemetryPayload::from([
            (
                "email".to_string(),
                TelemetryValue::Text("secret@example.com".to_string()),
            ),
            (
                "reason_code".to_string(),
                TelemetryValue::Text("safe_generic_issue".to_string()),
            ),
        ]);

        let result =
            scrub_or_reject_payload("relay_failed", &payload, SensitiveFieldPolicy::sanitize())
                .expect("sanitize mode should pass");
        assert!(!result.payload.contains_key("email"));
        assert_eq!(result.payload.len(), 1);
        assert_eq!(result.violations.len(), 1);
    }

    #[test]
    fn nested_json_payload_fields_are_scrubbed_safely() {
        let payload = TelemetryPayload::from([(
            "process_name".to_string(),
            TelemetryValue::Text(
                r#"{"exe":"ffxiv_dx11.exe","username":"player-secret","network":{"ip_address":"10.0.0.1"}}"#
                    .to_string(),
            ),
        )]);

        let result =
            scrub_or_reject_payload("game_detected", &payload, SensitiveFieldPolicy::sanitize())
                .expect("nested payload should be scrubbed");
        assert_eq!(result.violations.len(), 2);

        let scrubbed_text = match result.payload.get("process_name") {
            Some(TelemetryValue::Text(value)) => value,
            _ => panic!("process_name should remain text"),
        };
        let json = serde_json::from_str::<serde_json::Value>(scrubbed_text)
            .expect("scrubbed json should remain valid");
        assert_eq!(json.get("username"), None);
        assert_eq!(
            json.get("network")
                .and_then(|value| value.get("ip_address")),
            None
        );
        assert_eq!(
            json.get("exe").and_then(|value| value.as_str()),
            Some("ffxiv_dx11.exe")
        );
    }

    #[test]
    fn policy_is_explicitly_configurable_between_reject_and_sanitize() {
        let reject = SensitiveFieldPolicy::reject();
        let sanitize = SensitiveFieldPolicy::sanitize();

        assert_eq!(reject.mode, SensitiveFieldMode::Reject);
        assert_eq!(sanitize.mode, SensitiveFieldMode::Sanitize);
    }
}

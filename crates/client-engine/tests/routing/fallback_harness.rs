use client_engine::routing::{
    attempt_route, score_candidates, verify_manifest_or_fail, AttemptFailureReason, AttemptStatus,
    AttemptStepOutcome, ManifestGateResult, RelayHealthSnapshot, RelayHealthStatus,
    RelayManifestDto, RelayNodeDto, RelayScoringConfig, RouteAttemptFailureCode, RouteCandidate,
    RouteDialer, RouteProtocol,
};
use client_engine::security::signature::AllowlistSignatureVerifier;
use serde_json::Value;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
struct FallbackScenario {
    name: String,
    now_unix_s: u64,
    manifest: RelayManifestDto,
    health: Vec<RelayHealthSnapshot>,
    fail_protocols: HashSet<RouteProtocol>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum HarnessTerminalState {
    Connected { protocol: RouteProtocol },
    Failed { code: RouteAttemptFailureCode },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HarnessResult {
    terminal_state: HarnessTerminalState,
    trace: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FreshnessHarnessCycleResult {
    cycle_index: usize,
    terminal_state: HarnessTerminalState,
    relay_statuses: Vec<(String, RelayHealthStatus)>,
}

fn run_fallback_scenario(scenario: &FallbackScenario) -> HarnessResult {
    let verifier = AllowlistSignatureVerifier::new(vec!["known-good".to_string()]);
    let gate = verify_manifest_or_fail(&verifier, &scenario.manifest, scenario.now_unix_s);
    let candidates = match gate {
        ManifestGateResult::Passed { candidates, .. } => candidates,
        ManifestGateResult::Blocked { failure_code } => {
            return HarnessResult {
                terminal_state: HarnessTerminalState::Failed {
                    code: RouteAttemptFailureCode::NoEligibleCandidates,
                },
                trace: vec![format!("manifest_blocked:{}", failure_code.as_code())],
            };
        }
    };

    let scored = score_candidates(
        &candidates,
        &scenario.health,
        &RelayScoringConfig::default(),
    );
    let dialer = ScenarioDialer {
        fail_protocols: scenario.fail_protocols.clone(),
    };

    let result = attempt_route(
        &[
            RouteProtocol::WireGuard,
            RouteProtocol::TcpTls,
            RouteProtocol::Quic,
        ],
        &scored,
        &dialer,
    );

    let trace = build_trace(&result.attempts);
    let terminal_state = match result.status {
        AttemptStatus::Connected { protocol, .. } => HarnessTerminalState::Connected { protocol },
        AttemptStatus::Exhausted { failure_code } => {
            HarnessTerminalState::Failed { code: failure_code }
        }
    };

    HarnessResult {
        terminal_state,
        trace,
    }
}

fn relay_health_freshness_consumption_harness(
    payload_cycles: &[Value],
) -> Result<Vec<FreshnessHarnessCycleResult>, String> {
    let verifier = AllowlistSignatureVerifier::new(vec!["known-good".to_string()]);
    let manifest = RelayManifestDto {
        version: "2026.09.0".to_string(),
        valid_until_unix_s: 1_900_000_000,
        signature_b64: "known-good".to_string(),
        relays: vec![RelayNodeDto {
            relay_id: "sin-01".to_string(),
            region: "sin".to_string(),
            hostname: "sin-01.example.net".to_string(),
            priority: 1,
        }],
    };

    let candidates = match verify_manifest_or_fail(&verifier, &manifest, 1_700_000_000) {
        ManifestGateResult::Passed { candidates, .. } => candidates,
        ManifestGateResult::Blocked { failure_code } => {
            return Err(format!(
                "manifest verification blocked unexpectedly: {}",
                failure_code.as_code()
            ));
        }
    };

    let mut last_seen_updated_at_by_relay = BTreeMap::<String, String>::new();
    let dialer = ScenarioDialer {
        fail_protocols: HashSet::new(),
    };

    let mut cycle_results = Vec::with_capacity(payload_cycles.len());
    for (cycle_index, payload) in payload_cycles.iter().enumerate() {
        let snapshots = parse_controller_health_payload(
            payload,
            cycle_index,
            &mut last_seen_updated_at_by_relay,
        )?;

        let scored = score_candidates(&candidates, &snapshots, &RelayScoringConfig::default());
        let attempt = attempt_route(
            &[
                RouteProtocol::WireGuard,
                RouteProtocol::TcpTls,
                RouteProtocol::Quic,
            ],
            &scored,
            &dialer,
        );
        let terminal_state = match attempt.status {
            AttemptStatus::Connected { protocol, .. } => HarnessTerminalState::Connected { protocol },
            AttemptStatus::Exhausted { failure_code } => {
                HarnessTerminalState::Failed { code: failure_code }
            }
        };

        let mut relay_statuses = snapshots
            .iter()
            .map(|snapshot| (snapshot.relay_id.clone(), snapshot.status))
            .collect::<Vec<_>>();
        relay_statuses.sort_by(|left, right| left.0.cmp(&right.0));

        cycle_results.push(FreshnessHarnessCycleResult {
            cycle_index,
            terminal_state,
            relay_statuses,
        });
    }

    Ok(cycle_results)
}

fn parse_controller_health_payload(
    payload: &Value,
    cycle_index: usize,
    last_seen_updated_at_by_relay: &mut BTreeMap<String, String>,
) -> Result<Vec<RelayHealthSnapshot>, String> {
    let object = payload
        .as_object()
        .ok_or_else(|| format!("cycle #{cycle_index}: payload must be object"))?;
    let data = object
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("cycle #{cycle_index}: data must be array"))?;

    let mut snapshots = Vec::with_capacity(data.len());
    for item in data {
        let item_object = item
            .as_object()
            .ok_or_else(|| format!("cycle #{cycle_index}: health item must be object"))?;

        let relay_id = item_object
            .get("relay_id")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("cycle #{cycle_index}: relay_id must be string"))?;
        let updated_at = item_object
            .get("updated_at")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("cycle #{cycle_index}: updated_at must be string"))?;

        if let Some(previous) = last_seen_updated_at_by_relay.get(relay_id) {
            if updated_at <= previous.as_str() {
                return Err(format!(
                    "cycle #{cycle_index}: relay {relay_id} updated_at must increase (prev={previous}, now={updated_at})"
                ));
            }
        }
        last_seen_updated_at_by_relay.insert(relay_id.to_string(), updated_at.to_string());

        let status = match item_object
            .get("status")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("cycle #{cycle_index}: status must be string"))?
        {
            "ok" => RelayHealthStatus::Ok,
            "warn" => RelayHealthStatus::Warn,
            "dead" => RelayHealthStatus::Dead,
            invalid => return Err(format!("cycle #{cycle_index}: invalid status {invalid}")),
        };

        let latency_ms = item_object
            .get("latency_ms")
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("cycle #{cycle_index}: latency_ms must be integer"))?;

        snapshots.push(RelayHealthSnapshot {
            relay_id: relay_id.to_string(),
            status,
            latency_ms: latency_ms as u32,
        });
    }

    snapshots.sort_by(|left, right| left.relay_id.cmp(&right.relay_id));
    Ok(snapshots)
}

#[derive(Debug)]
struct ScenarioDialer {
    fail_protocols: HashSet<RouteProtocol>,
}

impl RouteDialer for ScenarioDialer {
    fn attempt(&self, protocol: RouteProtocol, _candidate: &RouteCandidate) -> AttemptStepOutcome {
        if self.fail_protocols.contains(&protocol) {
            AttemptStepOutcome::Failed(AttemptFailureReason::Timeout)
        } else {
            AttemptStepOutcome::Success
        }
    }
}

fn build_trace(attempts: &[client_engine::routing::AttemptRecord]) -> Vec<String> {
    attempts
        .iter()
        .map(|attempt| {
            let protocol = match attempt.protocol {
                RouteProtocol::WireGuard => "wireguard",
                RouteProtocol::TcpTls => "tcp_tls",
                RouteProtocol::Quic => "quic",
            };
            let outcome = match attempt.outcome {
                AttemptStepOutcome::Success => "success",
                AttemptStepOutcome::Failed(_) => "failed",
            };
            format!("{protocol}@{}:{outcome}", attempt.candidate.relay_id)
        })
        .collect()
}

fn load_scenario(path: &str) -> FallbackScenario {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let full_path = base
        .join("tests")
        .join("fixtures")
        .join("relay_scenarios")
        .join(path);
    let raw = fs::read_to_string(&full_path).expect("scenario fixture should be readable");

    let mut name = String::new();
    let mut now_unix_s = 0_u64;
    let mut valid_until_unix_s = 0_u64;
    let mut signature = String::new();
    let mut relay_id = String::new();
    let mut relay_region = String::new();
    let mut relay_host = String::new();
    let mut relay_priority = 0_u16;
    let mut health_status = RelayHealthStatus::Ok;
    let mut latency_ms = 0_u32;
    let mut fail_protocols = HashSet::new();

    for line in raw.lines().filter(|line| !line.trim().is_empty()) {
        let mut parts = line.splitn(2, '=');
        let key = parts.next().expect("key should exist").trim();
        let value = parts.next().expect("value should exist").trim();
        match key {
            "name" => name = value.to_string(),
            "now_unix_s" => now_unix_s = value.parse().expect("now_unix_s should parse"),
            "valid_until_unix_s" => {
                valid_until_unix_s = value.parse().expect("valid_until_unix_s should parse")
            }
            "signature" => signature = value.to_string(),
            "relay_id" => relay_id = value.to_string(),
            "relay_region" => relay_region = value.to_string(),
            "relay_host" => relay_host = value.to_string(),
            "relay_priority" => {
                relay_priority = value.parse().expect("relay_priority should parse")
            }
            "health_status" => {
                health_status = match value {
                    "ok" => RelayHealthStatus::Ok,
                    "warn" => RelayHealthStatus::Warn,
                    "dead" => RelayHealthStatus::Dead,
                    _ => panic!("unknown health_status: {value}"),
                }
            }
            "latency_ms" => latency_ms = value.parse().expect("latency_ms should parse"),
            "fail_protocols" => {
                for protocol in value.split(',').filter(|raw| !raw.trim().is_empty()) {
                    let parsed = match protocol.trim() {
                        "wireguard" => RouteProtocol::WireGuard,
                        "tcp_tls" => RouteProtocol::TcpTls,
                        "quic" => RouteProtocol::Quic,
                        _ => panic!("unknown protocol in fail_protocols: {protocol}"),
                    };
                    fail_protocols.insert(parsed);
                }
            }
            _ => panic!("unknown fixture key: {key}"),
        }
    }

    FallbackScenario {
        name,
        now_unix_s,
        manifest: RelayManifestDto {
            version: "2026.08.0".to_string(),
            valid_until_unix_s,
            signature_b64: signature,
            relays: vec![RelayNodeDto {
                relay_id: relay_id.clone(),
                region: relay_region,
                hostname: relay_host,
                priority: relay_priority,
            }],
        },
        health: vec![RelayHealthSnapshot {
            relay_id,
            status: health_status,
            latency_ms,
        }],
        fail_protocols,
    }
}

#[test]
fn harness_validates_wireguard_failure_to_tcp_tls_fallback() {
    let scenario = load_scenario("wg_to_tcp_tls.scn");
    let result = run_fallback_scenario(&scenario);

    assert_eq!(scenario.name, "wg_to_tcp_tls");
    assert_eq!(
        result.terminal_state,
        HarnessTerminalState::Connected {
            protocol: RouteProtocol::TcpTls
        }
    );
    assert_eq!(
        result.trace,
        vec![
            "wireguard@sin-01:failed".to_string(),
            "tcp_tls@sin-01:success".to_string()
        ]
    );
}

#[test]
fn harness_validates_tcp_tls_failure_to_quic_tertiary_path() {
    let scenario = load_scenario("tcp_tls_to_quic.scn");
    let result = run_fallback_scenario(&scenario);

    assert_eq!(scenario.name, "tcp_tls_to_quic");
    assert_eq!(
        result.terminal_state,
        HarnessTerminalState::Connected {
            protocol: RouteProtocol::Quic
        }
    );
    assert_eq!(
        result.trace,
        vec![
            "wireguard@sin-01:failed".to_string(),
            "tcp_tls@sin-01:failed".to_string(),
            "quic@sin-01:success".to_string()
        ]
    );
}

#[test]
fn harness_validates_exhaustion_terminal_failed_state() {
    let scenario = load_scenario("exhaust_all.scn");
    let result = run_fallback_scenario(&scenario);

    assert_eq!(scenario.name, "exhaust_all");
    assert_eq!(
        result.terminal_state,
        HarnessTerminalState::Failed {
            code: RouteAttemptFailureCode::AllAttemptsFailed
        }
    );
    assert_eq!(
        result.trace,
        vec![
            "wireguard@sin-01:failed".to_string(),
            "tcp_tls@sin-01:failed".to_string(),
            "quic@sin-01:failed".to_string()
        ]
    );
}

#[test]
fn relay_health_freshness_harness_tracks_dynamic_cycles_deterministically() {
    let cycle_one = serde_json::json!({
        "data": [
            {
                "relay_id": "sin-01",
                "status": "ok",
                "latency_ms": 42,
                "updated_at": "2026-08-01T00:00:10Z"
            }
        ],
        "page": {
            "next_cursor": null,
            "limit": 50
        }
    });
    let cycle_two = serde_json::json!({
        "data": [
            {
                "relay_id": "sin-01",
                "status": "dead",
                "latency_ms": 0,
                "updated_at": "2026-08-01T00:00:20Z"
            }
        ],
        "page": {
            "next_cursor": null,
            "limit": 50
        }
    });
    let cycle_three = serde_json::json!({
        "data": [
            {
                "relay_id": "sin-01",
                "status": "warn",
                "latency_ms": 125,
                "updated_at": "2026-08-01T00:00:30Z"
            }
        ],
        "page": {
            "next_cursor": null,
            "limit": 50
        }
    });

    let first_run = relay_health_freshness_consumption_harness(&[
        cycle_one.clone(),
        cycle_two.clone(),
        cycle_three.clone(),
    ])
    .expect("payload cycles should be consumable");
    let second_run = relay_health_freshness_consumption_harness(&[cycle_one, cycle_two, cycle_three])
        .expect("same payload cycles should remain deterministic");

    assert_eq!(first_run, second_run);
    assert_eq!(
        first_run
            .iter()
            .map(|entry| entry.terminal_state.clone())
            .collect::<Vec<_>>(),
        vec![
            HarnessTerminalState::Connected {
                protocol: RouteProtocol::WireGuard
            },
            HarnessTerminalState::Failed {
                code: RouteAttemptFailureCode::NoEligibleCandidates
            },
            HarnessTerminalState::Connected {
                protocol: RouteProtocol::WireGuard
            }
        ]
    );
}

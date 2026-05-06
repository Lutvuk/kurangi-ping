use client_engine::routing::{
    attempt_route, score_candidates, verify_manifest_or_fail, AttemptFailureReason, AttemptStatus,
    AttemptStepOutcome, ManifestGateResult, RelayHealthSnapshot, RelayHealthStatus,
    RelayManifestDto, RelayNodeDto, RelayScoringConfig, RouteAttemptFailureCode, RouteCandidate,
    RouteDialer, RouteProtocol,
};
use client_engine::security::signature::AllowlistSignatureVerifier;
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

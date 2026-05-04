//! Routing and manifest verification boundaries.

mod config;
mod manifest_gate;
mod policy;

pub use config::{RoutingConfig, RoutingConfigError};
pub use manifest_gate::{
    verify_manifest_or_fail, ManifestFailureCode, ManifestGateResult, RelayManifestDto,
    RelayNodeDto, RouteCandidate,
};
pub use policy::{
    ProtocolPriority, RetryPolicy, RouteProtocol, RoutingPolicy, DEFAULT_PROTOCOL_ORDER,
};

use crate::security::signature::ManifestSignatureVerifier;

/// Public routing service API.
#[derive(Debug, Clone)]
pub struct RoutingService {
    active: bool,
    policy: RoutingPolicy,
}

impl RoutingService {
    pub fn new() -> Self {
        Self {
            active: false,
            policy: RoutingPolicy::default(),
        }
    }

    pub fn from_config(config: RoutingConfig) -> Result<Self, RoutingConfigError> {
        let policy = config.into_policy()?;
        Ok(Self {
            active: false,
            policy,
        })
    }

    pub fn state(&self) -> &'static str {
        if self.active { "enabled" } else { "disabled" }
    }

    pub fn policy(&self) -> RoutingPolicy {
        self.policy
    }

    pub fn protocol_order(&self) -> [RouteProtocol; 3] {
        self.policy.protocol_order()
    }

    pub fn plan_activation<V: ManifestSignatureVerifier>(
        &self,
        verifier: &V,
        manifest: &RelayManifestDto,
        now_unix_s: u64,
    ) -> RoutePlan {
        let manifest_gate = verify_manifest_or_fail(verifier, manifest, now_unix_s);
        let (manifest_valid, attempted_protocols, route_candidates, failure_code) = match &manifest_gate {
            ManifestGateResult::Passed { candidates, .. } => (
                true,
                self.protocol_order().to_vec(),
                candidates.clone(),
                None,
            ),
            ManifestGateResult::Blocked { failure_code } => {
                (false, Vec::new(), Vec::new(), Some(*failure_code))
            }
        };

        RoutePlan {
            manifest_valid,
            manifest_gate,
            attempted_protocols,
            route_candidates,
            failure_code,
        }
    }
}

impl Default for RoutingService {
    fn default() -> Self {
        Self::new()
    }
}

/// Dry-run activation plan to support orchestration tests.
#[derive(Debug, Clone)]
pub struct RoutePlan {
    pub manifest_valid: bool,
    pub manifest_gate: ManifestGateResult,
    pub attempted_protocols: Vec<RouteProtocol>,
    pub route_candidates: Vec<RouteCandidate>,
    pub failure_code: Option<ManifestFailureCode>,
}

#[cfg(test)]
mod tests {
    use super::{
        ManifestFailureCode, ManifestGateResult, RelayManifestDto, RelayNodeDto, RouteProtocol,
        RoutingConfig, RoutingConfigError, RoutingService, DEFAULT_PROTOCOL_ORDER,
    };
    use crate::security::signature::AllowlistSignatureVerifier;

    fn sample_manifest(signature_b64: &str, valid_until_unix_s: u64) -> RelayManifestDto {
        RelayManifestDto {
            version: "2026.08.0".to_string(),
            valid_until_unix_s,
            signature_b64: signature_b64.to_string(),
            relays: vec![RelayNodeDto {
                relay_id: "sin-01".to_string(),
                region: "sin".to_string(),
                hostname: "sin-01.example.net".to_string(),
                priority: 1,
            }],
        }
    }

    #[test]
    fn service_defaults_to_architecture_protocol_order() {
        let service = RoutingService::new();
        assert_eq!(service.protocol_order(), DEFAULT_PROTOCOL_ORDER);
        assert_eq!(service.state(), "disabled");
    }

    #[test]
    fn service_from_config_rejects_invalid_priority_sets() {
        let invalid_config = RoutingConfig {
            protocol_priority: vec![
                RouteProtocol::WireGuard,
                RouteProtocol::WireGuard,
                RouteProtocol::Quic,
            ],
            ..RoutingConfig::default()
        };

        let err = RoutingService::from_config(invalid_config).expect_err("config must be invalid");
        assert_eq!(
            err,
            RoutingConfigError::DuplicateProtocol {
                protocol: RouteProtocol::WireGuard
            }
        );
    }

    #[test]
    fn plan_activation_uses_policy_protocol_order() {
        let service = RoutingService::new();
        let verifier = AllowlistSignatureVerifier::new(vec!["known-good".to_string()]);
        let manifest = sample_manifest("known-good", 1_800_000_000);

        let plan = service.plan_activation(&verifier, &manifest, 1_700_000_000);
        assert!(plan.manifest_valid);
        assert!(matches!(plan.manifest_gate, ManifestGateResult::Passed { .. }));
        assert_eq!(plan.attempted_protocols, DEFAULT_PROTOCOL_ORDER.to_vec());
        assert_eq!(plan.route_candidates.len(), 1);
        assert_eq!(plan.failure_code, None);
    }

    #[test]
    fn plan_activation_blocks_when_manifest_invalid() {
        let service = RoutingService::new();
        let verifier = AllowlistSignatureVerifier::new(vec!["known-good".to_string()]);
        let manifest = sample_manifest("bad-signature", 1_800_000_000);

        let plan = service.plan_activation(&verifier, &manifest, 1_700_000_000);
        assert!(!plan.manifest_valid);
        assert!(plan.attempted_protocols.is_empty());
        assert!(plan.route_candidates.is_empty());
        assert_eq!(plan.failure_code, Some(ManifestFailureCode::SignatureInvalid));
    }
}

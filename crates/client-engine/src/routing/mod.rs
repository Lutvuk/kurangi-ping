//! Routing and manifest verification boundaries.

mod config;
mod policy;

pub use config::{RoutingConfig, RoutingConfigError};
pub use policy::{
    ProtocolPriority, RetryPolicy, RouteProtocol, RoutingPolicy, DEFAULT_PROTOCOL_ORDER,
};

/// Minimal signed manifest input shape for later verification implementation.
#[derive(Debug, Clone)]
pub struct SignedManifest {
    pub version: String,
    pub signature_b64: String,
}

/// Manifest verification interface.
pub trait ManifestVerifier {
    fn verify(&self, manifest: &SignedManifest) -> bool;
}

/// Placeholder verifier used in scaffold phase.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopManifestVerifier;

impl ManifestVerifier for NoopManifestVerifier {
    fn verify(&self, _manifest: &SignedManifest) -> bool {
        // TODO(KP-030): replace with signature verification against configured public key.
        false
    }
}

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

    pub fn plan_activation<V: ManifestVerifier>(
        &self,
        verifier: &V,
        manifest: &SignedManifest,
    ) -> RoutePlan {
        // Scaffold only: returns a dry-run plan with no side effects.
        let manifest_valid = verifier.verify(manifest);
        RoutePlan {
            manifest_valid,
            attempted_protocols: self.protocol_order().to_vec(),
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
    pub attempted_protocols: Vec<RouteProtocol>,
}

#[cfg(test)]
mod tests {
    use super::{
        ManifestVerifier, RouteProtocol, RoutingConfig, RoutingConfigError, RoutingService, SignedManifest,
        DEFAULT_PROTOCOL_ORDER,
    };

    struct AlwaysValidVerifier;

    impl ManifestVerifier for AlwaysValidVerifier {
        fn verify(&self, _manifest: &SignedManifest) -> bool {
            true
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
        let verifier = AlwaysValidVerifier;
        let manifest = SignedManifest {
            version: "v1".to_string(),
            signature_b64: "sig".to_string(),
        };

        let plan = service.plan_activation(&verifier, &manifest);
        assert!(plan.manifest_valid);
        assert_eq!(plan.attempted_protocols, DEFAULT_PROTOCOL_ORDER.to_vec());
    }
}

//! Routing and manifest verification boundaries.

/// Ordered protocol preference for route attempts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteProtocol {
    WireGuard,
    TcpTls,
    Quic,
}

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
    protocol_order: [RouteProtocol; 3],
}

impl RoutingService {
    pub fn new() -> Self {
        Self {
            active: false,
            protocol_order: [
                RouteProtocol::WireGuard,
                RouteProtocol::TcpTls,
                RouteProtocol::Quic,
            ],
        }
    }

    pub fn state(&self) -> &'static str {
        if self.active { "enabled" } else { "disabled" }
    }

    pub fn protocol_order(&self) -> [RouteProtocol; 3] {
        self.protocol_order
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
            attempted_protocols: self.protocol_order.to_vec(),
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

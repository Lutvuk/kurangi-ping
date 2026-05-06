//! Signature verification abstraction.

/// Signature verifier boundary for trust-gated payloads.
pub trait ManifestSignatureVerifier {
    fn verify(&self, payload: &[u8], signature_b64: &str) -> bool;
}

/// Minimal verifier that accepts only exact known signatures.
/// This is a scaffold verifier until asymmetric cryptography is wired.
#[derive(Debug, Clone, Default)]
pub struct AllowlistSignatureVerifier {
    accepted_signatures: Vec<String>,
}

impl AllowlistSignatureVerifier {
    pub fn new(accepted_signatures: Vec<String>) -> Self {
        Self {
            accepted_signatures,
        }
    }
}

impl ManifestSignatureVerifier for AllowlistSignatureVerifier {
    fn verify(&self, payload: &[u8], signature_b64: &str) -> bool {
        !payload.is_empty()
            && self
                .accepted_signatures
                .iter()
                .any(|known| known == signature_b64)
    }
}

/// Fails closed by default and is safe for integration tests requiring rejection.
#[derive(Debug, Clone, Copy, Default)]
pub struct RejectAllSignatureVerifier;

impl ManifestSignatureVerifier for RejectAllSignatureVerifier {
    fn verify(&self, _payload: &[u8], _signature_b64: &str) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AllowlistSignatureVerifier, ManifestSignatureVerifier, RejectAllSignatureVerifier,
    };

    #[test]
    fn allowlist_verifier_accepts_exact_signature_when_payload_present() {
        let verifier = AllowlistSignatureVerifier::new(vec!["sig-abc".to_string()]);
        assert!(verifier.verify(b"manifest-payload", "sig-abc"));
    }

    #[test]
    fn allowlist_verifier_rejects_unknown_or_empty_payload() {
        let verifier = AllowlistSignatureVerifier::new(vec!["sig-abc".to_string()]);
        assert!(!verifier.verify(b"manifest-payload", "sig-other"));
        assert!(!verifier.verify(b"", "sig-abc"));
    }

    #[test]
    fn reject_all_verifier_is_fail_closed() {
        let verifier = RejectAllSignatureVerifier;
        assert!(!verifier.verify(b"manifest-payload", "sig-abc"));
    }
}

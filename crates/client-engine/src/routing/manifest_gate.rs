//! Trust gate for signed relay manifest prior to route activation.

use crate::security::signature::ManifestSignatureVerifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelayNodeDto {
    pub relay_id: String,
    pub region: String,
    pub hostname: String,
    pub priority: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelayManifestDto {
    pub version: String,
    pub valid_until_unix_s: u64,
    pub signature_b64: String,
    pub relays: Vec<RelayNodeDto>,
}

impl RelayManifestDto {
    pub fn unsigned_payload(&self) -> String {
        let relay_lines = self
            .relays
            .iter()
            .map(|relay| {
                format!(
                    "{},{},{},{}",
                    relay.relay_id, relay.region, relay.hostname, relay.priority
                )
            })
            .collect::<Vec<_>>()
            .join(";");

        format!(
            "{}|{}|{}",
            self.version, self.valid_until_unix_s, relay_lines
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteCandidate {
    pub relay_id: String,
    pub region: String,
    pub hostname: String,
    pub priority: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestFailureCode {
    SignatureInvalid,
    ManifestExpired,
    NoRelayCandidates,
}

impl ManifestFailureCode {
    pub const fn as_code(self) -> &'static str {
        match self {
            Self::SignatureInvalid => "MANIFEST_SIGNATURE_INVALID",
            Self::ManifestExpired => "MANIFEST_EXPIRED",
            Self::NoRelayCandidates => "MANIFEST_EMPTY_CANDIDATES",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestGateResult {
    Passed {
        manifest_version: String,
        candidates: Vec<RouteCandidate>,
    },
    Blocked {
        failure_code: ManifestFailureCode,
    },
}

impl ManifestGateResult {
    pub fn failure_code(&self) -> Option<ManifestFailureCode> {
        match self {
            Self::Passed { .. } => None,
            Self::Blocked { failure_code } => Some(*failure_code),
        }
    }
}

pub fn verify_manifest_or_fail<V: ManifestSignatureVerifier>(
    verifier: &V,
    manifest: &RelayManifestDto,
    now_unix_s: u64,
) -> ManifestGateResult {
    let unsigned_payload = manifest.unsigned_payload();
    let is_signature_valid = verifier.verify(unsigned_payload.as_bytes(), &manifest.signature_b64);
    if !is_signature_valid {
        return ManifestGateResult::Blocked {
            failure_code: ManifestFailureCode::SignatureInvalid,
        };
    }

    if manifest.valid_until_unix_s <= now_unix_s {
        return ManifestGateResult::Blocked {
            failure_code: ManifestFailureCode::ManifestExpired,
        };
    }

    if manifest.relays.is_empty() {
        return ManifestGateResult::Blocked {
            failure_code: ManifestFailureCode::NoRelayCandidates,
        };
    }

    let mut candidates = manifest
        .relays
        .iter()
        .map(|relay| RouteCandidate {
            relay_id: relay.relay_id.clone(),
            region: relay.region.clone(),
            hostname: relay.hostname.clone(),
            priority: relay.priority,
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|candidate| candidate.priority);

    ManifestGateResult::Passed {
        manifest_version: manifest.version.clone(),
        candidates,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        verify_manifest_or_fail, ManifestFailureCode, ManifestGateResult, RelayManifestDto,
        RelayNodeDto,
    };
    use crate::security::signature::AllowlistSignatureVerifier;

    fn sample_manifest(signature_b64: &str, valid_until_unix_s: u64) -> RelayManifestDto {
        RelayManifestDto {
            version: "2026.08.0".to_string(),
            valid_until_unix_s,
            signature_b64: signature_b64.to_string(),
            relays: vec![
                RelayNodeDto {
                    relay_id: "nrt-01".to_string(),
                    region: "nrt".to_string(),
                    hostname: "nrt-01.example.net".to_string(),
                    priority: 2,
                },
                RelayNodeDto {
                    relay_id: "sin-01".to_string(),
                    region: "sin".to_string(),
                    hostname: "sin-01.example.net".to_string(),
                    priority: 1,
                },
            ],
        }
    }

    #[test]
    fn invalid_signature_blocks_route_activation() {
        let verifier = AllowlistSignatureVerifier::new(vec!["known-good".to_string()]);
        let manifest = sample_manifest("unknown", 1_800_000_000);

        let result = verify_manifest_or_fail(&verifier, &manifest, 1_700_000_000);
        assert_eq!(
            result,
            ManifestGateResult::Blocked {
                failure_code: ManifestFailureCode::SignatureInvalid,
            }
        );
        assert_eq!(
            result
                .failure_code()
                .expect("failure code should exist")
                .as_code(),
            "MANIFEST_SIGNATURE_INVALID"
        );
    }

    #[test]
    fn expired_manifest_blocks_route_activation() {
        let verifier = AllowlistSignatureVerifier::new(vec!["known-good".to_string()]);
        let manifest = sample_manifest("known-good", 1_700_000_000);

        let result = verify_manifest_or_fail(&verifier, &manifest, 1_700_000_001);
        assert_eq!(
            result,
            ManifestGateResult::Blocked {
                failure_code: ManifestFailureCode::ManifestExpired,
            }
        );
        assert_eq!(
            result
                .failure_code()
                .expect("failure code should exist")
                .as_code(),
            "MANIFEST_EXPIRED"
        );
    }

    #[test]
    fn valid_manifest_passes_and_exposes_sorted_candidates() {
        let verifier = AllowlistSignatureVerifier::new(vec!["known-good".to_string()]);
        let manifest = sample_manifest("known-good", 1_800_000_000);

        let result = verify_manifest_or_fail(&verifier, &manifest, 1_700_000_000);

        let ManifestGateResult::Passed { candidates, .. } = result else {
            panic!("expected passed gate");
        };

        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].relay_id, "sin-01");
        assert_eq!(candidates[1].relay_id, "nrt-01");
    }
}

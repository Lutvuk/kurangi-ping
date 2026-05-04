//! Routing config validation and policy conversion.

use super::policy::{
    ProtocolPriority, RetryPolicy, RouteProtocol, RoutingPolicy, DEFAULT_PROTOCOL_ORDER,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingConfig {
    pub protocol_priority: Vec<RouteProtocol>,
    pub retry_policy: RetryPolicy,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            protocol_priority: DEFAULT_PROTOCOL_ORDER.to_vec(),
            retry_policy: RetryPolicy::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingConfigError {
    InvalidPriorityLength { expected: usize, actual: usize },
    DuplicateProtocol { protocol: RouteProtocol },
    MissingProtocol { protocol: RouteProtocol },
    RetryAttemptsMustBePositive,
    BaseBackoffMustBePositive,
    MaxBackoffLessThanBase { base_backoff_ms: u64, max_backoff_ms: u64 },
}

impl RoutingConfig {
    pub fn validate(&self) -> Result<(), RoutingConfigError> {
        if self.protocol_priority.len() != DEFAULT_PROTOCOL_ORDER.len() {
            return Err(RoutingConfigError::InvalidPriorityLength {
                expected: DEFAULT_PROTOCOL_ORDER.len(),
                actual: self.protocol_priority.len(),
            });
        }

        for protocol in DEFAULT_PROTOCOL_ORDER {
            let count = self
                .protocol_priority
                .iter()
                .filter(|candidate| **candidate == protocol)
                .count();

            if count == 0 {
                return Err(RoutingConfigError::MissingProtocol { protocol });
            }
            if count > 1 {
                return Err(RoutingConfigError::DuplicateProtocol { protocol });
            }
        }

        if self.retry_policy.max_attempts_per_protocol == 0 {
            return Err(RoutingConfigError::RetryAttemptsMustBePositive);
        }
        if self.retry_policy.base_backoff_ms == 0 {
            return Err(RoutingConfigError::BaseBackoffMustBePositive);
        }
        if self.retry_policy.max_backoff_ms < self.retry_policy.base_backoff_ms {
            return Err(RoutingConfigError::MaxBackoffLessThanBase {
                base_backoff_ms: self.retry_policy.base_backoff_ms,
                max_backoff_ms: self.retry_policy.max_backoff_ms,
            });
        }

        Ok(())
    }

    pub fn into_policy(self) -> Result<RoutingPolicy, RoutingConfigError> {
        self.validate()?;
        let ordered = [
            self.protocol_priority[0],
            self.protocol_priority[1],
            self.protocol_priority[2],
        ];

        Ok(RoutingPolicy {
            protocol_priority: ProtocolPriority::new(ordered),
            retry_policy: self.retry_policy,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{RoutingConfig, RoutingConfigError};
    use crate::routing::{RetryPolicy, RouteProtocol};

    #[test]
    fn default_config_validates_and_converts() {
        let config = RoutingConfig::default();
        let policy = config.into_policy().expect("default config should be valid");

        assert_eq!(
            policy.protocol_order(),
            [
                RouteProtocol::WireGuard,
                RouteProtocol::TcpTls,
                RouteProtocol::Quic,
            ]
        );
    }

    #[test]
    fn validate_rejects_invalid_priority_len() {
        let config = RoutingConfig {
            protocol_priority: vec![RouteProtocol::WireGuard, RouteProtocol::TcpTls],
            retry_policy: RetryPolicy::default(),
        };

        let err = config.validate().expect_err("config should be rejected");
        assert_eq!(
            err,
            RoutingConfigError::InvalidPriorityLength {
                expected: 3,
                actual: 2
            }
        );
    }

    #[test]
    fn validate_rejects_duplicate_protocol() {
        let config = RoutingConfig {
            protocol_priority: vec![
                RouteProtocol::WireGuard,
                RouteProtocol::WireGuard,
                RouteProtocol::Quic,
            ],
            retry_policy: RetryPolicy::default(),
        };

        let err = config.validate().expect_err("duplicate protocol should be rejected");
        assert_eq!(
            err,
            RoutingConfigError::DuplicateProtocol {
                protocol: RouteProtocol::WireGuard
            }
        );
    }

    #[test]
    fn validate_rejects_missing_protocol() {
        let config = RoutingConfig {
            protocol_priority: vec![
                RouteProtocol::WireGuard,
                RouteProtocol::Quic,
                RouteProtocol::Quic,
            ],
            retry_policy: RetryPolicy::default(),
        };

        let err = config.validate().expect_err("missing protocol should be rejected");
        assert_eq!(
            err,
            RoutingConfigError::MissingProtocol {
                protocol: RouteProtocol::TcpTls
            }
        );
    }

    #[test]
    fn validate_rejects_invalid_retry_policy() {
        let config = RoutingConfig {
            protocol_priority: vec![
                RouteProtocol::WireGuard,
                RouteProtocol::TcpTls,
                RouteProtocol::Quic,
            ],
            retry_policy: RetryPolicy {
                max_attempts_per_protocol: 0,
                base_backoff_ms: 250,
                max_backoff_ms: 1_000,
            },
        };

        let err = config.validate().expect_err("retry attempts must be positive");
        assert_eq!(err, RoutingConfigError::RetryAttemptsMustBePositive);
    }
}

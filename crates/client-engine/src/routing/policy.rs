//! Routing policy constants and typed policy model.

/// Ordered protocol preference for route attempts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RouteProtocol {
    WireGuard,
    TcpTls,
    Quic,
}

/// Architecture-approved default protocol order.
pub const DEFAULT_PROTOCOL_ORDER: [RouteProtocol; 3] = [
    RouteProtocol::WireGuard,
    RouteProtocol::TcpTls,
    RouteProtocol::Quic,
];

/// Strongly typed protocol priority list used by routing orchestration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProtocolPriority {
    ordered: [RouteProtocol; 3],
}

impl ProtocolPriority {
    pub const fn new(ordered: [RouteProtocol; 3]) -> Self {
        Self { ordered }
    }

    pub const fn ordered(self) -> [RouteProtocol; 3] {
        self.ordered
    }
}

impl Default for ProtocolPriority {
    fn default() -> Self {
        Self::new(DEFAULT_PROTOCOL_ORDER)
    }
}

/// Bounded retry behavior for protocol failover.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryPolicy {
    pub max_attempts_per_protocol: u8,
    pub base_backoff_ms: u64,
    pub max_backoff_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts_per_protocol: 2,
            base_backoff_ms: 250,
            max_backoff_ms: 3_000,
        }
    }
}

/// Centralized routing policy for protocol order and retry constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RoutingPolicy {
    pub protocol_priority: ProtocolPriority,
    pub retry_policy: RetryPolicy,
}

impl RoutingPolicy {
    pub const fn protocol_order(self) -> [RouteProtocol; 3] {
        self.protocol_priority.ordered()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ProtocolPriority, RetryPolicy, RouteProtocol, RoutingPolicy, DEFAULT_PROTOCOL_ORDER,
    };

    #[test]
    fn default_protocol_order_is_wireguard_tcp_tls_quic() {
        assert_eq!(
            DEFAULT_PROTOCOL_ORDER,
            [
                RouteProtocol::WireGuard,
                RouteProtocol::TcpTls,
                RouteProtocol::Quic,
            ]
        );
    }

    #[test]
    fn routing_policy_defaults_are_centralized() {
        let policy = RoutingPolicy::default();
        assert_eq!(policy.protocol_order(), DEFAULT_PROTOCOL_ORDER);
        assert_eq!(
            policy.retry_policy,
            RetryPolicy {
                max_attempts_per_protocol: 2,
                base_backoff_ms: 250,
                max_backoff_ms: 3_000,
            }
        );
    }

    #[test]
    fn protocol_priority_preserves_order() {
        let priority = ProtocolPriority::new([
            RouteProtocol::TcpTls,
            RouteProtocol::WireGuard,
            RouteProtocol::Quic,
        ]);
        assert_eq!(
            priority.ordered(),
            [
                RouteProtocol::TcpTls,
                RouteProtocol::WireGuard,
                RouteProtocol::Quic,
            ]
        );
    }
}


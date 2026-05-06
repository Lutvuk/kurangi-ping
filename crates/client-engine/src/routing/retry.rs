//! Bounded retry budget and deterministic backoff schedule.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryBudget {
    pub max_retries: u8,
    pub retries_used: u8,
}

impl RetryBudget {
    pub const fn new(max_retries: u8) -> Self {
        Self {
            max_retries,
            retries_used: 0,
        }
    }

    pub const fn remaining(self) -> u8 {
        self.max_retries.saturating_sub(self.retries_used)
    }

    pub const fn exhausted(self) -> bool {
        self.retries_used >= self.max_retries
    }

    pub fn consume_retry(
        &mut self,
        base_backoff_ms: u64,
        max_backoff_ms: u64,
        trigger_code: &str,
    ) -> Option<RetryMetadata> {
        if self.exhausted() {
            return None;
        }

        self.retries_used = self.retries_used.saturating_add(1);
        let delay_ms = next_retry_delay(base_backoff_ms, max_backoff_ms, self.retries_used);
        Some(RetryMetadata {
            retry_index: self.retries_used,
            delay_ms,
            retries_remaining: self.remaining(),
            trigger_code: trigger_code.to_string(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryMetadata {
    pub retry_index: u8,
    pub delay_ms: u64,
    pub retries_remaining: u8,
    pub trigger_code: String,
}

pub fn next_retry_delay(base_backoff_ms: u64, max_backoff_ms: u64, retry_index: u8) -> u64 {
    if retry_index == 0 {
        return 0;
    }

    let multiplier = 2_u64.saturating_pow((retry_index - 1) as u32);
    let computed = base_backoff_ms.saturating_mul(multiplier);
    computed.min(max_backoff_ms)
}

#[cfg(test)]
mod tests {
    use super::{next_retry_delay, RetryBudget};

    #[test]
    fn backoff_schedule_is_deterministic_and_capped() {
        let base = 250;
        let max = 3_000;

        assert_eq!(next_retry_delay(base, max, 0), 0);
        assert_eq!(next_retry_delay(base, max, 1), 250);
        assert_eq!(next_retry_delay(base, max, 2), 500);
        assert_eq!(next_retry_delay(base, max, 3), 1_000);
        assert_eq!(next_retry_delay(base, max, 4), 2_000);
        assert_eq!(next_retry_delay(base, max, 5), 3_000);
        assert_eq!(next_retry_delay(base, max, 6), 3_000);
    }

    #[test]
    fn budget_never_exceeds_max_retries() {
        let mut budget = RetryBudget::new(2);

        let first = budget
            .consume_retry(250, 3_000, "ROUTE_ALL_ATTEMPTS_FAILED")
            .expect("first retry should be available");
        assert_eq!(first.retry_index, 1);
        assert_eq!(first.retries_remaining, 1);

        let second = budget
            .consume_retry(250, 3_000, "ROUTE_ALL_ATTEMPTS_FAILED")
            .expect("second retry should be available");
        assert_eq!(second.retry_index, 2);
        assert_eq!(second.retries_remaining, 0);

        assert!(budget
            .consume_retry(250, 3_000, "ROUTE_ALL_ATTEMPTS_FAILED")
            .is_none());
        assert!(budget.exhausted());
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ProbeObservation {
    pub baseline_ping_ms: Option<f64>,
    pub routed_ping_ms: Option<f64>,
    pub routed_probe_attempted: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PingMetrics {
    pub baseline_ping_ms: f64,
    pub routed_ping_ms: Option<f64>,
    pub jitter_ms: Option<f64>,
    pub packet_loss_pct: Option<f64>,
    pub sample_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricsComputationErrorCode {
    EmptyWindow,
    MissingBaselineSamples,
    InvalidLatencySample,
    InvalidObservationWindow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricsComputationError {
    pub code: MetricsComputationErrorCode,
    pub message: String,
}

impl MetricsComputationError {
    fn new(code: MetricsComputationErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

pub fn compute_ping_metrics(
    observations: &[ProbeObservation],
) -> Result<PingMetrics, MetricsComputationError> {
    if observations.is_empty() {
        return Err(MetricsComputationError::new(
            MetricsComputationErrorCode::EmptyWindow,
            "probe observation window is empty",
        ));
    }

    let mut baseline_samples = Vec::with_capacity(observations.len());
    let mut routed_samples = Vec::with_capacity(observations.len());
    let mut routed_sent = 0usize;

    for (index, observation) in observations.iter().enumerate() {
        if observation.routed_probe_attempted {
            routed_sent = routed_sent.saturating_add(1);
        } else if observation.routed_ping_ms.is_some() {
            return Err(MetricsComputationError::new(
                MetricsComputationErrorCode::InvalidObservationWindow,
                format!(
                    "observation index {} has routed ping without attempted routed probe",
                    index
                ),
            ));
        }

        if let Some(baseline_ms) = observation.baseline_ping_ms {
            validate_latency_sample("baseline_ping_ms", baseline_ms, index)?;
            baseline_samples.push(baseline_ms);
        }

        if let Some(routed_ms) = observation.routed_ping_ms {
            validate_latency_sample("routed_ping_ms", routed_ms, index)?;
            routed_samples.push(routed_ms);
        }
    }

    if baseline_samples.is_empty() {
        return Err(MetricsComputationError::new(
            MetricsComputationErrorCode::MissingBaselineSamples,
            "baseline samples are required to compute ping metrics",
        ));
    }

    let baseline_ping_ms = mean(&baseline_samples);
    let routed_ping_ms = (!routed_samples.is_empty()).then(|| mean(&routed_samples));
    let jitter_ms = if routed_samples.len() >= 2 {
        compute_jitter(&routed_samples)
    } else {
        compute_jitter(&baseline_samples)
    };
    let packet_loss_pct = compute_packet_loss(routed_sent, routed_samples.len());

    Ok(PingMetrics {
        baseline_ping_ms,
        routed_ping_ms,
        jitter_ms,
        packet_loss_pct,
        sample_count: observations.len(),
    })
}

pub fn compute_jitter(latency_samples_ms: &[f64]) -> Option<f64> {
    if latency_samples_ms.len() < 2 {
        return None;
    }

    if latency_samples_ms
        .iter()
        .any(|value| !value.is_finite() || *value < 0.0)
    {
        return None;
    }

    let mut total_delta = 0.0f64;
    for window in latency_samples_ms.windows(2) {
        total_delta += (window[1] - window[0]).abs();
    }

    Some(total_delta / (latency_samples_ms.len().saturating_sub(1) as f64))
}

pub fn compute_packet_loss(total_sent: usize, total_received: usize) -> Option<f64> {
    if total_sent == 0 || total_received > total_sent {
        return None;
    }

    let lost = total_sent.saturating_sub(total_received);
    Some((lost as f64 / total_sent as f64) * 100.0)
}

fn validate_latency_sample(
    field_name: &str,
    value: f64,
    index: usize,
) -> Result<(), MetricsComputationError> {
    if !value.is_finite() || value < 0.0 {
        return Err(MetricsComputationError::new(
            MetricsComputationErrorCode::InvalidLatencySample,
            format!(
                "{field_name} at observation index {} must be finite and >= 0",
                index
            ),
        ));
    }

    Ok(())
}

fn mean(values: &[f64]) -> f64 {
    let sum: f64 = values.iter().sum();
    sum / values.len() as f64
}

#[cfg(test)]
mod tests {
    use super::{
        compute_jitter, compute_packet_loss, compute_ping_metrics, MetricsComputationErrorCode,
        ProbeObservation,
    };
    use serde_json::Value;

    fn obs(
        baseline_ping_ms: Option<f64>,
        routed_ping_ms: Option<f64>,
        routed_probe_attempted: bool,
    ) -> ProbeObservation {
        ProbeObservation {
            baseline_ping_ms,
            routed_ping_ms,
            routed_probe_attempted,
        }
    }

    fn approx_eq(left: f64, right: f64) {
        let delta = (left - right).abs();
        assert!(
            delta < 0.000_001,
            "expected {left} ~= {right}, delta={delta}"
        );
    }

    #[test]
    fn baseline_and_routed_ping_values_are_computed_consistently() {
        let window = vec![
            obs(Some(210.0), Some(154.0), true),
            obs(Some(220.0), Some(158.0), true),
            obs(Some(200.0), Some(152.0), true),
            obs(Some(215.0), None, true),
        ];

        let metrics = compute_ping_metrics(&window).expect("valid window should compute");
        approx_eq(metrics.baseline_ping_ms, 211.25);
        approx_eq(metrics.routed_ping_ms.expect("routed avg should exist"), 154.666_666_666_7);
        approx_eq(metrics.jitter_ms.expect("jitter should exist"), 5.0);
        approx_eq(
            metrics
                .packet_loss_pct
                .expect("packet loss should exist for routed attempts"),
            25.0,
        );
        assert_eq!(metrics.sample_count, 4);
    }

    #[test]
    fn jitter_and_packet_loss_formulas_are_deterministic() {
        let jitter = compute_jitter(&[120.0, 140.0, 100.0, 130.0]).expect("jitter should compute");
        approx_eq(jitter, 30.0);

        let loss_first = compute_packet_loss(12, 9).expect("loss should compute");
        let loss_second = compute_packet_loss(12, 9).expect("loss should be deterministic");
        approx_eq(loss_first, 25.0);
        approx_eq(loss_second, 25.0);
    }

    #[test]
    fn invalid_or_missing_windows_are_handled_safely() {
        let empty = compute_ping_metrics(&[]).expect_err("empty window must fail safely");
        assert_eq!(empty.code, MetricsComputationErrorCode::EmptyWindow);

        let missing_baseline = compute_ping_metrics(&[obs(None, Some(155.0), true)])
            .expect_err("missing baseline must fail safely");
        assert_eq!(
            missing_baseline.code,
            MetricsComputationErrorCode::MissingBaselineSamples
        );

        let invalid_latency = compute_ping_metrics(&[obs(Some(-1.0), Some(155.0), true)])
            .expect_err("negative latency must fail safely");
        assert_eq!(
            invalid_latency.code,
            MetricsComputationErrorCode::InvalidLatencySample
        );

        let invalid_window = compute_ping_metrics(&[obs(Some(210.0), Some(150.0), false)])
            .expect_err("inconsistent routed sample must fail safely");
        assert_eq!(
            invalid_window.code,
            MetricsComputationErrorCode::InvalidObservationWindow
        );

        assert_eq!(compute_packet_loss(0, 0), None);
        assert_eq!(compute_packet_loss(3, 4), None);
        assert_eq!(compute_jitter(&[100.0]), None);
    }

    #[test]
    fn output_format_matches_ui_and_persistence_field_contract() {
        let metrics = compute_ping_metrics(&[
            obs(Some(205.0), Some(150.0), true),
            obs(Some(210.0), Some(155.0), true),
        ])
        .expect("window should compute");

        let payload = serde_json::to_value(metrics).expect("metrics should serialize");
        let object = payload
            .as_object()
            .expect("metrics payload should be serialized as object");

        for key in [
            "baseline_ping_ms",
            "routed_ping_ms",
            "jitter_ms",
            "packet_loss_pct",
        ] {
            assert!(
                object.contains_key(key),
                "serialized metrics payload must contain {key}"
            );
        }

        assert!(matches!(
            payload.get("baseline_ping_ms"),
            Some(Value::Number(_))
        ));
    }
}

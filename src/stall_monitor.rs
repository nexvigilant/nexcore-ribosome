// Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Ribosomal Quality Control (RQC) stall monitor.
//!
//! ## Biology Analog
//!
//! In biology, the RQC pathway detects ribosomes that stall during translation.
//! When a ribosome stops making progress, the RQC complex is recruited to
//! rescue the stalled ribosome and mark the incomplete protein for degradation.
//!
//! ## Purpose
//!
//! Detects pipeline execution stalls through three independent signals:
//! 1. **No Progress**: Same checkpoint repeated without advancement
//! 2. **Circular Execution**: Low Shannon entropy in tool usage (doing the same things)
//! 3. **Confidence Plateau**: Linear regression slope near zero (not learning)
//!
//! ## Primitive Grounding: nu(Frequency) + sigma(Sequence) + N(Quantity)

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

/// Stall detection signals.
///
/// Each variant represents a distinct stall pattern detected by the monitor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StallSignal {
    /// No meaningful progress — same checkpoint repeated.
    NoProgress {
        /// When the stall was first detected.
        stalled_since: DateTime,
        /// The checkpoint name being repeated.
        repeated_checkpoint: String,
        /// How many times it was repeated.
        repetitions: usize,
    },
    /// Circular execution pattern — low Shannon entropy in tool usage.
    CircularExecution {
        /// Observed Shannon entropy (bits).
        entropy: f64,
        /// Threshold that was violated.
        threshold: f64,
        /// Top tool frequencies.
        tool_distribution: Vec<(String, u32)>,
    },
    /// Confidence has plateaued — linear regression slope near zero.
    ConfidencePlateau {
        /// Observed slope of confidence over time.
        slope: f64,
        /// Window size used for detection.
        window_size: usize,
        /// Recent confidence values.
        recent_values: Vec<f64>,
    },
}

/// Configuration for stall detection thresholds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StallConfig {
    /// How many identical checkpoints before NoProgress (default: 3).
    pub no_progress_threshold: usize,
    /// Shannon entropy threshold for circular detection in bits (default: 1.0).
    /// Lower entropy = more repetitive.
    pub entropy_threshold: f64,
    /// Absolute slope threshold for plateau detection (default: 0.01).
    pub plateau_slope_threshold: f64,
    /// Minimum observations before plateau detection activates (default: 5).
    pub plateau_window_size: usize,
}

impl Default for StallConfig {
    fn default() -> Self {
        Self {
            no_progress_threshold: 3,
            entropy_threshold: 1.0,
            plateau_slope_threshold: 0.01,
            plateau_window_size: 5,
        }
    }
}

/// An execution observation fed to the stall detector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionObservation {
    /// Name of the current checkpoint.
    pub checkpoint_name: String,
    /// Tools used since the last observation.
    pub tools_used: Vec<String>,
    /// Current confidence score.
    pub confidence: f64,
    /// When this observation was recorded.
    pub timestamp: DateTime,
}

/// The RQC stall detector — monitors pipeline execution for stall patterns.
///
/// Accumulates observations and detects stalls through three independent channels:
/// no-progress, circular execution, and confidence plateau.
///
/// ## Design Principles
///
/// 1. **Stateful** — accumulates observations over time
/// 2. **Multi-channel** — three independent detection methods
/// 3. **Configurable** — thresholds tunable per use case
/// 4. **Non-blocking** — detection is O(n) in observation window
#[derive(Debug, Clone)]
pub struct StallDetector {
    config: StallConfig,
    history: Vec<ExecutionObservation>,
}

impl Default for StallDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl StallDetector {
    /// Create a stall detector with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: StallConfig::default(),
            history: Vec::new(),
        }
    }

    /// Create a stall detector with custom configuration.
    #[must_use]
    pub fn with_config(config: StallConfig) -> Self {
        Self {
            config,
            history: Vec::new(),
        }
    }

    /// Record a new execution observation.
    pub fn observe(&mut self, obs: ExecutionObservation) {
        self.history.push(obs);
    }

    /// Run all stall detection channels and return any signals.
    #[must_use]
    pub fn detect(&self) -> Vec<StallSignal> {
        let mut signals = Vec::new();

        if let Some(s) = self.detect_no_progress() {
            signals.push(s);
        }
        if let Some(s) = self.detect_circular() {
            signals.push(s);
        }
        if let Some(s) = self.detect_plateau() {
            signals.push(s);
        }

        signals
    }

    /// Clear all accumulated observations.
    pub fn clear(&mut self) {
        self.history.clear();
    }

    /// Number of accumulated observations.
    #[must_use]
    pub fn observation_count(&self) -> usize {
        self.history.len()
    }

    /// Detect no-progress stall: same checkpoint name repeated N times.
    fn detect_no_progress(&self) -> Option<StallSignal> {
        if self.history.len() < self.config.no_progress_threshold {
            return None;
        }

        let recent = &self.history[self.history.len() - self.config.no_progress_threshold..];
        let first_name = &recent[0].checkpoint_name;

        if recent.iter().all(|o| o.checkpoint_name == *first_name) {
            Some(StallSignal::NoProgress {
                stalled_since: recent[0].timestamp,
                repeated_checkpoint: first_name.clone(),
                repetitions: recent.len(),
            })
        } else {
            None
        }
    }

    /// Detect circular execution: low Shannon entropy in recent tool usage.
    fn detect_circular(&self) -> Option<StallSignal> {
        if self.history.len() < self.config.no_progress_threshold {
            return None;
        }

        let mut tool_counts: std::collections::HashMap<&str, u32> =
            std::collections::HashMap::new();
        let window = &self.history[self.history.len().saturating_sub(10)..];

        for obs in window {
            for tool in &obs.tools_used {
                *tool_counts.entry(tool.as_str()).or_insert(0) += 1;
            }
        }

        if tool_counts.is_empty() {
            return None;
        }

        let total: u32 = tool_counts.values().sum();
        let total_f = f64::from(total);

        // Shannon entropy: H = -Sum(p_i * log2(p_i))
        let entropy: f64 = tool_counts
            .values()
            .map(|&count| {
                let p = f64::from(count) / total_f;
                if p > 0.0 { -p * p.log2() } else { 0.0 }
            })
            .sum();

        if entropy < self.config.entropy_threshold {
            let mut distribution: Vec<(String, u32)> = tool_counts
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect();
            distribution.sort_by(|a, b| b.1.cmp(&a.1));

            Some(StallSignal::CircularExecution {
                entropy,
                threshold: self.config.entropy_threshold,
                tool_distribution: distribution,
            })
        } else {
            None
        }
    }

    /// Detect confidence plateau: linear regression slope near zero.
    fn detect_plateau(&self) -> Option<StallSignal> {
        if self.history.len() < self.config.plateau_window_size {
            return None;
        }

        let window = &self.history[self.history.len() - self.config.plateau_window_size..];
        let values: Vec<f64> = window.iter().map(|o| o.confidence).collect();

        let slope = linear_regression_slope(&values);

        if slope.abs() < self.config.plateau_slope_threshold {
            Some(StallSignal::ConfidencePlateau {
                slope,
                window_size: self.config.plateau_window_size,
                recent_values: values,
            })
        } else {
            None
        }
    }
}

/// Compute the slope of a simple linear regression on y-values.
///
/// Uses x = 0, 1, 2, ... as indices.
/// Formula: slope = (n*Sum_xy - Sum_x*Sum_y) / (n*Sum_x2 - (Sum_x)^2)
#[allow(clippy::similar_names)] // sum_x, sum_y, sum_xy are standard regression variable names
fn linear_regression_slope(values: &[f64]) -> f64 {
    #[allow(clippy::cast_precision_loss)] // Acceptable: regression on small observation windows
    let n = values.len() as f64;
    if n < 2.0 {
        return 0.0;
    }

    let mut sum_x: f64 = 0.0;
    let mut sum_y: f64 = 0.0;
    let mut sum_xy: f64 = 0.0;
    let mut sum_x2: f64 = 0.0;

    for (i, &y) in values.iter().enumerate() {
        #[allow(clippy::cast_precision_loss)] // Index values are small
        let x = i as f64;
        sum_x += x;
        sum_y += y;
        sum_xy += x * y;
        sum_x2 += x * x;
    }

    #[allow(clippy::suspicious_operation_groupings)]
    // Correct linear regression formula: n*Σx² - (Σx)²
    let denom = n.mul_add(sum_x2, -(sum_x * sum_x));
    if denom.abs() < f64::EPSILON {
        return 0.0;
    }

    n.mul_add(sum_xy, -(sum_x * sum_y)) / denom
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_obs(name: &str, tools: Vec<&str>, confidence: f64) -> ExecutionObservation {
        ExecutionObservation {
            checkpoint_name: name.to_string(),
            tools_used: tools.into_iter().map(String::from).collect(),
            confidence,
            timestamp: DateTime::now(),
        }
    }

    #[test]
    fn test_no_stall_with_progress() {
        let mut detector = StallDetector::new();
        detector.observe(make_obs("step1", vec!["read"], 0.5));
        detector.observe(make_obs("step2", vec!["write"], 0.6));
        detector.observe(make_obs("step3", vec!["test"], 0.7));
        assert!(detector.detect().is_empty());
    }

    #[test]
    fn test_no_progress_detection() {
        let mut detector = StallDetector::new();
        detector.observe(make_obs("stuck", vec!["read"], 0.5));
        detector.observe(make_obs("stuck", vec!["read"], 0.5));
        detector.observe(make_obs("stuck", vec!["read"], 0.5));
        let signals = detector.detect();
        assert!(
            signals
                .iter()
                .any(|s| matches!(s, StallSignal::NoProgress { .. }))
        );
    }

    #[test]
    fn test_circular_execution_detection() {
        let config = StallConfig {
            entropy_threshold: 2.0,
            ..StallConfig::default()
        };
        let mut detector = StallDetector::with_config(config);
        for _ in 0..5 {
            detector.observe(make_obs("step", vec!["read", "read", "read"], 0.5));
        }
        let signals = detector.detect();
        assert!(
            signals
                .iter()
                .any(|s| matches!(s, StallSignal::CircularExecution { .. }))
        );
    }

    #[test]
    fn test_high_entropy_no_circular() {
        let config = StallConfig {
            entropy_threshold: 0.5,
            ..StallConfig::default()
        };
        let mut detector = StallDetector::with_config(config);
        detector.observe(make_obs("s1", vec!["read", "write", "test", "grep"], 0.5));
        detector.observe(make_obs("s2", vec!["glob", "edit", "bash", "task"], 0.6));
        detector.observe(make_obs(
            "s3",
            vec!["search", "build", "run", "deploy"],
            0.7,
        ));
        let signals = detector.detect();
        assert!(
            !signals
                .iter()
                .any(|s| matches!(s, StallSignal::CircularExecution { .. }))
        );
    }

    #[test]
    fn test_confidence_plateau_detection() {
        let config = StallConfig {
            plateau_window_size: 5,
            plateau_slope_threshold: 0.01,
            ..StallConfig::default()
        };
        let mut detector = StallDetector::with_config(config);
        for _ in 0..5 {
            detector.observe(make_obs("step", vec!["read"], 0.5));
        }
        let signals = detector.detect();
        assert!(
            signals
                .iter()
                .any(|s| matches!(s, StallSignal::ConfidencePlateau { .. }))
        );
    }

    #[test]
    fn test_increasing_confidence_no_plateau() {
        let config = StallConfig {
            plateau_window_size: 5,
            plateau_slope_threshold: 0.01,
            ..StallConfig::default()
        };
        let mut detector = StallDetector::with_config(config);
        for i in 0..5 {
            detector.observe(make_obs("step", vec!["read"], 0.3 + (i as f64 * 0.1)));
        }
        let signals = detector.detect();
        assert!(
            !signals
                .iter()
                .any(|s| matches!(s, StallSignal::ConfidencePlateau { .. }))
        );
    }

    #[test]
    fn test_linear_regression_flat() {
        let values = vec![0.5, 0.5, 0.5, 0.5, 0.5];
        let slope = linear_regression_slope(&values);
        assert!(slope.abs() < 0.001);
    }

    #[test]
    fn test_linear_regression_positive() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let slope = linear_regression_slope(&values);
        assert!((slope - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_linear_regression_single_value() {
        let values = vec![0.5];
        let slope = linear_regression_slope(&values);
        assert!(slope.abs() < f64::EPSILON);
    }

    #[test]
    fn test_clear_resets() {
        let mut detector = StallDetector::new();
        detector.observe(make_obs("step", vec!["read"], 0.5));
        assert_eq!(detector.observation_count(), 1);
        detector.clear();
        assert_eq!(detector.observation_count(), 0);
    }

    #[test]
    fn test_insufficient_data_no_signals() {
        let detector = StallDetector::new();
        assert!(detector.detect().is_empty());
    }
}

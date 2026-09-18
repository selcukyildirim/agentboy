use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub counters: HashMap<String, u64>,
    pub gauges: HashMap<String, f64>,
    pub histograms: HashMap<String, Vec<f64>>,
}

impl Metrics {
    #[must_use]
    pub fn new() -> Self {
        Self {
            counters: HashMap::new(),
            gauges: HashMap::new(),
            histograms: HashMap::new(),
        }
    }

    pub fn increment_counter(&mut self, name: &str, value: u64) {
        *self.counters.entry(name.to_string()).or_insert(0) += value;
    }

    pub fn set_gauge(&mut self, name: &str, value: f64) {
        self.gauges.insert(name.to_string(), value);
    }

    pub fn record_histogram(&mut self, name: &str, value: f64) {
        self.histograms
            .entry(name.to_string())
            .or_default()
            .push(value);
    }

    #[must_use]
    pub fn get_counter(&self, name: &str) -> u64 {
        self.counters.get(name).copied().unwrap_or(0)
    }

    #[must_use]
    pub fn get_gauge(&self, name: &str) -> f64 {
        self.gauges.get(name).copied().unwrap_or(0.0)
    }

    #[must_use]
    pub fn get_histogram_stats(&self, name: &str) -> Option<HistogramStats> {
        let values = self.histograms.get(name)?;
        if values.is_empty() {
            return None;
        }

        let mut sorted = values.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let sum: f64 = sorted.iter().sum();
        let count = sorted.len();
        let mean = sum / count as f64;
        let min = sorted[0];
        let max = sorted[count - 1];
        let p50 = sorted[count / 2];
        let p95 = sorted[(count as f64 * 0.95) as usize];
        let p99 = sorted[(count as f64 * 0.99) as usize];

        Some(HistogramStats {
            count,
            mean,
            min,
            max,
            p50,
            p95,
            p99,
        })
    }

    #[must_use]
    pub fn export_json(&self) -> serde_json::Value {
        serde_json::json!({
            "counters": self.counters,
            "gauges": self.gauges,
            "histograms": self.histograms.keys().collect::<Vec<_>>()
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramStats {
    pub count: usize,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics() {
        let mut metrics = Metrics::new();
        metrics.increment_counter("requests", 1);
        metrics.increment_counter("requests", 5);
        assert_eq!(metrics.get_counter("requests"), 6);

        metrics.set_gauge("cpu_usage", 0.75);
        assert_eq!(metrics.get_gauge("cpu_usage"), 0.75);

        metrics.record_histogram("latency", 10.0);
        metrics.record_histogram("latency", 20.0);
        metrics.record_histogram("latency", 30.0);

        let stats = metrics.get_histogram_stats("latency").unwrap();
        assert_eq!(stats.count, 3);
        assert!((stats.mean - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_counter_default_zero() {
        let metrics = Metrics::new();
        assert_eq!(metrics.get_counter("nonexistent"), 0);
    }

    #[test]
    fn test_gauge_default_zero() {
        let metrics = Metrics::new();
        assert_eq!(metrics.get_gauge("nonexistent"), 0.0);
    }

    #[test]
    fn test_histogram_nonexistent_none() {
        let metrics = Metrics::new();
        assert!(metrics.get_histogram_stats("nonexistent").is_none());
    }

    #[test]
    fn test_histogram_empty_none() {
        let mut metrics = Metrics::new();
        metrics.histograms.insert("empty".to_string(), vec![]);
        assert!(metrics.get_histogram_stats("empty").is_none());
    }

    #[test]
    fn test_histogram_percentiles() {
        let mut metrics = Metrics::new();
        for i in 1..=100 {
            metrics.record_histogram("latency", f64::from(i));
        }

        let stats = metrics.get_histogram_stats("latency").unwrap();
        assert_eq!(stats.count, 100);
        assert!((stats.min - 1.0).abs() < 0.01);
        assert!((stats.max - 100.0).abs() < 0.01);
        assert!((stats.mean - 50.5).abs() < 0.01);
        assert!(stats.p50 >= 50.0);
        assert!(stats.p95 >= 95.0);
        assert!(stats.p99 >= 99.0);
    }

    #[test]
    fn test_export_json() {
        let mut metrics = Metrics::new();
        metrics.increment_counter("test", 1);
        metrics.set_gauge("test_gauge", 1.0);

        let json = metrics.export_json();
        assert!(json.is_object());
    }

    #[test]
    fn test_metrics_default() {
        let metrics = Metrics::default();
        assert!(metrics.counters.is_empty());
        assert!(metrics.gauges.is_empty());
        assert!(metrics.histograms.is_empty());
    }
}

//! ASI Analytics — enterprise data lake with BI, trends, predictive insights.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A usage data point collected over time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePoint {
    pub timestamp: u64,
    pub tokens_used: u64,
    pub requests: u64,
    pub errors: u64,
    pub active_users: u64,
    pub avg_latency_ms: f64,
}

/// Aggregated analytics summary.
#[derive(Debug, Clone, Serialize)]
pub struct AnalyticsSummary {
    pub period: String,
    pub total_requests: u64,
    pub total_tokens: u64,
    pub total_errors: u64,
    pub error_rate: f64,
    pub avg_latency_ms: f64,
    pub peak_rps: f64,
    pub active_users: u64,
    pub trend: TrendDirection,
    pub insights: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TrendDirection { Improving, Stable, Degrading }

/// Enterprise data lake — collects and analyzes usage data.
pub struct DataLake {
    points: std::sync::Mutex<Vec<UsagePoint>>,
}

impl DataLake {
    pub fn new() -> Self { Self { points: std::sync::Mutex::new(Vec::new()) } }

    /// Record a usage data point.
    pub fn record(&self, point: UsagePoint) {
        self.points.lock().unwrap().push(point);
    }

    /// Generate an analytics summary for the collected data.
    pub fn summarize(&self) -> AnalyticsSummary {
        let points = self.points.lock().unwrap();
        if points.is_empty() { return AnalyticsSummary { period:"no data".into(), total_requests:0, total_tokens:0, total_errors:0, error_rate:0.0, avg_latency_ms:0.0, peak_rps:0.0, active_users:0, trend:TrendDirection::Stable, insights:vec!["No data collected yet".into()] }; }

        let total_requests: u64 = points.iter().map(|p| p.requests).sum();
        let total_tokens: u64 = points.iter().map(|p| p.tokens_used).sum();
        let total_errors: u64 = points.iter().map(|p| p.errors).sum();
        let avg_latency: f64 = points.iter().map(|p| p.avg_latency_ms).sum::<f64>() / points.len() as f64;
        let peak = points.iter().map(|p| p.requests as f64 / 60.0).fold(0.0f64, f64::max);
        let active = points.last().map(|p| p.active_users).unwrap_or(0);

        let error_rate = if total_requests > 0 { total_errors as f64 / total_requests as f64 } else { 0.0 };

        // Trend analysis: compare first half vs second half.
        let mid = points.len() / 2;
        let first_half_rps: f64 = points[..mid.max(1)].iter().map(|p| p.requests as f64).sum::<f64>() / mid.max(1) as f64;
        let second_half_rps: f64 = points[mid..].iter().map(|p| p.requests as f64).sum::<f64>() / (points.len() - mid).max(1) as f64;
        let trend = if second_half_rps > first_half_rps * 1.1 { TrendDirection::Improving }
            else if second_half_rps < first_half_rps * 0.9 { TrendDirection::Degrading }
            else { TrendDirection::Stable };

        let mut insights = Vec::new();
        if error_rate > 0.05 { insights.push(format!("High error rate: {:.1}%", error_rate * 100.0)); }
        if avg_latency > 5000.0 { insights.push(format!("High latency: {:.0}ms avg — consider scaling", avg_latency)); }
        if trend == TrendDirection::Improving { insights.push("Usage trending upward — capacity planning recommended".into()); }
        if total_tokens > 1_000_000 { insights.push("High token usage — review caching strategy".into()); }

        AnalyticsSummary { period: format!("{} data points", points.len()), total_requests, total_tokens, total_errors, error_rate, avg_latency_ms: avg_latency, peak_rps: peak, active_users: active, trend, insights }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_lake() {
        let lake = DataLake::new();
        let summary = lake.summarize();
        assert_eq!(summary.total_requests, 0);
    }

    #[test]
    fn test_record_and_summarize() {
        let lake = DataLake::new();
        for i in 0..10 {
            lake.record(UsagePoint { timestamp:i, tokens_used:100, requests:50, errors:if i==5{5}else{0}, active_users:3, avg_latency_ms:500.0 });
        }
        let summary = lake.summarize();
        assert_eq!(summary.total_requests, 500);
        assert_eq!(summary.total_tokens, 1000);
        assert!(summary.error_rate > 0.0);
    }
}

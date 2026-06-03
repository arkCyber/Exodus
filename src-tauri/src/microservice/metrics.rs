//! Metrics Collection and Aggregation
//!
//! Collect and aggregate metrics from microservices

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// Metric type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

/// Metric value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    pub metric_type: MetricType,
    pub value: f64,
    pub timestamp: u64,
    pub labels: HashMap<String, String>,
}

/// Histogram bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramBucket {
    pub le: f64, // less than or equal
    pub count: u64,
}

/// Histogram data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Histogram {
    pub buckets: Vec<HistogramBucket>,
    pub sum: f64,
    pub count: u64,
}

impl Histogram {
    pub fn new() -> Self {
        Self {
            buckets: vec![
                HistogramBucket { le: 0.005, count: 0 },
                HistogramBucket { le: 0.01, count: 0 },
                HistogramBucket { le: 0.025, count: 0 },
                HistogramBucket { le: 0.05, count: 0 },
                HistogramBucket { le: 0.1, count: 0 },
                HistogramBucket { le: 0.25, count: 0 },
                HistogramBucket { le: 0.5, count: 0 },
                HistogramBucket { le: 1.0, count: 0 },
                HistogramBucket { le: 2.5, count: 0 },
                HistogramBucket { le: 5.0, count: 0 },
                HistogramBucket { le: 10.0, count: 0 },
                HistogramBucket { le: f64::INFINITY, count: 0 },
            ],
            sum: 0.0,
            count: 0,
        }
    }

    pub fn observe(&mut self, value: f64) {
        self.sum += value;
        self.count += 1;
        
        for bucket in &mut self.buckets {
            if value <= bucket.le {
                bucket.count += 1;
            }
        }
    }

    pub fn percentile(&self, p: f64) -> f64 {
        if self.count == 0 {
            return 0.0;
        }
        
        let target_count = (self.count as f64 * p / 100.0).ceil() as u64;
        
        for bucket in &self.buckets {
            if bucket.count >= target_count {
                return bucket.le;
            }
        }
        
        f64::INFINITY
    }
}

impl Default for Histogram {
    fn default() -> Self {
        Self::new()
    }
}

/// Metric entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub metric_type: MetricType,
    pub help: String,
    pub values: Vec<MetricValue>,
}

/// Metrics collector
pub struct MetricsCollector {
    metrics: Arc<RwLock<HashMap<String, Metric>>>,
    histograms: Arc<RwLock<HashMap<String, Histogram>>>,
    retention_duration: Duration,
    max_values_per_metric: usize,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            histograms: Arc::new(RwLock::new(HashMap::new())),
            retention_duration: Duration::from_secs(3600), // 1 hour
            max_values_per_metric: 1000,
        }
    }

    pub fn with_retention(mut self, duration: Duration) -> Self {
        self.retention_duration = duration;
        self
    }

    pub fn with_max_values(mut self, max: usize) -> Self {
        self.max_values_per_metric = max;
        self
    }

    /// Record a counter metric
    pub async fn counter(&self, name: impl Into<String>, value: f64, labels: HashMap<String, String>) {
        let name = name.into();
        let mut metrics = self.metrics.write().await;
        
        let metric = metrics.entry(name.clone()).or_insert_with(|| Metric {
            name: name.clone(),
            metric_type: MetricType::Counter,
            help: format!("Counter metric: {}", name),
            values: Vec::new(),
        });
        
        metric.values.push(MetricValue {
            metric_type: MetricType::Counter,
            value,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs(),
            labels,
        });
        
        // Limit values
        if metric.values.len() > self.max_values_per_metric {
            metric.values.remove(0);
        }
    }

    /// Record a gauge metric
    pub async fn gauge(&self, name: impl Into<String>, value: f64, labels: HashMap<String, String>) {
        let name = name.into();
        let mut metrics = self.metrics.write().await;
        
        let metric = metrics.entry(name.clone()).or_insert_with(|| Metric {
            name: name.clone(),
            metric_type: MetricType::Gauge,
            help: format!("Gauge metric: {}", name),
            values: Vec::new(),
        });
        
        metric.values.push(MetricValue {
            metric_type: MetricType::Gauge,
            value,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs(),
            labels,
        });
        
        // Limit values
        if metric.values.len() > self.max_values_per_metric {
            metric.values.remove(0);
        }
    }

    /// Record a histogram observation
    pub async fn histogram(&self, name: impl Into<String>, value: f64) {
        let name = name.into();
        let mut histograms = self.histograms.write().await;
        
        let histogram = histograms.entry(name).or_insert_with(Histogram::new);
        histogram.observe(value);
    }

    /// Get a metric by name
    pub async fn get_metric(&self, name: &str) -> Option<Metric> {
        let metrics = self.metrics.read().await;
        metrics.get(name).cloned()
    }

    /// Get histogram by name
    pub async fn get_histogram(&self, name: &str) -> Option<Histogram> {
        let histograms = self.histograms.read().await;
        histograms.get(name).cloned()
    }

    /// List all metric names
    pub async fn list_metrics(&self) -> Vec<String> {
        let metrics = self.metrics.read().await;
        metrics.keys().cloned().collect()
    }

    /// Get all metrics
    pub async fn get_all_metrics(&self) -> Vec<Metric> {
        let metrics = self.metrics.read().await;
        metrics.values().cloned().collect()
    }

    /// Get metrics statistics
    pub async fn get_stats(&self) -> MetricsStats {
        let metrics = self.metrics.read().await;
        let histograms = self.histograms.read().await;
        
        let total_metrics = metrics.len();
        let total_histograms = histograms.len();
        let total_data_points: usize = metrics.values().map(|m| m.values.len()).sum();
        
        MetricsStats {
            total_metrics,
            total_histograms,
            total_data_points,
        }
    }

    /// Clean up old metric values
    pub async fn cleanup_old_values(&self) -> usize {
        let mut metrics = self.metrics.write().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        let retention_secs = self.retention_duration.as_secs();
        let mut removed_count = 0;
        
        for metric in metrics.values_mut() {
            let initial_len = metric.values.len();
            metric.values.retain(|v| now - v.timestamp < retention_secs);
            removed_count += initial_len - metric.values.len();
        }
        
        // Remove metrics with no values
        metrics.retain(|_, m| !m.values.is_empty());
        
        removed_count
    }

    /// Export metrics in Prometheus format
    pub async fn export_prometheus(&self) -> String {
        let metrics = self.metrics.read().await;
        let histograms = self.histograms.read().await;
        
        let mut output = String::new();
        
        // Export regular metrics
        for metric in metrics.values() {
            output.push_str(&format!("# HELP {} {}\n", metric.name, metric.help));
            output.push_str(&format!("# TYPE {} {:?}\n", metric.name, metric.metric_type).to_lowercase());
            
            for value in &metric.values {
                let labels = if value.labels.is_empty() {
                    String::new()
                } else {
                    let label_str: Vec<String> = value.labels
                        .iter()
                        .map(|(k, v)| format!("{}=\"{}\"", k, v))
                        .collect();
                    format!("{{{}}}", label_str.join(","))
                };
                
                output.push_str(&format!("{}{} {} {}\n", 
                    metric.name, labels, value.value, value.timestamp * 1000));
            }
            output.push('\n');
        }
        
        // Export histograms
        for (name, histogram) in histograms.iter() {
            output.push_str(&format!("# HELP {} Histogram\n", name));
            output.push_str(&format!("# TYPE {} histogram\n", name));
            
            for bucket in &histogram.buckets {
                output.push_str(&format!("{}_bucket{{le=\"{}\"}} {}\n", 
                    name, bucket.le, bucket.count));
            }
            
            output.push_str(&format!("{}_sum {}\n", name, histogram.sum));
            output.push_str(&format!("{}_count {}\n", name, histogram.count));
            output.push('\n');
        }
        
        output
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsStats {
    pub total_metrics: usize,
    pub total_histograms: usize,
    pub total_data_points: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_counter_metric() {
        let collector = MetricsCollector::new();
        let mut labels = HashMap::new();
        labels.insert("service".to_string(), "test".to_string());
        
        collector.counter("requests_total", 1.0, labels).await;
        
        let metric = collector.get_metric("requests_total").await;
        assert!(metric.is_some());
        
        let metric = metric.expect("Expected metric");
        assert_eq!(metric.metric_type, MetricType::Counter);
        assert_eq!(metric.values.len(), 1);
    }

    #[tokio::test]
    async fn test_histogram() {
        let collector = MetricsCollector::new();
        
        collector.histogram("request_duration", 0.1).await;
        collector.histogram("request_duration", 0.5).await;
        collector.histogram("request_duration", 1.0).await;
        
        let histogram = collector.get_histogram("request_duration").await;
        assert!(histogram.is_some());
        
        let histogram = histogram.expect("Expected histogram");
        assert_eq!(histogram.count, 3);
        assert_eq!(histogram.sum, 1.6);
    }
}

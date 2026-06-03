//! Distributed Tracing for Request Tracking
//!
//! Track requests across microservices for debugging and performance analysis

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Trace context for propagating across services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceContext {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub sampled: bool,
}

impl TraceContext {
    pub fn new() -> Self {
        Self {
            trace_id: Uuid::new_v4().to_string(),
            span_id: Uuid::new_v4().to_string(),
            parent_span_id: None,
            sampled: true,
        }
    }

    pub fn child_span(&self) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            span_id: Uuid::new_v4().to_string(),
            parent_span_id: Some(self.span_id.clone()),
            sampled: self.sampled,
        }
    }
}

impl Default for TraceContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Span represents a unit of work in a trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub service_name: String,
    pub operation_name: String,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub duration_ms: Option<u64>,
    pub tags: HashMap<String, String>,
    pub logs: Vec<SpanLog>,
    pub status: SpanStatus,
}

/// Span status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpanStatus {
    Ok,
    Error,
    Unknown,
}

/// Span log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanLog {
    pub timestamp: u64,
    pub message: String,
    pub level: LogLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl Span {
    pub fn new(
        context: &TraceContext,
        service_name: impl Into<String>,
        operation_name: impl Into<String>,
    ) -> Self {
        Self {
            trace_id: context.trace_id.clone(),
            span_id: context.span_id.clone(),
            parent_span_id: context.parent_span_id.clone(),
            service_name: service_name.into(),
            operation_name: operation_name.into(),
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_millis() as u64,
            end_time: None,
            duration_ms: None,
            tags: HashMap::new(),
            logs: Vec::new(),
            status: SpanStatus::Unknown,
        }
    }

    pub fn add_tag(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.tags.insert(key.into(), value.into());
    }

    pub fn add_log(&mut self, message: impl Into<String>, level: LogLevel) {
        self.logs.push(SpanLog {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_millis() as u64,
            message: message.into(),
            level,
        });
    }

    pub fn finish(&mut self, status: SpanStatus) {
        let end_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        self.end_time = Some(end_time);
        self.duration_ms = Some(end_time - self.start_time);
        self.status = status;
    }
}

/// Trace represents a complete request flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    pub trace_id: String,
    pub spans: Vec<Span>,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub total_duration_ms: Option<u64>,
    pub service_count: usize,
}

/// Distributed tracing collector
pub struct TracingCollector {
    traces: Arc<RwLock<HashMap<String, Vec<Span>>>>,
    max_traces: usize,
    retention_duration: Duration,
}

impl TracingCollector {
    pub fn new() -> Self {
        Self {
            traces: Arc::new(RwLock::new(HashMap::new())),
            max_traces: 1000,
            retention_duration: Duration::from_secs(3600), // 1 hour
        }
    }

    pub fn with_max_traces(mut self, max: usize) -> Self {
        self.max_traces = max;
        self
    }

    pub fn with_retention(mut self, duration: Duration) -> Self {
        self.retention_duration = duration;
        self
    }

    /// Record a span
    pub async fn record_span(&self, span: Span) {
        let mut traces = self.traces.write().await;
        
        let spans = traces
            .entry(span.trace_id.clone())
            .or_insert_with(Vec::new);
        
        spans.push(span);
        
        // Limit number of traces
        if traces.len() > self.max_traces {
            // Remove oldest trace
            if let Some(oldest_key) = traces.keys().next().cloned() {
                traces.remove(&oldest_key);
            }
        }
    }

    /// Get a complete trace
    pub async fn get_trace(&self, trace_id: &str) -> Option<Trace> {
        let traces = self.traces.read().await;
        
        traces.get(trace_id).map(|spans| {
            let start_time = spans.iter().map(|s| s.start_time).min().unwrap_or(0);
            let end_time = spans.iter().filter_map(|s| s.end_time).max();
            let total_duration_ms = end_time.map(|end| end - start_time);
            
            let mut services = std::collections::HashSet::new();
            for span in spans {
                services.insert(span.service_name.clone());
            }
            
            Trace {
                trace_id: trace_id.to_string(),
                spans: spans.clone(),
                start_time,
                end_time,
                total_duration_ms,
                service_count: services.len(),
            }
        })
    }

    /// List all trace IDs
    pub async fn list_traces(&self) -> Vec<String> {
        let traces = self.traces.read().await;
        traces.keys().cloned().collect()
    }

    /// Get trace statistics
    pub async fn get_stats(&self) -> TracingStats {
        let traces = self.traces.read().await;
        
        let total_traces = traces.len();
        let total_spans: usize = traces.values().map(|spans| spans.len()).sum();
        
        let mut durations: Vec<u64> = Vec::new();
        for spans in traces.values() {
            if let Some(min_start) = spans.iter().map(|s| s.start_time).min() {
                if let Some(max_end) = spans.iter().filter_map(|s| s.end_time).max() {
                    durations.push(max_end - min_start);
                }
            }
        }
        
        let avg_duration_ms = if !durations.is_empty() {
            durations.iter().sum::<u64>() / durations.len() as u64
        } else {
            0
        };
        
        TracingStats {
            total_traces,
            total_spans,
            avg_duration_ms,
        }
    }

    /// Clean up old traces
    pub async fn cleanup_old_traces(&self) -> usize {
        let mut traces = self.traces.write().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        let retention_ms = self.retention_duration.as_millis() as u64;
        let initial_count = traces.len();
        
        traces.retain(|_, spans| {
            if let Some(latest_time) = spans.iter().filter_map(|s| s.end_time).max() {
                now - latest_time < retention_ms
            } else {
                // Keep traces with unfinished spans
                true
            }
        });
        
        initial_count - traces.len()
    }

    /// Search traces by service name
    pub async fn search_by_service(&self, service_name: &str) -> Vec<String> {
        let traces = self.traces.read().await;
        
        traces
            .iter()
            .filter(|(_, spans)| {
                spans.iter().any(|s| s.service_name == service_name)
            })
            .map(|(trace_id, _)| trace_id.clone())
            .collect()
    }

    /// Search traces by operation name
    pub async fn search_by_operation(&self, operation_name: &str) -> Vec<String> {
        let traces = self.traces.read().await;
        
        traces
            .iter()
            .filter(|(_, spans)| {
                spans.iter().any(|s| s.operation_name == operation_name)
            })
            .map(|(trace_id, _)| trace_id.clone())
            .collect()
    }
}

impl Default for TracingCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Tracing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingStats {
    pub total_traces: usize,
    pub total_spans: usize,
    pub avg_duration_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_trace_recording() {
        let collector = TracingCollector::new();
        let context = TraceContext::new();
        
        let mut span = Span::new(&context, "test-service", "test-operation");
        span.add_tag("key", "value");
        span.finish(SpanStatus::Ok);
        
        collector.record_span(span).await;
        
        let trace = collector.get_trace(&context.trace_id).await;
        assert!(trace.is_some());
        
        let trace = trace.expect("Expected trace");
        assert_eq!(trace.spans.len(), 1);
        assert_eq!(trace.service_count, 1);
    }

    #[tokio::test]
    async fn test_child_spans() {
        let collector = TracingCollector::new();
        let parent_context = TraceContext::new();
        let child_context = parent_context.child_span();
        
        let mut parent_span = Span::new(&parent_context, "service-a", "operation-a");
        parent_span.finish(SpanStatus::Ok);
        
        let mut child_span = Span::new(&child_context, "service-b", "operation-b");
        child_span.finish(SpanStatus::Ok);
        
        collector.record_span(parent_span).await;
        collector.record_span(child_span).await;
        
        let trace = collector.get_trace(&parent_context.trace_id).await.expect("Expected trace");
        assert_eq!(trace.spans.len(), 2);
        assert_eq!(trace.service_count, 2);
    }
}

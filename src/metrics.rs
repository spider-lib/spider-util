//! # Metrics Utilities
//!
//! Common metrics-related utilities and structures for the spider framework.

use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{Duration, Instant};

// Thread-safe exponential moving average for tracking recent rates
#[derive(Debug)]
pub struct ExpMovingAverage {
    alpha: f64,
    rate: Arc<RwLock<f64>>,
    last_update: Arc<RwLock<Instant>>,
    event_count: Arc<RwLock<usize>>,
}

impl ExpMovingAverage {
    pub fn new(alpha: f64) -> Self {
        ExpMovingAverage {
            alpha,
            rate: Arc::new(RwLock::new(0.0)),
            last_update: Arc::new(RwLock::new(Instant::now())),
            event_count: Arc::new(RwLock::new(0)),
        }
    }

    pub fn update(&self, count: usize) {
        let now = Instant::now();
        let mut last_update = self.last_update.write();
        let mut event_count = self.event_count.write();

        *event_count += count;
        let time_delta = now.duration_since(*last_update).as_secs_f64();

        // Update rate every second or so to prevent excessive computation
        if time_delta >= 1.0 {
            let current_rate = *event_count as f64 / time_delta;
            let mut rate = self.rate.write();
            // Apply exponential moving average formula
            *rate = self.alpha * current_rate + (1.0 - self.alpha) * (*rate);

            // Reset for next interval
            *event_count = 0;
            *last_update = now;
        }
    }

    pub fn get_rate(&self) -> f64 {
        *self.rate.read()
    }
}

/// Trait for formatting duration values
pub trait DurationFormatter {
    fn formatted_duration(&self, duration: Duration) -> String;
    fn formatted_request_time(&self, duration: Option<Duration>) -> String;
}

/// Default implementation for duration formatting
pub struct DefaultDurationFormatter;

impl DurationFormatter for DefaultDurationFormatter {
    fn formatted_duration(&self, duration: Duration) -> String {
        format!("{:?}", duration)
    }

    fn formatted_request_time(&self, duration: Option<Duration>) -> String {
        match duration {
            Some(d) => {
                if d.as_millis() < 1000 {
                    format!("{} ms", d.as_millis())
                } else {
                    format!("{:.2} s", d.as_secs_f64())
                }
            }
            None => "N/A".to_string(),
        }
    }
}

/// Trait for formatting byte values
pub trait ByteFormatter {
    fn formatted_bytes(&self, bytes: usize) -> String;
}

/// Default implementation for byte formatting
pub struct DefaultByteFormatter;

impl ByteFormatter for DefaultByteFormatter {
    fn formatted_bytes(&self, bytes: usize) -> String {
        const KB: usize = 1024;
        const MB: usize = 1024 * KB;
        const GB: usize = 1024 * MB;

        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }
}

/// Trait for calculating rates
pub trait RateCalculator {
    fn calculate_rate(&self, count: usize, elapsed: Duration) -> f64;
}

/// Default implementation for rate calculation
pub struct DefaultRateCalculator;

impl RateCalculator for DefaultRateCalculator {
    fn calculate_rate(&self, count: usize, elapsed: Duration) -> f64 {
        let elapsed = elapsed.as_secs_f64();
        if elapsed > 0.0 {
            count as f64 / elapsed
        } else {
            0.0
        }
    }
}

// Snapshot of statistics for reporting purposes
#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricsSnapshot {
    pub requests_enqueued: usize,
    pub requests_sent: usize,
    pub requests_succeeded: usize,
    pub requests_failed: usize,
    pub requests_retried: usize,
    pub requests_dropped: usize,
    pub responses_received: usize,
    pub responses_from_cache: usize,
    pub total_bytes_downloaded: usize,
    pub items_scraped: usize,
    pub items_processed: usize,
    pub items_dropped_by_pipeline: usize,
    pub response_status_counts: std::collections::HashMap<u16, usize>,
    pub elapsed_duration: Duration,
    pub average_request_time: Option<Duration>,
    pub fastest_request_time: Option<Duration>,
    pub slowest_request_time: Option<Duration>,
    pub request_time_count: usize,
    pub average_parsing_time: Option<Duration>,
    pub fastest_parsing_time: Option<Duration>,
    pub slowest_parsing_time: Option<Duration>,
    pub parsing_time_count: usize,
    pub recent_requests_per_second: f64,
    pub recent_responses_per_second: f64,
    pub recent_items_per_second: f64,
}

impl MetricsSnapshot {
    pub fn formatted_duration(&self) -> String {
        DefaultDurationFormatter.formatted_duration(self.elapsed_duration)
    }

    pub fn formatted_request_time(&self, duration: Option<Duration>) -> String {
        DefaultDurationFormatter.formatted_request_time(duration)
    }

    pub fn requests_per_second(&self) -> f64 {
        DefaultRateCalculator.calculate_rate(self.requests_sent, self.elapsed_duration)
    }

    pub fn responses_per_second(&self) -> f64 {
        DefaultRateCalculator.calculate_rate(self.responses_received, self.elapsed_duration)
    }

    pub fn items_per_second(&self) -> f64 {
        DefaultRateCalculator.calculate_rate(self.items_scraped, self.elapsed_duration)
    }

    pub fn formatted_bytes(&self) -> String {
        DefaultByteFormatter.formatted_bytes(self.total_bytes_downloaded)
    }
}

// Trait for creating snapshots from metric collectors
pub trait SnapshotProvider {
    type Snapshot: Clone;
    fn create_snapshot(&self) -> Self::Snapshot;
}

// Trait for exporting metrics in different formats
pub trait MetricsExporter<T> {
    fn to_json_string(&self) -> Result<String, crate::error::SpiderError>;
    fn to_json_string_pretty(&self) -> Result<String, crate::error::SpiderError>;
    fn to_markdown_string(&self) -> String;
    fn to_display_string(&self) -> String;
}

// Default implementation for displaying metrics
pub struct MetricsDisplayFormatter;

impl MetricsDisplayFormatter {
    pub fn format_metrics<T: MetricsSnapshotProvider>(&self, snapshot: &T) -> String {
        format!(
            "\nCrawl Statistics\n----------------\n  duration : {}\n  speed    : req/s: {:.2}, resp/s: {:.2}, item/s: {:.2}\n  requests : enqueued: {}, sent: {}, ok: {}, fail: {}, retry: {}, drop: {}\n  response : received: {}, from_cache: {}, downloaded: {}\n  items    : scraped: {}, processed: {}, dropped: {}\n  req time : avg: {}, fastest: {}, slowest: {}, total: {}\n  parsing  : avg: {}, fastest: {}, slowest: {}, total: {}\n  status   : {}\n",
            snapshot.formatted_duration(),
            snapshot.get_recent_requests_per_second(),
            snapshot.get_recent_responses_per_second(),
            snapshot.get_recent_items_per_second(),
            snapshot.get_requests_enqueued(),
            snapshot.get_requests_sent(),
            snapshot.get_requests_succeeded(),
            snapshot.get_requests_failed(),
            snapshot.get_requests_retried(),
            snapshot.get_requests_dropped(),
            snapshot.get_responses_received(),
            snapshot.get_responses_from_cache(),
            snapshot.formatted_bytes(),
            snapshot.get_items_scraped(),
            snapshot.get_items_processed(),
            snapshot.get_items_dropped_by_pipeline(),
            snapshot.formatted_request_time(snapshot.get_average_request_time()),
            snapshot.formatted_request_time(snapshot.get_fastest_request_time()),
            snapshot.formatted_request_time(snapshot.get_slowest_request_time()),
            snapshot.get_request_time_count(),
            snapshot.formatted_request_time(snapshot.get_average_parsing_time()),
            snapshot.formatted_request_time(snapshot.get_fastest_parsing_time()),
            snapshot.formatted_request_time(snapshot.get_slowest_parsing_time()),
            snapshot.get_parsing_time_count(),
            if snapshot.get_response_status_counts().is_empty() {
                "none".to_string()
            } else {
                snapshot
                    .get_response_status_counts()
                    .iter()
                    .map(|(code, count)| format!("{}: {}", code, count))
                    .collect::<Vec<String>>()
                    .join(", ")
            }
        )
    }
}

// Trait for metrics that can provide snapshot data
pub trait MetricsSnapshotProvider {
    fn get_requests_enqueued(&self) -> usize;
    fn get_requests_sent(&self) -> usize;
    fn get_requests_succeeded(&self) -> usize;
    fn get_requests_failed(&self) -> usize;
    fn get_requests_retried(&self) -> usize;
    fn get_requests_dropped(&self) -> usize;
    fn get_responses_received(&self) -> usize;
    fn get_responses_from_cache(&self) -> usize;
    fn get_total_bytes_downloaded(&self) -> usize;
    fn get_items_scraped(&self) -> usize;
    fn get_items_processed(&self) -> usize;
    fn get_items_dropped_by_pipeline(&self) -> usize;
    fn get_response_status_counts(&self) -> &std::collections::HashMap<u16, usize>;
    fn get_elapsed_duration(&self) -> Duration;
    fn get_average_request_time(&self) -> Option<Duration>;
    fn get_fastest_request_time(&self) -> Option<Duration>;
    fn get_slowest_request_time(&self) -> Option<Duration>;
    fn get_request_time_count(&self) -> usize;
    fn get_average_parsing_time(&self) -> Option<Duration>;
    fn get_fastest_parsing_time(&self) -> Option<Duration>;
    fn get_slowest_parsing_time(&self) -> Option<Duration>;
    fn get_parsing_time_count(&self) -> usize;
    fn get_recent_requests_per_second(&self) -> f64;
    fn get_recent_responses_per_second(&self) -> f64;
    fn get_recent_items_per_second(&self) -> f64;
    fn formatted_duration(&self) -> String;
    fn formatted_request_time(&self, duration: Option<Duration>) -> String;
    fn formatted_bytes(&self) -> String;
}


use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::Instant,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceRecord {
    pub candidate: String,
    pub span: String,
    pub duration_ns: u128,
    pub attributes: BTreeMap<String, String>,
}

#[derive(Clone, Default)]
pub struct TraceCollector {
    records: Arc<Mutex<Vec<TraceRecord>>>,
}

impl TraceCollector {
    pub async fn measure<F, T>(
        &self,
        candidate: &str,
        span: &str,
        attributes: BTreeMap<String, String>,
        future: F,
    ) -> T
    where
        F: std::future::Future<Output = T>,
    {
        let start = Instant::now();
        let output = future.await;
        self.records.lock().expect("trace lock").push(TraceRecord {
            candidate: candidate.to_owned(),
            span: span.to_owned(),
            duration_ns: start.elapsed().as_nanos(),
            attributes,
        });
        output
    }

    pub fn snapshot(&self) -> Vec<TraceRecord> {
        self.records.lock().expect("trace lock").clone()
    }
}

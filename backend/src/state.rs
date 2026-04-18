use std::collections::HashMap;
use std::sync::Mutex;

use reqwest::Client;

use crate::error::AppError;
use crate::stellar::DisbursementOutcome;

#[derive(Clone)]
pub struct CachedSubmission {
    pub ai_status: &'static str,
    pub disbursement: DisbursementOutcome,
}

pub struct AppState {
    pub http: Client,
    cache: Mutex<HashMap<String, CachedSubmission>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .expect("reqwest client"),
            cache: Mutex::new(HashMap::new()),
        }
    }
}

impl AppState {
    pub fn get_cached(&self, key: &str) -> Option<CachedSubmission> {
        self.cache.lock().ok()?.get(key).cloned()
    }

    pub fn insert_cache(&self, key: String, value: CachedSubmission) -> Result<(), AppError> {
        let mut g = self.cache.lock().map_err(|_| AppError {
            status: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            message: "cache lock poisoned".into(),
        })?;
        g.insert(key, value);
        Ok(())
    }
}

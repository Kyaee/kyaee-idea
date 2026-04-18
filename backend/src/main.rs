mod ai;
mod error;
mod state;
mod stellar;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::{DefaultBodyLimit, Multipart, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

#[derive(Serialize)]
struct SubmitResponse {
    ai_status: &'static str,
    disbursement: stellar::DisbursementOutcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    idempotency_replay: Option<bool>,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "river-warrior-backend",
    })
}

async fn submit(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Response, AppError> {
    let idem_from_header = headers
        .get(header::HeaderName::from_static("idempotency-key"))
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);

    let mut collector: Option<String> = None;
    let mut idem_from_field: Option<String> = None;
    let mut image_bytes: Option<Vec<u8>> = None;
    let mut image_content_type: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::bad_request(format!("multipart: {e}")))?
    {
        let name = field.name().map(str::to_string).unwrap_or_default();
        match name.as_str() {
            "collector_g_address" | "collector_public_key" => {
                let t = field
                    .text()
                    .await
                    .map_err(|e| AppError::bad_request(format!("collector field: {e}")))?;
                collector = Some(t.trim().to_string());
            }
            "idempotency_key" => {
                let t = field
                    .text()
                    .await
                    .map_err(|e| AppError::bad_request(format!("idempotency field: {e}")))?;
                idem_from_field = Some(t.trim().to_string());
            }
            "image" => {
                image_content_type = field.content_type().map(|s| s.to_string());
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::bad_request(format!("image field: {e}")))?;
                image_bytes = Some(data.to_vec());
            }
            _ => {}
        }
    }

    let collector = collector.ok_or_else(|| AppError::bad_request("missing collector_g_address"))?;
    stellar::validate_collector_address(&collector)?;

    let idempotency_key = idem_from_header
        .or(idem_from_field)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| uuid_placeholder(&collector, image_bytes.as_deref()));

    if let Some(cached) = state.get_cached(&idempotency_key) {
        let body = SubmitResponse {
            ai_status: cached.ai_status,
            disbursement: cached.disbursement.clone(),
            idempotency_replay: Some(true),
        };
        return Ok((StatusCode::OK, Json(body)).into_response());
    }

    let image = image_bytes.ok_or_else(|| AppError::bad_request("missing image file field"))?;
    if image.is_empty() {
        return Err(AppError::bad_request("empty image"));
    }
    if image.len() > 8 * 1024 * 1024 {
        return Err(AppError::bad_request("image larger than 8 MiB"));
    }

    let ai_status = ai::evaluate_claim(&state, &image, image_content_type.as_deref()).await?;

    let disbursement = if ai_status {
        stellar::maybe_disburse(&collector).await
    } else {
        stellar::DisbursementOutcome::skipped("ai_rejected")
    };

    state.insert_cache(
        idempotency_key,
        state::CachedSubmission {
            ai_status: if ai_status { "VERIFIED" } else { "REJECTED" },
            disbursement: disbursement.clone(),
        },
    )?;

    let body = SubmitResponse {
        ai_status: if ai_status { "VERIFIED" } else { "REJECTED" },
        disbursement,
        idempotency_replay: Some(false),
    };
    Ok((StatusCode::OK, Json(body)).into_response())
}

/// Deterministic fallback idempotency key when client sends none.
fn uuid_placeholder(collector: &str, image: Option<&[u8]>) -> String {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    collector.hash(&mut h);
    image.map(|b| b.len()).hash(&mut h);
    format!("auto-{:x}", h.finish())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let state = Arc::new(AppState::default());

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/submit", post(submit))
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8787);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::db::{DB, types};

// called by the TUI: queue a URL to open on a specific phone
pub async fn create_pending_open(
    State(db): State<Arc<DB>>,
    Json(payload): Json<types::NewPendingOpen>,
) -> StatusCode {
    let now = chrono::Utc::now().timestamp();
    match db.create_pending_open(&payload, now) {
        Ok(id) => {
            tracing::info!(
                "queued pending open {id} for {}: {}",
                payload.target_device,
                payload.url
            );
            StatusCode::OK
        }
        Err(e) => {
            tracing::error!("failed to create pending open: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

// called by a phone's sync job: what's waiting for me?
pub async fn get_pending_opens(
    State(db): State<Arc<DB>>,
    Path(target_device): Path<String>,
) -> Result<Json<Vec<types::PendingOpen>>, StatusCode> {
    db.get_pending_opens(&target_device)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

// called by a phone once it's actually opened the URL
pub async fn ack_pending_open(
    State(db): State<Arc<DB>>,
    Path(id): Path<i64>,
) -> StatusCode {
    match db.mark_pending_open_delivered(id) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

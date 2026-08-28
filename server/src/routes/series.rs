use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::db::{DB, types};

pub async fn post_event(
    State(db): State<Arc<DB>>,
    State(tx): State<tokio::sync::mpsc::UnboundedSender<i64>>,
    Json(payload): Json<types::NewEvent>,
) -> StatusCode {
    tracing::info!("got event: {payload:?}");
    match db.insert_event(&payload) {
        Ok(event_id) => {
            let _ = tx.send(event_id);
            StatusCode::OK
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn list_series(
    State(db): State<Arc<DB>>,
) -> Result<Json<Vec<types::Series>>, StatusCode> {
    db.list_series()
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn get_series(
    State(db): State<Arc<DB>>,
    Path(id): Path<i64>,
) -> Result<Json<types::Series>, StatusCode> {
    db.get_series(id)
        .map(Json)
        .map_err(|_| StatusCode::NOT_FOUND)
}

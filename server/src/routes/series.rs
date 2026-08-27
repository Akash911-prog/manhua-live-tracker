use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::db::{DB, types};

pub async fn post_event(
    State(db): State<Arc<DB>>,
    Json(payload): Json<types::NewEvent>,
) -> StatusCode {
    match db.insert_event(&payload) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn get_series(
    State(db): State<Arc<DB>>,
    Path(id): Path<i64>,
) -> Result<Json<types::Series>, StatusCode> {
    db.get_series(id)
        .map(Json)
        .map_err(|_| StatusCode::NOT_FOUND)
}

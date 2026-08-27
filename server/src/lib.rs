use std::sync::Arc;

use crate::db::DB;

pub mod db;
pub mod middleware;
pub mod process;
pub mod routes;
pub mod static_files;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DB>,
    pub tx: tokio::sync::mpsc::UnboundedSender<i64>,
}

impl axum::extract::FromRef<AppState> for Arc<DB> {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}

impl axum::extract::FromRef<AppState> for tokio::sync::mpsc::UnboundedSender<i64> {
    fn from_ref(state: &AppState) -> Self {
        state.tx.clone()
    }
}

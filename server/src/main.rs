use axum::{
    Router,
    http::StatusCode,
    routing::{get, post},
};
use axum_server::tls_rustls::RustlsConfig;
use manhua_live_tracker::{
    AppState,
    db::DB,
    middleware::require_auth,
    process::resolve_event,
    routes::series::{get_series, post_event},
};
use std::{net::SocketAddr, sync::Arc};

#[tokio::main]
async fn main() {
    // gets users home sandbox and create the dir and database file if not exists
    let data_dir = dirs::data_dir()
        .expect("no data dir")
        .join("manhua-tracker");
    std::fs::create_dir_all(&data_dir).expect("failed to create data dir");
    let db_path_buff = data_dir.join("manhua.db");
    let db_path = db_path_buff
        .to_str()
        .expect("failed to convert path to str");

    // initializes the database
    let db = Arc::new(DB::init(db_path).expect("failed to init db"));
    let token = Arc::new(std::env::var("MT_TOKEN").expect("MANHUA_AUTH_TOKEN not set"));

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<i64>();

    let worker_db = db.clone();
    tokio::spawn(async move {
        while let Some(event_id) = rx.recv().await {
            if let Err(e) = resolve_event(&worker_db, event_id) {
                tracing::error!("resolver failed for event {event_id}: {e}");
            }
        }
    });

    // creates the App
    let protected = Router::new()
        .route("/series/{id}", get(get_series))
        .route("/events", post(post_event))
        .layer(axum::middleware::from_fn_with_state(
            token.clone(),
            require_auth,
        ));

    let public = Router::new().route("/health", get(|| async { StatusCode::OK }));

    let state = AppState { db, tx };

    let app = protected.merge(public).with_state(state);

    let tls_config = RustlsConfig::from_pem_file("cert.pem", "key.pem")
        .await
        .expect("failed to load TLS cert/key");

    let addr: SocketAddr = "0.0.0.0:1409".parse().unwrap();
    axum_server::bind_rustls(addr, tls_config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

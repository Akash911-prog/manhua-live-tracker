use axum::{
    Router,
    http::StatusCode,
    routing::{get, post},
};
use axum_server::tls_rustls::RustlsConfig;
use manhua_live_tracker::{
    db::DB,
    middleware::require_auth,
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

    // creates the App
    let protected = Router::new()
        .route("/series/{id}", get(get_series))
        .route("/events", post(post_event))
        .layer(axum::middleware::from_fn_with_state(
            token.clone(),
            require_auth,
        ));

    let public = Router::new().route("/health", get(|| async { StatusCode::OK }));

    let app = protected.merge(public).with_state(db);

    let tls_config = RustlsConfig::from_pem_file("cert.pem", "key.pem")
        .await
        .expect("failed to load TLS cert/key");

    let addr: SocketAddr = "0.0.0.0:4177".parse().unwrap();
    axum_server::bind_rustls(addr, tls_config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

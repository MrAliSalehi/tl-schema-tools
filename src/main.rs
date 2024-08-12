use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use axum::routing::get;
use dotenv::{dotenv, var};
use meilisearch_sdk::client::Client;
use sqlx::PgPool;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};
use crate::app_state::AppState;
use crate::prelude::{init_logger, Res};
use crate::tl::schema_manager::SchemaManager;

mod prelude;
mod app_state;
mod components;
mod github;
mod models;
mod db;
mod tl;

#[tokio::main]
async fn main() -> Res {
    dotenv().ok();
    init_logger().await?;
    let (ms_url, ms_key) = (var("MS_PATH")?, var("MS_API_KEY")?);
    let db: Arc<PgPool> = Arc::new(PgPool::connect(&var("DATABASE_URL")?).await?);

    let r = Arc::new(AtomicBool::new(true));
    let running = r.clone();
    ctrlc::set_handler(move || { r.store(false, Ordering::SeqCst); })?;

    github::run_task(db.clone()).await?;

    let layers = db::tl_layer::get_all(&db.clone()).await?;
    let meilisearch = Client::new(ms_url, Some(ms_key))?;
    let schema_manager = SchemaManager::new(layers, meilisearch).await?;
    let state = AppState::new(db, schema_manager);

    let app = axum::Router::new()
        .route("/", get(components::root))
        .nest("/api", components::routes(state))
        .layer(CompressionLayer::new().gzip(true))
        .layer(cors());

    let listener = tokio::net::TcpListener::bind(var("ADDRESS")?).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            while running.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
            log::info!("Ctrl+C detected, closing...");
        })
        .await?;
    Ok(())
}
fn cors() -> CorsLayer {
    CorsLayer::new()
        .allow_headers(Any)
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_private_network(true)
}

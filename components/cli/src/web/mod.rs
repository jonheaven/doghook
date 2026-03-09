use axum::{
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use config::DogecoinConfig;
use deadpool_postgres;
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;

mod handlers;

use handlers::*;

/// Capacity of the SSE broadcast channel (events buffered per subscriber lag).
const SSE_CHANNEL_CAPACITY: usize = 256;

/// Shared application state for the web server
#[derive(Clone)]
pub struct AppState {
    pub doginals_pool: Arc<deadpool_postgres::Pool>,
    pub drc20_pool: Option<Arc<deadpool_postgres::Pool>>,
    pub dunes_pool: Option<Arc<deadpool_postgres::Pool>>,
    pub dogecoin_config: DogecoinConfig,
    /// Broadcast channel sender — indexer events arrive via POST /api/webhook
    /// and are fanned out to all /api/events SSE subscribers.
    pub event_tx: broadcast::Sender<String>,
}

/// Start the doghook web explorer server.
/// Returns the `broadcast::Sender` so the caller can inject the local webhook URL.
pub async fn start_web_server(
    addr: SocketAddr,
    doginals_pool: Arc<deadpool_postgres::Pool>,
    drc20_pool: Option<Arc<deadpool_postgres::Pool>>,
    dunes_pool: Option<Arc<deadpool_postgres::Pool>>,
    _burn_address: String,
    dogecoin_config: DogecoinConfig,
) -> Result<broadcast::Sender<String>, Box<dyn std::error::Error>> {
    let (event_tx, _) = broadcast::channel(SSE_CHANNEL_CAPACITY);
    let state = AppState {
        doginals_pool,
        drc20_pool,
        dunes_pool,
        dogecoin_config,
        event_tx: event_tx.clone(),
    };

    let app = Router::new()
        // API endpoints
        .route("/api/inscriptions", get(get_inscriptions))
        .route("/api/inscriptions/recent", get(get_recent_inscriptions))
        .route("/api/drc20/tokens", get(get_drc20_tokens))
        .route("/api/dunes/tokens", get(get_dunes_tokens))
        .route("/api/lotto/tickets", get(get_lotto_tickets))
        .route("/api/lotto/winners", get(get_lotto_winners))
        .route("/api/lotto/verify", get(lotto_verify))
        .route("/api/dns/names", get(get_dns_names))
        .route("/api/dogemap/claims", get(get_dogemap_claims))
        .route("/api/dogetags", get(get_dogetags))
        .route("/api/status", get(get_status))
        .route("/api/decode", get(decode_inscription))
        .route("/content/:inscription_id", get(get_inscription_content))
        // HTML pages
        .route("/", get(index_page))
        .route("/inscriptions", get(inscriptions_page))
        .route("/drc20", get(drc20_page))
        .route("/dunes", get(dunes_page))
        .route("/lotto", get(lotto_page))
        // Static assets
        .route("/wallet.js", get(wallet_js))
        // SSE event stream + webhook receiver
        .route("/api/events", get(sse_events))
        .route("/api/webhook", post(receive_webhook))
        // Health check
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive())
        .with_state(state);

    println!("🌐 Doghook explorer starting on http://{}", addr);
    println!("   Visit http://{}/ to view the inscription explorer", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(event_tx)
}

async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "service": "doghook-explorer"
    }))
}

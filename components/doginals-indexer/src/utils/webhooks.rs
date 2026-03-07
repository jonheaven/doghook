use dogecoin::{try_warn, utils::Context};
use reqwest::Client;
use serde_json::Value;

/// Fire-and-forget: POST `payload` to every URL in `urls`.
/// Errors are logged as warnings — a failed delivery never blocks indexing.
pub async fn fire_webhooks(urls: &[String], payload: Value, ctx: &Context) {
    if urls.is_empty() {
        return;
    }
    let client = Client::new();
    for url in urls {
        match client.post(url).json(&payload).send().await {
            Ok(resp) if resp.status().is_success() => {}
            Ok(resp) => {
                try_warn!(ctx, "Webhook POST to {url} returned status {}", resp.status());
            }
            Err(e) => {
                try_warn!(ctx, "Webhook POST to {url} failed: {e}");
            }
        }
    }
}

/// Build a DNS registration event payload.
pub fn dns_event(
    name: &str,
    inscription_id: &str,
    block_height: u64,
    block_timestamp: u32,
) -> Value {
    serde_json::json!({
        "event": "dns.registered",
        "name": name,
        "inscription_id": inscription_id,
        "block_height": block_height,
        "block_timestamp": block_timestamp,
    })
}

/// Build a Dogemap claim event payload.
pub fn dogemap_event(
    block_number: u32,
    inscription_id: &str,
    claim_height: u64,
    claim_timestamp: u32,
) -> Value {
    serde_json::json!({
        "event": "dogemap.claimed",
        "block_number": block_number,
        "inscription_id": inscription_id,
        "claim_height": claim_height,
        "claim_timestamp": claim_timestamp,
    })
}

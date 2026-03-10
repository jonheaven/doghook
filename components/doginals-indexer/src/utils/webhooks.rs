use hmac::{Hmac, Mac};
use prometheus::{register_int_counter, IntCounter};
use rand::RngCore;
use reqwest::Client;
use serde_json::Value;
use sha2::Sha256;
use std::sync::OnceLock;

type HmacSha256 = Hmac<Sha256>;

// Shared client so all webhook tasks reuse the same connection pool.
// 30-second timeout prevents a hung receiver from holding connections indefinitely.
static CLIENT: OnceLock<Client> = OnceLock::new();

fn client() -> &'static Client {
    CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("failed to build reqwest client")
    })
}

static WEBHOOK_SUCCESS: OnceLock<IntCounter> = OnceLock::new();
static WEBHOOK_FAILURE: OnceLock<IntCounter> = OnceLock::new();

fn webhook_success() -> &'static IntCounter {
    WEBHOOK_SUCCESS.get_or_init(|| {
        register_int_counter!(
            "doghook_webhook_deliveries_total",
            "Successful webhook deliveries"
        )
        .unwrap()
    })
}

fn webhook_failure() -> &'static IntCounter {
    WEBHOOK_FAILURE.get_or_init(|| {
        register_int_counter!(
            "doghook_webhook_failures_total",
            "Webhook deliveries that exhausted all retries"
        )
        .unwrap()
    })
}

/// Spawn a background task to deliver `payload` to all `urls` with HMAC signing and
/// exponential-backoff retries. Returns immediately — never blocks the indexer.
///
/// Each delivery includes:
///   `X-Doghook-Event: <event type from payload["event"]>`
///   `_id: <16-byte random hex>` field stamped into the payload for deduplication
///
/// If `hmac_secret` is `Some`, each request also includes:
///   `X-Doghook-Signature: sha256=<hex(HMAC-SHA256(secret, body))>`
///
/// Attempts: up to 5, with delays of 2 s / 4 s / 8 s / 16 s / 32 s.
/// Delivery failures are printed to stderr — they never block indexing.
pub fn fire_webhooks(urls: Vec<String>, hmac_secret: Option<String>, payload: Value) {
    if urls.is_empty() {
        return;
    }
    tokio::spawn(async move {
        let client = client();

        // Stamp a unique idempotency key so receivers can deduplicate retried deliveries.
        let mut payload = payload;
        let mut id_bytes = [0u8; 16];
        rand::rng().fill_bytes(&mut id_bytes);
        payload["_id"] = serde_json::Value::String(hex::encode(id_bytes));

        // Extract event type for the X-Doghook-Event header.
        let event_type = payload["event"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        let body = payload.to_string();

        let sig = hmac_secret.as_deref().map(|secret| {
            let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
                .expect("HMAC accepts keys of any length");
            mac.update(body.as_bytes());
            format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
        });

        for url in &urls {
            let mut attempts: u32 = 0;
            loop {
                let mut builder = client
                    .post(url)
                    .header("Content-Type", "application/json")
                    .header("X-Doghook-Event", &event_type)
                    .body(body.clone());
                if let Some(ref s) = sig {
                    builder = builder.header("X-Doghook-Signature", s);
                }
                match builder.send().await {
                    Ok(r) if r.status().is_success() => {
                        webhook_success().inc();
                        break;
                    }
                    Ok(r) => {
                        if attempts >= 4 {
                            eprintln!("[doghook] webhook {url} gave up after {attempts} retries (status {})", r.status());
                            webhook_failure().inc();
                            break;
                        }
                    }
                    Err(e) => {
                        if attempts >= 4 {
                            eprintln!("[doghook] webhook {url} gave up after {attempts} retries: {e}");
                            webhook_failure().inc();
                            break;
                        }
                    }
                }
                attempts += 1;
                tokio::time::sleep(tokio::time::Duration::from_secs(2u64.pow(attempts))).await;
            }
        }
    });
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

/// Build a Dogetag on-chain graffiti event payload.
pub fn dogetag_event(
    txid: &str,
    sender_address: &str,
    message: &str,
    block_height: u64,
    block_timestamp: u32,
) -> Value {
    serde_json::json!({
        "event": "dogetag.tagged",
        "txid": txid,
        "sender_address": sender_address,
        "message": message,
        "block_height": block_height,
        "block_timestamp": block_timestamp,
    })
}

/// Build a doge-lotto ticket event payload.
pub fn lotto_ticket_event(
    lotto_id: &str,
    ticket_id: &str,
    inscription_id: &str,
    tx_id: &str,
    minted_height: u64,
    minted_timestamp: u64,
    seed_numbers: &[u16],
    tip_percent: u8,
) -> Value {
    serde_json::json!({
        "event": "lotto.ticket_minted",
        "lotto_id": lotto_id,
        "ticket_id": ticket_id,
        "inscription_id": inscription_id,
        "tx_id": tx_id,
        "minted_height": minted_height,
        "minted_timestamp": minted_timestamp,
        "seed_numbers": seed_numbers,
        "tip_percent": tip_percent,
    })
}

/// Build a doge-lotto winner resolution event payload.
pub fn lotto_winner_event(
    lotto_id: &str,
    ticket_id: &str,
    inscription_id: &str,
    resolved_height: u64,
    rank: u32,
    score: u64,
    payout_bps: u32,
    gross_payout_koinu: u64,
    tip_percent: u8,
    tip_deduction_koinu: u64,
    payout_koinu: u64,
    seed_numbers: &[u16],
    drawn_numbers: &[u16],
) -> Value {
    serde_json::json!({
        "event": "lotto.winner_resolved",
        "lotto_id": lotto_id,
        "ticket_id": ticket_id,
        "inscription_id": inscription_id,
        "resolved_height": resolved_height,
        "rank": rank,
        "score": score,
        "payout_bps": payout_bps,
        "gross_payout_koinu": gross_payout_koinu,
        "tip_percent": tip_percent,
        "tip_deduction_koinu": tip_deduction_koinu,
        "payout_koinu": payout_koinu,
        "seed_numbers": seed_numbers,
        "drawn_numbers": drawn_numbers,
    })
}

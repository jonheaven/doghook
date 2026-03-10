use dogecoin::{try_warn, utils::Context};
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde_json::Value;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// POST `payload` to every URL in `urls`, with exponential-backoff retries and optional
/// HMAC-SHA256 request signing.
///
/// If `hmac_secret` is `Some`, each request includes:
///   `X-Doghook-Signature: sha256=<hex(HMAC-SHA256(secret, body))>`
/// Receivers can verify authenticity with the same secret.
///
/// Delivery failures are logged as warnings — they never block indexing.
/// Each URL is delivered serially with up to 5 attempts (waits: 2 s, 4 s, 8 s, 16 s, 32 s).
pub async fn fire_webhooks(
    urls: &[String],
    hmac_secret: Option<&str>,
    payload: Value,
    ctx: &Context,
) {
    if urls.is_empty() {
        return;
    }
    let client = Client::new();
    let body = payload.to_string();

    let sig = hmac_secret.map(|secret| {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .expect("HMAC accepts keys of any length");
        mac.update(body.as_bytes());
        format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
    });

    for url in urls {
        let mut attempts: u32 = 0;
        loop {
            let mut builder = client
                .post(url)
                .header("Content-Type", "application/json")
                .body(body.clone());
            if let Some(ref s) = sig {
                builder = builder.header("X-Doghook-Signature", s);
            }
            match builder.send().await {
                Ok(resp) if resp.status().is_success() => break,
                Ok(resp) => {
                    if attempts >= 4 {
                        try_warn!(
                            ctx,
                            "Webhook {url} gave up after {} retries (last status: {})",
                            attempts,
                            resp.status()
                        );
                        break;
                    }
                }
                Err(e) => {
                    if attempts >= 4 {
                        try_warn!(
                            ctx,
                            "Webhook {url} gave up after {} retries: {e}",
                            attempts
                        );
                        break;
                    }
                }
            }
            attempts += 1;
            tokio::time::sleep(tokio::time::Duration::from_secs(2u64.pow(attempts))).await;
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

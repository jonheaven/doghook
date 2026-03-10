use std::convert::Infallible;

use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        Html, IntoResponse, Json, Response,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt as _;

use dogecoin::bitcoincore_rpc::RpcApi;
use doginals::envelope::ParsedEnvelope;

use super::AppState;

#[derive(Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

#[derive(Serialize)]
pub struct InscriptionRow {
    pub inscription_id: String,
    pub inscription_number: i64,
    pub block_height: i64,
    pub block_timestamp: i64,
    pub content_type: Option<String>,
    pub content_length: Option<i64>,
}

#[derive(Serialize)]
pub struct Drc20Token {
    pub tick: String,
    pub max_supply: String,
    pub minted: String,
    pub deployer: String,
    pub block_height: i64,
}

#[derive(Serialize)]
pub struct DunesToken {
    pub name: String,
    pub spaced_name: String,
    pub block: i64,
    pub mints: i64,
    pub burned: String,
    pub divisibility: i32,
}

#[derive(Serialize)]
pub struct LottoTicket {
    pub ticket_id: String,
    pub lotto_name: String,
    pub player_address: String,
    pub block_height: i64,
    pub tip_percent: i32,
}

#[derive(Serialize)]
pub struct LottoWinner {
    pub ticket_id: String,
    pub lotto_name: String,
    pub player_address: String,
    pub gross_payout_koinu: i64,
    pub tip_deduction_koinu: i64,
    pub draw_block: i64,
}

#[derive(Serialize)]
pub struct DnsName {
    pub name: String,
    pub inscription_id: String,
    pub block_height: i64,
    pub block_timestamp: i64,
}

#[derive(Serialize)]
pub struct DogemapClaim {
    pub block_number: i64,
    pub inscription_id: String,
    pub claim_height: i64,
    pub claim_timestamp: i64,
}

#[derive(Serialize)]
pub struct DogetagEntry {
    pub id: i64,
    pub txid: String,
    pub block_height: i64,
    pub block_timestamp: i64,
    pub sender_address: Option<String>,
    pub message: String,
    pub message_bytes: i32,
}

#[derive(Serialize)]
pub struct CharmsBalanceEntry {
    pub ticker: String,
    pub address: String,
    pub balance: String,
}

#[derive(Serialize)]
pub struct CharmsSpellEntry {
    pub txid: String,
    pub vout: u32,
    pub block_height: u64,
    pub block_timestamp: u32,
    pub version: String,
    pub tag: String,
    pub op: String,
    pub id: String,
    pub chain_id: String,
    pub ticker: Option<String>,
    pub name: Option<String>,
    pub amount: Option<u64>,
    pub decimals: Option<u8>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub beam_to: Option<String>,
    pub beam_proof: Option<String>,
    pub raw_cbor: String,
}

pub async fn get_inscriptions(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<InscriptionRow>>, StatusCode> {
    let client = state
        .doginals_pool
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rows = client
        .query(
            "SELECT inscription_id, number AS inscription_number, block_height::bigint,
                    timestamp AS block_timestamp, content_type, content_length
             FROM inscriptions
             ORDER BY number DESC
             LIMIT $1 OFFSET $2",
            &[&params.limit, &params.offset],
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let inscriptions: Vec<InscriptionRow> = rows
        .iter()
        .map(|row| InscriptionRow {
            inscription_id: row.get(0),
            inscription_number: row.get(1),
            block_height: row.get(2),
            block_timestamp: row.get(3),
            content_type: row.get(4),
            content_length: row.get(5),
        })
        .collect();

    Ok(Json(inscriptions))
}

pub async fn get_recent_inscriptions(
    State(state): State<AppState>,
) -> Result<Json<Vec<InscriptionRow>>, StatusCode> {
    let client = state
        .doginals_pool
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rows = client
        .query(
            "SELECT inscription_id, number AS inscription_number, block_height::bigint,
                    timestamp AS block_timestamp, content_type, content_length
             FROM inscriptions
             ORDER BY number DESC
             LIMIT 20",
            &[],
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let inscriptions: Vec<InscriptionRow> = rows
        .iter()
        .map(|row| InscriptionRow {
            inscription_id: row.get(0),
            inscription_number: row.get(1),
            block_height: row.get(2),
            block_timestamp: row.get(3),
            content_type: row.get(4),
            content_length: row.get(5),
        })
        .collect();

    Ok(Json(inscriptions))
}

pub async fn get_drc20_tokens(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<Drc20Token>>, StatusCode> {
    let pool = state.drc20_pool.as_ref().ok_or(StatusCode::NOT_FOUND)?;

    let client = pool
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rows = client
        .query(
            "SELECT ticker AS tick, max::text AS max_supply,
                    COALESCE(minted_supply, 0)::text AS minted,
                    address AS deployer, block_height::bigint
             FROM tokens
             ORDER BY block_height DESC
             LIMIT $1 OFFSET $2",
            &[&params.limit, &params.offset],
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tokens: Vec<Drc20Token> = rows
        .iter()
        .map(|row| Drc20Token {
            tick: row.get(0),
            max_supply: row.get(1),
            minted: row.get(2),
            deployer: row.get(3),
            block_height: row.get(4),
        })
        .collect();

    Ok(Json(tokens))
}

pub async fn get_dunes_tokens(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<DunesToken>>, StatusCode> {
    let pool = state.dunes_pool.as_ref().ok_or(StatusCode::NOT_FOUND)?;

    let client = pool
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rows = client
        .query(
            "SELECT r.name, r.spaced_name, r.block_height::bigint AS block,
                    COALESCE(sc.total_mints, 0)::bigint AS mints,
                    COALESCE(sc.burned, 0)::text AS burned,
                    r.divisibility::int AS divisibility
             FROM runes r
             LEFT JOIN LATERAL (
                 SELECT total_mints, burned
                 FROM supply_changes
                 WHERE rune_id = r.id
                 ORDER BY block_height DESC
                 LIMIT 1
             ) sc ON TRUE
             ORDER BY r.block_height DESC
             LIMIT $1 OFFSET $2",
            &[&params.limit, &params.offset],
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tokens: Vec<DunesToken> = rows
        .iter()
        .map(|row| DunesToken {
            name: row.get(0),
            spaced_name: row.get(1),
            block: row.get(2),
            mints: row.get(3),
            burned: row.get(4),
            divisibility: row.get(5),
        })
        .collect();

    Ok(Json(tokens))
}

pub async fn get_lotto_tickets(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<LottoTicket>>, StatusCode> {
    let client = state
        .doginals_pool
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rows = client
        .query(
            "SELECT ticket_id, lotto_id AS lotto_name,
                    inscription_id AS player_address,
                    minted_height AS block_height, tip_percent
             FROM lotto_tickets
             ORDER BY minted_height DESC
             LIMIT $1 OFFSET $2",
            &[&params.limit, &params.offset],
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tickets: Vec<LottoTicket> = rows
        .iter()
        .map(|row| LottoTicket {
            ticket_id: row.get(0),
            lotto_name: row.get(1),
            player_address: row.get(2),
            block_height: row.get(3),
            tip_percent: row.get(4),
        })
        .collect();

    Ok(Json(tickets))
}

pub async fn get_lotto_winners(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<LottoWinner>>, StatusCode> {
    let client = state
        .doginals_pool
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rows = client
        .query(
            "SELECT ticket_id, lotto_id AS lotto_name,
                    inscription_id AS player_address,
                    gross_payout_koinu, tip_deduction_koinu,
                    resolved_height AS draw_block
             FROM lotto_winners
             ORDER BY resolved_height DESC
             LIMIT $1 OFFSET $2",
            &[&params.limit, &params.offset],
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let winners: Vec<LottoWinner> = rows
        .iter()
        .map(|row| LottoWinner {
            ticket_id: row.get(0),
            lotto_name: row.get(1),
            player_address: row.get(2),
            gross_payout_koinu: row.get(3),
            tip_deduction_koinu: row.get(4),
            draw_block: row.get(5),
        })
        .collect();

    Ok(Json(winners))
}

/// Query params for GET /api/lotto/verify
#[derive(Deserialize)]
pub struct LottoVerifyParams {
    /// Hex block hash of the draw block (the target u256).
    pub block_hash: String,
    /// Comma-separated seed numbers chosen on the ticket (e.g. "1,7,42,69,100,420").
    pub numbers: String,
    /// Optional: lotto_id to scope winner lookup. If omitted, only fingerprint data is returned.
    pub lotto_id: Option<String>,
}

/// Response for GET /api/lotto/verify
/// All computations are derived purely from Dogecoin chain data and are 100% verifiable.
#[derive(Serialize)]
pub struct LottoVerifyResponse {
    /// SHA256(sorted seed u16 pairs as big-endian bytes) = the ticket's fingerprint.
    pub fingerprint: String,
    /// block_hash interpreted as a big-endian u256.
    pub draw_target: String,
    /// Hex u256: |fingerprint − draw_target|. Smaller = closer = better rank.
    pub distance: String,
    /// Classic numbers (1-49) derived deterministically from the fingerprint.
    pub classic_numbers: Vec<u16>,
    /// Tie rule: tickets sharing the exact same distance split that tier equally.
    /// Display rank within a tie is sorted by inscription_id lex (smaller first).
    pub tie_rule: &'static str,
    /// If lotto_id was provided and the lottery is resolved: winner details, or null.
    pub winner: Option<serde_json::Value>,
}

pub async fn lotto_verify(
    State(state): State<AppState>,
    Query(params): Query<LottoVerifyParams>,
) -> Result<Json<LottoVerifyResponse>, StatusCode> {
    use doginals_indexer::core::meta_protocols::lotto::{
        compute_ticket_fingerprint, derive_classic_numbers, u256_abs_diff,
    };

    // Parse numbers
    let seed_numbers: Vec<u16> = params
        .numbers
        .split(',')
        .filter_map(|s| s.trim().parse::<u16>().ok())
        .collect();
    if seed_numbers.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let fp_bytes = compute_ticket_fingerprint(&seed_numbers);
    let fp_hex = hex::encode(fp_bytes);

    // Parse block hash as u256
    let hash_hex = params.block_hash.trim_start_matches("0x");
    let hash_bytes_vec = hex::decode(hash_hex).map_err(|_| StatusCode::BAD_REQUEST)?;
    let mut draw_target = [0u8; 32];
    let copy_len = hash_bytes_vec.len().min(32);
    draw_target[..copy_len].copy_from_slice(&hash_bytes_vec[..copy_len]);

    let distance_bytes = u256_abs_diff(&fp_bytes, &draw_target);
    let distance_hex = hex::encode(distance_bytes);

    let classic_numbers = derive_classic_numbers(&fp_bytes);

    // Optionally look up winner record
    let winner = if let Some(ref lotto_id) = params.lotto_id {
        let client = state
            .doginals_pool
            .get()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        client
            .query_opt(
                "SELECT w.rank, w.payout_koinu, w.classic_matches, w.classic_payout_koinu,
                        w.fingerprint_distance, w.inscription_id
                 FROM lotto_winners w
                 WHERE w.lotto_id = $1 AND w.fingerprint_distance = $2",
                &[lotto_id, &distance_hex],
            )
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .map(|row| {
                json!({
                    "rank": row.get::<_, i32>("rank"),
                    "payout_koinu": row.get::<_, i64>("payout_koinu"),
                    "classic_matches": row.get::<_, i32>("classic_matches"),
                    "classic_payout_koinu": row.get::<_, i64>("classic_payout_koinu"),
                    "fingerprint_distance": row.get::<_, Option<String>>("fingerprint_distance"),
                    "inscription_id": row.get::<_, String>("inscription_id"),
                })
            })
    } else {
        None
    };

    Ok(Json(LottoVerifyResponse {
        fingerprint: fp_hex,
        draw_target: hex::encode(draw_target),
        distance: distance_hex,
        classic_numbers,
        tie_rule: "Tickets sharing the exact same |fingerprint - draw_target| distance split \
                   that prize tier equally. Within a tie group, display rank is sorted by \
                   inscription_id lexicographic (lex-smaller first) for display purposes only.",
        winner,
    }))
}

pub async fn get_dns_names(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<DnsName>>, StatusCode> {
    let client = state
        .doginals_pool
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rows = client
        .query(
            "SELECT name, inscription_id, block_height, block_timestamp
             FROM dns_names
             ORDER BY block_height DESC
             LIMIT $1 OFFSET $2",
            &[&params.limit, &params.offset],
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let names: Vec<DnsName> = rows
        .iter()
        .map(|row| DnsName {
            name: row.get(0),
            inscription_id: row.get(1),
            block_height: row.get(2),
            block_timestamp: row.get(3),
        })
        .collect();

    Ok(Json(names))
}

pub async fn get_dogemap_claims(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<DogemapClaim>>, StatusCode> {
    let client = state
        .doginals_pool
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rows = client
        .query(
            "SELECT block_number, inscription_id, claim_height, claim_timestamp
             FROM dogemap_claims
             ORDER BY claim_height DESC
             LIMIT $1 OFFSET $2",
            &[&params.limit, &params.offset],
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let claims: Vec<DogemapClaim> = rows
        .iter()
        .map(|row| DogemapClaim {
            block_number: row.get(0),
            inscription_id: row.get(1),
            claim_height: row.get(2),
            claim_timestamp: row.get(3),
        })
        .collect();

    Ok(Json(claims))
}

pub async fn get_status(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let client = state
        .doginals_pool
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let count_row = client
        .query_one("SELECT COUNT(*) FROM inscriptions", &[])
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_inscriptions: i64 = count_row.get(0);

    let latest_row = client
        .query_one(
            "SELECT block_height::bigint, timestamp FROM inscriptions
             ORDER BY number DESC LIMIT 1",
            &[],
        )
        .await
        .ok();

    let (latest_block, latest_timestamp) = if let Some(row) = latest_row {
        (Some(row.get::<_, i64>(0)), Some(row.get::<_, i64>(1)))
    } else {
        (None, None)
    };

    Ok(Json(json!({
        "status": "running",
        "total_inscriptions": total_inscriptions,
        "latest_indexed_block": latest_block,
        "latest_block_timestamp": latest_timestamp,
    })))
}

pub async fn get_dogetags(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<DogetagEntry>>, StatusCode> {
    let client = state
        .doginals_pool
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rows = client
        .query(
            "SELECT id, txid, block_height::bigint, block_timestamp,
                    sender_address, message, message_bytes
             FROM dogetags
             ORDER BY block_height DESC, id DESC
             LIMIT $1 OFFSET $2",
            &[&params.limit, &params.offset],
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tags: Vec<DogetagEntry> = rows
        .iter()
        .map(|row| DogetagEntry {
            id: row.get(0),
            txid: row.get(1),
            block_height: row.get(2),
            block_timestamp: row.get(3),
            sender_address: row.get(4),
            message: row.get(5),
            message_bytes: row.get(6),
        })
        .collect();

    Ok(Json(tags))
}

pub async fn get_charms_balance(
    State(state): State<AppState>,
    Path((ticker, address)): Path<(String, String)>,
) -> Result<Json<CharmsBalanceEntry>, StatusCode> {
    let client = state
        .doginals_pool
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let balance = client
        .query_opt(
            "SELECT balance::text
             FROM charms_balances
             WHERE ticker = $1 AND address = $2",
            &[&ticker, &address],
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(|row| row.get::<_, String>(0))
        .unwrap_or_else(|| "0".to_string());

    Ok(Json(CharmsBalanceEntry {
        ticker,
        address,
        balance,
    }))
}

pub async fn get_charms_history(
    State(state): State<AppState>,
    Path((ticker, address)): Path<(String, String)>,
) -> Result<Json<Vec<CharmsSpellEntry>>, StatusCode> {
    let client = state
        .doginals_pool
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rows = client
        .query(
            "SELECT txid, vout, block_height::bigint, block_timestamp, version, tag, op,
                    identity, chain_id, ticker, name, amount::text, decimals::int,
                    from_addr, to_addr, beam_to, beam_proof, raw_cbor
             FROM charms_spells
             WHERE ticker = $1
               AND (from_addr = $2 OR to_addr = $2)
             ORDER BY block_height DESC, id DESC",
            &[&ticker, &address],
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let spells: Vec<CharmsSpellEntry> = rows
        .iter()
        .map(|row| CharmsSpellEntry {
            txid: row.get(0),
            vout: row.get::<_, i64>(1) as u32,
            block_height: row.get::<_, i64>(2) as u64,
            block_timestamp: row.get::<_, i64>(3) as u32,
            version: row.get(4),
            tag: row.get(5),
            op: row.get(6),
            id: row.get(7),
            chain_id: row.get(8),
            ticker: row.get(9),
            name: row.get(10),
            amount: row
                .get::<_, Option<String>>(11)
                .and_then(|value| value.parse::<u64>().ok()),
            decimals: row.get::<_, Option<i32>>(12).map(|value| value as u8),
            from: row.get(13),
            to: row.get(14),
            beam_to: row.get(15),
            beam_proof: row.get(16),
            raw_cbor: hex::encode(row.get::<_, Vec<u8>>(17)),
        })
        .collect();

    Ok(Json(spells))
}

pub async fn get_charms_spells(
    State(state): State<AppState>,
    Path(txid): Path<String>,
) -> Result<Json<Vec<CharmsSpellEntry>>, StatusCode> {
    let client = state
        .doginals_pool
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rows = client
        .query(
            "SELECT txid, vout, block_height::bigint, block_timestamp, version, tag, op,
                    identity, chain_id, ticker, name, amount::text, decimals::int,
                    from_addr, to_addr, beam_to, beam_proof, raw_cbor
             FROM charms_spells
             WHERE LOWER(txid) = LOWER($1)
             ORDER BY vout ASC, id ASC",
            &[&txid],
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let spells: Vec<CharmsSpellEntry> = rows
        .iter()
        .map(|row| CharmsSpellEntry {
            txid: row.get(0),
            vout: row.get::<_, i64>(1) as u32,
            block_height: row.get::<_, i64>(2) as u64,
            block_timestamp: row.get::<_, i64>(3) as u32,
            version: row.get(4),
            tag: row.get(5),
            op: row.get(6),
            id: row.get(7),
            chain_id: row.get(8),
            ticker: row.get(9),
            name: row.get(10),
            amount: row
                .get::<_, Option<String>>(11)
                .and_then(|value| value.parse::<u64>().ok()),
            decimals: row.get::<_, Option<i32>>(12).map(|value| value as u8),
            from: row.get(13),
            to: row.get(14),
            beam_to: row.get(15),
            beam_proof: row.get(16),
            raw_cbor: hex::encode(row.get::<_, Vec<u8>>(17)),
        })
        .collect();

    Ok(Json(spells))
}

// ---------------------------------------------------------------------------
// Inscription decode — no index required, hits Dogecoin Core RPC directly
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct DecodeParams {
    pub inscription_id: Option<String>,
    pub txid: Option<String>,
}

fn txid_from_inscription_id(iid: &str) -> String {
    match iid.rfind('i') {
        Some(pos) => iid[..pos].to_string(),
        None => iid.to_string(),
    }
}

fn parse_envelope_index(inscription_id: &str) -> usize {
    inscription_id
        .rfind('i')
        .and_then(|pos| inscription_id[pos + 1..].parse::<usize>().ok())
        .unwrap_or(0)
}

fn fetch_envelopes(
    dogecoin_config: &config::DogecoinConfig,
    txid_str: &str,
) -> Result<(Vec<ParsedEnvelope>, String), String> {
    let ctx = dogecoin::utils::Context::empty();
    let rpc = dogecoin::utils::bitcoind::dogecoin_get_client(dogecoin_config, &ctx);
    let txid: dogecoin::bitcoincore_rpc::bitcoin::Txid = txid_str
        .parse()
        .map_err(|e| format!("Invalid txid '{}': {}", txid_str, e))?;
    let raw_hex = rpc
        .get_raw_transaction_hex(&txid, None)
        .map_err(|e| format!("getrawtransaction {}: {}", txid_str, e))?;
    let raw_bytes = hex::decode(&raw_hex).map_err(|e| format!("hex decode error: {}", e))?;
    let tx: bitcoin::Transaction = bitcoin::consensus::deserialize(&raw_bytes)
        .map_err(|e| format!("tx deserialize error: {}", e))?;
    let envelopes = ParsedEnvelope::from_transactions_dogecoin(&[tx]);
    Ok((envelopes, txid_str.to_string()))
}

pub async fn decode_inscription(
    State(state): State<AppState>,
    Query(params): Query<DecodeParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let (raw_id, envelope_index) = if let Some(iid) = &params.inscription_id {
        (txid_from_inscription_id(iid), parse_envelope_index(iid))
    } else if let Some(t) = &params.txid {
        (t.clone(), 0)
    } else {
        return Err((
            StatusCode::BAD_REQUEST,
            "Provide inscription_id or txid".to_string(),
        ));
    };

    let config = state.dogecoin_config.clone();
    let txid_str = raw_id.clone();
    let (envelopes, _) = tokio::task::spawn_blocking(move || fetch_envelopes(&config, &txid_str))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    if envelopes.is_empty() {
        return Ok(Json(json!({
            "found": false,
            "inscription_id": format!("{}i0", raw_id),
            "error": "No inscriptions found in this transaction"
        })));
    }

    let env = envelopes.get(envelope_index).unwrap_or(&envelopes[0]);
    let insc = &env.payload;
    let content_type = insc
        .content_type
        .as_ref()
        .and_then(|ct| std::str::from_utf8(ct).ok())
        .map(str::to_string);
    let metaprotocol = insc
        .metaprotocol
        .as_ref()
        .and_then(|mp| std::str::from_utf8(mp).ok())
        .map(str::to_string);
    let content_length = insc.body.as_ref().map(|b| b.len());
    let ct = content_type.as_deref().unwrap_or("");
    let body_text = if ct.starts_with("text/") || ct == "application/json" {
        insc.body
            .as_ref()
            .and_then(|b| std::str::from_utf8(b).ok())
            .map(str::to_string)
    } else {
        None
    };
    let has_content = insc.body.as_ref().map(|b| !b.is_empty()).unwrap_or(false);

    Ok(Json(json!({
        "found": true,
        "inscription_id": format!("{}i{}", raw_id, envelope_index),
        "content_type": content_type,
        "content_length": content_length,
        "metaprotocol": metaprotocol,
        "has_content": has_content,
        "body_text": body_text,
        "content_url": format!("/content/{}i{}", raw_id, envelope_index),
    })))
}

pub async fn get_inscription_content(
    State(state): State<AppState>,
    Path(inscription_id): Path<String>,
) -> Response {
    let txid_str = txid_from_inscription_id(&inscription_id);
    let envelope_index = parse_envelope_index(&inscription_id);
    let config = state.dogecoin_config.clone();
    let txid_clone = txid_str.clone();

    let result = tokio::task::spawn_blocking(move || fetch_envelopes(&config, &txid_clone)).await;

    let envelopes = match result {
        Ok(Ok((e, _))) => e,
        Ok(Err(e)) => {
            return (StatusCode::BAD_REQUEST, e).into_response();
        }
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    let env = match envelopes.get(envelope_index) {
        Some(e) => e,
        None => {
            return (StatusCode::NOT_FOUND, "Inscription not found").into_response();
        }
    };

    let insc = &env.payload;
    let content_type = insc
        .content_type
        .as_ref()
        .and_then(|ct| std::str::from_utf8(ct).ok())
        .unwrap_or("application/octet-stream")
        .to_string();
    let body = insc.body.clone().unwrap_or_default();

    ([(header::CONTENT_TYPE, content_type)], Bytes::from(body)).into_response()
}

pub async fn index_page() -> Html<&'static str> {
    Html(include_str!("../../static/index.html"))
}

/// GET /api/events — SSE stream of indexer events.
///
/// Clients subscribe once and receive a real-time stream of JSON event objects
/// (same payloads that webhooks deliver). A 30-second keepalive comment is sent
/// so proxies and browsers don't close idle connections.
///
/// Example client (JS):
///   const es = new EventSource('https://api.wzrd.dog/api/events');
///   es.onmessage = e => console.log(JSON.parse(e.data));
pub async fn sse_events(
    State(state): State<super::AppState>,
) -> Sse<impl futures_core::Stream<Item = Result<Event, Infallible>>> {
    let rx = state.event_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|msg| match msg {
        Ok(json) => Some(Ok(Event::default().data(json))),
        // BroadcastStream::Lagged — subscriber was too slow, skip missed events
        Err(_) => None,
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// POST /api/webhook — receives indexer webhook payloads and fans them out to SSE subscribers.
///
/// doghook automatically registers http://127.0.0.1:{port}/api/webhook as a webhook
/// URL at startup, so no manual config is needed.
pub async fn receive_webhook(State(state): State<super::AppState>, body: String) -> StatusCode {
    // Ignore send errors — they just mean no SSE clients are connected right now.
    let _ = state.event_tx.send(body);
    StatusCode::OK
}

pub async fn wallet_js() -> impl IntoResponse {
    (
        [(
            header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        include_str!("../../static/wallet.js"),
    )
}

pub async fn inscriptions_page() -> Html<&'static str> {
    Html(include_str!("../../static/index.html"))
}

pub async fn drc20_page() -> Html<&'static str> {
    Html(include_str!("../../static/index.html"))
}

pub async fn dunes_page() -> Html<&'static str> {
    Html(include_str!("../../static/index.html"))
}

pub async fn lotto_page() -> Html<&'static str> {
    Html(include_str!("../../static/index.html"))
}

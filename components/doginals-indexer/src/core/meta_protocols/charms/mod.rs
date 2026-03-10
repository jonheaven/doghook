//! Charms OP_RETURN protocol parser.
//!
//! Charms spells live entirely inside OP_RETURN outputs. The pushed payload must:
//! - start with the ASCII magic prefix `CHARMS`
//! - contain a CBOR-encoded `CharmsSpell` immediately after the prefix
//! - target `chain_id == "doge"`
//!
//! Dogecoin Core already enforces the standard 80-byte OP_RETURN relay limit, so
//! parsing here is intentionally permissive: if the output is not a Charms spell,
//! or the CBOR payload is malformed, we ignore it silently like Dogetag.

use std::io::Cursor;

use serde::{Deserialize, Serialize};

pub const CHARMS_MAGIC: &[u8] = b"CHARMS";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CharmsSpell {
    pub version: String,
    pub tag: String,
    pub op: String,
    pub id: Vec<u8>,
    pub chain_id: String,
    pub ticker: Option<String>,
    pub name: Option<String>,
    pub amount: Option<u64>,
    pub decimals: Option<u8>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub beam_to: Option<String>,
    pub beam_proof: Option<String>,
    pub txid: String,
    pub vout: u32,
    pub block_height: u64,
    pub block_timestamp: u32,
}

#[derive(Debug, Clone)]
pub struct IndexedCharmsSpell {
    pub spell: CharmsSpell,
    pub raw_cbor: Vec<u8>,
}

pub fn identity_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

/// Attempt to decode a Charms spell from a raw `scriptPubKey` hex string.
///
/// Returns the decoded spell and the original CBOR bytes when:
/// - the script is an OP_RETURN push
/// - the pushed data starts with `CHARMS`
/// - the trailing bytes decode into `CharmsSpell`
/// - `chain_id == "doge"`
/// - `id` is exactly 32 bytes
pub fn try_parse_charms_spell(script_hex: &str) -> Option<IndexedCharmsSpell> {
    let payload = extract_op_return_payload(script_hex)?;
    let cbor = payload.strip_prefix(CHARMS_MAGIC)?;
    let spell: CharmsSpell = ciborium::from_reader(Cursor::new(cbor)).ok()?;

    if spell.chain_id != "doge" || spell.id.len() != 32 {
        return None;
    }

    Some(IndexedCharmsSpell {
        spell,
        raw_cbor: cbor.to_vec(),
    })
}

fn extract_op_return_payload(script_hex: &str) -> Option<Vec<u8>> {
    let hex = script_hex.trim_start_matches("0x");
    if !hex.starts_with("6a") {
        return None;
    }

    let script = hex::decode(hex).ok()?;
    if script.len() < 2 || script[0] != 0x6a {
        return None;
    }

    match script[1] {
        0x00 => None,
        n if n <= 0x4b => {
            let len = n as usize;
            if 2 + len > script.len() {
                return None;
            }
            Some(script[2..2 + len].to_vec())
        }
        0x4c => {
            if script.len() < 3 {
                return None;
            }
            let len = script[2] as usize;
            if 3 + len > script.len() {
                return None;
            }
            Some(script[3..3 + len].to_vec())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{try_parse_charms_spell, CharmsSpell, CHARMS_MAGIC};

    fn spell() -> CharmsSpell {
        CharmsSpell {
            version: "1.0.0".to_string(),
            tag: "t".to_string(),
            op: "mint".to_string(),
            id: vec![0x11; 32],
            chain_id: "doge".to_string(),
            ticker: Some("WOW".to_string()),
            name: Some("Much Wow".to_string()),
            amount: Some(42),
            decimals: Some(8),
            from: Some("DFrom".to_string()),
            to: Some("DTo".to_string()),
            beam_to: None,
            beam_proof: None,
            txid: "abc123".to_string(),
            vout: 1,
            block_height: 123,
            block_timestamp: 456,
        }
    }

    fn op_return_hex(payload: &[u8]) -> String {
        let mut bytes = vec![0x6a];
        if payload.len() <= 0x4b {
            bytes.push(payload.len() as u8);
        } else {
            bytes.push(0x4c);
            bytes.push(payload.len() as u8);
        }
        bytes.extend_from_slice(payload);
        format!("0x{}", hex::encode(bytes))
    }

    #[test]
    fn parses_valid_charms_spell() {
        let spell = spell();
        let mut cbor = Vec::new();
        ciborium::into_writer(&spell, &mut cbor).unwrap();

        let mut payload = CHARMS_MAGIC.to_vec();
        payload.extend_from_slice(&cbor);

        let parsed = try_parse_charms_spell(&op_return_hex(&payload)).unwrap();
        assert_eq!(parsed.spell, spell);
        assert_eq!(parsed.raw_cbor, cbor);
    }

    #[test]
    fn rejects_non_doge_chain() {
        let mut spell = spell();
        spell.chain_id = "btc".to_string();

        let mut cbor = Vec::new();
        ciborium::into_writer(&spell, &mut cbor).unwrap();

        let mut payload = CHARMS_MAGIC.to_vec();
        payload.extend_from_slice(&cbor);

        assert!(try_parse_charms_spell(&op_return_hex(&payload)).is_none());
    }
}

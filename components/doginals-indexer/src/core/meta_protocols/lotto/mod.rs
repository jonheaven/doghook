use rand::rng;
/// DogeLotto meta-protocol structs and fingerprint logic
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LottoDeploy {
    #[serde(alias = "li")]
    pub lotto_id: String,
    #[serde(alias = "te")]
    pub template: LottoTemplate,
    #[serde(alias = "db")]
    pub draw_block: u64,
    #[serde(alias = "cb")]
    pub cutoff_block: u64,
    #[serde(alias = "pk")]
    pub ticket_price_koinu: u64,
    #[serde(alias = "pa")]
    pub prize_pool_address: String,
    #[serde(alias = "fp")]
    pub fee_percent: u8,
    #[serde(alias = "mn")]
    pub main_numbers: NumberConfig,
    #[serde(alias = "bn")]
    pub bonus_numbers: NumberConfig,
    #[serde(alias = "rm")]
    pub resolution_mode: ResolutionMode,
    #[serde(alias = "re")]
    pub rollover_enabled: bool,
    #[serde(alias = "gm")]
    pub guaranteed_min_prize_koinu: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NumberConfig {
    #[serde(alias = "p")]
    pub pick: u16,
    #[serde(alias = "m")]
    pub max: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LottoTemplate {
    #[serde(alias = "closest_wins")]
    ClosestWins,
    #[serde(alias = "6_49_classic")]
    Six49Classic,
    #[serde(alias = "life_annuity")]
    LifeAnnuity,
    #[serde(alias = "powerball_dual_drum")]
    PowerballDualDrum,
    #[serde(alias = "rollover_jackpot")]
    RolloverJackpot,
    #[serde(alias = "always_winner")]
    AlwaysWinner,
    #[serde(alias = "custom")]
    #[serde(alias = "deno")]
    Custom,
    #[serde(alias = "closest_fingerprint")]
    ClosestFingerprint,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResolutionMode {
    #[serde(alias = "always_winner")]
    AlwaysWinner,
    #[serde(alias = "exact_only_with_rollover")]
    ExactOnlyWithRollover,
    #[serde(alias = "closest_wins")]
    ClosestWins,
    #[serde(alias = "closest_fingerprint")]
    ClosestFingerprint,
}

#[derive(Debug, Clone)]
pub struct LottoDraw {
    pub main_numbers: Vec<u16>,
    pub bonus_numbers: Vec<u16>,
}

pub const GLOBAL_NUMBER_MIN: u16 = 1;
pub const CLASSIC_MAX: u16 = 49;

pub const FINGERPRINT_TIER_BPS: [u32; 4] = [5500, 2000, 1000, 500];

pub fn compute_ticket_fingerprint(seed_numbers: &[u16]) -> [u8; 32] {
    let mut sorted = seed_numbers.to_vec();
    sorted.sort_unstable();
    let input = sorted
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let digest = Sha256::digest(input.as_bytes());
    let mut fp = [0u8; 32];
    fp.copy_from_slice(&digest);
    fp
}

pub fn u256_abs_diff(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    if a >= b {
        sub_be_32(a, b)
    } else {
        sub_be_32(b, a)
    }
}

fn sub_be_32(large: &[u8; 32], small: &[u8; 32]) -> [u8; 32] {
    let mut out = [0u8; 32];
    let mut borrow = 0i16;
    for i in (0..32).rev() {
        let mut value = large[i] as i16 - small[i] as i16 - borrow;
        if value < 0 {
            value += 256;
            borrow = 1;
        } else {
            borrow = 0;
        }
        out[i] = value as u8;
    }
    out
}

pub fn score_ticket(ticket: &[u16], drawn: &[u16]) -> u64 {
    let matches = ticket.iter().filter(|n| drawn.contains(n)).count() as u64;
    matches * matches
}

pub fn count_classic_matches(ticket: &[u16], drawn: &[u16]) -> usize {
    ticket.iter().filter(|n| drawn.contains(n)).count()
}

pub fn derive_classic_numbers(fp_bytes: &[u8]) -> Vec<u16> {
    let mut numbers = Vec::with_capacity(6);
    for i in (0..fp_bytes.len()).step_by(2) {
        if numbers.len() >= 6 {
            break;
        }
        let raw = u16::from_be_bytes([fp_bytes[i], fp_bytes[i + 1]]);
        let number = (raw % CLASSIC_MAX) + GLOBAL_NUMBER_MIN;
        if !numbers.contains(&number) {
            numbers.push(number);
        }
    }
    numbers
}

pub fn derive_classic_drawn_numbers(block_hash: &str) -> Vec<u16> {
    let hash_hex = block_hash.trim_start_matches("0x");
    let bytes = hex::decode(hash_hex).unwrap_or_default();
    derive_classic_numbers(&bytes)
}

fn is_deno_lotto(lotto_id: &str) -> bool {
    lotto_id.eq_ignore_ascii_case("deno") || lotto_id == "Ðeno"
}

fn derive_unique_numbers_from_hash(block_hash: &str, pick: usize, max: u16) -> Vec<u16> {
    let mut out = Vec::with_capacity(pick);
    let mut counter: u32 = 0;
    while out.len() < pick {
        let input = format!("{}:{}", block_hash, counter);
        let digest = Sha256::digest(input.as_bytes());
        for chunk in digest.chunks_exact(2) {
            if out.len() >= pick {
                break;
            }
            let raw = u16::from_be_bytes([chunk[0], chunk[1]]);
            let value = (raw % max) + GLOBAL_NUMBER_MIN;
            if !out.contains(&value) {
                out.push(value);
            }
        }
        counter = counter.saturating_add(1);
    }
    out.sort_unstable();
    out
}

pub fn derive_draw_for_deploy(block_hash: &str, deploy: &LottoDeploy) -> LottoDraw {
    if is_deno_lotto(&deploy.lotto_id) {
        let main = derive_unique_numbers_from_hash(block_hash, 20, deploy.main_numbers.max.max(20));
        return LottoDraw {
            main_numbers: main,
            bonus_numbers: vec![],
        };
    }

    let main = derive_classic_drawn_numbers(block_hash);
    LottoDraw {
        main_numbers: main,
        bonus_numbers: vec![],
    }
}

pub fn classic_prize_bps(matches: usize) -> u32 {
    match matches {
        6 => 10000,
        5 => 5000,
        4 => 1000,
        _ => 0,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LottoMint {
    #[serde(alias = "li")]
    pub lotto_id: String,
    #[serde(alias = "ti")]
    pub ticket_id: String,
    #[serde(alias = "sn", default)]
    pub seed_numbers: Vec<u16>,
    #[serde(alias = "lm", default)]
    pub luck_marks: Option<Vec<u16>>,
    #[serde(alias = "tp", default)]
    pub tip_percent: u8,
}

pub fn try_parse_lotto_deploy(body: &[u8]) -> Option<LottoDeploy> {
    let value = serde_json::from_slice::<serde_json::Value>(body).ok()?;
    let protocol = value
        .get("p")
        .and_then(|v| v.as_str())
        .unwrap_or("DogeLotto");
    if protocol != "DogeLotto" && protocol != "dl" {
        return None;
    }
    let op = value.get("op").and_then(|v| v.as_str()).unwrap_or("deploy");
    if op != "deploy" && op != "d" {
        return None;
    }
    serde_json::from_value(value).ok()
}

pub fn try_parse_lotto_mint(body: &[u8]) -> Option<LottoMint> {
    let value = serde_json::from_slice::<serde_json::Value>(body).ok()?;
    let protocol = value
        .get("p")
        .and_then(|v| v.as_str())
        .unwrap_or("DogeLotto");
    if protocol != "DogeLotto" && protocol != "dl" {
        return None;
    }
    let op = value.get("op").and_then(|v| v.as_str()).unwrap_or("mint");
    if op != "mint" && op != "m" {
        return None;
    }
    let mut parsed: LottoMint = serde_json::from_value(value).ok()?;
    if parsed.seed_numbers.is_empty() {
        if let Some(marks) = parsed.luck_marks.clone() {
            parsed.seed_numbers = marks;
        }
    }
    Some(parsed)
}

pub fn validate_mint_against_deploy(mint: &LottoMint, deploy: &LottoDeploy) -> bool {
    if mint.seed_numbers.len() != deploy.main_numbers.pick as usize {
        return false;
    }
    let mut sorted = mint.seed_numbers.clone();
    sorted.sort_unstable();
    sorted.dedup();
    if sorted.len() != deploy.main_numbers.pick as usize {
        return false;
    }
    if sorted
        .iter()
        .any(|number| *number < GLOBAL_NUMBER_MIN || *number > deploy.main_numbers.max)
    {
        return false;
    }
    if is_deno_lotto(&deploy.lotto_id) {
        if let Some(marks) = &mint.luck_marks {
            let mut marks_sorted = marks.clone();
            marks_sorted.sort_unstable();
            marks_sorted.dedup();
            if marks_sorted != sorted {
                return false;
            }
        }
    }
    true
}

impl NumberConfig {
    pub fn has_numbers(&self) -> bool {
        self.pick > 0
    }
    pub fn is_disabled(&self) -> bool {
        self.pick == 0
    }
}

use rand::prelude::*;

pub fn quickpick_for_config(config: &NumberConfig) -> Vec<u16> {
    if config.pick == 0 {
        return vec![];
    }
    let mut rng = rng();
    let mut numbers = Vec::new();
    while numbers.len() < config.pick as usize {
        let num = rng.random_range(GLOBAL_NUMBER_MIN..=config.max);
        if !numbers.contains(&num) {
            numbers.push(num);
        }
    }
    numbers.sort_unstable();
    numbers
}

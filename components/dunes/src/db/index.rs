// ...existing code...
use super::{cache::index_cache::IndexCache, pg_get_max_dune_number, pg_roll_back_block};
use bitcoin::hashes::Hash;
use bitcoin::{
    absolute::LockTime,
    transaction::{TxOut, Version},
};
use bitcoin::{Amount, Network, ScriptBuf, Transaction};
use deadpool_postgres::Client;
use dogecoin::utils::Context;
use dogecoin::{
    try_info,
    types::{DogecoinBlockData, DogecoinTransactionData},
};
use doginals_parser::{Artifact, Dunestone, Flaw};
use postgres::pg_begin;
use std::collections::HashMap;

// Use types from dogecoin::types
// ...existing code...

use crate::{
    db::cache::transaction_location::TransactionLocation, utils::monitoring::PrometheusMonitoring,
};
pub fn get_dune_genesis_block_height(network: Network) -> u64 {
    // Dogecoin Dunes activation height is intentionally unset for now.
    // Use u64::MAX so indexing stays disabled until explicitly activated.
    match network {
        Network::Bitcoin => u64::MAX,
        Network::Testnet | Network::Testnet4 => u64::MAX,
        Network::Signet => u64::MAX,
        // Regtest remains available for local testing.
        Network::Regtest => 0,
    }
}

/// Transforms a Bitcoin transaction from a Chainhook format to a rust bitcoin crate format so it can be parsed by the ord crate
/// to look for `Artifact`s. Also, takes all non-OP_RETURN outputs and returns them so they can be used later to receive dunes.
fn bitcoin_tx_from_chainhook_tx(
    block: &DogecoinBlockData,
    tx: &DogecoinTransactionData,
) -> (Transaction, HashMap<u32, ScriptBuf>, Option<u32>, u32) {
    let mut inputs = Vec::with_capacity(tx.metadata.inputs.len());
    let mut outputs = Vec::with_capacity(tx.metadata.outputs.len());
    let mut eligible_outputs = HashMap::new();
    let mut first_eligible_output: Option<u32> = None;
    for (i, output) in tx.metadata.outputs.iter().enumerate() {
        let script = ScriptBuf::from_bytes(output.get_script_pubkey_bytes());
        if !script.is_op_return() {
            eligible_outputs.insert(i as u32, script.clone());
            if first_eligible_output.is_none() {
                first_eligible_output = Some(i as u32);
            }
        }
        outputs.push(TxOut {
            value: Amount::from_sat(output.value),
            script_pubkey: script,
        });
    }
    for input in tx.metadata.inputs.iter() {
        inputs.push(bitcoin::TxIn {
            previous_output: bitcoin::OutPoint {
                txid: bitcoin::Txid::from_raw_hash(
                    Hash::from_slice(&input.previous_output.txid.get_hash_bytes()).unwrap(),
                ),
                vout: input.previous_output.vout,
            },
            script_sig: bitcoin::ScriptBuf::from_bytes(input.script_sig.as_bytes().to_vec()),
            sequence: bitcoin::Sequence(input.sequence),
            witness: bitcoin::Witness::default(),
        });
    }
    (
        Transaction {
            version: Version::TWO,
            lock_time: LockTime::from_time(block.timestamp).unwrap(),
            input: inputs,
            output: outputs,
        },
        eligible_outputs,
        first_eligible_output,
        tx.metadata.outputs.len() as u32,
    )
}

/// Index a Bitcoin block for dunes data.
pub async fn index_block(
    pg_client: &mut Client,
    index_cache: &mut IndexCache,
    block: &mut DogecoinBlockData,
    prometheus: &PrometheusMonitoring,
    ctx: &Context,
) -> Result<(), String> {
    let stopwatch = std::time::Instant::now();
    let block_hash = &block.block_identifier.hash;
    let block_height = block.block_identifier.index;
    try_info!(ctx, "DunesIndexer indexing block #{block_height}...");

    // Track operation counts
    let mut etching_count: u64 = 0;
    let mut mint_count: u64 = 0;
    let mut edict_count: u64 = 0;
    let mut cenotaph_etching_count: u64 = 0;
    let mut cenotaph_mint_count: u64 = 0;
    let mut cenotaph_count: u64 = 0;
    let mut inputs_count: u64 = 0;

    let mut db_tx = pg_begin(pg_client).await.unwrap();
    index_cache.reset_max_dune_number(&mut db_tx).await;

    // Measure parsing time
    let parsing_start = std::time::Instant::now();

    for tx in block.transactions.iter() {
        let (transaction, eligible_outputs, first_eligible_output, total_outputs) =
            bitcoin_tx_from_chainhook_tx(block, tx);
        let tx_index = tx.metadata.index;
        let tx_id = &tx.transaction_identifier.hash;
        let location = TransactionLocation {
            network: index_cache.network,
            block_hash: block_hash.clone(),
            block_height,
            tx_index,
            tx_id: tx_id.clone(),
            timestamp: block.timestamp,
        };
        index_cache
            .begin_transaction(
                location,
                &tx.metadata.inputs,
                eligible_outputs,
                first_eligible_output,
                total_outputs,
                &mut db_tx,
                ctx,
            )
            .await;
        if let Some(artifact) = Dunestone::decipher(&transaction) {
            match artifact {
                Artifact::Dunestone(dunestone) => {
                    index_cache
                        .apply_dunestone(&dunestone, &mut db_tx, ctx)
                        .await;
                    if let Some(etching) = dunestone.etching {
                        index_cache
                            .apply_etching(
                                &etching,
                                &mut db_tx,
                                ctx,
                                &mut etching_count,
                                &transaction,
                                &mut inputs_count,
                            )
                            .await?;
                    }
                    if let Some(mint_dune_id) = dunestone.mint {
                        index_cache
                            .apply_mint(&mint_dune_id, &mut db_tx, ctx, &mut mint_count)
                            .await;
                    }
                    for edict in dunestone.edicts.iter() {
                        index_cache
                            .apply_edict(edict, &mut db_tx, ctx, &mut edict_count)
                            .await;
                    }
                }
                Artifact::Cenotaph(cenotaph) => {
                    index_cache
                        .apply_cenotaph(&cenotaph, &mut db_tx, ctx, &mut cenotaph_count)
                        .await;

                    if cenotaph.flaw != Some(Flaw::Varint) {
                        if let Some(etching) = cenotaph.etching {
                            index_cache
                                .apply_cenotaph_etching(
                                    &etching,
                                    &mut db_tx,
                                    ctx,
                                    &mut cenotaph_etching_count,
                                    &transaction,
                                    &mut inputs_count,
                                )
                                .await?;
                        }
                        if let Some(mint_dune_id) = cenotaph.mint {
                            index_cache
                                .apply_cenotaph_mint(
                                    &mint_dune_id,
                                    &mut db_tx,
                                    ctx,
                                    &mut cenotaph_mint_count,
                                )
                                .await;
                        }
                    }
                }
            }
        }
        index_cache.end_transaction(&mut db_tx, ctx);
    }
    prometheus.metrics_record_dune_parsing_time(parsing_start.elapsed().as_millis() as f64);

    // Measure computation time
    let computation_start = std::time::Instant::now();
    index_cache.end_block();
    prometheus.metrics_record_dune_computation_time(computation_start.elapsed().as_millis() as f64);

    // Measure database write time
    let dune_db_write_start = std::time::Instant::now();
    index_cache.db_cache.flush(&mut db_tx, ctx).await;
    db_tx
        .commit()
        .await
        .expect("Unable to commit pg transaction");
    prometheus.metrics_record_dune_db_write_time(dune_db_write_start.elapsed().as_millis() as f64);

    prometheus.metrics_record_dunes_etching_per_block(etching_count);
    prometheus.metrics_record_dunes_edict_per_block(edict_count);
    prometheus.metrics_record_dunes_mint_per_block(mint_count);
    prometheus.metrics_record_dunes_cenotaph_per_block(cenotaph_count);
    prometheus.metrics_record_dunes_cenotaph_etching_per_block(cenotaph_etching_count);
    prometheus.metrics_record_dunes_cenotaph_mint_per_block(cenotaph_mint_count);
    prometheus.metrics_record_dunes_etching_inputs_checked_per_block(inputs_count);
    // Record metrics
    prometheus.metrics_block_indexed(block_height);
    let current_dune_number = pg_get_max_dune_number(pg_client).await;
    prometheus.metrics_dune_indexed(current_dune_number as u64);
    prometheus.metrics_record_dunes_per_block(etching_count);

    // Record overall processing time
    let elapsed = stopwatch.elapsed();
    prometheus.metrics_record_block_processing_time(elapsed.as_millis() as f64);
    try_info!(
        ctx,
        "DunesIndexer indexed block #{block_height}: {etching_count} etchings, {mint_count} mints, {edict_count} edicts, {cenotaph_count} cenotaphs ({cenotaph_etching_count} etchings, {cenotaph_mint_count} mints) in {}s",
        elapsed.as_secs_f32()
    );

    Ok(())
}

/// Roll back a Bitcoin block because of a re-org.
pub async fn roll_back_block(pg_client: &mut Client, block_height: u64, ctx: &Context) {
    let stopwatch = std::time::Instant::now();
    try_info!(ctx, "Rolling back block {block_height}...");
    let mut db_tx = pg_client
        .transaction()
        .await
        .expect("Unable to begin block roll back pg transaction");
    pg_roll_back_block(block_height, &mut db_tx, ctx).await;
    db_tx
        .commit()
        .await
        .expect("Unable to commit pg transaction");
    try_info!(
        ctx,
        "Block {block_height} rolled back in {elapsed:.4}s",
        elapsed = stopwatch.elapsed().as_secs_f32()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use dogecoin::types::{
        dogecoin::{OutPoint, TxIn, TxOut},
        BlockIdentifier, DogecoinBlockData, DogecoinBlockMetadata, DogecoinNetwork,
        DogecoinTransactionData, DogecoinTransactionMetadata, TransactionIdentifier,
    };
    use doginals_parser::Artifact;

    fn build_block(block_height: u64, block_hash_hex: &str, timestamp: u32) -> DogecoinBlockData {
        DogecoinBlockData {
            block_identifier: BlockIdentifier {
                hash: format!("0x{}", block_hash_hex.to_lowercase()),
                index: block_height,
            },
            parent_block_identifier: BlockIdentifier {
                hash: "0x0000000000000000000000000000000000000000000000000000000000000000"
                    .to_string(),
                index: block_height - 1,
            },
            timestamp,
            transactions: vec![],
            metadata: DogecoinBlockMetadata {
                network: DogecoinNetwork::Mainnet,
            },
        }
    }

    fn build_tx(
        txid: &str,
        prev_txid: &str,
        outputs: Vec<(&str, u64)>,
        index: u32,
    ) -> DogecoinTransactionData {
        DogecoinTransactionData {
            transaction_identifier: TransactionIdentifier::new(txid),
            operations: vec![],
            metadata: DogecoinTransactionMetadata {
                inputs: vec![TxIn {
                    previous_output: OutPoint {
                        txid: TransactionIdentifier::new(prev_txid),
                        vout: 0,
                        value: 0,
                        block_height: 0,
                    },
                    script_sig: String::new(),
                    sequence: 4_294_967_293,
                }],
                outputs: outputs
                    .into_iter()
                    .map(|(script_pubkey, value)| TxOut {
                        script_pubkey: script_pubkey.to_string(),
                        value,
                    })
                    .collect(),
                doginal_operations: vec![],
                drc20_operation: None,
                proof: None,
                fee: 0,
                index,
            },
        }
    }

    #[test]
    fn valid_and_invalid_etch_output_selection_and_parsing() {
        let block = build_block(
            840_021,
            "00000000000000000001a6a69ead163c499c0543dcef13c05499a798addb638f",
            1_713_583_272,
        );

        let tx_valid =
            build_tx(
                "3a11c5bc4eee38645934607ba63e0d7ac834d399e53c7c06a0ced093a711f1a2",
                "1cf46d1a3192e5cdcce62441a7a40691ed4f7e34dc97dd3bfc7f96ff2069846e",
                vec![
                ("0x5120f1e73bbd97fd0eac833e781abdbea9c223951aede9e2275ac6c03e8c1b24394b", 546),
                ("0x5120f1e73bbd97fd0eac833e781abdbea9c223951aede9e2275ac6c03e8c1b24394b", 546),
                ("0x6a5d22020704a7e6e7dbbcf1f2b38203010003800105bded070690c8020a6408aed3191601", 0),
            ],
                0,
            );
        let tx_invalid = build_tx(
            "66d084fe5e206c7183293d1e379caa2011e7750018c65dfd2fd3174ea9f298fc",
            "871fb5e4042dca1549326da5848bd2257d6e609984a0cfb867e4ff24a56806d0",
            vec![
                (
                    "0x6a5d1b020304cfb4c2acf497b13e0380068094ebdc030a8094ebdc030801",
                    0,
                ),
                (
                    "0x5120f1e73bbd97fd0eac833e781abdbea9c223951aede9e2275ac6c03e8c1b24394b",
                    546,
                ),
                ("0x0014f32b49757996ef8db8d3d029b3dc997560e77d12", 15_765),
            ],
            1,
        );

        let (tx1, eligible1, first_eligible1, total_outputs1) =
            bitcoin_tx_from_chainhook_tx(&block, &tx_valid);
        assert_eq!(total_outputs1, 3);
        assert_eq!(first_eligible1, Some(0));
        assert!(eligible1.contains_key(&0));
        assert!(eligible1.contains_key(&1));
        assert!(!eligible1.contains_key(&2));

        let (tx2, eligible2, first_eligible2, total_outputs2) =
            bitcoin_tx_from_chainhook_tx(&block, &tx_invalid);
        assert_eq!(total_outputs2, 3);
        assert_eq!(first_eligible2, Some(1));
        assert!(eligible2.contains_key(&1));
        assert!(eligible2.contains_key(&2));
        assert!(!eligible2.contains_key(&0));

        let art1 = Dunestone::decipher(&tx1).expect("dunestone");
        let Artifact::Dunestone(rs1) = art1 else {
            panic!("expected Dunestone");
        };
        assert!(rs1.etching.is_some());

        let art2 = Dunestone::decipher(&tx2).expect("dunestone");
        if let Artifact::Dunestone(rs2) = art2 {
            if let Some(e2) = rs2.etching.as_ref() {
                let is_incomplete = e2.divisibility.is_none()
                    && e2.premine.is_none()
                    && e2.dune.is_none()
                    && e2.spacers.is_none()
                    && e2.symbol.is_none()
                    && e2.terms.is_none()
                    && !e2.turbo;
                assert!(
                    is_incomplete,
                    "invalid tx should not produce a complete etching"
                );
            }
        }
    }
}

use super::TransactionIdentifier;

/// A transaction input, which defines old coins to be consumed.
/// Dogecoin does not support SegWit, so there is no witness field.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub struct TxIn {
    /// The reference to the previous output that is being used as an input.
    pub previous_output: OutPoint,
    /// The script which pushes values on the stack which will cause
    /// the referenced output's script to be accepted.
    pub script_sig: String,
    /// The sequence number, which suggests to miners which of two
    /// conflicting transactions should be preferred, or 0xFFFFFFFF
    /// to ignore this feature.
    pub sequence: u32,
}

/// A transaction output, which defines new coins to be created from old ones.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub struct TxOut {
    /// The value of the output, in koinu (Dogecoin's base unit, analogous to satoshis).
    pub value: u64,
    /// The script which must be satisfied for the output to be spent.
    pub script_pubkey: String,
}

/// A reference to a transaction output.
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct OutPoint {
    /// The referenced transaction's txid.
    pub txid: TransactionIdentifier,
    /// The index of the referenced output in its transaction's vout.
    pub vout: u32,
    /// The value of the referenced output.
    pub value: u64,
    /// The block height where the referenced output was created.
    pub block_height: u64,
}

impl TxOut {
    pub fn get_script_pubkey_bytes(&self) -> Vec<u8> {
        hex::decode(self.get_script_pubkey_hex()).expect("not provided for coinbase txs")
    }

    pub fn get_script_pubkey_hex(&self) -> &str {
        &self.script_pubkey[2..]
    }
}

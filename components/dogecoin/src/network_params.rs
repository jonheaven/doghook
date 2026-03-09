/// Dogecoin mainnet network parameters.
///
/// Reference: https://github.com/dogecoin/dogecoin/blob/master/src/chainparams.cpp
pub mod mainnet {
    /// Version byte for P2PKH addresses (Base58Check prefix → 'D').
    pub const PUBKEY_HASH: u8 = 0x1e; // 30

    /// Version byte for P2SH addresses (Base58Check prefix → 'A' or '9').
    pub const SCRIPT_HASH: u8 = 0x16; // 22

    /// WIF private key prefix (Base58Check prefix → 'Q' or '6').
    pub const WIF: u8 = 0x9e; // 158

    /// BIP32 extended public key version bytes ("dgub").
    pub const BIP32_PUBLIC: u32 = 0x02_FA_CA_FD;

    /// BIP32 extended private key version bytes ("dgpv").
    pub const BIP32_PRIVATE: u32 = 0x02_FA_C3_98;
}

/// Dogecoin testnet network parameters.
pub mod testnet {
    /// Version byte for P2PKH addresses (Base58Check prefix → 'n').
    pub const PUBKEY_HASH: u8 = 0x71; // 113

    /// Version byte for P2SH addresses (Base58Check prefix → '2').
    pub const SCRIPT_HASH: u8 = 0xc4; // 196

    /// WIF private key prefix (Base58Check prefix → 'c').
    pub const WIF: u8 = 0xf1; // 241

    /// BIP32 extended public key version bytes ("tgub").
    pub const BIP32_PUBLIC: u32 = 0x04_35_87_CF;

    /// BIP32 extended private key version bytes ("tgpv").
    pub const BIP32_PRIVATE: u32 = 0x04_35_83_94;
}

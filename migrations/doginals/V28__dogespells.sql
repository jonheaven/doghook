-- DogeSpells protocol indexing.
-- Spells are append-only OP_RETURN events; balances and NFT metadata are derived tables.

CREATE TABLE IF NOT EXISTS dogespells (
    id               BIGSERIAL PRIMARY KEY,
    txid             TEXT        NOT NULL,
    vout             BIGINT      NOT NULL,
    block_height     NUMERIC     NOT NULL,
    block_timestamp  BIGINT      NOT NULL,
    version          TEXT        NOT NULL,
    tag              TEXT        NOT NULL,
    op               TEXT        NOT NULL,
    identity         TEXT        NOT NULL,
    chain_id         TEXT        NOT NULL,
    ticker           TEXT,
    name             TEXT,
    amount           NUMERIC,
    decimals         SMALLINT,
    from_addr        TEXT,
    to_addr          TEXT,
    beam_to          TEXT,
    beam_proof       TEXT,
    raw_cbor         BYTEA       NOT NULL,
    UNIQUE (txid, vout)
);

CREATE TABLE IF NOT EXISTS dogespells_balances (
    ticker   TEXT    NOT NULL,
    address  TEXT    NOT NULL,
    balance  NUMERIC NOT NULL,
    PRIMARY KEY (ticker, address)
);

CREATE TABLE IF NOT EXISTS dogespells_nfts (
    identity       TEXT PRIMARY KEY,
    ticker         TEXT,
    metadata_json  JSONB
);

CREATE INDEX IF NOT EXISTS dogespells_block_height_idx ON dogespells (block_height DESC);
CREATE INDEX IF NOT EXISTS dogespells_identity_idx ON dogespells (identity);
CREATE INDEX IF NOT EXISTS dogespells_ticker_idx ON dogespells (ticker);
CREATE INDEX IF NOT EXISTS dogespells_txid_idx ON dogespells (txid);
CREATE INDEX IF NOT EXISTS dogespells_balances_address_idx ON dogespells_balances (address);
CREATE INDEX IF NOT EXISTS dogespells_nfts_ticker_idx ON dogespells_nfts (ticker);

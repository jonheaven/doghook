-- Charms protocol indexing.
-- Spells are append-only OP_RETURN events; balances and NFT metadata are derived tables.

CREATE TABLE IF NOT EXISTS charms_spells (
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

CREATE TABLE IF NOT EXISTS charms_balances (
    ticker   TEXT    NOT NULL,
    address  TEXT    NOT NULL,
    balance  NUMERIC NOT NULL,
    PRIMARY KEY (ticker, address)
);

CREATE TABLE IF NOT EXISTS charms_nfts (
    identity       TEXT PRIMARY KEY,
    ticker         TEXT,
    metadata_json  JSONB
);

CREATE INDEX IF NOT EXISTS charms_spells_block_height_idx ON charms_spells (block_height DESC);
CREATE INDEX IF NOT EXISTS charms_spells_identity_idx ON charms_spells (identity);
CREATE INDEX IF NOT EXISTS charms_spells_ticker_idx ON charms_spells (ticker);
CREATE INDEX IF NOT EXISTS charms_spells_txid_idx ON charms_spells (txid);
CREATE INDEX IF NOT EXISTS charms_balances_address_idx ON charms_balances (address);
CREATE INDEX IF NOT EXISTS charms_nfts_ticker_idx ON charms_nfts (ticker);

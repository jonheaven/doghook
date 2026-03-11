-- DoginalMarket Protocol (DMP) — open inscription-based marketplace for Doginals.
-- Every listing, bid, and settlement is an inscription with "protocol":"DMP".
-- PSBTs live off-chain (IPFS/Arweave CID only) — never stored on-chain.
-- This schema indexes the on-chain inscription activity only.

CREATE TABLE IF NOT EXISTS dmp_listings (
    listing_id          TEXT        NOT NULL,   -- inscription_id of the listing inscription
    inscription_id      TEXT        NOT NULL,   -- same as listing_id (canonical key)
    seller              TEXT        NOT NULL,   -- Dogecoin seller address
    price_koinu         BIGINT      NOT NULL,   -- asking price in koinu
    psbt_cid            TEXT        NOT NULL,   -- ipfs://Qm... or ar://... CID
    expiry_height       BIGINT      NOT NULL,   -- block height at which listing expires
    nonce               BIGINT      NOT NULL,
    signature           TEXT        NOT NULL,   -- hex sig of the canonical fields
    block_height        BIGINT      NOT NULL,
    block_timestamp     BIGINT      NOT NULL,
    cancelled           BOOLEAN     NOT NULL DEFAULT FALSE,
    settled             BOOLEAN     NOT NULL DEFAULT FALSE,
    PRIMARY KEY (listing_id)
);

CREATE TABLE IF NOT EXISTS dmp_bids (
    bid_id              TEXT        NOT NULL,   -- inscription_id of the bid inscription
    listing_id          TEXT        NOT NULL,   -- references dmp_listings.listing_id
    bidder              TEXT        NOT NULL,   -- Dogecoin bidder address (seller field in spec)
    price_koinu         BIGINT      NOT NULL,
    psbt_cid            TEXT        NOT NULL,
    expiry_height       BIGINT      NOT NULL,
    nonce               BIGINT      NOT NULL,
    signature           TEXT        NOT NULL,
    block_height        BIGINT      NOT NULL,
    block_timestamp     BIGINT      NOT NULL,
    cancelled           BOOLEAN     NOT NULL DEFAULT FALSE,
    settled             BOOLEAN     NOT NULL DEFAULT FALSE,
    PRIMARY KEY (bid_id)
);

CREATE TABLE IF NOT EXISTS dmp_settlements (
    settlement_id       TEXT        NOT NULL,   -- inscription_id of the settle inscription
    listing_id          TEXT        NOT NULL,
    bid_id              TEXT,                   -- optional: which bid was accepted
    settler             TEXT        NOT NULL,   -- address that broadcast the settle
    psbt_cid            TEXT        NOT NULL,
    nonce               BIGINT      NOT NULL,
    signature           TEXT        NOT NULL,
    block_height        BIGINT      NOT NULL,
    block_timestamp     BIGINT      NOT NULL,
    PRIMARY KEY (settlement_id)
);

CREATE TABLE IF NOT EXISTS dmp_cancels (
    cancel_id           TEXT        NOT NULL,   -- inscription_id of the cancel inscription
    listing_id          TEXT        NOT NULL,
    canceller           TEXT        NOT NULL,
    nonce               BIGINT      NOT NULL,
    signature           TEXT        NOT NULL,
    block_height        BIGINT      NOT NULL,
    block_timestamp     BIGINT      NOT NULL,
    PRIMARY KEY (cancel_id)
);

-- Indexes for common query patterns
CREATE INDEX IF NOT EXISTS dmp_listings_seller_idx
    ON dmp_listings (seller);
CREATE INDEX IF NOT EXISTS dmp_listings_height_idx
    ON dmp_listings (block_height);
CREATE INDEX IF NOT EXISTS dmp_listings_expiry_idx
    ON dmp_listings (expiry_height) WHERE NOT cancelled AND NOT settled;

CREATE INDEX IF NOT EXISTS dmp_bids_listing_idx
    ON dmp_bids (listing_id);
CREATE INDEX IF NOT EXISTS dmp_bids_bidder_idx
    ON dmp_bids (bidder);
CREATE INDEX IF NOT EXISTS dmp_bids_height_idx
    ON dmp_bids (block_height);

CREATE INDEX IF NOT EXISTS dmp_settlements_listing_idx
    ON dmp_settlements (listing_id);
CREATE INDEX IF NOT EXISTS dmp_settlements_height_idx
    ON dmp_settlements (block_height);

CREATE INDEX IF NOT EXISTS dmp_cancels_listing_idx
    ON dmp_cancels (listing_id);
CREATE INDEX IF NOT EXISTS dmp_cancels_height_idx
    ON dmp_cancels (block_height);

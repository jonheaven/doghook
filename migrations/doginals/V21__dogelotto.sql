-- DogeLotto — the canonical trustless on-chain lotto on Dogecoin.
-- Inscriptions carry JSON with "p":"DogeLotto"; the protocol parser
-- validates deploys and mints before writing to these tables.

CREATE TABLE IF NOT EXISTS dogelotto_lotteries (
    lotto_id                         TEXT        NOT NULL,
    inscription_id                   TEXT        NOT NULL,
    deploy_tx_id                     TEXT        NOT NULL,
    deploy_height                    BIGINT      NOT NULL,
    deploy_timestamp                 BIGINT      NOT NULL,
    draw_block                       BIGINT      NOT NULL,
    ticket_price_koinu               BIGINT      NOT NULL,
    prize_pool_address               TEXT        NOT NULL,
    fee_percent                      INTEGER     NOT NULL,
    resolution_mode                  TEXT        NOT NULL,
    rollover_enabled                 BOOLEAN     NOT NULL,
    guaranteed_min_prize_koinu       BIGINT,
    resolved                         BOOLEAN     NOT NULL DEFAULT FALSE,
    resolved_height                  BIGINT,
    resolved_timestamp               BIGINT,
    resolved_block_hash              TEXT,
    drawn_numbers                    INTEGER[],
    verified_ticket_count            BIGINT,
    verified_sales_koinu             BIGINT,
    fee_koinu                        BIGINT,
    net_prize_koinu                  BIGINT,
    rollover_occurred                BOOLEAN     NOT NULL DEFAULT FALSE,
    PRIMARY KEY (lotto_id)
);

CREATE TABLE IF NOT EXISTS dogelotto_tickets (
    inscription_id                   TEXT        NOT NULL,
    lotto_id                         TEXT        NOT NULL,
    ticket_id                        TEXT        NOT NULL,
    tx_id                            TEXT        NOT NULL,
    minted_height                    BIGINT      NOT NULL,
    minted_timestamp                 BIGINT      NOT NULL,
    seed_numbers                     INTEGER[]   NOT NULL,
    PRIMARY KEY (inscription_id),
    CONSTRAINT dogelotto_tickets_lotto_ticket_id_key UNIQUE (lotto_id, ticket_id),
    CONSTRAINT dogelotto_tickets_lotto_id_fkey FOREIGN KEY (lotto_id)
        REFERENCES dogelotto_lotteries (lotto_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS dogelotto_winners (
    lotto_id                         TEXT        NOT NULL,
    inscription_id                   TEXT        NOT NULL,
    ticket_id                        TEXT        NOT NULL,
    resolved_height                  BIGINT      NOT NULL,
    rank                             INTEGER     NOT NULL,
    score                            BIGINT      NOT NULL,
    payout_bps                       INTEGER     NOT NULL,
    payout_koinu                     BIGINT      NOT NULL,
    seed_numbers                     INTEGER[]   NOT NULL,
    drawn_numbers                    INTEGER[]   NOT NULL,
    PRIMARY KEY (lotto_id, inscription_id),
    CONSTRAINT dogelotto_winners_lotto_id_fkey FOREIGN KEY (lotto_id)
        REFERENCES dogelotto_lotteries (lotto_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS dogelotto_lotteries_draw_block_idx
    ON dogelotto_lotteries (draw_block);
CREATE INDEX IF NOT EXISTS dogelotto_lotteries_resolved_height_idx
    ON dogelotto_lotteries (resolved_height);
CREATE INDEX IF NOT EXISTS dogelotto_tickets_lotto_height_idx
    ON dogelotto_tickets (lotto_id, minted_height);
CREATE INDEX IF NOT EXISTS dogelotto_winners_resolved_height_idx
    ON dogelotto_winners (resolved_height);

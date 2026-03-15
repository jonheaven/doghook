-- DogeLotto meta-protocol: lotteries, tickets, winners, and burn mechanics.
-- Consolidated final schema (development migrations V21–V32 squashed).

CREATE TABLE IF NOT EXISTS dogelotto_lotteries (
    lotto_id                         TEXT        NOT NULL,
    inscription_id                   TEXT        NOT NULL,
    deploy_tx_id                     TEXT        NOT NULL,
    deploy_height                    BIGINT      NOT NULL,
    deploy_timestamp                 BIGINT      NOT NULL,
    template                         TEXT        NOT NULL DEFAULT 'custom',
    draw_block                       BIGINT      NOT NULL,
    cutoff_block                     BIGINT      NOT NULL,
    ticket_price_koinu               BIGINT      NOT NULL,
    prize_pool_address               TEXT        NOT NULL,
    fee_percent                      INTEGER     NOT NULL,
    main_numbers_pick                INTEGER     NOT NULL DEFAULT 69,
    main_numbers_max                 INTEGER     NOT NULL DEFAULT 420,
    bonus_numbers_pick               INTEGER     NOT NULL DEFAULT 0,
    bonus_numbers_max                INTEGER     NOT NULL DEFAULT 0,
    resolution_mode                  TEXT        NOT NULL,
    rollover_enabled                 BOOLEAN     NOT NULL DEFAULT FALSE,
    guaranteed_min_prize_koinu       BIGINT,
    resolved                         BOOLEAN     NOT NULL DEFAULT FALSE,
    resolved_height                  BIGINT,
    resolved_timestamp               BIGINT,
    resolved_block_hash              TEXT,
    drawn_numbers                    INTEGER[],
    bonus_drawn_numbers              INTEGER[]   NOT NULL DEFAULT ARRAY[]::INTEGER[],
    verified_ticket_count            BIGINT,
    verified_sales_koinu             BIGINT,
    fee_koinu                        BIGINT,
    net_prize_koinu                  BIGINT,
    rollover_occurred                BOOLEAN     NOT NULL DEFAULT FALSE,
    draw_target                      TEXT,
    classic_drawn_numbers            INTEGER[]   NOT NULL DEFAULT ARRAY[]::INTEGER[],
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
    tip_percent                      INTEGER     NOT NULL DEFAULT 0,
    fingerprint                      TEXT,
    classic_numbers                  INTEGER[]   NOT NULL DEFAULT ARRAY[]::INTEGER[],
    luck_marks                       INTEGER[],
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
    bonus_drawn_numbers              INTEGER[]   NOT NULL DEFAULT ARRAY[]::INTEGER[],
    gross_payout_koinu               BIGINT      NOT NULL DEFAULT 0,
    tip_percent                      INTEGER     NOT NULL DEFAULT 0,
    tip_deduction_koinu              BIGINT      NOT NULL DEFAULT 0,
    fingerprint_distance             TEXT,
    classic_matches                  INTEGER     NOT NULL DEFAULT 0,
    classic_payout_koinu             BIGINT      NOT NULL DEFAULT 0,
    PRIMARY KEY (lotto_id, inscription_id),
    CONSTRAINT dogelotto_winners_lotto_id_fkey FOREIGN KEY (lotto_id)
        REFERENCES dogelotto_lotteries (lotto_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS dogelotto_burn_points (
    owner_address                    TEXT        NOT NULL,
    burn_points                      BIGINT      NOT NULL DEFAULT 0,
    last_burn_height                 BIGINT,
    last_burn_timestamp              BIGINT,
    total_tickets_burned             BIGINT      NOT NULL DEFAULT 0,
    PRIMARY KEY (owner_address)
);

CREATE TABLE IF NOT EXISTS dogelotto_burn_events (
    inscription_id                   TEXT        NOT NULL,
    lotto_id                         TEXT        NOT NULL,
    ticket_id                        TEXT        NOT NULL,
    owner_address                    TEXT        NOT NULL,
    burn_height                      BIGINT      NOT NULL,
    burn_timestamp                   BIGINT      NOT NULL,
    burn_tx_id                       TEXT        NOT NULL,
    PRIMARY KEY (inscription_id)
);

CREATE INDEX IF NOT EXISTS dogelotto_lotteries_draw_block_idx
    ON dogelotto_lotteries (draw_block);
CREATE INDEX IF NOT EXISTS dogelotto_lotteries_resolved_height_idx
    ON dogelotto_lotteries (resolved_height);
CREATE INDEX IF NOT EXISTS dogelotto_lotteries_cutoff_block_idx
    ON dogelotto_lotteries (cutoff_block);
CREATE INDEX IF NOT EXISTS dogelotto_tickets_lotto_height_idx
    ON dogelotto_tickets (lotto_id, minted_height);
CREATE INDEX IF NOT EXISTS dogelotto_tickets_fingerprint_idx
    ON dogelotto_tickets (fingerprint) WHERE fingerprint IS NOT NULL;
CREATE INDEX IF NOT EXISTS dogelotto_winners_resolved_height_idx
    ON dogelotto_winners (resolved_height);
CREATE INDEX IF NOT EXISTS dogelotto_winners_fingerprint_distance_idx
    ON dogelotto_winners (fingerprint_distance) WHERE fingerprint_distance IS NOT NULL;
CREATE INDEX IF NOT EXISTS dogelotto_burn_points_points_idx
    ON dogelotto_burn_points (burn_points DESC);
CREATE INDEX IF NOT EXISTS dogelotto_burn_events_owner_idx
    ON dogelotto_burn_events (owner_address);
CREATE INDEX IF NOT EXISTS dogelotto_burn_events_height_idx
    ON dogelotto_burn_events (burn_height);

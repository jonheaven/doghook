-- Track Burn Points for the "Burners" mechanic
-- Users send expired lottery tickets to burn address and earn Burn Points
-- Every 10 points = entry into monthly Burners Bonus Draw

CREATE TABLE IF NOT EXISTS dogelotto_burn_points (
    owner_address                    TEXT        NOT NULL,
    burn_points                      BIGINT      NOT NULL DEFAULT 0,
    last_burn_height                 BIGINT,
    last_burn_timestamp              BIGINT,
    total_tickets_burned             BIGINT      NOT NULL DEFAULT 0,
    PRIMARY KEY (owner_address)
);

-- Track individual burn events for auditability
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

CREATE INDEX IF NOT EXISTS dogelotto_burn_points_points_idx
    ON dogelotto_burn_points (burn_points DESC);
CREATE INDEX IF NOT EXISTS dogelotto_burn_events_owner_idx
    ON dogelotto_burn_events (owner_address);
CREATE INDEX IF NOT EXISTS dogelotto_burn_events_height_idx
    ON dogelotto_burn_events (burn_height);

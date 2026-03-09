-- closest_fingerprint resolution mode for doge-69-420, doge-4-20-flash, doge-max
--
-- Fingerprint = SHA256(sorted seed_numbers as big-endian u16 pairs)
-- Target      = draw_block's block hash interpreted as a u256 big-endian integer
-- Distance    = |fingerprint - target|  (unsigned 256-bit absolute difference)
-- Ranking     = closest distance wins; ties split the tier equally,
--               secondary display sort by inscription_id lexicographic (tx-id order)

-- lotto_tickets: pre-computed fingerprint + derived classic lottery numbers
ALTER TABLE lotto_tickets
    ADD COLUMN IF NOT EXISTS fingerprint     TEXT,
    ADD COLUMN IF NOT EXISTS classic_numbers INTEGER[] NOT NULL DEFAULT ARRAY[]::INTEGER[];

-- lotto_lotteries: block-hash-as-u256 target + 6 classic numbers drawn at resolution
ALTER TABLE lotto_lotteries
    ADD COLUMN IF NOT EXISTS draw_target           TEXT,
    ADD COLUMN IF NOT EXISTS classic_drawn_numbers INTEGER[] NOT NULL DEFAULT ARRAY[]::INTEGER[];

-- lotto_winners: distance stored as hex u256, plus classic-tier match data
ALTER TABLE lotto_winners
    ADD COLUMN IF NOT EXISTS fingerprint_distance TEXT,
    ADD COLUMN IF NOT EXISTS classic_matches      INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS classic_payout_koinu BIGINT  NOT NULL DEFAULT 0;

-- Index for /verify endpoint lookups and leaderboard queries
CREATE INDEX IF NOT EXISTS lotto_tickets_fingerprint_idx
    ON lotto_tickets (fingerprint) WHERE fingerprint IS NOT NULL;

CREATE INDEX IF NOT EXISTS lotto_winners_fingerprint_distance_idx
    ON lotto_winners (fingerprint_distance) WHERE fingerprint_distance IS NOT NULL;

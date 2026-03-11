ALTER TABLE dogelotto_lotteries
    ADD COLUMN IF NOT EXISTS cutoff_block BIGINT;

UPDATE dogelotto_lotteries
SET cutoff_block = GREATEST(draw_block - 10, 1)
WHERE cutoff_block IS NULL;

ALTER TABLE dogelotto_lotteries
    ALTER COLUMN cutoff_block SET NOT NULL;

CREATE INDEX IF NOT EXISTS dogelotto_lotteries_cutoff_block_idx
    ON dogelotto_lotteries (cutoff_block);

ALTER TABLE lotto_lotteries
    ADD COLUMN IF NOT EXISTS cutoff_block BIGINT;

UPDATE lotto_lotteries
SET cutoff_block = GREATEST(draw_block - 10, 1)
WHERE cutoff_block IS NULL;

ALTER TABLE lotto_lotteries
    ALTER COLUMN cutoff_block SET NOT NULL;

CREATE INDEX IF NOT EXISTS lotto_lotteries_cutoff_block_idx
    ON lotto_lotteries (cutoff_block);

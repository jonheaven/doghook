ALTER TABLE dogelotto_lotteries
    ADD COLUMN IF NOT EXISTS template TEXT NOT NULL DEFAULT 'custom',
    ADD COLUMN IF NOT EXISTS main_numbers_pick INTEGER NOT NULL DEFAULT 69,
    ADD COLUMN IF NOT EXISTS main_numbers_max INTEGER NOT NULL DEFAULT 420,
    ADD COLUMN IF NOT EXISTS bonus_numbers_pick INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS bonus_numbers_max INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS bonus_drawn_numbers INTEGER[] NOT NULL DEFAULT ARRAY[]::INTEGER[];

UPDATE dogelotto_lotteries
SET template = CASE resolution_mode
    WHEN 'always_winner' THEN 'always_winner'
    WHEN 'closest_wins' THEN 'closest_wins'
    ELSE 'rollover_jackpot'
END
WHERE template = 'custom';

ALTER TABLE dogelotto_winners
    ADD COLUMN IF NOT EXISTS bonus_drawn_numbers INTEGER[] NOT NULL DEFAULT ARRAY[]::INTEGER[];

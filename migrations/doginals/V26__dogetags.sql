-- Dogetag on-chain graffiti: one row per OP_RETURN text message found in a tx output.
-- Multiple tags per block are allowed; all are independent.
CREATE TABLE IF NOT EXISTS dogetags (
    id               BIGSERIAL PRIMARY KEY,
    txid             TEXT        NOT NULL,
    block_height     NUMERIC     NOT NULL,
    block_timestamp  BIGINT      NOT NULL,
    -- The address that created the tag (derived from vin[0] if available, else NULL).
    sender_address   TEXT,
    -- The decoded UTF-8 message.
    message          TEXT        NOT NULL,
    -- Length of the raw message in bytes.
    message_bytes    INT         NOT NULL,
    -- The raw scriptPubKey hex of the OP_RETURN output.
    raw_script       TEXT        NOT NULL
);

CREATE INDEX IF NOT EXISTS dogetags_block_height_idx ON dogetags (block_height DESC);
CREATE INDEX IF NOT EXISTS dogetags_sender_address_idx ON dogetags (sender_address);
CREATE INDEX IF NOT EXISTS dogetags_txid_idx ON dogetags (txid);

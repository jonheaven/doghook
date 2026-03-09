CREATE TABLE IF NOT EXISTS balance_changes (
    dune_id                 TEXT NOT NULL,
    block_height            NUMERIC NOT NULL,
    address                 TEXT NOT NULL,
    balance                 NUMERIC NOT NULL,
    total_operations        BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (dune_id, block_height, address)
);

CREATE INDEX balance_changes_address_balance_index ON balance_changes (address, block_height, balance DESC);
CREATE INDEX balance_changes_dune_id_balance_index ON balance_changes (dune_id, block_height, balance DESC);

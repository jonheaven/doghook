CREATE TABLE IF NOT EXISTS supply_changes (
    dune_id                 TEXT NOT NULL,
    block_height            NUMERIC NOT NULL,
    minted                  NUMERIC NOT NULL DEFAULT 0,
    total_mints             NUMERIC NOT NULL DEFAULT 0,
    burned                  NUMERIC NOT NULL DEFAULT 0,
    total_burns             NUMERIC NOT NULL DEFAULT 0,
    total_operations        NUMERIC NOT NULL DEFAULT 0,
    PRIMARY KEY (dune_id, block_height)
);

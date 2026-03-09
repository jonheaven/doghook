CREATE INDEX IF NOT EXISTS balance_changes_dune_id_address_block_height_index ON balance_changes (dune_id, address, block_height DESC);

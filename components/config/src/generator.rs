pub fn generate_toml_config(network: &str) -> String {
    let conf = format!(
        r#"[storage]
working_dir = "tmp"

[metrics]
enabled = true
prometheus_port = 9153

[doginals.db]
database = "doginals"
host = "localhost"
port = 5432
username = "postgres"
password = "postgres"

[doginals.meta_protocols.drc20]
enabled = true
lru_cache_size = 10000

[doginals.meta_protocols.drc20.db]
database = "drc20"
host = "localhost"
port = 5432
username = "postgres"
password = "postgres"

[dunes]
lru_cache_size = 10000

[dunes.db]
database = "dunes"
host = "localhost"
port = 5432
username = "postgres"
password = "postgres"

[dogecoin]
network = "{network}"
rpc_url = "http://localhost:22555"
rpc_username = "devnet"
rpc_password = "devnet"
zmq_url = "tcp://0.0.0.0:28332"

[resources]
ulimit = 2048
cpu_core_available = 6
memory_available = 16
dogecoin_rpc_threads = 2
dogecoin_rpc_timeout = 15
indexer_channel_capacity = 10
"#,
        network = network.to_lowercase(),
    );
    conf
}

use clap::{Parser, Subcommand};

/// doghook — Dogecoin Doginals / DNS / Dogemap / Dunes indexer
#[derive(Parser, Debug)]
#[clap(name = "doghook", author, version, about, long_about = None)]
pub enum Protocol {
    /// Doginals index commands
    #[clap(subcommand)]
    Doginals(Command),
    /// Dunes index commands
    #[clap(subcommand)]
    Dunes(Command),
    /// Dogecoin Name System (DNS) query commands
    #[clap(subcommand)]
    Dns(DnsCommand),
    /// Dogemap (block claim) query commands
    #[clap(subcommand)]
    Dogemap(DogemapCommand),
    /// Configuration file commands
    #[clap(subcommand)]
    Config(ConfigCommand),
}

// ---------------------------------------------------------------------------
// DNS subcommands
// ---------------------------------------------------------------------------

#[derive(Subcommand, PartialEq, Clone, Debug)]
pub enum DnsCommand {
    /// Resolve a Dogecoin Name System name (e.g. satoshi.doge)
    #[clap(name = "resolve")]
    Resolve(DnsResolveCommand),
    /// List registered DNS names
    #[clap(name = "list")]
    List(DnsListCommand),
}

#[derive(Parser, PartialEq, Clone, Debug)]
pub struct DnsResolveCommand {
    /// Name to resolve (e.g. satoshi.doge)
    pub name: String,
    #[clap(long = "config-path")]
    pub config_path: String,
    /// Output as JSON
    #[clap(long)]
    pub json: bool,
}

#[derive(Parser, PartialEq, Clone, Debug)]
pub struct DnsListCommand {
    /// Filter by namespace (e.g. doge, shibe, kabosu)
    #[clap(long)]
    pub namespace: Option<String>,
    /// Maximum number of results
    #[clap(long, default_value = "100")]
    pub limit: usize,
    #[clap(long = "config-path")]
    pub config_path: String,
    /// Output as JSON
    #[clap(long)]
    pub json: bool,
}

// ---------------------------------------------------------------------------
// Dogemap subcommands
// ---------------------------------------------------------------------------

#[derive(Subcommand, PartialEq, Clone, Debug)]
pub enum DogemapCommand {
    /// Show claim status for a block number
    #[clap(name = "status")]
    Status(DogemapStatusCommand),
    /// List all claimed block numbers
    #[clap(name = "list")]
    List(DogemapListCommand),
}

#[derive(Parser, PartialEq, Clone, Debug)]
pub struct DogemapStatusCommand {
    /// Block number to query (e.g. 5056597)
    pub block_number: u32,
    #[clap(long = "config-path")]
    pub config_path: String,
    /// Output as JSON
    #[clap(long)]
    pub json: bool,
}

#[derive(Parser, PartialEq, Clone, Debug)]
pub struct DogemapListCommand {
    /// Maximum number of results
    #[clap(long, default_value = "100")]
    pub limit: usize,
    #[clap(long = "config-path")]
    pub config_path: String,
    /// Output as JSON
    #[clap(long)]
    pub json: bool,
}

#[derive(Subcommand, PartialEq, Clone, Debug)]
pub enum Command {
    /// Stream and index Bitcoin blocks
    #[clap(subcommand)]
    Service(ServiceCommand),
    /// Perform maintenance operations on local index
    #[clap(subcommand)]
    Index(IndexCommand),
    /// Database operations
    #[clap(subcommand)]
    Database(DatabaseCommand),
}

#[derive(Subcommand, PartialEq, Clone, Debug)]
pub enum DatabaseCommand {
    /// Migrates database
    #[clap(name = "migrate", bin_name = "migrate")]
    Migrate(MigrateDatabaseCommand),
}

#[derive(Parser, PartialEq, Clone, Debug)]
pub struct MigrateDatabaseCommand {
    #[clap(long = "config-path")]
    pub config_path: String,
}

#[derive(Subcommand, PartialEq, Clone, Debug)]
#[clap(bin_name = "config", aliases = &["config"])]
pub enum ConfigCommand {
    /// Generate new config
    #[clap(name = "new", bin_name = "new", aliases = &["generate"])]
    New(NewConfigCommand),
}

#[derive(Parser, PartialEq, Clone, Debug)]
pub struct NewConfigCommand {
    /// Target Regtest network
    #[clap(
        long = "regtest",
        conflicts_with = "testnet",
        conflicts_with = "mainnet"
    )]
    pub regtest: bool,
    /// Target Testnet network
    #[clap(
        long = "testnet",
        conflicts_with = "regtest",
        conflicts_with = "mainnet"
    )]
    pub testnet: bool,
    /// Target Mainnet network
    #[clap(
        long = "mainnet",
        conflicts_with = "testnet",
        conflicts_with = "regtest"
    )]
    pub mainnet: bool,
}

#[derive(Subcommand, PartialEq, Clone, Debug)]
pub enum ServiceCommand {
    /// Start service
    #[clap(name = "start", bin_name = "start")]
    Start(ServiceStartCommand),
}

#[derive(Parser, PartialEq, Clone, Debug)]
pub struct ServiceStartCommand {
    #[clap(long = "config-path")]
    pub config_path: String,
}

#[derive(Subcommand, PartialEq, Clone, Debug)]
pub enum IndexCommand {
    /// Sync index to latest bitcoin block
    #[clap(name = "sync", bin_name = "sync")]
    Sync(SyncIndexCommand),
    /// Rollback index blocks
    #[clap(name = "rollback", bin_name = "drop")]
    Rollback(RollbackIndexCommand),
}

#[derive(Parser, PartialEq, Clone, Debug)]
pub struct SyncIndexCommand {
    #[clap(long = "config-path")]
    pub config_path: String,
}

#[derive(Parser, PartialEq, Clone, Debug)]
pub struct RollbackIndexCommand {
    /// Number of blocks to rollback from index tip
    pub blocks: u32,
    #[clap(long = "config-path")]
    pub config_path: String,
}

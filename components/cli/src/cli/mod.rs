use std::{
    path::PathBuf,
    process,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::sleep,
    time::Duration,
};

use dogecoin::{try_error, try_info, types::BlockIdentifier, utils::Context};
use clap::Parser;
use commands::{
    Command, ConfigCommand, DatabaseCommand, DnsCommand, DogemapCommand, IndexCommand,
    LottoCommand, Protocol, ServiceCommand,
};
use config::{generator::generate_toml_config, Config};
use hiro_system_kit;

mod commands;

pub fn main() {
    let logger = hiro_system_kit::log::setup_logger();
    let _guard = hiro_system_kit::log::setup_global_logger(logger.clone());
    let ctx = Context {
        logger: Some(logger),
        tracer: false,
    };

    let opts: Protocol = match Protocol::try_parse() {
        Ok(opts) => opts,
        Err(e) => {
            println!("{}", e);
            process::exit(1);
        }
    };

    if let Err(e) = hiro_system_kit::nestable_block_on(handle_command(opts, &ctx)) {
        try_error!(&ctx, "{e}");
        std::thread::sleep(std::time::Duration::from_millis(500));
        process::exit(1);
    }
}

fn check_maintenance_mode(ctx: &Context) {
    let maintenance_enabled = std::env::var("DOGHOOK_MAINTENANCE").unwrap_or("0".into());
    if maintenance_enabled.eq("1") {
        try_info!(
            ctx,
            "Entering maintenance mode. Unset DOGHOOK_MAINTENANCE and reboot to resume operations"
        );
        sleep(Duration::from_secs(u64::MAX))
    }
}

fn confirm_rollback(
    current_chain_tip: &BlockIdentifier,
    blocks_to_rollback: u32,
) -> Result<(), String> {
    println!("Index chain tip is at #{current_chain_tip}");
    println!(
        "{} blocks will be dropped. New index chain tip will be at #{}. Confirm? [Y/n]",
        blocks_to_rollback,
        current_chain_tip.index - blocks_to_rollback as u64
    );
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer).unwrap();
    if buffer.starts_with('n') {
        return Err("Deletion aborted".to_string());
    }
    Ok(())
}

async fn handle_command(opts: Protocol, ctx: &Context) -> Result<(), String> {
    // Set up the interrupt signal handler.
    let abort_signal = Arc::new(AtomicBool::new(false));
    let abort_signal_clone = abort_signal.clone();
    let ctx_moved = ctx.clone();
    ctrlc::set_handler(move || {
        try_info!(
            ctx_moved,
            "dogecoin-indexer received interrupt signal, shutting down..."
        );
        abort_signal_clone.store(true, Ordering::SeqCst);
    })
    .map_err(|e| format!("dogecoin-indexer failed to set interrupt signal handler: {e}"))?;

    match opts {
        Protocol::Doginals(subcmd) => match subcmd {
            Command::Service(subcmd) => match subcmd {
                ServiceCommand::Start(cmd) => {
                    check_maintenance_mode(ctx);
                    let config = Config::from_file_path(&cmd.config_path)?;
                    config.assert_doginals_config()?;
                    doginals_indexer::start_doginals_indexer(true, &abort_signal, &config, ctx).await?
                }
            },
            Command::Index(index_command) => match index_command {
                IndexCommand::Sync(cmd) => {
                    let config = Config::from_file_path(&cmd.config_path)?;
                    config.assert_doginals_config()?;
                    doginals_indexer::start_doginals_indexer(false, &abort_signal, &config, ctx).await?
                }
                IndexCommand::Rollback(cmd) => {
                    let config = Config::from_file_path(&cmd.config_path)?;
                    config.assert_doginals_config()?;
                    let chain_tip = doginals_indexer::get_chain_tip(&config).await?;
                    confirm_rollback(&chain_tip, cmd.blocks)?;
                    doginals_indexer::rollback_block_range(
                        chain_tip.index - cmd.blocks as u64,
                        chain_tip.index,
                        &config,
                        ctx,
                    )
                    .await?;
                    println!("{} blocks dropped", cmd.blocks);
                }
            },
            Command::Database(database_command) => match database_command {
                DatabaseCommand::Migrate(cmd) => {
                    let config = Config::from_file_path(&cmd.config_path)?;
                    config.assert_doginals_config()?;
                    doginals_indexer::db::migrate_dbs(&config, ctx).await?;
                }
            },
        },
        Protocol::Dunes(subcmd) => match subcmd {
            Command::Service(service_command) => match service_command {
                ServiceCommand::Start(cmd) => {
                    check_maintenance_mode(ctx);
                    let config = Config::from_file_path(&cmd.config_path)?;
                    config.assert_dunes_config()?;
                    dunes::start_dunes_indexer(true, &abort_signal, &config, ctx).await?
                }
            },
            Command::Index(index_command) => match index_command {
                IndexCommand::Sync(cmd) => {
                    let config = Config::from_file_path(&cmd.config_path)?;
                    config.assert_dunes_config()?;
                    dunes::start_dunes_indexer(false, &abort_signal, &config, ctx).await?
                }
                IndexCommand::Rollback(cmd) => {
                    let config = Config::from_file_path(&cmd.config_path)?;
                    config.assert_dunes_config()?;
                    let chain_tip = dunes::get_chain_tip(&config).await?;
                    confirm_rollback(&chain_tip, cmd.blocks)?;
                    dunes::rollback_block_range(
                        chain_tip.index - cmd.blocks as u64,
                        chain_tip.index,
                        &config,
                        ctx,
                    )
                    .await?;
                    println!("{} blocks dropped", cmd.blocks);
                }
            },
            Command::Database(database_command) => match database_command {
                DatabaseCommand::Migrate(cmd) => {
                    let config = Config::from_file_path(&cmd.config_path)?;
                    config.assert_dunes_config()?;
                    dunes::db::run_migrations(&config, ctx).await;
                }
            },
        },
        Protocol::Dns(subcmd) => match subcmd {
            DnsCommand::Resolve(cmd) => {
                let config = Config::from_file_path(&cmd.config_path)?;
                config.assert_doginals_config()?;
                match doginals_indexer::dns_resolve(&cmd.name, &config).await? {
                    Some(row) => {
                        if cmd.json {
                            println!(
                                "{}",
                                serde_json::json!({
                                    "name": row.name,
                                    "inscription_id": row.inscription_id,
                                    "block_height": row.block_height,
                                    "block_timestamp": row.block_timestamp,
                                })
                            );
                        } else {
                            println!("Name:           {}", row.name);
                            println!("Inscription ID: {}", row.inscription_id);
                            println!("Block Height:   {}", row.block_height);
                            println!("Timestamp:      {}", row.block_timestamp);
                        }
                    }
                    None => {
                        if cmd.json {
                            println!("null");
                        } else {
                            println!("Name not found: {}", cmd.name);
                        }
                        process::exit(1);
                    }
                }
            }
            DnsCommand::List(cmd) => {
                let config = Config::from_file_path(&cmd.config_path)?;
                config.assert_doginals_config()?;
                let (rows, total) =
                    doginals_indexer::dns_list(cmd.namespace.as_deref(), cmd.limit, 0, &config)
                        .await?;
                if cmd.json {
                    let json_rows: Vec<_> = rows
                        .iter()
                        .map(|r| {
                            serde_json::json!({
                                "name": r.name,
                                "inscription_id": r.inscription_id,
                                "block_height": r.block_height,
                                "block_timestamp": r.block_timestamp,
                            })
                        })
                        .collect();
                    println!(
                        "{}",
                        serde_json::json!({ "total": total, "names": json_rows })
                    );
                } else {
                    println!("DNS Names (Total: {total})");
                    println!("{:<40} {:<70} {}", "Name", "Inscription ID", "Height");
                    println!("{}", "-".repeat(115));
                    for row in &rows {
                        println!("{:<40} {:<70} {}", row.name, row.inscription_id, row.block_height);
                    }
                }
            }
        },
        Protocol::Dogemap(subcmd) => match subcmd {
            DogemapCommand::Status(cmd) => {
                let config = Config::from_file_path(&cmd.config_path)?;
                config.assert_doginals_config()?;
                match doginals_indexer::dogemap_status(cmd.block_number, &config).await? {
                    Some(row) => {
                        if cmd.json {
                            println!(
                                "{}",
                                serde_json::json!({
                                    "block_number": row.block_number,
                                    "inscription_id": row.inscription_id,
                                    "claim_height": row.claim_height,
                                    "claim_timestamp": row.claim_timestamp,
                                })
                            );
                        } else {
                            println!("Block Number:   {}", row.block_number);
                            println!("Inscription ID: {}", row.inscription_id);
                            println!("Claim Height:   {}", row.claim_height);
                            println!("Timestamp:      {}", row.claim_timestamp);
                        }
                    }
                    None => {
                        if cmd.json {
                            println!("null");
                        } else {
                            println!("Block {} is unclaimed", cmd.block_number);
                        }
                        process::exit(1);
                    }
                }
            }
            DogemapCommand::List(cmd) => {
                let config = Config::from_file_path(&cmd.config_path)?;
                config.assert_doginals_config()?;
                let (rows, total) =
                    doginals_indexer::dogemap_list(cmd.limit, 0, &config).await?;
                if cmd.json {
                    let json_rows: Vec<_> = rows
                        .iter()
                        .map(|r| {
                            serde_json::json!({
                                "block_number": r.block_number,
                                "inscription_id": r.inscription_id,
                                "claim_height": r.claim_height,
                                "claim_timestamp": r.claim_timestamp,
                            })
                        })
                        .collect();
                    println!(
                        "{}",
                        serde_json::json!({ "total": total, "claims": json_rows })
                    );
                } else {
                    println!("Dogemap Claims (Total: {total})");
                    println!("{:<12} {:<70} {}", "Block", "Inscription ID", "Claim Height");
                    println!("{}", "-".repeat(95));
                    for row in &rows {
                        println!("{:<12} {:<70} {}", row.block_number, row.inscription_id, row.claim_height);
                    }
                }
            }
        },
        Protocol::Lotto(subcmd) => match subcmd {
            LottoCommand::Deploy(cmd) => {
                let resolution_mode = normalize_resolution_mode(&cmd.resolution_mode)?;
                if !(0..=10).contains(&cmd.fee_percent) {
                    return Err("fee_percent must be between 0 and 10".into());
                }
                if matches!(cmd.lotto_id.as_str(), "doge-69-420" | "doge-max") && cmd.fee_percent != 0 {
                    return Err(format!("{} must be deployed with fee_percent = 0", cmd.lotto_id));
                }
                let payload = serde_json::json!({
                    "p": "doge-lotto",
                    "op": "deploy",
                    "lotto_id": cmd.lotto_id,
                    "draw_block": cmd.draw_block,
                    "ticket_price_koinu": cmd.ticket_price_koinu,
                    "prize_pool_address": cmd.prize_pool_address,
                    "fee_percent": cmd.fee_percent,
                    "resolution_mode": resolution_mode,
                    "rollover_enabled": cmd.rollover_enabled,
                    "guaranteed_min_prize_koinu": cmd.guaranteed_min_prize_koinu,
                });
                let payload = compact_json_without_nulls(payload)?;
                if cmd.json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "content_type": "text/plain",
                            "payload": payload,
                        })
                    );
                } else {
                    println!("{}", payload);
                }
            }
            LottoCommand::Mint(cmd) => {
                let config = Config::from_file_path(&cmd.config_path)?;
                config.assert_doginals_config()?;
                let Some(status) = doginals_indexer::lotto_status(&cmd.lotto_id, &config).await? else {
                    return Err(format!("Lotto not found: {}", cmd.lotto_id));
                };

                let seed_numbers = if let Some(seed_numbers) = &cmd.seed_numbers {
                    parse_seed_numbers(seed_numbers)?
                } else {
                    doginals_indexer::core::meta_protocols::lotto::quickpick()
                };
                let ticket_id = cmd
                    .ticket_id
                    .clone()
                    .unwrap_or_else(generate_ticket_id);

                let payload = serde_json::json!({
                    "p": "doge-lotto",
                    "op": "mint",
                    "lotto_id": cmd.lotto_id,
                    "ticket_id": ticket_id,
                    "seed_numbers": seed_numbers,
                });
                let payload = compact_json_without_nulls(payload)?;

                if cmd.json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "content_type": "text/plain",
                            "payload": payload,
                            "payment": {
                                "address": status.summary.prize_pool_address,
                                "amount_koinu": status.summary.ticket_price_koinu,
                            }
                        })
                    );
                } else {
                    println!("{}", payload);
                    eprintln!(
                        "pay exact {} koinu to {} in the same transaction",
                        status.summary.ticket_price_koinu,
                        status.summary.prize_pool_address,
                    );
                }
            }
            LottoCommand::Status(cmd) => {
                let config = Config::from_file_path(&cmd.config_path)?;
                config.assert_doginals_config()?;
                match doginals_indexer::lotto_status(&cmd.lotto_id, &config).await? {
                    Some(row) => {
                        if cmd.json {
                            let winners: Vec<_> = row
                                .winners
                                .iter()
                                .map(|winner| {
                                    serde_json::json!({
                                        "lotto_id": winner.lotto_id,
                                        "inscription_id": winner.inscription_id,
                                        "ticket_id": winner.ticket_id,
                                        "resolved_height": winner.resolved_height,
                                        "rank": winner.rank,
                                        "score": winner.score,
                                        "payout_bps": winner.payout_bps,
                                        "payout_koinu": winner.payout_koinu,
                                        "seed_numbers": winner.seed_numbers,
                                        "drawn_numbers": winner.drawn_numbers,
                                    })
                                })
                                .collect();
                            println!(
                                "{}",
                                serde_json::json!({
                                    "lotto_id": row.summary.lotto_id,
                                    "inscription_id": row.summary.inscription_id,
                                    "deploy_height": row.summary.deploy_height,
                                    "deploy_timestamp": row.summary.deploy_timestamp,
                                    "draw_block": row.summary.draw_block,
                                    "ticket_price_koinu": row.summary.ticket_price_koinu,
                                    "prize_pool_address": row.summary.prize_pool_address,
                                    "fee_percent": row.summary.fee_percent,
                                    "resolution_mode": row.summary.resolution_mode,
                                    "rollover_enabled": row.summary.rollover_enabled,
                                    "guaranteed_min_prize_koinu": row.summary.guaranteed_min_prize_koinu,
                                    "resolved": row.summary.resolved,
                                    "resolved_height": row.summary.resolved_height,
                                    "verified_ticket_count": row.summary.verified_ticket_count,
                                    "verified_sales_koinu": row.summary.verified_sales_koinu,
                                    "net_prize_koinu": row.summary.net_prize_koinu,
                                    "rollover_occurred": row.summary.rollover_occurred,
                                    "current_ticket_count": row.summary.current_ticket_count,
                                    "winners": winners,
                                })
                            );
                        } else {
                            println!("Lotto ID:               {}", row.summary.lotto_id);
                            println!("Inscription ID:         {}", row.summary.inscription_id);
                            println!("Deploy Height:          {}", row.summary.deploy_height);
                            println!("Draw Block:             {}", row.summary.draw_block);
                            println!("Ticket Price (koinu):   {}", row.summary.ticket_price_koinu);
                            println!("Prize Pool Address:     {}", row.summary.prize_pool_address);
                            println!("Fee Percent:            {}", row.summary.fee_percent);
                            println!("Resolution Mode:        {}", row.summary.resolution_mode);
                            println!("Rollover Enabled:       {}", row.summary.rollover_enabled);
                            println!("Guaranteed Min Prize:   {}", row.summary.guaranteed_min_prize_koinu.map(|v| v.to_string()).unwrap_or_else(|| "-".into()));
                            println!("Current Ticket Count:   {}", row.summary.current_ticket_count);
                            println!("Resolved:               {}", row.summary.resolved);
                            println!("Resolved Height:        {}", row.summary.resolved_height.map(|v| v.to_string()).unwrap_or_else(|| "-".into()));
                            println!("Verified Ticket Count:  {}", row.summary.verified_ticket_count.map(|v| v.to_string()).unwrap_or_else(|| "-".into()));
                            println!("Verified Sales (koinu): {}", row.summary.verified_sales_koinu.map(|v| v.to_string()).unwrap_or_else(|| "-".into()));
                            println!("Net Prize (koinu):      {}", row.summary.net_prize_koinu.map(|v| v.to_string()).unwrap_or_else(|| "-".into()));
                            println!("Rollover Occurred:      {}", row.summary.rollover_occurred);
                            if row.winners.is_empty() {
                                println!("Winners:                none");
                            } else {
                                println!("Winners:");
                                for winner in &row.winners {
                                    println!(
                                        "  rank {} ticket {} payout {} koinu score {} inscription {}",
                                        winner.rank,
                                        winner.ticket_id,
                                        winner.payout_koinu,
                                        winner.score,
                                        winner.inscription_id
                                    );
                                }
                            }
                        }
                    }
                    None => {
                        if cmd.json {
                            println!("null");
                        } else {
                            println!("Lotto not found: {}", cmd.lotto_id);
                        }
                        process::exit(1);
                    }
                }
            }
            LottoCommand::List(cmd) => {
                let config = Config::from_file_path(&cmd.config_path)?;
                config.assert_doginals_config()?;
                let (rows, total) = doginals_indexer::lotto_list(cmd.limit, 0, &config).await?;
                if cmd.json {
                    let json_rows: Vec<_> = rows
                        .iter()
                        .map(|row| {
                            serde_json::json!({
                                "lotto_id": row.lotto_id,
                                "inscription_id": row.inscription_id,
                                "deploy_height": row.deploy_height,
                                "draw_block": row.draw_block,
                                "ticket_price_koinu": row.ticket_price_koinu,
                                "prize_pool_address": row.prize_pool_address,
                                "fee_percent": row.fee_percent,
                                "resolution_mode": row.resolution_mode,
                                "resolved": row.resolved,
                                "resolved_height": row.resolved_height,
                                "current_ticket_count": row.current_ticket_count,
                                "verified_ticket_count": row.verified_ticket_count,
                                "net_prize_koinu": row.net_prize_koinu,
                            })
                        })
                        .collect();
                    println!(
                        "{}",
                        serde_json::json!({ "total": total, "lottos": json_rows })
                    );
                } else {
                    println!("doge-lotto Deployments (Total: {total})");
                    println!(
                        "{:<24} {:<12} {:<10} {:<8} {:<8} {}",
                        "Lotto ID", "Draw Block", "Tickets", "Fee %", "Resolved", "Mode"
                    );
                    println!("{}", "-".repeat(78));
                    for row in &rows {
                        println!(
                            "{:<24} {:<12} {:<10} {:<8} {:<8} {}",
                            row.lotto_id,
                            row.draw_block,
                            row.current_ticket_count,
                            row.fee_percent,
                            row.resolved,
                            row.resolution_mode,
                        );
                    }
                }
            }
        },
        Protocol::Config(subcmd) => match subcmd {
            ConfigCommand::New(cmd) => {
                use std::{fs::File, io::Write};
                let network = match (cmd.mainnet, cmd.testnet, cmd.regtest) {
                    (true, false, false) => "mainnet",
                    (false, true, false) => "testnet",
                    (false, false, true) => "regtest",
                    _ => return Err("Invalid network".into()),
                };
                let config_content = generate_toml_config(network);
                let mut file_path = PathBuf::new();
                file_path.push("Indexer.toml");
                let mut file = File::create(&file_path)
                    .map_err(|e| format!("unable to open file {}\n{}", file_path.display(), e))?;
                file.write_all(config_content.as_bytes())
                    .map_err(|e| format!("unable to write file {}\n{}", file_path.display(), e))?;
                println!("Created file Indexer.toml");
            }
        },
    }
    Ok(())
}

fn normalize_resolution_mode(value: &str) -> Result<&'static str, String> {
    match value {
        "always_winner" => Ok("always_winner"),
        "closest_wins" => Ok("closest_wins"),
        "exact_only_with_rollover" => Ok("exact_only_with_rollover"),
        _ => Err(format!(
            "invalid resolution mode: {} (expected always_winner, closest_wins, or exact_only_with_rollover)",
            value
        )),
    }
}

fn compact_json_without_nulls(mut value: serde_json::Value) -> Result<String, String> {
    prune_nulls(&mut value);
    serde_json::to_string(&value).map_err(|e| format!("unable to serialize lotto payload: {e}"))
}

fn prune_nulls(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            map.retain(|_, inner| {
                prune_nulls(inner);
                !inner.is_null()
            });
        }
        serde_json::Value::Array(values) => {
            for inner in values {
                prune_nulls(inner);
            }
        }
        _ => {}
    }
}

fn parse_seed_numbers(value: &str) -> Result<Vec<u16>, String> {
    let parsed: Vec<u16> = value
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| {
            part.parse::<u16>()
                .map_err(|e| format!("invalid seed number '{}': {}", part, e))
        })
        .collect::<Result<_, _>>()?;

    let payload = serde_json::json!({
        "p": "doge-lotto",
        "op": "mint",
        "lotto_id": "validation-only",
        "ticket_id": "validation-only",
        "seed_numbers": parsed,
    });
    doginals_indexer::core::meta_protocols::lotto::try_parse_lotto_mint(
        compact_json_without_nulls(payload)?.as_bytes(),
    )
    .map(|mint| mint.seed_numbers)
    .ok_or("seed_numbers must contain exactly 69 unique values in [1, 420]".into())
}

fn generate_ticket_id() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("ticket-{}-{}", now.as_secs(), now.subsec_nanos())
}

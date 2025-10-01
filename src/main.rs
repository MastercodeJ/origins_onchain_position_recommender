use anyhow::Result;
use clap::Parser;
use tracing::{info, Level};
use tracing_subscriber;

mod config;
mod position;
mod recommender;
mod utils;
mod ai_predictor;
mod uniswap;

use config::Config;
use recommender::PositionRecommender;
use uniswap::UniswapClient;

#[derive(Parser)]
#[command(name = "origins-onchain-position-recommender")]
#[command(about = "Onchain position recommender for Origins protocol")]
struct Cli {
    /// Configuration file path
    #[arg(short, long, default_value = "config.toml")]
    config: String,
    
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// List top N Uniswap pools and exit
    #[arg(long, default_value_t = 0)]
    list_top_pools: usize,

    /// Fetch a Uniswap V3 position by tokenId and exit
    #[arg(long)]
    position_id: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    let level = if cli.verbose { Level::DEBUG } else { Level::INFO };
    tracing_subscriber::fmt()
        .with_max_level(level)
        .init();
    
    info!("Starting Origins Onchain Position Recommender");
    
    // Load configuration
    let config = Config::load(&cli.config)?;
    info!("Configuration loaded from {}", cli.config);

    // If a position id is requested, fetch on-chain and exit
    if let Some(token_id) = cli.position_id.as_deref() {
        let client = UniswapClient::from_config(&config);
        let rpc = config.rpc_url.as_str();
        let pos = client.get_onchain_position(rpc, token_id).await?;
        println!(
            "[UNISWAP ONCHAIN] tokenId={} {}({})-{}({}) fee={} tickRange=[{},{}] priceRange[{} per {}]=[{}, {}] midPrice={} liquidity={} owed0={} owed1={}",
            pos.token_id,
            pos.token0_symbol,
            pos.token0,
            pos.token1_symbol,
            pos.token1,
            pos.fee,
            pos.tick_lower,
            pos.tick_upper,
            pos.token1_symbol,
            pos.token0_symbol,
            pos.price_lower_quote_per_base,
            pos.price_upper_quote_per_base,
            pos.mid_price_quote_per_base,
            pos.liquidity,
            pos.tokens_owed0,
            pos.tokens_owed1
        );
        return Ok(());
    }

    // Optional: List top Uniswap pools and exit
    if cli.list_top_pools > 0 {
        let client = UniswapClient::from_config(&config);
        let pools = client.top_pools(cli.list_top_pools).await?;
        info!("Fetched {} pools", pools.len());
        for (i, p) in pools.iter().enumerate() {
            info!(
                "{}. {} | {}-{} | TVL(USD): {} | Volume(USD): {}",
                i + 1,
                p.id,
                p.token0.symbol,
                p.token1.symbol,
                p.total_value_locked_usd,
                p.volume_usd
            );
        }
        return Ok(());
    }

    // Background task: quote configured Uniswap pools periodically
    if let Some(uniswap_cfg) = &config.uniswap {
        let client = UniswapClient::from_config(&config);
        let pool_ids = uniswap_cfg.pool_ids.clone();
        let position_ids = uniswap_cfg.position_ids.clone();
        let interval = uniswap_cfg.quote_interval_secs;
        if !pool_ids.is_empty() || !position_ids.is_empty() {
            tokio::spawn(async move {
                loop {
                    // Quote pools by id
                    for pid in &pool_ids {
                        match client.get_pool_by_id(pid).await {
                            Ok(Some(pool)) => {
                                println!(
                                    "[UNISWAP] Pool {} | {}-{} | TVL(USD): {} | Volume(USD): {}",
                                    pool.id,
                                    pool.token0.symbol,
                                    pool.token1.symbol,
                                    pool.total_value_locked_usd,
                                    pool.volume_usd
                                );
                            }
                            Ok(None) => println!("[UNISWAP] Pool {} not found", pid),
                            Err(e) => println!("[UNISWAP] Error fetching pool {}: {}", pid, e),
                        }
                    }

                    // Quote pools by position id (resolve to pool)
                    for pos_id in &position_ids {
                        match client.get_pool_by_position_id(pos_id).await {
                            Ok(Some(pool)) => {
                                println!(
                                    "[UNISWAP] Position {} -> Pool {} | {}-{} | TVL(USD): {} | Volume(USD): {}",
                                    pos_id,
                                    pool.id,
                                    pool.token0.symbol,
                                    pool.token1.symbol,
                                    pool.total_value_locked_usd,
                                    pool.volume_usd
                                );
                            }
                            Ok(None) => println!("[UNISWAP] Position {} not found", pos_id),
                            Err(e) => println!("[UNISWAP] Error fetching position {}: {}", pos_id, e),
                        }
                    }

                    tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
                }
            });
        }
    }
    
    // Initialize position recommender
    let mut recommender = PositionRecommender::new(config).await?;
    
    // Run the recommender
    recommender.run().await?;
    
    info!("Position recommender completed successfully");
    Ok(())
}

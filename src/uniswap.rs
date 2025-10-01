use anyhow::{Context, Result};
use ethabi::{ParamType, Token as AbiToken};
use ethereum_types::{Address, U256};
use reqwest::{header::HeaderMap, Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

use crate::config::Config;

#[derive(Clone)]
pub struct UniswapClient {
    http: Client,
    graph_endpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pool {
    pub id: String,
    pub token0: Token,
    pub token1: Token,
    pub fee_tier: String,
    pub liquidity: String,
    pub volume_usd: String,
    pub total_value_locked_usd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub decimals: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GraphRequest {
    query: String,
    variables: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GraphErrorItem {
    message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GraphResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphErrorItem>>, // present when the graph returns an error
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PoolsData {
    pools: Vec<Pool>,
}

impl UniswapClient {
    fn alias_symbol(&self, token_address_hex: &str, raw_symbol: &str) -> String {
        let addr = token_address_hex.to_lowercase();
        let sym = raw_symbol.to_uppercase();
        // Address-specific mappings (Arbitrum canonical tokens)
        let mapped_by_addr = match addr.as_str() {
            // WETH -> ETH
            "0x82af49447d8a07e3bd95bd0d56f35241523fbab1" => Some("ETH"),
            // USDC (native)
            "0xaf88d065e77c8cc2239327c5edb3a432268e5831" => Some("USDC"),
            // USDC.e (bridged)
            "0xff970a61a04b1ca14834a43f5de4533ebddb5cc8" => Some("USDC"),
            // USDT
            "0xfd086bc7cd5c481dcc9c85ebe478a1c0b69fcbb9" => Some("USDT"),
            // DAI
            "0xda10009cbd5d07dd0cecc66161fc93d7c9000da1" => Some("DAI"),
            // WBTC -> BTC
            "0x2f2a2543b76a4166549f7aab2e75bef0aefc5b0f" => Some("BTC"),
            // ARB
            "0x912ce59144191c1204e64559fe8253a0e49e6548" => Some("ARB"),
            _ => None,
        };
        if let Some(s) = mapped_by_addr { return s.to_string(); }

        // Generic symbol aliases
        match sym.as_str() {
            "WETH" | "WETH9" => "ETH".to_string(),
            "WBTC" => "BTC".to_string(),
            "USDC.E" => "USDC".to_string(),
            other => other.to_string(),
        }
    }
    pub fn from_config(config: &Config) -> Self {
        // Prefer config.api.thegraph_api_url if present
        let endpoint = config
            .api
            .as_ref()
            .and_then(|a| a.thegraph_api_url.clone())
            .unwrap_or_else(|| "https://api.thegraph.com/subgraphs/name/uniswap/uniswap-v3".to_string());

        // Optional Graph API key support (Graph Gateway requires Authorization header)
        let mut headers = HeaderMap::new();
        if let Some(api_cfg) = &config.api {
            if let Some(key) = &api_cfg.thegraph_api_key {
                if !key.is_empty() {
                    if let Ok(value) = format!("Bearer {}", key).parse() {
                        headers.insert("Authorization", value);
                    }
                    // Some deployments may expect 'apikey' header instead
                    if let Ok(value) = key.parse() {
                        headers.insert("apikey", value);
                    }
                }
            }
        }

        let http = Client::builder()
            .default_headers(headers)
            .user_agent("origins-uniswap-client/0.1")
            .timeout(Duration::from_secs(15))
            .build()
            .expect("failed to build reqwest client");

        Self {
            http,
            graph_endpoint: endpoint,
        }
    }

    async fn post_with_retry<T: for<'de> Deserialize<'de>>(&self, req: &GraphRequest) -> Result<T> {
        let mut attempt: u32 = 0;
        let max_attempts: u32 = 3;
        let mut last_status: Option<StatusCode> = None;
        loop {
            info!(target: "uniswap.fetch", endpoint = %self.graph_endpoint, attempt = attempt + 1, "sending request to The Graph");
            let resp = self.http
                .post(&self.graph_endpoint)
                .json(req)
                .send()
                .await
                .with_context(|| "sending request to The Graph")?;

            let status = resp.status();
            if status.is_success() {
                let text = resp.text().await.with_context(|| "reading graph response text")?;
                let envelope: GraphResponse<T> = serde_json::from_str(&text)
                    .with_context(|| format!("decoding graph response JSON: {}", text))?;

                if let Some(errors) = envelope.errors {
                    let msg = errors.first().map(|e| e.message.clone()).unwrap_or_else(|| "unknown graph error".to_string());
                    info!(target: "uniswap.fetch", endpoint = %self.graph_endpoint, %msg, "graph returned errors");
                    return Err(anyhow::anyhow!("graph error: {}", msg));
                }

                if let Some(data) = envelope.data {
                    info!(target: "uniswap.fetch", endpoint = %self.graph_endpoint, attempt = attempt + 1, "graph request succeeded");
                    return Ok(data);
                } else {
                    return Err(anyhow::anyhow!("graph response missing data field"));
                }
            }

            last_status = Some(status);
            attempt += 1;
            if attempt >= max_attempts || status == StatusCode::BAD_REQUEST {
                break;
            }
            // Exponential backoff: 300ms, 900ms, 2700ms
            let backoff_ms = 300u64 * 3u64.pow((attempt - 1) as u32);
            info!(target: "uniswap.fetch", endpoint = %self.graph_endpoint, attempt = attempt + 1, status = %status, backoff_ms, "graph request failed, backing off and retrying");
            sleep(Duration::from_millis(backoff_ms)).await;
        }
        Err(anyhow::anyhow!("Uniswap graph request failed, status={:?}", last_status))
    }

    pub async fn top_pools(&self, first: usize) -> Result<Vec<Pool>> {
        info!(target: "uniswap.fetch", endpoint = %self.graph_endpoint, first, "fetching top pools");
        let query = r#"
        query TopPools($first: Int!) {
          pools(first: $first, orderBy: totalValueLockedUSD, orderDirection: desc) {
            id
            feeTier
            liquidity
            volumeUSD
            totalValueLockedUSD
            token0 { id symbol name decimals }
            token1 { id symbol name decimals }
          }
        }
        "#;

        let req = GraphRequest {
            query: query.to_string(),
            variables: serde_json::json!({ "first": first as i64 }),
        };

        let body: PoolsData = self.post_with_retry(&req).await?;
        info!(target: "uniswap.fetch", endpoint = %self.graph_endpoint, count = body.pools.len(), "fetched top pools");
        Ok(body.pools)
    }

    pub async fn top_pools_paginated(&self, total: usize, page_size: usize) -> Result<Vec<Pool>> {
        info!(target: "uniswap.fetch", endpoint = %self.graph_endpoint, total, page_size, "fetching top pools paginated");
        let mut all: Vec<Pool> = Vec::new();
        let mut skip: usize = 0;
        let page = page_size.max(1);
        while all.len() < total {
            let batch = self.top_pools_with_skip(page, skip).await?;
            if batch.is_empty() {
                break;
            }
            all.extend(batch);
            skip += page;
        }
        all.truncate(total);
        info!(target: "uniswap.fetch", endpoint = %self.graph_endpoint, count = all.len(), "completed paginated fetch of top pools");
        Ok(all)
    }

    async fn top_pools_with_skip(&self, first: usize, skip: usize) -> Result<Vec<Pool>> {
        info!(target: "uniswap.fetch", endpoint = %self.graph_endpoint, first, skip, "fetching top pools page");
        let query = r#"
        query TopPools($first: Int!, $skip: Int!) {
          pools(first: $first, skip: $skip, orderBy: totalValueLockedUSD, orderDirection: desc) {
            id
            feeTier
            liquidity
            volumeUSD
            totalValueLockedUSD
            token0 { id symbol name decimals }
            token1 { id symbol name decimals }
          }
        }
        "#;

        let req = GraphRequest {
            query: query.to_string(),
            variables: serde_json::json!({ "first": first as i64, "skip": skip as i64 }),
        };

        let body: PoolsData = self.post_with_retry(&req).await?;
        info!(target: "uniswap.fetch", endpoint = %self.graph_endpoint, first, skip, count = body.pools.len(), "fetched top pools page");
        Ok(body.pools)
    }

    pub async fn get_pool_by_id(&self, pool_id: &str) -> Result<Option<Pool>> {
        info!(target: "uniswap.fetch", endpoint = %self.graph_endpoint, pool_id = pool_id, "fetching pool by id");
        let query = r#"
        query PoolById($id: ID!) {
          pool(id: $id) {
            id
            feeTier
            liquidity
            volumeUSD
            totalValueLockedUSD
            token0 { id symbol name decimals }
            token1 { id symbol name decimals }
          }
        }
        "#;

        let req = GraphRequest {
            query: query.to_string(),
            variables: serde_json::json!({ "id": pool_id }),
        };

        #[derive(Deserialize)]
        struct PoolData { pool: Option<Pool> }
        let body: PoolData = self.post_with_retry(&req).await?;
        info!(target: "uniswap.fetch", endpoint = %self.graph_endpoint, pool_id = pool_id, found = body.pool.is_some(), "fetched pool by id");
        Ok(body.pool)
    }

    /// Resolve a Uniswap v3 position NFT id to its pool id, then fetch the pool
    pub async fn get_pool_by_position_id(&self, position_id: &str) -> Result<Option<Pool>> {
        info!(target: "uniswap.fetch", endpoint = %self.graph_endpoint, position_id = position_id, "resolving pool by position id");
        let query = r#"
        query PositionById($id: ID!) {
          position(id: $id) {
            id
            pool { id }
          }
        }
        "#;

        #[derive(Deserialize)]
        struct PositionResp { position: Option<PositionMin> }
        #[derive(Deserialize)]
        struct PositionMin { pool: PoolRef }
        #[derive(Deserialize)]
        struct PoolRef { id: String }

        let req = GraphRequest {
            query: query.to_string(),
            variables: serde_json::json!({ "id": position_id }),
        };

        let body: PositionResp = self.post_with_retry(&req).await?;
        if let Some(pos) = body.position {
            info!(target: "uniswap.fetch", endpoint = %self.graph_endpoint, position_id = position_id, pool_id = %pos.pool.id, "resolved position to pool, fetching");
            self.get_pool_by_id(&pos.pool.id).await
        } else {
            info!(target: "uniswap.fetch", endpoint = %self.graph_endpoint, position_id = position_id, "position not found");
            Ok(None)
        }
    }

// ================= On-chain Position Manager fetcher =================
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnchainPosition {
    pub token_id: String,
    pub operator: String,
    pub token0: String,
    pub token1: String,
    pub token0_symbol: String,
    pub token1_symbol: String,
    pub fee: u32,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub liquidity: String,
    pub tokens_owed0: String,
    pub tokens_owed1: String,
    pub price_lower_quote_per_base: String,
    pub price_upper_quote_per_base: String,
    pub mid_price_quote_per_base: String,
}

impl UniswapClient {
    async fn eth_call_raw(&self, rpc_url: &str, to_addr: &str, data: &[u8]) -> Result<Vec<u8>> {
        let params = serde_json::json!({
            "to": to_addr,
            "data": format!("0x{}", hex::encode(data)),
        });
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_call",
            "params": [params, "latest"]
        });
        let resp = self.http.post(rpc_url).json(&body).send().await?.error_for_status()?;
        let json: serde_json::Value = resp.json().await?;
        let result_hex = json.get("result").and_then(|v| v.as_str()).unwrap_or("");
        if result_hex.is_empty() {
            return Err(anyhow::anyhow!("empty eth_call result"));
        }
        let bytes = hex::decode(result_hex.trim_start_matches("0x"))?;
        Ok(bytes)
    }

    async fn resolve_erc20_symbol(&self, rpc_url: &str, token_address_hex: &str) -> Result<String> {
        use sha3::{Digest, Keccak256};
        // Try symbol() -> string
        let symbol_selector = {
            let mut h = Keccak256::new();
            h.update(b"symbol()");
            let out = h.finalize();
            [out[0], out[1], out[2], out[3]]
        };
        let mut data = Vec::with_capacity(4);
        data.extend_from_slice(&symbol_selector);
        if let Ok(bytes) = self.eth_call_raw(rpc_url, token_address_hex, &data).await {
            if let Ok(tokens) = ethabi::decode(&[ParamType::String], &bytes) {
                if let Some(AbiToken::String(s)) = tokens.get(0).cloned() {
                    if !s.is_empty() { return Ok(s); }
                }
            }
        }
        // Fallback to bytes32
        let mut data = Vec::with_capacity(4);
        let bytes32_selector = {
            let mut h = Keccak256::new();
            h.update(b"symbol()bytes32"); // not standard; keep original selector
            let out = h.finalize();
            [out[0], out[1], out[2], out[3]]
        };
        data.extend_from_slice(&symbol_selector); // many tokens still use same selector but return bytes32
        if let Ok(bytes) = self.eth_call_raw(rpc_url, token_address_hex, &data).await {
            if let Ok(tokens) = ethabi::decode(&[ParamType::FixedBytes(32)], &bytes) {
                if let Some(AbiToken::FixedBytes(raw)) = tokens.get(0).cloned() {
                    let trimmed = String::from_utf8(raw.clone()).unwrap_or_default().trim_matches(char::from(0)).to_string();
                    if !trimmed.is_empty() { return Ok(trimmed); }
                }
            }
        }
        Ok(token_address_hex.to_string())
    }

    async fn resolve_erc20_decimals(&self, rpc_url: &str, token_address_hex: &str) -> u8 {
        use sha3::{Digest, Keccak256};
        let selector = {
            let mut h = Keccak256::new();
            h.update(b"decimals()");
            let out = h.finalize();
            [out[0], out[1], out[2], out[3]]
        };
        let mut data = Vec::with_capacity(4);
        data.extend_from_slice(&selector);
        if let Ok(bytes) = self.eth_call_raw(rpc_url, token_address_hex, &data).await {
            if let Ok(tokens) = ethabi::decode(&[ParamType::Uint(8)], &bytes) {
                if let Some(AbiToken::Uint(v)) = tokens.get(0).cloned() {
                    return v.low_u32() as u8;
                }
            }
        }
        18
    }

    pub async fn get_onchain_position(&self, rpc_url: &str, token_id: &str) -> Result<OnchainPosition> {
        // Encode call data for positions(uint256)
        let fn_selector = {
            // keccak256("positions(uint256)")[0..4]
            use sha3::{Digest, Keccak256};
            let mut hasher = Keccak256::new();
            hasher.update(b"positions(uint256)");
            let hash = hasher.finalize();
            [hash[0], hash[1], hash[2], hash[3]]
        };
        let id = U256::from_dec_str(token_id)?;
        let encoded_args = ethabi::encode(&[AbiToken::Uint(id.into())]);
        let mut data = Vec::with_capacity(4 + encoded_args.len());
        data.extend_from_slice(&fn_selector);
        data.extend_from_slice(&encoded_args);

        info!(target: "uniswap.onchain", token_id, "fetching on-chain position");
        let to_addr = "0xC36442b4a4522E871399CD717aBDD847Ab11FE88";
        let bytes = self.eth_call_raw(rpc_url, to_addr, &data).await?;

        // Decode tuple per ABI
        let output_types = vec![
            ParamType::Uint(96),               // nonce
            ParamType::Address,                // operator
            ParamType::Address,                // token0
            ParamType::Address,                // token1
            ParamType::Uint(24),               // fee
            ParamType::Int(24),                // tickLower
            ParamType::Int(24),                // tickUpper
            ParamType::Uint(128),              // liquidity
            ParamType::Uint(256),              // feeGrowthInside0LastX128
            ParamType::Uint(256),              // feeGrowthInside1LastX128
            ParamType::Uint(128),              // tokensOwed0
            ParamType::Uint(128),              // tokensOwed1
        ];
        let tokens = ethabi::decode(&output_types, &bytes)?;

        let operator = tokens[1].clone().into_address().unwrap();
        let token0 = tokens[2].clone().into_address().unwrap();
        let token1 = tokens[3].clone().into_address().unwrap();
        let fee_u256 = tokens[4].clone().into_uint().unwrap();
        let tick_lower_i256 = tokens[5].clone().into_int().unwrap();
        let tick_upper_i256 = tokens[6].clone().into_int().unwrap();
        let liquidity = tokens[7].clone().into_uint().unwrap();
        let owed0 = tokens[10].clone().into_uint().unwrap();
        let owed1 = tokens[11].clone().into_uint().unwrap();

        // Resolve token symbols
        let token0_hex = format!("0x{:x}", token0);
        let token1_hex = format!("0x{:x}", token1);
        let sym0_raw = self.resolve_erc20_symbol(rpc_url, &token0_hex).await.unwrap_or(token0_hex.clone());
        let sym1_raw = self.resolve_erc20_symbol(rpc_url, &token1_hex).await.unwrap_or(token1_hex.clone());
        let sym0 = self.alias_symbol(&token0_hex, &sym0_raw);
        let sym1 = self.alias_symbol(&token1_hex, &sym1_raw);

        // Decimals and price range (token1 per token0)
        let dec0 = self.resolve_erc20_decimals(rpc_url, &token0_hex).await as i32;
        let dec1 = self.resolve_erc20_decimals(rpc_url, &token1_hex).await as i32;
        // Price of token1 quoted in token0 units: 1.0001^tick * 10^(dec0 - dec1)
        let scale = 10f64.powi(dec0 - dec1);
        let price_lower = 1.0001f64.powi(tick_lower_i256.low_u32() as i32) * scale;
        let price_upper = 1.0001f64.powi(tick_upper_i256.low_u32() as i32) * scale;
        let mid_price = (price_lower * price_upper).sqrt();

        let pos = OnchainPosition {
            token_id: token_id.to_string(),
            operator: format!("0x{:x}", operator),
            token0: token0_hex,
            token1: token1_hex,
            token0_symbol: sym0,
            token1_symbol: sym1,
            fee: fee_u256.low_u32(),
            tick_lower: tick_lower_i256.low_u32() as i32,
            tick_upper: tick_upper_i256.low_u32() as i32,
            liquidity: liquidity.to_string(),
            tokens_owed0: owed0.to_string(),
            tokens_owed1: owed1.to_string(),
            price_lower_quote_per_base: format!("{:.2}", price_lower),
            price_upper_quote_per_base: format!("{:.2}", price_upper),
            mid_price_quote_per_base: format!("{:.2}", mid_price),
        };
        info!(target: "uniswap.onchain", token_id, liquidity = %pos.liquidity, fee = pos.fee, "fetched on-chain position");
        Ok(pos)
    }
}



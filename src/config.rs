use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

// =============================================================================
// BLOCKCHAIN CONFIGURATION
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    pub rpc_url: String,
    pub backup_rpc_urls: Option<Vec<String>>,
    pub origins_contract_address: String,
    pub origins_abi_path: Option<String>,
}

// =============================================================================
// API CONFIGURATION
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub coingecko_api_url: String,
    pub coinmarketcap_api_url: Option<String>,
    pub coinmarketcap_api_key: Option<String>,
    pub defipulse_api_url: Option<String>,
    pub thegraph_api_url: Option<String>,
    pub thegraph_api_key: Option<String>,
}

// =============================================================================
// RISK ASSESSMENT CONFIGURATION
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub max_risk_score: f64,
    pub min_liquidity_score: f64,
    pub volatility_threshold: f64,
}

// =============================================================================
// RECOMMENDATION CONFIGURATION
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationTypes {
    pub hold_recommendations: bool,
    pub increase_recommendations: bool,
    pub decrease_recommendations: bool,
    pub exit_recommendations: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationConfig {
    pub recommendation_interval: u64,
    pub recommendation_types: RecommendationTypes,
}

// =============================================================================
// LOGGING CONFIGURATION
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub log_level: String,
    pub detailed_logging: bool,
    pub performance_logging: bool,
}

// =============================================================================
// SECURITY CONFIGURATION
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasSettings {
    pub max_gas_price: u64,
    pub gas_limit: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub private_key: Option<String>,
    pub enable_transaction_signing: bool,
    pub gas_settings: GasSettings,
}

// =============================================================================
// MARKET DATA CONFIGURATION
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataConfig {
    pub market_data_refresh_interval: u64,
    pub real_time_prices: bool,
    pub price_sources: Vec<String>,
}

// =============================================================================
// NOTIFICATION CONFIGURATION
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub to_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannels {
    pub discord_webhook: Option<String>,
    pub slack_webhook: Option<String>,
    pub email: Option<EmailConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub notifications_enabled: bool,
    pub notification_channels: Option<NotificationChannels>,
}

// =============================================================================
// DEVELOPMENT CONFIGURATION
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockDataConfig {
    pub mock_positions_count: usize,
    pub mock_tokens: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentConfig {
    pub test_mode: bool,
    pub mock_data: MockDataConfig,
}

// =============================================================================
// UNISWAP CONFIGURATION
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniswapConfig {
    /// List of Uniswap v3 pool IDs (addresses) to quote on startup
    pub pool_ids: Vec<String>,
    /// Quote interval in seconds
    pub quote_interval_secs: u64,
    /// List of Uniswap v3 position NFT IDs to resolve and quote their pools
    pub position_ids: Vec<String>,
}

// =============================================================================
// MAIN CONFIGURATION STRUCTURE
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // Core blockchain settings
    pub rpc_url: String,
    pub origins_contract_address: String,
    pub position_threshold: f64,
    pub max_positions: usize,
    
    // Optional private key (for transaction signing)
    pub private_key: Option<String>,
    
    // Extended configuration sections
    pub blockchain: Option<BlockchainConfig>,
    pub api: Option<ApiConfig>,
    pub risk_assessment: Option<RiskAssessment>,
    pub recommendations: Option<RecommendationConfig>,
    pub logging: Option<LoggingConfig>,
    pub security: Option<SecurityConfig>,
    pub market_data: Option<MarketDataConfig>,
    pub notifications: Option<NotificationConfig>,
    pub development: Option<DevelopmentConfig>,
    pub uniswap: Option<UniswapConfig>,
}

impl Config {
    /// Load configuration from a TOML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
    
    /// Create a default configuration
    pub fn default() -> Self {
        Self {
            rpc_url: "https://mainnet.infura.io/v3/your-project-id".to_string(),
            origins_contract_address: "0x0000000000000000000000000000000000000000".to_string(),
            position_threshold: 0.1,
            max_positions: 10,
            private_key: None,
            blockchain: Some(BlockchainConfig {
                rpc_url: "https://mainnet.infura.io/v3/your-project-id".to_string(),
                backup_rpc_urls: None,
                origins_contract_address: "0x0000000000000000000000000000000000000000".to_string(),
                origins_abi_path: None,
            }),
            api: Some(ApiConfig {
                coingecko_api_url: "https://api.coingecko.com/api/v3".to_string(),
                coinmarketcap_api_url: None,
                coinmarketcap_api_key: None,
                defipulse_api_url: None,
                thegraph_api_url: None,
                thegraph_api_key: None,
            }),
            risk_assessment: Some(RiskAssessment {
                max_risk_score: 0.8,
                min_liquidity_score: 0.3,
                volatility_threshold: 0.5,
            }),
            recommendations: Some(RecommendationConfig {
                recommendation_interval: 300,
                recommendation_types: RecommendationTypes {
                    hold_recommendations: true,
                    increase_recommendations: true,
                    decrease_recommendations: true,
                    exit_recommendations: true,
                },
            }),
            logging: Some(LoggingConfig {
                log_level: "info".to_string(),
                detailed_logging: false,
                performance_logging: true,
            }),
            security: Some(SecurityConfig {
                private_key: None,
                enable_transaction_signing: false,
                gas_settings: GasSettings {
                    max_gas_price: 50,
                    gas_limit: 200000,
                },
            }),
            market_data: Some(MarketDataConfig {
                market_data_refresh_interval: 60,
                real_time_prices: true,
                price_sources: vec!["coingecko".to_string(), "coinmarketcap".to_string()],
            }),
            notifications: Some(NotificationConfig {
                notifications_enabled: false,
                notification_channels: None,
            }),
            development: Some(DevelopmentConfig {
                test_mode: false,
                mock_data: MockDataConfig {
                    mock_positions_count: 5,
                    mock_tokens: vec![
                        "0xA0b86a33E6441b8C4C8C0C4C0C4C0C4C0C4C0C4C".to_string(),
                        "0xB1c97d44F5552b8D5D5D5D5D5D5D5D5D5D5D5D5D".to_string(),
                        "0xC2d88e66F6663c9E6E6E6E6E6E6E6E6E6E6E6E6E".to_string(),
                    ],
                },
            }),
            uniswap: Some(UniswapConfig {
                pool_ids: Vec::new(),
                quote_interval_secs: 300,
                position_ids: Vec::new(),
            }),
        }
    }
    
    /// Get the recommendation interval, with fallback to default
    pub fn get_recommendation_interval(&self) -> u64 {
        self.recommendations
            .as_ref()
            .map(|r| r.recommendation_interval)
            .unwrap_or(300)
    }
    
    /// Get the log level, with fallback to default
    pub fn get_log_level(&self) -> &str {
        self.logging
            .as_ref()
            .map(|l| l.log_level.as_str())
            .unwrap_or("info")
    }
    
    /// Check if test mode is enabled
    pub fn is_test_mode(&self) -> bool {
        self.development
            .as_ref()
            .map(|d| d.test_mode)
            .unwrap_or(false)
    }
    
    /// Check if notifications are enabled
    pub fn notifications_enabled(&self) -> bool {
        self.notifications
            .as_ref()
            .map(|n| n.notifications_enabled)
            .unwrap_or(false)
    }
    
    /// Get backup RPC URLs
    pub fn get_backup_rpc_urls(&self) -> Vec<String> {
        self.blockchain
            .as_ref()
            .and_then(|b| b.backup_rpc_urls.clone())
            .unwrap_or_default()
    }
    
    /// Get API configuration
    pub fn get_api_config(&self) -> Option<&ApiConfig> {
        self.api.as_ref()
    }
    
    /// Get risk assessment configuration
    pub fn get_risk_assessment(&self) -> Option<&RiskAssessment> {
        self.risk_assessment.as_ref()
    }
    
    /// Get market data configuration
    pub fn get_market_data_config(&self) -> Option<&MarketDataConfig> {
        self.market_data.as_ref()
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate RPC URL
        if self.rpc_url.is_empty() {
            return Err(anyhow::anyhow!("RPC URL cannot be empty"));
        }
        
        // Validate position threshold
        if self.position_threshold < 0.0 {
            return Err(anyhow::anyhow!("Position threshold must be non-negative"));
        }
        
        // Validate max positions
        if self.max_positions == 0 {
            return Err(anyhow::anyhow!("Max positions must be greater than 0"));
        }
        
        // Validate contract address format (basic check)
        if !self.origins_contract_address.starts_with("0x") || self.origins_contract_address.len() != 42 {
            return Err(anyhow::anyhow!("Invalid contract address format"));
        }
        
        Ok(())
    }
}

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: String,
    pub user_address: String,
    pub token_address: String,
    pub amount: Decimal,
    pub value_usd: Decimal,
    pub risk_score: f64,
    pub liquidity_score: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionRecommendation {
    pub position: Position,
    pub recommendation_score: f64,
    pub reasoning: String,
    pub suggested_action: Action,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Hold,
    Increase,
    Decrease,
    Exit,
}

#[derive(Debug, Clone)]
pub struct PositionMetrics {
    pub total_value: Decimal,
    pub risk_distribution: HashMap<String, f64>,
    pub liquidity_distribution: HashMap<String, f64>,
    pub concentration_risk: f64,
}

impl Position {
    pub fn new(
        id: String,
        user_address: String,
        token_address: String,
        amount: Decimal,
        value_usd: Decimal,
    ) -> Self {
        Self {
            id,
            user_address,
            token_address,
            amount,
            value_usd,
            risk_score: 0.0,
            liquidity_score: 0.0,
            timestamp: chrono::Utc::now().timestamp() as u64,
        }
    }
    
    pub fn calculate_risk_score(&mut self, market_data: &MarketData) {
        // Simple risk calculation based on volatility and market cap
        let volatility = market_data.get_volatility(&self.token_address);
        let market_cap = market_data.get_market_cap(&self.token_address);
        
        self.risk_score = volatility * (1.0 / market_cap.sqrt());
    }
    
    pub fn calculate_liquidity_score(&mut self, market_data: &MarketData) {
        // Simple liquidity calculation based on volume and depth
        let volume = market_data.get_volume(&self.token_address);
        let depth = market_data.get_depth(&self.token_address);
        
        self.liquidity_score = volume * depth;
    }
}

#[derive(Debug, Clone)]
pub struct MarketData {
    pub token_data: HashMap<String, TokenData>,
}

#[derive(Debug, Clone)]
pub struct TokenData {
    pub volatility: f64,
    pub market_cap: f64,
    pub volume: f64,
    pub depth: f64,
}

impl MarketData {
    pub fn new() -> Self {
        Self {
            token_data: HashMap::new(),
        }
    }
    
    pub fn get_volatility(&self, token_address: &str) -> f64 {
        self.token_data
            .get(token_address)
            .map(|data| data.volatility)
            .unwrap_or(0.1) // Default volatility
    }
    
    pub fn get_market_cap(&self, token_address: &str) -> f64 {
        self.token_data
            .get(token_address)
            .map(|data| data.market_cap)
            .unwrap_or(1_000_000.0) // Default market cap
    }
    
    pub fn get_volume(&self, token_address: &str) -> f64 {
        self.token_data
            .get(token_address)
            .map(|data| data.volume)
            .unwrap_or(100_000.0) // Default volume
    }
    
    pub fn get_depth(&self, token_address: &str) -> f64 {
        self.token_data
            .get(token_address)
            .map(|data| data.depth)
            .unwrap_or(0.5) // Default depth
    }
}

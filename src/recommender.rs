use anyhow::Result;
use tracing::{info, error};
use std::collections::HashMap;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use crate::config::Config;
use crate::position::{Position, PositionRecommendation, PositionMetrics, MarketData, Action};

pub struct PositionRecommender {
    config: Config,
    market_data: MarketData,
    positions: Vec<Position>,
}

impl PositionRecommender {
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing position recommender");
        
        // Initialize market data (in a real implementation, this would fetch from APIs)
        let market_data = MarketData::new();
        
        Ok(Self {
            config,
            market_data,
            positions: Vec::new(),
        })
    }
    
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting position recommendation process");
        
        loop {
            match self.recommend_positions().await {
                Ok(recommendations) => {
                    info!("Generated {} position recommendations", recommendations.len());
                    self.display_recommendations(&recommendations);
                }
                Err(e) => {
                    error!("Error generating recommendations: {}", e);
                }
            }
            
            // Wait for the configured interval
            tokio::time::sleep(tokio::time::Duration::from_secs(self.config.get_recommendation_interval())).await;
        }
    }
    
    async fn recommend_positions(&mut self) -> Result<Vec<PositionRecommendation>> {
        info!("Analyzing positions and generating recommendations");
        
        // In a real implementation, this would:
        // 1. Fetch current positions from the blockchain
        // 2. Analyze market data
        // 3. Calculate risk metrics
        // 4. Generate recommendations
        
        let mut recommendations = Vec::new();
        
        // Simulate position analysis
        for position in &mut self.positions {
            position.calculate_risk_score(&self.market_data);
            position.calculate_liquidity_score(&self.market_data);
        }
        
        for position in &self.positions {
            let recommendation = self.analyze_position(position).await?;
            recommendations.push(recommendation);
        }
        
        // Sort by recommendation score
        recommendations.sort_by(|a, b| b.recommendation_score.partial_cmp(&a.recommendation_score).unwrap());
        
        // Limit to max positions
        recommendations.truncate(self.config.max_positions);
        
        Ok(recommendations)
    }
    
    async fn analyze_position(&self, position: &Position) -> Result<PositionRecommendation> {
        let recommendation_score = self.calculate_recommendation_score(position);
        let (suggested_action, reasoning) = self.determine_action(position, recommendation_score);
        
        Ok(PositionRecommendation {
            position: position.clone(),
            recommendation_score,
            reasoning,
            suggested_action,
        })
    }
    
    fn calculate_recommendation_score(&self, position: &Position) -> f64 {
        // Simple scoring algorithm
        let risk_factor = 1.0 - position.risk_score;
        let liquidity_factor = position.liquidity_score;
        let value_factor = position.value_usd.to_f64().unwrap_or(0.0) / 1000.0; // Normalize value
        
        (risk_factor * 0.4 + liquidity_factor * 0.4 + value_factor * 0.2).min(1.0)
    }
    
    fn determine_action(&self, _position: &Position, score: f64) -> (Action, String) {
        if score > 0.8 {
            (Action::Increase, "Strong fundamentals and low risk".to_string())
        } else if score > 0.6 {
            (Action::Hold, "Good position, maintain current allocation".to_string())
        } else if score > 0.4 {
            (Action::Decrease, "Consider reducing exposure due to risk factors".to_string())
        } else {
            (Action::Exit, "High risk or poor liquidity, consider exiting".to_string())
        }
    }
    
    fn display_recommendations(&self, recommendations: &[PositionRecommendation]) {
        info!("=== POSITION RECOMMENDATIONS ===");
        
        for (i, rec) in recommendations.iter().enumerate() {
            info!(
                "Recommendation {}: {} {} (Score: {:.2})",
                i + 1,
                format!("{:?}", rec.suggested_action),
                rec.position.token_address,
                rec.recommendation_score
            );
            info!("Reasoning: {}", rec.reasoning);
            info!("Value: ${:.2}", rec.position.value_usd);
            info!("---");
        }
    }
    
    pub fn add_position(&mut self, position: Position) {
        let position_id = position.id.clone();
        self.positions.push(position);
        info!("Added position: {}", position_id);
    }
    
    pub fn get_position_metrics(&self) -> PositionMetrics {
        let total_value: Decimal = self.positions.iter()
            .map(|p| p.value_usd)
            .sum();
        
        let mut risk_distribution = HashMap::new();
        let mut liquidity_distribution = HashMap::new();
        
        for position in &self.positions {
            let token = position.token_address.clone();
            *risk_distribution.entry(token.clone()).or_insert(0.0) += position.risk_score;
            *liquidity_distribution.entry(token).or_insert(0.0) += position.liquidity_score;
        }
        
        // Calculate concentration risk (simplified)
        let concentration_risk = if self.positions.len() > 1 {
            let max_value = self.positions.iter()
                .map(|p| p.value_usd.to_f64().unwrap_or(0.0))
                .fold(0.0, f64::max);
            max_value / total_value.to_f64().unwrap_or(1.0)
        } else {
            1.0
        };
        
        PositionMetrics {
            total_value,
            risk_distribution,
            liquidity_distribution,
            concentration_risk,
        }
    }
}

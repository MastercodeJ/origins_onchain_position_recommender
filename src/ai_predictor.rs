use anyhow::Result;
use rust_decimal::prelude::ToPrimitive;
use smartcore::linalg::basic::matrix::DenseMatrix;
use smartcore::linear::linear_regression::LinearRegression;
use smartcore::ensemble::random_forest_regressor::RandomForestRegressor;
use std::collections::HashMap;
use tracing::{info, warn, error};

use crate::position::{Position, MarketData};
use crate::config::Config;

/// AI-powered position predictor using multiple ML approaches
pub struct AIPredictor {
    config: Config,
    models: HashMap<String, Box<dyn PredictionModel>>,
    market_data: MarketData,
}

/// Trait for different prediction models
pub trait PredictionModel {
    fn predict(&self, features: &[f64]) -> Result<f64>;
    fn train(&mut self, features: &[Vec<f64>], targets: &[f64]) -> Result<()>;
    fn model_name(&self) -> &str;
}

/// Random Forest Model using SmartCore
pub struct RandomForestModel {
    model: Option<RandomForestRegressor<f64, f64, DenseMatrix<f64>, Vec<f64>>>,
}

impl RandomForestModel {
    pub fn new() -> Self {
        Self { model: None }
    }
}

impl PredictionModel for RandomForestModel {
    fn predict(&self, features: &[f64]) -> Result<f64> {
        if let Some(ref model) = self.model {
            let mat = DenseMatrix::from_2d_array(&[features]);
            let prediction = model.predict(&mat)?;
            Ok(prediction[0])
        } else {
            Err(anyhow::anyhow!("Model not trained"))
        }
    }

    fn train(&mut self, features: &[Vec<f64>], targets: &[f64]) -> Result<()> {
        let x = DenseMatrix::from_2d_vec(&features.to_vec());
        let y = targets.to_vec();
        let model: RandomForestRegressor<f64, f64, DenseMatrix<f64>, Vec<f64>> =
            RandomForestRegressor::fit(&x, &y, Default::default())?;
        self.model = Some(model);
        Ok(())
    }

    fn model_name(&self) -> &str {
        "RandomForest"
    }
}

/// Linear Regression Model using SmartCore
pub struct LinearRegressionModel {
    model: Option<LinearRegression<f64, f64, DenseMatrix<f64>, Vec<f64>>>,
}

impl LinearRegressionModel {
    pub fn new() -> Self {
        Self { model: None }
    }
}

impl PredictionModel for LinearRegressionModel {
    fn predict(&self, features: &[f64]) -> Result<f64> {
        if let Some(ref model) = self.model {
            let mat = DenseMatrix::from_2d_array(&[features]);
            let prediction = model.predict(&mat)?;
            Ok(prediction[0])
        } else {
            Err(anyhow::anyhow!("Model not trained"))
        }
    }

    fn train(&mut self, features: &[Vec<f64>], targets: &[f64]) -> Result<()> {
        let x = DenseMatrix::from_2d_vec(&features.to_vec());
        let y = targets.to_vec();
        let model: LinearRegression<f64, f64, DenseMatrix<f64>, Vec<f64>> =
            LinearRegression::fit(&x, &y, Default::default())?;
        self.model = Some(model);
        Ok(())
    }

    fn model_name(&self) -> &str {
        "LinearRegression"
    }
}

/// Ensemble Model that combines multiple predictions
pub struct EnsembleModel {
    models: Vec<Box<dyn PredictionModel>>,
    weights: Vec<f64>,
}

impl EnsembleModel {
    pub fn new() -> Self {
        Self {
            models: Vec::new(),
            weights: Vec::new(),
        }
    }

    pub fn add_model(&mut self, model: Box<dyn PredictionModel>, weight: f64) {
        self.models.push(model);
        self.weights.push(weight);
    }
}

impl PredictionModel for EnsembleModel {
    fn predict(&self, features: &[f64]) -> Result<f64> {
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;

        for (model, weight) in self.models.iter().zip(self.weights.iter()) {
            match model.predict(features) {
                Ok(prediction) => {
                    weighted_sum += prediction * weight;
                    total_weight += weight;
                }
                Err(e) => {
                    warn!("Model {} failed to predict: {}", model.model_name(), e);
                }
            }
        }

        if total_weight > 0.0 {
            Ok(weighted_sum / total_weight)
        } else {
            Err(anyhow::anyhow!("All models failed to predict"))
        }
    }

    fn train(&mut self, features: &[Vec<f64>], targets: &[f64]) -> Result<()> {
        for model in self.models.iter_mut() {
            if let Err(e) = model.train(features, targets) {
                warn!("Model {} failed to train: {}", model.model_name(), e);
            }
        }
        Ok(())
    }

    fn model_name(&self) -> &str {
        "Ensemble"
    }
}

impl AIPredictor {
    pub fn new(config: Config) -> Self {
        let mut predictor = Self {
            config,
            models: HashMap::new(),
            market_data: MarketData::new(),
        };

        // Initialize models
        predictor.initialize_models();
        predictor
    }

    fn initialize_models(&mut self) {
        // Add linear regression model
        let lr_model = Box::new(LinearRegressionModel::new());
        self.models.insert("linear_regression".to_string(), lr_model);

        // Add random forest model
        let rf_model = Box::new(RandomForestModel::new());
        self.models.insert("random_forest".to_string(), rf_model);

        // Add ensemble model
        let mut ensemble = EnsembleModel::new();
        ensemble.add_model(Box::new(RandomForestModel::new()), 0.5);
        ensemble.add_model(Box::new(LinearRegressionModel::new()), 0.3);
        self.models.insert("ensemble".to_string(), Box::new(ensemble));

        info!("Initialized {} AI models", self.models.len());
    }

    /// Extract features from a position for ML prediction
    pub fn extract_features(&self, position: &Position) -> Vec<f64> {
        vec![
            position.value_usd.to_f64().unwrap_or(0.0),
            position.risk_score,
            position.liquidity_score,
            self.market_data.get_volatility(&position.token_address),
            self.market_data.get_market_cap(&position.token_address),
            self.market_data.get_volume(&position.token_address),
            self.market_data.get_depth(&position.token_address),
            position.timestamp as f64,
            // Add more features as needed
            self.calculate_momentum_score(position),
            self.calculate_technical_indicators(position),
        ]
    }

    /// Calculate momentum score for a position
    fn calculate_momentum_score(&self, position: &Position) -> f64 {
        // Simple momentum calculation based on recent performance
        let volatility = self.market_data.get_volatility(&position.token_address);
        let volume = self.market_data.get_volume(&position.token_address);
        
        // Higher volume and lower volatility = better momentum
        volume / (volatility + 0.1) // Add small constant to avoid division by zero
    }

    /// Calculate technical indicators
    fn calculate_technical_indicators(&self, position: &Position) -> f64 {
        // Simple RSI-like calculation
        let market_cap = self.market_data.get_market_cap(&position.token_address);
        let volume = self.market_data.get_volume(&position.token_address);
        
        // Normalize to 0-1 range
        (volume / market_cap).min(1.0)
    }

    /// Train all models with historical data
    pub async fn train_models(&mut self, training_data: &[(Position, f64)]) -> Result<()> {
        if training_data.is_empty() {
            warn!("No training data provided, using default models");
            return Ok(());
        }

        info!("Training AI models with {} data points", training_data.len());

        // Extract features and targets
        let features: Vec<Vec<f64>> = training_data
            .iter()
            .map(|(position, _)| self.extract_features(position))
            .collect();
        
        let targets: Vec<f64> = training_data
            .iter()
            .map(|(_, target)| *target)
            .collect();

        // Train each model
        for (name, model) in self.models.iter_mut() {
            match model.train(&features, &targets) {
                Ok(_) => info!("Successfully trained model: {}", name),
                Err(e) => error!("Failed to train model {}: {}", name, e),
            }
        }

        Ok(())
    }

    /// Predict the recommendation score for a position
    pub async fn predict_recommendation_score(&self, position: &Position) -> Result<f64> {
        let features = self.extract_features(position);
        
        // Use ensemble model for prediction
        if let Some(ensemble_model) = self.models.get("ensemble") {
            match ensemble_model.predict(&features) {
                Ok(score) => {
                    info!("AI prediction for position {}: {:.3}", position.id, score);
                    Ok(score.clamp(0.0, 1.0)) // Clamp to 0-1 range
                }
                Err(e) => {
                    warn!("Ensemble model failed, using fallback: {}", e);
                    self.fallback_prediction(position)
                }
            }
        } else {
            self.fallback_prediction(position)
        }
    }

    /// Fallback prediction when AI models fail
    fn fallback_prediction(&self, position: &Position) -> Result<f64> {
        // Simple heuristic-based prediction
        let risk_factor = 1.0 - position.risk_score;
        let liquidity_factor = position.liquidity_score;
        let value_factor = (position.value_usd.to_f64().unwrap_or(0.0) / 1000.0).min(1.0);
        
        let score = (risk_factor * 0.4 + liquidity_factor * 0.4 + value_factor * 0.2).min(1.0);
        Ok(score)
    }

    /// Get model performance metrics
    pub fn get_model_performance(&self) -> HashMap<String, f64> {
        let mut performance = HashMap::new();
        
        for (name, model) in &self.models {
            // In a real implementation, you'd calculate actual performance metrics
            // For now, return placeholder values
            performance.insert(name.clone(), 0.85); // 85% accuracy placeholder
        }
        
        performance
    }

    /// Update market data for better predictions
    pub fn update_market_data(&mut self, new_market_data: MarketData) {
        self.market_data = new_market_data;
        info!("Updated market data for AI predictions");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_feature_extraction() {
        let config = Config::default();
        let predictor = AIPredictor::new(config);
        
        let position = Position::new(
            "test_id".to_string(),
            "0x123".to_string(),
            "0x456".to_string(),
            Decimal::from(100),
            Decimal::from(1000),
        );
        
        let features = predictor.extract_features(&position);
        assert_eq!(features.len(), 10);
    }

    #[test]
    fn test_ensemble_prediction() {
        let config = Config::default();
        let predictor = AIPredictor::new(config);
        
        let position = Position::new(
            "test_id".to_string(),
            "0x123".to_string(),
            "0x456".to_string(),
            Decimal::from(100),
            Decimal::from(1000),
        );
        
        // This will use fallback prediction since models aren't trained
        let result = tokio_test::block_on(predictor.predict_recommendation_score(&position));
        assert!(result.is_ok());
    }
}

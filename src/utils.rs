use anyhow::Result;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::str::FromStr;

/// Utility functions for the position recommender

/// Parse a decimal from string with proper error handling
pub fn parse_decimal(s: &str) -> Result<Decimal> {
    Decimal::from_str(s)
        .map_err(|e| anyhow::anyhow!("Failed to parse decimal '{}': {}", s, e))
}

/// Format a decimal as USD currency
pub fn format_usd(decimal: &Decimal) -> String {
    format!("${:.2}", decimal.to_f64().unwrap_or(0.0))
}

/// Calculate percentage change between two values
pub fn calculate_percentage_change(old_value: f64, new_value: f64) -> f64 {
    if old_value == 0.0 {
        0.0
    } else {
        ((new_value - old_value) / old_value) * 100.0
    }
}

/// Validate Ethereum address format
pub fn is_valid_ethereum_address(address: &str) -> bool {
    address.starts_with("0x") && address.len() == 42 && address[2..].chars().all(|c| c.is_ascii_hexdigit())
}

/// Convert wei to ether
pub fn wei_to_ether(wei: &str) -> Result<Decimal> {
    let wei_decimal = parse_decimal(wei)?;
    let ether = wei_decimal / Decimal::from(1_000_000_000_000_000_000u64);
    Ok(ether)
}

/// Convert ether to wei
pub fn ether_to_wei(ether: &Decimal) -> Decimal {
    ether * Decimal::from(1_000_000_000_000_000_000u64)
}

/// Calculate simple moving average
pub fn calculate_sma(values: &[f64], period: usize) -> Vec<f64> {
    if values.len() < period {
        return vec![];
    }
    
    let mut sma = Vec::new();
    for i in (period - 1)..values.len() {
        let sum: f64 = values[(i - period + 1)..=i].iter().sum();
        sma.push(sum / period as f64);
    }
    
    sma
}

/// Calculate exponential moving average
pub fn calculate_ema(values: &[f64], period: usize) -> Vec<f64> {
    if values.is_empty() {
        return vec![];
    }
    
    let multiplier = 2.0 / (period + 1) as f64;
    let mut ema = vec![values[0]];
    
    for &value in &values[1..] {
        let prev_ema = ema.last().unwrap();
        let new_ema = (value * multiplier) + (prev_ema * (1.0 - multiplier));
        ema.push(new_ema);
    }
    
    ema
}

/// Calculate volatility (standard deviation)
pub fn calculate_volatility(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values.iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>() / (values.len() - 1) as f64;
    
    variance.sqrt()
}

/// Safe division that handles zero division
pub fn safe_divide(numerator: f64, denominator: f64) -> f64 {
    if denominator == 0.0 {
        0.0
    } else {
        numerator / denominator
    }
}

/// Clamp a value between min and max
pub fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}

/// Normalize a value to 0-1 range
pub fn normalize(value: f64, min: f64, max: f64) -> f64 {
    if max == min {
        0.5
    } else {
        clamp((value - min) / (max - min), 0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_decimal() {
        assert!(parse_decimal("123.45").is_ok());
        assert!(parse_decimal("invalid").is_err());
    }

    #[test]
    fn test_ethereum_address_validation() {
        assert!(is_valid_ethereum_address("0x742d35Cc6634C0532925a3b8D0C4C5C5C5C5C5C5"));
        assert!(!is_valid_ethereum_address("0x742d35Cc6634C0532925a3b8D0C4C5C5C5C5C5C"));
        assert!(!is_valid_ethereum_address("742d35Cc6634C0532925a3b8D0C4C5C5C5C5C5C5"));
    }

    #[test]
    fn test_calculate_percentage_change() {
        assert_eq!(calculate_percentage_change(100.0, 110.0), 10.0);
        assert_eq!(calculate_percentage_change(100.0, 90.0), -10.0);
        assert_eq!(calculate_percentage_change(0.0, 100.0), 0.0);
    }

    #[test]
    fn test_sma_calculation() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let sma = calculate_sma(&values, 3);
        assert_eq!(sma, vec![2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_normalize() {
        assert_eq!(normalize(5.0, 0.0, 10.0), 0.5);
        assert_eq!(normalize(15.0, 0.0, 10.0), 1.0);
        assert_eq!(normalize(-5.0, 0.0, 10.0), 0.0);
    }
}

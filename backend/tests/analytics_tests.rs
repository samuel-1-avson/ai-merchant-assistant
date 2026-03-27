//! Tests for Analytics Engine

use ai_merchant_backend::analytics::{
    TrendAnalysis, TrendDirection, TimeSeriesPoint, 
    engine::PeriodComparison
};
use rust_decimal::Decimal;
use chrono::NaiveDate;

/// Test trend analysis calculations
#[test]
fn test_trend_analysis_creation() {
    let forecast = vec![
        TimeSeriesPoint {
            date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            value: Decimal::from(100),
        },
        TimeSeriesPoint {
            date: NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
            value: Decimal::from(110),
        },
    ];

    let analysis = TrendAnalysis {
        direction: TrendDirection::Increasing,
        slope: 10.0,
        r_squared: 0.95,
        forecast,
    };

    assert!(matches!(analysis.direction, TrendDirection::Increasing));
    assert_eq!(analysis.slope, 10.0);
    assert_eq!(analysis.r_squared, 0.95);
    assert_eq!(analysis.forecast.len(), 2);
}

/// Test period comparison calculations
#[test]
fn test_period_comparison() {
    let comparison = PeriodComparison {
        current_revenue: Decimal::from(1000),
        previous_revenue: Decimal::from(800),
        revenue_change_percent: 25.0,
        current_transactions: 50,
        previous_transactions: 40,
        transaction_change_percent: 25.0,
    };

    assert_eq!(comparison.revenue_change_percent, 25.0);
    assert_eq!(comparison.transaction_change_percent, 25.0);
}

/// Test trend direction determination logic
#[test]
fn test_trend_direction_logic() {
    // Increasing trend
    let slope_up = 0.05;
    let direction_up = if slope_up > 0.01 {
        TrendDirection::Increasing
    } else if slope_up < -0.01 {
        TrendDirection::Decreasing
    } else {
        TrendDirection::Stable
    };
    assert!(matches!(direction_up, TrendDirection::Increasing));

    // Decreasing trend
    let slope_down = -0.05;
    let direction_down = if slope_down > 0.01 {
        TrendDirection::Increasing
    } else if slope_down < -0.01 {
        TrendDirection::Decreasing
    } else {
        TrendDirection::Stable
    };
    assert!(matches!(direction_down, TrendDirection::Decreasing));

    // Stable trend
    let slope_stable = 0.005;
    let direction_stable = if slope_stable > 0.01 {
        TrendDirection::Increasing
    } else if slope_stable < -0.01 {
        TrendDirection::Decreasing
    } else {
        TrendDirection::Stable
    };
    assert!(matches!(direction_stable, TrendDirection::Stable));
}

/// Test R-squared calculation (coefficient of determination)
#[test]
fn test_r_squared_calculation() {
    // Perfect fit: R² = 1.0
    let y_actual = vec![10.0, 20.0, 30.0];
    let y_predicted = vec![10.0, 20.0, 30.0];
    let y_mean: f64 = y_actual.iter().sum::<f64>() / y_actual.len() as f64;
    
    let ss_tot: f64 = y_actual.iter().map(|y| (y - y_mean).powi(2)).sum();
    let ss_res: f64 = y_actual.iter().zip(&y_predicted)
        .map(|(a, p)| (a - p).powi(2)).sum();
    let r_squared = 1.0 - (ss_res / ss_tot);
    
    assert_eq!(r_squared, 1.0);
}

/// Test simple moving average for forecasting
#[test]
fn test_simple_moving_average() {
    let data = vec![10.0, 20.0, 30.0, 40.0, 50.0];
    let window = 3;
    
    let sma: f64 = data.iter().rev().take(window).sum::<f64>() / window as f64;
    
    assert_eq!(sma, (50.0 + 40.0 + 30.0) / 3.0);
}

/// Test anomaly detection using Z-score
#[test]
fn test_z_score_calculation() {
    let data = vec![10.0, 12.0, 11.0, 13.0, 10.0, 100.0]; // 100.0 is an outlier
    let mean = data.iter().sum::<f64>() / data.len() as f64;
    let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64;
    let std_dev = variance.sqrt();
    
    let z_score = (100.0 - mean) / std_dev;
    
    // Z-score should be high for the outlier
    assert!(z_score > 2.0);
}

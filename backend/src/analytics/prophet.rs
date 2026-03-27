//! Advanced time-series forecasting using statistical methods
//! Inspired by Facebook Prophet's additive regression model

use chrono::{NaiveDate, Duration};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use uuid::Uuid;

/// Trend component types
#[derive(Debug, Clone)]
pub enum TrendType {
    Linear,
    Logistic,
    Flat,
}

/// Seasonality configuration
#[derive(Debug, Clone)]
pub struct Seasonality {
    pub period: i32,  // Period in days
    pub fourier_order: i32,
    pub prior_scale: f64,
}

/// Holiday/Special event
#[derive(Debug, Clone)]
pub struct Holiday {
    pub name: String,
    pub date: NaiveDate,
    pub lower_window: i32,
    pub upper_window: i32,
    pub prior_scale: f64,
}

/// Prophet-like forecasting model
pub struct ProphetModel {
    pub trend_type: TrendType,
    pub yearly_seasonality: Option<Seasonality>,
    pub weekly_seasonality: Option<Seasonality>,
    pub daily_seasonality: Option<Seasonality>,
    pub holidays: Vec<Holiday>,
    pub changepoint_prior_scale: f64,
    pub seasonality_prior_scale: f64,
}

impl Default for ProphetModel {
    fn default() -> Self {
        Self {
            trend_type: TrendType::Linear,
            yearly_seasonality: Some(Seasonality {
                period: 365,
                fourier_order: 10,
                prior_scale: 10.0,
            }),
            weekly_seasonality: Some(Seasonality {
                period: 7,
                fourier_order: 3,
                prior_scale: 10.0,
            }),
            daily_seasonality: None,
            holidays: Vec::new(),
            changepoint_prior_scale: 0.05,
            seasonality_prior_scale: 10.0,
        }
    }
}

impl ProphetModel {
    /// Fit model to historical data and generate forecast
    pub fn forecast(
        &self,
        dates: &[NaiveDate],
        values: &[f64],
        periods: i64,
    ) -> anyhow::Result<ProphetForecast> {
        if dates.len() < 2 || values.len() < 2 {
            return Err(anyhow::anyhow!("Need at least 2 data points"));
        }

        // Calculate trend (simple linear regression)
        let trend = self.calculate_trend(dates, values)?;
        
        // Calculate weekly seasonality
        let weekly_effect = self.calculate_weekly_seasonality(dates, values)?;
        
        // Calculate yearly seasonality (if enough data)
        let yearly_effect = if dates.len() >= 365 {
            self.calculate_yearly_seasonality(dates, values)?
        } else {
            vec![0.0; dates.len()]
        };

        // Generate future dates
        let last_date = *dates.last().unwrap();
        let mut future_dates = Vec::new();
        for i in 1..=periods {
            future_dates.push(last_date + Duration::days(i));
        }

        // Generate forecast
        let mut forecast_values = Vec::new();
        let mut lower_bounds = Vec::new();
        let mut upper_bounds = Vec::new();

        for (i, date) in future_dates.iter().enumerate() {
            let trend_val = trend.slope * (dates.len() + i) as f64 + trend.intercept;
            let weekly_val = self.get_weekly_effect(*date, &weekly_effect);
            let yearly_val = self.get_yearly_effect(*date, &yearly_effect);
            
            let forecast = trend_val + weekly_val + yearly_val;
            let uncertainty = trend.uncertainty * (1.0 + i as f64 * 0.05); // Increasing uncertainty
            
            forecast_values.push(forecast);
            lower_bounds.push(forecast - 1.96 * uncertainty);
            upper_bounds.push(forecast + 1.96 * uncertainty);
        }

        Ok(ProphetForecast {
            dates: future_dates,
            forecast: forecast_values,
            lower_bound: lower_bounds,
            upper_bound: upper_bounds,
            trend: (0..periods as usize).map(|i| trend.slope * i as f64 + trend.intercept).collect(),
            weekly: (0..periods).map(|i| {
                self.get_weekly_effect(last_date + Duration::days(i + 1), &weekly_effect)
            }).collect(),
            yearly: (0..periods).map(|i| {
                self.get_yearly_effect(last_date + Duration::days(i + 1), &yearly_effect)
            }).collect(),
        })
    }

    /// Calculate linear trend
    fn calculate_trend(&self, dates: &[NaiveDate], values: &[f64]) -> anyhow::Result<TrendResult> {
        let n = values.len() as f64;
        let sum_x: f64 = (0..values.len()).map(|i| i as f64).sum();
        let sum_y: f64 = values.iter().sum();
        let sum_xy: f64 = values.iter().enumerate().map(|(i, y)| i as f64 * y).sum();
        let sum_x2: f64 = (0..values.len()).map(|i| (i as f64).powi(2)).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));
        let intercept = (sum_y - slope * sum_x) / n;

        // Calculate uncertainty (standard error)
        let predictions: Vec<f64> = (0..values.len())
            .map(|i| slope * i as f64 + intercept)
            .collect();
        let residuals: Vec<f64> = values.iter()
            .zip(&predictions)
            .map(|(actual, pred)| actual - pred)
            .collect();
        let mse: f64 = residuals.iter().map(|r| r.powi(2)).sum::<f64>() / (n - 2.0);
        let uncertainty = mse.sqrt();

        Ok(TrendResult {
            slope,
            intercept,
            uncertainty,
        })
    }

    /// Calculate weekly seasonality using Fourier series
    fn calculate_weekly_seasonality(
        &self,
        dates: &[NaiveDate],
        values: &[f64],
    ) -> anyhow::Result<Vec<f64>> {
        let mut day_averages = vec![0.0; 7];
        let mut day_counts = vec![0; 7];

        for (date, value) in dates.iter().zip(values.iter()) {
            let day_of_week = date.weekday().num_days_from_monday() as usize;
            day_averages[day_of_week] += value;
            day_counts[day_of_week] += 1;
        }

        for i in 0..7 {
            if day_counts[i] > 0 {
                day_averages[i] /= day_counts[i] as f64;
            }
        }

        // Calculate deviation from mean
        let overall_mean = values.iter().sum::<f64>() / values.len() as f64;
        let seasonality: Vec<f64> = day_averages.iter()
            .map(|&avg| avg - overall_mean)
            .collect();

        Ok(seasonality)
    }

    /// Calculate yearly seasonality using Fourier series
    fn calculate_yearly_seasonality(
        &self,
        _dates: &[NaiveDate],
        _values: &[f64],
    ) -> anyhow::Result<Vec<f64>> {
        // Simplified - would use Fourier transform in full implementation
        Ok(vec![0.0; 365])
    }

    /// Get weekly effect for a specific date
    fn get_weekly_effect(&self, date: NaiveDate, weekly_effect: &[f64]) -> f64 {
        let day_of_week = date.weekday().num_days_from_monday() as usize;
        weekly_effect.get(day_of_week).copied().unwrap_or(0.0)
    }

    /// Get yearly effect for a specific date
    fn get_yearly_effect(&self, _date: NaiveDate, _yearly_effect: &[f64]) -> f64 {
        // Simplified implementation
        0.0
    }

    /// Detect changepoints in the trend
    pub fn detect_changepoints(&self, values: &[f64]) -> Vec<usize> {
        let mut changepoints = Vec::new();
        let window_size = 7; // Weekly window

        for i in window_size..values.len() - window_size {
            let before: f64 = values[i - window_size..i].iter().sum::<f64>() / window_size as f64;
            let after: f64 = values[i..i + window_size].iter().sum::<f64>() / window_size as f64;
            let change = (after - before).abs();

            if change > self.changepoint_prior_scale * before.abs() {
                changepoints.push(i);
            }
        }

        changepoints
    }

    /// Add holidays to the model
    pub fn add_holiday(&mut self, holiday: Holiday) {
        self.holidays.push(holiday);
    }
}

#[derive(Debug, Clone)]
pub struct TrendResult {
    pub slope: f64,
    pub intercept: f64,
    pub uncertainty: f64,
}

#[derive(Debug, Clone)]
pub struct ProphetForecast {
    pub dates: Vec<NaiveDate>,
    pub forecast: Vec<f64>,
    pub lower_bound: Vec<f64>,
    pub upper_bound: Vec<f64>,
    pub trend: Vec<f64>,
    pub weekly: Vec<f64>,
    pub yearly: Vec<f64>,
}

/// Cross-validation for forecast accuracy
pub struct CrossValidator;

impl CrossValidator {
    /// Perform cross-validation
    pub fn cross_validate(
        dates: &[NaiveDate],
        values: &[f64],
        initial: i64,
        period: i64,
        horizon: i64,
    ) -> Vec<CrossValidationResult> {
        let mut results = Vec::new();
        let mut cutoff = initial;

        while cutoff + horizon <= dates.len() as i64 {
            let train_dates = &dates[..cutoff as usize];
            let train_values = &values[..cutoff as usize];
            let test_dates = &dates[cutoff as usize..(cutoff + horizon) as usize];
            let test_values = &values[cutoff as usize..(cutoff + horizon) as usize];

            // Fit model and predict
            let model = ProphetModel::default();
            if let Ok(forecast) = model.forecast(train_dates, train_values, horizon) {
                // Calculate metrics
                let mape = Self::calculate_mape(test_values, &forecast.forecast);
                let rmse = Self::calculate_rmse(test_values, &forecast.forecast);

                results.push(CrossValidationResult {
                    cutoff: dates[cutoff as usize - 1],
                    mape,
                    rmse,
                });
            }

            cutoff += period;
        }

        results
    }

    fn calculate_mape(actual: &[f64], predicted: &[f64]) -> f64 {
        let sum: f64 = actual.iter()
            .zip(predicted.iter())
            .map(|(a, p)| ((a - p) / a).abs())
            .sum();
        sum / actual.len() as f64 * 100.0
    }

    fn calculate_rmse(actual: &[f64], predicted: &[f64]) -> f64 {
        let sum_sq: f64 = actual.iter()
            .zip(predicted.iter())
            .map(|(a, p)| (a - p).powi(2))
            .sum();
        (sum_sq / actual.len() as f64).sqrt()
    }
}

#[derive(Debug, Clone)]
pub struct CrossValidationResult {
    pub cutoff: NaiveDate,
    pub mape: f64,
    pub rmse: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_prophet_forecast() {
        let dates: Vec<NaiveDate> = (0..30)
            .map(|i| NaiveDate::from_ymd_opt(2024, 1, 1).unwrap() + Duration::days(i))
            .collect();
        let values: Vec<f64> = (0..30).map(|i| 100.0 + i as f64 * 2.0).collect();

        let model = ProphetModel::default();
        let forecast = model.forecast(&dates, &values, 7).unwrap();

        assert_eq!(forecast.dates.len(), 7);
        assert_eq!(forecast.forecast.len(), 7);
        assert_eq!(forecast.lower_bound.len(), 7);
        assert_eq!(forecast.upper_bound.len(), 7);
    }

    #[test]
    fn test_changepoint_detection() {
        // Create data with a clear changepoint
        let values: Vec<f64> = (0..50)
            .map(|i| if i < 25 { 100.0 } else { 150.0 })
            .collect();

        let model = ProphetModel::default();
        let changepoints = model.detect_changepoints(&values);

        assert!(!changepoints.is_empty());
        // Changepoint should be detected around index 25
        assert!(changepoints.iter().any(|&cp| cp >= 20 && cp <= 30));
    }

    #[test]
    fn test_cross_validation() {
        let dates: Vec<NaiveDate> = (0..60)
            .map(|i| NaiveDate::from_ymd_opt(2024, 1, 1).unwrap() + Duration::days(i))
            .collect();
        let values: Vec<f64> = (0..60).map(|i| 100.0 + i as f64 * 1.5).collect();

        let results = CrossValidator::cross_validate(&dates, &values, 30, 10, 7);

        assert!(!results.is_empty());
        for result in &results {
            assert!(result.mape >= 0.0);
            assert!(result.rmse >= 0.0);
        }
    }
}

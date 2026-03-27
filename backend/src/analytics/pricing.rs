//! Price Optimization Engine - Dynamic pricing suggestions based on demand elasticity

use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use uuid::Uuid;
use std::collections::HashMap;

/// Price elasticity model
#[derive(Debug, Clone)]
pub struct ElasticityModel {
    pub product_id: Uuid,
    pub current_price: Decimal,
    pub current_demand: f64, // Units sold per period
    pub price_elasticity: f64, // Negative value (e.g., -1.5 means 10% price increase → 15% demand decrease)
    pub optimal_price: Decimal,
    pub expected_revenue_change: f64,
}

/// Competitor pricing data
#[derive(Debug, Clone)]
pub struct CompetitorPrice {
    pub competitor_name: String,
    pub price: Decimal,
    pub last_updated: chrono::NaiveDate,
}

/// Price optimization recommendation
#[derive(Debug, Clone)]
pub struct PriceRecommendation {
    pub product_id: Uuid,
    pub product_name: String,
    pub current_price: Decimal,
    pub recommended_price: Decimal,
    pub confidence: f64,
    pub reason: PriceReason,
    pub expected_impact: ExpectedImpact,
}

#[derive(Debug, Clone)]
pub enum PriceReason {
    HighDemand,
    LowDemand,
    CompetitorPricing,
    SeasonalTrend,
    InventoryLevel,
    MarginOptimization,
}

#[derive(Debug, Clone)]
pub struct ExpectedImpact {
    pub revenue_change_percent: f64,
    pub volume_change_percent: f64,
    pub margin_change_percent: f64,
}

/// Price optimization engine
pub struct PricingEngine;

impl PricingEngine {
    /// Calculate price elasticity of demand
    /// Formula: % Change in Quantity / % Change in Price
    pub fn calculate_elasticity(
        price_history: &[(Decimal, f64)], // (price, quantity_sold)
    ) -> f64 {
        if price_history.len() < 2 {
            return -1.0; // Default elasticity
        }

        let mut elasticities = Vec::new();
        
        for i in 1..price_history.len() {
            let (price1, qty1) = price_history[i - 1];
            let (price2, qty2) = price_history[i];

            let price_change = (price2 - price1).to_f64().unwrap_or(0.0) / price1.to_f64().unwrap_or(1.0);
            let qty_change = (qty2 - qty1) / qty1;

            if price_change != 0.0 {
                let elasticity = qty_change / price_change;
                elasticities.push(elasticity);
            }
        }

        if elasticities.is_empty() {
            -1.0
        } else {
            elasticities.iter().sum::<f64>() / elasticities.len() as f64
        }
    }

    /// Find optimal price to maximize revenue
    pub fn optimize_price(
        product_id: Uuid,
        product_name: &str,
        current_price: Decimal,
        cost_price: Decimal,
        current_demand: f64,
        elasticity: f64,
        min_margin_percent: f64,
    ) -> PriceRecommendation {
        let current_revenue = current_price.to_f64().unwrap_or(0.0) * current_demand;
        let current_margin = (current_price - cost_price).to_f64().unwrap_or(0.0) * current_demand;

        // Test different price points
        let test_prices: Vec<f64> = (70..=130).step_by(5)
            .map(|p| current_price.to_f64().unwrap_or(0.0) * p as f64 / 100.0)
            .collect();

        let mut best_price = current_price;
        let mut best_revenue = current_revenue;
        let mut best_margin = current_margin;

        for test_price in test_prices {
            // Calculate expected demand at new price
            let price_change_percent = (test_price - current_price.to_f64().unwrap_or(0.0)) 
                / current_price.to_f64().unwrap_or(1.0);
            let demand_change_percent = elasticity * price_change_percent;
            let expected_demand = current_demand * (1.0 + demand_change_percent);

            // Calculate expected revenue and margin
            let expected_revenue = test_price * expected_demand;
            let margin_per_unit = test_price - cost_price.to_f64().unwrap_or(0.0);
            let expected_margin = margin_per_unit * expected_demand;
            let margin_percent = (margin_per_unit / test_price) * 100.0;

            // Check minimum margin constraint
            if margin_percent >= min_margin_percent && expected_revenue > best_revenue {
                best_price = Decimal::from_f64(test_price).unwrap_or(current_price);
                best_revenue = expected_revenue;
                best_margin = expected_margin;
            }
        }

        // Calculate expected impact
        let revenue_change = (best_revenue - current_revenue) / current_revenue * 100.0;
        let price_change = (best_price - current_price).to_f64().unwrap_or(0.0) 
            / current_price.to_f64().unwrap_or(1.0) * 100.0;
        let volume_change = elasticity * price_change;
        let margin_change = (best_margin - current_margin) / current_margin * 100.0;

        let reason = if best_price > current_price {
            PriceReason::MarginOptimization
        } else if best_price < current_price {
            PriceReason::HighDemand
        } else {
            PriceReason::MarginOptimization
        };

        PriceRecommendation {
            product_id,
            product_name: product_name.to_string(),
            current_price,
            recommended_price: best_price,
            confidence: 0.75,
            reason,
            expected_impact: ExpectedImpact {
                revenue_change_percent: revenue_change,
                volume_change_percent: volume_change,
                margin_change_percent: margin_change,
            },
        }
    }

    /// Suggest prices based on competitor analysis
    pub fn competitor_based_pricing(
        product_id: Uuid,
        product_name: &str,
        current_price: Decimal,
        cost_price: Decimal,
        competitor_prices: &[CompetitorPrice],
        strategy: PricingStrategy,
    ) -> PriceRecommendation {
        if competitor_prices.is_empty() {
            return Self::optimize_price(
                product_id,
                product_name,
                current_price,
                cost_price,
                100.0,
                -1.0,
                20.0,
            );
        }

        let competitor_avg: Decimal = competitor_prices.iter()
            .map(|c| c.price)
            .fold(Decimal::ZERO, |acc, x| acc + x) / Decimal::from(competitor_prices.len() as i32);

        let recommended_price = match strategy {
            PricingStrategy::MatchCompetitors => competitor_avg,
            PricingStrategy::Undercut(percent) => {
                competitor_avg * Decimal::from_f64(1.0 - percent / 100.0).unwrap_or(Decimal::ONE)
            }
            PricingStrategy::Premium(percent) => {
                competitor_avg * Decimal::from_f64(1.0 + percent / 100.0).unwrap_or(Decimal::ONE)
            }
            PricingStrategy::MaintainMargin => {
                let min_price = cost_price * Decimal::from(12) / Decimal::from(10); // 20% margin
                if competitor_avg > min_price {
                    competitor_avg
                } else {
                    min_price
                }
            }
        };

        let price_change = (recommended_price - current_price).to_f64().unwrap_or(0.0) 
            / current_price.to_f64().unwrap_or(1.0) * 100.0;

        PriceRecommendation {
            product_id,
            product_name: product_name.to_string(),
            current_price,
            recommended_price,
            confidence: 0.70,
            reason: PriceReason::CompetitorPricing,
            expected_impact: ExpectedImpact {
                revenue_change_percent: -price_change * 0.5, // Assume -0.5 elasticity
                volume_change_percent: price_change * 0.5,
                margin_change_percent: 0.0,
            },
        }
    }

    /// Dynamic pricing based on inventory levels
    pub fn inventory_based_pricing(
        product_id: Uuid,
        product_name: &str,
        current_price: Decimal,
        cost_price: Decimal,
        stock_quantity: i32,
        avg_daily_sales: f64,
    ) -> Option<PriceRecommendation> {
        let days_of_stock = if avg_daily_sales > 0.0 {
            stock_quantity as f64 / avg_daily_sales
        } else {
            999.0
        };

        if days_of_stock < 3.0 {
            // Low stock - increase price to slow demand
            let new_price = current_price * Decimal::from_f64(1.15).unwrap_or(Decimal::ONE);
            Some(PriceRecommendation {
                product_id,
                product_name: product_name.to_string(),
                current_price,
                recommended_price: new_price,
                confidence: 0.80,
                reason: PriceReason::InventoryLevel,
                expected_impact: ExpectedImpact {
                    revenue_change_percent: 5.0,
                    volume_change_percent: -10.0,
                    margin_change_percent: 15.0,
                },
            })
        } else if days_of_stock > 90.0 {
            // Overstocked - decrease price to increase demand
            let new_price = current_price * Decimal::from_f64(0.90).unwrap_or(Decimal::ONE);
            Some(PriceRecommendation {
                product_id,
                product_name: product_name.to_string(),
                current_price,
                recommended_price: new_price,
                confidence: 0.75,
                reason: PriceReason::InventoryLevel,
                expected_impact: ExpectedImpact {
                    revenue_change_percent: 8.0,
                    volume_change_percent: 20.0,
                    margin_change_percent: -10.0,
                },
            })
        } else {
            None
        }
    }

    /// Bundle pricing recommendations
    pub fn suggest_bundle(
        products: &[(Uuid, &str, Decimal)],
        frequently_bought_together: &[(Uuid, Uuid)],
    ) -> Vec<BundleSuggestion> {
        let mut bundles = Vec::new();

        for (id1, id2) in frequently_bought_together.iter().take(10) {
            if let (Some(p1), Some(p2)) = (
                products.iter().find(|(id, _, _)| id == id1),
                products.iter().find(|(id, _, _)| id == id2),
            ) {
                let total_price = p1.2 + p2.2;
                let bundle_price = total_price * Decimal::from_f64(0.90).unwrap_or(Decimal::ONE);
                let savings = total_price - bundle_price;

                bundles.push(BundleSuggestion {
                    name: format!("{} + {}", p1.1, p2.1),
                    product_ids: vec![*id1, *id2],
                    individual_price: total_price,
                    bundle_price,
                    savings,
                    expected_uplift: 25.0, // 25% increase in combined sales
                });
            }
        }

        bundles
    }
}

#[derive(Debug, Clone)]
pub enum PricingStrategy {
    MatchCompetitors,
    Undercut(f64), // Percentage to undercut
    Premium(f64),  // Percentage premium
    MaintainMargin,
}

#[derive(Debug, Clone)]
pub struct BundleSuggestion {
    pub name: String,
    pub product_ids: Vec<Uuid>,
    pub individual_price: Decimal,
    pub bundle_price: Decimal,
    pub savings: Decimal,
    pub expected_uplift: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_elasticity() {
        let history = vec![
            (Decimal::from(10), 100.0),
            (Decimal::from(11), 90.0),  // 10% price increase, 10% demand decrease
        ];

        let elasticity = PricingEngine::calculate_elasticity(&history);
        
        // Should be around -1.0
        assert!(elasticity < 0.0);
        assert!(elasticity > -2.0);
    }

    #[test]
    fn test_optimize_price() {
        let recommendation = PricingEngine::optimize_price(
            Uuid::new_v4(),
            "Test Product",
            Decimal::from(100),
            Decimal::from(50),
            100.0,
            -1.2,
            20.0,
        );

        assert!(recommendation.confidence > 0.0);
        assert!(recommendation.expected_impact.revenue_change_percent >= -100.0);
    }

    #[test]
    fn test_inventory_based_pricing_low_stock() {
        let recommendation = PricingEngine::inventory_based_pricing(
            Uuid::new_v4(),
            "Test Product",
            Decimal::from(100),
            Decimal::from(50),
            5,       // 5 units in stock
            2.0,     // 2 units sold per day
        );

        assert!(recommendation.is_some());
        let rec = recommendation.unwrap();
        assert!(rec.recommended_price > rec.current_price);
    }

    #[test]
    fn test_inventory_based_pricing_overstock() {
        let recommendation = PricingEngine::inventory_based_pricing(
            Uuid::new_v4(),
            "Test Product",
            Decimal::from(100),
            Decimal::from(50),
            200,     // 200 units in stock
            1.0,     // 1 unit sold per day
        );

        assert!(recommendation.is_some());
        let rec = recommendation.unwrap();
        assert!(rec.recommended_price < rec.current_price);
    }
}

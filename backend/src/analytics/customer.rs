//! Customer Analytics - Cohort Analysis and Lifetime Value calculations

use chrono::{NaiveDate, Duration};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use uuid::Uuid;
use std::collections::HashMap;

/// Customer cohort identifier
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct CohortId {
    pub year: i32,
    pub month: u32,
    pub week: u32,
}

impl CohortId {
    pub fn from_date(date: NaiveDate) -> Self {
        Self {
            year: date.year(),
            month: date.month(),
            week: date.iso_week().week(),
        }
    }

    pub fn to_string(&self, period: CohortPeriod) -> String {
        match period {
            CohortPeriod::Monthly => format!("{:04}-{:02}", self.year, self.month),
            CohortPeriod::Weekly => format!("{:04}-W{:02}", self.year, self.week),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CohortPeriod {
    Monthly,
    Weekly,
}

/// Customer transaction data
#[derive(Debug, Clone)]
pub struct CustomerTransaction {
    pub customer_id: Uuid,
    pub transaction_id: Uuid,
    pub date: NaiveDate,
    pub amount: Decimal,
}

/// Cohort analysis result
#[derive(Debug, Clone)]
pub struct CohortAnalysis {
    pub cohort_id: String,
    pub cohort_size: usize,
    pub periods: Vec<CohortPeriodData>,
    pub retention_curve: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct CohortPeriodData {
    pub period: i32, // Period number (0 = acquisition, 1 = first month, etc.)
    pub active_customers: usize,
    pub total_revenue: Decimal,
    pub average_order_value: Decimal,
}

/// Customer Lifetime Value calculation
#[derive(Debug, Clone)]
pub struct CustomerLTV {
    pub customer_id: Uuid,
    pub first_purchase: NaiveDate,
    pub last_purchase: NaiveDate,
    pub total_orders: i32,
    pub total_revenue: Decimal,
    pub average_order_value: Decimal,
    pub purchase_frequency: f64, // Orders per month
    pub predicted_ltv: Decimal,
}

pub struct CustomerAnalytics;

impl CustomerAnalytics {
    /// Perform cohort analysis on customer transaction data
    pub fn cohort_analysis(
        transactions: &[CustomerTransaction],
        period: CohortPeriod,
        periods_to_analyze: i32,
    ) -> Vec<CohortAnalysis> {
        // Group customers by cohort
        let mut cohorts: HashMap<String, Vec<Uuid>> = HashMap::new();
        let mut customer_first_purchase: HashMap<Uuid, NaiveDate> = HashMap::new();

        for tx in transactions {
            if !customer_first_purchase.contains_key(&tx.customer_id) {
                customer_first_purchase.insert(tx.customer_id, tx.date);
                
                let cohort_id = CohortId::from_date(tx.date).to_string(period);
                cohorts.entry(cohort_id)
                    .or_default()
                    .push(tx.customer_id);
            }
        }

        // Calculate retention for each cohort
        let mut results = Vec::new();
        for (cohort_id, customers) in cohorts {
            let mut periods = Vec::new();
            let mut retention_curve = Vec::new();

            for period_num in 0..=periods_to_analyze {
                let active_customers = Self::count_active_customers(
                    transactions,
                    &customers,
                    &cohort_id,
                    period,
                    period_num,
                );

                let period_revenue = Self::calculate_period_revenue(
                    transactions,
                    &customers,
                    &cohort_id,
                    period,
                    period_num,
                );

                let aov = if active_customers > 0 {
                    period_revenue / Decimal::from(active_customers)
                } else {
                    Decimal::ZERO
                };

                periods.push(CohortPeriodData {
                    period: period_num,
                    active_customers,
                    total_revenue: period_revenue,
                    average_order_value: aov,
                });

                let retention = if !customers.is_empty() {
                    active_customers as f64 / customers.len() as f64 * 100.0
                } else {
                    0.0
                };
                retention_curve.push(retention);
            }

            results.push(CohortAnalysis {
                cohort_id,
                cohort_size: customers.len(),
                periods,
                retention_curve,
            });
        }

        // Sort by cohort_id
        results.sort_by(|a, b| a.cohort_id.cmp(&b.cohort_id));
        results
    }

    fn count_active_customers(
        transactions: &[CustomerTransaction],
        cohort_customers: &[Uuid],
        cohort_id: &str,
        period: CohortPeriod,
        period_num: i32,
    ) -> usize {
        // Parse cohort start date
        let cohort_start = Self::parse_cohort_date(cohort_id, period);
        
        let period_start = match period {
            CohortPeriod::Monthly => cohort_start + Duration::days(30 * period_num as i64),
            CohortPeriod::Weekly => cohort_start + Duration::weeks(period_num as i64),
        };

        let period_end = match period {
            CohortPeriod::Monthly => period_start + Duration::days(30),
            CohortPeriod::Weekly => period_start + Duration::weeks(1),
        };

        let mut active = std::collections::HashSet::new();
        for tx in transactions {
            if cohort_customers.contains(&tx.customer_id)
                && tx.date >= period_start
                && tx.date < period_end
            {
                active.insert(tx.customer_id);
            }
        }

        active.len()
    }

    fn calculate_period_revenue(
        transactions: &[CustomerTransaction],
        cohort_customers: &[Uuid],
        cohort_id: &str,
        period: CohortPeriod,
        period_num: i32,
    ) -> Decimal {
        let cohort_start = Self::parse_cohort_date(cohort_id, period);
        
        let period_start = match period {
            CohortPeriod::Monthly => cohort_start + Duration::days(30 * period_num as i64),
            CohortPeriod::Weekly => cohort_start + Duration::weeks(period_num as i64),
        };

        let period_end = match period {
            CohortPeriod::Monthly => period_start + Duration::days(30),
            CohortPeriod::Weekly => period_start + Duration::weeks(1),
        };

        transactions
            .iter()
            .filter(|tx| {
                cohort_customers.contains(&tx.customer_id)
                    && tx.date >= period_start
                    && tx.date < period_end
            })
            .map(|tx| tx.amount)
            .fold(Decimal::ZERO, |acc, x| acc + x)
    }

    fn parse_cohort_date(cohort_id: &str, period: CohortPeriod) -> NaiveDate {
        match period {
            CohortPeriod::Monthly => {
                let parts: Vec<&str> = cohort_id.split('-').collect();
                let year = parts[0].parse::<i32>().unwrap_or(2024);
                let month = parts[1].parse::<u32>().unwrap_or(1);
                NaiveDate::from_ymd_opt(year, month, 1).unwrap_or_else(|| NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())
            }
            CohortPeriod::Weekly => {
                // Simplified - would use ISO week date parsing
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()
            }
        }
    }

    /// Calculate Customer Lifetime Value for each customer
    pub fn calculate_ltv(
        transactions: &[CustomerTransaction],
        prediction_months: i32,
    ) -> Vec<CustomerLTV> {
        let mut customer_data: HashMap<Uuid, Vec<CustomerTransaction>> = HashMap::new();

        // Group transactions by customer
        for tx in transactions {
            customer_data.entry(tx.customer_id)
                .or_default()
                .push(tx.clone());
        }

        let mut ltv_results = Vec::new();
        let now = transactions.iter().map(|t| t.date).max().unwrap_or_else(|| {
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()
        });

        for (customer_id, txs) in customer_data {
            if txs.is_empty() {
                continue;
            }

            let first_purchase = txs.iter().map(|t| t.date).min().unwrap();
            let last_purchase = txs.iter().map(|t| t.date).max().unwrap();
            let total_orders = txs.len() as i32;
            let total_revenue: Decimal = txs.iter().map(|t| t.amount).sum();
            let aov = total_revenue / Decimal::from(total_orders);

            // Calculate purchase frequency (orders per month)
            let months_active = (last_purchase - first_purchase).num_days() as f64 / 30.0;
            let frequency = if months_active > 0.0 {
                total_orders as f64 / months_active
            } else {
                1.0
            };

            // Predict LTV using simple formula: AOV × Frequency × Prediction Months
            let predicted_ltv = aov * Decimal::from(frequency) * Decimal::from(prediction_months);

            ltv_results.push(CustomerLTV {
                customer_id,
                first_purchase,
                last_purchase,
                total_orders,
                total_revenue,
                average_order_value: aov,
                purchase_frequency: frequency,
                predicted_ltv,
            });
        }

        // Sort by predicted LTV descending
        ltv_results.sort_by(|a, b| b.predicted_ltv.cmp(&a.predicted_ltv));
        ltv_results
    }

    /// Segment customers by value
    pub fn segment_customers(ltv_results: &[CustomerLTV]) -> CustomerSegments {
        let total_customers = ltv_results.len();
        if total_customers == 0 {
            return CustomerSegments::default();
        }

        let vip_threshold = total_customers / 10; // Top 10%
        let loyal_threshold = total_customers / 5; // Top 20%

        let vip: Vec<Uuid> = ltv_results.iter().take(vip_threshold).map(|c| c.customer_id).collect();
        let loyal: Vec<Uuid> = ltv_results.iter().take(loyal_threshold).skip(vip_threshold).map(|c| c.customer_id).collect();
        let at_risk: Vec<Uuid> = ltv_results.iter()
            .filter(|c| c.purchase_frequency < 0.5 && c.total_orders > 1)
            .map(|c| c.customer_id)
            .collect();

        CustomerSegments {
            vip,
            loyal,
            at_risk,
            total: total_customers,
        }
    }

    /// Calculate repeat purchase rate
    pub fn repeat_purchase_rate(transactions: &[CustomerTransaction]) -> f64 {
        let mut customer_orders: HashMap<Uuid, i32> = HashMap::new();
        
        for tx in transactions {
            *customer_orders.entry(tx.customer_id).or_insert(0) += 1;
        }

        let repeat_customers = customer_orders.values().filter(|&&count| count > 1).count();
        let total_customers = customer_orders.len();

        if total_customers > 0 {
            repeat_customers as f64 / total_customers as f64 * 100.0
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CustomerSegments {
    pub vip: Vec<Uuid>,      // Top 10% by LTV
    pub loyal: Vec<Uuid>,    // Top 20% by LTV (excluding VIP)
    pub at_risk: Vec<Uuid>,  // Low frequency but previous customers
    pub total: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_transactions() -> Vec<CustomerTransaction> {
        vec![
            CustomerTransaction {
                customer_id: Uuid::new_v4(),
                transaction_id: Uuid::new_v4(),
                date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                amount: Decimal::from(100),
            },
            CustomerTransaction {
                customer_id: Uuid::new_v4(),
                transaction_id: Uuid::new_v4(),
                date: NaiveDate::from_ymd_opt(2024, 2, 10).unwrap(),
                amount: Decimal::from(150),
            },
        ]
    }

    #[test]
    fn test_cohort_id_from_date() {
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        let cohort = CohortId::from_date(date);
        
        assert_eq!(cohort.year, 2024);
        assert_eq!(cohort.month, 3);
    }

    #[test]
    fn test_cohort_analysis() {
        let transactions = create_test_transactions();
        let analysis = CustomerAnalytics::cohort_analysis(
            &transactions,
            CohortPeriod::Monthly,
            3,
        );

        assert!(!analysis.is_empty());
    }

    #[test]
    fn test_calculate_ltv() {
        let transactions = create_test_transactions();
        let ltv = CustomerAnalytics::calculate_ltv(&transactions, 12);

        assert!(!ltv.is_empty());
        assert!(ltv[0].predicted_ltv > Decimal::ZERO);
    }

    #[test]
    fn test_repeat_purchase_rate() {
        let mut transactions = create_test_transactions();
        let customer_id = transactions[0].customer_id;
        
        // Add another transaction for the same customer
        transactions.push(CustomerTransaction {
            customer_id,
            transaction_id: Uuid::new_v4(),
            date: NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
            amount: Decimal::from(75),
        });

        let rate = CustomerAnalytics::repeat_purchase_rate(&transactions);
        assert!(rate > 0.0);
    }
}

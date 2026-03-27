use std::sync::Arc;
use uuid::Uuid;
use chrono::{Utc, Duration};

use crate::services::transaction_service::TransactionService;
use crate::models::analytics::{AnalyticsSummary, TopProduct, DailySale};

pub struct AnalyticsService {
    transaction_service: Arc<TransactionService>,
}

impl AnalyticsService {
    pub fn new(transaction_service: Arc<TransactionService>) -> Self {
        Self { transaction_service }
    }

    pub async fn get_summary(&self, user_id: Uuid, days: i64) -> anyhow::Result<AnalyticsSummary> {
        let end = Utc::now();
        let start = end - Duration::days(days);
        
        let transactions = self.transaction_service.get_by_date_range(user_id, start, end).await?;
        
        // Calculate metrics
        let total_revenue = transactions.iter()
            .map(|t| t.total)
            .fold(rust_decimal::Decimal::ZERO, |acc, x| acc + x);
        
        let total_transactions = transactions.len() as i64;
        
        let total_items_sold = transactions.iter()
            .map(|t| t.quantity)
            .fold(rust_decimal::Decimal::ZERO, |acc, x| acc + x);
        
        let average_transaction_value = if total_transactions > 0 {
            total_revenue / rust_decimal::Decimal::from(total_transactions)
        } else {
            rust_decimal::Decimal::ZERO
        };

        Ok(AnalyticsSummary {
            total_revenue,
            total_transactions,
            total_items_sold,
            average_transaction_value,
            top_products: vec![], // Would calculate from transactions
            daily_sales: vec![],  // Would group by date
        })
    }
}

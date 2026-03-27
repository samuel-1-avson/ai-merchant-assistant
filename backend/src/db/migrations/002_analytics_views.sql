-- ============================================
-- ANALYTICS MATERIALIZED VIEWS
-- ============================================

-- Daily sales summary (for fast dashboard queries)
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_daily_sales_summary AS
SELECT 
    user_id,
    DATE(created_at) as sale_date,
    COUNT(*) as transaction_count,
    SUM(total) as total_revenue,
    AVG(total) as avg_transaction_value,
    SUM(quantity) as total_items_sold
FROM transactions
GROUP BY user_id, DATE(created_at);

CREATE UNIQUE INDEX IF NOT EXISTS idx_mv_daily_sales ON mv_daily_sales_summary(user_id, sale_date);

-- Top products by revenue
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_top_products AS
SELECT 
    t.user_id,
    t.product_id,
    p.name as product_name,
    COUNT(*) as times_sold,
    SUM(t.quantity) as total_quantity,
    SUM(t.total) as total_revenue
FROM transactions t
JOIN products p ON t.product_id = p.id
GROUP BY t.user_id, t.product_id, p.name;

CREATE UNIQUE INDEX IF NOT EXISTS idx_mv_top_products ON mv_top_products(user_id, product_id);

-- Hourly sales pattern (for identifying peak hours)
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_hourly_patterns AS
SELECT 
    user_id,
    EXTRACT(HOUR FROM created_at) as hour_of_day,
    AVG(total) as avg_sales,
    COUNT(*) as transaction_count
FROM transactions
WHERE created_at >= NOW() - INTERVAL '30 days'
GROUP BY user_id, EXTRACT(HOUR FROM created_at);

CREATE UNIQUE INDEX IF NOT EXISTS idx_mv_hourly_patterns ON mv_hourly_patterns(user_id, hour_of_day);

-- Weekly trends
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_weekly_trends AS
SELECT 
    user_id,
    DATE_TRUNC('week', created_at) as week_start,
    COUNT(*) as transaction_count,
    SUM(total) as total_revenue,
    AVG(total) as avg_transaction_value
FROM transactions
GROUP BY user_id, DATE_TRUNC('week', created_at);

CREATE UNIQUE INDEX IF NOT EXISTS idx_mv_weekly_trends ON mv_weekly_trends(user_id, week_start);

-- Refresh function
CREATE OR REPLACE FUNCTION refresh_analytics_views()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY mv_daily_sales_summary;
    REFRESH MATERIALIZED VIEW CONCURRENTLY mv_top_products;
    REFRESH MATERIALIZED VIEW CONCURRENTLY mv_hourly_patterns;
    REFRESH MATERIALIZED VIEW CONCURRENTLY mv_weekly_trends;
END;
$$ LANGUAGE plpgsql;

-- Auto-refresh trigger (optional - can be called by scheduled job)
-- SELECT cron.schedule('refresh-analytics', '0 * * * *', 'SELECT refresh_analytics_views()');

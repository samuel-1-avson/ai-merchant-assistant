-- AI Merchant Assistant - Supabase Database Schema
-- Run this in Supabase SQL Editor to set up your database

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    full_name VARCHAR(255),
    business_name VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Products table
CREATE TABLE IF NOT EXISTS products (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    sku VARCHAR(100),
    default_price DECIMAL(12, 2),
    cost_price DECIMAL(12, 2),
    unit VARCHAR(50) DEFAULT 'piece',
    stock_quantity INTEGER DEFAULT 0,
    low_stock_threshold INTEGER DEFAULT 10,
    is_active BOOLEAN DEFAULT TRUE,
    image_url TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Transactions table
CREATE TABLE IF NOT EXISTS transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    product_id UUID REFERENCES products(id) ON DELETE SET NULL,
    quantity DECIMAL(10, 2) NOT NULL,
    unit VARCHAR(50) NOT NULL,
    price DECIMAL(12, 2) NOT NULL,
    total DECIMAL(12, 2) NOT NULL,
    notes TEXT,
    voice_recording_url TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Alerts table
CREATE TABLE IF NOT EXISTS alerts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    alert_type VARCHAR(50) NOT NULL,
    severity VARCHAR(20) NOT NULL, -- info, warning, critical
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    data JSONB,
    is_read BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Analytics cache table
CREATE TABLE IF NOT EXISTS analytics_cache (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    cache_key VARCHAR(255) UNIQUE NOT NULL,
    data JSONB NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_transactions_user_id ON transactions(user_id);
CREATE INDEX IF NOT EXISTS idx_transactions_product_id ON transactions(product_id);
CREATE INDEX IF NOT EXISTS idx_transactions_created_at ON transactions(created_at);
CREATE INDEX IF NOT EXISTS idx_products_user_id ON products(user_id);
CREATE INDEX IF NOT EXISTS idx_alerts_user_id ON alerts(user_id);
CREATE INDEX IF NOT EXISTS idx_alerts_unread ON alerts(user_id, is_read) WHERE is_read = FALSE;

-- Enable Row Level Security (RLS)
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE products ENABLE ROW LEVEL SECURITY;
ALTER TABLE transactions ENABLE ROW LEVEL SECURITY;
ALTER TABLE alerts ENABLE ROW LEVEL SECURITY;
ALTER TABLE analytics_cache ENABLE ROW LEVEL SECURITY;

-- RLS Policies
CREATE POLICY "Users can only access their own data" ON users
    FOR ALL USING (auth.uid() = id);

CREATE POLICY "Users can only access their own products" ON products
    FOR ALL USING (auth.uid() = user_id);

CREATE POLICY "Users can only access their own transactions" ON transactions
    FOR ALL USING (auth.uid() = user_id);

CREATE POLICY "Users can only access their own alerts" ON alerts
    FOR ALL USING (auth.uid() = user_id);

-- Functions for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Triggers for updated_at
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_products_updated_at BEFORE UPDATE ON products
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_transactions_updated_at BEFORE UPDATE ON transactions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Analytics Views

-- Daily sales summary
CREATE OR REPLACE VIEW daily_sales AS
SELECT 
    user_id,
    DATE(created_at) as sale_date,
    COUNT(*) as transaction_count,
    SUM(total) as total_revenue,
    SUM(quantity) as items_sold
FROM transactions
GROUP BY user_id, DATE(created_at);

-- Low stock products view
CREATE OR REPLACE VIEW low_stock_products AS
SELECT 
    id,
    user_id,
    name,
    stock_quantity,
    low_stock_threshold,
    (stock_quantity <= low_stock_threshold) as is_low_stock
FROM products
WHERE is_active = TRUE;

-- Top selling products
CREATE OR REPLACE VIEW top_selling_products AS
SELECT 
    t.product_id,
    p.user_id,
    p.name as product_name,
    SUM(t.quantity) as total_quantity,
    SUM(t.total) as total_revenue,
    COUNT(*) as times_sold
FROM transactions t
JOIN products p ON t.product_id = p.id
GROUP BY t.product_id, p.user_id, p.name;

-- Grant permissions
GRANT ALL ON ALL TABLES IN SCHEMA public TO postgres;
GRANT ALL ON ALL SEQUENCES IN SCHEMA public TO postgres;

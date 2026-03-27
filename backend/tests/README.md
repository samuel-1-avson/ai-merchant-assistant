# Backend Tests

This directory contains comprehensive tests for the AI Merchant Assistant backend.

## Test Structure

### Unit Tests

| File | Description | Test Count |
|------|-------------|------------|
| `integration_tests.rs` | Data structure and serialization tests | 5 |
| `ai_tests.rs` | AI agent and intent classification tests | 4 |
| `analytics_tests.rs` | Analytics engine and calculations tests | 6 |
| `alert_tests.rs` | Alert system and notification tests | 7 |
| `model_tests.rs` | Data model validation tests | 12 |

**Total: 34 tests**

## Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test file
cargo test --test integration_tests
cargo test --test ai_tests
cargo test --test analytics_tests
cargo test --test alert_tests
cargo test --test model_tests

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

## Test Categories

### 1. Integration Tests
- Entity extraction data structures
- JSON serialization
- Decimal calculations for financial data

### 2. AI Tests
- Intent classification from strings
- Conversation context management
- Shared state with RwLock

### 3. Analytics Tests
- Trend analysis calculations
- Linear regression (R²)
- Simple moving average
- Z-score anomaly detection
- Period comparison

### 4. Alert Tests
- Alert creation and serialization
- Alert types and severity levels
- Inventory recommendation logic
- Sales drop detection

### 5. Model Tests
- User model validation
- Transaction model validation
- Product model validation
- Analytics model validation
- JSON roundtrip serialization

## Key Test Scenarios

### Intent Classification
Tests verify correct intent detection from various inputs:
- "record_sale", "sale", "SOLD" → RecordSale
- "query_analytics", "analytics" → QueryAnalytics
- "update_inventory", "stock" → UpdateInventory

### Analytics Calculations
- **Linear Regression**: Slope and R² calculation
- **SMA**: Simple moving average for forecasting
- **Z-Score**: Anomaly detection (>2 std dev)
- **Trend Direction**: Increasing/Decreasing/Stable

### Alert Logic
- **Days of Stock**: current_stock / daily_demand
- **Sales Drop**: < -30% change triggers alert
- **Severity**: Info/Warning/Critical

### Financial Calculations
All financial calculations use `rust_decimal::Decimal` for precision:
```rust
let price = Decimal::from(10);
let quantity = Decimal::from(5);
let total = price * quantity; // Exactly 50
```

## Adding New Tests

```rust
#[test]
fn test_new_feature() {
    // Arrange
    let input = ...;
    
    // Act
    let result = function_under_test(input);
    
    // Assert
    assert_eq!(result, expected);
}
```

## Continuous Integration

Tests run automatically on:
- Every push to main
- Every pull request
- Daily scheduled runs

See `.github/workflows/test.yml` for CI configuration.

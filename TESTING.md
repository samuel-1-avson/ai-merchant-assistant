# Testing Guide

## Overview

This document describes how to test the AI Merchant Assistant application.

## Backend Tests

### Prerequisites
```bash
cd backend
```

### Running Unit Tests
```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_intent_from_string

# Run tests by file
cargo test --test ai_tests
cargo test --test analytics_tests
cargo test --test alert_tests
cargo test --test model_tests
```

### Running with Coverage
```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html

# Open report
open tarpaulin-report.html
```

### Test Structure
```
tests/
├── integration_tests.rs    # Data structure tests
├── ai_tests.rs             # AI agent tests
├── analytics_tests.rs      # Analytics engine tests
├── alert_tests.rs          # Alert system tests
├── model_tests.rs          # Data model tests
└── README.md               # Test documentation
```

## Frontend Tests

### Prerequisites
```bash
cd frontend
npm install
```

### Running Tests
```bash
# Run tests
npm test

# Run with coverage
npm test -- --coverage

# Run in CI mode
npm test -- --watchAll=false
```

### Linting
```bash
npm run lint
npm run lint:fix
```

## Integration Tests

### Using Docker Compose
```bash
# Start test environment
docker-compose -f docker-compose.test.yml up -d

# Wait for services
sleep 10

# Run tests
curl http://localhost:3000/health

# Stop environment
docker-compose -f docker-compose.test.yml down
```

### Manual API Testing
```bash
# Health check
curl http://localhost:3000/health

# Register user
curl -X POST http://localhost:3000/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@test.com","password":"password"}'

# Login
curl -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@test.com","password":"password"}'

# Get transactions
curl http://localhost:3000/api/v1/transactions

# Get analytics
curl "http://localhost:3000/api/v1/analytics/summary?days=7"

# Get alerts
curl http://localhost:3000/api/v1/alerts
```

## Test Coverage

### Backend Coverage Areas
- ✅ Data models (User, Transaction, Product, Analytics)
- ✅ AI intent classification
- ✅ Analytics calculations (trends, forecasts, anomalies)
- ✅ Alert logic (inventory, sales drops, trending)
- ✅ JSON serialization/deserialization

### Frontend Coverage Areas
- 🔄 Component rendering
- 🔄 User interactions
- 🔄 API integration
- 🔄 State management
- 🔄 Error handling

## Performance Tests

### Backend Load Testing
```bash
# Install hey (HTTP load generator)
# macOS: brew install hey
# Linux: go install github.com/rakyll/hey@latest

# Test health endpoint
hey -n 1000 -c 50 http://localhost:3000/health

# Test API with concurrency
hey -n 1000 -c 50 -m GET http://localhost:3000/api/v1/transactions
```

## E2E Tests

### Using Playwright (Future)
```bash
# Install Playwright
npm init playwright@latest

# Run E2E tests
npx playwright test
```

## Continuous Integration

Tests run automatically via GitHub Actions:
- On every push to `main` or `develop`
- On every pull request
- Daily at midnight (UTC)

See `.github/workflows/test.yml` for configuration.

## Debugging Tests

### Backend
```bash
# Run with logging
RUST_LOG=debug cargo test -- --nocapture

# Run specific test with backtrace
RUST_BACKTRACE=1 cargo test test_name
```

### Frontend
```bash
# Debug mode
npm test -- --verbose

# With browser
npm test -- --browser
```

## Test Data

### Sample Entities
```json
{
  "product": "Eggs",
  "quantity": 2,
  "unit": "crate",
  "price": 10.00,
  "currency": "USD"
}
```

### Sample Transaction
```json
{
  "product_id": "uuid",
  "quantity": 5,
  "unit": "piece",
  "price": 15.00,
  "notes": "Test transaction"
}
```

## Troubleshooting

### Common Issues

**Issue**: `cargo test` fails with linking errors
**Solution**: `cargo clean && cargo test`

**Issue**: Database connection errors in tests
**Solution**: Ensure PostgreSQL is running and DATABASE_URL is set

**Issue**: Frontend tests timeout
**Solution**: Increase timeout in jest.config.js

## Adding New Tests

### Backend Example
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

### Frontend Example
```typescript
import { render, screen } from '@testing-library/react';
import Component from './Component';

test('renders correctly', () => {
  render(<Component />);
  expect(screen.getByText('Hello')).toBeInTheDocument();
});
```

## Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Backend Test Coverage | >80% | 34 tests |
| Frontend Test Coverage | >70% | TBD |
| E2E Test Coverage | >50% | TBD |
| CI Pass Rate | >95% | N/A |

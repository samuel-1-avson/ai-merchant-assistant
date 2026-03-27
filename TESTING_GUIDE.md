# AI Merchant Assistant - Testing Guide

This guide covers all testing aspects of the AI Merchant Assistant.

---

## Test Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Test Layers                          │
├─────────────────────────────────────────────────────────┤
│  E2E Tests (Playwright)                                 │
│  └── Real browser testing                               │
├─────────────────────────────────────────────────────────┤
│  Integration Tests                                      │
│  └── API endpoint testing                               │
├─────────────────────────────────────────────────────────┤
│  Unit Tests                                             │
│  ├── Backend (Rust)                                     │
│  └── Frontend (Jest)                                    │
└─────────────────────────────────────────────────────────┘
```

---

## Backend Tests

### Running Tests

```bash
cd ai-merchant-assistant/backend

# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test websocket_tests

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

### Test Categories

#### 1. WebSocket Tests (`tests/websocket_tests.rs`)

```rust
// Test message serialization
cargo test test_websocket_message_serialization

// Test JWT extraction
cargo test test_websocket_jwt_extraction

// Test heartbeat
cargo test test_websocket_heartbeat
```

#### 2. Alert Tests (`tests/alert_tests.rs`)

```rust
// Run alert tests
cargo test alert_tests
```

#### 3. Integration Tests (`tests/integration_tests.rs`)

```rust
// Run integration tests
cargo test integration_tests
```

### Writing Backend Tests

```rust
#[test]
fn test_my_feature() {
    // Arrange
    let input = "test";
    
    // Act
    let result = process_input(input);
    
    // Assert
    assert_eq!(result, "expected");
}

#[tokio::test]
async fn test_async_feature() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

---

## Frontend Tests

### Running Tests

```bash
cd ai-merchant-assistant/frontend

# Run all tests
npm test

# Run in watch mode
npm test -- --watch

# Run with coverage
npm test -- --coverage

# Run specific test
npm test -- websocket.test.ts
```

### Test Structure

```
frontend/src/__tests__/
├── api.test.ts          # API client tests
├── websocket.test.ts    # WebSocket client tests
└── utils.test.ts        # Utility function tests
```

### Writing Frontend Tests

```typescript
import { apiClient } from '@/lib/api/client';

describe('ApiClient', () => {
  it('should login successfully', async () => {
    // Mock the API response
    global.fetch = jest.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ success: true, data: { token: 'abc' } }),
    });

    const result = await apiClient.login('test@example.com', 'password');
    
    expect(result.success).toBe(true);
  });
});
```

---

## E2E Tests

### Setup

```bash
cd ai-merchant-assistant/tests

# Install Playwright
npm init -y
npm install @playwright/test
npx playwright install
```

### Running E2E Tests

```bash
cd ai-merchant-assistant/tests

# Run all E2E tests
npx playwright test

# Run specific test
npx playwright test auth.spec.ts

# Run with UI
npx playwright test --ui

# Run in headed mode (see browser)
npx playwright test --headed

# Run specific project
npx playwright test --project=chromium
```

### Test Files

```
tests/e2e/
├── auth.spec.ts       # Authentication flows
└── dashboard.spec.ts  # Dashboard functionality
```

### Writing E2E Tests

```typescript
import { test, expect } from '@playwright/test';

test('should login successfully', async ({ page }) => {
  // Navigate to login
  await page.goto('/login');
  
  // Fill form
  await page.fill('input[type="email"]', 'test@example.com');
  await page.fill('input[type="password"]', 'password');
  
  // Submit
  await page.click('button[type="submit"]');
  
  // Assert
  await expect(page).toHaveURL(/.*dashboard/);
});
```

---

## Manual Testing Checklist

### Authentication

- [ ] Register new user
- [ ] Login with correct credentials
- [ ] Login with incorrect credentials (should fail)
- [ ] Logout
- [ ] Session persistence after refresh
- [ ] Access protected routes without auth (should redirect)

### Transactions

- [ ] Create transaction manually
- [ ] Create transaction via voice
- [ ] View transaction list
- [ ] View transaction details
- [ ] Verify real-time update via WebSocket

### Voice Recording

- [ ] Start recording
- [ ] Stop recording
- [ ] Process audio
- [ ] Handle errors (no speech, unclear)
- [ ] Verify transaction created from voice

### Dashboard

- [ ] View stats cards
- [ ] View recent transactions
- [ ] Refresh data
- [ ] View AI insights
- [ ] Real-time updates indicator

### WebSocket

- [ ] Connection indicator shows "Live"
- [ ] Notification bell shows unread count
- [ ] New transaction triggers notification
- [ ] Reconnection after network loss

---

## Performance Testing

### Backend Load Testing (with k6)

```javascript
// loadtest.js
import http from 'k6/http';
import { check } from 'k6';

export const options = {
  stages: [
    { duration: '1m', target: 100 },
    { duration: '3m', target: 100 },
    { duration: '1m', target: 0 },
  ],
};

export default function () {
  const res = http.get('http://localhost:3000/health');
  check(res, { 'status is 200': (r) => r.status === 200 });
}
```

Run with: `k6 run loadtest.js`

### Frontend Performance

```bash
# Lighthouse CI
npm install -g @lhci/cli
lhci autorun
```

---

## Security Testing

### Authentication

```bash
# Test JWT validation
curl -H "Authorization: Bearer invalid_token" \
  http://localhost:3000/api/v1/transactions

# Should return 401
```

### SQL Injection

```bash
# Test for SQL injection
curl -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "\' OR \'1\'=\'1", "password": "password"}'

# Should not authenticate
```

### Rate Limiting

```bash
# Test rate limits
for i in {1..100}; do
  curl http://localhost:3000/api/v1/health
done

# Should start returning 429 after limit
```

---

## Test Data

### Test Users

```json
{
  "email": "test@example.com",
  "password": "TestPassword123!",
  "full_name": "Test User",
  "business_name": "Test Business"
}
```

### Test Products

```json
{
  "name": "Test Product",
  "description": "A test product",
  "price": 99.99,
  "stock_quantity": 100,
  "category": "Test Category"
}
```

---

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Tests

on: [push, pull_request]

jobs:
  backend-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test
        working-directory: ./backend

  frontend-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: 20
      - run: npm ci
        working-directory: ./frontend
      - run: npm test
        working-directory: ./frontend

  e2e-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: 20
      - run: npm install
        working-directory: ./tests
      - run: npx playwright install
        working-directory: ./tests
      - run: npx playwright test
        working-directory: ./tests
```

---

## Troubleshooting

### Backend Tests Fail

```bash
# Check if database is accessible
psql $DATABASE_URL -c "SELECT 1"

# Run with more output
cargo test -- --nocapture

# Check for compilation errors
cargo check
```

### Frontend Tests Fail

```bash
# Clear Jest cache
npm test -- --clearCache

# Run with verbose output
npm test -- --verbose

# Check for TypeScript errors
npm run type-check
```

### E2E Tests Fail

```bash
# Install browsers
npx playwright install

# Run in debug mode
npx playwright test --debug

# View trace
npx playwright show-trace trace.zip
```

---

## Coverage Requirements

- Backend: > 70% line coverage
- Frontend: > 60% line coverage
- E2E: All critical user flows

---

## Resources

- [Rust Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Jest Documentation](https://jestjs.io/)
- [Playwright Documentation](https://playwright.dev/)

---

*Last Updated: March 27, 2026*

/**
 * E2E Tests - Dashboard
 */

import { test, expect } from '@playwright/test';

const BASE_URL = process.env.FRONTEND_URL || 'http://localhost:3001';

test.describe('Dashboard', () => {
  test.beforeEach(async ({ page }) => {
    // Login first
    await page.goto(`${BASE_URL}/login`);
    await page.fill('input[type="email"]', 'test@example.com');
    await page.fill('input[type="password"]', 'password');
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL(/.*dashboard/, { timeout: 10000 });
  });

  test('should display dashboard with stats', async ({ page }) => {
    await expect(page.locator('h1')).toContainText('Dashboard');
    await expect(page.locator('text=Total Revenue')).toBeVisible();
    await expect(page.locator('text=Total Orders')).toBeVisible();
  });

  test('should display voice recorder', async ({ page }) => {
    await expect(page.locator('text=Quick Sale')).toBeVisible();
    await expect(page.locator('text=Record a new transaction')).toBeVisible();
  });

  test('should display recent transactions', async ({ page }) => {
    await expect(page.locator('text=Recent Transactions')).toBeVisible();
  });

  test('should refresh dashboard data', async ({ page }) => {
    await page.click('text=Refresh');
    await expect(page.locator('.toast-success')).toBeVisible();
  });
});

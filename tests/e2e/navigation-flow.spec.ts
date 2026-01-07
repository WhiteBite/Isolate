import { test, expect } from '@playwright/test';

/**
 * E2E tests for Navigation Flow
 * Tests navigation between main pages: Dashboard → Services → Routing → Proxies
 */

test.describe('Navigation Flow', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Wait for app to be ready
    await expect(page.locator('aside')).toBeVisible({ timeout: 10000 });
  });

  test('complete navigation flow: Dashboard → Services → Routing → Proxies', async ({ page }) => {
    // Step 1: Start at Dashboard
    await expect(page).toHaveURL('/');
    const dashboardBreadcrumb = page.locator('header span:has-text("Dashboard")');
    await expect(dashboardBreadcrumb).toBeVisible();

    // Step 2: Navigate to Services
    await page.click('a:has-text("Services")');
    await expect(page).toHaveURL('/services');
    const servicesBreadcrumb = page.locator('header span:has-text("Services")');
    await expect(servicesBreadcrumb).toBeVisible();

    // Step 3: Navigate to Routing
    await page.click('a:has-text("Routing")');
    await expect(page).toHaveURL('/routing');
    const routingBreadcrumb = page.locator('header span:has-text("Routing")');
    await expect(routingBreadcrumb).toBeVisible();

    // Step 4: Navigate to Proxies
    await page.click('a:has-text("Proxies")');
    await expect(page).toHaveURL('/proxies');
    const proxiesBreadcrumb = page.locator('header span:has-text("Proxies")');
    await expect(proxiesBreadcrumb).toBeVisible();
  });

  test('can navigate back to Dashboard from any page', async ({ page }) => {
    // Navigate to Proxies first
    await page.click('a:has-text("Proxies")');
    await expect(page).toHaveURL('/proxies');

    // Navigate back to Dashboard
    await page.click('a:has-text("Dashboard")');
    await expect(page).toHaveURL('/');
    await expect(page.locator('header span:has-text("Dashboard")')).toBeVisible();
  });

  test('sidebar navigation items are all visible', async ({ page }) => {
    const sidebar = page.locator('aside');
    await expect(sidebar).toBeVisible();

    // Check all main navigation items
    const navItems = [
      'Dashboard',
      'Services',
      'Routing',
      'Proxies',
      'Strategies',
      'Settings',
      'Logs'
    ];

    for (const item of navItems) {
      const navLink = sidebar.locator(`a:has-text("${item}")`);
      await expect(navLink).toBeVisible();
    }
  });

  test('active navigation item is highlighted correctly', async ({ page }) => {
    // Dashboard should be active initially
    const dashboardLink = page.locator('aside a:has-text("Dashboard")');
    await expect(dashboardLink).toHaveClass(/bg-/);

    // Navigate to Services
    await page.click('a:has-text("Services")');
    await expect(page).toHaveURL('/services');

    // Services link should now be active
    const servicesLink = page.locator('aside a:has-text("Services")');
    await expect(servicesLink).toHaveClass(/bg-/);
  });

  test('navigation preserves app state (header status)', async ({ page }) => {
    // Check header status indicator exists
    const statusIndicator = page.locator('header .rounded-full');
    await expect(statusIndicator.first()).toBeVisible();

    // Navigate through pages and verify status indicator persists
    await page.click('a:has-text("Services")');
    await expect(statusIndicator.first()).toBeVisible();

    await page.click('a:has-text("Routing")');
    await expect(statusIndicator.first()).toBeVisible();

    await page.click('a:has-text("Proxies")');
    await expect(statusIndicator.first()).toBeVisible();
  });

  test('can navigate to Settings page', async ({ page }) => {
    await page.click('a:has-text("Settings")');
    await expect(page).toHaveURL('/settings');
    await expect(page.locator('header span:has-text("Settings")')).toBeVisible();
  });

  test('can navigate to Diagnostics page', async ({ page }) => {
    await page.click('a:has-text("Diagnostics")');
    await expect(page).toHaveURL('/diagnostics');
    await expect(page.locator('header span:has-text("Diagnostics")')).toBeVisible();
  });
});

test.describe('Navigation via Direct URL', () => {
  test('can access Services page directly', async ({ page }) => {
    await page.goto('/services');
    await expect(page.locator('aside')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('header span:has-text("Services")')).toBeVisible();
  });

  test('can access Routing page directly', async ({ page }) => {
    await page.goto('/routing');
    await expect(page.locator('aside')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('header span:has-text("Routing")')).toBeVisible();
  });

  test('can access Proxies page directly', async ({ page }) => {
    await page.goto('/proxies');
    await expect(page.locator('aside')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('header span:has-text("Proxies")')).toBeVisible();
  });

  test('can access Settings page directly', async ({ page }) => {
    await page.goto('/settings');
    await expect(page.locator('aside')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('header span:has-text("Settings")')).toBeVisible();
  });
});

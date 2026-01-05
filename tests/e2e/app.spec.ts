import { test, expect } from '@playwright/test';

/**
 * Basic application tests for Isolate
 * Tests core functionality: startup, navigation, and basic UI elements
 */

test.describe('Application Startup', () => {
  test('application loads successfully', async ({ page }) => {
    await page.goto('/');
    
    // Wait for the app to be ready
    await expect(page.locator('body')).toBeVisible();
    
    // Check that the main layout is rendered
    await expect(page.locator('aside')).toBeVisible({ timeout: 10000 });
  });

  test('dashboard page loads with correct title', async ({ page }) => {
    await page.goto('/');
    
    // Check for Dashboard heading
    const heading = page.locator('h1:has-text("Dashboard")');
    await expect(heading).toBeVisible({ timeout: 10000 });
  });

  test('sidebar navigation is visible', async ({ page }) => {
    await page.goto('/');
    
    // Check sidebar elements
    const sidebar = page.locator('aside');
    await expect(sidebar).toBeVisible();
    
    // Check logo
    const logo = sidebar.locator('h1:has-text("Isolate")');
    await expect(logo).toBeVisible();
    
    // Check navigation items exist
    const navItems = [
      'Dashboard',
      'Proxies',
      'Routing',
      'Strategies',
      'Orchestra',
      'Testing',
      'Settings',
      'Logs'
    ];
    
    for (const item of navItems) {
      const navLink = sidebar.locator(`a:has-text("${item}")`);
      await expect(navLink).toBeVisible();
    }
  });

  test('toolbar is visible with controls', async ({ page }) => {
    await page.goto('/');
    
    // Check toolbar exists
    const toolbar = page.locator('.h-14.bg-\\[\\#1a1f3a\\]');
    await expect(toolbar).toBeVisible();
    
    // Check for mode toggles
    await expect(page.getByText('System Proxy')).toBeVisible();
    await expect(page.getByText('TUN Mode')).toBeVisible();
    await expect(page.getByText('QUIC Block')).toBeVisible();
    
    // Check for quick action buttons
    await expect(page.getByRole('button', { name: /Turbo/i })).toBeVisible();
    await expect(page.getByRole('button', { name: /Panic/i })).toBeVisible();
  });
});

test.describe('Navigation', () => {
  test('can navigate to Strategies page', async ({ page }) => {
    await page.goto('/');
    
    // Click on Strategies link
    await page.click('a:has-text("Strategies")');
    
    // Verify we're on the strategies page
    await expect(page).toHaveURL('/strategies');
    await expect(page.locator('h1:has-text("Стратегии")')).toBeVisible();
  });

  test('can navigate to Proxies page', async ({ page }) => {
    await page.goto('/');
    
    await page.click('a:has-text("Proxies")');
    
    await expect(page).toHaveURL('/proxies');
  });

  test('can navigate to Settings page', async ({ page }) => {
    await page.goto('/');
    
    await page.click('a:has-text("Settings")');
    
    await expect(page).toHaveURL('/settings');
  });

  test('can navigate to Logs page', async ({ page }) => {
    await page.goto('/');
    
    await page.click('a:has-text("Logs")');
    
    await expect(page).toHaveURL('/logs');
  });

  test('can navigate to Testing page', async ({ page }) => {
    await page.goto('/');
    
    await page.click('a:has-text("Testing")');
    
    await expect(page).toHaveURL('/testing');
  });

  test('can navigate back to Dashboard', async ({ page }) => {
    await page.goto('/strategies');
    
    await page.click('a:has-text("Dashboard")');
    
    await expect(page).toHaveURL('/');
    await expect(page.locator('h1:has-text("Dashboard")')).toBeVisible();
  });

  test('active navigation item is highlighted', async ({ page }) => {
    await page.goto('/');
    
    // Dashboard should be active
    const dashboardLink = page.locator('a:has-text("Dashboard")');
    await expect(dashboardLink).toHaveClass(/bg-\[#2a2f4a\]/);
    
    // Navigate to Strategies
    await page.click('a:has-text("Strategies")');
    
    // Strategies should now be active
    const strategiesLink = page.locator('a:has-text("Strategies")');
    await expect(strategiesLink).toHaveClass(/bg-\[#2a2f4a\]/);
  });
});

test.describe('Dashboard UI Elements', () => {
  test('status card is visible', async ({ page }) => {
    await page.goto('/');
    
    // Main status card should be visible
    const statusCard = page.locator('.bg-\\[\\#1a1f3a\\].rounded-2xl').first();
    await expect(statusCard).toBeVisible();
  });

  test('quick action buttons are visible', async ({ page }) => {
    await page.goto('/');
    
    // Check for Turbo button
    const turboButton = page.locator('button:has-text("Turbo")').first();
    await expect(turboButton).toBeVisible();
    
    // Check for Deep button
    const deepButton = page.locator('button:has-text("Deep")').first();
    await expect(deepButton).toBeVisible();
  });

  test('quick links are visible', async ({ page }) => {
    await page.goto('/');
    
    // Check for "Добавить прокси" link
    const addProxyLink = page.locator('a:has-text("Добавить прокси")');
    await expect(addProxyLink).toBeVisible();
    
    // Check for "Настройки" link
    const settingsLink = page.locator('a:has-text("Настройки")');
    await expect(settingsLink).toBeVisible();
  });

  test('system status bar is visible', async ({ page }) => {
    await page.goto('/');
    
    // Check for system status section
    const systemStatus = page.locator('text=Системный статус');
    await expect(systemStatus).toBeVisible();
  });
});

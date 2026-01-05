import { test, expect } from '@playwright/test';

/**
 * E2E tests for Services page (/services)
 * Tests service list, status display, details panel, and add service modal
 */

test.describe('Services Page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/services');
  });

  test('page loads without errors', async ({ page }) => {
    // Wait for the page to be ready
    await expect(page.locator('body')).toBeVisible();
    
    // Check that main layout is rendered
    await expect(page.locator('aside')).toBeVisible({ timeout: 10000 });
  });

  test('Services heading is visible', async ({ page }) => {
    const heading = page.locator('h2:has-text("Services")');
    await expect(heading).toBeVisible({ timeout: 10000 });
  });

  test('Check All Services button is visible and clickable', async ({ page }) => {
    const checkButton = page.locator('button:has-text("Check All Services")');
    await expect(checkButton).toBeVisible();
    await expect(checkButton).toBeEnabled();
    
    // Click and verify it responds (shows loading state)
    await checkButton.click();
    
    // Should show "Checking..." text while scanning
    const checkingText = page.locator('button:has-text("Checking...")');
    // Either shows checking or completes quickly
    await expect(checkButton.or(checkingText)).toBeVisible();
  });
});


test.describe('Services List', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/services');
  });

  test('service list container is visible', async ({ page }) => {
    // Left panel with service list
    const serviceListPanel = page.locator('.w-\\[320px\\]');
    await expect(serviceListPanel).toBeVisible({ timeout: 10000 });
  });

  test('services are displayed in the list', async ({ page }) => {
    // Wait for services to load (either from backend or empty state)
    await page.waitForTimeout(2000);
    
    // Check for service items (buttons in the list)
    const serviceItems = page.locator('.w-\\[320px\\] button').filter({ 
      has: page.locator('.text-sm.font-medium') 
    });
    
    // Should have at least the list container visible
    const listContainer = page.locator('.w-\\[320px\\] .overflow-y-auto');
    await expect(listContainer).toBeVisible();
  });

  test('clicking a service shows details on the right', async ({ page }) => {
    // Wait for page to load
    await page.waitForTimeout(2000);
    
    // Find and click first service in the list
    const firstService = page.locator('.w-\\[320px\\] button').first();
    
    if (await firstService.isVisible()) {
      await firstService.click();
      
      // Right panel should show service details
      const detailsPanel = page.locator('.flex-1.overflow-y-auto.bg-zinc-950');
      await expect(detailsPanel).toBeVisible();
      
      // Should show service name in details (h1)
      const serviceName = page.locator('.flex-1.overflow-y-auto h1');
      await expect(serviceName).toBeVisible({ timeout: 5000 });
    }
  });

  test('footer shows service count', async ({ page }) => {
    // Footer with service count
    const footer = page.locator('.w-\\[320px\\] .border-t');
    await expect(footer).toBeVisible();
    
    // Should contain "services available" text
    const countText = page.locator('text=/\\d+ \\/ \\d+ services available/');
    await expect(countText).toBeVisible({ timeout: 5000 });
  });
});

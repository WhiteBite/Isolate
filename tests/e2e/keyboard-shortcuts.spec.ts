import { test, expect } from '@playwright/test';

/**
 * E2E tests for Keyboard Shortcuts
 * Tests main shortcuts: Ctrl+1-4 navigation, Ctrl+S, ?, F1
 */

test.describe('Keyboard Shortcuts - Navigation', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Wait for app to be ready
    await expect(page.locator('aside')).toBeVisible({ timeout: 10000 });
  });

  test('Ctrl+1 navigates to Dashboard', async ({ page }) => {
    // First navigate away from Dashboard
    await page.goto('/settings');
    await expect(page).toHaveURL('/settings');

    // Press Ctrl+1
    await page.keyboard.press('Control+1');

    // Should navigate to Dashboard
    await expect(page).toHaveURL('/');
  });

  test('Ctrl+2 navigates to Services', async ({ page }) => {
    // Press Ctrl+2
    await page.keyboard.press('Control+2');

    // Should navigate to Services
    await expect(page).toHaveURL('/services');
  });

  test('Ctrl+3 navigates to Routing', async ({ page }) => {
    // Press Ctrl+3
    await page.keyboard.press('Control+3');

    // Should navigate to Routing
    await expect(page).toHaveURL('/routing');
  });

  test('Ctrl+4 navigates to Proxies', async ({ page }) => {
    // Press Ctrl+4
    await page.keyboard.press('Control+4');

    // Should navigate to Proxies
    await expect(page).toHaveURL('/proxies');
  });

  test('Ctrl+, navigates to Settings', async ({ page }) => {
    // Press Ctrl+,
    await page.keyboard.press('Control+,');

    // Should navigate to Settings
    await expect(page).toHaveURL('/settings');
  });

  test('Ctrl+M navigates to Marketplace', async ({ page }) => {
    // Press Ctrl+M
    await page.keyboard.press('Control+m');

    // Should navigate to Marketplace
    await expect(page).toHaveURL('/marketplace');
  });
});

test.describe('Keyboard Shortcuts - Help Modal', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('aside')).toBeVisible({ timeout: 10000 });
  });

  test('? opens keyboard shortcuts help modal', async ({ page }) => {
    // Press ? key
    await page.keyboard.press('?');

    // Help modal should be visible
    const modal = page.locator('[role="dialog"]').filter({ hasText: /shortcuts|горячие/i });
    await expect(modal).toBeVisible({ timeout: 5000 });
  });

  test('F1 opens keyboard shortcuts help modal', async ({ page }) => {
    // Press F1 key
    await page.keyboard.press('F1');

    // Help modal should be visible
    const modal = page.locator('[role="dialog"]').filter({ hasText: /shortcuts|горячие/i });
    await expect(modal).toBeVisible({ timeout: 5000 });
  });

  test('help modal can be closed with Escape', async ({ page }) => {
    // Open help modal
    await page.keyboard.press('?');
    
    const modal = page.locator('[role="dialog"]').filter({ hasText: /shortcuts|горячие/i });
    await expect(modal).toBeVisible();

    // Press Escape to close
    await page.keyboard.press('Escape');

    // Modal should be hidden
    await expect(modal).not.toBeVisible();
  });
});

test.describe('Keyboard Shortcuts - Input Focus Handling', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('aside')).toBeVisible({ timeout: 10000 });
  });

  test('shortcuts are ignored when input is focused', async ({ page }) => {
    // Open command palette to get an input
    await page.keyboard.press('Control+k');
    const palette = page.locator('[role="dialog"][aria-label="Command Palette"]');
    await expect(palette).toBeVisible();

    // Input should be focused
    const searchInput = palette.locator('input');
    await expect(searchInput).toBeFocused();

    // Type "1" - should go into input, not trigger Ctrl+1 navigation
    await searchInput.fill('1');
    
    // Should still be on same page (command palette open)
    await expect(palette).toBeVisible();
    await expect(searchInput).toHaveValue('1');
  });

  test('Ctrl+K works even when other shortcuts are blocked in input', async ({ page }) => {
    // Open command palette
    await page.keyboard.press('Control+k');
    const palette = page.locator('[role="dialog"][aria-label="Command Palette"]');
    await expect(palette).toBeVisible();

    // Ctrl+K should still close it
    await page.keyboard.press('Control+k');
    await expect(palette).not.toBeVisible();
  });
});

test.describe('Keyboard Shortcuts - Sequential Navigation', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('aside')).toBeVisible({ timeout: 10000 });
  });

  test('can navigate through all pages using shortcuts', async ({ page }) => {
    // Start at Dashboard
    await expect(page).toHaveURL('/');

    // Ctrl+2 → Services
    await page.keyboard.press('Control+2');
    await expect(page).toHaveURL('/services');

    // Ctrl+3 → Routing
    await page.keyboard.press('Control+3');
    await expect(page).toHaveURL('/routing');

    // Ctrl+4 → Proxies
    await page.keyboard.press('Control+4');
    await expect(page).toHaveURL('/proxies');

    // Ctrl+1 → Back to Dashboard
    await page.keyboard.press('Control+1');
    await expect(page).toHaveURL('/');
  });

  test('shortcuts work from any page', async ({ page }) => {
    // Start at Settings
    await page.goto('/settings');
    await expect(page).toHaveURL('/settings');

    // Ctrl+3 should still work
    await page.keyboard.press('Control+3');
    await expect(page).toHaveURL('/routing');

    // Ctrl+, should go back to Settings
    await page.keyboard.press('Control+,');
    await expect(page).toHaveURL('/settings');
  });
});

test.describe('Keyboard Shortcuts - Command Palette Integration', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('aside')).toBeVisible({ timeout: 10000 });
  });

  test('Ctrl+K opens command palette', async ({ page }) => {
    await page.keyboard.press('Control+k');
    
    const palette = page.locator('[role="dialog"][aria-label="Command Palette"]');
    await expect(palette).toBeVisible();
  });

  test('Ctrl+Shift+P also opens command palette', async ({ page }) => {
    await page.keyboard.press('Control+Shift+p');
    
    const palette = page.locator('[role="dialog"][aria-label="Command Palette"]');
    await expect(palette).toBeVisible();
  });
});

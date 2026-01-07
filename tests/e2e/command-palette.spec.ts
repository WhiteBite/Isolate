import { test, expect } from '@playwright/test';

/**
 * E2E tests for Command Palette
 * Tests opening, searching, and executing commands via Ctrl+K
 */

test.describe('Command Palette', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Wait for app to be ready
    await expect(page.locator('aside')).toBeVisible({ timeout: 10000 });
  });

  test('opens with Ctrl+K', async ({ page }) => {
    // Press Ctrl+K to open command palette
    await page.keyboard.press('Control+k');

    // Command palette should be visible
    const palette = page.locator('[role="dialog"][aria-label="Command Palette"]');
    await expect(palette).toBeVisible({ timeout: 5000 });

    // Search input should be visible and focused
    const searchInput = palette.locator('input[placeholder*="command"]');
    await expect(searchInput).toBeVisible();
    await expect(searchInput).toBeFocused();
  });

  test('closes with Escape', async ({ page }) => {
    // Open command palette
    await page.keyboard.press('Control+k');
    const palette = page.locator('[role="dialog"][aria-label="Command Palette"]');
    await expect(palette).toBeVisible();

    // Press Escape to close
    await page.keyboard.press('Escape');

    // Palette should be hidden
    await expect(palette).not.toBeVisible();
  });

  test('closes with Ctrl+K when already open', async ({ page }) => {
    // Open command palette
    await page.keyboard.press('Control+k');
    const palette = page.locator('[role="dialog"][aria-label="Command Palette"]');
    await expect(palette).toBeVisible();

    // Press Ctrl+K again to close
    await page.keyboard.press('Control+k');

    // Palette should be hidden
    await expect(palette).not.toBeVisible();
  });

  test('closes when clicking backdrop', async ({ page }) => {
    // Open command palette
    await page.keyboard.press('Control+k');
    const palette = page.locator('[role="dialog"][aria-label="Command Palette"]');
    await expect(palette).toBeVisible();

    // Click on backdrop (outside the dialog)
    await page.locator('.fixed.inset-0').click({ position: { x: 10, y: 10 } });

    // Palette should be hidden
    await expect(palette).not.toBeVisible();
  });

  test('shows all command categories', async ({ page }) => {
    // Open command palette
    await page.keyboard.press('Control+k');
    const palette = page.locator('[role="dialog"][aria-label="Command Palette"]');
    await expect(palette).toBeVisible();

    // Check for category headers
    await expect(palette.locator('text=Navigation')).toBeVisible();
    await expect(palette.locator('text=Actions')).toBeVisible();
    await expect(palette.locator('text=Settings')).toBeVisible();
  });

  test('shows navigation commands', async ({ page }) => {
    // Open command palette
    await page.keyboard.press('Control+k');
    const palette = page.locator('[role="dialog"][aria-label="Command Palette"]');
    await expect(palette).toBeVisible();

    // Check for navigation commands
    await expect(palette.locator('text=Go to Dashboard')).toBeVisible();
    await expect(palette.locator('text=Go to Diagnostics')).toBeVisible();
    await expect(palette.locator('text=Go to Proxies')).toBeVisible();
    await expect(palette.locator('text=Go to Settings')).toBeVisible();
  });

  test('shows action commands', async ({ page }) => {
    // Open command palette
    await page.keyboard.press('Control+k');
    const palette = page.locator('[role="dialog"][aria-label="Command Palette"]');
    await expect(palette).toBeVisible();

    // Check for action commands
    await expect(palette.locator('text=Start Protection')).toBeVisible();
    await expect(palette.locator('text=Stop Protection')).toBeVisible();
    await expect(palette.locator('text=Add Proxy')).toBeVisible();
    await expect(palette.locator('text=Test Connection')).toBeVisible();
  });

  test('filters commands by search query', async ({ page }) => {
    // Open command palette
    await page.keyboard.press('Control+k');
    const palette = page.locator('[role="dialog"][aria-label="Command Palette"]');
    await expect(palette).toBeVisible();

    // Type search query
    const searchInput = palette.locator('input[placeholder*="command"]');
    await searchInput.fill('dashboard');

    // Only Dashboard command should be visible
    await expect(palette.locator('text=Go to Dashboard')).toBeVisible();
    
    // Other navigation commands should be filtered out
    await expect(palette.locator('text=Go to Proxies')).not.toBeVisible();
  });

  test('shows "No commands found" for invalid search', async ({ page }) => {
    // Open command palette
    await page.keyboard.press('Control+k');
    const palette = page.locator('[role="dialog"][aria-label="Command Palette"]');
    await expect(palette).toBeVisible();

    // Type invalid search query
    const searchInput = palette.locator('input[placeholder*="command"]');
    await searchInput.fill('xyznonexistent');

    // Should show no results message
    await expect(palette.locator('text=No commands found')).toBeVisible();
  });

  test('navigates commands with arrow keys', async ({ page }) => {
    // Open command palette
    await page.keyboard.press('Control+k');
    const palette = page.locator('[role="dialog"][aria-label="Command Palette"]');
    await expect(palette).toBeVisible();

    // First item should be selected by default
    const firstCommand = palette.locator('[data-command-index="0"]');
    await expect(firstCommand).toHaveClass(/bg-\[#00d4ff\]/);

    // Press down arrow
    await page.keyboard.press('ArrowDown');

    // Second item should now be selected
    const secondCommand = palette.locator('[data-command-index="1"]');
    await expect(secondCommand).toHaveClass(/bg-\[#00d4ff\]/);

    // Press up arrow
    await page.keyboard.press('ArrowUp');

    // First item should be selected again
    await expect(firstCommand).toHaveClass(/bg-\[#00d4ff\]/);
  });

  test('executes command with Enter key', async ({ page }) => {
    // Open command palette
    await page.keyboard.press('Control+k');
    const palette = page.locator('[role="dialog"][aria-label="Command Palette"]');
    await expect(palette).toBeVisible();

    // Search for Settings command
    const searchInput = palette.locator('input[placeholder*="command"]');
    await searchInput.fill('settings');

    // Press Enter to execute
    await page.keyboard.press('Enter');

    // Should navigate to settings page
    await expect(page).toHaveURL('/settings');
    
    // Palette should be closed
    await expect(palette).not.toBeVisible();
  });

  test('executes command by clicking', async ({ page }) => {
    // Open command palette
    await page.keyboard.press('Control+k');
    const palette = page.locator('[role="dialog"][aria-label="Command Palette"]');
    await expect(palette).toBeVisible();

    // Click on "Go to Proxies" command
    await palette.locator('button:has-text("Go to Proxies")').click();

    // Should navigate to proxies page
    await expect(page).toHaveURL('/proxies');
    
    // Palette should be closed
    await expect(palette).not.toBeVisible();
  });

  test('shows keyboard shortcuts in footer', async ({ page }) => {
    // Open command palette
    await page.keyboard.press('Control+k');
    const palette = page.locator('[role="dialog"][aria-label="Command Palette"]');
    await expect(palette).toBeVisible();

    // Check footer hints
    await expect(palette.locator('text=to navigate')).toBeVisible();
    await expect(palette.locator('text=to select')).toBeVisible();
    await expect(palette.locator('text=to close')).toBeVisible();
  });

  test('shows ESC hint in search bar', async ({ page }) => {
    // Open command palette
    await page.keyboard.press('Control+k');
    const palette = page.locator('[role="dialog"][aria-label="Command Palette"]');
    await expect(palette).toBeVisible();

    // Check ESC hint
    await expect(palette.locator('kbd:has-text("ESC")')).toBeVisible();
  });

  test('can open via header search button', async ({ page }) => {
    // Click on search button in header
    const searchButton = page.locator('header button:has-text("Search")');
    await searchButton.click();

    // Command palette should be visible
    const palette = page.locator('[role="dialog"][aria-label="Command Palette"]');
    await expect(palette).toBeVisible({ timeout: 5000 });
  });

  test('fuzzy search works correctly', async ({ page }) => {
    // Open command palette
    await page.keyboard.press('Control+k');
    const palette = page.locator('[role="dialog"][aria-label="Command Palette"]');
    await expect(palette).toBeVisible();

    // Type partial/fuzzy query
    const searchInput = palette.locator('input[placeholder*="command"]');
    await searchInput.fill('prox');

    // Should find "Go to Proxies" and "Add Proxy"
    await expect(palette.locator('text=Go to Proxies')).toBeVisible();
    await expect(palette.locator('text=Add Proxy')).toBeVisible();
  });

  test('resets selection when search query changes', async ({ page }) => {
    // Open command palette
    await page.keyboard.press('Control+k');
    const palette = page.locator('[role="dialog"][aria-label="Command Palette"]');
    await expect(palette).toBeVisible();

    // Navigate down a few times
    await page.keyboard.press('ArrowDown');
    await page.keyboard.press('ArrowDown');

    // Type search query - selection should reset to first item
    const searchInput = palette.locator('input[placeholder*="command"]');
    await searchInput.fill('dash');

    // First filtered item should be selected
    const firstCommand = palette.locator('[data-command-index="0"]');
    await expect(firstCommand).toHaveClass(/bg-\[#00d4ff\]/);
  });
});

test.describe('Command Palette - Alternative Opening', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('aside')).toBeVisible({ timeout: 10000 });
  });

  test('opens with Ctrl+Shift+P', async ({ page }) => {
    // Press Ctrl+Shift+P to open command palette
    await page.keyboard.press('Control+Shift+p');

    // Command palette should be visible
    const palette = page.locator('[role="dialog"][aria-label="Command Palette"]');
    await expect(palette).toBeVisible({ timeout: 5000 });
  });
});

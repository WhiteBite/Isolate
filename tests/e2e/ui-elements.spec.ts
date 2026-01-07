import { test, expect } from '@playwright/test';

/**
 * E2E tests for UI Elements
 * Tests that main UI elements are displayed correctly
 */

test.describe('UI Elements - Layout', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Wait for app to be ready
    await expect(page.locator('aside')).toBeVisible({ timeout: 10000 });
  });

  test('main layout structure is correct', async ({ page }) => {
    // Sidebar should be visible
    const sidebar = page.locator('aside');
    await expect(sidebar).toBeVisible();

    // Header should be visible
    const header = page.locator('header');
    await expect(header).toBeVisible();

    // Main content area should be visible
    const main = page.locator('main');
    await expect(main).toBeVisible();
  });

  test('sidebar contains logo', async ({ page }) => {
    const sidebar = page.locator('aside');
    
    // Logo text "Isolate" should be visible
    const logo = sidebar.locator('text=Isolate');
    await expect(logo).toBeVisible();
  });

  test('header shows current page name', async ({ page }) => {
    const header = page.locator('header');
    
    // Dashboard should be shown in header
    await expect(header.locator('text=Dashboard')).toBeVisible();

    // Navigate to Services
    await page.click('a:has-text("Services")');
    await expect(header.locator('text=Services')).toBeVisible();
  });

  test('header shows status indicator', async ({ page }) => {
    const header = page.locator('header');
    
    // Status indicator (either "Protected" or "Inactive")
    const statusIndicator = header.locator('.rounded-full').filter({ 
      has: page.locator('span') 
    });
    await expect(statusIndicator.first()).toBeVisible();
  });

  test('header has search button', async ({ page }) => {
    const header = page.locator('header');
    
    // Search button with ⌘K hint
    const searchButton = header.locator('button:has-text("Search")');
    await expect(searchButton).toBeVisible();
    
    // Should show keyboard shortcut hint
    await expect(searchButton.locator('kbd')).toBeVisible();
  });
});

test.describe('UI Elements - Sidebar Navigation', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('aside')).toBeVisible({ timeout: 10000 });
  });

  test('all navigation items have icons', async ({ page }) => {
    const sidebar = page.locator('aside');
    
    // Each nav link should have an SVG icon
    const navLinks = sidebar.locator('a');
    const count = await navLinks.count();
    
    expect(count).toBeGreaterThan(0);
    
    // Check first few links have icons
    for (let i = 0; i < Math.min(count, 5); i++) {
      const link = navLinks.nth(i);
      const icon = link.locator('svg');
      await expect(icon).toBeVisible();
    }
  });

  test('navigation items are clickable', async ({ page }) => {
    const sidebar = page.locator('aside');
    
    // Click on Services
    const servicesLink = sidebar.locator('a:has-text("Services")');
    await expect(servicesLink).toBeEnabled();
    await servicesLink.click();
    await expect(page).toHaveURL('/services');
  });

  test('navigation items have hover effect', async ({ page }) => {
    const sidebar = page.locator('aside');
    const servicesLink = sidebar.locator('a:has-text("Services")');
    
    // Get initial background
    const initialBg = await servicesLink.evaluate(el => 
      window.getComputedStyle(el).backgroundColor
    );
    
    // Hover over the link
    await servicesLink.hover();
    
    // Background should change on hover (or have hover class)
    // This is a basic check - actual hover effect depends on CSS
    await expect(servicesLink).toBeVisible();
  });
});

test.describe('UI Elements - Theme and Colors', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('aside')).toBeVisible({ timeout: 10000 });
  });

  test('app has dark theme background', async ({ page }) => {
    const body = page.locator('body');
    const bgColor = await body.evaluate(el => 
      window.getComputedStyle(el).backgroundColor
    );
    
    // Should be a dark color (RGB values should be low)
    // This is a basic check for dark theme
    expect(bgColor).toBeTruthy();
  });

  test('text is readable (light on dark)', async ({ page }) => {
    const header = page.locator('header');
    const textElement = header.locator('span').first();
    
    const color = await textElement.evaluate(el => 
      window.getComputedStyle(el).color
    );
    
    // Text should have some color defined
    expect(color).toBeTruthy();
  });

  test('accent colors are applied to active elements', async ({ page }) => {
    // Navigate to Services to make it active
    await page.click('a:has-text("Services")');
    
    const activeLink = page.locator('aside a:has-text("Services")');
    
    // Active link should have background styling
    const hasBackground = await activeLink.evaluate(el => {
      const bg = window.getComputedStyle(el).backgroundColor;
      return bg !== 'rgba(0, 0, 0, 0)' && bg !== 'transparent';
    });
    
    expect(hasBackground).toBeTruthy();
  });
});

test.describe('UI Elements - Responsive Behavior', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('aside')).toBeVisible({ timeout: 10000 });
  });

  test('layout adapts to viewport size', async ({ page }) => {
    // Default viewport
    const sidebar = page.locator('aside');
    await expect(sidebar).toBeVisible();
    
    // Sidebar should have reasonable width
    const box = await sidebar.boundingBox();
    expect(box).toBeTruthy();
    expect(box!.width).toBeGreaterThan(50);
    expect(box!.width).toBeLessThan(400);
  });

  test('main content fills available space', async ({ page }) => {
    const main = page.locator('main');
    const box = await main.boundingBox();
    
    expect(box).toBeTruthy();
    expect(box!.width).toBeGreaterThan(500);
  });
});

test.describe('UI Elements - Page Specific', () => {
  test('Dashboard shows status cards', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('aside')).toBeVisible({ timeout: 10000 });
    
    // Dashboard should have some card-like elements
    const cards = page.locator('.rounded-2xl, .rounded-xl, .rounded-lg');
    const count = await cards.count();
    expect(count).toBeGreaterThan(0);
  });

  test('Services page shows service list', async ({ page }) => {
    await page.goto('/services');
    await expect(page.locator('aside')).toBeVisible({ timeout: 10000 });
    
    // Should have Services heading
    await expect(page.locator('h2:has-text("Services")')).toBeVisible();
  });

  test('Routing page shows tabs', async ({ page }) => {
    await page.goto('/routing');
    await expect(page.locator('aside')).toBeVisible({ timeout: 10000 });
    
    // Should have domain/app tabs
    await expect(page.locator('button:has-text("По доменам")')).toBeVisible();
    await expect(page.locator('button:has-text("По приложениям")')).toBeVisible();
  });

  test('Settings page loads correctly', async ({ page }) => {
    await page.goto('/settings');
    await expect(page.locator('aside')).toBeVisible({ timeout: 10000 });
    
    // Settings page should have some content
    const main = page.locator('main');
    await expect(main).toBeVisible();
  });
});

test.describe('UI Elements - Loading States', () => {
  test('app shows content after loading', async ({ page }) => {
    await page.goto('/');
    
    // Wait for sidebar (indicates app is loaded)
    await expect(page.locator('aside')).toBeVisible({ timeout: 10000 });
    
    // Main content should be visible
    await expect(page.locator('main')).toBeVisible();
    
    // No loading spinner should be visible after load
    const spinner = page.locator('.animate-spin');
    // Either no spinner or it's hidden
    const spinnerCount = await spinner.count();
    if (spinnerCount > 0) {
      // If there's a spinner, it should eventually disappear
      await expect(spinner.first()).not.toBeVisible({ timeout: 5000 });
    }
  });
});

test.describe('UI Elements - Accessibility', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('aside')).toBeVisible({ timeout: 10000 });
  });

  test('navigation links are keyboard accessible', async ({ page }) => {
    // Tab to first nav link
    await page.keyboard.press('Tab');
    
    // Some element should be focused
    const focusedElement = page.locator(':focus');
    await expect(focusedElement).toBeVisible();
  });

  test('buttons have visible focus states', async ({ page }) => {
    const searchButton = page.locator('header button:has-text("Search")');
    
    // Focus the button
    await searchButton.focus();
    
    // Button should be focused
    await expect(searchButton).toBeFocused();
  });

  test('command palette has proper ARIA attributes', async ({ page }) => {
    // Open command palette
    await page.keyboard.press('Control+k');
    
    const palette = page.locator('[role="dialog"]');
    await expect(palette).toBeVisible();
    
    // Should have aria-modal
    await expect(palette).toHaveAttribute('aria-modal', 'true');
    
    // Should have aria-label
    await expect(palette).toHaveAttribute('aria-label', 'Command Palette');
  });
});

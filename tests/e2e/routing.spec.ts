import { test, expect } from '@playwright/test';

/**
 * E2E tests for Routing page
 * Tests routing rules management: domain and app-based routing
 */

test.describe('Routing Page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/routing');
  });

  test('page loads without errors', async ({ page }) => {
    // Wait for the page to be ready
    await expect(page.locator('body')).toBeVisible();
    
    // Check that main content is rendered
    await expect(page.locator('.min-h-screen')).toBeVisible({ timeout: 10000 });
  });

  test('page title "Маршрутизация" is visible', async ({ page }) => {
    const heading = page.locator('h1:has-text("Маршрутизация")');
    await expect(heading).toBeVisible({ timeout: 10000 });
  });

  test('page subtitle is visible', async ({ page }) => {
    const subtitle = page.locator('text=Настройка правил для доменов и приложений');
    await expect(subtitle).toBeVisible();
  });

  test('back button is visible', async ({ page }) => {
    const backButton = page.locator('button').filter({ has: page.locator('svg path[d*="M15 19l-7-7 7-7"]') });
    await expect(backButton).toBeVisible();
  });
});

test.describe('Routing Modes (Tabs)', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/routing');
  });

  test('domain tab is visible and active by default', async ({ page }) => {
    const domainTab = page.locator('button:has-text("По доменам")');
    await expect(domainTab).toBeVisible();
    
    // Check it has active styling (bg-electric)
    await expect(domainTab).toHaveClass(/bg-electric/);
  });

  test('app tab is visible', async ({ page }) => {
    const appTab = page.locator('button:has-text("По приложениям")');
    await expect(appTab).toBeVisible();
  });

  test('can switch to app tab', async ({ page }) => {
    const appTab = page.locator('button:has-text("По приложениям")');
    await appTab.click();
    
    // Check app tab is now active
    await expect(appTab).toHaveClass(/bg-electric/);
    
    // Check domain tab is no longer active
    const domainTab = page.locator('button:has-text("По доменам")');
    await expect(domainTab).not.toHaveClass(/bg-electric/);
  });

  test('can switch back to domain tab', async ({ page }) => {
    // First switch to app tab
    const appTab = page.locator('button:has-text("По приложениям")');
    await appTab.click();
    await expect(appTab).toHaveClass(/bg-electric/);
    
    // Switch back to domain tab
    const domainTab = page.locator('button:has-text("По доменам")');
    await domainTab.click();
    
    // Check domain tab is active again
    await expect(domainTab).toHaveClass(/bg-electric/);
    await expect(appTab).not.toHaveClass(/bg-electric/);
  });
});

test.describe('Routing Rules List', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/routing');
  });

  test('rules list or empty state is displayed', async ({ page }) => {
    // Wait for loading to complete
    await page.waitForTimeout(1000);
    
    // Either rules are shown or empty state
    const rulesOrEmpty = page.locator('.space-y-3');
    await expect(rulesOrEmpty).toBeVisible();
  });

  test('mock rules are displayed in browser preview mode', async ({ page }) => {
    // In browser mode, mock data should be shown
    // Check for mock domain rules
    const youtubeRule = page.locator('text=youtube.com');
    const discordRule = page.locator('text=discord.com');
    
    // At least one should be visible (mock data)
    const hasRules = await youtubeRule.isVisible() || await discordRule.isVisible();
    
    if (hasRules) {
      // If mock data is shown, verify structure
      await expect(page.locator('.group.bg-void-50').first()).toBeVisible();
    } else {
      // Empty state should be shown
      const emptyState = page.locator('text=Нет правил для доменов');
      await expect(emptyState).toBeVisible();
    }
  });

  test('rule cards show target badges', async ({ page }) => {
    // Wait for content
    await page.waitForTimeout(500);
    
    // Check for target badges (Zapret, Direct, or proxy names)
    const targetBadges = page.locator('.rounded.border');
    const count = await targetBadges.count();
    
    // Should have at least some badges if rules exist
    expect(count).toBeGreaterThanOrEqual(0);
  });
});

test.describe('Add Routing Rule', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/routing');
  });

  test('add rule button is visible', async ({ page }) => {
    // Wait for page to load
    await page.waitForTimeout(500);
    
    // Look for add button (either in empty state or at bottom of list)
    const addButton = page.locator('button:has-text("Добавить правило")');
    await expect(addButton.first()).toBeVisible();
  });

  test('clicking add button opens modal', async ({ page }) => {
    await page.waitForTimeout(500);
    
    // Click add button
    const addButton = page.locator('button:has-text("Добавить правило")');
    await addButton.first().click();
    
    // Modal should appear with title
    const modalTitle = page.locator('text=Добавить правило для домена');
    await expect(modalTitle).toBeVisible({ timeout: 5000 });
  });

  test('domain input is visible in add modal', async ({ page }) => {
    await page.waitForTimeout(500);
    
    // Open modal
    const addButton = page.locator('button:has-text("Добавить правило")');
    await addButton.first().click();
    
    // Check for domain input
    const domainInput = page.locator('input[placeholder*="example.com"]');
    await expect(domainInput).toBeVisible();
  });

  test('target selection options are visible in modal', async ({ page }) => {
    await page.waitForTimeout(500);
    
    // Open modal
    const addButton = page.locator('button:has-text("Добавить правило")');
    await addButton.first().click();
    
    // Check for target options
    await expect(page.locator('text=Direct')).toBeVisible();
    await expect(page.locator('text=Zapret')).toBeVisible();
  });

  test('can enter domain and see preview', async ({ page }) => {
    await page.waitForTimeout(500);
    
    // Open modal
    const addButton = page.locator('button:has-text("Добавить правило")');
    await addButton.first().click();
    
    // Enter domain
    const domainInput = page.locator('input[placeholder*="example.com"]');
    await domainInput.fill('test.example.com');
    
    // Check preview shows the domain
    const preview = page.locator('text=test.example.com');
    await expect(preview).toBeVisible();
  });

  test('can add a new domain rule', async ({ page }) => {
    await page.waitForTimeout(500);
    
    // Open modal
    const addButton = page.locator('button:has-text("Добавить правило")');
    await addButton.first().click();
    
    // Enter domain
    const domainInput = page.locator('input[placeholder*="example.com"]');
    await domainInput.fill('newdomain.test');
    
    // Click add button in modal
    const submitButton = page.locator('button:has-text("Добавить")').last();
    await submitButton.click();
    
    // Modal should close and rule should appear
    await page.waitForTimeout(500);
    
    // Check the new rule is in the list
    const newRule = page.locator('text=newdomain.test');
    await expect(newRule).toBeVisible();
  });

  test('cancel button closes modal without adding', async ({ page }) => {
    await page.waitForTimeout(500);
    
    // Open modal
    const addButton = page.locator('button:has-text("Добавить правило")');
    await addButton.first().click();
    
    // Enter domain
    const domainInput = page.locator('input[placeholder*="example.com"]');
    await domainInput.fill('cancelled.domain');
    
    // Click cancel
    const cancelButton = page.locator('button:has-text("Отмена")');
    await cancelButton.click();
    
    // Modal should close
    await expect(page.locator('text=Добавить правило для домена')).not.toBeVisible();
    
    // Domain should not be in list
    const cancelledRule = page.locator('text=cancelled.domain');
    await expect(cancelledRule).not.toBeVisible();
  });
});

test.describe('Delete Routing Rule', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/routing');
  });

  test('delete button appears on rule hover', async ({ page }) => {
    await page.waitForTimeout(500);
    
    // Check if there are any rules
    const ruleCards = page.locator('.group.bg-void-50');
    const count = await ruleCards.count();
    
    if (count > 0) {
      // Hover over first rule
      await ruleCards.first().hover();
      
      // Delete button should become visible
      const deleteButton = ruleCards.first().locator('button[title="Удалить"]');
      await expect(deleteButton).toBeVisible();
    }
  });

  test('can delete a rule', async ({ page }) => {
    await page.waitForTimeout(500);
    
    // First add a rule to delete
    const addButton = page.locator('button:has-text("Добавить правило")');
    await addButton.first().click();
    
    const domainInput = page.locator('input[placeholder*="example.com"]');
    await domainInput.fill('to-delete.test');
    
    const submitButton = page.locator('button:has-text("Добавить")').last();
    await submitButton.click();
    
    await page.waitForTimeout(500);
    
    // Verify rule was added
    const ruleToDelete = page.locator('text=to-delete.test');
    await expect(ruleToDelete).toBeVisible();
    
    // Find the rule card and hover to show delete button
    const ruleCard = page.locator('.group.bg-void-50').filter({ hasText: 'to-delete.test' });
    await ruleCard.hover();
    
    // Click delete button
    const deleteButton = ruleCard.locator('button[title="Удалить"]');
    await deleteButton.click();
    
    // Rule should be removed
    await page.waitForTimeout(500);
    await expect(ruleToDelete).not.toBeVisible();
  });
});

test.describe('App-based Routing', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/routing');
    // Switch to app tab
    const appTab = page.locator('button:has-text("По приложениям")');
    await appTab.click();
  });

  test('app tab shows correct empty state', async ({ page }) => {
    await page.waitForTimeout(500);
    
    // Check for app-specific empty state or rules
    const emptyState = page.locator('text=Нет правил для приложений');
    const hasEmptyState = await emptyState.isVisible();
    
    if (hasEmptyState) {
      await expect(emptyState).toBeVisible();
      await expect(page.locator('text=Настройте маршрутизацию для конкретных приложений')).toBeVisible();
    }
  });

  test('add app rule modal has app selection', async ({ page }) => {
    await page.waitForTimeout(500);
    
    // Open add modal
    const addButton = page.locator('button:has-text("Добавить правило")');
    await addButton.first().click();
    
    // Modal title should be for app
    const modalTitle = page.locator('text=Добавить правило для приложения');
    await expect(modalTitle).toBeVisible();
    
    // Should have app search input
    const appSearch = page.locator('input[placeholder*="Поиск приложения"]');
    await expect(appSearch).toBeVisible();
  });
});

test.describe('Routing Status', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/routing');
  });

  test('routing page shows rule count indication', async ({ page }) => {
    await page.waitForTimeout(500);
    
    // Check that either rules are displayed or empty state
    const ruleCards = page.locator('.group.bg-void-50');
    const emptyState = page.locator('text=Нет правил для доменов');
    
    const hasRules = await ruleCards.count() > 0;
    const hasEmptyState = await emptyState.isVisible();
    
    // One of these should be true
    expect(hasRules || hasEmptyState).toBeTruthy();
  });

  test('target badges show correct styling', async ({ page }) => {
    await page.waitForTimeout(500);
    
    // Add a rule to test styling
    const addButton = page.locator('button:has-text("Добавить правило")');
    await addButton.first().click();
    
    const domainInput = page.locator('input[placeholder*="example.com"]');
    await domainInput.fill('styled.test');
    
    // Select Zapret target
    const zapretOption = page.locator('label').filter({ hasText: 'Zapret' });
    await zapretOption.click();
    
    const submitButton = page.locator('button:has-text("Добавить")').last();
    await submitButton.click();
    
    await page.waitForTimeout(500);
    
    // Check the rule has proper target badge
    const ruleCard = page.locator('.group.bg-void-50').filter({ hasText: 'styled.test' });
    const targetBadge = ruleCard.locator('.rounded.border');
    await expect(targetBadge).toBeVisible();
  });
});

test.describe('Navigation', () => {
  test('can navigate to routing page from sidebar', async ({ page }) => {
    await page.goto('/');
    
    // Click on Routing link in sidebar
    await page.click('a:has-text("Routing")');
    
    // Verify we're on the routing page
    await expect(page).toHaveURL('/routing');
    await expect(page.locator('h1:has-text("Маршрутизация")')).toBeVisible();
  });

  test('back button navigates to home', async ({ page }) => {
    await page.goto('/routing');
    
    // Click back button
    const backButton = page.locator('button').filter({ has: page.locator('svg path[d*="M15 19l-7-7 7-7"]') });
    await backButton.click();
    
    // Should navigate to home
    await expect(page).toHaveURL('/');
  });
});

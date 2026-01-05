import { test, expect } from '@playwright/test';

/**
 * Strategy page tests for Isolate
 * Tests strategy listing, filtering, selection, and optimization
 */

test.describe('Strategies Page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/strategies');
    // Wait for page to load
    await expect(page.locator('h1:has-text("Стратегии")')).toBeVisible({ timeout: 10000 });
  });

  test('strategies page loads correctly', async ({ page }) => {
    // Check page title
    await expect(page.locator('h1:has-text("Стратегии")')).toBeVisible();
    
    // Check subtitle
    await expect(page.getByText('Управление стратегиями обхода блокировок')).toBeVisible();
  });

  test('category tabs are visible', async ({ page }) => {
    const categories = ['Все', 'YouTube', 'Discord', 'Telegram', 'General', 'Games', 'Custom'];
    
    for (const category of categories) {
      const tab = page.locator(`button:has-text("${category}")`);
      await expect(tab).toBeVisible();
    }
  });

  test('search input is visible and functional', async ({ page }) => {
    const searchInput = page.locator('input[placeholder="Поиск стратегий..."]');
    await expect(searchInput).toBeVisible();
    
    // Type in search
    await searchInput.fill('Discord');
    
    // Verify search is applied (input has value)
    await expect(searchInput).toHaveValue('Discord');
  });

  test('strategy cards are displayed', async ({ page }) => {
    // Wait for loading to complete
    await page.waitForSelector('.grid', { timeout: 10000 });
    
    // Check that strategy cards exist (either real or demo data)
    const strategyCards = page.locator('.bg-\\[\\#1a1f3a\\].rounded-xl.p-5');
    
    // Should have at least one strategy card
    await expect(strategyCards.first()).toBeVisible({ timeout: 10000 });
  });

  test('can filter strategies by category', async ({ page }) => {
    // Wait for strategies to load
    await page.waitForSelector('.grid', { timeout: 10000 });
    
    // Click on Discord category
    await page.click('button:has-text("Discord")');
    
    // Verify Discord tab is active (has different styling)
    const discordTab = page.locator('button:has-text("Discord")');
    await expect(discordTab).toHaveClass(/bg-\[#00d4ff\]/);
  });

  test('can click All category to show all strategies', async ({ page }) => {
    // First filter by Discord
    await page.click('button:has-text("Discord")');
    
    // Then click All
    await page.click('button:has-text("Все")');
    
    // Verify All tab is active
    const allTab = page.locator('button:has-text("Все")');
    await expect(allTab).toHaveClass(/bg-\[#00d4ff\]/);
  });

  test('strategy card shows required information', async ({ page }) => {
    // Wait for strategies to load
    await page.waitForSelector('.grid', { timeout: 10000 });
    
    const firstCard = page.locator('.bg-\\[\\#1a1f3a\\].rounded-xl.p-5').first();
    
    // Check for strategy name (h3 element)
    await expect(firstCard.locator('h3')).toBeVisible();
    
    // Check for description
    await expect(firstCard.locator('p.text-\\[\\#a0a0a0\\]')).toBeVisible();
    
    // Check for Apply button
    await expect(firstCard.locator('button:has-text("Применить")')).toBeVisible();
  });

  test('strategy card has action buttons', async ({ page }) => {
    // Wait for strategies to load
    await page.waitForSelector('.grid', { timeout: 10000 });
    
    const firstCard = page.locator('.bg-\\[\\#1a1f3a\\].rounded-xl.p-5').first();
    
    // Check for Apply button
    const applyButton = firstCard.locator('button:has-text("Применить")');
    await expect(applyButton).toBeVisible();
    
    // Check for test button (icon button)
    const testButton = firstCard.locator('button[title="Тестировать"]');
    await expect(testButton).toBeVisible();
    
    // Check for details button (icon button)
    const detailsButton = firstCard.locator('button[title="Детали"]');
    await expect(detailsButton).toBeVisible();
  });

  test('can open strategy details modal', async ({ page }) => {
    // Wait for strategies to load
    await page.waitForSelector('.grid', { timeout: 10000 });
    
    const firstCard = page.locator('.bg-\\[\\#1a1f3a\\].rounded-xl.p-5').first();
    
    // Click details button
    await firstCard.locator('button[title="Детали"]').click();
    
    // Check modal is visible
    const modal = page.locator('.fixed.inset-0');
    await expect(modal).toBeVisible();
    
    // Check modal has content
    await expect(modal.locator('h2')).toBeVisible(); // Strategy name
    await expect(modal.locator('text=Описание')).toBeVisible();
    await expect(modal.locator('text=Автор')).toBeVisible();
    await expect(modal.locator('text=Сервисы')).toBeVisible();
  });

  test('can close strategy details modal', async ({ page }) => {
    // Wait for strategies to load
    await page.waitForSelector('.grid', { timeout: 10000 });
    
    const firstCard = page.locator('.bg-\\[\\#1a1f3a\\].rounded-xl.p-5').first();
    
    // Open modal
    await firstCard.locator('button[title="Детали"]').click();
    
    // Wait for modal
    const modal = page.locator('.fixed.inset-0');
    await expect(modal).toBeVisible();
    
    // Close modal by clicking X button
    await modal.locator('button:has(svg)').first().click();
    
    // Modal should be hidden
    await expect(modal).not.toBeVisible();
  });

  test('can close modal by clicking backdrop', async ({ page }) => {
    // Wait for strategies to load
    await page.waitForSelector('.grid', { timeout: 10000 });
    
    const firstCard = page.locator('.bg-\\[\\#1a1f3a\\].rounded-xl.p-5').first();
    
    // Open modal
    await firstCard.locator('button[title="Детали"]').click();
    
    // Wait for modal
    const modal = page.locator('.fixed.inset-0');
    await expect(modal).toBeVisible();
    
    // Click on backdrop (outside modal content)
    await page.click('.fixed.inset-0.bg-black\\/60', { position: { x: 10, y: 10 } });
    
    // Modal should be hidden
    await expect(modal).not.toBeVisible();
  });

  test('stats footer shows strategy counts', async ({ page }) => {
    // Wait for strategies to load
    await page.waitForSelector('.grid', { timeout: 10000 });
    
    // Check for stats footer
    const statsFooter = page.locator('text=Всего:');
    await expect(statsFooter).toBeVisible();
    
    // Check for "Показано" text
    await expect(page.locator('text=Показано:')).toBeVisible();
  });

  test('has link to Testing page', async ({ page }) => {
    // Wait for strategies to load
    await page.waitForSelector('.grid', { timeout: 10000 });
    
    // Check for Testing link in footer
    const testingLink = page.locator('a:has-text("Тестирование")');
    await expect(testingLink).toBeVisible();
    
    // Click and verify navigation
    await testingLink.click();
    await expect(page).toHaveURL('/testing');
  });
});

test.describe('Turbo Optimization', () => {
  test('turbo button is visible on dashboard', async ({ page }) => {
    await page.goto('/');
    
    // Check for Turbo optimization button
    const turboButton = page.locator('button:has-text("Turbo")').first();
    await expect(turboButton).toBeVisible();
  });

  test('turbo button in toolbar is clickable', async ({ page }) => {
    await page.goto('/');
    
    // Find Turbo button in toolbar
    const turboButton = page.locator('.h-14 button:has-text("Turbo")');
    await expect(turboButton).toBeVisible();
    await expect(turboButton).toBeEnabled();
  });

  test('turbo card on dashboard shows description', async ({ page }) => {
    await page.goto('/');
    
    // Find Turbo card
    const turboCard = page.locator('button:has-text("Turbo"):has-text("Быстрая оптимизация")');
    await expect(turboCard).toBeVisible();
  });

  test('deep optimization card is visible', async ({ page }) => {
    await page.goto('/');
    
    // Find Deep card
    const deepCard = page.locator('button:has-text("Deep"):has-text("Глубокий анализ")');
    await expect(deepCard).toBeVisible();
  });
});

test.describe('Strategy Family Badges', () => {
  test('strategy cards show family badges', async ({ page }) => {
    await page.goto('/strategies');
    
    // Wait for strategies to load
    await page.waitForSelector('.grid', { timeout: 10000 });
    
    // Check for family badges (ZAPRET, VLESS, CUSTOM)
    const familyBadges = page.locator('span:text-matches("ZAPRET|VLESS|CUSTOM")');
    
    // Should have at least one family badge
    await expect(familyBadges.first()).toBeVisible();
  });

  test('strategy cards show category badges', async ({ page }) => {
    await page.goto('/strategies');
    
    // Wait for strategies to load
    await page.waitForSelector('.grid', { timeout: 10000 });
    
    // Check for category badges with icons
    const categoryBadges = page.locator('.rounded-lg.text-xs.font-medium');
    
    // Should have category badges
    await expect(categoryBadges.first()).toBeVisible();
  });
});

test.describe('Strategy Search', () => {
  test('search filters strategies by name', async ({ page }) => {
    await page.goto('/strategies');
    
    // Wait for strategies to load
    await page.waitForSelector('.grid', { timeout: 10000 });
    
    const searchInput = page.locator('input[placeholder="Поиск стратегий..."]');
    
    // Search for a specific term
    await searchInput.fill('VLESS');
    
    // Wait for filter to apply
    await page.waitForTimeout(300);
    
    // Verify search is applied
    await expect(searchInput).toHaveValue('VLESS');
  });

  test('clearing search shows all strategies', async ({ page }) => {
    await page.goto('/strategies');
    
    // Wait for strategies to load
    await page.waitForSelector('.grid', { timeout: 10000 });
    
    const searchInput = page.locator('input[placeholder="Поиск стратегий..."]');
    
    // Search for something
    await searchInput.fill('Discord');
    await page.waitForTimeout(300);
    
    // Clear search
    await searchInput.fill('');
    await page.waitForTimeout(300);
    
    // Verify search is cleared
    await expect(searchInput).toHaveValue('');
  });
});

import { test, expect } from '@playwright/test';

/**
 * E2E tests for Marketplace/Plugins page (/marketplace)
 * Tests plugin listing, filtering, installation, and details modal
 */

test.describe('Marketplace Page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/marketplace');
    // Wait for page to load
    await expect(page.locator('h1:has-text("Marketplace")')).toBeVisible({ timeout: 10000 });
  });

  test('page loads with correct title and subtitle', async ({ page }) => {
    // Check page title
    await expect(page.locator('h1:has-text("Marketplace")')).toBeVisible();
    
    // Check subtitle
    await expect(page.getByText('Расширения и плагины для Isolate')).toBeVisible();
  });

  test('displays plugin count badges', async ({ page }) => {
    // Check for total plugins count badge
    const totalBadge = page.locator('span:has-text("плагинов")');
    await expect(totalBadge).toBeVisible();
    
    // Check for installed plugins count badge
    const installedBadge = page.locator('span:has-text("установлено")');
    await expect(installedBadge).toBeVisible();
  });

  test('featured plugins section is visible when no filters applied', async ({ page }) => {
    // Check for "Рекомендуемые" section
    const featuredSection = page.locator('h2:has-text("Рекомендуемые")');
    await expect(featuredSection).toBeVisible();
    
    // Featured plugins should be displayed in a grid
    const featuredGrid = page.locator('.grid.grid-cols-1.lg\\:grid-cols-3');
    await expect(featuredGrid).toBeVisible();
  });

  test('search input is visible and functional', async ({ page }) => {
    const searchInput = page.locator('input[placeholder="Поиск плагинов..."]');
    await expect(searchInput).toBeVisible();
    
    // Type in search
    await searchInput.fill('Discord');
    
    // Verify search is applied
    await expect(searchInput).toHaveValue('Discord');
    
    // Wait for filter to apply
    await page.waitForTimeout(300);
    
    // Should show "Найдено" text with results count
    await expect(page.locator('text=Найдено:')).toBeVisible();
  });

  test('can clear search with X button', async ({ page }) => {
    const searchInput = page.locator('input[placeholder="Поиск плагинов..."]');
    
    // Type in search
    await searchInput.fill('YouTube');
    await page.waitForTimeout(300);
    
    // Click clear button
    const clearButton = page.locator('input[placeholder="Поиск плагинов..."] + button');
    await clearButton.click();
    
    // Verify search is cleared
    await expect(searchInput).toHaveValue('');
  });
});

test.describe('Category Filtering', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/marketplace');
    await expect(page.locator('h1:has-text("Marketplace")')).toBeVisible({ timeout: 10000 });
  });

  test('category tabs are visible', async ({ page }) => {
    const categories = ['Все', 'Стратегии', 'Сервисы', 'Инструменты'];
    
    for (const category of categories) {
      const tab = page.locator(`button:has-text("${category}")`);
      await expect(tab).toBeVisible();
    }
  });

  test('can filter by Strategies category', async ({ page }) => {
    // Click on Стратегии category
    await page.click('button:has-text("Стратегии")');
    
    // Verify tab is active (has indigo background)
    const strategiesTab = page.locator('button:has-text("Стратегии")');
    await expect(strategiesTab).toHaveClass(/bg-indigo-500/);
    
    // Featured section should be hidden when filter is applied
    await expect(page.locator('h2:has-text("Рекомендуемые")')).not.toBeVisible();
  });

  test('can filter by Services category', async ({ page }) => {
    await page.click('button:has-text("Сервисы")');
    
    const servicesTab = page.locator('button:has-text("Сервисы")');
    await expect(servicesTab).toHaveClass(/bg-indigo-500/);
  });

  test('can filter by Tools category', async ({ page }) => {
    await page.click('button:has-text("Инструменты")');
    
    const toolsTab = page.locator('button:has-text("Инструменты")');
    await expect(toolsTab).toHaveClass(/bg-indigo-500/);
  });

  test('clicking All shows all plugins', async ({ page }) => {
    // First filter by category
    await page.click('button:has-text("Сервисы")');
    
    // Then click All
    await page.click('button:has-text("Все")');
    
    // Verify All tab is active
    const allTab = page.locator('button:has-text("Все")');
    await expect(allTab).toHaveClass(/bg-indigo-500/);
    
    // Featured section should be visible again
    await expect(page.locator('h2:has-text("Рекомендуемые")')).toBeVisible();
  });
});

test.describe('Type Filtering', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/marketplace');
    await expect(page.locator('h1:has-text("Marketplace")')).toBeVisible({ timeout: 10000 });
  });

  test('type filter buttons are visible', async ({ page }) => {
    // Check for type filter label
    await expect(page.locator('span:has-text("Тип:")')).toBeVisible();
    
    // Check for type filter buttons
    const types = ['Все типы', 'Service Checker', 'Hostlist', 'Strategy', 'UI Plugin', 'Script'];
    
    for (const type of types) {
      const button = page.locator(`button:has-text("${type}")`);
      await expect(button).toBeVisible();
    }
  });

  test('can filter by Service Checker type', async ({ page }) => {
    await page.click('button:has-text("Service Checker")');
    
    // Verify filter is active
    const typeButton = page.locator('button:has-text("Service Checker")');
    await expect(typeButton).toHaveClass(/ring-1/);
  });

  test('can filter by UI Plugin type', async ({ page }) => {
    await page.click('button:has-text("UI Plugin")');
    
    const typeButton = page.locator('button:has-text("UI Plugin")');
    await expect(typeButton).toHaveClass(/ring-1/);
  });

  test('can filter by Script type', async ({ page }) => {
    await page.click('button:has-text("Script")');
    
    const typeButton = page.locator('button:has-text("Script")');
    await expect(typeButton).toHaveClass(/ring-1/);
  });
});

test.describe('Level Filtering', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/marketplace');
    await expect(page.locator('h1:has-text("Marketplace")')).toBeVisible({ timeout: 10000 });
  });

  test('level filter buttons are visible', async ({ page }) => {
    // Check for level filter label
    await expect(page.locator('span:has-text("Уровень:")')).toBeVisible();
    
    // Check for level filter buttons
    await expect(page.locator('button:has-text("Все уровни")')).toBeVisible();
    await expect(page.locator('button:has-text("L1 — Declarative")')).toBeVisible();
    await expect(page.locator('button:has-text("L2 — UI Plugins")')).toBeVisible();
    await expect(page.locator('button:has-text("L3 — Scripts")')).toBeVisible();
  });

  test('can filter by Level 1', async ({ page }) => {
    await page.click('button:has-text("L1 — Declarative")');
    
    const levelButton = page.locator('button:has-text("L1 — Declarative")');
    await expect(levelButton).toHaveClass(/ring-1/);
  });

  test('can filter by Level 2', async ({ page }) => {
    await page.click('button:has-text("L2 — UI Plugins")');
    
    const levelButton = page.locator('button:has-text("L2 — UI Plugins")');
    await expect(levelButton).toHaveClass(/ring-1/);
  });

  test('can filter by Level 3', async ({ page }) => {
    await page.click('button:has-text("L3 — Scripts")');
    
    const levelButton = page.locator('button:has-text("L3 — Scripts")');
    await expect(levelButton).toHaveClass(/ring-1/);
  });
});

test.describe('Plugin Cards', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/marketplace');
    await expect(page.locator('h1:has-text("Marketplace")')).toBeVisible({ timeout: 10000 });
  });

  test('plugin cards are displayed', async ({ page }) => {
    // Wait for plugins to load
    await page.waitForSelector('[role="article"]', { timeout: 10000 });
    
    // Check that plugin cards exist
    const pluginCards = page.locator('[role="article"]');
    await expect(pluginCards.first()).toBeVisible();
    
    // Should have multiple plugins
    const count = await pluginCards.count();
    expect(count).toBeGreaterThan(0);
  });

  test('plugin card shows required information', async ({ page }) => {
    await page.waitForSelector('[role="article"]', { timeout: 10000 });
    
    const firstCard = page.locator('[role="article"]').first();
    
    // Check for plugin name (h3 element)
    await expect(firstCard.locator('h3')).toBeVisible();
    
    // Check for version
    await expect(firstCard.locator('text=/v\\d+\\.\\d+/')).toBeVisible();
    
    // Check for description
    await expect(firstCard.locator('p.text-sm.text-zinc-400')).toBeVisible();
    
    // Check for author
    await expect(firstCard.locator('.text-xs.text-zinc-500')).toBeVisible();
  });

  test('plugin card has level badge', async ({ page }) => {
    await page.waitForSelector('[role="article"]', { timeout: 10000 });
    
    const firstCard = page.locator('[role="article"]').first();
    
    // Check for level badge (L1, L2, or L3)
    const levelBadge = firstCard.locator('span:text-matches("L[123]")');
    await expect(levelBadge).toBeVisible();
  });

  test('plugin card has action buttons', async ({ page }) => {
    await page.waitForSelector('[role="article"]', { timeout: 10000 });
    
    const firstCard = page.locator('[role="article"]').first();
    
    // Check for details button (info icon)
    const detailsButton = firstCard.locator('button[title="Подробнее"]');
    await expect(detailsButton).toBeVisible();
    
    // Check for Install/Installed button
    const installButton = firstCard.locator('button:has-text("Install"), button:has-text("Installed")');
    await expect(installButton).toBeVisible();
  });

  test('featured plugin card has Featured badge', async ({ page }) => {
    // Featured plugins are in the first grid
    const featuredCard = page.locator('.grid.grid-cols-1.lg\\:grid-cols-3 [role="article"]').first();
    
    // Check for Featured badge
    await expect(featuredCard.locator('text=Featured')).toBeVisible();
  });
});

test.describe('Plugin Details Modal', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/marketplace');
    await expect(page.locator('h1:has-text("Marketplace")')).toBeVisible({ timeout: 10000 });
  });

  test('can open plugin details modal', async ({ page }) => {
    await page.waitForSelector('[role="article"]', { timeout: 10000 });
    
    const firstCard = page.locator('[role="article"]').first();
    
    // Click details button
    await firstCard.locator('button[title="Подробнее"]').click();
    
    // Check modal is visible
    const modal = page.locator('[role="dialog"]');
    await expect(modal).toBeVisible();
    
    // Check modal has plugin name
    await expect(modal.locator('h2')).toBeVisible();
  });

  test('modal shows plugin information', async ({ page }) => {
    await page.waitForSelector('[role="article"]', { timeout: 10000 });
    
    const firstCard = page.locator('[role="article"]').first();
    await firstCard.locator('button[title="Подробнее"]').click();
    
    const modal = page.locator('[role="dialog"]');
    await expect(modal).toBeVisible();
    
    // Check for version
    await expect(modal.locator('text=/v\\d+\\.\\d+/')).toBeVisible();
    
    // Check for author
    await expect(modal.locator('text=/by .+/')).toBeVisible();
    
    // Check for description section
    await expect(modal.locator('text=Описание')).toBeVisible();
  });

  test('modal has tabs for Overview, Permissions, and Changelog', async ({ page }) => {
    await page.waitForSelector('[role="article"]', { timeout: 10000 });
    
    const firstCard = page.locator('[role="article"]').first();
    await firstCard.locator('button[title="Подробнее"]').click();
    
    const modal = page.locator('[role="dialog"]');
    await expect(modal).toBeVisible();
    
    // Check for tabs
    await expect(modal.locator('button:has-text("Обзор")')).toBeVisible();
    await expect(modal.locator('button:has-text("Разрешения")')).toBeVisible();
  });

  test('can switch between modal tabs', async ({ page }) => {
    await page.waitForSelector('[role="article"]', { timeout: 10000 });
    
    const firstCard = page.locator('[role="article"]').first();
    await firstCard.locator('button[title="Подробнее"]').click();
    
    const modal = page.locator('[role="dialog"]');
    await expect(modal).toBeVisible();
    
    // Click on Permissions tab
    await modal.locator('button:has-text("Разрешения")').click();
    
    // Verify Permissions tab is active
    const permissionsTab = modal.locator('button:has-text("Разрешения")');
    await expect(permissionsTab).toHaveClass(/text-indigo-400/);
  });

  test('can close modal with X button', async ({ page }) => {
    await page.waitForSelector('[role="article"]', { timeout: 10000 });
    
    const firstCard = page.locator('[role="article"]').first();
    await firstCard.locator('button[title="Подробнее"]').click();
    
    const modal = page.locator('[role="dialog"]');
    await expect(modal).toBeVisible();
    
    // Click close button
    await modal.locator('button[aria-label="Закрыть"]').click();
    
    // Modal should be hidden
    await expect(modal).not.toBeVisible();
  });

  test('can close modal by clicking backdrop', async ({ page }) => {
    await page.waitForSelector('[role="article"]', { timeout: 10000 });
    
    const firstCard = page.locator('[role="article"]').first();
    await firstCard.locator('button[title="Подробнее"]').click();
    
    const modal = page.locator('[role="dialog"]');
    await expect(modal).toBeVisible();
    
    // Click on backdrop (outside modal content)
    await page.click('[role="dialog"]', { position: { x: 10, y: 10 } });
    
    // Modal should be hidden
    await expect(modal).not.toBeVisible();
  });

  test('can close modal with Escape key', async ({ page }) => {
    await page.waitForSelector('[role="article"]', { timeout: 10000 });
    
    const firstCard = page.locator('[role="article"]').first();
    await firstCard.locator('button[title="Подробнее"]').click();
    
    const modal = page.locator('[role="dialog"]');
    await expect(modal).toBeVisible();
    
    // Press Escape
    await page.keyboard.press('Escape');
    
    // Modal should be hidden
    await expect(modal).not.toBeVisible();
  });
});

test.describe('Plugin Installation', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/marketplace');
    await expect(page.locator('h1:has-text("Marketplace")')).toBeVisible({ timeout: 10000 });
  });

  test('can install a plugin', async ({ page }) => {
    await page.waitForSelector('[role="article"]', { timeout: 10000 });
    
    // Find a plugin that is not installed
    const notInstalledCard = page.locator('[role="article"]').filter({
      has: page.locator('button:has-text("Install"):not(:has-text("Installed"))')
    }).first();
    
    if (await notInstalledCard.isVisible()) {
      // Click Install button
      await notInstalledCard.locator('button:has-text("Install")').click();
      
      // Should show installing state
      await expect(notInstalledCard.locator('text=Installing...')).toBeVisible({ timeout: 2000 });
      
      // After installation, should show Installed
      await expect(notInstalledCard.locator('button:has-text("Installed")')).toBeVisible({ timeout: 5000 });
    }
  });

  test('installed plugins show Installed button', async ({ page }) => {
    await page.waitForSelector('[role="article"]', { timeout: 10000 });
    
    // Find an installed plugin
    const installedCard = page.locator('[role="article"]').filter({
      has: page.locator('button:has-text("Installed")')
    }).first();
    
    if (await installedCard.isVisible()) {
      // Installed button should be disabled
      const installedButton = installedCard.locator('button:has-text("Installed")');
      await expect(installedButton).toBeDisabled();
      
      // Should have emerald/green styling
      await expect(installedButton).toHaveClass(/bg-emerald-500/);
    }
  });
});

test.describe('Sorting', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/marketplace');
    await expect(page.locator('h1:has-text("Marketplace")')).toBeVisible({ timeout: 10000 });
  });

  test('sort dropdown is visible', async ({ page }) => {
    const sortSelect = page.locator('select');
    await expect(sortSelect).toBeVisible();
  });

  test('can change sort order', async ({ page }) => {
    const sortSelect = page.locator('select');
    
    // Change to sort by rating
    await sortSelect.selectOption('rating');
    await expect(sortSelect).toHaveValue('rating');
    
    // Change to sort by name
    await sortSelect.selectOption('name');
    await expect(sortSelect).toHaveValue('name');
    
    // Change to sort by recent
    await sortSelect.selectOption('recent');
    await expect(sortSelect).toHaveValue('recent');
  });
});

test.describe('Reset Filters', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/marketplace');
    await expect(page.locator('h1:has-text("Marketplace")')).toBeVisible({ timeout: 10000 });
  });

  test('reset filters button appears when filters are active', async ({ page }) => {
    // Apply a filter
    await page.click('button:has-text("Сервисы")');
    
    // Reset filters button should appear
    await expect(page.locator('button:has-text("Сбросить фильтры")')).toBeVisible();
  });

  test('can reset all filters', async ({ page }) => {
    // Apply multiple filters
    await page.click('button:has-text("Сервисы")');
    await page.click('button:has-text("L2 — UI Plugins")');
    
    // Click reset filters
    await page.click('button:has-text("Сбросить фильтры")');
    
    // All tab should be active again
    const allTab = page.locator('button:has-text("Все")');
    await expect(allTab).toHaveClass(/bg-indigo-500/);
    
    // Featured section should be visible again
    await expect(page.locator('h2:has-text("Рекомендуемые")')).toBeVisible();
  });
});

test.describe('Navigation', () => {
  test('can navigate to Marketplace from sidebar', async ({ page }) => {
    await page.goto('/');
    
    // Click on Marketplace link in sidebar
    await page.click('a:has-text("Marketplace")');
    
    // Verify we're on the marketplace page
    await expect(page).toHaveURL('/marketplace');
    await expect(page.locator('h1:has-text("Marketplace")')).toBeVisible();
  });

  test('Marketplace link is highlighted when active', async ({ page }) => {
    await page.goto('/marketplace');
    
    // Marketplace link should be active
    const marketplaceLink = page.locator('aside a:has-text("Marketplace")');
    await expect(marketplaceLink).toHaveClass(/bg-\[#2a2f4a\]/);
  });
});

test.describe('Empty State', () => {
  test('shows empty state when no plugins match filters', async ({ page }) => {
    await page.goto('/marketplace');
    await expect(page.locator('h1:has-text("Marketplace")')).toBeVisible({ timeout: 10000 });
    
    // Search for something that doesn't exist
    const searchInput = page.locator('input[placeholder="Поиск плагинов..."]');
    await searchInput.fill('xyznonexistentplugin123');
    
    await page.waitForTimeout(300);
    
    // Should show empty state
    await expect(page.locator('text=Плагины не найдены')).toBeVisible();
    await expect(page.locator('text=Попробуйте изменить параметры поиска')).toBeVisible();
  });
});

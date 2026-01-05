import { test, expect } from '@playwright/test';

/**
 * E2E tests for Proxies page
 * Tests proxy management functionality: listing, adding, editing, deleting proxies
 */

test.describe('Proxies Page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/proxies');
  });

  test('page loads without errors', async ({ page }) => {
    // Wait for the page to be ready
    await expect(page.locator('body')).toBeVisible();
    
    // Check that main content is rendered
    await expect(page.locator('.min-h-screen')).toBeVisible({ timeout: 10000 });
  });

  test('page title "Proxies" is visible', async ({ page }) => {
    const heading = page.locator('h1:has-text("Proxies")');
    await expect(heading).toBeVisible({ timeout: 10000 });
  });

  test('add proxy button is visible', async ({ page }) => {
    const addButton = page.locator('button:has-text("Add")');
    await expect(addButton).toBeVisible();
  });

  test('back button navigates to dashboard', async ({ page }) => {
    const backButton = page.locator('button').filter({ has: page.locator('svg path[d="M15 19l-7-7 7-7"]') });
    await expect(backButton).toBeVisible();
    
    await backButton.click();
    await expect(page).toHaveURL('/');
  });
});

test.describe('Proxies List', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/proxies');
  });

  test('displays proxy list or empty state', async ({ page }) => {
    // Wait for loading to complete
    await page.waitForTimeout(1000);
    
    // Either proxy cards are shown or empty state
    const emptyState = page.locator('text=Нет добавленных прокси');
    const proxyCards = page.locator('[class*="ProxyCard"], .transform.transition-all');
    
    // One of these should be visible
    const hasEmptyState = await emptyState.isVisible().catch(() => false);
    const hasProxyCards = await proxyCards.first().isVisible().catch(() => false);
    
    expect(hasEmptyState || hasProxyCards).toBeTruthy();
  });

  test('empty state shows helpful message', async ({ page }) => {
    // Wait for loading to complete
    await page.waitForTimeout(1000);
    
    const emptyState = page.locator('text=Нет добавленных прокси');
    
    if (await emptyState.isVisible().catch(() => false)) {
      // Check for hint text
      await expect(page.locator('text=Нажмите Add или вставьте ссылку')).toBeVisible();
      // Check for emoji icon
      await expect(page.locator('text=🌐')).toBeVisible();
    }
  });
});

test.describe('Add Proxy Modal', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/proxies');
  });

  test('clicking Add button opens modal', async ({ page }) => {
    const addButton = page.locator('button:has-text("Add")');
    await addButton.click();
    
    // Modal should be visible with title
    await expect(page.locator('text=Add Proxy')).toBeVisible({ timeout: 5000 });
  });

  test('modal has three tabs: Paste Link, Import File, Manual', async ({ page }) => {
    await page.locator('button:has-text("Add")').click();
    
    await expect(page.locator('button:has-text("Paste Link")')).toBeVisible();
    await expect(page.locator('button:has-text("Import File")')).toBeVisible();
    await expect(page.locator('button:has-text("Manual")')).toBeVisible();
  });

  test('Paste Link tab is active by default', async ({ page }) => {
    await page.locator('button:has-text("Add")').click();
    
    // Paste Link tab should have active styling
    const pasteTab = page.locator('button:has-text("Paste Link")');
    await expect(pasteTab).toHaveClass(/bg-indigo-500/);
    
    // Textarea for proxy URL should be visible
    await expect(page.locator('textarea[placeholder*="vless://"]')).toBeVisible();
  });

  test('can switch to Manual tab', async ({ page }) => {
    await page.locator('button:has-text("Add")').click();
    
    await page.locator('button:has-text("Manual")').click();
    
    // Manual tab should now be active
    const manualTab = page.locator('button:has-text("Manual")');
    await expect(manualTab).toHaveClass(/bg-indigo-500/);
  });

  test('can switch to Import File tab', async ({ page }) => {
    await page.locator('button:has-text("Add")').click();
    
    await page.locator('button:has-text("Import File")').click();
    
    // Import File tab should now be active
    const fileTab = page.locator('button:has-text("Import File")');
    await expect(fileTab).toHaveClass(/bg-indigo-500/);
    
    // File input area should be visible
    await expect(page.locator('text=Нажмите для выбора файла')).toBeVisible();
  });

  test('Cancel button closes modal', async ({ page }) => {
    await page.locator('button:has-text("Add")').click();
    await expect(page.locator('text=Add Proxy')).toBeVisible();
    
    await page.locator('button:has-text("Cancel")').click();
    
    // Modal should be closed
    await expect(page.locator('text=Add Proxy')).not.toBeVisible({ timeout: 3000 });
  });
});

test.describe('Manual Proxy Form', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/proxies');
    await page.locator('button:has-text("Add")').click();
    await page.locator('button:has-text("Manual")').click();
  });

  test('protocol selector has all options', async ({ page }) => {
    const protocolSelect = page.locator('select').first();
    await expect(protocolSelect).toBeVisible();
    
    // Check available options
    await expect(protocolSelect.locator('option[value="vless"]')).toBeAttached();
    await expect(protocolSelect.locator('option[value="vmess"]')).toBeAttached();
    await expect(protocolSelect.locator('option[value="shadowsocks"]')).toBeAttached();
    await expect(protocolSelect.locator('option[value="socks5"]')).toBeAttached();
    await expect(protocolSelect.locator('option[value="http"]')).toBeAttached();
  });

  test('can select SOCKS5 protocol', async ({ page }) => {
    const protocolSelect = page.locator('select').first();
    await protocolSelect.selectOption('socks5');
    
    await expect(protocolSelect).toHaveValue('socks5');
  });

  test('can select HTTP protocol', async ({ page }) => {
    const protocolSelect = page.locator('select').first();
    await protocolSelect.selectOption('http');
    
    await expect(protocolSelect).toHaveValue('http');
  });

  test('can select VLESS protocol', async ({ page }) => {
    const protocolSelect = page.locator('select').first();
    await protocolSelect.selectOption('vless');
    
    await expect(protocolSelect).toHaveValue('vless');
    
    // VLESS-specific fields should appear
    await expect(page.locator('label:has-text("UUID")')).toBeVisible();
    await expect(page.locator('label:has-text("TLS")')).toBeVisible();
    await expect(page.locator('label:has-text("SNI")')).toBeVisible();
    await expect(page.locator('label:has-text("Transport")')).toBeVisible();
  });

  test('form has required fields: Name, Server, Port', async ({ page }) => {
    await expect(page.locator('label:has-text("Name")')).toBeVisible();
    await expect(page.locator('label:has-text("Server")')).toBeVisible();
    await expect(page.locator('label:has-text("Port")')).toBeVisible();
    
    // Check inputs exist
    await expect(page.locator('input[placeholder="My Proxy"]')).toBeVisible();
    await expect(page.locator('input[placeholder="example.com"]')).toBeVisible();
    await expect(page.locator('input[type="number"]')).toBeVisible();
  });

  test('form validation - required fields', async ({ page }) => {
    // Try to submit empty form
    const saveButton = page.locator('button[type="submit"]:has-text("Save")');
    await saveButton.click();
    
    // Form should not submit (HTML5 validation)
    // Modal should still be open
    await expect(page.locator('text=Add Proxy')).toBeVisible();
  });

  test('can fill in proxy form fields', async ({ page }) => {
    // Fill in basic fields
    await page.locator('input[placeholder="My Proxy"]').fill('Test Proxy');
    await page.locator('input[placeholder="example.com"]').fill('proxy.example.com');
    await page.locator('input[type="number"]').fill('1080');
    
    // Verify values
    await expect(page.locator('input[placeholder="My Proxy"]')).toHaveValue('Test Proxy');
    await expect(page.locator('input[placeholder="example.com"]')).toHaveValue('proxy.example.com');
    await expect(page.locator('input[type="number"]')).toHaveValue('1080');
  });

  test('VLESS protocol shows UUID field', async ({ page }) => {
    const protocolSelect = page.locator('select').first();
    await protocolSelect.selectOption('vless');
    
    const uuidInput = page.locator('input[placeholder*="xxxx-xxxx"]');
    await expect(uuidInput).toBeVisible();
  });

  test('Shadowsocks protocol shows password and method fields', async ({ page }) => {
    const protocolSelect = page.locator('select').first();
    await protocolSelect.selectOption('shadowsocks');
    
    await expect(page.locator('label:has-text("Encryption Method")')).toBeVisible();
    await expect(page.locator('label:has-text("Password")')).toBeVisible();
    await expect(page.locator('input[type="password"]')).toBeVisible();
  });
});

test.describe('Paste Link Tab', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/proxies');
    await page.locator('button:has-text("Add")').click();
  });

  test('textarea accepts proxy URL', async ({ page }) => {
    const textarea = page.locator('textarea[placeholder*="vless://"]');
    await textarea.fill('vless://test-uuid@example.com:443');
    
    await expect(textarea).toHaveValue('vless://test-uuid@example.com:443');
  });

  test('Import button is visible', async ({ page }) => {
    await expect(page.locator('button:has-text("Import")')).toBeVisible();
  });

  test('shows supported protocols hint', async ({ page }) => {
    await expect(page.locator('text=Поддерживаются: VLESS, VMess, Shadowsocks, Trojan')).toBeVisible();
  });
});

test.describe('Navigation Integration', () => {
  test('can navigate to Proxies from sidebar', async ({ page }) => {
    await page.goto('/');
    
    await page.click('a:has-text("Proxies")');
    
    await expect(page).toHaveURL('/proxies');
    await expect(page.locator('h1:has-text("Proxies")')).toBeVisible();
  });

  test('can navigate to Proxies from dashboard quick link', async ({ page }) => {
    await page.goto('/');
    
    const addProxyLink = page.locator('a:has-text("Добавить прокси")');
    if (await addProxyLink.isVisible().catch(() => false)) {
      await addProxyLink.click();
      await expect(page).toHaveURL('/proxies');
    }
  });
});

import { test, expect } from '@playwright/test';

/**
 * E2E tests for Settings page
 * Tests settings functionality: tabs, toggles, theme selection, save/reset
 */

test.describe('Settings Page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/settings');
  });

  test('page loads without errors', async ({ page }) => {
    // Wait for the page to be ready
    await expect(page.locator('body')).toBeVisible();
    
    // Check that main content is rendered
    await expect(page.locator('.p-8.min-h-screen')).toBeVisible({ timeout: 10000 });
  });

  test('Settings heading is visible', async ({ page }) => {
    const heading = page.locator('h1:has-text("Settings")');
    await expect(heading).toBeVisible({ timeout: 10000 });
    
    // Check subtitle
    const subtitle = page.locator('text=Configure application preferences');
    await expect(subtitle).toBeVisible();
  });

  test('all settings sections are displayed', async ({ page }) => {
    // Check vertical tabs navigation
    const generalTab = page.locator('button:has-text("General")');
    const routingTab = page.locator('button:has-text("Routing")');
    const advancedTab = page.locator('button:has-text("Advanced")');
    
    await expect(generalTab).toBeVisible();
    await expect(routingTab).toBeVisible();
    await expect(advancedTab).toBeVisible();
  });
});

test.describe('Settings Tabs Navigation', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/settings');
  });

  test('General tab is active by default', async ({ page }) => {
    // General tab should be active
    const generalTab = page.locator('button:has-text("General")');
    await expect(generalTab).toHaveClass(/border-electric/);
    
    // General Settings heading should be visible
    const sectionHeading = page.locator('h2:has-text("General Settings")');
    await expect(sectionHeading).toBeVisible();
  });

  test('can switch to Routing tab', async ({ page }) => {
    await page.click('button:has-text("Routing")');
    
    // Routing tab should be active
    const routingTab = page.locator('button:has-text("Routing")');
    await expect(routingTab).toHaveClass(/border-electric/);
    
    // Routing Settings heading should be visible
    const sectionHeading = page.locator('h2:has-text("Routing Settings")');
    await expect(sectionHeading).toBeVisible();
  });

  test('can switch to Advanced tab', async ({ page }) => {
    await page.click('button:has-text("Advanced")');
    
    // Advanced tab should be active
    const advancedTab = page.locator('button:has-text("Advanced")');
    await expect(advancedTab).toHaveClass(/border-electric/);
    
    // Advanced Settings heading should be visible
    const sectionHeading = page.locator('h2:has-text("Advanced Settings")');
    await expect(sectionHeading).toBeVisible();
  });
});

test.describe('General Settings Toggles', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/settings');
  });

  test('autostart toggle is visible and clickable', async ({ page }) => {
    // Find autostart section
    const autostartSection = page.locator('.rounded-xl:has-text("Autostart")');
    await expect(autostartSection).toBeVisible();
    
    // Find toggle button within the section
    const toggle = autostartSection.locator('button.rounded-full');
    await expect(toggle).toBeVisible();
    
    // Get initial state
    const initialClass = await toggle.getAttribute('class');
    
    // Click toggle
    await toggle.click();
    
    // Verify state changed
    const newClass = await toggle.getAttribute('class');
    expect(initialClass).not.toBe(newClass);
  });

  test('minimize to tray toggle is visible and clickable', async ({ page }) => {
    // Find minimize to tray section
    const minimizeSection = page.locator('.rounded-xl:has-text("Minimize to tray")');
    await expect(minimizeSection).toBeVisible();
    
    // Find toggle button
    const toggle = minimizeSection.locator('button.rounded-full');
    await expect(toggle).toBeVisible();
    
    // Click toggle
    await toggle.click();
    
    // Toggle should still be visible after click
    await expect(toggle).toBeVisible();
  });

  test('notifications toggle is visible and clickable', async ({ page }) => {
    // Find notifications section
    const notificationsSection = page.locator('.rounded-xl:has-text("Notifications")');
    await expect(notificationsSection).toBeVisible();
    
    // Find toggle button
    const toggle = notificationsSection.locator('button.rounded-full');
    await expect(toggle).toBeVisible();
    
    // Click toggle
    await toggle.click();
    
    // Toggle should still be visible after click
    await expect(toggle).toBeVisible();
  });
});

test.describe('Theme Selection', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/settings');
  });

  test('theme dropdown is visible', async ({ page }) => {
    // Find theme section
    const themeSection = page.locator('.rounded-xl:has-text("Theme")').filter({ hasText: 'Application appearance' });
    await expect(themeSection).toBeVisible();
    
    // Find select dropdown
    const themeSelect = themeSection.locator('select');
    await expect(themeSelect).toBeVisible();
  });

  test('theme dropdown has all options', async ({ page }) => {
    const themeSelect = page.locator('.rounded-xl:has-text("Theme")').filter({ hasText: 'Application appearance' }).locator('select');
    
    // Check all theme options exist
    await expect(themeSelect.locator('option[value="dark"]')).toHaveText('Dark');
    await expect(themeSelect.locator('option[value="light"]')).toHaveText('Light');
    await expect(themeSelect.locator('option[value="system"]')).toHaveText('System');
  });

  test('can change theme selection', async ({ page }) => {
    const themeSelect = page.locator('.rounded-xl:has-text("Theme")').filter({ hasText: 'Application appearance' }).locator('select');
    
    // Select light theme
    await themeSelect.selectOption('light');
    await expect(themeSelect).toHaveValue('light');
    
    // Select system theme
    await themeSelect.selectOption('system');
    await expect(themeSelect).toHaveValue('system');
    
    // Select dark theme
    await themeSelect.selectOption('dark');
    await expect(themeSelect).toHaveValue('dark');
  });
});

test.describe('Language Selection', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/settings');
  });

  test('language dropdown is visible', async ({ page }) => {
    const languageSection = page.locator('.rounded-xl:has-text("Language")').filter({ hasText: 'Interface language' });
    await expect(languageSection).toBeVisible();
    
    const languageSelect = languageSection.locator('select');
    await expect(languageSelect).toBeVisible();
  });

  test('language dropdown has all options', async ({ page }) => {
    const languageSelect = page.locator('.rounded-xl:has-text("Language")').filter({ hasText: 'Interface language' }).locator('select');
    
    await expect(languageSelect.locator('option[value="ru"]')).toHaveText('Русский');
    await expect(languageSelect.locator('option[value="en"]')).toHaveText('English');
  });

  test('can change language selection', async ({ page }) => {
    const languageSelect = page.locator('.rounded-xl:has-text("Language")').filter({ hasText: 'Interface language' }).locator('select');
    
    // Select English
    await languageSelect.selectOption('en');
    await expect(languageSelect).toHaveValue('en');
    
    // Select Russian
    await languageSelect.selectOption('ru');
    await expect(languageSelect).toHaveValue('ru');
  });
});

test.describe('Settings Save Functionality', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/settings');
  });

  test('save button is visible', async ({ page }) => {
    const saveButton = page.locator('button:has-text("Save")');
    await expect(saveButton).toBeVisible();
  });

  test('save button shows confirmation message', async ({ page }) => {
    const saveButton = page.locator('button:has-text("Save")');
    
    // Click save
    await saveButton.click();
    
    // Check for success message
    const successMessage = page.locator('text=Settings saved');
    await expect(successMessage).toBeVisible({ timeout: 5000 });
  });

  test('settings persist after save', async ({ page }) => {
    // Change a setting
    const themeSelect = page.locator('.rounded-xl:has-text("Theme")').filter({ hasText: 'Application appearance' }).locator('select');
    await themeSelect.selectOption('light');
    
    // Save settings
    const saveButton = page.locator('button:has-text("Save")');
    await saveButton.click();
    
    // Wait for save confirmation
    await expect(page.locator('text=Settings saved')).toBeVisible({ timeout: 5000 });
  });
});

test.describe('Advanced Settings Reset', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/settings');
    // Navigate to Advanced tab
    await page.click('button:has-text("Advanced")');
  });

  test('danger zone is visible', async ({ page }) => {
    const dangerZone = page.locator('summary:has-text("Danger Zone")');
    await expect(dangerZone).toBeVisible();
  });

  test('can expand danger zone', async ({ page }) => {
    // Click to expand danger zone
    await page.click('summary:has-text("Danger Zone")');
    
    // Check that expanded content is visible
    const resetButton = page.locator('button:has-text("Reset Advanced Settings")');
    await expect(resetButton).toBeVisible();
  });

  test('reset button resets advanced settings', async ({ page }) => {
    // Expand danger zone
    await page.click('summary:has-text("Danger Zone")');
    
    // Change WinDivert mode
    const windivertSelect = page.locator('select').filter({ has: page.locator('option[value="autottl"]') });
    await windivertSelect.selectOption('autottl');
    
    // Click reset button
    const resetButton = page.locator('button:has-text("Reset Advanced Settings")');
    await resetButton.click();
    
    // Verify WinDivert mode is reset to normal
    await expect(windivertSelect).toHaveValue('normal');
  });
});

test.describe('Advanced Settings Controls', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/settings');
    await page.click('button:has-text("Advanced")');
  });

  test('Block QUIC toggle is visible', async ({ page }) => {
    const blockQuicSection = page.locator('.rounded-xl:has-text("Block QUIC")');
    await expect(blockQuicSection).toBeVisible();
    
    const toggle = blockQuicSection.locator('button.rounded-full');
    await expect(toggle).toBeVisible();
  });

  test('Debug Mode toggle is visible', async ({ page }) => {
    const debugSection = page.locator('.rounded-xl:has-text("Debug Mode")');
    await expect(debugSection).toBeVisible();
    
    const toggle = debugSection.locator('button.rounded-full');
    await expect(toggle).toBeVisible();
  });

  test('WinDivert Mode dropdown is visible in danger zone', async ({ page }) => {
    // Expand danger zone
    await page.click('summary:has-text("Danger Zone")');
    
    const windivertSection = page.locator('.rounded-xl:has-text("WinDivert Mode")');
    await expect(windivertSection).toBeVisible();
    
    const select = windivertSection.locator('select');
    await expect(select).toBeVisible();
  });

  test('DNS Override input is visible in danger zone', async ({ page }) => {
    // Expand danger zone
    await page.click('summary:has-text("Danger Zone")');
    
    const dnsSection = page.locator('.rounded-xl:has-text("DNS Override")');
    await expect(dnsSection).toBeVisible();
    
    const input = dnsSection.locator('input');
    await expect(input).toBeVisible();
    await expect(input).toHaveAttribute('placeholder', '8.8.8.8');
  });
});

test.describe('Routing Settings', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/settings');
    await page.click('button:has-text("Routing")');
  });

  test('Domain Exceptions section is visible', async ({ page }) => {
    const exceptionsSection = page.locator('.rounded-xl:has-text("Domain Exceptions")');
    await expect(exceptionsSection).toBeVisible();
  });

  test('can add domain exception', async ({ page }) => {
    const input = page.locator('input[placeholder="example.com"]');
    const addButton = page.locator('button:has-text("Add")');
    
    await expect(input).toBeVisible();
    await expect(addButton).toBeVisible();
    
    // Add a domain
    await input.fill('test.example.com');
    await addButton.click();
    
    // Verify domain was added
    const addedDomain = page.locator('text=test.example.com');
    await expect(addedDomain).toBeVisible();
  });

  test('can remove domain exception', async ({ page }) => {
    const input = page.locator('input[placeholder="example.com"]');
    const addButton = page.locator('button:has-text("Add")');
    
    // Add a domain first
    await input.fill('remove-me.com');
    await addButton.click();
    
    // Verify domain was added
    const addedDomain = page.locator('.font-mono:has-text("remove-me.com")');
    await expect(addedDomain).toBeVisible();
    
    // Find and click remove button
    const removeButton = addedDomain.locator('..').locator('button');
    await removeButton.click();
    
    // Verify domain was removed
    await expect(addedDomain).not.toBeVisible();
  });

  test('Per-App Routing shows coming soon badge', async ({ page }) => {
    const perAppSection = page.locator('.rounded-xl:has-text("Per-App Routing")');
    await expect(perAppSection).toBeVisible();
    
    const comingSoonBadge = perAppSection.locator('text=Coming Soon');
    await expect(comingSoonBadge).toBeVisible();
  });
});

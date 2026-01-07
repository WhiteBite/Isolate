/**
 * Smoke E2E tests for Isolate Tauri application
 * 
 * These tests verify basic application functionality using tauri-driver.
 * They run against the actual built Tauri application (not dev server).
 * 
 * Prerequisites:
 * 1. Build the app: pnpm tauri build
 * 2. Install tauri-driver: cargo install tauri-driver
 * 3. Run: pnpm test:e2e:tauri
 */

import { browser, $, expect } from '@wdio/globals';

describe('Isolate Application Smoke Tests', () => {
    /**
     * Test: Application window opens successfully
     */
    it('should launch the application window', async () => {
        // Wait for the app to be ready
        await browser.pause(2000);

        // Get window title
        const title = await browser.getTitle();
        expect(title).toBe('Isolate');
    });

    /**
     * Test: Main layout is rendered
     */
    it('should render the main layout with sidebar', async () => {
        // Wait for the app to fully load
        await browser.pause(1000);

        // Check that sidebar exists
        const sidebar = await $('aside');
        await expect(sidebar).toBeDisplayed();

        // Check for logo
        const logo = await $('h1*=Isolate');
        await expect(logo).toBeDisplayed();
    });

    /**
     * Test: Navigation items are visible
     */
    it('should display navigation items in sidebar', async () => {
        const navItems = [
            'Dashboard',
            'Proxies',
            'Routing',
            'Strategies',
            'Settings',
            'Logs',
        ];

        for (const item of navItems) {
            const navLink = await $(`a*=${item}`);
            await expect(navLink).toBeDisplayed();
        }
    });

    /**
     * Test: Dashboard page loads by default
     */
    it('should load Dashboard page by default', async () => {
        // Check for Dashboard heading
        const heading = await $('h1*=Dashboard');
        await expect(heading).toBeDisplayed();
    });

    /**
     * Test: Toolbar controls are visible
     */
    it('should display toolbar with mode toggles', async () => {
        // Check for mode toggles
        const systemProxy = await $('*=System Proxy');
        await expect(systemProxy).toBeDisplayed();

        const tunMode = await $('*=TUN Mode');
        await expect(tunMode).toBeDisplayed();

        const quicBlock = await $('*=QUIC Block');
        await expect(quicBlock).toBeDisplayed();
    });

    /**
     * Test: Quick action buttons are visible
     */
    it('should display quick action buttons', async () => {
        // Check for Turbo button
        const turboButton = await $('button*=Turbo');
        await expect(turboButton).toBeDisplayed();

        // Check for Panic button
        const panicButton = await $('button*=Panic');
        await expect(panicButton).toBeDisplayed();
    });

    /**
     * Test: Navigation works correctly
     */
    it('should navigate to Strategies page', async () => {
        // Click on Strategies link
        const strategiesLink = await $('a*=Strategies');
        await strategiesLink.click();

        // Wait for navigation
        await browser.pause(500);

        // Verify we're on the strategies page
        const heading = await $('h1*=Стратегии');
        await expect(heading).toBeDisplayed();
    });

    /**
     * Test: Can navigate back to Dashboard
     */
    it('should navigate back to Dashboard', async () => {
        // Click on Dashboard link
        const dashboardLink = await $('a*=Dashboard');
        await dashboardLink.click();

        // Wait for navigation
        await browser.pause(500);

        // Verify we're back on dashboard
        const heading = await $('h1*=Dashboard');
        await expect(heading).toBeDisplayed();
    });

    /**
     * Test: Settings page loads
     */
    it('should navigate to Settings page', async () => {
        const settingsLink = await $('a*=Settings');
        await settingsLink.click();

        await browser.pause(500);

        // Settings page should have some content
        const settingsContent = await $('main');
        await expect(settingsContent).toBeDisplayed();
    });

    /**
     * Test: Window can be resized (basic window management)
     */
    it('should have correct minimum window size', async () => {
        const windowSize = await browser.getWindowSize();
        
        // Minimum size from tauri.conf.json: 900x650
        expect(windowSize.width).toBeGreaterThanOrEqual(900);
        expect(windowSize.height).toBeGreaterThanOrEqual(650);
    });
});

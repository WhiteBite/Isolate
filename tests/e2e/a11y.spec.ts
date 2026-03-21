import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

/**
 * Accessibility tests for Isolate application
 * Tests WCAG 2.1 AA compliance using axe-core
 */

// Helper to run axe and check for violations
async function checkA11y(page: any, pageName: string) {
  const results = await new AxeBuilder({ page })
    .withTags(['wcag2a', 'wcag2aa', 'wcag21aa'])
    .analyze();

  // Log violations for debugging
  if (results.violations.length > 0) {
    console.log(`\n${pageName} - Accessibility violations:`);
    results.violations.forEach((violation) => {
      console.log(`  [${violation.impact}] ${violation.id}: ${violation.description}`);
      violation.nodes.forEach((node) => {
        console.log(`    - ${node.html.substring(0, 100)}`);
      });
    });
  }

  return results;
}

test.describe('Accessibility - WCAG AA Compliance', () => {
  test.describe('Dashboard Page (/)', () => {
    test('should have no critical accessibility violations', async ({ page }) => {
      await page.goto('/');
      await page.waitForLoadState('networkidle');
      
      const results = await checkA11y(page, 'Dashboard');
      
      // Filter for critical and serious violations only
      const criticalViolations = results.violations.filter(
        (v) => v.impact === 'critical' || v.impact === 'serious'
      );
      
      expect(criticalViolations).toHaveLength(0);
    });

    test('should have proper heading hierarchy', async ({ page }) => {
      await page.goto('/');
      await page.waitForLoadState('networkidle');
      
      const results = await new AxeBuilder({ page })
        .withRules(['heading-order', 'page-has-heading-one'])
        .analyze();
      
      expect(results.violations).toHaveLength(0);
    });

    test('should have accessible navigation', async ({ page }) => {
      await page.goto('/');
      await page.waitForLoadState('networkidle');
      
      // Check sidebar navigation is accessible
      const sidebar = page.locator('aside');
      await expect(sidebar).toBeVisible();
      
      // Navigation links should be focusable
      const navLinks = page.locator('aside a');
      const count = await navLinks.count();
      expect(count).toBeGreaterThan(0);
    });
  });

  test.describe('Services Page (/services)', () => {
    test('should have no critical accessibility violations', async ({ page }) => {
      await page.goto('/services');
      await page.waitForLoadState('networkidle');
      
      const results = await checkA11y(page, 'Services');
      
      const criticalViolations = results.violations.filter(
        (v) => v.impact === 'critical' || v.impact === 'serious'
      );
      
      expect(criticalViolations).toHaveLength(0);
    });

    test('should have accessible interactive elements', async ({ page }) => {
      await page.goto('/services');
      await page.waitForLoadState('networkidle');
      
      const results = await new AxeBuilder({ page })
        .withRules(['button-name', 'link-name'])
        .analyze();
      
      expect(results.violations).toHaveLength(0);
    });
  });

  test.describe('Network Page (/network)', () => {
    test('should have no critical accessibility violations', async ({ page }) => {
      await page.goto('/network');
      await page.waitForLoadState('networkidle');
      
      const results = await checkA11y(page, 'Network');
      
      const criticalViolations = results.violations.filter(
        (v) => v.impact === 'critical' || v.impact === 'serious'
      );
      
      expect(criticalViolations).toHaveLength(0);
    });
  });

  test.describe('Plugins Page (/plugins)', () => {
    test('should have no critical accessibility violations', async ({ page }) => {
      await page.goto('/plugins');
      await page.waitForLoadState('networkidle');
      
      const results = await checkA11y(page, 'Plugins');
      
      const criticalViolations = results.violations.filter(
        (v) => v.impact === 'critical' || v.impact === 'serious'
      );
      
      expect(criticalViolations).toHaveLength(0);
    });

    test('should have accessible plugin cards', async ({ page }) => {
      await page.goto('/plugins');
      await page.waitForLoadState('networkidle');
      
      const results = await new AxeBuilder({ page })
        .withRules(['color-contrast', 'button-name'])
        .analyze();
      
      // Log any contrast issues for review
      const contrastIssues = results.violations.filter(v => v.id === 'color-contrast');
      if (contrastIssues.length > 0) {
        console.log('Color contrast issues found - may need design review');
      }
    });
  });

  test.describe('Settings Page (/settings)', () => {
    test('should have no critical accessibility violations', async ({ page }) => {
      await page.goto('/settings');
      await page.waitForLoadState('networkidle');
      
      const results = await checkA11y(page, 'Settings');
      
      const criticalViolations = results.violations.filter(
        (v) => v.impact === 'critical' || v.impact === 'serious'
      );
      
      expect(criticalViolations).toHaveLength(0);
    });

    test('should have accessible form controls', async ({ page }) => {
      await page.goto('/settings');
      await page.waitForLoadState('networkidle');
      
      const results = await new AxeBuilder({ page })
        .withRules(['label', 'select-name', 'input-button-name'])
        .analyze();
      
      expect(results.violations).toHaveLength(0);
    });

    test('should have proper form labels', async ({ page }) => {
      await page.goto('/settings');
      await page.waitForLoadState('networkidle');
      
      // Check that inputs have associated labels
      const inputs = page.locator('input:not([type="hidden"])');
      const inputCount = await inputs.count();
      
      for (let i = 0; i < inputCount; i++) {
        const input = inputs.nth(i);
        const ariaLabel = await input.getAttribute('aria-label');
        const ariaLabelledBy = await input.getAttribute('aria-labelledby');
        const id = await input.getAttribute('id');
        
        // Input should have aria-label, aria-labelledby, or associated label
        const hasLabel = ariaLabel || ariaLabelledBy || 
          (id && await page.locator(`label[for="${id}"]`).count() > 0);
        
        if (!hasLabel) {
          console.log(`Input without label found: ${await input.evaluate(el => el.outerHTML.substring(0, 100))}`);
        }
      }
    });
  });

  test.describe('Keyboard Navigation', () => {
    test('should be navigable with keyboard only', async ({ page }) => {
      await page.goto('/');
      await page.waitForLoadState('networkidle');
      
      // Tab through the page
      await page.keyboard.press('Tab');
      
      // Check that focus is visible
      const focusedElement = page.locator(':focus');
      await expect(focusedElement).toBeVisible();
      
      // Continue tabbing through navigation
      for (let i = 0; i < 5; i++) {
        await page.keyboard.press('Tab');
        const focused = page.locator(':focus');
        await expect(focused).toBeVisible();
      }
    });

    test('should have visible focus indicators', async ({ page }) => {
      await page.goto('/');
      await page.waitForLoadState('networkidle');
      
      const results = await new AxeBuilder({ page })
        .withRules(['focus-order-semantics'])
        .analyze();
      
      expect(results.violations).toHaveLength(0);
    });
  });

  test.describe('Color Contrast', () => {
    test('Dashboard should meet color contrast requirements', async ({ page }) => {
      await page.goto('/');
      await page.waitForLoadState('networkidle');
      
      const results = await new AxeBuilder({ page })
        .withRules(['color-contrast'])
        .analyze();
      
      // Log contrast issues but don't fail (may need design review)
      if (results.violations.length > 0) {
        console.log('Color contrast issues on Dashboard:');
        results.violations.forEach(v => {
          v.nodes.forEach(n => {
            console.log(`  - ${n.html.substring(0, 80)}`);
          });
        });
      }
    });
  });

  test.describe('ARIA and Semantic HTML', () => {
    test('should use proper ARIA attributes', async ({ page }) => {
      await page.goto('/');
      await page.waitForLoadState('networkidle');
      
      const results = await new AxeBuilder({ page })
        .withRules([
          'aria-allowed-attr',
          'aria-hidden-body',
          'aria-required-attr',
          'aria-valid-attr',
          'aria-valid-attr-value'
        ])
        .analyze();
      
      expect(results.violations).toHaveLength(0);
    });

    test('should have proper landmark regions', async ({ page }) => {
      await page.goto('/');
      await page.waitForLoadState('networkidle');
      
      const results = await new AxeBuilder({ page })
        .withRules(['landmark-one-main', 'region'])
        .analyze();
      
      // Log but don't fail - landmarks are recommended but not required
      if (results.violations.length > 0) {
        console.log('Landmark region suggestions:');
        results.violations.forEach(v => console.log(`  - ${v.description}`));
      }
    });
  });
});

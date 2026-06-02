import { test, expect } from '@playwright/test';

// CSP violations are reported by the browser as console errors or via the
// SecurityPolicyViolationEvent. We capture both to produce a useful report.

test.describe('Content Security Policy', () => {
  test('no CSP violations on initial load', async ({ page }) => {
    const violations: string[] = [];

    // Chrome reports CSP violations as console errors
    page.on('console', msg => {
      if (
        msg.type() === 'error' &&
        (msg.text().toLowerCase().includes('content security policy') ||
          msg.text().toLowerCase().includes("refused to"))
      ) {
        violations.push(`[console] ${msg.text()}`);
      }
    });

    // Catch any uncaught page errors too
    page.on('pageerror', err => {
      if (
        err.message.toLowerCase().includes('content security policy') ||
        err.message.toLowerCase().includes('csp')
      ) {
        violations.push(`[pageerror] ${err.message}`);
      }
    });

    // Collect SecurityPolicyViolationEvent via injected listener
    await page.addInitScript(() => {
      (window as any).__cspViolations = [];
      document.addEventListener('securitypolicyviolation', (e) => {
        (window as any).__cspViolations.push(
          `${e.violatedDirective}: blocked '${e.blockedURI}'`
        );
      });
    });

    await page.goto('/');

    // Wait for the WASM grid to paint (rAF-driven canvas renderer)
    await expect(page.locator('canvas')).toBeVisible({ timeout: 15_000 });
    await page.waitForTimeout(1_000);

    // Retrieve in-page violations captured by the DOM event
    const domViolations: string[] = await page.evaluate(
      () => (window as any).__cspViolations ?? []
    );
    violations.push(...domViolations.map(v => `[dom-event] ${v}`));

    expect(
      violations,
      violations.length
        ? `CSP violations detected:\n${violations.map(v => `  • ${v}`).join('\n')}`
        : ''
    ).toHaveLength(0);
  });

  test('canvas is rendered under CSP', async ({ page }) => {
    await page.goto('/');
    const canvas = page.locator('canvas');
    await expect(canvas).toBeVisible({ timeout: 15_000 });

    // Verify canvas has non-zero dimensions — proves WASM executed correctly
    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();
    expect(box!.width).toBeGreaterThan(0);
    expect(box!.height).toBeGreaterThan(0);
  });
});

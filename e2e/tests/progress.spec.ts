import { test, expect, Page } from '@playwright/test';

// ── helpers ────────────────────────────────────────────────────────────────────

/** Wait for the rAF loop to paint at least one frame. */
async function waitForPaint(page: Page, ms = 300) {
  await page.waitForTimeout(ms);
}

// The "Completion" column (CellFormat::ProgressBar) lives just after the base
// columns, off-screen at the default scroll. Scroll right to bring it into
// view. The value-driven DaisyUI colours (error / warning / success) are
// produced by the registered class resolver.
const SCROLL_X = 1200;

// ── progress bar cell renderer ───────────────────────────────────────────────

test.describe('progress bar', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
  });

  test('scrolling to the Completion column does not crash', async ({
    page,
  }) => {
    const errors: string[] = [];
    page.on('pageerror', err => errors.push(err.message));
    const canvas = page.locator('canvas');
    await canvas.hover();
    await page.mouse.wheel(SCROLL_X, 0);
    await waitForPaint(page);
    await expect(canvas).toBeVisible();
    expect(errors).toHaveLength(0);
  });

  // Visual regression for the value-driven DaisyUI progress bars.
  // Tolerance: 2% of pixels may differ (antialiasing, font hinting).
  test('renders value-driven progress bars', async ({ page }) => {
    const canvas = page.locator('canvas');
    await canvas.hover();
    await page.mouse.wheel(SCROLL_X, 0);
    await waitForPaint(page);
    await expect(canvas).toHaveScreenshot('progress-bars.png', {
      maxDiffPixelRatio: 0.02,
    });
  });
});

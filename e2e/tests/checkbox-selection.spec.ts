import { test, expect, Page } from '@playwright/test';

// ── helpers ────────────────────────────────────────────────────────────────────

/** Wait for the rAF loop to paint at least one frame. */
async function waitForPaint(page: Page, ms = 300) {
  await page.waitForTimeout(ms);
}

// Approximate click-target geometry for this fixture (mirrors the
// GUTTER/HEADER convention in grid.spec.ts — good enough to land inside the
// intended zone, not exact pixel math).
const GUTTER = 55;
const HEADER = 60;
const ROW_H = 42;
const CHECKBOX_COL_W = 42; // GridModel::CHECKBOX_COLUMN_WIDTH

const checkboxToggle = (page: Page) =>
  page.locator('[data-testid="show-checkbox-column-toggle"]');

/** Enable the row-selection checkbox column via the e2e-only toggle. */
async function enableCheckboxColumn(page: Page) {
  await checkboxToggle(page).click();
  await waitForPaint(page);
}

// ── row-selection checkbox column ────────────────────────────────────────────

test.describe('row-selection checkboxes', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
  });

  test('enabling the checkbox column does not crash', async ({ page }) => {
    const errors: string[] = [];
    page.on('pageerror', err => errors.push(err.message));
    await enableCheckboxColumn(page);
    const canvas = page.locator('canvas');
    await expect(canvas).toBeVisible();
    expect(errors).toHaveLength(0);
  });

  test('clicking a row checkbox toggles it', async ({ page }) => {
    await enableCheckboxColumn(page);
    const canvas = page.locator('canvas');
    const rowCheckboxPos = {
      x: GUTTER + CHECKBOX_COL_W / 2,
      y: HEADER + ROW_H / 2,
    };

    await canvas.click({ position: rowCheckboxPos });
    await waitForPaint(page);
    await expect(canvas).toHaveScreenshot('checkbox-row-checked.png', {
      maxDiffPixelRatio: 0.02,
    });

    // Toggling again clears it.
    await canvas.click({ position: rowCheckboxPos });
    await waitForPaint(page);
    await expect(canvas).toBeVisible();
  });

  test('clicking the header checkbox checks every row', async ({ page }) => {
    await enableCheckboxColumn(page);
    const canvas = page.locator('canvas');

    await canvas.click({
      position: { x: GUTTER + CHECKBOX_COL_W / 2, y: HEADER / 2 },
    });
    await waitForPaint(page);
    await expect(canvas).toHaveScreenshot('checkbox-header-all-checked.png', {
      maxDiffPixelRatio: 0.02,
    });
  });

  test('header checkbox toggles back to unchecked (all rows visible, no filter)', async ({
    page,
  }) => {
    await enableCheckboxColumn(page);
    const canvas = page.locator('canvas');
    const headerPos = { x: GUTTER + CHECKBOX_COL_W / 2, y: HEADER / 2 };

    await canvas.click({ position: headerPos });
    await waitForPaint(page);
    await canvas.click({ position: headerPos });
    await waitForPaint(page);
    await expect(canvas).toHaveScreenshot('checkbox-header-all-unchecked.png', {
      maxDiffPixelRatio: 0.02,
    });
  });

  // The checkbox column is deliberately NOT pinned — it's the first column
  // of the scrollable region, so it scrolls away with the data on
  // horizontal scroll (unlike the row-number gutter, which stays fixed).
  test('checkbox column scrolls away with the data on horizontal scroll', async ({
    page,
  }) => {
    await enableCheckboxColumn(page);
    const canvas = page.locator('canvas');
    await canvas.hover();
    await page.mouse.wheel(500, 0);
    await waitForPaint(page);
    await expect(canvas).toHaveScreenshot('checkbox-column-scrolled-away.png', {
      maxDiffPixelRatio: 0.02,
    });
  });
});

import { test, expect, Page, Locator } from '@playwright/test';

// ── helpers ──────────────────────────────────────────────────────────────────

/** Wait for at least one rAF paint cycle. */
async function waitForPaint(page: Page, ms = 300) {
  await page.waitForTimeout(ms);
}

// Grid layout constants (must match build_model: row_height=40, header_height=60).
const GUTTER = 55;
const HEADER = 60;
const ROW_H = 40;

/** Return the center of cell (row, col) in canvas-relative coordinates. */
function cellCenter(
  row: number,
  col: number,
  colWidths: number[] = [200, 260, 140, 160, 120, 80, 60],
) {
  let x = GUTTER;
  for (let i = 0; i < col; i++) x += colWidths[i] ?? 120;
  x += (colWidths[col] ?? 120) / 2;
  const y = HEADER + row * ROW_H + ROW_H / 2;
  return { x, y };
}

/** Locate a control <select> by its label text inside .app-control. */
function controlSelect(page: Page, label: string): Locator {
  return page
    .locator('.app-control')
    .filter({ hasText: label })
    .locator('select');
}

/** Right-click a cell to open the context menu. */
async function openContextMenu(
  page: Page,
  canvas: Locator,
  row: number,
  col: number,
) {
  await canvas.click({ position: cellCenter(row, col), button: 'right' });
  await waitForPaint(page, 200);
}

/**
 * Locate the grid's inline edit input.
 * The grid creates a bare `<input>` (no type attribute) with
 * `position: fixed; z-index: 10000` — distinct from the app's
 * filter `<input type="text">`.
 */
function editInput(page: Page): Locator {
  return page.locator('input[style*="position: fixed"]');
}

// ── canvas inline style ─────────────────────────────────────────────────────

test.describe('canvas inline style', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
  });

  test('canvas has correct inline style attributes', async ({ page }) => {
    const canvas = page.locator('canvas');
    const style = await canvas.getAttribute('style');
    expect(style).toContain('width:');
    expect(style).toContain('height:');
    expect(style).toContain('display: block');
  });

  test('canvas has non-zero bounding dimensions', async ({ page }) => {
    const canvas = page.locator('canvas');
    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();
    expect(box!.width).toBeGreaterThan(100);
    expect(box!.height).toBeGreaterThan(100);
  });
});

// ── locale switching ────────────────────────────────────────────────────────

test.describe('locale switching', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
  });

  test('default locale shows English context menu items', async ({ page }) => {
    // Force English in case the browser default is something else.
    const langSelect = controlSelect(page, 'Language');
    await langSelect.selectOption('en');
    await waitForPaint(page);

    const canvas = page.locator('canvas');
    await canvas.click({ position: cellCenter(0, 0) });
    await waitForPaint(page, 100);
    await openContextMenu(page, canvas, 0, 0);

    const menu = page.locator('#rs-grid-ctx-menu');
    await expect(menu).toBeVisible({ timeout: 2000 });
    await expect(menu.getByText('Cut')).toBeVisible();
    await expect(menu.getByText('Copy', { exact: true })).toBeVisible();
    await expect(menu.getByText('Paste')).toBeVisible();

    await page.keyboard.press('Escape');
  });

  test('switching to French updates context menu labels', async ({ page }) => {
    const langSelect = controlSelect(page, 'Language');
    await langSelect.selectOption('fr');
    await waitForPaint(page);

    const canvas = page.locator('canvas');
    await canvas.click({ position: cellCenter(0, 0) });
    await waitForPaint(page, 100);
    await openContextMenu(page, canvas, 0, 0);

    const menu = page.locator('#rs-grid-ctx-menu');
    await expect(menu).toBeVisible({ timeout: 2000 });
    await expect(menu.getByText('Couper')).toBeVisible();
    await expect(menu.getByText('Copier', { exact: true })).toBeVisible();
    await expect(menu.getByText('Coller')).toBeVisible();

    await page.keyboard.press('Escape');
  });

  test('switching to German updates context menu labels', async ({ page }) => {
    const langSelect = controlSelect(page, 'Language');
    await langSelect.selectOption('de');
    await waitForPaint(page);

    const canvas = page.locator('canvas');
    await canvas.click({ position: cellCenter(0, 0) });
    await waitForPaint(page, 100);
    await openContextMenu(page, canvas, 0, 0);

    const menu = page.locator('#rs-grid-ctx-menu');
    await expect(menu).toBeVisible({ timeout: 2000 });
    await expect(menu.getByText('Ausschneiden')).toBeVisible();

    await page.keyboard.press('Escape');
  });

  test('switching locale back to English restores labels', async ({ page }) => {
    const langSelect = controlSelect(page, 'Language');

    // Switch to French
    await langSelect.selectOption('fr');
    await waitForPaint(page);

    // Switch back to English
    await langSelect.selectOption('en');
    await waitForPaint(page);

    const canvas = page.locator('canvas');
    await canvas.click({ position: cellCenter(0, 0) });
    await waitForPaint(page, 100);
    await openContextMenu(page, canvas, 0, 0);

    const menu = page.locator('#rs-grid-ctx-menu');
    await expect(menu).toBeVisible({ timeout: 2000 });
    await expect(menu.getByText('Cut')).toBeVisible();
    await expect(menu.getByText('Copy', { exact: true })).toBeVisible();

    await page.keyboard.press('Escape');
  });
});

// ── validation error callback ───────────────────────────────────────────────

test.describe('validation error callback', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
  });

  test('entering negative salary shows validation error', async ({ page }) => {
    const canvas = page.locator('canvas');
    // Salary is column index 4
    await canvas.dblclick({ position: cellCenter(0, 4) });
    await waitForPaint(page, 150);

    const input = editInput(page);
    await expect(input).toBeVisible({ timeout: 2000 });

    await input.fill('-100');
    await page.keyboard.press('Enter');
    await waitForPaint(page);

    const errorEl = page.locator('.app-validation-error');
    await expect(errorEl).toBeVisible({ timeout: 2000 });
    await expect(errorEl).toContainText('salary');
    await expect(errorEl).toContainText('positive number');
  });

  test('entering non-numeric salary shows validation error', async ({ page }) => {
    const canvas = page.locator('canvas');
    await canvas.dblclick({ position: cellCenter(0, 4) });
    await waitForPaint(page, 150);

    const input = editInput(page);
    await expect(input).toBeVisible({ timeout: 2000 });

    await input.fill('abc');
    await page.keyboard.press('Enter');
    await waitForPaint(page);

    const errorEl = page.locator('.app-validation-error');
    await expect(errorEl).toBeVisible({ timeout: 2000 });
    await expect(errorEl).toContainText('salary');
  });

  test('entering valid salary does not show validation error', async ({ page }) => {
    const canvas = page.locator('canvas');
    await canvas.dblclick({ position: cellCenter(0, 4) });
    await waitForPaint(page, 150);

    const input = editInput(page);
    await expect(input).toBeVisible({ timeout: 2000 });

    await input.fill('50000');
    await page.keyboard.press('Enter');
    await waitForPaint(page);

    // Fresh page, no prior validation error — div should not appear.
    const errorEl = page.locator('.app-validation-error');
    await expect(errorEl).not.toBeVisible();
  });

  test('error format includes column key and message', async ({ page }) => {
    const canvas = page.locator('canvas');
    await canvas.dblclick({ position: cellCenter(0, 4) });
    await waitForPaint(page, 150);

    const input = editInput(page);
    await input.fill('-5');
    await page.keyboard.press('Enter');
    await waitForPaint(page);

    const errorEl = page.locator('.app-validation-error');
    await expect(errorEl).toBeVisible();
    const text = await errorEl.textContent();
    expect(text).toMatch(/\[salary\].*Salary must be a positive number/);
  });

  test('zero salary is accepted (validator allows n >= 0)', async ({ page }) => {
    const canvas = page.locator('canvas');
    await canvas.dblclick({ position: cellCenter(0, 4) });
    await waitForPaint(page, 150);

    const input = editInput(page);
    await expect(input).toBeVisible({ timeout: 2000 });

    await input.fill('0');
    await page.keyboard.press('Enter');
    await waitForPaint(page);

    const errorEl = page.locator('.app-validation-error');
    await expect(errorEl).not.toBeVisible();
  });
});

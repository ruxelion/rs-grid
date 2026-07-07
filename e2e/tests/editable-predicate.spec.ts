import { test, expect, Page } from '@playwright/test';

// Layout constants — must match build_model in example-common/src/lib.rs.
// The "Notes" column is the last base column (11th), added specifically to
// exercise ColumnDef::editable_predicate (locked on even row indices).
const HEADER = 60;   // header height (px)
const ROW_H  = 40;   // row height (px)
const GUTTER = 60;   // row-number gutter width (px)

// Cumulative content-space width of every base column before "Notes"
// (name 200, email 260, role 140, dept 170, salary 120, active 80,
// status 120, avatar 60, actions 160, completion 160) = 1470.
const NOTES_ABS_START = 1470;
const NOTES_W = 160;

// Scroll far enough right to bring "Notes" comfortably into view
// (mirrors progress.spec.ts's SCROLL_X pattern for the "Completion"
// column, which lives just before "Notes").
const SCROLL_X = 1400;
const NOTES_X = GUTTER + (NOTES_ABS_START + NOTES_W / 2) - SCROLL_X; // 210

const LOCKED_ROW_Y   = HEADER + ROW_H / 2;              // row 0 (even → locked)
const UNLOCKED_ROW_Y = HEADER + ROW_H + ROW_H / 2;      // row 1 (odd → unlocked)

async function waitForPaint(page: Page, ms = 300) {
  await page.waitForTimeout(ms);
}

async function scrollToNotes(page: Page) {
  const canvas = page.locator('canvas');
  await canvas.hover();
  await page.mouse.wheel(SCROLL_X, 0);
  await waitForPaint(page);
  return canvas;
}

test.describe('editable predicate (per-cell locking)', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
  });

  test('dblclick on an unlocked (odd row) Notes cell opens the text input', async ({ page }) => {
    const canvas = await scrollToNotes(page);
    await canvas.dblclick({ position: { x: NOTES_X, y: UNLOCKED_ROW_Y } });
    await waitForPaint(page, 400);
    await expect(page.locator('input, textarea')).toHaveCount(1);
  });

  test('dblclick on a locked (even row) Notes cell does not open the text input', async ({ page }) => {
    const canvas = await scrollToNotes(page);
    await canvas.dblclick({ position: { x: NOTES_X, y: LOCKED_ROW_Y } });
    await waitForPaint(page, 400);
    await expect(page.locator('input, textarea')).toHaveCount(0);
    await expect(canvas).toBeVisible();
  });

  test('hovering a locked cell shows the not-allowed cursor', async ({ page }) => {
    const canvas = await scrollToNotes(page);
    await canvas.hover({ position: { x: NOTES_X, y: LOCKED_ROW_Y } });
    await waitForPaint(page, 200);
    const cursor = await canvas.evaluate((el) => getComputedStyle(el).cursor);
    expect(cursor).toBe('not-allowed');
  });

  test('hovering an unlocked cell shows the default cursor', async ({ page }) => {
    const canvas = await scrollToNotes(page);
    await canvas.hover({ position: { x: NOTES_X, y: UNLOCKED_ROW_Y } });
    await waitForPaint(page, 200);
    const cursor = await canvas.evaluate((el) => getComputedStyle(el).cursor);
    expect(cursor).toBe('default');
  });
});

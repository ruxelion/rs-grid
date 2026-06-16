import { test, expect, Page } from '@playwright/test';

// Layout constants — must match build_model in example-common/src/lib.rs
// and grid.spec.ts.
//
// For 1 000 rows (fixture default):
//   row_number_width = floor(log10(1000)) + 1 digits * 9 + 24 = 4*9+24 = 60
const GUTTER = 60;   // row-number gutter width (px)
const HEADER = 60;   // header height (px), from build_model: header_height=60
const ROW_H  = 40;   // row height (px), from build_model: row_height=40

// Column widths — must match build_model base column definitions.
const NAME_W  = 200; // "Name"  column
const EMAIL_W = 260; // "Email" column
const ROLE_W  = 140; // "Role"  column

// Centers of first-row cells.
const NAME_X  = GUTTER + NAME_W / 2;                          // 160
const NAME_Y  = HEADER + ROW_H / 2;                           //  80
const ROLE_X  = GUTTER + NAME_W + EMAIL_W + ROLE_W / 2;       // 590
const ROLE_Y  = NAME_Y;                                        //  80

async function waitForPaint(page: Page, ms = 300) {
  await page.waitForTimeout(ms);
}

test.describe('editing', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
  });

  // ── PR fix: column with CellEditor::Text opens a text <input> ────────────
  test('dblclick on Name cell (CellEditor::Text) opens text input', async ({ page }) => {
    const canvas = page.locator('canvas');
    await canvas.dblclick({ position: { x: NAME_X, y: NAME_Y } });
    await waitForPaint(page, 400);

    const input = page.locator('input[type="text"]');
    await expect(input).toBeVisible();
  });

  // ── PR fix: column with editor=None must NOT open any input ──────────────
  // Regression guard for the bug fixed in rs-grid-web show_edit_input():
  // the old `_ =>` catch-all opened a text <input> even when column.editor
  // was None. The fix dispatches CancelEdit and returns without creating
  // any DOM overlay.
  test('dblclick on Role cell (editor=None) does not open any input', async ({ page }) => {
    const canvas = page.locator('canvas');
    await canvas.dblclick({ position: { x: ROLE_X, y: ROLE_Y } });
    await waitForPaint(page, 400);

    // No <input> of any type should exist in the DOM.
    await expect(page.locator('input')).toHaveCount(0);
    // Canvas must still be visible — no crash.
    await expect(canvas).toBeVisible();
  });

  // ── Escape key closes the editor ─────────────────────────────────────────
  test('Escape closes the text editor', async ({ page }) => {
    const canvas = page.locator('canvas');
    await canvas.dblclick({ position: { x: NAME_X, y: NAME_Y } });
    await waitForPaint(page, 400);

    await expect(page.locator('input[type="text"]')).toBeVisible();

    await page.keyboard.press('Escape');
    await waitForPaint(page, 200);

    await expect(page.locator('input[type="text"]')).toHaveCount(0);
    await expect(canvas).toBeVisible();
  });
});

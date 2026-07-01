import { test, expect, Page } from '@playwright/test';

// Layout constants — must match build_model in example-common/src/lib.rs
// and grid.spec.ts / editing.spec.ts.
const GUTTER = 60;   // row-number gutter width (px)
const HEADER = 60;   // header height (px)
const ROW_H  = 40;   // row height (px)

const NAME_W = 200; // "Name" column — has CellEditor::Text + required()

const NAME_X = GUTTER + NAME_W / 2; // 160
const NAME_Y = HEADER + ROW_H / 2;  //  80

async function waitForPaint(page: Page, ms = 300) {
  await page.waitForTimeout(ms);
}

test.describe('validation', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
  });

  test('typing an empty value into a required cell shows the invalid style', async ({ page }) => {
    const canvas = page.locator('canvas');
    await canvas.dblclick({ position: { x: NAME_X, y: NAME_Y } });
    await waitForPaint(page, 400);

    const input = page.locator('input[type="text"]');
    await expect(input).toBeVisible();

    // Clear the field — "required" rule fails on every keystroke.
    await input.fill('');
    await waitForPaint(page, 200);

    const border = await input.evaluate(
      (el) => getComputedStyle(el).borderColor,
    );
    // Default fallback --rs-grid-editor-border-invalid: #dc2626
    expect(border).toBe('rgb(220, 38, 38)');
  });

  test('typing a valid value clears the invalid style', async ({ page }) => {
    const canvas = page.locator('canvas');
    await canvas.dblclick({ position: { x: NAME_X, y: NAME_Y } });
    await waitForPaint(page, 400);

    const input = page.locator('input[type="text"]');
    await input.fill('');
    await waitForPaint(page, 200);
    await input.fill('Someone');
    await waitForPaint(page, 200);

    const border = await input.evaluate(
      (el) => getComputedStyle(el).borderColor,
    );
    // Default fallback --rs-grid-editor-border: #2563eb
    expect(border).toBe('rgb(37, 99, 235)');
  });

  test('Enter with an invalid value reverts and closes the editor (default Revert mode)', async ({ page }) => {
    const canvas = page.locator('canvas');
    await canvas.dblclick({ position: { x: NAME_X, y: NAME_Y } });
    await waitForPaint(page, 400);

    const input = page.locator('input[type="text"]');
    await input.fill('');
    await page.keyboard.press('Enter');
    await waitForPaint(page, 300);

    // The editor closed even though the value was invalid.
    await expect(page.locator('input[type="text"]')).toHaveCount(0);
    await expect(canvas).toBeVisible();
  });

  // ── native title fallback ────────────────────────────────────────────────
  test('invalid value sets a native title attribute, valid value clears it', async ({ page }) => {
    const canvas = page.locator('canvas');
    await canvas.dblclick({ position: { x: NAME_X, y: NAME_Y } });
    await waitForPaint(page, 400);

    const input = page.locator('input[type="text"]');
    await input.fill('');
    await waitForPaint(page, 200);
    await expect(input).toHaveAttribute('title', 'This field is required.');

    await input.fill('Someone');
    await waitForPaint(page, 200);
    await expect(input).not.toHaveAttribute('title');
  });

  // ── generic API: on_validation_state_changed (live, per-keystroke) ───────
  test('on_validation_state_changed fires live, not just on commit', async ({ page }) => {
    const canvas = page.locator('canvas');
    const state = page.locator('[data-testid="validation-state"]');

    // No active edit yet — empty.
    await expect(state).toHaveText('');

    await canvas.dblclick({ position: { x: NAME_X, y: NAME_Y } });
    await waitForPaint(page, 400);

    const input = page.locator('input[type="text"]');
    await input.fill('');
    await waitForPaint(page, 200);
    // Fired on the keystroke itself, well before any commit attempt.
    await expect(state).toHaveText('This field is required.');

    await input.fill('Someone');
    await waitForPaint(page, 200);
    await expect(state).toHaveText('');

    await page.keyboard.press('Escape');
    await waitForPaint(page, 200);
    await expect(state).toHaveText('');
  });
});

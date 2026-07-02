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

  // ── at-rest hover tooltip (set_validation_tooltip_class) ─────────────────
  test.describe('at-rest validation tooltip', () => {
    // Row 10's "name" is seeded empty (invalid, required()) by the fixture,
    // without ever going through CommitEdit/PasteAt — simulates data
    // loaded already-invalid from an external source. Row 0 is left alone
    // since other specs dblclick it for edit-flow tests.
    const INVALID_X = NAME_X;
    const INVALID_Y = HEADER + ROW_H * 10 + ROW_H / 2; // row 10, seeded invalid
    const VALID_Y = NAME_Y; // row 0, untouched / valid

    function tooltip(page: Page) {
      return page.locator('div[data-tip]');
    }

    test('hovering the seeded at-rest-invalid cell shows the tooltip', async ({ page }) => {
      const canvas = page.locator('canvas');
      await canvas.hover({ position: { x: INVALID_X, y: INVALID_Y } });
      await waitForPaint(page, 200);

      const tip = tooltip(page);
      await expect(tip).toHaveAttribute('data-tip', 'This field is required.');
      await expect(tip).toHaveClass('tooltip tooltip-open tooltip-error');
      await expect(tip).toBeVisible();
    });

    test('moving off the invalid cell hides the tooltip', async ({ page }) => {
      const canvas = page.locator('canvas');
      await canvas.hover({ position: { x: INVALID_X, y: INVALID_Y } });
      await waitForPaint(page, 200);
      await expect(tooltip(page)).toBeVisible();

      await canvas.hover({ position: { x: INVALID_X, y: VALID_Y } });
      await waitForPaint(page, 200);
      await expect(tooltip(page)).toBeHidden();
    });

    test('hovering a valid cell never shows the tooltip', async ({ page }) => {
      const canvas = page.locator('canvas');
      await canvas.hover({ position: { x: INVALID_X, y: VALID_Y } });
      await waitForPaint(page, 200);
      await expect(tooltip(page)).toBeHidden();
    });

    test('scrolling while hovering hides the tooltip', async ({ page }) => {
      const canvas = page.locator('canvas');
      await canvas.hover({ position: { x: INVALID_X, y: INVALID_Y } });
      await waitForPaint(page, 200);
      await expect(tooltip(page)).toBeVisible();

      await canvas.dispatchEvent('wheel', { deltaY: 200 });
      await waitForPaint(page, 200);
      await expect(tooltip(page)).toBeHidden();
    });
  });
});

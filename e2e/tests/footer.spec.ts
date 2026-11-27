import { test, expect, Page } from '@playwright/test';

// Footer bar + pagination. The harness starts with both OFF under
// Playwright (navigator.webdriver), so every other spec's pixel baseline is
// untouched; these tests switch them on with the two toggle buttons.
//
// The footer is canvas-drawn, so interaction uses fixed coordinates derived
// from the layout in `rs-grid-scene/src/builder/footer.rs`: controls are
// vertically centered in the footer band (`footer_height` = 48px), and the
// page strip is right-aligned with a 12px margin, 32px square buttons:
//   [<<][<][1][2]…[N][>][>>]  →  `>>` centre = width − 28, `>` = width − 60.

test.use({ locale: 'en-US' });

const FOOTER_H = 48;
const waitForPaint = (page: Page, ms = 300) => page.waitForTimeout(ms);

const pageState = (page: Page) =>
  page.locator('[data-testid="page-state"]').textContent();

async function enable(page: Page) {
  await page.locator('[data-testid="show-footer-toggle"]').click();
  await page.locator('[data-testid="pagination-toggle"]').click();
  await waitForPaint(page);
}

/** Click at `x` px from the canvas' right edge, in the footer's middle. */
async function clickFooterFromRight(page: Page, fromRight: number) {
  const box = (await page.locator('canvas').boundingBox())!;
  await page.mouse.click(
    box.x + box.width - fromRight,
    box.y + box.height - FOOTER_H / 2,
  );
  await waitForPaint(page);
}

test.describe('footer & pagination', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page, 500);
  });

  test('footer and pagination are off by default', async ({ page }) => {
    await expect(page.locator('[data-testid="show-footer-toggle"]')).toHaveText(
      'Footer: off',
    );
    expect(await pageState(page)).toBe('');
    // Nothing in the footer band reacts while it is off.
    await clickFooterFromRight(page, 60);
    expect(await pageState(page)).toBe('');
  });

  test('next page button moves to page 2', async ({ page }) => {
    await enable(page);
    await clickFooterFromRight(page, 60); // `>`
    expect(await pageState(page)).toBe('2/25');
  });

  test('last page button jumps to the final page', async ({ page }) => {
    await enable(page);
    await clickFooterFromRight(page, 28); // `>>`
    // 1 000 rows at 25 per page.
    expect(await pageState(page)).toBe('40/25');
  });

  test('page-size menu changes the rows per page', async ({ page }) => {
    await enable(page);
    const box = (await page.locator('canvas').boundingBox())!;
    const y = box.y + box.height - FOOTER_H / 2;
    // The selector sits after the count labels, whose widths depend on
    // the text; sweep that stretch until its menu opens.
    const menu = page.locator('#rs-grid-ctx-menu');
    for (let x = box.x + 250; x < box.x + 520; x += 10) {
      await page.mouse.click(x, y);
      await waitForPaint(page, 120);
      if (await menu.count()) break;
    }
    await expect(menu).toBeVisible();
    await menu.getByText('50', { exact: true }).click();
    await waitForPaint(page);
    expect(await pageState(page)).toBe('1/50');
  });

  test('clicks in the footer never select a data cell', async ({ page }) => {
    await enable(page);
    const errors: string[] = [];
    page.on('pageerror', (e) => errors.push(String(e)));
    // Dead space between the labels and the page strip.
    const box = (await page.locator('canvas').boundingBox())!;
    await page.mouse.click(
      box.x + box.width / 2 + 40,
      box.y + box.height - FOOTER_H / 2,
    );
    await waitForPaint(page);
    expect(errors).toEqual([]);
    expect(await pageState(page)).toBe('');
  });

  test('turning pagination off restores continuous scrolling', async ({
    page,
  }) => {
    await enable(page);
    await page.locator('[data-testid="pagination-toggle"]').click();
    await waitForPaint(page);
    // The page strip is gone: the same spot no longer pages.
    await clickFooterFromRight(page, 60);
    expect(await pageState(page)).toBe('');
  });
});

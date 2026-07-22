import { test, expect, Page } from '@playwright/test';

// ── helpers ──────────────────────────────────────────────────────────────────

/** Wait for the rAF loop to paint at least one frame. */
async function waitForPaint(page: Page, ms = 300) {
  await page.waitForTimeout(ms);
}

// Default fixture: 20 columns, first is "name" (Text editor, required,
// width 200), "salary" is the 5th column (Currency-formatted, numeric-like).
// header_height = 40 (the harness's own default header-height <select>,
// e2e/leptos-harness/src/lib.rs), row_height = 40, and the row-number
// gutter is 60px wide at the default 1000-row count (4 digits × 9px +
// 24px padding).
//
// The column header itself has **no** filter icon — only a column name
// and the "⋮" menu icon (AG-Grid-style: the header stays plain, the
// floating filter row is the only click path to the advanced
// condition/checklist popup, via its own mini funnel icon). So every
// test that needs the popup must enable the filter row first
// (`enableFilterRow`) and click that row's icon, not the header.
const HEADER_H = 40;
const GUTTER = 60;
const NAME_COL_WIDTH = 200;
const EMAIL_COL_OFFSET = 200;
const EMAIL_COL_WIDTH = 260;
// name(200) + email(260) = 460 before role.
const ROLE_COL_OFFSET = 460;
const ROLE_COL_WIDTH = 140;
// name(200) + email(260) + role(140) + dept(170) = 770 before salary.
const SALARY_COL_OFFSET = 770;
const SALARY_COL_WIDTH = 120;

// ── floating filter row ──────────────────────────────────────────────────
// `GridModel::filter_row_height` default (rs-grid-scene's `Theme::light()`
// et al) — the harness doesn't override it via a CSS var (unlike
// `--rs-grid-header-height`), so the plain Rust default applies.
const FILTER_ROW_H = 36;

function filterRowCellCenter(colOffset: number, colWidth: number) {
  return {
    x: GUTTER + colOffset + colWidth / 2,
    y: HEADER_H + FILTER_ROW_H / 2,
  };
}

// The filter row's mini funnel icon sits flush at the column's right
// edge — no competing "⋮" menu icon down here, so it only accounts for
// its own margin/width (`header_filter_icon_margin_r`(6)/`_btn_w`(20),
// still read from `Theme::header_filter_icon_*` even though the header
// itself no longer draws this icon).
function filterRowIconX(colRightVx: number): number {
  const btnRx = colRightVx - 6;
  const btnLx = btnRx - 20;
  return (btnLx + btnRx) / 2;
}

function filterRowIcon(colOffset: number, colWidth: number) {
  return {
    x: filterRowIconX(GUTTER + colOffset + colWidth),
    y: HEADER_H + FILTER_ROW_H / 2,
  };
}

const NAME_FILTER_ROW_ICON = filterRowIcon(0, NAME_COL_WIDTH);
const EMAIL_FILTER_ROW_ICON = filterRowIcon(EMAIL_COL_OFFSET, EMAIL_COL_WIDTH);
// "role" is a small, deterministic, hash-generated enum (Account
// Executive, CEO, CFO, ... ~21 distinct values across the 1000-row
// fixture) — low-cardinality, so it's a fast/light checklist to test
// against (the value checklist itself has no cap on distinct values —
// see `MAX_VALUE_FILTER_OPTIONS` in `filter_popup.rs`).
const ROLE_FILTER_ROW_ICON = filterRowIcon(ROLE_COL_OFFSET, ROLE_COL_WIDTH);
const SALARY_FILTER_ROW_ICON = filterRowIcon(
  SALARY_COL_OFFSET,
  SALARY_COL_WIDTH,
);

const ROLE_FILTER_ROW_CELL = filterRowCellCenter(
  ROLE_COL_OFFSET,
  ROLE_COL_WIDTH,
);

const popup = (page: Page) => page.locator('#rs-grid-ctx-menu');
// The operator control is a custom combobox (trigger div + a
// role="option" list), not a native <select> — see filter_popup.rs's
// "operator combobox" comment for why (daisyUI-pixel-perfect on every
// browser, since a native <select>'s open popup has no reliable
// cross-browser CSS hook).
const opTrigger = (page: Page) => popup(page).getByRole('combobox');
const opOption = (page: Page, label: string) =>
  popup(page).getByRole('option', { name: label, exact: true });
const valueInput = (page: Page) => popup(page).locator('input').first();
// The operator select + value input live behind a collapsed-by-default
// "Text Filter" disclosure row (AG-Grid-style flyout) — see
// filter_popup.rs's "Text Filter row + condition-editor flyout" comment.
const textFilterRow = (page: Page) =>
  popup(page).getByRole('button', { name: 'Text Filter' });
const applyBtn = (page: Page) =>
  popup(page).getByText('Apply', { exact: true });
const clearBtn = (page: Page) =>
  popup(page).getByText('Clear Filter', { exact: true });
const searchInput = (page: Page) =>
  popup(page).locator('input[placeholder="Search..."]');
const selectAllCheckbox = (page: Page) =>
  popup(page).getByRole('checkbox', { name: '(Select All)', exact: true });
const valueCheckbox = (page: Page, value: string) =>
  popup(page).getByRole('checkbox', { name: value, exact: true });

async function openFilterPopup(
  page: Page,
  icon: { x: number; y: number },
) {
  await page.locator('canvas').click({ position: icon });
  await waitForPaint(page);
}

/**
 * Open the popup, then expand the "Text Filter" row — for tests that
 * need the operator select / value input, which start hidden.
 */
async function openConditionEditor(
  page: Page,
  icon: { x: number; y: number },
) {
  await openFilterPopup(page, icon);
  await textFilterRow(page).click();
}

/** Open the operator dropdown and click the option with this label. */
async function selectOp(page: Page, label: string) {
  await opTrigger(page).click();
  await opOption(page, label).click();
}

// ── floating filter row helpers ──────────────────────────────────────────
// The quick-filter `<input>` (quick_filter.rs) is a transient overlay
// appended directly to `<body>` — same lifecycle/parent as the inline
// cell editor, and never nested inside `#rs-grid-ctx-menu` (the popup),
// so `body > input` unambiguously targets it (a cell edit is never open
// at the same time in these tests).
const quickFilterInput = (page: Page) => page.locator('body > input');
const filterRowToggle = (page: Page) =>
  page.getByTestId('show-filter-row-toggle');

async function enableFilterRow(page: Page) {
  await filterRowToggle(page).click();
  await waitForPaint(page);
}

test.describe('column filter popup', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
    // The popup's only click path is the floating filter row's own
    // funnel icon — the header itself has none.
    await enableFilterRow(page);
  });

  test('is absent until a filter icon is clicked', async ({ page }) => {
    await expect(popup(page)).toHaveCount(0);
  });

  test('the condition editor starts collapsed behind a "Text Filter" row', async ({
    page,
  }) => {
    await openFilterPopup(page, NAME_FILTER_ROW_ICON);
    await expect(textFilterRow(page)).toBeVisible();
    await expect(opTrigger(page)).toBeHidden();
    await expect(valueInput(page)).toBeHidden();
    await textFilterRow(page).click();
    await expect(opTrigger(page)).toBeVisible();
    await expect(valueInput(page)).toBeVisible();
  });

  test('clicking a filter icon opens a popup; expanding Text Filter reveals the operator select and value input', async ({
    page,
  }) => {
    const errors: string[] = [];
    page.on('pageerror', err => errors.push(err.message));

    await openConditionEditor(page, NAME_FILTER_ROW_ICON);
    await expect(popup(page)).toBeVisible();
    await expect(opTrigger(page)).toHaveText('Contains');
    await expect(valueInput(page)).toHaveValue('');
    // 12 FilterOp variants.
    await opTrigger(page).click();
    await expect(popup(page).getByRole('option')).toHaveCount(12);
    expect(errors).toHaveLength(0);
  });

  test('Apply filters and closes the popup; reopening shows the applied condition', async ({
    page,
  }) => {
    await openConditionEditor(page, NAME_FILTER_ROW_ICON);
    await valueInput(page).fill('zzz-not-a-real-name-zzz');
    await applyBtn(page).click();
    await waitForPaint(page);
    await expect(popup(page)).toHaveCount(0);

    await openConditionEditor(page, NAME_FILTER_ROW_ICON);
    await expect(opTrigger(page)).toHaveText('Contains');
    await expect(valueInput(page)).toHaveValue('zzz-not-a-real-name-zzz');
  });

  test('a numeric operator on the Currency-formatted salary column round-trips', async ({
    page,
  }) => {
    await openConditionEditor(page, SALARY_FILTER_ROW_ICON);
    await selectOp(page, 'Greater than');
    await valueInput(page).fill('50000');
    await applyBtn(page).click();
    await waitForPaint(page);

    await openConditionEditor(page, SALARY_FILTER_ROW_ICON);
    await expect(opTrigger(page)).toHaveText('Greater than');
    await expect(valueInput(page)).toHaveValue('50000');
  });

  test('selecting Blank hides the value input', async ({ page }) => {
    await openConditionEditor(page, NAME_FILTER_ROW_ICON);
    await expect(valueInput(page)).toBeVisible();
    await selectOp(page, 'Is blank');
    await expect(valueInput(page)).toBeHidden();
  });

  test('the operator dropdown opens on click and closes on outside click', async ({
    page,
  }) => {
    await openConditionEditor(page, NAME_FILTER_ROW_ICON);
    await expect(opOption(page, 'Contains')).toBeHidden();
    await opTrigger(page).click();
    await expect(opOption(page, 'Contains')).toBeVisible();
    // The heading, above the trigger, is never covered by the open
    // dropdown — unlike the value input, which sits right behind it.
    await popup(page).click({ position: { x: 5, y: 5 } });
    await expect(opOption(page, 'Contains')).toBeHidden();
    // Closing the dropdown must not also close the popup.
    await expect(popup(page)).toBeVisible();
  });

  test('Escape closes the operator dropdown first, then the popup on a second press', async ({
    page,
  }) => {
    await openConditionEditor(page, NAME_FILTER_ROW_ICON);
    await opTrigger(page).click();
    await expect(opOption(page, 'Contains')).toBeVisible();
    await page.keyboard.press('Escape');
    await expect(opOption(page, 'Contains')).toBeHidden();
    await expect(popup(page)).toBeVisible();
    await page.keyboard.press('Escape');
    await expect(popup(page)).toHaveCount(0);
  });

  test('Clear resets the filter and closes the popup', async ({ page }) => {
    await openConditionEditor(page, NAME_FILTER_ROW_ICON);
    await valueInput(page).fill('abc');
    await applyBtn(page).click();
    await waitForPaint(page);

    await openFilterPopup(page, NAME_FILTER_ROW_ICON);
    await clearBtn(page).click();
    await waitForPaint(page);
    await expect(popup(page)).toHaveCount(0);

    await openConditionEditor(page, NAME_FILTER_ROW_ICON);
    await expect(opTrigger(page)).toHaveText('Contains');
    await expect(valueInput(page)).toHaveValue('');
  });

  test('clicking the backdrop closes the popup without applying', async ({
    page,
  }) => {
    await openConditionEditor(page, NAME_FILTER_ROW_ICON);
    await valueInput(page).fill('should not apply');
    await page.locator('#rs-grid-ctx-backdrop').click({
      position: { x: 5, y: 5 },
    });
    await waitForPaint(page);
    await expect(popup(page)).toHaveCount(0);

    await openConditionEditor(page, NAME_FILTER_ROW_ICON);
    await expect(valueInput(page)).toHaveValue('');
  });

  test('Escape closes the popup', async ({ page }) => {
    await openFilterPopup(page, NAME_FILTER_ROW_ICON);
    await page.keyboard.press('Escape');
    await waitForPaint(page);
    await expect(popup(page)).toHaveCount(0);
  });

  test('header context menu Clear Filter clears a filter set via the popup', async ({
    page,
  }) => {
    await openConditionEditor(page, NAME_FILTER_ROW_ICON);
    await valueInput(page).fill('abc');
    await applyBtn(page).click();
    await waitForPaint(page);

    // Right-click the "Name" column header to open the context menu.
    await page
      .locator('canvas')
      .click({ position: { x: 100, y: 30 }, button: 'right' });
    await waitForPaint(page);

    const clearFilterItem = page.getByText('Clear Filter', { exact: true });
    await expect(clearFilterItem).toBeVisible();
    await clearFilterItem.click();
    await waitForPaint(page);

    await openConditionEditor(page, NAME_FILTER_ROW_ICON);
    await expect(valueInput(page)).toHaveValue('');
  });

  // ── value checklist ("Set Filter") ───────────────────────────────────

  test('a low-cardinality column renders the checklist, all values checked by default', async ({
    page,
  }) => {
    await openFilterPopup(page, ROLE_FILTER_ROW_ICON);
    await expect(searchInput(page)).toBeVisible();
    await expect(selectAllCheckbox(page)).toBeChecked();
    await expect(valueCheckbox(page, 'CEO')).toBeVisible();
    await expect(valueCheckbox(page, 'CEO')).toBeChecked();
  });

  test('unchecking a value and Apply persists across reopening', async ({
    page,
  }) => {
    await openFilterPopup(page, ROLE_FILTER_ROW_ICON);
    await valueCheckbox(page, 'CEO').uncheck();
    await applyBtn(page).click();
    await waitForPaint(page);
    await expect(popup(page)).toHaveCount(0);

    await openFilterPopup(page, ROLE_FILTER_ROW_ICON);
    await expect(valueCheckbox(page, 'CEO')).not.toBeChecked();
    // Every other value is still checked.
    await expect(valueCheckbox(page, 'CFO')).toBeChecked();
    // Unchecking one value (not all) leaves Select All indeterminate,
    // not unchecked.
    await expect(selectAllCheckbox(page)).not.toBeChecked();
  });

  test('rechecking every value clears the value filter (Select All round-trip)', async ({
    page,
  }) => {
    await openFilterPopup(page, ROLE_FILTER_ROW_ICON);
    await valueCheckbox(page, 'CEO').uncheck();
    await applyBtn(page).click();
    await waitForPaint(page);

    await openFilterPopup(page, ROLE_FILTER_ROW_ICON);
    await expect(valueCheckbox(page, 'CEO')).not.toBeChecked();
    await valueCheckbox(page, 'CEO').check();
    await applyBtn(page).click();
    await waitForPaint(page);

    await openFilterPopup(page, ROLE_FILTER_ROW_ICON);
    await expect(valueCheckbox(page, 'CEO')).toBeChecked();
    await expect(selectAllCheckbox(page)).toBeChecked();
  });

  test('Select All unchecks and rechecks every visible value', async ({
    page,
  }) => {
    await openFilterPopup(page, ROLE_FILTER_ROW_ICON);
    await selectAllCheckbox(page).uncheck();
    await expect(valueCheckbox(page, 'CEO')).not.toBeChecked();
    await expect(valueCheckbox(page, 'CFO')).not.toBeChecked();

    await selectAllCheckbox(page).check();
    await expect(valueCheckbox(page, 'CEO')).toBeChecked();
    await expect(valueCheckbox(page, 'CFO')).toBeChecked();
  });

  test('search hides non-matching values without discarding their checked state', async ({
    page,
  }) => {
    await openFilterPopup(page, ROLE_FILTER_ROW_ICON);
    await valueCheckbox(page, 'CEO').uncheck();
    await searchInput(page).fill('CFO');
    await expect(valueCheckbox(page, 'CFO')).toBeVisible();
    await expect(valueCheckbox(page, 'CEO')).toBeHidden();

    await searchInput(page).fill('');
    await expect(valueCheckbox(page, 'CEO')).toBeVisible();
    // Still unchecked — the search box only hid it, it never cleared it.
    await expect(valueCheckbox(page, 'CEO')).not.toBeChecked();
  });

  test('a high-cardinality column still renders a full checklist (no cap)', async ({
    page,
  }) => {
    // `MAX_VALUE_FILTER_OPTIONS` is effectively unbounded — even a
    // near-unique-per-row column like Email gets a real checklist,
    // not the "Too many distinct values" fallback message.
    await openFilterPopup(page, EMAIL_FILTER_ROW_ICON);
    await expect(searchInput(page)).toBeVisible();
    await expect(
      popup(page).getByText('Too many distinct values', { exact: false }),
    ).toHaveCount(0);
    const count = await popup(page).getByRole('checkbox').count();
    // (Select All) + one per distinct email — 1000 rows in the default
    // fixture, comfortably more than the old 200 cap.
    expect(count).toBeGreaterThan(200);
  });

  test('Clear resets a value filter set via the checklist', async ({
    page,
  }) => {
    await openFilterPopup(page, ROLE_FILTER_ROW_ICON);
    await valueCheckbox(page, 'CEO').uncheck();
    await applyBtn(page).click();
    await waitForPaint(page);

    await openFilterPopup(page, ROLE_FILTER_ROW_ICON);
    await clearBtn(page).click();
    await waitForPaint(page);

    await openFilterPopup(page, ROLE_FILTER_ROW_ICON);
    await expect(valueCheckbox(page, 'CEO')).toBeChecked();
    await expect(selectAllCheckbox(page)).toBeChecked();
  });

  test('header context menu Clear Filter also clears a checklist-only value filter', async ({
    page,
  }) => {
    await openFilterPopup(page, ROLE_FILTER_ROW_ICON);
    await valueCheckbox(page, 'CEO').uncheck();
    await applyBtn(page).click();
    await waitForPaint(page);

    // Right-click the "Role" column header to open the context menu.
    const roleHeaderX = GUTTER + ROLE_COL_OFFSET + ROLE_COL_WIDTH / 2;
    await page
      .locator('canvas')
      .click({ position: { x: roleHeaderX, y: HEADER_H / 2 }, button: 'right' });
    await waitForPaint(page);

    const clearFilterItem = page.getByText('Clear Filter', { exact: true });
    await expect(clearFilterItem).toBeVisible();
    await clearFilterItem.click();
    await waitForPaint(page);

    await openFilterPopup(page, ROLE_FILTER_ROW_ICON);
    await expect(valueCheckbox(page, 'CEO')).toBeChecked();
  });
});

test.describe('floating filter row', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
  });

  test('is hidden until the toggle is enabled', async ({ page }) => {
    await expect(quickFilterInput(page)).toHaveCount(0);
    await page
      .locator('canvas')
      .click({ position: ROLE_FILTER_ROW_CELL });
    await waitForPaint(page);
    // Nothing opens — the row doesn't exist yet.
    await expect(quickFilterInput(page)).toHaveCount(0);
  });

  test('clicking a cell opens a quick-filter input; typing + Enter filters rows', async ({
    page,
  }) => {
    await enableFilterRow(page);
    await page
      .locator('canvas')
      .click({ position: ROLE_FILTER_ROW_CELL });
    await waitForPaint(page);
    await expect(quickFilterInput(page)).toBeVisible();
    await expect(quickFilterInput(page)).toHaveValue('');

    await quickFilterInput(page).fill('CEO');
    await quickFilterInput(page).press('Enter');
    await waitForPaint(page);
    await expect(quickFilterInput(page)).toHaveCount(0);

    // Row 1's Role cell should now read "CEO" — filtered via the row's
    // Contains condition, dispatched the same way the popup's own value
    // input does.
    await openConditionEditor(page, ROLE_FILTER_ROW_ICON);
    await expect(valueInput(page)).toHaveValue('CEO');
  });

  test('re-opening the cell after committing shows the current value, not the placeholder', async ({
    page,
  }) => {
    await enableFilterRow(page);
    await page
      .locator('canvas')
      .click({ position: ROLE_FILTER_ROW_CELL });
    await quickFilterInput(page).fill('CEO');
    await quickFilterInput(page).press('Enter');
    await waitForPaint(page);

    await page
      .locator('canvas')
      .click({ position: ROLE_FILTER_ROW_CELL });
    await waitForPaint(page);
    await expect(quickFilterInput(page)).toHaveValue('CEO');
  });

  test('Escape cancels without applying', async ({ page }) => {
    await enableFilterRow(page);
    await page
      .locator('canvas')
      .click({ position: ROLE_FILTER_ROW_CELL });
    await quickFilterInput(page).fill('should not apply');
    await quickFilterInput(page).press('Escape');
    await waitForPaint(page);
    await expect(quickFilterInput(page)).toHaveCount(0);

    await openConditionEditor(page, ROLE_FILTER_ROW_ICON);
    await expect(valueInput(page)).toHaveValue('');
  });

  test('clicking the mini funnel icon opens the same full popup, unchanged', async ({
    page,
  }) => {
    await enableFilterRow(page);
    await page
      .locator('canvas')
      .click({ position: ROLE_FILTER_ROW_ICON });
    await waitForPaint(page);
    await expect(popup(page)).toBeVisible();
    await textFilterRow(page).click();
    await expect(opTrigger(page)).toHaveText('Contains');
    await expect(searchInput(page)).toBeVisible();
  });

  test('an advanced (non-Contains) condition set via the popup shows its raw value in the row', async ({
    page,
  }) => {
    await enableFilterRow(page);
    await openConditionEditor(page, SALARY_FILTER_ROW_ICON);
    await selectOp(page, 'Greater than');
    await valueInput(page).fill('50000');
    await applyBtn(page).click();
    await waitForPaint(page);

    const salaryFilterRowCell = filterRowCellCenter(
      SALARY_COL_OFFSET,
      SALARY_COL_WIDTH,
    );
    await page.locator('canvas').click({ position: salaryFilterRowCell });
    await waitForPaint(page);
    // The row shows the raw value regardless of the stored operator —
    // best-effort, same AG-Grid-style simplification the popup's own
    // Apply closure documents.
    await expect(quickFilterInput(page)).toHaveValue('50000');
  });
});

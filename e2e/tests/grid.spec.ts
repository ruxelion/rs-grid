import { test, expect, Page } from '@playwright/test';

// ── helpers ────────────────────────────────────────────────────────────────────

/** Wait for the rAF loop to paint at least one frame. */
async function waitForPaint(page: Page, ms = 300) {
  await page.waitForTimeout(ms);
}

// ── smoke ──────────────────────────────────────────────────────────────────────

test.describe('smoke', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
  });

  // TODO: Reload triggers a "Failed to fetch" error related to WASM loading
  //       on npx serve (no hot-reload). Needs investigation.
  test.skip('la page se charge sans erreur JS', async ({ page }) => {
    const errors: string[] = [];
    page.on('pageerror', err => errors.push(err.message));
    await page.reload();
    await waitForPaint(page);
    expect(errors).toHaveLength(0);
  });

  test('le titre est visible', async ({ page }) => {
    await expect(page.getByText('rs-grid basic example')).toBeVisible();
  });

  test('le canvas est visible avec des dimensions', async ({ page }) => {
    const canvas = page.locator('canvas');
    await expect(canvas).toBeVisible();
    const box = await canvas.boundingBox();
    expect(box!.width).toBeGreaterThan(200);
    expect(box!.height).toBeGreaterThan(200);
  });

  test('affiche 1 000 lignes par défaut', async ({ page }) => {
    await expect(page.locator('strong', { hasText: '1 000 rows' })).toBeVisible();
  });

  test('affiche 20 colonnes par défaut', async ({ page }) => {
    await expect(page.locator('strong', { hasText: '20 columns' })).toBeVisible();
  });
});

// ── contrôles DOM ──────────────────────────────────────────────────────────────

test.describe('contrôles', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
  });

  test('passage à 100 000 lignes', async ({ page }) => {
    const select = page.locator('select').first();
    await select.selectOption('100000');
    await waitForPaint(page);
    await expect(page.locator('strong', { hasText: '100 000 rows' })).toBeVisible();
    // Canvas must remain visible after re-render
    await expect(page.locator('canvas')).toBeVisible();
  });

  test('passage à 100 colonnes', async ({ page }) => {
    const select = page.locator('select').nth(1);
    await select.selectOption('100');
    await waitForPaint(page);
    await expect(page.locator('strong', { hasText: '100 columns' })).toBeVisible();
    await expect(page.locator('canvas')).toBeVisible();
  });

  test('changement combiné lignes + colonnes', async ({ page }) => {
    await page.locator('select').first().selectOption('100000');
    await page.locator('select').nth(1).selectOption('100');
    await waitForPaint(page);
    await expect(page.locator('strong', { hasText: '100 000 rows' })).toBeVisible();
    await expect(page.locator('strong', { hasText: '100 columns' })).toBeVisible();
    await expect(page.locator('canvas')).toBeVisible();
  });
});

// ── interaction canvas ────────────────────────────────────────────────────────
//
// Rendering is on <canvas> — interactions use viewport coordinates.
// Approximate grid layout:
//   - Gutter (row numbers): ~50 px on the left
//   - Header (column labels): ~60 px at the top
//   - First data cell: around (80, 80) in canvas space

test.describe('interaction canvas', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
  });

  test('clic sur une cellule de données ne plante pas', async ({ page }) => {
    const canvas = page.locator('canvas');
    // Approximate position: first data cell
    await canvas.click({ position: { x: 80, y: 80 } });
    await waitForPaint(page, 100);
    await expect(canvas).toBeVisible();
  });

  test('scroll molette dans le canvas', async ({ page }) => {
    const canvas = page.locator('canvas');
    await canvas.hover();
    await page.mouse.wheel(0, 300);
    await waitForPaint(page, 100);
    await expect(canvas).toBeVisible();
  });

  test('scroll puis clic sur une cellule', async ({ page }) => {
    const canvas = page.locator('canvas');
    await canvas.hover();
    await page.mouse.wheel(0, 200);
    await waitForPaint(page, 100);
    await canvas.click({ position: { x: 80, y: 80 } });
    await waitForPaint(page, 100);
    await expect(canvas).toBeVisible();
  });

  test('shift+clic étend la sélection', async ({ page }) => {
    const canvas = page.locator('canvas');
    await canvas.click({ position: { x: 80, y: 80 } });
    await canvas.click({ position: { x: 200, y: 120 }, modifiers: ['Shift'] });
    await waitForPaint(page, 100);
    // Visual check: multi-cell selection must be visible (blue background).
    await expect(canvas).toHaveScreenshot('shift-click-selection.png', {
      maxDiffPixelRatio: 0.02,
    });
  });
});

// ── colonnes pinnées ────────────────────────────────────────────────────────────
//
// The demo does not expose a <select> for pinned column count.
// We go through the context menu (right-click on header) or dispatch
// the command via the JS API exposed by the Leptos app.
// For now we right-click the column header to open the menu and
// select "Pin column".
//
// Helper: right-click column header at index `col` and click the Pin option.
async function pinColumnsViaContextMenu(page: Page, count: number) {
  // Right-click the Name column header (first column, x ≈ GUTTER + 100)
  const canvas = page.locator('canvas');
  for (let i = 0; i < count; i++) {
    const colX = 55 + 100 * (i + 1); // approximate header center
    await canvas.click({ position: { x: colX, y: 30 }, button: 'right' });
    await page.waitForTimeout(100);
    // Look for a "Pin" or "Pinned" item in the context menu
    const pinItem = page.locator('text=/pin/i').first();
    if (await pinItem.isVisible({ timeout: 500 }).catch(() => false)) {
      await pinItem.click();
      await page.waitForTimeout(100);
    } else {
      // Close menu if no pin option found
      await page.keyboard.press('Escape');
    }
  }
}

test.describe('colonnes pinnées', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
  });

  test('pin 1 colonne ne plante pas', async ({ page }) => {
    await pinColumnsViaContextMenu(page, 1);
    await waitForPaint(page);
    await expect(page.locator('canvas')).toBeVisible();
  });

  test('pin 3 colonnes ne plante pas', async ({ page }) => {
    await pinColumnsViaContextMenu(page, 3);
    await waitForPaint(page);
    await expect(page.locator('canvas')).toBeVisible();
  });

  test('scroll horizontal avec colonnes pinnées', async ({ page }) => {
    await pinColumnsViaContextMenu(page, 2);
    await waitForPaint(page);
    const canvas = page.locator('canvas');
    await canvas.hover();
    await page.mouse.wheel(500, 0);
    await waitForPaint(page);
    await expect(canvas).toBeVisible();
  });

  test('clic cellule après pin + scroll horizontal', async ({ page }) => {
    await pinColumnsViaContextMenu(page, 1);
    await waitForPaint(page);
    const canvas = page.locator('canvas');
    await canvas.hover();
    await page.mouse.wheel(300, 0);
    await waitForPaint(page);
    await canvas.click({ position: { x: 80, y: 80 } });
    await waitForPaint(page, 100);
    await expect(canvas).toBeVisible();
  });
});

// ── régression visuelle ────────────────────────────────────────────────────────
//
// These tests compare rendering pixel-by-pixel against reference screenshots.
// To generate references: npm run update-snapshots
// Tolerance: 2% of pixels may differ (antialiasing, minor rendering variation).

test.describe('visual regression', () => {
  test('état initial', async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
    await expect(page).toHaveScreenshot('initial.png', {
      maxDiffPixelRatio: 0.02,
    });
  });

  test('après scroll vertical', async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
    const canvas = page.locator('canvas');
    await canvas.hover();
    await page.mouse.wheel(0, 500);
    await waitForPaint(page);
    await expect(canvas).toHaveScreenshot('scrolled-down.png', {
      maxDiffPixelRatio: 0.02,
    });
  });

  test('cellule sélectionnée', async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
    await page.locator('canvas').click({ position: { x: 80, y: 80 } });
    await waitForPaint(page, 100);
    await expect(page.locator('canvas')).toHaveScreenshot('cell-selected.png', {
      maxDiffPixelRatio: 0.02,
    });
  });

  test('100 000 lignes', async ({ page }) => {
    await page.goto('/');
    await page.locator('select').first().selectOption('100000');
    await waitForPaint(page);
    await expect(page.locator('canvas')).toHaveScreenshot('100k-rows.png', {
      maxDiffPixelRatio: 0.02,
    });
  });

  test('colonnes pinnées', async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
    await pinColumnsViaContextMenu(page, 2);
    await waitForPaint(page);
    await expect(page.locator('canvas')).toHaveScreenshot('pinned-cols.png', {
      maxDiffPixelRatio: 0.02,
    });
  });

  test('colonnes pinnées + scroll horizontal', async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
    await pinColumnsViaContextMenu(page, 2);
    await waitForPaint(page);
    const canvas = page.locator('canvas');
    await canvas.hover();
    await page.mouse.wheel(500, 0);
    await waitForPaint(page);
    await expect(canvas).toHaveScreenshot('pinned-scroll-h.png', {
      maxDiffPixelRatio: 0.02,
    });
  });

  test('colonnes pinnées + scroll vertical', async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
    await pinColumnsViaContextMenu(page, 2);
    await waitForPaint(page);
    const canvas = page.locator('canvas');
    await canvas.hover();
    await page.mouse.wheel(0, 500);
    await waitForPaint(page);
    await expect(canvas).toHaveScreenshot('pinned-scroll-v.png', {
      maxDiffPixelRatio: 0.02,
    });
  });
});

// ── features récentes ─────────────────────────────────────────────────────
//
// Tests for recently added features:
//  - auto-scroll during drag-select
//  - column drag for reordering
//  - copy of a full column selection → header only
//  - shift+click on header → sort (not ExtendColSelection)

test.describe('features récentes', () => {
  // Layout constants (matching build_model and editing.spec.ts)
  const GUTTER = 55;
  const HEADER = 60;
  const SB_W = 14; // scrollbar_width (theme)

  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
  });

  test('auto-scroll — drag vers le bas défile et étend la sélection', async ({ page }) => {
    const canvas = page.locator('canvas');
    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();

    // Start drag on a top cell (row 0, Name col)
    await page.mouse.move(box!.x + GUTTER + 100, box!.y + HEADER + 20);
    await page.mouse.down();
    await waitForPaint(page, 50);

    // Move toward the bottom edge (into the 50px auto-scroll zone)
    await page.mouse.move(box!.x + GUTTER + 100, box!.y + box!.height - 10, { steps: 5 });
    // Wait for auto-scroll to advance several rows (~400 ms)
    await waitForPaint(page, 600);
    // Release BEFORE screenshot to stop auto-scroll and let the canvas settle.
    await page.mouse.up();
    await waitForPaint(page, 200);

    await expect(canvas).toHaveScreenshot('autoscroll-drag-down.png', {
      maxDiffPixelRatio: 0.02,
    });
  });

  test('auto-scroll — drag vers le haut défile vers le haut', async ({ page }) => {
    const canvas = page.locator('canvas');
    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();

    // Scroll down first to have room to scroll back up
    await canvas.hover();
    await page.mouse.wheel(0, 400);
    await waitForPaint(page, 200);

    // Start a drag and move toward the top edge
    await page.mouse.move(box!.x + GUTTER + 100, box!.y + HEADER + 100);
    await page.mouse.down();
    await waitForPaint(page, 50);
    await page.mouse.move(box!.x + GUTTER + 100, box!.y + HEADER + 5, { steps: 5 });
    await waitForPaint(page, 400);

    await expect(canvas).toHaveScreenshot('autoscroll-drag-up.png', {
      maxDiffPixelRatio: 0.02,
    });
    await page.mouse.up();
  });

  test('drag colonne réordonne les colonnes', async ({ page }) => {
    const canvas = page.locator('canvas');
    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();

    // Center of Name header (col 0) and Email header (col 1)
    const nameHeaderX = GUTTER + 100; // approximate center of Name (200px wide)
    const emailHeaderX = GUTTER + 200 + 130; // approximate center of Email
    const headerY = HEADER / 2;

    // Drag Name → past Email
    await page.mouse.move(box!.x + nameHeaderX, box!.y + headerY);
    await page.mouse.down();
    await page.mouse.move(box!.x + nameHeaderX + 30, box!.y + headerY, { steps: 3 });
    await page.mouse.move(box!.x + emailHeaderX, box!.y + headerY, { steps: 10 });
    await page.mouse.up();
    await waitForPaint(page, 400);

    await expect(canvas).toHaveScreenshot('column-reordered.png', {
      maxDiffPixelRatio: 0.02,
    });
  });

  test('shift+clic sur header trie sans étendre la sélection cellule', async ({ page }) => {
    const canvas = page.locator('canvas');
    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();

    // Select a cell to establish a selection state
    await canvas.click({ position: { x: GUTTER + 100, y: HEADER + 20 } });
    await waitForPaint(page, 100);

    // Shift+click on the Role header — must sort, not extend the selection
    const roleHeaderX = GUTTER + 200 + 260 + 70; // center of Role (3rd col)
    await canvas.click({
      position: { x: roleHeaderX, y: HEADER / 2 },
      modifiers: ['Shift'],
    });
    await waitForPaint(page, 200);

    // Grid must be sorted by Role — first row alphabetically,
    // AND the selection must NOT span all Role rows.
    await expect(canvas).toHaveScreenshot('shift-click-header-sorts.png', {
      maxDiffPixelRatio: 0.02,
    });
  });
});

// ── scrollbar logarithmique ────────────────────────────────────────────────
//
// Beyond ~33 333 rows (1 000 000 px) the scrollbar switches to logarithmic
// mapping. These tests verify:
//   1. No crash at large scale
//   2. Thumb is at the top when scroll=0, at the bottom when scroll=max
//   3. Dragging the thumb from the top genuinely changes the position
//   4. Track-click navigates in the correct direction
//   5. Thumb at 50% of logarithmic travel is MUCH closer to the bottom
//      than the linear midpoint (the key property of log mapping)

test.describe('scrollbar logarithmique', () => {
  test('10^9 lignes — aucun crash au chargement', async ({ page }) => {
    await page.goto('/');
    await page.locator('select').first().selectOption('1000000000');
    await waitForPaint(page, 500);
    await expect(page.locator('canvas')).toBeVisible();
    const errors: string[] = [];
    page.on('pageerror', e => errors.push(e.message));
    await waitForPaint(page, 200);
    expect(errors).toHaveLength(0);
  });

  test('10^9 lignes — thumb au sommet au démarrage', async ({ page }) => {
    await page.goto('/');
    await page.locator('select').first().selectOption('1000000000');
    await waitForPaint(page, 400);
    const canvas = page.locator('canvas');
    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();
    // Reference screenshot for the scrollbar position
    await expect(canvas).toHaveScreenshot('log-sb-1b-top.png', {
      maxDiffPixelRatio: 0.02,
    });
  });

  test('10^9 lignes — wheel scroll déplace le thumb', async ({ page }) => {
    await page.goto('/');
    await page.locator('select').first().selectOption('1000000000');
    await waitForPaint(page, 400);
    const canvas = page.locator('canvas');
    // Wheel scroll: move content a few rows
    await canvas.hover();
    await page.mouse.wheel(0, 3000);
    await waitForPaint(page, 300);
    // Thumb must have moved (screenshot differs from top reference)
    await expect(canvas).toHaveScreenshot('log-sb-1b-scrolled.png', {
      maxDiffPixelRatio: 0.02,
    });
  });

  test('10^9 lignes — clic track milieu ne plante pas', async ({ page }) => {
    await page.goto('/');
    await page.locator('select').first().selectOption('1000000000');
    await waitForPaint(page, 400);
    const canvas = page.locator('canvas');
    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();
    // Click the middle of the vertical scrollbar
    const sbX = box!.width - 8;
    const sbY = box!.height / 2;
    await canvas.click({ position: { x: sbX, y: sbY } });
    await waitForPaint(page, 300);
    await expect(canvas).toBeVisible();
  });

  test('10^9 lignes — drag thumb depuis le haut change la position', async ({ page }) => {
    await page.goto('/');
    await page.locator('select').first().selectOption('1000000000');
    await waitForPaint(page, 400);
    const canvas = page.locator('canvas');
    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();

    // Scrollbar position: right edge of canvas
    const sbX = box!.x + box!.width - 8;
    // Thumb position at scroll_y=0: top of track = header_h + arrow_h.
    // Constants from editing.spec.ts (HEADER=60) and theme.scrollbar_width=14.
    const GRID_HEADER_H = 60; // model.header_height
    const SB_ARROW_H = 14;    // theme.scrollbar_width (= arrow button height)
    const thumbStartY = box!.y + GRID_HEADER_H + SB_ARROW_H; // top of thumb at scroll=0
    // Drag down 100 px
    await page.mouse.move(sbX, thumbStartY);
    await page.mouse.down();
    await page.mouse.move(sbX, thumbStartY + 100, { steps: 10 });
    await page.mouse.up();
    await waitForPaint(page, 300);

    // Verify the render changed (scroll occurred)
    await expect(canvas).toHaveScreenshot('log-sb-1b-dragged.png', {
      maxDiffPixelRatio: 0.02,
    });
  });

  test('10^12 lignes — scrollbar visible et fonctionnelle', async ({ page }) => {
    await page.goto('/');
    await page.locator('select').first().selectOption('1000000000000');
    await waitForPaint(page, 500);
    const canvas = page.locator('canvas');
    await expect(canvas).toBeVisible();
    // Wheel scroll
    await canvas.hover();
    await page.mouse.wheel(0, 1000);
    await waitForPaint(page, 300);
    await expect(canvas).toBeVisible();
  });

  test('mapping log — drag 50% du travel ≠ 50% des données', async ({ page }) => {
    // At 10^9 rows with log mapping, the thumb at mid-travel corresponds to
    // ln(1 + max/2) / ln(1 + max) ≈ 0.97 of max_scroll, i.e. ~97% of content.
    // This test verifies that scrolling the thumb halfway down the track
    // shows data near the bottom, not the middle.
    await page.goto('/');
    await page.locator('select').first().selectOption('1000000000');
    await waitForPaint(page, 400);
    const canvas = page.locator('canvas');
    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();

    // Click the middle of the scrollbar track using named constants.
    const GRID_HEADER_H = 60; // grid header height (model.header_height)
    const SB_ARROW_H = 14;    // scrollbar arrow height (= theme.scrollbar_width)
    const trackTop = GRID_HEADER_H + SB_ARROW_H;       // 74 px
    const trackBottom = box!.height - SB_ARROW_H;      // height - 14 px
    const sbX = box!.width - 8;
    const sbMidY = Math.round((trackTop + trackBottom) / 2);
    await canvas.click({ position: { x: sbX, y: sbMidY } });
    await waitForPaint(page, 400);

    // Reference screenshot: with log mapping, shows data very close to the
    // end of the dataset (rows ~10^8+ visible)
    await expect(canvas).toHaveScreenshot('log-sb-mid-travel.png', {
      maxDiffPixelRatio: 0.02,
    });
  });
});

// ── précision f64 à grande échelle ──────────────────────────────────────────
//
// At 1M+ rows, pixel positions (row_top − scroll_y) risk losing f64 precision.
// These tests verify that rendering and hit-testing remain aligned after
// scrolling to the end of a large dataset.

test.describe('précision f64 grande échelle', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
  });

  /**
   * Scroll the vertical scrollbar to the bottom by clicking
   * on the track just above the bottom edge.
   */
  async function scrollToBottom(page: Page, canvas: any, box: any) {
    // The vertical scrollbar is on the right side of the canvas.
    // Click near the bottom of the track to jump to the end.
    const sbX = box.width - 5;
    const sbY = box.height - 20;
    await canvas.click({ position: { x: sbX, y: sbY } });
    await waitForPaint(page);
    // Second click to refine if the thumb is not quite at the bottom.
    await canvas.click({ position: { x: sbX, y: sbY } });
    await waitForPaint(page);
  }

  test('1M lignes — scroll en bas + clic aligné', async ({ page }) => {
    await page.locator('select').first().selectOption('1000000');
    await waitForPaint(page);

    const canvas = page.locator('canvas');
    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();

    await scrollToBottom(page, canvas, box!);

    // Click a visible cell (center of canvas)
    const clickY = box!.height / 2;
    const clickX = box!.width / 3;
    await canvas.click({ position: { x: clickX, y: clickY } });
    await waitForPaint(page);

    // Screenshot: the blue selection must align with the clicked row —
    // no off-by-one row shift.
    await expect(canvas).toHaveScreenshot('1m-bottom-click.png', {
      maxDiffPixelRatio: 0.02,
    });
  });

  test('100M lignes — scroll en bas + clic aligné', async ({ page }) => {
    await page.locator('select').first().selectOption('100000000');
    await waitForPaint(page);

    const canvas = page.locator('canvas');
    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();

    await scrollToBottom(page, canvas, box!);

    const clickY = box!.height / 2;
    const clickX = box!.width / 3;
    await canvas.click({ position: { x: clickX, y: clickY } });
    await waitForPaint(page);

    await expect(canvas).toHaveScreenshot('100m-bottom-click.png', {
      maxDiffPixelRatio: 0.02,
    });
  });

  test('1M lignes — scroll milieu + sélection cohérente', async ({ page }) => {
    await page.locator('select').first().selectOption('1000000');
    await waitForPaint(page);

    const canvas = page.locator('canvas');
    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();

    // Scroll to ~50% via click on the middle of the track
    const sbX = box!.width - 5;
    const sbY = box!.height / 2;
    await canvas.click({ position: { x: sbX, y: sbY } });
    await waitForPaint(page);

    // Click then shift+click to select a range
    const y1 = box!.height * 0.3;
    const y2 = box!.height * 0.6;
    const x = box!.width / 3;
    await canvas.click({ position: { x, y: y1 } });
    await waitForPaint(page, 100);
    await canvas.click({ position: { x, y: y2 }, modifiers: ['Shift'] });
    await waitForPaint(page);

    // Selection must cover exactly the rows between the two clicks —
    // no row offset.
    await expect(canvas).toHaveScreenshot('1m-mid-range-select.png', {
      maxDiffPixelRatio: 0.02,
    });
  });

  // TODO: Double-click on canvas via Playwright fails on port 4173 (npx serve).
  //       Works on port 9080 (trunk serve). Root cause needs investigation.
  test.skip('1M lignes — double-clic édition en bas', async ({ page }) => {
    await page.locator('select').first().selectOption('1000000');
    await waitForPaint(page);

    const canvas = page.locator('canvas');
    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();

    await scrollToBottom(page, canvas, box!);

    // Double-click to edit — the input must appear on the correct cell
    const clickY = box!.height / 2;
    const clickX = box!.width / 3;
    await canvas.dblclick({ position: { x: clickX, y: clickY } });
    await waitForPaint(page);

    // Verify an edit input appeared
    const input = page.locator('input[type="text"]');
    await expect(input).toBeVisible();

    await expect(canvas).toHaveScreenshot('1m-bottom-edit.png', {
      maxDiffPixelRatio: 0.02,
    });
  });
});

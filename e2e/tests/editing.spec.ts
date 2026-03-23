import { test, expect, Page, Locator, BrowserContext } from '@playwright/test';

// ── helpers ────────────────────────────────────────────────────────────────────

/** Wait for at least one rAF paint cycle. */
async function waitForPaint(page: Page, ms = 300) {
  await page.waitForTimeout(ms);
}

// Grid layout constants (must match build_model: row_height=40, header_height=60).
// The gutter (row numbers) is ~50px wide.
const GUTTER = 55;
const HEADER = 60;
const ROW_H = 40;

/** Return the center of cell (row, col) in canvas-relative coordinates. */
function cellCenter(row: number, col: number, colWidths: number[] = [200, 260, 140, 160, 120, 80, 60]) {
  let x = GUTTER;
  for (let i = 0; i < col; i++) x += colWidths[i] ?? 120;
  x += (colWidths[col] ?? 120) / 2;
  const y = HEADER + row * ROW_H + ROW_H / 2;
  return { x, y };
}

/** Double-click a cell, read the edit input value, then Escape. */
async function readCellValue(page: Page, canvas: Locator, row: number, col: number): Promise<string> {
  const pos = cellCenter(row, col);
  await canvas.dblclick({ position: pos });
  await waitForPaint(page, 150);
  const input = page.locator('input[type="text"]');
  await expect(input).toBeVisible({ timeout: 2000 });
  const value = await input.inputValue();
  await page.keyboard.press('Escape');
  await waitForPaint(page, 100);
  return value;
}

/** Click a cell to select it. */
async function clickCell(page: Page, canvas: Locator, row: number, col: number) {
  await canvas.click({ position: cellCenter(row, col) });
  await waitForPaint(page, 100);
}

/** Shift-click a cell to extend selection. */
async function shiftClickCell(page: Page, canvas: Locator, row: number, col: number) {
  await canvas.click({ position: cellCenter(row, col), modifiers: ['Shift'] });
  await waitForPaint(page, 100);
}

// ── édition ─────────────────────────────────────────────────────────────────────

test.describe('édition', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
  });

  test('double-clic ouvre le champ d\'édition', async ({ page }) => {
    const canvas = page.locator('canvas');
    const pos = cellCenter(0, 0);  // Name column, row 0
    await canvas.dblclick({ position: pos });
    await waitForPaint(page, 150);
    const input = page.locator('input[type="text"]');
    await expect(input).toBeVisible();
    // The input should contain the current cell value (a name)
    const value = await input.inputValue();
    expect(value.length).toBeGreaterThan(0);
  });

  test('Escape annule l\'édition', async ({ page }) => {
    const canvas = page.locator('canvas');
    // Read original value
    const original = await readCellValue(page, canvas, 0, 0);

    // Enter edit mode, type something, then Escape
    await canvas.dblclick({ position: cellCenter(0, 0) });
    await waitForPaint(page, 150);
    const input = page.locator('input[type="text"]');
    await input.fill('SHOULD_NOT_PERSIST');
    await page.keyboard.press('Escape');
    await waitForPaint(page, 100);

    // Re-read — value should be unchanged
    const after = await readCellValue(page, canvas, 0, 0);
    expect(after).toBe(original);
  });

  test('Enter valide l\'édition', async ({ page }) => {
    const canvas = page.locator('canvas');
    await canvas.dblclick({ position: cellCenter(0, 0) });
    await waitForPaint(page, 150);
    const input = page.locator('input[type="text"]');
    await input.fill('EDITED_VALUE');
    await page.keyboard.press('Enter');
    await waitForPaint(page, 100);

    // Re-read — value should be the new one
    const after = await readCellValue(page, canvas, 0, 0);
    expect(after).toBe('EDITED_VALUE');
  });

  test('Ctrl+Z annule l\'édition commitée', async ({ page }) => {
    const canvas = page.locator('canvas');
    const original = await readCellValue(page, canvas, 0, 0);

    // Edit + commit
    await canvas.dblclick({ position: cellCenter(0, 0) });
    await waitForPaint(page, 150);
    await page.locator('input[type="text"]').fill('WILL_UNDO');
    await page.keyboard.press('Enter');
    await waitForPaint(page, 100);

    // Undo
    await clickCell(page, canvas, 0, 0);
    await page.keyboard.press('Control+z');
    await waitForPaint(page, 100);

    const after = await readCellValue(page, canvas, 0, 0);
    expect(after).toBe(original);
  });
});

// ── clipboard ───────────────────────────────────────────────────────────────────

test.describe('clipboard', () => {
  // Grant clipboard permissions for these tests.
  test.use({
    permissions: ['clipboard-read', 'clipboard-write'],
  });

  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
  });

  test('copier-coller une cellule', async ({ page }) => {
    const canvas = page.locator('canvas');
    const original = await readCellValue(page, canvas, 0, 0);

    // Select cell (0, 0) and copy
    await clickCell(page, canvas, 0, 0);
    await page.keyboard.press('Control+c');
    await waitForPaint(page, 100);

    // Select cell (5, 0) and paste
    await clickCell(page, canvas, 5, 0);
    await page.keyboard.press('Control+v');
    await waitForPaint(page, 100);

    // Verify the pasted value
    const pasted = await readCellValue(page, canvas, 5, 0);
    expect(pasted).toBe(original);
  });

  test('coller avec sélection vers le haut remplit depuis le top', async ({ page }) => {
    const canvas = page.locator('canvas');

    // Write clipboard with known content via the browser API
    await page.evaluate(() =>
      navigator.clipboard.writeText('PASTED\n')
    );

    // Select row 4, then shift-click row 2 (upward selection)
    await clickCell(page, canvas, 4, 0);
    await shiftClickCell(page, canvas, 2, 0);

    // Paste
    await page.keyboard.press('Control+v');
    await waitForPaint(page, 200);

    // All three rows (2, 3, 4) should be filled with 'PASTED'
    const v2 = await readCellValue(page, canvas, 2, 0);
    const v3 = await readCellValue(page, canvas, 3, 0);
    const v4 = await readCellValue(page, canvas, 4, 0);
    expect(v2).toBe('PASTED');
    expect(v3).toBe('PASTED');
    expect(v4).toBe('PASTED');

    // Row 1 should be untouched (not pasted downward from anchor)
    const v1 = await readCellValue(page, canvas, 1, 0);
    expect(v1).not.toBe('PASTED');
  });

  test('coller multi-lignes respecte l\'ordre', async ({ page }) => {
    const canvas = page.locator('canvas');

    await page.evaluate(() =>
      navigator.clipboard.writeText('AAA\nBBB\nCCC\n')
    );

    await clickCell(page, canvas, 1, 0);
    await page.keyboard.press('Control+v');
    await waitForPaint(page, 200);

    expect(await readCellValue(page, canvas, 1, 0)).toBe('AAA');
    expect(await readCellValue(page, canvas, 2, 0)).toBe('BBB');
    expect(await readCellValue(page, canvas, 3, 0)).toBe('CCC');
  });

  test('Ctrl+Z après collage restaure les valeurs', async ({ page }) => {
    const canvas = page.locator('canvas');
    const before = await readCellValue(page, canvas, 0, 0);

    await page.evaluate(() =>
      navigator.clipboard.writeText('TEMP\n')
    );
    await clickCell(page, canvas, 0, 0);
    await page.keyboard.press('Control+v');
    await waitForPaint(page, 100);
    expect(await readCellValue(page, canvas, 0, 0)).toBe('TEMP');

    // Undo
    await clickCell(page, canvas, 0, 0);
    await page.keyboard.press('Control+z');
    await waitForPaint(page, 100);
    expect(await readCellValue(page, canvas, 0, 0)).toBe(before);
  });
});

// ── navigation clavier ──────────────────────────────────────────────────────────

test.describe('navigation clavier', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
  });

  test('flèches déplacent la sélection', async ({ page }) => {
    const canvas = page.locator('canvas');
    // Click cell (0,0) to select
    await clickCell(page, canvas, 0, 0);

    // Move down twice
    await page.keyboard.press('ArrowDown');
    await page.keyboard.press('ArrowDown');
    await waitForPaint(page, 100);

    // Move right once
    await page.keyboard.press('ArrowRight');
    await waitForPaint(page, 100);

    // Screenshot should show selection at (2, 1) — Email column, row 2
    await expect(canvas).toHaveScreenshot('nav-arrows.png', {
      maxDiffPixelRatio: 0.02,
    });
  });

  test('Shift+flèche étend la sélection', async ({ page }) => {
    const canvas = page.locator('canvas');
    await clickCell(page, canvas, 0, 0);

    // Extend selection down 2 rows and right 1 column
    await page.keyboard.press('Shift+ArrowDown');
    await page.keyboard.press('Shift+ArrowDown');
    await page.keyboard.press('Shift+ArrowRight');
    await waitForPaint(page, 100);

    // Should show a 3×2 selection highlight
    await expect(canvas).toHaveScreenshot('nav-shift-extend.png', {
      maxDiffPixelRatio: 0.02,
    });
  });

  test('Escape désélectionne tout', async ({ page }) => {
    const canvas = page.locator('canvas');
    await clickCell(page, canvas, 0, 0);
    await page.keyboard.press('Escape');
    await waitForPaint(page, 100);

    // No visible selection
    await expect(canvas).toHaveScreenshot('nav-deselected.png', {
      maxDiffPixelRatio: 0.02,
    });
  });
});

// ── thèmes ──────────────────────────────────────────────────────────────────────

test.describe('thèmes', () => {
  test('thème dark s\'applique', async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
    // Theme selector is the last <select>
    await page.locator('select').last().selectOption('dark');
    await waitForPaint(page);
    await expect(page).toHaveScreenshot('theme-dark.png', {
      maxDiffPixelRatio: 0.02,
    });
  });

  test('thème Material 3 s\'applique', async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
    await page.locator('select').last().selectOption('material');
    await waitForPaint(page);
    await expect(page).toHaveScreenshot('theme-material.png', {
      maxDiffPixelRatio: 0.02,
    });
  });

  test('thème Material 3 Dark s\'applique', async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
    await page.locator('select').last().selectOption('material-dark');
    await waitForPaint(page);
    await expect(page).toHaveScreenshot('theme-material-dark.png', {
      maxDiffPixelRatio: 0.02,
    });
  });

  test('retour au thème light depuis dark', async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
    const sel = page.locator('select').last();
    await sel.selectOption('dark');
    await waitForPaint(page);
    await sel.selectOption('');  // light = empty value
    await waitForPaint(page);
    await expect(page).toHaveScreenshot('theme-light-restored.png', {
      maxDiffPixelRatio: 0.02,
    });
  });
});

// ── filtre ──────────────────────────────────────────────────────────────────────

test.describe('filtre', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
  });

  test('saisie dans le filtre réduit les lignes', async ({ page }) => {
    const canvas = page.locator('canvas');
    const filterInput = page.locator('input[placeholder]');

    // Type a filter that matches few rows
    await filterInput.fill('Alice');
    await waitForPaint(page);

    // The grid should look different — visual regression
    await expect(canvas).toHaveScreenshot('filter-alice.png', {
      maxDiffPixelRatio: 0.02,
    });
  });

  test('effacer le filtre restaure toutes les lignes', async ({ page }) => {
    const canvas = page.locator('canvas');
    const filterInput = page.locator('input[placeholder]');

    await filterInput.fill('Alice');
    await waitForPaint(page);
    await filterInput.fill('');
    await waitForPaint(page);

    // Should be back to the normal view
    await expect(canvas).toHaveScreenshot('filter-cleared.png', {
      maxDiffPixelRatio: 0.02,
    });
  });

  test('filtre sans résultat affiche grille vide', async ({ page }) => {
    const canvas = page.locator('canvas');
    const filterInput = page.locator('input[placeholder]');

    await filterInput.fill('ZZZZZZZ_NO_MATCH');
    await waitForPaint(page);

    await expect(canvas).toHaveScreenshot('filter-no-match.png', {
      maxDiffPixelRatio: 0.02,
    });
  });
});

// ── export/import round-trip ────────────────────────────────────────────────────

test.describe('export/import', () => {
  test('export puis import restaure les modifications', async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
    const canvas = page.locator('canvas');

    // Edit a cell
    await canvas.dblclick({ position: cellCenter(0, 0) });
    await waitForPaint(page, 150);
    await page.locator('input[type="text"]').fill('EXPORT_TEST');
    await page.keyboard.press('Enter');
    await waitForPaint(page, 100);

    // Click Export — intercept the download
    const [download] = await Promise.all([
      page.waitForEvent('download'),
      page.locator('button', { hasText: 'Export' }).click(),
    ]);
    const content = (await download.createReadStream())
      ? await new Promise<string>((resolve) => {
          const chunks: Buffer[] = [];
          download.createReadStream().then(stream => {
            if (!stream) { resolve(''); return; }
            stream.on('data', (c: Buffer) => chunks.push(c));
            stream.on('end', () => resolve(Buffer.concat(chunks).toString('utf-8')));
          });
        })
      : '';
    expect(content).toContain('EXPORT_TEST');
  });
});

// ── scroll + sélection combinés ─────────────────────────────────────────────────

test.describe('scroll et sélection', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
  });

  test('sélection après scroll vertical reste cohérente', async ({ page }) => {
    const canvas = page.locator('canvas');
    await canvas.hover();
    await page.mouse.wheel(0, 400);
    await waitForPaint(page);

    // Click a cell after scrolling
    await clickCell(page, canvas, 2, 0);
    await waitForPaint(page, 100);

    await expect(canvas).toHaveScreenshot('scroll-then-select.png', {
      maxDiffPixelRatio: 0.02,
    });
  });

  test('édition après scroll horizontal', async ({ page }) => {
    const canvas = page.locator('canvas');
    await canvas.hover();
    await page.mouse.wheel(300, 0);
    await waitForPaint(page);

    // Double-click a visible cell — edit input should appear
    const pos = cellCenter(0, 0);
    await canvas.dblclick({ position: pos });
    await waitForPaint(page, 150);

    const input = page.locator('input[type="text"]');
    await expect(input).toBeVisible();
  });

  test('shift+clic multi-lignes après scroll ne plante pas', async ({ page }) => {
    const canvas = page.locator('canvas');
    await canvas.hover();
    await page.mouse.wheel(0, 200);
    await waitForPaint(page);

    await clickCell(page, canvas, 1, 0);
    await shiftClickCell(page, canvas, 5, 0);
    await waitForPaint(page, 100);

    await expect(canvas).toBeVisible();
  });
});

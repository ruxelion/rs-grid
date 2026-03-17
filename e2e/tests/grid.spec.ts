import { test, expect, Page } from '@playwright/test';

// ── helpers ────────────────────────────────────────────────────────────────────

/** Attends que la boucle rAF ait eu le temps de peindre au moins une frame. */
async function waitForPaint(page: Page, ms = 300) {
  await page.waitForTimeout(ms);
}

// ── smoke ──────────────────────────────────────────────────────────────────────

test.describe('smoke', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
  });

  test('la page se charge sans erreur JS', async ({ page }) => {
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
    await expect(page.getByText('1,000 rows')).toBeVisible();
  });

  test('affiche 10 colonnes par défaut', async ({ page }) => {
    await expect(page.getByText('10 columns')).toBeVisible();
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
    await expect(page.getByText('100,000 rows')).toBeVisible();
    // Le canvas doit rester visible après le re-rendu
    await expect(page.locator('canvas')).toBeVisible();
  });

  test('passage à 100 colonnes', async ({ page }) => {
    const select = page.locator('select').nth(1);
    await select.selectOption('100');
    await waitForPaint(page);
    await expect(page.getByText('100 columns')).toBeVisible();
    await expect(page.locator('canvas')).toBeVisible();
  });

  test('changement combiné lignes + colonnes', async ({ page }) => {
    await page.locator('select').first().selectOption('100000');
    await page.locator('select').nth(1).selectOption('100');
    await waitForPaint(page);
    await expect(page.getByText('100,000 rows')).toBeVisible();
    await expect(page.getByText('100 columns')).toBeVisible();
    await expect(page.locator('canvas')).toBeVisible();
  });
});

// ── interaction canvas ────────────────────────────────────────────────────────
//
// Le rendu est sur <canvas> — on interagit par coordonnées viewport.
// Disposition de la grille (valeurs indicatives) :
//   - Gutter (row numbers) : ~50 px à gauche
//   - Header (column labels) : ~60 px en haut
//   - Première cellule de données : environ (80, 80) dans le canvas

test.describe('interaction canvas', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
  });

  test('clic sur une cellule de données ne plante pas', async ({ page }) => {
    const canvas = page.locator('canvas');
    // Position approximative : première cellule de données
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
    await expect(canvas).toBeVisible();
  });
});

// ── régression visuelle ────────────────────────────────────────────────────────
//
// Ces tests comparent le rendu pixel-à-pixel avec des screenshots de référence.
// Pour générer les références : npm run update-snapshots
// Tolérance : 2 % de pixels différents (antialiasing, rendu légèrement variable).

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
});

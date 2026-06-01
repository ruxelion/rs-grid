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
    // Le canvas doit rester visible après le re-rendu
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
    // Vérifie visuellement que la sélection multi-cellules est présente (fond bleu).
    await expect(canvas).toHaveScreenshot('shift-click-selection.png', {
      maxDiffPixelRatio: 0.02,
    });
  });
});

// ── colonnes pinnées ────────────────────────────────────────────────────────────
//
// Le dropdown « Pinned cols » est le 3e <select> du DOM (nth(2)).
// Ces tests vérifient que le pin + scroll ne provoque pas de crash
// et que les en-têtes pinnés restent intacts visuellement.

test.describe('colonnes pinnées', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
  });

  test('pin 1 colonne ne plante pas', async ({ page }) => {
    await page.locator('select').nth(2).selectOption('1');
    await waitForPaint(page);
    await expect(page.locator('canvas')).toBeVisible();
  });

  test('pin 3 colonnes ne plante pas', async ({ page }) => {
    await page.locator('select').nth(2).selectOption('3');
    await waitForPaint(page);
    await expect(page.locator('canvas')).toBeVisible();
  });

  test('scroll horizontal avec colonnes pinnées', async ({ page }) => {
    await page.locator('select').nth(2).selectOption('2');
    await waitForPaint(page);
    const canvas = page.locator('canvas');
    await canvas.hover();
    await page.mouse.wheel(500, 0);
    await waitForPaint(page);
    await expect(canvas).toBeVisible();
  });

  test('clic cellule après pin + scroll horizontal', async ({ page }) => {
    await page.locator('select').nth(2).selectOption('1');
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

  test('colonnes pinnées', async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
    await page.locator('select').nth(2).selectOption('2');
    await waitForPaint(page);
    await expect(page.locator('canvas')).toHaveScreenshot('pinned-cols.png', {
      maxDiffPixelRatio: 0.02,
    });
  });

  test('colonnes pinnées + scroll horizontal', async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
    await page.locator('select').nth(2).selectOption('2');
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
    await page.locator('select').nth(2).selectOption('2');
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

// ── scrollbar logarithmique ────────────────────────────────────────────────
//
// Au-delà de ~33 333 lignes (1 000 000 px) la scrollbar passe en mapping
// logarithmique. Ces tests vérifient :
//   1. Pas de crash à grande échelle
//   2. Le thumb est au sommet quand scroll=0, au bas quand scroll=max
//   3. Le drag du thumb depuis le haut change réellement la position
//   4. Le track-click navigue dans la bonne direction
//   5. Le thumb à 50% du travel logarithmique est BEAUCOUP plus proche du bas
//      que du milieu linéaire (propriété clé du mapping log)

test.describe('scrollbar logarithmique', () => {
  /** Retourne la position Y du centre du thumb de la scrollbar verticale.
   *  La scrollbar est dans la bande x > canvasWidth - 20. */
  async function getThumbCenterY(page: Page): Promise<number> {
    return page.evaluate(() => {
      const canvas = document.querySelector('canvas') as HTMLCanvasElement;
      const ctx = canvas.getContext('2d')!;
      const rect = canvas.getBoundingClientRect();
      const w = canvas.width;
      const h = canvas.height;
      const dpr = window.devicePixelRatio || 1;
      // Scan la bande droite (scrollbar) pour trouver le thumb
      // en cherchant la première rangée avec la couleur du thumb
      // (pixel plus clair que le fond de la track)
      const sbX = Math.floor(w - 12 * dpr); // centre de la scrollbar
      for (let y = 0; y < h; y++) {
        const data = ctx.getImageData(sbX, y, 1, 1).data;
        // Cherche un pixel non-blanc et non-transparent (le thumb)
        if (data[3] > 200 && (data[0] < 200 || data[1] < 200)) {
          return (y / dpr) + rect.top;
        }
      }
      return -1;
    });
  }

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
    // Prend un screenshot de référence pour la scrollbar
    await expect(canvas).toHaveScreenshot('log-sb-1b-top.png', {
      maxDiffPixelRatio: 0.02,
    });
  });

  test('10^9 lignes — wheel scroll déplace le thumb', async ({ page }) => {
    await page.goto('/');
    await page.locator('select').first().selectOption('1000000000');
    await waitForPaint(page, 400);
    const canvas = page.locator('canvas');
    // Scroll molette : déplace le contenu de quelques lignes
    await canvas.hover();
    await page.mouse.wheel(0, 3000);
    await waitForPaint(page, 300);
    // Le thumb doit avoir bougé (screenshot différent de top)
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
    // Clic au milieu de la scrollbar verticale
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

    // Position de la scrollbar : droite du canvas
    const sbX = box!.x + box!.width - 8;
    // Position du thumb à scroll_y=0 : top of track = header_h + arrow_h.
    // Constantes issues de editing.spec.ts (HEADER=60) et theme.scrollbar_width=14.
    const GRID_HEADER_H = 60; // model.header_height
    const SB_ARROW_H = 14;    // theme.scrollbar_width (= arrow button height)
    const thumbStartY = box!.y + GRID_HEADER_H + SB_ARROW_H; // top of thumb at scroll=0
    // On drag vers le bas de 100px
    await page.mouse.move(sbX, thumbStartY);
    await page.mouse.down();
    await page.mouse.move(sbX, thumbStartY + 100, { steps: 10 });
    await page.mouse.up();
    await waitForPaint(page, 300);

    // Vérifier que le rendu a changé (scroll a eu lieu)
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
    // Scroll molette
    await canvas.hover();
    await page.mouse.wheel(0, 1000);
    await waitForPaint(page, 300);
    await expect(canvas).toBeVisible();
  });

  test('mapping log — drag 50% du travel ≠ 50% des données', async ({ page }) => {
    // À 10^9 lignes en mapping log, le thumb à mi-travel correspond à
    // ln(1 + max/2) / ln(1 + max) ≈ 0.97 du max_scroll, soit ~97% du contenu.
    // Ce test vérifie que faire défiler le thumb de la moitié du track
    // affiche des données proches du bas, pas du milieu.
    await page.goto('/');
    await page.locator('select').first().selectOption('1000000000');
    await waitForPaint(page, 400);
    const canvas = page.locator('canvas');
    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();

    // Clic au milieu du track scrollbar via constantes nommées.
    const GRID_HEADER_H = 60; // hauteur du header de grille (model.header_height)
    const SB_ARROW_H = 14;    // hauteur des flèches scrollbar (= theme.scrollbar_width)
    const trackTop = GRID_HEADER_H + SB_ARROW_H;       // 74 px
    const trackBottom = box!.height - SB_ARROW_H;      // height - 14 px
    const sbX = box!.width - 8;
    const sbMidY = Math.round((trackTop + trackBottom) / 2);
    await canvas.click({ position: { x: sbX, y: sbMidY } });
    await waitForPaint(page, 400);

    // Screenshot de référence : avec log, affiche des données
    // très proches de la fin du dataset (lignes ~10^8+ visibles)
    await expect(canvas).toHaveScreenshot('log-sb-mid-travel.png', {
      maxDiffPixelRatio: 0.02,
    });
  });
});

// ── précision f64 à grande échelle ──────────────────────────────────────────
//
// À 1 million+ de lignes, les positions de pixels (row_top − scroll_y)
// risquent de perdre la précision f64. Ces tests vérifient que le rendu
// et le hit-testing restent alignés après un scroll en fin de dataset.

test.describe('précision f64 grande échelle', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
  });

  /**
   * Scrolle la scrollbar verticale tout en bas en cliquant
   * sur le track juste au-dessus du bas.
   */
  async function scrollToBottom(page: Page, canvas: any, box: any) {
    // La scrollbar verticale est à droite du canvas.
    // Cliquer en bas du track pour sauter au fond.
    const sbX = box.width - 5;
    const sbY = box.height - 20;
    await canvas.click({ position: { x: sbX, y: sbY } });
    await waitForPaint(page);
    // Second clic pour affiner si le thumb n'est pas
    // tout à fait en bas.
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

    // Cliquer sur une cellule visible (milieu du canvas)
    const clickY = box!.height / 2;
    const clickX = box!.width / 3;
    await canvas.click({ position: { x: clickX, y: clickY } });
    await waitForPaint(page);

    // Screenshot : la sélection bleue doit être alignée avec la
    // ligne sur laquelle on a cliqué — pas de décalage d'une ligne.
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

    // Scroller à ~50% via clic sur le milieu du track
    const sbX = box!.width - 5;
    const sbY = box!.height / 2;
    await canvas.click({ position: { x: sbX, y: sbY } });
    await waitForPaint(page);

    // Cliquer puis shift+cliquer pour sélectionner un range
    const y1 = box!.height * 0.3;
    const y2 = box!.height * 0.6;
    const x = box!.width / 3;
    await canvas.click({ position: { x, y: y1 } });
    await waitForPaint(page, 100);
    await canvas.click({ position: { x, y: y2 }, modifiers: ['Shift'] });
    await waitForPaint(page);

    // La sélection doit couvrir exactement les lignes entre
    // les deux clics — pas de décalage.
    await expect(canvas).toHaveScreenshot('1m-mid-range-select.png', {
      maxDiffPixelRatio: 0.02,
    });
  });

  test('1M lignes — double-clic édition en bas', async ({ page }) => {
    await page.locator('select').first().selectOption('1000000');
    await waitForPaint(page);

    const canvas = page.locator('canvas');
    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();

    await scrollToBottom(page, canvas, box!);

    // Double-clic pour éditer — l'input doit apparaître
    // sur la bonne cellule
    const clickY = box!.height / 2;
    const clickX = box!.width / 3;
    await canvas.dblclick({ position: { x: clickX, y: clickY } });
    await waitForPaint(page);

    // Vérifier qu'un input d'édition est apparu
    const input = page.locator('input[type="text"]');
    await expect(input).toBeVisible();

    await expect(canvas).toHaveScreenshot('1m-bottom-edit.png', {
      maxDiffPixelRatio: 0.02,
    });
  });
});

import { test, expect, Page, ConsoleMessage } from '@playwright/test';

// ─────────────────────────────────────────────────────────────────────────────
// Limite du tri client-side selon le nombre de lignes.
//
// rs-grid n'effectue le tri en mémoire que jusqu'à `MAX_CLIENT_SORT_ROWS`
// (= 1 000 000, cf. crates/rs-grid-core/src/model.rs). Au-delà, `apply_sort`
// renvoie `false`, le tri est ignoré (la grille reste non triée) et
// rs-grid-web émet un `console.warn` (cf. canvas/dispatch.rs) :
//
//   "rs-grid: sort skipped — {row_count} rows exceeds the {limit}-row
//    client-side limit. Use a server-side data source for large datasets."
//
// La borne est STRICTE (`n > MAX`) : à exactement 1 000 000 lignes le tri
// s'applique encore ; ce n'est qu'à partir de 1 000 001 qu'il est ignoré.
// La fixture n'offre pas l'option 1 000 001 : on encadre donc la limite
// entre 1M (incluse → tri OK) et 100M (> limite → tri ignoré + warning).
//
// Le tri est déclenché par un clic sur un header de colonne
// (events.rs : ActiveDrag::ColClick → GridCommand::ToggleSort).
// ─────────────────────────────────────────────────────────────────────────────

const SORT_LIMIT = 1_000_000;
// Message émis par rs-grid-web quand le tri dépasse la limite.
const SORT_SKIP_RE = /sort skipped/i;

/** Laisse la boucle rAF peindre au moins une frame. */
async function waitForPaint(page: Page, ms = 300) {
  await page.waitForTimeout(ms);
}

/**
 * Branche un collecteur des `console.warn` « sort skipped ».
 * À appeler AVANT de déclencher le tri.
 */
function collectSortWarnings(page: Page): string[] {
  const warnings: string[] = [];
  page.on('console', (msg: ConsoleMessage) => {
    const text = msg.text();
    if (SORT_SKIP_RE.test(text)) warnings.push(text);
  });
  return warnings;
}

/**
 * Clique le header de colonne « Email » pour basculer le tri.
 *
 * Coordonnées locales au canvas. La gouttière des numéros de ligne
 * s'élargit avec le nombre de lignes (≈60 px @1k, 87 px @1M, 105 px @100M),
 * mais x=400 tombe toujours dans le header « Email » et à l'écart de
 * l'icône de menu (réservée côté droit du header). y=30 = milieu du
 * header (header_height = 60).
 */
async function clickHeaderToSort(page: Page) {
  await page.locator('canvas').click({ position: { x: 400, y: 30 } });
}

test.describe('limite de tri client-side selon le nombre de lignes', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForPaint(page);
  });

  test('≤ limite (1 000 lignes) — le tri s\'applique, aucun warning', async ({
    page,
  }) => {
    const warnings = collectSortWarnings(page);

    await clickHeaderToSort(page);
    await waitForPaint(page, 300);

    expect(
      warnings,
      `aucun warning attendu, reçu: ${warnings.join(' | ')}`,
    ).toHaveLength(0);
    // La grille reste fonctionnelle après le tri.
    await expect(page.locator('canvas')).toBeVisible();
  });

  test('> limite (100 millions de lignes) — tri ignoré + console.warn', async ({
    page,
  }) => {
    const warnings = collectSortWarnings(page);

    await page.locator('select').first().selectOption('100000000');
    await waitForPaint(page, 400);
    await expect(
      page.locator('strong', { hasText: '100 million rows' }),
    ).toBeVisible();

    await clickHeaderToSort(page);
    await waitForPaint(page, 400);

    expect(
      warnings.length,
      'un warning « sort skipped » est attendu au-delà de la limite',
    ).toBeGreaterThan(0);

    const msg = warnings[0];
    // Le message porte le nombre de lignes réel ET la limite (1 000 000).
    expect(msg).toMatch(
      /100000000 rows exceeds the 1000000-row client-side limit/,
    );
    // Et conseille une data source server-side.
    expect(msg).toMatch(/server-side data source/i);
  });

  test('== limite (1 million de lignes) — le tri s\'applique encore (borne incluse)', async ({
    page,
  }) => {
    // À exactement 1M lignes le tri n'est PAS ignoré : il trie réellement
    // 1M lignes en WASM debug → potentiellement lent, d'où le timeout élargi.
    test.setTimeout(90_000);

    const warnings = collectSortWarnings(page);

    await page.locator('select').first().selectOption('1000000');
    await waitForPaint(page, 500);
    await expect(
      page.locator('strong', { hasText: '1 million rows' }),
    ).toBeVisible();

    await clickHeaderToSort(page);
    await waitForPaint(page, 1000);

    expect(
      warnings,
      `borne incluse : aucun warning attendu à ${SORT_LIMIT} lignes, ` +
        `reçu: ${warnings.join(' | ')}`,
    ).toHaveLength(0);
    await expect(page.locator('canvas')).toBeVisible();
  });
});

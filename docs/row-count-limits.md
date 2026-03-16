# Limites du nombre de lignes — historique

## Résumé

| Époque | Type indice | Type coordonnées | Limite pratique | Raison |
|--------|-------------|------------------|-----------------|--------|
| v1 — f32 | `usize` | `f32` | ~300 000 lignes | précision `f32` |
| v2 — usize | `usize` | `f64` | ~4.3 milliards (wasm32) | `usize` 32-bit sur wasm32 |
| v3 — u64 | `u64` | `f64` | ~9 × 10¹⁴ lignes | précision `f64` du scroll |

---

## v1 — Coordonnées `f32`

Les positions de rendu (coordonnées canvas, `scroll_y`, `row_top`) étaient en `f32`.

`f32` a 23 bits de mantisse → représente exactement les entiers jusqu'à **2²⁴ = 16 777 216**.

Au-delà, l'epsilon grossit :

| `scroll_y` (f32) | epsilon    | lignes (28 px/row) | résultat           |
|------------------|------------|--------------------|--------------------|
| 8 × 10⁶          | ~1 px      | ~285 000           | limite scroll fluide |
| 4.7 × 10⁸        | ~64 px     | ~16.7 millions     | sauts majeurs      |

**Limite pratique v1 : ~300 000 lignes.**

---

## v2 — Indices `usize`, coordonnées `f64`

Les coordonnées passent en `f64` (precision scroll largement suffisante à cette échelle),
mais les indices de ligne restent `usize`.

Sur **wasm32**, `usize` est 32-bit → `usize::MAX = 4 294 967 295` (~4.3 milliards).
C'est ce plafond qui bloquait, pas la précision flottante.

Sur x86-64, `usize` est 64-bit donc pas de limite en natif — mais le code wasm32
dépassait `usize::MAX` silencieusement (overflow ou panic selon le build).

**Limite pratique v2 : ~4.3 milliards de lignes sur wasm32.**

---

## v3 — Indices `u64`, coordonnées `f64` (2026-03-16)

Les indices de ligne (`row_count`, `CellCoord.row`, `get_cell`, `row_top`,
`visible_rows`) passent de `usize` à `u64`. Les indices de colonne restent
`usize` (jamais > quelques centaines).

Le plafond n'est plus le type d'indice mais la précision de `scroll_y: f64`.
`f64` a 52 bits de mantisse → représente exactement les entiers jusqu'à **2⁵³**.

| `scroll_y` (f64) | epsilon f64   | lignes (28 px/row) | résultat              |
|------------------|---------------|--------------------|------------------------|
| 2.8 × 10¹⁰       | ~6 × 10⁻⁶ px | ~10⁹               | parfait                |
| 2.8 × 10¹³       | ~0.006 px     | ~10¹²              | OK                     |
| 2.8 × 10¹⁵       | ~0.6 px       | ~10¹⁴              | tremblement perceptible |
| 2.8 × 10¹⁶       | ~6 px         | ~10¹⁵              | sauts de lignes        |

### Test concret (2026-03-16, `row_height = 28`)

| `count`                      | valeur            | résultat                       |
|------------------------------|-------------------|--------------------------------|
| `9_007_199_254_740_99_u64`   | ~9 × 10¹⁴        | ✅ scroll et sélection OK      |
| `9_007_199_254_740_992_u64`  | 2⁵³ (~9 × 10¹⁵)  | ❌ cellules qui se chevauchent |

**Limite maximale recommandée : `9_007_199_254_740_99_u64`** (~9 × 10¹⁴ lignes).

---

## Valeurs dans les exemples

- `basic-web` : `10_000_000_000_u64` (10 milliards — confortable)
- `basic-leptos` : `9_007_199_254_740_99_u64` (limite maximale testée)

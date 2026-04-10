/**
 * generate_class_map.mjs
 *
 * Reads DaisyUI v5 theme variables (light) from the installed
 * node_modules and generates `class_map_data.rs` with exact
 * sRGB constants derived from oklch values.
 *
 * Usage:  node scripts/generate_class_map.mjs
 *         (or: npm run gen)
 *
 * Requires culori to be installed (npm install).
 */

import { createRequire } from 'module';
import { converter }     from 'culori';
import { writeFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { resolve, dirname } from 'path';

// ── Load DaisyUI theme variables ──────────────────────────

const require = createRequire(import.meta.url);
// object.js is an ESM-compiled module: { __esModule: true, default: { light: {...}, dark: {...}, ... } }
const themes = require('daisyui/theme/object.js');
const vars   = (themes.default ?? themes).light;

// ── oklch → sRGB conversion ───────────────────────────────

const toRgb = converter('rgb');

/**
 * Convert a DaisyUI oklch string to [r, g, b] (0-255).
 * culori accepts "oklch(45% 0.24 277.023)" directly.
 */
function oklchToRgb(str) {
    const rgb = toRgb(str);
    const clamp = (x) => Math.max(0, Math.min(255, Math.round((x ?? 0) * 255)));
    return [clamp(rgb?.r), clamp(rgb?.g), clamp(rgb?.b)];
}

// ── Unit helpers ──────────────────────────────────────────

/** "0.5rem" → 8.0 (16px base)  |  "1px" → 1.0 */
function toPx(val) {
    const s = String(val).trim();
    if (s.endsWith('rem')) return parseFloat(s) * 16;
    return parseFloat(s);
}

// ── DaisyUI badge geometry ────────────────────────────────
//
// DaisyUI v5 badge formula:
//   --size      = --size-selector × N
//   height      = --size
//   padding-inline = --size / 2 - --border
//   font sizes  : xs=10 sm=12 md=14 lg=16 xl=18 (px)
//   padding_y   = (height - font_size) / 2

const border  = toPx(vars['--border']);          // 1 px
const radSel  = toPx(vars['--radius-selector']); // 8 px
const sizeSel = toPx(vars['--size-selector']);   // 4 px

const FONT_PX = { 4: 10, 5: 12, 6: 14, 7: 16, 8: 18 };

function badgeSize(n) {
    const size = sizeSel * n;
    const px   = size / 2 - border;
    const fs   = FONT_PX[n];
    const py   = (size - fs) / 2;
    const fd   = fs - 14;   // delta relative to assumed base font 14px
    return { px, py, fd };
}

const [szXs, szSm, szMd, szLg, szXl] = [4, 5, 6, 7, 8].map(badgeSize);

// ── Color variants ────────────────────────────────────────

const VARIANTS = [
    'primary', 'secondary', 'accent',
    'success', 'error', 'warning', 'info', 'neutral',
];

function colorConst(name, oklchStr) {
    const [r, g, b] = oklchToRgb(oklchStr);
    return `Color::rgb(${r}, ${g}, ${b})`;
}

const colorLines = VARIANTS.flatMap(v => {
    const bg = colorConst(`${v.toUpperCase()}_BG`, vars[`--color-${v}`]);
    const fg = colorConst(`${v.toUpperCase()}_FG`, vars[`--color-${v}-content`]);
    return [
        `pub const ${v.toUpperCase()}_BG: Color = ${bg};`,
        `pub const ${v.toUpperCase()}_FG: Color = ${fg};`,
    ];
}).join('\n');

const [base200R, base200G, base200B] = oklchToRgb(vars['--color-base-200']);

// ── Emit Rust source ──────────────────────────────────────

function f(n) {
    // Format f64 literal: always include decimal point.
    return Number.isInteger(n) ? `${n}.0` : `${n}`;
}

const output = `\
// AUTO-GENERATED — do not edit by hand.
// Regenerate: just gen-class-map  (or: cd examples/basic-leptos && npm run gen)
// Source: daisyui/theme/object.js  light theme  (daisyui v${require('daisyui/package.json').version})
// Generated: ${new Date().toISOString().slice(0, 10)}

use crate::primitives::Color;

// ── Badge geometry ─────────────────────────────────────────────────────────
// Derived from DaisyUI v5 CSS variables:
//   --radius-selector = ${vars['--radius-selector']}  →  ${radSel}px
//   --border          = ${vars['--border']}           →  ${border}px
//   --size-selector   = ${vars['--size-selector']}  →  ${sizeSel}px
//
// Badge height per size: xs=${sizeSel*4} sm=${sizeSel*5} md=${sizeSel*6} lg=${sizeSel*7} xl=${sizeSel*8} (px)

pub const BADGE_RADIUS: f64 = ${f(radSel)};
pub const BADGE_BORDER: f64 = ${f(border)};

/// Per-size badge geometry: horizontal padding, vertical padding,
/// font-size delta relative to the 14px base font.
pub struct BadgeSz {
    pub px: f64,  // padding-inline (each side)
    pub py: f64,  // padding-block  (each side)
    pub fd: f64,  // font-size delta vs 14px base
}

pub const SZ_XS: BadgeSz = BadgeSz { px: ${f(szXs.px)}, py: ${f(szXs.py)}, fd: ${f(szXs.fd)} };
pub const SZ_SM: BadgeSz = BadgeSz { px: ${f(szSm.px)}, py: ${f(szSm.py)}, fd: ${f(szSm.fd)} };
pub const SZ_MD: BadgeSz = BadgeSz { px: ${f(szMd.px)}, py: ${f(szMd.py)}, fd: ${f(szMd.fd)} };
pub const SZ_LG: BadgeSz = BadgeSz { px: ${f(szLg.px)}, py: ${f(szLg.py)}, fd: ${f(szLg.fd)} };
pub const SZ_XL: BadgeSz = BadgeSz { px: ${f(szXl.px)}, py: ${f(szXl.py)}, fd: ${f(szXl.fd)} };

// ── Badge colors — oklch → sRGB ────────────────────────────────────────────
// Each variant: _BG = badge fill, _FG = text/content colour.

${colorLines}

// base-200 — used by badge-ghost (DaisyUI: bg-base-200 border-base-200)
pub const BASE_200: Color = Color::rgb(${base200R}, ${base200G}, ${base200B});
`;

// ── Write output ──────────────────────────────────────────

// scripts/ → basic-leptos/ → examples/ → rs-grid/ → crates/…
const outPath = resolve(
    dirname(fileURLToPath(import.meta.url)),
    '../../../crates/rs-grid-scene/src/class_map_data.rs',
);

writeFileSync(outPath, output, 'utf8');
console.log(`✓  class_map_data.rs written → ${outPath}`);

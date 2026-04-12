/**
 * generate_class_map.mjs — v2
 *
 * Reads DaisyUI v5 component sources directly from node_modules,
 * resolves CSS var() + calc() chains, converts oklch → sRGB, and
 * writes class_map_data.rs with one Rust module per component.
 *
 * Usage:  node scripts/generate_class_map.mjs  (or: npm run gen)
 */

import { createRequire } from 'module';
import { converter }     from 'culori';
import { writeFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { resolve, dirname } from 'path';

const __dir   = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);

// ── DaisyUI sources ───────────────────────────────────────────────────────────

const daisyPkg    = require('daisyui/package.json');
const daisyThemes = require('daisyui/theme/object.js');

/** All CSS custom properties for the DaisyUI light theme */
const THEME = (daisyThemes.default ?? daisyThemes).light;

// ── oklch → sRGB conversion ───────────────────────────────────────────────────

const toRgb = converter('rgb');

function oklchToRgb(str) {
    const rgb = toRgb(str);
    const c = x => Math.max(0, Math.min(255, Math.round((x ?? 0) * 255)));
    return [c(rgb?.r), c(rgb?.g), c(rgb?.b)];
}

// ── CSS var() + calc() resolver ───────────────────────────────────────────────

/** "0.5rem" → 8  |  "1px" → 1  |  "0.5em" → 8  |  "14" → 14  |  other → null */
function parseUnit(s) {
    s = String(s ?? '').trim();
    if (s.endsWith('rem')) return parseFloat(s) * 16;
    if (s.endsWith('px'))  return parseFloat(s);
    if (s.endsWith('em'))  return parseFloat(s) * 16; // 1em = 16px (html base)
    const n = parseFloat(s);
    return isNaN(n) ? null : n;
}

/**
 * Resolve a CSS value with var() / calc() to a number, RGB triple,
 * or raw string. `vars` is a flat map of --name → value.
 */
function resolveVal(val, vars, depth = 0) {
    if (depth > 8 || val == null) return null;
    val = String(val).trim();

    // var(--x) or var(--x, fallback)
    const vm = val.match(/^var\(--([^,)]+?)(?:\s*,\s*([\s\S]*?))?\)$/);
    if (vm) {
        const found = vars['--' + vm[1].trim()];
        if (found != null) return resolveVal(found, vars, depth + 1);
        if (vm[2])         return resolveVal(vm[2].trim(), vars, depth + 1);
        return null;
    }

    // calc(...)
    if (val.startsWith('calc(') && val.endsWith(')')) {
        return evalCalcStr(val.slice(5, -1), vars, depth + 1);
    }

    // oklch colour
    if (val.startsWith('oklch(')) return oklchToRgb(val);

    // unit value
    const n = parseUnit(val);
    if (n !== null) return n;

    return val; // raw string ("currentColor", "none", etc.)
}

/** Evaluate the inner expression of a calc() after resolving var()s */
function evalCalcStr(expr, vars, depth = 0) {
    let s = expr.replace(/var\(--([^,)]+?)(?:\s*,\s*([^)]*))?\)/g, (_, name, fb) => {
        let v = vars['--' + name.trim()];
        if (v == null && fb) v = fb.trim();
        if (v == null) return '0';
        const r = resolveVal(v, vars, depth + 1);
        if (typeof r === 'number') return r;
        return parseUnit(r) ?? 0;
    });
    s = s.replace(/([\d.]+)rem/g, (_, n) => String(parseFloat(n) * 16));
    s = s.replace(/([\d.]+)px/g,  (_, n) => n);
    try {
        return Function('"use strict"; return (' + s + ')')();
    } catch {
        return null;
    }
}

// ── Component object.js flattener ─────────────────────────────────────────────

/** Unwrap "@layer ..." wrapper and return the inner CSS-in-JS object */
function unwrapLayer(rules) {
    if (typeof rules !== 'object' || rules == null) return {};
    for (const [k, v] of Object.entries(rules)) {
        if (k.startsWith('@layer') && v && typeof v === 'object') return v;
    }
    return rules;
}

/**
 * Flatten a DaisyUI component object.js into a simple map:
 *   { '.badge': { 'border-radius': 'var(--radius-selector)', ... }, ... }
 * Only top-level class selectors are kept; @layer wrappers are stripped.
 */
function flattenComponent(obj) {
    const result = {};
    for (const [sel, val] of Object.entries(obj)) {
        if (!sel.startsWith('.')) continue;
        result[sel] = unwrapLayer(val);
    }
    return result;
}

// ── Data extraction helpers ───────────────────────────────────────────────────

const SEMANTIC   = ['primary', 'secondary', 'accent', 'success', 'error', 'warning', 'info', 'neutral'];
const SIZE_NAMES = ['xs', 'sm', 'md', 'lg', 'xl'];

/** Extract the numeric multiplier from a DaisyUI --size calc expression.
 *  "calc(var(--size-selector, 0.25rem) * 6)"  →  6  */
function sizeMultiplier(calcExpr) {
    const m = String(calcExpr ?? '').match(/\*\s*([\d.]+)/);
    return m ? parseFloat(m[1]) : null;
}

/**
 * Extract geometry from the base component class (.badge, .btn, etc.).
 * Returns { radius, borderWidth, fontSize, paddingX, paddingY }.
 */
function extractGeometry(baseRules) {
    const vars = { ...THEME };

    // border-radius — also check logical property fallback used by btn
    const rRaw = baseRules['border-radius']
        ?? baseRules['border-start-start-radius'];
    let radius = 0;
    if (rRaw) {
        const r = resolveVal(rRaw, vars);
        radius = typeof r === 'number' ? r : (parseUnit(r) ?? 0);
    }

    // border width (read from global --border)
    const borderWidth = parseUnit(THEME['--border'] ?? '1px') ?? 1;

    // font-size
    let fontSize = 14; // 0.875rem default
    const fsRaw = baseRules['font-size'];
    if (fsRaw) {
        const r = resolveVal(fsRaw, vars);
        fontSize = typeof r === 'number' ? r : (parseUnit(r) ?? 14);
    }

    // padding-inline — resolve with MD height injected as --size
    let paddingX = 0;
    const pxRaw = baseRules['padding-inline'];
    if (pxRaw) {
        const sizeSel = parseUnit(THEME['--size-selector'] ?? '0.25rem') ?? 4;
        const sizeField = parseUnit(THEME['--size-field'] ?? '0.25rem') ?? 4;
        // Try both; inject the MD height as --size
        const mdSizeSel = sizeSel * 6;
        const mdSizeField = sizeField * 10;
        const pxVarsSel   = { ...vars, '--size': String(mdSizeSel) };
        const pxVarsField = { ...vars, '--size': String(mdSizeField) };
        let r = resolveVal(pxRaw, pxVarsSel);
        if (r == null || isNaN(r)) r = resolveVal(pxRaw, pxVarsField);
        paddingX = typeof r === 'number' ? r : 0;
    }

    // padding-block
    let paddingY = 0;
    const pyRaw = baseRules['padding-block'] ?? baseRules['padding'];
    if (pyRaw) {
        const r = resolveVal(pyRaw, vars);
        paddingY = typeof r === 'number' ? r : (parseUnit(r) ?? 0);
    }

    return { radius, borderWidth, fontSize, paddingX, paddingY };
}

/**
 * Try to detect and compute the DaisyUI size system for a component.
 * Looks for .{comp}-xs … .{comp}-xl with --size: calc(... * N).
 * Returns { xs, sm, md, lg, xl } → { px, py, fd, height, fontSize }
 * or null if no size system is found.
 */
function extractSizes(flat, compName) {
    const base = flat[`.${compName}`] ?? {};

    // Check that there's a --size variable in the base or a -xs class
    const baseSizeExpr = base['--size'] ?? '';
    if (!baseSizeExpr && !flat[`.${compName}-xs`]) return null;

    // Detect size-selector vs size-field
    const usesField = String(baseSizeExpr).includes('size-field');
    const sizeBase  = parseUnit(
        THEME[usesField ? '--size-field' : '--size-selector'] ?? '0.25rem'
    ) ?? 4;
    const border = parseUnit(THEME['--border'] ?? '1px') ?? 1;

    // DaisyUI default font sizes per tier (rem → px)
    const FONT_SIZES = { xs: 10, sm: 12, md: 14, lg: 16, xl: 18 };
    const BASE_FONT  = 14;

    // Default multipliers for size-selector-based components
    const DEFAULT_MULT_SEL   = { xs: 4, sm: 5, md: 6, lg: 7, xl: 8  };
    // Default multipliers for size-field-based components (btn)
    const DEFAULT_MULT_FIELD = { xs: 6, sm: 8, md: 10, lg: 12, xl: 14 };
    const defaultMult = usesField ? DEFAULT_MULT_FIELD : DEFAULT_MULT_SEL;

    const result = {};
    for (const sz of SIZE_NAMES) {
        const cls = flat[`.${compName}-${sz}`];
        let mult = defaultMult[sz];

        // Read multiplier from component class if it exists
        if (cls?.['--size']) {
            mult = sizeMultiplier(cls['--size']) ?? mult;
        } else if (sz === 'md' && baseSizeExpr) {
            mult = sizeMultiplier(baseSizeExpr) ?? mult;
        }

        // Font size: use component-defined value if available
        let fontSize = FONT_SIZES[sz];
        if (cls?.['font-size']) {
            const r = resolveVal(cls['font-size'], THEME);
            if (typeof r === 'number' && r > 0) fontSize = r;
        }

        const height = sizeBase * mult;
        const px     = height / 2 - border;            // padding-inline each side
        const py     = Math.max(0, (height - fontSize) / 2); // padding-block each side
        const fd     = fontSize - BASE_FONT;            // font-size delta

        result[sz] = { px, py, fd, height, fontSize };
    }

    return result;
}

/**
 * Detect colour variants for a component.
 * Looks for .{comp}-{color} classes. Uses theme oklch values as ground truth.
 * Returns { primary: { bg:[r,g,b], fg:[r,g,b] }, ... }
 */
function extractColors(flat, compName) {
    const result = {};
    for (const color of SEMANTIC) {
        if (!flat[`.${compName}-${color}`]) continue;
        const bg = oklchToRgb(THEME[`--color-${color}`]         ?? 'oklch(50% 0 0)');
        const fg = oklchToRgb(THEME[`--color-${color}-content`] ?? 'oklch(100% 0 0)');
        result[color] = { bg, fg };
    }
    return result;
}

// ── Rust code generator ───────────────────────────────────────────────────────

/** Format a number as a Rust f64 literal */
function f(n) {
    if (n == null || isNaN(n)) return '0.0';
    const r = Math.round(n * 100) / 100;
    return Number.isInteger(r) ? `${r}.0` : `${r}`;
}

function rustColor(rgb) {
    return `Color::rgb(${rgb[0]}, ${rgb[1]}, ${rgb[2]})`;
}

function generateModule(compName, dirName, { geometry, colors, sizes }) {
    const lines = [];
    const hasSizes  = sizes  && Object.keys(sizes).length  > 0;
    const hasColors = colors && Object.keys(colors).length > 0;

    lines.push(`/// Auto-generated from daisyui v${daisyPkg.version} \`components/${dirName}/object.js\``);
    lines.push(`pub mod ${compName} {`);
    if (hasColors) {
        lines.push(`    use rs_grid_scene::primitives::Color;`);
        lines.push('');
    }
    // ── Geometry ──────────────────────────────────────────────────────────────
    lines.push(`    // ── Geometry ──────────────────────────────────────────────────────────`);
    lines.push(`    pub const RADIUS: f64 = ${f(geometry.radius)};`);
    lines.push(`    pub const BORDER: f64 = ${f(geometry.borderWidth)};`);
    if (geometry.paddingX) lines.push(`    pub const PADDING_X: f64 = ${f(geometry.paddingX)};`);
    if (geometry.paddingY) lines.push(`    pub const PADDING_Y: f64 = ${f(geometry.paddingY)};`);
    lines.push(`    pub const FONT_SIZE: f64 = ${f(geometry.fontSize)};`);

    // ── Size system ───────────────────────────────────────────────────────────
    if (hasSizes) {
        lines.push('');
        lines.push(`    // ── Size system ───────────────────────────────────────────────────────`);
        lines.push(`    /// Per-size geometry: padding-inline (px), padding-block (py),`);
        lines.push(`    /// and font-size delta relative to the 14px base (fd).`);
        lines.push(`    pub struct Sz { pub px: f64, pub py: f64, pub fd: f64 }`);
        lines.push('');
        for (const [sz, v] of Object.entries(sizes)) {
            lines.push(`    /// h=${f(v.height)}px  font=${f(v.fontSize)}px`);
            lines.push(`    pub const ${sz.toUpperCase()}: Sz = Sz { px: ${f(v.px)}, py: ${f(v.py)}, fd: ${f(v.fd)} };`);
        }
    }

    // ── Colour variants ───────────────────────────────────────────────────────
    if (hasColors) {
        lines.push('');
        lines.push(`    // ── Colour variants (oklch → sRGB, light theme) ──────────────────────`);
        lines.push(`    // _BG = badge/component fill  |  _FG = text/content colour`);
        for (const [color, { bg, fg }] of Object.entries(colors)) {
            lines.push(`    pub const ${color.toUpperCase()}_BG: Color = ${rustColor(bg)};`);
            lines.push(`    pub const ${color.toUpperCase()}_FG: Color = ${rustColor(fg)};`);
        }
        // ghost / base-200 colour available for components that use it
        lines.push(`    pub const GHOST_BG: Color = ${rustColor(oklchToRgb(THEME['--color-base-200']))};`);
    }

    lines.push('}');
    return lines.join('\n');
}

// ── Canvas-renderable components ──────────────────────────────────────────────
//
// We process only components that make sense inside a canvas cell.
// Layout/modal/navigation components are intentionally excluded.

// name = CSS class prefix; dir = daisyui components/<dir>/object.js path
const CANVAS_COMPONENTS = [
    { name: 'badge',    dir: 'badge'    }, // inline annotation
    { name: 'btn',      dir: 'button'   }, // action button (dir ≠ name)
    { name: 'alert',    dir: 'alert'    }, // row-level banner
    { name: 'kbd',      dir: 'kbd'      }, // keyboard shortcut label
    { name: 'status',   dir: 'status'   }, // status dot
    { name: 'progress', dir: 'progress' }, // progress bar
];

// ── Main ──────────────────────────────────────────────────────────────────────

console.log(`DaisyUI v${daisyPkg.version} — generating class_map_data.rs\n`);

const modules = [];

for (const { name, dir } of CANVAS_COMPONENTS) {
    let raw;
    try {
        raw = require(`daisyui/components/${dir}/object.js`);
    } catch {
        console.log(`  ⟶ ${name} (${dir}): component not found, skipped`);
        continue;
    }

    const obj      = raw.default ?? raw;
    const flat     = flattenComponent(obj);
    const base     = flat[`.${name}`] ?? {};

    const geometry = extractGeometry(base);
    const colors   = extractColors(flat, name);
    const sizes    = extractSizes(flat, name);

    const nColors = Object.keys(colors).length;
    const nSizes  = sizes ? Object.keys(sizes).length : 0;

    // Skip if there's nothing canvas-useful to emit
    if (nColors === 0 && nSizes === 0 && !geometry.radius) {
        console.log(`  ⟶ ${name}: nothing canvas-renderable, skipped`);
        continue;
    }

    modules.push(generateModule(name, dir, { geometry, colors, sizes }));
    console.log(`✓  ${name}: radius=${f(geometry.radius)}px  ${nColors} colors  ${nSizes} sizes`);
}

// ── Assemble output ───────────────────────────────────────────────────────────

const sharedColorLines = SEMANTIC.flatMap(c => {
    const bg = oklchToRgb(THEME[`--color-${c}`]);
    const fg = oklchToRgb(THEME[`--color-${c}-content`]);
    return [
        `pub const ${c.toUpperCase()}_BG: Color = ${rustColor(bg)};`,
        `pub const ${c.toUpperCase()}_FG: Color = ${rustColor(fg)};`,
    ];
});

const output = [
    `// AUTO-GENERATED — do not edit by hand.`,
    `// Regenerate: just gen-class-map  (or: cd examples/basic-leptos && npm run gen)`,
    `// Source: daisyui/theme/object.js + daisyui/components/*/object.js`,
    `//         Light theme — daisyui v${daisyPkg.version}`,
    `// Generated: ${new Date().toISOString().slice(0, 10)}`,
    ``,
    `use rs_grid_scene::primitives::Color;`,
    ``,
    `// ── Shared semantic colours (light theme, same across all components) ────────`,
    `// Derived from oklch() values in daisyui/theme/object.js`,
    ...sharedColorLines,
    ``,
    `/// base-200 colour — used by ghost variants and skeleton`,
    `pub const BASE_200:     Color = ${rustColor(oklchToRgb(THEME['--color-base-200']))};`,
    `/// base-content — dark text on light backgrounds`,
    `pub const BASE_CONTENT: Color = ${rustColor(oklchToRgb(THEME['--color-base-content']))};`,
    `/// Global border width from --border`,
    `pub const BORDER_W:     f64   = ${f(parseUnit(THEME['--border'] ?? '1px'))};`,
    ``,
    `// ── Per-component geometry modules ────────────────────────────────────────────`,
    `// Each module contains RADIUS, BORDER, FONT_SIZE, optional Sz size table,`,
    `// and colour constants for variants available on that component.`,
    `// Colours are redundant with the shared constants above but are provided`,
    `// for ergonomic access via e.g. badge::SUCCESS_BG.`,
    ``,
    modules.join('\n\n'),
].join('\n');

const outPath = resolve(__dir, '../../../examples/example-common/src/class_map_data.rs');
writeFileSync(outPath, output, 'utf8');
console.log(`\n✓  Written → ${outPath}`);

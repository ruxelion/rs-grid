//! Footer bar: row counts and rows-per-page selector on the left,
//! pagination on the right.
//!
//! Drawn below the scrollable body (`[vp.height, vp.height + footer_h)`)
//! using only existing primitives. Clickable regions are registered as
//! [`FooterZone`]s so the web layer never recomputes this geometry.
//!
//! Visual reference: daisyUI. The page strip is a `join` of `btn btn-sm`
//! (one rounded group with inner dividers, the current
//! page filled with the primary color); the rows-per-page box is a
//! `select-sm`; the selected-rows count is a soft primary `badge`. The bar
//! mirrors the header band (same background and border by default) and
//! its controls use the grid background, like the filter row's input
//! boxes. Every color and size comes from the `Theme`
//! (`--rs-grid-footer-*`).
//!
//! Layout is driven by real text widths when the host provides a
//! [`TextMeasurer`] (the web layer does, via Canvas2D `measureText`), and
//! by a per-character estimate otherwise (native tests, scene dumps).
//! When the bar is too narrow, items are dropped in order of importance:
//! the range label, the rows-per-page label, the page numbers, then the
//! selected-rows badge.

use rs_grid_core::{
    pagination::{PageItem, group_thousands, page_items},
    state::GridState,
};

use super::TextMeasurer;
use crate::{
    frame::{FooterAction, FooterZone, SceneFrame},
    primitives::{
        Color, LinePrimitive, RectPrimitive, ScenePrimitive, TextAlign,
        TextPrimitive,
    },
    theme::Theme,
};

/// Gap between two unrelated groups (counts | selector | pagination).
const GROUP_GAP: f64 = 20.0;
/// Gap between two items of the same group.
const ITEM_GAP: f64 = 12.0;
/// Gap between a label and the control it describes.
const LABEL_GAP: f64 = 8.0;
/// Horizontal padding inside the selected-rows badge.
const BADGE_PAD_X: f64 = 10.0;
/// Horizontal padding inside a page-number button.
const PAGE_PAD_X: f64 = 9.0;
/// Start padding of the rows-per-page box.
const SELECT_PAD_START: f64 = 10.0;
/// End padding of the rows-per-page box (room for the caret).
const SELECT_PAD_END: f64 = 26.0;
/// Half the height of a page chevron glyph.
const CHEVRON_HALF_H: f64 = 4.0;
/// Horizontal reach of a page chevron glyph.
const CHEVRON_W: f64 = 4.0;
/// Stroke width of the chevron glyphs.
const CHEVRON_STROKE: f64 = 1.5;

/// Rough advance of `ch` as a fraction of the font size, for a typical
/// UI sans-serif. Only used when no [`TextMeasurer`] is available.
fn char_advance(ch: char) -> f64 {
    match ch {
        'i' | 'j' | 'l' | 'I' | '.' | ',' | ':' | ';' | '\'' | '|' | '!' => {
            0.26
        }
        ' ' | '\u{a0}' | '\u{202f}' | 'f' | 't' | 'r' | '(' | ')' => 0.32,
        'm' | 'w' | 'M' | 'W' => 0.82,
        '0'..='9' => 0.54,
        'A'..='Z' => 0.64,
        '\u{2013}' | '\u{2014}' | '\u{2026}' => 0.6,
        c if (c as u32) >= 0x2e80 => 1.0,
        _ => 0.52,
    }
}

fn estimate_width(text: &str, font_size: f64) -> f64 {
    text.chars().map(char_advance).sum::<f64>() * font_size
}

/// One button of the page strip.
struct StripButton {
    kind: StripKind,
    width: f64,
    enabled: bool,
    action: Option<FooterAction>,
}

enum StripKind {
    First,
    Prev,
    Next,
    Last,
    Page { label: String, current: bool },
    Ellipsis,
}

/// The page strip's buttons, left to right. `compact` keeps only
/// previous / current page / next.
fn build_strip(
    current: u64,
    pages: u64,
    compact: bool,
    btn: f64,
    width_of: &dyn Fn(&str, bool) -> f64,
) -> Vec<StripButton> {
    let at_first = current == 0;
    let at_last = current + 1 >= pages;
    let nav = |kind, enabled: bool, action| StripButton {
        kind,
        width: btn,
        enabled,
        action: enabled.then_some(action),
    };
    let page = |p: u64| {
        let label = group_thousands(p + 1);
        let is_current = p == current;
        let w = (width_of(&label, is_current) + PAGE_PAD_X * 2.0)
            .max(btn)
            .round();
        StripButton {
            kind: StripKind::Page {
                label,
                current: is_current,
            },
            width: w,
            enabled: true,
            action: (!is_current).then_some(FooterAction::GoToPage(p)),
        }
    };

    let mut strip = Vec::new();
    if !compact {
        strip.push(nav(StripKind::First, !at_first, FooterAction::FirstPage));
    }
    strip.push(nav(StripKind::Prev, !at_first, FooterAction::PrevPage));
    if compact {
        strip.push(page(current));
    } else {
        for item in page_items(current, pages) {
            strip.push(match item {
                PageItem::Page(p) => page(p),
                PageItem::Ellipsis => StripButton {
                    kind: StripKind::Ellipsis,
                    width: btn,
                    enabled: false,
                    action: None,
                },
            });
        }
    }
    strip.push(nav(StripKind::Next, !at_last, FooterAction::NextPage));
    if !compact {
        strip.push(nav(StripKind::Last, !at_last, FooterAction::LastPage));
    }
    strip
}

/// Fill one segment of a joined group, rounding only the group's outer
/// corners. A rounded rect that overshoots the segment on its inner side
/// is clipped back to the segment, so a translucent fill never overlaps
/// itself.
#[allow(clippy::too_many_arguments)]
fn push_segment(
    frame: &mut SceneFrame,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    radius: f64,
    first: bool,
    last: bool,
    fill: Color,
) {
    let (rx, rw, corner, clip) = match (first, last) {
        (true, true) => (x, w, radius, None),
        (true, false) => (x, w + radius, radius, Some([x, y, w, h])),
        (false, true) => (x - radius, w + radius, radius, Some([x, y, w, h])),
        (false, false) => (x, w, 0.0, None),
    };
    frame.push(ScenePrimitive::Rect(RectPrimitive {
        x: rx,
        y,
        width: rw,
        height: h,
        fill,
        stroke: None,
        stroke_width: 0.0,
        corner_radius: corner,
        clip,
    }));
}

pub(super) fn emit_footer(
    frame: &mut SceneFrame,
    state: &GridState,
    t: &Theme,
    measure: Option<&TextMeasurer>,
    hovered: Option<FooterAction>,
) {
    let model = &state.model;
    let footer_h = model.effective_footer_height();
    if footer_h <= 0.0 {
        return;
    }
    let width = state.viewport.width;
    let top = state.viewport.height;
    let bw = t.footer_border_width;
    let fs = t.footer_font_size;
    let pad = t.cell_padding;
    let radius = t.footer_button_radius;
    let btn_h = t.footer_button_height.min((footer_h - 4.0).max(8.0));
    let btn_y = (top + (footer_h - btn_h) * 0.5).round();
    let mid_y = btn_y + btn_h * 0.5;
    // 0.35 ≈ half cap-height for system-ui (same convention as cells).
    let text_y = mid_y + fs * 0.35;
    let labels = &model.footer_labels;
    let transparent = Color::rgba(0, 0, 0, 0);

    // Like the header's labels, every footer text is bold by default.
    let all_bold = t.footer_font_bold;
    let width_of = |s: &str, bold: bool| -> f64 {
        let bold = bold || all_bold;
        match measure {
            Some(m) => m(s, fs, bold),
            None => estimate_width(s, fs),
        }
    };
    let text =
        |x: f64, s: String, color: Color, align: TextAlign, bold: bool| {
            ScenePrimitive::Text(TextPrimitive {
                x,
                y: text_y,
                text: s,
                color,
                font_size: fs,
                bold: bold || all_bold,
                italic: false,
                clip: None,
                align,
                max_width: None,
            })
        };

    // ── background + divider ────────────────────────────────
    frame.push(ScenePrimitive::Rect(RectPrimitive {
        x: 0.0,
        y: top,
        width,
        height: footer_h,
        fill: t.footer_bg,
        stroke: None,
        stroke_width: 0.0,
        corner_radius: 0.0,
        clip: None,
    }));
    frame.push(ScenePrimitive::Line(LinePrimitive {
        x1: 0.0,
        y1: top + bw * 0.5,
        x2: width,
        y2: top + bw * 0.5,
        color: t.footer_border,
        width: bw,
    }));

    // ── measure everything first ────────────────────────────
    let total = model.display_row_count();
    let rows_text = labels.rows_text(total);
    let rows_w = width_of(&rows_text, false);

    let selected = state.selected_row_count();
    let has_selection = selected > 0;
    let badge_text = labels.selected_text(selected);
    let badge_h = (btn_h - 8.0).max(fs + 4.0);
    let badge_w =
        (width_of(&badge_text, has_selection) + BADGE_PAD_X * 2.0).round();

    let paginated = model.pagination_enabled;
    let label = labels.rows_per_page.clone();
    let label_w = width_of(&label, false);
    let value = group_thousands(model.page_size);
    let select_w =
        (width_of(&value, false) + SELECT_PAD_START + SELECT_PAD_END)
            .max(btn_h * 1.75)
            .round();

    let pages = model.page_count();
    let current = model.current_page();
    let (start, end) = model.page_window();
    let range_text =
        labels.range_text(if end > start { start + 1 } else { 0 }, end, total);
    let range_w = width_of(&range_text, false);

    // ── fit: drop the least important items until it all fits ──
    let mut show_range = paginated;
    let mut show_label = paginated;
    let mut compact = false;
    let mut show_badge = true;
    let mut strip = if paginated {
        build_strip(current, pages, compact, btn_h, &width_of)
    } else {
        Vec::new()
    };
    for step in 0..=4 {
        let strip_w: f64 = strip.iter().map(|b| b.width).sum();
        let mut need = pad + rows_w + pad;
        if show_badge {
            need += ITEM_GAP + badge_w;
        }
        if paginated {
            need += GROUP_GAP * 2.0 + select_w + GROUP_GAP + strip_w;
            if show_label {
                need += label_w + LABEL_GAP;
            }
            if show_range {
                need += range_w + ITEM_GAP;
            }
        }
        if need <= width || step == 4 {
            break;
        }
        match step {
            0 => show_range = false,
            1 => show_label = false,
            2 => {
                compact = true;
                strip = build_strip(current, pages, compact, btn_h, &width_of);
            }
            _ => show_badge = false,
        }
    }

    // ── left: counts ────────────────────────────────────────
    let mut x = pad;
    frame.push(text(x, rows_text, t.footer_text, TextAlign::Left, false));
    x += rows_w;

    if show_badge {
        x = (x + ITEM_GAP).round();
        if has_selection {
            frame.push(ScenePrimitive::Rect(RectPrimitive {
                x,
                y: (mid_y - badge_h * 0.5).round(),
                width: badge_w,
                height: badge_h,
                fill: t.footer_badge_bg,
                stroke: None,
                stroke_width: 0.0,
                // Same radius as the footer's buttons, not a pill: every
                // rounded element of the bar shares one radius.
                corner_radius: t.footer_button_radius,
                clip: None,
            }));
        }
        let color = if has_selection {
            t.footer_badge_text
        } else {
            t.footer_text_muted
        };
        frame.push(text(
            x + badge_w * 0.5,
            badge_text,
            color,
            TextAlign::Center,
            has_selection,
        ));
        x += badge_w;
    }

    if !paginated {
        return;
    }

    // ── left: rows-per-page selector (daisyUI `select-sm`) ──
    x += GROUP_GAP;
    let sep_x = x.floor() + 0.5;
    frame.push(ScenePrimitive::Line(LinePrimitive {
        x1: sep_x,
        y1: mid_y - 8.0,
        x2: sep_x,
        y2: mid_y + 8.0,
        color: t.footer_border,
        width: bw,
    }));
    x += GROUP_GAP;
    if show_label {
        frame.push(text(x, label, t.footer_text_muted, TextAlign::Left, false));
        x += label_w + LABEL_GAP;
    }
    let select_x = x.round();
    let select_hovered = hovered == Some(FooterAction::PageSizeMenu);
    frame.push(ScenePrimitive::Rect(RectPrimitive {
        x: select_x,
        y: btn_y,
        width: select_w,
        height: btn_h,
        fill: t.footer_button_bg,
        stroke: Some(t.footer_button_border),
        stroke_width: bw,
        corner_radius: radius,
        clip: None,
    }));
    if select_hovered {
        frame.push(ScenePrimitive::Rect(RectPrimitive {
            x: select_x + bw,
            y: btn_y + bw,
            width: select_w - bw * 2.0,
            height: btn_h - bw * 2.0,
            fill: t.footer_button_hover_bg,
            stroke: None,
            stroke_width: 0.0,
            corner_radius: (radius - bw).max(0.0),
            clip: None,
        }));
    }
    frame.push(text(
        select_x + SELECT_PAD_START,
        value,
        t.footer_text,
        TextAlign::Left,
        false,
    ));
    emit_chevron(
        frame,
        select_x + select_w - 14.0,
        mid_y + 0.5,
        ChevronDir::Down,
        t.footer_text_muted,
    );
    frame.push_footer_zone(FooterZone {
        action: FooterAction::PageSizeMenu,
        x: select_x,
        y: btn_y,
        width: select_w,
        height: btn_h,
    });

    // ── right: range label + page strip (daisyUI `join`) ────
    let strip_w: f64 = strip.iter().map(|b| b.width).sum();
    let strip_x = (width - pad - strip_w).round();

    if show_range {
        frame.push(text(
            strip_x - ITEM_GAP - 4.0,
            range_text,
            t.footer_text_muted,
            TextAlign::Right,
            false,
        ));
    }

    // Soft fill for the whole group.
    frame.push(ScenePrimitive::Rect(RectPrimitive {
        x: strip_x,
        y: btn_y,
        width: strip_w,
        height: btn_h,
        fill: t.footer_button_bg,
        stroke: None,
        stroke_width: 0.0,
        corner_radius: radius,
        clip: None,
    }));

    // Hover fill, under the dividers and the outline.
    let last_idx = strip.len().saturating_sub(1);
    let mut bx = strip_x;
    for (i, b) in strip.iter().enumerate() {
        if b.action.is_some() && b.action == hovered {
            push_segment(
                frame,
                bx,
                btn_y,
                b.width,
                btn_h,
                radius,
                i == 0,
                i == last_idx,
                t.footer_button_hover_bg,
            );
        }
        bx += b.width;
    }

    // Inner dividers, then one outline around the group.
    let mut dx = strip_x;
    for b in strip.iter().take(last_idx) {
        dx += b.width;
        let lx = dx - bw * 0.5;
        frame.push(ScenePrimitive::Line(LinePrimitive {
            x1: lx,
            y1: btn_y + bw,
            x2: lx,
            y2: btn_y + btn_h - bw,
            color: t.footer_button_border,
            width: bw,
        }));
    }
    frame.push(ScenePrimitive::Rect(RectPrimitive {
        x: strip_x,
        y: btn_y,
        width: strip_w,
        height: btn_h,
        fill: transparent,
        stroke: Some(t.footer_button_border),
        stroke_width: bw,
        corner_radius: radius,
        clip: None,
    }));

    // Current page fill (over the outline and its neighbour dividers),
    // glyphs, labels and click zones.
    let mut bx = strip_x;
    for (i, b) in strip.iter().enumerate() {
        let cx = bx + b.width * 0.5;
        let ink = if b.enabled {
            t.footer_text
        } else {
            t.footer_disabled_text
        };
        match &b.kind {
            StripKind::Page { label, current } => {
                if *current {
                    let first = i == 0;
                    let last = i == last_idx;
                    // Cover the dividers on both inner sides.
                    let x0 = if first { bx } else { bx - bw };
                    let x1 = bx + b.width;
                    push_segment(
                        frame,
                        x0,
                        btn_y,
                        x1 - x0,
                        btn_h,
                        radius,
                        first,
                        last,
                        t.footer_page_active_bg,
                    );
                }
                let color = if *current {
                    t.footer_page_active_text
                } else {
                    ink
                };
                frame.push(text(
                    cx,
                    label.clone(),
                    color,
                    TextAlign::Center,
                    *current,
                ));
            }
            StripKind::Ellipsis => frame.push(text(
                cx,
                "\u{2026}".to_owned(),
                ink,
                TextAlign::Center,
                false,
            )),
            StripKind::First => {
                emit_chevron(frame, cx - 2.5, mid_y, ChevronDir::Left, ink);
                emit_chevron(frame, cx + 2.5, mid_y, ChevronDir::Left, ink);
            }
            StripKind::Last => {
                emit_chevron(frame, cx - 2.5, mid_y, ChevronDir::Right, ink);
                emit_chevron(frame, cx + 2.5, mid_y, ChevronDir::Right, ink);
            }
            StripKind::Prev => {
                emit_chevron(frame, cx, mid_y, ChevronDir::Left, ink);
            }
            StripKind::Next => {
                emit_chevron(frame, cx, mid_y, ChevronDir::Right, ink);
            }
        }
        if let Some(action) = b.action {
            frame.push_footer_zone(FooterZone {
                action,
                x: bx,
                y: btn_y,
                width: b.width,
                height: btn_h,
            });
        }
        bx += b.width;
    }
}

#[derive(Clone, Copy)]
enum ChevronDir {
    Left,
    Right,
    Down,
}

/// A two-segment chevron centered on `(cx, cy)`.
fn emit_chevron(
    frame: &mut SceneFrame,
    cx: f64,
    cy: f64,
    dir: ChevronDir,
    color: Color,
) {
    let hw = CHEVRON_W * 0.5;
    let (p0, p1, p2) = match dir {
        ChevronDir::Left => (
            (cx + hw, cy - CHEVRON_HALF_H),
            (cx - hw, cy),
            (cx + hw, cy + CHEVRON_HALF_H),
        ),
        ChevronDir::Right => (
            (cx - hw, cy - CHEVRON_HALF_H),
            (cx + hw, cy),
            (cx - hw, cy + CHEVRON_HALF_H),
        ),
        // The select caret is flatter than the page chevrons.
        ChevronDir::Down => {
            ((cx - 3.5, cy - 2.0), (cx, cy + 1.5), (cx + 3.5, cy - 2.0))
        }
    };
    for (a, b) in [(p0, p1), (p1, p2)] {
        frame.push(ScenePrimitive::Line(LinePrimitive {
            x1: a.0,
            y1: a.1,
            x2: b.0,
            y2: b.1,
            color,
            width: CHEVRON_STROKE,
        }));
    }
}

use std::collections::HashSet;

use rs_grid_core::{
    column::{ButtonStyle, ColumnDef},
    datasource::CellStatus,
    format::{CellAlign, CellElement, CellFormat, format_cell},
    model::GridModel,
    selection::SelectionState,
};

use super::FlashHint;
use crate::{
    class_map::ClassResolver,
    frame::SceneFrame,
    primitives::{
        Color, ImagePrimitive, RectPrimitive, ScenePrimitive, TextAlign,
        TextPrimitive,
    },
    theme::Theme,
};

/// Emit selection fill, search highlight, and cell content
/// (text, image, or skeleton) for a single cell.
///
/// Shared by the scrollable-column and pinned-column render
/// loops to avoid duplicating logic.
#[allow(clippy::too_many_arguments)]
pub(super) fn emit_cell(
    frame: &mut SceneFrame,
    col: &ColumnDef,
    model: &GridModel,
    ri: u64,
    ci: usize,
    cx: f64,
    ry: f64,
    mid_y: f64,
    row_height: f64,
    cell_status: CellStatus,
    sel: &SelectionState,
    search_set: &HashSet<(u64, usize)>,
    search_current: Option<(u64, usize)>,
    t: &Theme,
    flash: Option<&FlashHint>,
    class_resolver: Option<&ClassResolver>,
    invalid_borders: &mut Vec<(f64, f64, f64, f64)>,
) {
    // Selection fill (no border — outer border drawn separately)
    if sel.is_selected(ri, ci) {
        frame.push(ScenePrimitive::Rect(RectPrimitive {
            x: cx,
            y: ry,
            width: col.width,
            height: row_height,
            fill: t.selection_fill,
            stroke: None,
            stroke_width: 0.0,
            corner_radius: 0.0,
            clip: None,
        }));
    }

    // Flash overlay — themed fade on paste, restricted to the cells
    // actually written (`flash.cells`), not the whole selection
    // rectangle, which may extend past cells skipped for being
    // locked or failing validation.
    if let Some(f) = flash
        && f.cells.contains(&(ri, ci))
    {
        let base = if f.is_error {
            t.flash_error_fill
        } else {
            t.flash_fill
        };
        let a = (base.a as f64 * f.alpha_factor).round() as u8;
        frame.push(ScenePrimitive::Rect(RectPrimitive {
            x: cx,
            y: ry,
            width: col.width,
            height: row_height,
            fill: Color::rgba(base.r, base.g, base.b, a),
            stroke: None,
            stroke_width: 0.0,
            corner_radius: 0.0,
            clip: None,
        }));
    }

    // Search highlight
    if search_set.contains(&(ri, ci)) {
        let fill = if search_current == Some((ri, ci)) {
            t.search_current
        } else {
            t.search_highlight
        };
        frame.push(ScenePrimitive::Rect(RectPrimitive {
            x: cx,
            y: ry,
            width: col.width,
            height: row_height,
            fill,
            stroke: None,
            stroke_width: 0.0,
            corner_radius: 0.0,
            clip: None,
        }));
    }

    // Locked-cell overlay (static `editable=false` or a
    // false-resolving `editable_predicate`). Skipped when fully
    // transparent to avoid an extra draw call (mirrors the
    // `row_hover_bg` "transparent = disabled" convention).
    let locked = !col.is_cell_editable(ri, model);
    if locked && t.locked_cell_bg.a > 0 {
        frame.push(ScenePrimitive::Rect(RectPrimitive {
            x: cx,
            y: ry,
            width: col.width,
            height: row_height,
            fill: t.locked_cell_bg,
            stroke: None,
            stroke_width: 0.0,
            corner_radius: 0.0,
            clip: None,
        }));
    }

    // At-rest invalid-value background/border — a cell can fail
    // `ColumnDef.rules`/`validator` without ever being edited (e.g.
    // loaded that way from the data source), so this doesn't wait for
    // an active edit session the way the DOM editor's invalid style
    // does. Background and border are independent primitives (either,
    // both, or neither can be themed on), each skipped when fully
    // transparent (same convention as `locked_cell_bg` above).
    let invalid = matches!(
        &cell_status,
        CellStatus::Ready(raw) if col.validate_value(raw).is_err()
    );
    if invalid && t.invalid_cell_bg.a > 0 {
        frame.push(ScenePrimitive::Rect(RectPrimitive {
            x: cx,
            y: ry,
            width: col.width,
            height: row_height,
            fill: t.invalid_cell_bg,
            stroke: None,
            stroke_width: 0.0,
            corner_radius: 0.0,
            clip: None,
        }));
    }
    if invalid && t.invalid_cell_border.a > 0 {
        // Deferred rather than drawn here as a stroked `Rect`: a
        // stroke sharing this cell's exact bounds lands on the
        // same pixel as this row's grid line / this column's
        // separator, both drawn later in the frame — which then
        // paint over the bottom/right edges. Collecting the
        // bounds and emitting them as boundary lines at the end
        // (same technique as the selection outer border) keeps
        // them on top, so all four edges stay visible.
        invalid_borders.push((cx, ry, col.width, row_height));
    }

    // At-rest cell decoration — consumer-supplied border/tint driven by
    // `ColumnDef::decorator`, resolved after the built-in locked/invalid
    // overlays so it layers on top of them rather than being suppressed.
    if let Some(deco) = col.cell_decoration(ri, model) {
        if let Some(tint) = deco.background_tint {
            frame.push(ScenePrimitive::Rect(RectPrimitive {
                x: cx,
                y: ry,
                width: col.width,
                height: row_height,
                fill: Color::rgba(tint[0], tint[1], tint[2], tint[3]),
                stroke: None,
                stroke_width: 0.0,
                corner_radius: 0.0,
                clip: None,
            }));
        }
        if let Some(border) = deco.border_color {
            frame.push(ScenePrimitive::Rect(RectPrimitive {
                x: cx,
                y: ry,
                width: col.width,
                height: row_height,
                fill: Color::rgba(0, 0, 0, 0),
                stroke: Some(Color::rgba(
                    border[0], border[1], border[2], border[3],
                )),
                stroke_width: t.decoration_border_width,
                corner_radius: 0.0,
                clip: None,
            }));
        }
    }

    // Clamp cell-content clip rects to the sticky header/gutter
    // boundary, mirroring the row-number gutter's own clamp in
    // builder.rs ("Clamp everything to below the sticky header").
    // Without this, an overscanned row (ry < hh) or column (cx < rnw)
    // paints its text/image past the header/gutter's opaque overlay,
    // which becomes visible whenever that overlay has any
    // transparency or transiently disagrees with these bounds.
    let hh = model.effective_header_height();
    let rnw = model.effective_row_number_width();
    let clip_x = cx.max(rnw);
    let clip_w = (cx + col.width - clip_x).max(0.0);
    let clip_y = ry.max(hh);
    let clip_h = (ry + row_height - clip_y).max(0.0);
    let clip: [f64; 4] = [clip_x, clip_y, clip_w, clip_h];

    // Cell text, image, or skeleton
    match cell_status {
        CellStatus::Ready(raw) if !raw.is_empty() => {
            if let Some(CellFormat::Styled(cb)) = &col.format {
                emit_styled(
                    frame,
                    &cb(&raw),
                    cx,
                    ry,
                    mid_y,
                    col.width,
                    row_height,
                    clip,
                    t,
                    class_resolver,
                );
            } else if let Some(CellFormat::Image {
                base_url,
                border_radius,
                padding,
            }) = &col.format
            {
                let url = match base_url {
                    Some(base) => format!("{base}{raw}"),
                    None => raw,
                };
                let pad = *padding;
                frame.push(ScenePrimitive::Image(ImagePrimitive {
                    url,
                    x: cx + pad,
                    y: ry + pad,
                    width: col.width - pad * 2.0,
                    height: row_height - pad * 2.0,
                    corner_radius: *border_radius,
                    clip: Some(clip),
                    placeholder_color: t.skeleton_fg,
                }));
            } else if let Some(CellFormat::ImageText {
                base_url,
                suffix,
                image_size,
                border_radius,
                gap,
            }) = &col.format
            {
                emit_image_text(
                    frame,
                    &raw,
                    cx,
                    ry,
                    col.width,
                    row_height,
                    clip,
                    mid_y,
                    t,
                    base_url,
                    suffix,
                    *image_size,
                    *border_radius,
                    *gap,
                );
            } else if let Some(CellFormat::ProgressBar {
                min,
                max,
                show_label,
                class_of,
            }) = &col.format
            {
                emit_progress_bar(
                    frame,
                    &raw,
                    cx,
                    ry,
                    col.width,
                    row_height,
                    clip,
                    mid_y,
                    t,
                    class_resolver,
                    *min,
                    *max,
                    *show_label,
                    class_of.as_deref(),
                );
            } else {
                let (txt, align, bold, italic, color) =
                    if let Some(fmt) = &col.format {
                        let fc = format_cell(&raw, fmt);
                        let a = match fc.align.unwrap_or_default() {
                            CellAlign::Left => TextAlign::Left,
                            CellAlign::Right => TextAlign::Right,
                            CellAlign::Center => TextAlign::Center,
                            _ => TextAlign::Left,
                        };
                        let default_color = if locked {
                            t.locked_cell_text
                        } else {
                            t.cell_text
                        };
                        let c = fc
                            .color
                            .map(|c| Color::rgba(c[0], c[1], c[2], c[3]))
                            .unwrap_or(default_color);
                        (fc.text, a, fc.bold || col.bold, fc.italic, c)
                    } else {
                        let default_color = if locked {
                            t.locked_cell_text
                        } else {
                            t.cell_text
                        };
                        (raw, TextAlign::Left, col.bold, false, default_color)
                    };
                let x = match align {
                    TextAlign::Right => cx + col.width - t.cell_padding,
                    TextAlign::Center => cx + col.width / 2.0,
                    TextAlign::Left => cx + t.cell_padding,
                };
                frame.push(ScenePrimitive::Text(TextPrimitive {
                    x,
                    y: mid_y,
                    text: txt,
                    color,
                    font_size: t.font_size,
                    bold,
                    italic,
                    clip: Some(clip),
                    align,
                    max_width: Some(
                        (col.width - 2.0 * t.cell_padding).max(0.0),
                    ),
                }));
            }
        }
        CellStatus::Loading => {
            let bar_w = col.width * 0.6;
            let bar_h = t.font_size * 0.5;
            let bar_x = cx + t.cell_padding;
            let bar_y = ry + (row_height - bar_h) / 2.0;
            frame.push(ScenePrimitive::Rect(RectPrimitive {
                x: bar_x,
                y: bar_y,
                width: bar_w,
                height: bar_h,
                fill: t.skeleton_fg,
                stroke: None,
                stroke_width: 0.0,
                corner_radius: bar_h / 2.0,
                clip: None,
            }));
        }
        _ => {}
    }

    // Cell buttons — always rendered, on top of cell content.
    emit_cell_buttons(frame, col, ri, ci, cx, ry, row_height, clip, t);
}

/// Emit an image + text pair for `CellFormat::ImageText`.
///
/// Raw value = `"KEY Label"`. Image URL is built from
/// `base_url + key + suffix`. The image is rendered on the
/// left, text on the right.
#[allow(clippy::too_many_arguments)]
fn emit_image_text(
    frame: &mut SceneFrame,
    raw: &str,
    cx: f64,
    ry: f64,
    col_width: f64,
    row_height: f64,
    clip: [f64; 4],
    mid_y: f64,
    t: &Theme,
    base_url: &str,
    suffix: &str,
    image_size: f64,
    border_radius: f64,
    gap: f64,
) {
    let (key, label) = raw.split_once(' ').unwrap_or((raw, ""));

    // Image — vertically centered in the cell.
    let img_pad = (row_height - image_size) / 2.0;
    let img_x = cx + t.cell_padding;
    let img_y = ry + img_pad;
    let url = format!("{base_url}{key}{suffix}");
    frame.push(ScenePrimitive::Image(ImagePrimitive {
        url,
        x: img_x,
        y: img_y,
        width: image_size,
        height: image_size,
        corner_radius: border_radius,
        clip: Some(clip),
        placeholder_color: t.skeleton_fg,
    }));

    // Text — offset after the image.
    if !label.is_empty() {
        let text_x = img_x + image_size + gap;
        frame.push(ScenePrimitive::Text(TextPrimitive {
            x: text_x,
            y: mid_y,
            text: label.to_owned(),
            color: t.cell_text,
            font_size: t.font_size,
            bold: false,
            italic: false,
            clip: Some(clip),
            align: TextAlign::Left,
            max_width: Some(
                (col_width - 2.0 * t.cell_padding - image_size - gap).max(0.0),
            ),
        }));
    }
}

/// Emit a value-driven progress bar for
/// `CellFormat::ProgressBar`.
///
/// The raw value is parsed as `f64` and mapped to a fraction in
/// `[0, 1]` via `(value - min) / (max - min)`. A track rectangle
/// spans the available width; a fill rectangle is scaled by the
/// fraction. The fill colour comes from `class_of(raw)` resolved
/// through `class_resolver` (its `background`), falling back to
/// `Theme::progress_fill`. When `show_label` is set, the
/// percentage is drawn right-aligned after the bar.
#[allow(clippy::too_many_arguments)]
fn emit_progress_bar(
    frame: &mut SceneFrame,
    raw: &str,
    cx: f64,
    ry: f64,
    col_width: f64,
    row_height: f64,
    clip: [f64; 4],
    mid_y: f64,
    t: &Theme,
    class_resolver: Option<&ClassResolver>,
    min: f64,
    max: f64,
    show_label: bool,
    class_of: Option<&dyn Fn(&str) -> String>,
) {
    // Value → fraction in [0, 1].
    let value = raw.parse::<f64>().unwrap_or(min);
    let frac = if max > min {
        ((value - min) / (max - min)).clamp(0.0, 1.0)
    } else {
        0.0
    };

    // Resolve the per-value fill colour (and optional radius).
    let style = class_of
        .and_then(|f| class_resolver.map(|r| r(&f(raw))))
        .unwrap_or_default();
    let fill = style.background.unwrap_or(t.progress_fill);

    // Reserve space for the "NN%" label when shown.
    let label = if show_label {
        Some(format!("{:.0}%", frac * 100.0))
    } else {
        None
    };
    let label_w = label
        .as_ref()
        .map(|s| s.len() as f64 * t.font_size * 0.65 + 6.0)
        .unwrap_or(0.0);

    let bar_x = cx + t.cell_padding;
    let bar_w = (col_width - 2.0 * t.cell_padding - label_w).max(0.0);
    let bar_h = t.progress_height.min(row_height - 2.0).max(0.0);
    let bar_y = ry + (row_height - bar_h) / 2.0;
    let track_radius = t.progress_radius.min(bar_h / 2.0);

    // Track (unfilled background).
    frame.push(ScenePrimitive::Rect(RectPrimitive {
        x: bar_x,
        y: bar_y,
        width: bar_w,
        height: bar_h,
        fill: t.progress_track,
        stroke: None,
        stroke_width: 0.0,
        corner_radius: track_radius,
        clip: Some(clip),
    }));

    // Fill (scaled by the fraction).
    let fill_w = bar_w * frac;
    if fill_w > 0.0 {
        frame.push(ScenePrimitive::Rect(RectPrimitive {
            x: bar_x,
            y: bar_y,
            width: fill_w,
            height: bar_h,
            fill,
            stroke: None,
            stroke_width: 0.0,
            corner_radius: track_radius.min(fill_w / 2.0),
            clip: Some(clip),
        }));
    }

    // Percentage label, right-aligned in the reserved space.
    if let Some(text) = label {
        frame.push(ScenePrimitive::Text(TextPrimitive {
            x: cx + col_width - t.cell_padding,
            y: mid_y,
            text,
            color: t.cell_text,
            font_size: t.font_size,
            bold: false,
            italic: false,
            clip: Some(clip),
            align: TextAlign::Right,
            max_width: Some(label_w.max(0.0)),
        }));
    }
}

/// Emit a row of styled elements (badges, chips…) for a
/// `CellFormat::Styled` cell.
///
/// Elements flow left-to-right starting at
/// `cx + cell_padding`, with a 4 px gap between them.
/// Badge width is estimated from character count so that
/// no Canvas2D measurement is needed at the scene layer.
#[allow(clippy::too_many_arguments)]
fn emit_styled(
    frame: &mut SceneFrame,
    elements: &[CellElement],
    cx: f64,
    ry: f64,
    mid_y: f64,
    cell_w: f64,
    row_height: f64,
    clip: [f64; 4],
    t: &Theme,
    class_resolver: Option<&ClassResolver>,
) {
    let mut x = cx + t.cell_padding;

    for el in elements {
        let style = class_resolver.map(|r| r(&el.class)).unwrap_or_default();
        let font_size = (t.font_size + style.font_size_delta).max(8.0);

        // Estimated badge width from character count.
        // 0.65 provides enough margin for wide Latin glyphs
        // (e.g. 'E', 'W', 'm') in system-ui at any size.
        // Capped to remaining cell space so the background rect
        // never overflows the column boundary on resize.
        let available_w = (cx + cell_w - x - t.cell_padding).max(0.0);
        let text_w = el.text.len() as f64 * font_size * 0.65;
        let badge_w =
            (text_w + style.padding_x * 2.0).min(available_w).max(0.0);
        let badge_h = (font_size + style.padding_y * 2.0).min(row_height - 2.0);
        let badge_y = ry + (row_height - badge_h) / 2.0;

        // ── background rect / outline ─────────────────────────
        let has_bg = style.background.is_some();
        let has_border = style.border_color.is_some();

        if has_bg || has_border {
            frame.push(ScenePrimitive::Rect(RectPrimitive {
                x,
                y: badge_y,
                width: badge_w,
                height: badge_h,
                fill: style.background.unwrap_or(Color::rgba(0, 0, 0, 0)),
                stroke: style.border_color,
                stroke_width: style.border_width,
                corner_radius: style.border_radius,
                clip: Some(clip),
            }));
        }

        // ── text centred inside the badge ─────────────────────
        let text_color = style.color.unwrap_or(t.cell_text);
        frame.push(ScenePrimitive::Text(TextPrimitive {
            x: x + badge_w / 2.0,
            y: mid_y,
            text: el.text.clone(),
            color: text_color,
            font_size,
            bold: style.bold,
            italic: style.italic,
            clip: Some(clip),
            align: TextAlign::Center,
            // Clip to the full badge width (including padding) so
            // that text centred in the badge doesn't get truncated
            // when the estimated width is slightly off.
            max_width: Some(badge_w.max(0.0)),
        }));

        // Gap between consecutive badges.
        x += badge_w + 4.0;
    }
}

/// Emit Rect + Text primitives for each [`ButtonDef`] in
/// `col.cell_buttons` and record their hit zones.
///
/// Buttons are laid out right-to-left: the first entry in
/// `cell_buttons` is the rightmost button.  This makes
/// positions stable when more buttons are added.
///
/// Skips any button that would overflow the left cell edge.
#[allow(clippy::too_many_arguments)]
fn emit_cell_buttons(
    frame: &mut SceneFrame,
    col: &ColumnDef,
    ri: u64,
    ci: usize,
    cx: f64,
    ry: f64,
    row_height: f64,
    clip: [f64; 4],
    t: &Theme,
) {
    use crate::frame::ButtonZone;

    if col.cell_buttons.is_empty() {
        return;
    }

    let btn_h = (t.font_size + t.cell_btn_padding_y * 2.0)
        .min(row_height - 4.0)
        .max(0.0);
    let btn_y = ry + (row_height - btn_h) / 2.0;
    // Baseline for vertically-centred text inside the button.
    // 0.35 ≈ half cap-height for system-ui.
    let mid_y = btn_y + btn_h * 0.5 + t.font_size * 0.35;

    // Accumulate right edge inward from the cell's right border.
    let mut right_x = cx + col.width - t.cell_btn_margin_r;

    for btn in col.cell_buttons.iter().rev() {
        // Width from character count (same heuristic as
        // emit_styled: 0.65 × font_size per char).
        let text_w = btn.label.len() as f64 * t.font_size * 0.65;
        let btn_w = (text_w + t.cell_btn_padding_x * 2.0).max(0.0);
        let btn_x = right_x - btn_w;

        // Skip if the button would bleed past the left edge.
        if btn_x < cx {
            right_x = btn_x - t.cell_btn_gap;
            continue;
        }

        let (fill, text_color, stroke) = match btn.style {
            ButtonStyle::Primary => {
                (Some(t.cell_btn_primary_bg), t.cell_btn_primary_text, None)
            }
            ButtonStyle::Secondary => (
                Some(t.cell_btn_secondary_bg),
                t.cell_btn_secondary_text,
                None,
            ),
            ButtonStyle::Danger => {
                (Some(t.cell_btn_danger_bg), t.cell_btn_danger_text, None)
            }
            ButtonStyle::Ghost => {
                (None, t.cell_btn_ghost_color, Some(t.cell_btn_ghost_color))
            }
            // Future variants via #[non_exhaustive].
            _ => {
                right_x = btn_x - t.cell_btn_gap;
                continue;
            }
        };

        // Background / border rect.
        frame.push(ScenePrimitive::Rect(RectPrimitive {
            x: btn_x,
            y: btn_y,
            width: btn_w,
            height: btn_h,
            fill: fill.unwrap_or(Color::rgba(0, 0, 0, 0)),
            stroke,
            stroke_width: if stroke.is_some() { 1.0 } else { 0.0 },
            corner_radius: t.cell_btn_radius,
            clip: Some(clip),
        }));

        // Label centred inside the button.
        frame.push(ScenePrimitive::Text(TextPrimitive {
            x: btn_x + btn_w / 2.0,
            y: mid_y,
            text: btn.label.clone(),
            color: text_color,
            font_size: t.font_size,
            bold: false,
            italic: false,
            clip: Some(clip),
            align: TextAlign::Center,
            max_width: Some(btn_w.max(0.0)),
        }));

        // Hit zone — viewport-relative coordinates.
        frame.push_button_zone(ButtonZone {
            row: ri,
            col: ci,
            button_id: btn.id.clone(),
            x: btn_x,
            y: btn_y,
            width: btn_w,
            height: btn_h,
        });

        right_x = btn_x - t.cell_btn_gap;
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use rs_grid_core::{
        column::{CellDecoration, ColumnDef},
        datasource::CellStatus,
        format::CellFormat,
        selection::SelectionState,
    };

    use super::emit_cell;
    use crate::{
        builder::FlashHint, frame::SceneFrame, primitives::ScenePrimitive,
        theme::Theme,
    };

    // ── helpers ──────────────────────────────────────────────

    fn make_frame() -> SceneFrame {
        SceneFrame::new(800.0, 600.0, 1.0)
    }

    fn make_col() -> ColumnDef {
        ColumnDef::new("a", "Alpha", 100.0)
    }

    fn make_model(col: &ColumnDef) -> rs_grid_core::model::GridModel {
        rs_grid_core::model::GridModel::new(
            vec![col.clone()],
            vec![rs_grid_core::row::RowRecord::new(0)],
            42.0,
            30.0,
        )
    }

    fn no_search() -> HashSet<(u64, usize)> {
        HashSet::new()
    }

    // ── CellStatus::Loading ──────────────────────────────────

    #[test]
    fn emit_cell_loading_emits_skeleton_rect() {
        let mut frame = make_frame();
        let col = make_col();
        let sel = SelectionState::default();
        let t = Theme::light();
        emit_cell(
            &mut frame,
            &col,
            &make_model(&col),
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Loading,
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        assert_eq!(frame.primitive_count(), 1);
        match &frame.primitives[0] {
            ScenePrimitive::Rect(r) => {
                assert_eq!(r.fill, t.skeleton_fg);
            }
            _ => panic!("expected Rect"),
        }
    }

    // ── CellStatus::Ready (empty) / Absent ───────────────────

    #[test]
    fn emit_cell_ready_empty_emits_nothing() {
        let mut frame = make_frame();
        let col = make_col();
        let sel = SelectionState::default();
        let t = Theme::light();
        emit_cell(
            &mut frame,
            &col,
            &make_model(&col),
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Ready(String::new()),
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        assert_eq!(frame.primitive_count(), 0);
    }

    #[test]
    fn emit_cell_absent_emits_nothing() {
        let mut frame = make_frame();
        let col = make_col();
        let sel = SelectionState::default();
        let t = Theme::light();
        emit_cell(
            &mut frame,
            &col,
            &make_model(&col),
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Absent,
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        assert_eq!(frame.primitive_count(), 0);
    }

    // ── Flash overlay ────────────────────────────────────────

    #[test]
    fn emit_cell_flash_on_selected_emits_two_rects() {
        let mut frame = make_frame();
        let col = make_col();
        let mut sel = SelectionState::default();
        sel.select_cell(0, 0);
        let t = Theme::light();
        let flash = FlashHint {
            alpha_factor: 0.5,
            cells: [(0, 0)].into_iter().collect(),
            is_error: false,
        };
        emit_cell(
            &mut frame,
            &col,
            &make_model(&col),
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Absent,
            &sel,
            &no_search(),
            None,
            &t,
            Some(&flash),
            None,
            &mut Vec::new(),
        );
        // selection fill + flash overlay = 2 Rect primitives
        assert_eq!(frame.primitive_count(), 2);
        assert!(
            frame
                .primitives
                .iter()
                .all(|p| matches!(p, ScenePrimitive::Rect(_)))
        );
    }

    #[test]
    fn emit_cell_selected_but_not_flashed_emits_no_flash_overlay() {
        // A cell can be selected (e.g. part of a paste's target
        // rectangle) without being in `flash.cells` (e.g. it was
        // skipped for being locked or failing validation) — only
        // the selection fill should render, no flash overlay.
        let mut frame = make_frame();
        let col = make_col();
        let mut sel = SelectionState::default();
        sel.select_cell(0, 1);
        let t = Theme::light();
        let flash = FlashHint {
            alpha_factor: 1.0,
            cells: [(0, 0)].into_iter().collect(),
            is_error: false,
        };
        emit_cell(
            &mut frame,
            &col,
            &make_model(&col),
            0,
            1,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Absent,
            &sel,
            &no_search(),
            None,
            &t,
            Some(&flash),
            None,
            &mut Vec::new(),
        );
        // Only the selection fill — no flash overlay for (0, 1).
        assert_eq!(frame.primitive_count(), 1);
        match &frame.primitives[0] {
            ScenePrimitive::Rect(r) => assert_eq!(r.fill, t.selection_fill),
            _ => panic!("expected the selection fill Rect"),
        }
    }

    // ── Search highlight ─────────────────────────────────────

    #[test]
    fn emit_cell_search_non_current_uses_highlight_color() {
        let mut frame = make_frame();
        let col = make_col();
        let sel = SelectionState::default();
        let t = Theme::light();
        let mut search = HashSet::new();
        search.insert((0u64, 0usize));
        emit_cell(
            &mut frame,
            &col,
            &make_model(&col),
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Absent,
            &sel,
            &search,
            None,
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        assert_eq!(frame.primitive_count(), 1);
        match &frame.primitives[0] {
            ScenePrimitive::Rect(r) => {
                assert_eq!(r.fill, t.search_highlight);
            }
            _ => panic!("expected Rect"),
        }
    }

    #[test]
    fn emit_cell_search_current_uses_current_color() {
        let mut frame = make_frame();
        let col = make_col();
        let sel = SelectionState::default();
        let t = Theme::light();
        let mut search = HashSet::new();
        search.insert((0u64, 0usize));
        emit_cell(
            &mut frame,
            &col,
            &make_model(&col),
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Absent,
            &sel,
            &search,
            Some((0, 0)),
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        assert_eq!(frame.primitive_count(), 1);
        match &frame.primitives[0] {
            ScenePrimitive::Rect(r) => {
                assert_eq!(r.fill, t.search_current);
            }
            _ => panic!("expected Rect"),
        }
    }

    // ── CellFormat::Image ────────────────────────────────────

    #[test]
    fn emit_cell_image_format_emits_image_primitive() {
        let mut frame = make_frame();
        let col = ColumnDef::new("img", "Image", 100.0).with_format(
            CellFormat::Image {
                base_url: Some("https://cdn/".into()),
                border_radius: 4.0,
                padding: 4.0,
            },
        );
        let sel = SelectionState::default();
        let t = Theme::light();
        emit_cell(
            &mut frame,
            &col,
            &make_model(&col),
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Ready("photo.png".into()),
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        assert_eq!(frame.primitive_count(), 1);
        match &frame.primitives[0] {
            ScenePrimitive::Image(img) => {
                assert!(img.url.contains("photo.png"));
            }
            _ => panic!("expected Image"),
        }
    }

    #[test]
    fn emit_cell_image_no_base_url_uses_raw() {
        let mut frame = make_frame();
        let col = ColumnDef::new("img", "Image", 100.0).with_format(
            CellFormat::Image {
                base_url: None,
                border_radius: 0.0,
                padding: 0.0,
            },
        );
        let sel = SelectionState::default();
        let t = Theme::light();
        emit_cell(
            &mut frame,
            &col,
            &make_model(&col),
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Ready("https://img/x.png".into()),
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        assert_eq!(frame.primitive_count(), 1);
        match &frame.primitives[0] {
            ScenePrimitive::Image(img) => {
                assert_eq!(img.url, "https://img/x.png");
            }
            _ => panic!("expected Image"),
        }
    }

    // ── CellFormat::ImageText ────────────────────────────────

    #[test]
    fn emit_cell_image_text_with_label_emits_image_and_text() {
        let mut frame = make_frame();
        let col = ColumnDef::new("flag", "Flag", 150.0).with_format(
            CellFormat::ImageText {
                base_url: "https://flags/".into(),
                suffix: ".svg".into(),
                image_size: 20.0,
                border_radius: 0.0,
                gap: 4.0,
            },
        );
        let sel = SelectionState::default();
        let t = Theme::light();
        // raw = "FR France" → key="FR", label="France"
        emit_cell(
            &mut frame,
            &col,
            &make_model(&col),
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Ready("FR France".into()),
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        let has_image = frame
            .primitives
            .iter()
            .any(|p| matches!(p, ScenePrimitive::Image(_)));
        let has_text = frame
            .primitives
            .iter()
            .any(|p| matches!(p, ScenePrimitive::Text(_)));
        assert!(has_image, "expected Image primitive");
        assert!(has_text, "expected Text primitive");
    }

    // ── CellFormat with text alignment ─────────────────────

    #[test]
    fn emit_cell_formatted_right_aligned() {
        use crate::primitives::TextAlign;
        let mut frame = make_frame();
        let col =
            ColumnDef::new("v", "V", 100.0).with_format(CellFormat::Number {
                decimal_places: 2,
                thousands_sep: None,
                decimal_sep: '.',
            });
        let sel = SelectionState::default();
        let t = Theme::light();
        emit_cell(
            &mut frame,
            &col,
            &make_model(&col),
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Ready("1234.5".into()),
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        assert_eq!(frame.primitive_count(), 1);
        match &frame.primitives[0] {
            ScenePrimitive::Text(txt) => {
                assert_eq!(txt.text, "1234.50");
                assert_eq!(txt.align, TextAlign::Right);
            }
            _ => panic!("expected Text"),
        }
    }

    #[test]
    fn emit_cell_formatted_center_aligned() {
        use crate::primitives::TextAlign;
        let mut frame = make_frame();
        let col =
            ColumnDef::new("b", "B", 100.0).with_format(CellFormat::Boolean {
                true_label: "Yes".into(),
                false_label: "No".into(),
            });
        let sel = SelectionState::default();
        let t = Theme::light();
        emit_cell(
            &mut frame,
            &col,
            &make_model(&col),
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Ready("true".into()),
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        assert_eq!(frame.primitive_count(), 1);
        match &frame.primitives[0] {
            ScenePrimitive::Text(txt) => {
                assert_eq!(txt.text, "Yes");
                assert_eq!(txt.align, TextAlign::Center);
            }
            _ => panic!("expected Text"),
        }
    }

    // ── CellFormat::Styled ───────────────────────────────

    #[test]
    fn emit_cell_styled_no_bg_emits_text_only() {
        use std::rc::Rc;

        use rs_grid_core::format::{CellAlign, CellElement};

        let mut frame = make_frame();
        let col = ColumnDef::new("s", "S", 150.0).with_format(
            CellFormat::Styled(Rc::new(|_raw| {
                vec![CellElement {
                    text: "active".into(),
                    class: "".into(),
                    align: CellAlign::Left,
                }]
            })),
        );
        let sel = SelectionState::default();
        let t = Theme::light();
        emit_cell(
            &mut frame,
            &col,
            &make_model(&col),
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Ready("active".into()),
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        // No background → only a Text primitive.
        assert_eq!(frame.primitive_count(), 1);
        assert!(matches!(frame.primitives[0], ScenePrimitive::Text(_)));
    }

    #[test]
    fn emit_cell_styled_with_bg_emits_rect_and_text() {
        use std::rc::Rc;

        use rs_grid_core::format::{CellAlign, CellElement};

        use crate::{class_map::CellElementStyle, primitives::Color};

        let mut frame = make_frame();
        let col = ColumnDef::new("s", "S", 150.0).with_format(
            CellFormat::Styled(Rc::new(|_raw| {
                vec![CellElement {
                    text: "badge".into(),
                    class: "bg".into(),
                    align: CellAlign::Left,
                }]
            })),
        );
        let sel = SelectionState::default();
        let t = Theme::light();
        // Resolver that returns a background color.
        let resolver: &crate::class_map::ClassResolver =
            &|_class: &str| CellElementStyle {
                background: Some(Color::rgb(255, 0, 0)),
                padding_x: 4.0,
                padding_y: 2.0,
                ..CellElementStyle::default()
            };
        emit_cell(
            &mut frame,
            &col,
            &make_model(&col),
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Ready("badge".into()),
            &sel,
            &no_search(),
            None,
            &t,
            None,
            Some(resolver),
            &mut Vec::new(),
        );
        // background rect + text
        assert_eq!(frame.primitive_count(), 2);
        assert!(matches!(frame.primitives[0], ScenePrimitive::Rect(_)));
        assert!(matches!(frame.primitives[1], ScenePrimitive::Text(_)));
    }

    #[test]
    fn emit_cell_styled_multiple_elements_emits_all() {
        use std::rc::Rc;

        use rs_grid_core::format::{CellAlign, CellElement};

        let mut frame = make_frame();
        let col = ColumnDef::new("s", "S", 300.0).with_format(
            CellFormat::Styled(Rc::new(|_raw| {
                vec![
                    CellElement {
                        text: "A".into(),
                        class: "".into(),
                        align: CellAlign::Left,
                    },
                    CellElement {
                        text: "B".into(),
                        class: "".into(),
                        align: CellAlign::Left,
                    },
                ]
            })),
        );
        let sel = SelectionState::default();
        let t = Theme::light();
        emit_cell(
            &mut frame,
            &col,
            &make_model(&col),
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Ready("x".into()),
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        // 2 elements → 2 Text primitives (no bg).
        assert_eq!(frame.primitive_count(), 2);
    }

    // ── CellFormat::ProgressBar ──────────────────────────

    fn progress_col(show_label: bool) -> ColumnDef {
        ColumnDef::new("p", "P", 120.0).with_format(CellFormat::ProgressBar {
            min: 0.0,
            max: 1.0,
            show_label,
            class_of: None,
        })
    }

    #[test]
    fn emit_cell_progress_bar_emits_track_and_fill() {
        let mut frame = make_frame();
        let col = progress_col(false);
        let sel = SelectionState::default();
        let t = Theme::light();
        emit_cell(
            &mut frame,
            &col,
            &make_model(&col),
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Ready("0.5".into()),
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        // track + fill = 2 rects (no label).
        assert_eq!(frame.primitive_count(), 2);
        assert!(
            frame
                .primitives
                .iter()
                .all(|p| matches!(p, ScenePrimitive::Rect(_)))
        );
        // Fill width is half the track width.
        let widths: Vec<f64> = frame
            .primitives
            .iter()
            .filter_map(|p| match p {
                ScenePrimitive::Rect(r) => Some(r.width),
                _ => None,
            })
            .collect();
        let track = widths[0];
        let fill = widths[1];
        assert!((fill - track * 0.5).abs() < 0.001);
    }

    #[test]
    fn emit_cell_progress_bar_with_label_adds_text() {
        let mut frame = make_frame();
        let col = progress_col(true);
        let sel = SelectionState::default();
        let t = Theme::light();
        emit_cell(
            &mut frame,
            &col,
            &make_model(&col),
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Ready("0.7".into()),
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        // track + fill + label text.
        assert_eq!(frame.primitive_count(), 3);
        match &frame.primitives[2] {
            ScenePrimitive::Text(txt) => assert_eq!(txt.text, "70%"),
            _ => panic!("expected label Text"),
        }
    }

    #[test]
    fn emit_cell_progress_bar_clamps_and_omits_zero_fill() {
        // Value below min → fraction 0 → no fill rect.
        let mut frame = make_frame();
        let col = progress_col(false);
        let sel = SelectionState::default();
        let t = Theme::light();
        emit_cell(
            &mut frame,
            &col,
            &make_model(&col),
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Ready("-1".into()),
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        // Only the track rect (zero fill is skipped).
        assert_eq!(frame.primitive_count(), 1);
    }

    #[test]
    fn emit_cell_progress_bar_resolved_class_sets_fill() {
        use crate::{class_map::CellElementStyle, primitives::Color};

        let mut frame = make_frame();
        let col = ColumnDef::new("p", "P", 120.0).with_format(
            CellFormat::ProgressBar {
                min: 0.0,
                max: 1.0,
                show_label: false,
                class_of: Some(std::rc::Rc::new(|_raw| {
                    "progress progress-success".into()
                })),
            },
        );
        let sel = SelectionState::default();
        let t = Theme::light();
        let resolver: &crate::class_map::ClassResolver =
            &|_class: &str| CellElementStyle {
                background: Some(Color::rgb(0, 211, 144)),
                ..CellElementStyle::default()
            };
        emit_cell(
            &mut frame,
            &col,
            &make_model(&col),
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Ready("0.9".into()),
            &sel,
            &no_search(),
            None,
            &t,
            None,
            Some(resolver),
            &mut Vec::new(),
        );
        // Fill rect (second) uses the resolved background.
        match &frame.primitives[1] {
            ScenePrimitive::Rect(r) => {
                assert_eq!(r.fill, Color::rgb(0, 211, 144));
            }
            _ => panic!("expected fill Rect"),
        }
    }

    #[test]
    fn emit_cell_progress_bar_no_resolver_uses_theme_fill() {
        let mut frame = make_frame();
        let col = progress_col(false);
        let sel = SelectionState::default();
        let t = Theme::light();
        emit_cell(
            &mut frame,
            &col,
            &make_model(&col),
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Ready("0.8".into()),
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        match &frame.primitives[1] {
            ScenePrimitive::Rect(r) => assert_eq!(r.fill, t.progress_fill),
            _ => panic!("expected fill Rect"),
        }
    }

    // ── emit_cell_buttons ────────────────────────────────

    #[test]
    fn emit_cell_button_primary_emits_rect_text_and_zone() {
        use rs_grid_core::column::{ButtonDef, ButtonStyle};

        let mut frame = make_frame();
        let col = ColumnDef::new("x", "X", 200.0).with_cell_buttons(vec![
            ButtonDef::new("save", "Save", ButtonStyle::Primary),
        ]);
        let sel = SelectionState::default();
        let t = Theme::light();
        emit_cell(
            &mut frame,
            &col,
            &make_model(&col),
            1,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Absent,
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        // Rect + Text for the button.
        assert_eq!(frame.primitive_count(), 2);
        assert_eq!(frame.button_zones.len(), 1);
        assert_eq!(frame.button_zones[0].button_id, "save");
        assert_eq!(frame.button_zones[0].row, 1);
    }

    #[test]
    fn emit_cell_button_ghost_has_stroke() {
        use rs_grid_core::column::{ButtonDef, ButtonStyle};

        use crate::primitives::ScenePrimitive;

        let mut frame = make_frame();
        let col = ColumnDef::new("x", "X", 200.0).with_cell_buttons(vec![
            ButtonDef::new("g", "Ghost", ButtonStyle::Ghost),
        ]);
        let sel = SelectionState::default();
        let t = Theme::light();
        emit_cell(
            &mut frame,
            &col,
            &make_model(&col),
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Absent,
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        let has_stroke = frame.primitives.iter().any(
            |p| matches!(p, ScenePrimitive::Rect(r) if r.stroke.is_some()),
        );
        assert!(has_stroke, "Ghost button should have a stroke rect");
    }

    #[test]
    fn emit_cell_button_secondary_and_danger_emit_zones() {
        use rs_grid_core::column::{ButtonDef, ButtonStyle};

        let mut frame = make_frame();
        let col = ColumnDef::new("x", "X", 400.0).with_cell_buttons(vec![
            ButtonDef::new("d", "Del", ButtonStyle::Danger),
            ButtonDef::new("s", "Sec", ButtonStyle::Secondary),
        ]);
        let sel = SelectionState::default();
        let t = Theme::light();
        emit_cell(
            &mut frame,
            &col,
            &make_model(&col),
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Absent,
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        assert_eq!(frame.button_zones.len(), 2);
    }

    #[test]
    fn emit_cell_image_text_no_label_emits_only_image() {
        let mut frame = make_frame();
        let col = ColumnDef::new("flag", "Flag", 150.0).with_format(
            CellFormat::ImageText {
                base_url: "https://flags/".into(),
                suffix: ".svg".into(),
                image_size: 20.0,
                border_radius: 0.0,
                gap: 4.0,
            },
        );
        let sel = SelectionState::default();
        let t = Theme::light();
        // raw = "FR" → no space → key="FR", label=""
        emit_cell(
            &mut frame,
            &col,
            &make_model(&col),
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Ready("FR".into()),
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        assert_eq!(frame.primitive_count(), 1);
        assert!(
            matches!(frame.primitives[0], ScenePrimitive::Image(_)),
            "expected Image only"
        );
    }

    // ── header/gutter clip clamping ──────────────────────────

    #[test]
    fn emit_cell_text_clip_clamped_below_header_when_row_overlaps() {
        // make_model() uses header_height = 30.0. ry = 10.0 simulates
        // an overscanned row whose top edge is still under the header.
        let mut frame = make_frame();
        let col = make_col();
        let sel = SelectionState::default();
        let t = Theme::light();
        emit_cell(
            &mut frame,
            &col,
            &make_model(&col),
            0,
            0,
            0.0,
            10.0,
            21.0,
            42.0,
            CellStatus::Ready("hello".into()),
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        match &frame.primitives[0] {
            ScenePrimitive::Text(txt) => {
                let clip = txt.clip.expect("text should carry a clip rect");
                assert_eq!(
                    clip[1], 30.0,
                    "clip.y must clamp to header_height, not ry"
                );
                assert!(
                    clip[3] < 42.0,
                    "clip.h must shrink to exclude the header overlap"
                );
            }
            _ => panic!("expected Text primitive"),
        }
    }

    #[test]
    fn emit_cell_text_clip_clamped_right_of_gutter_when_col_overlaps() {
        // make_model() auto-computes row_number_width = 40.0 for a
        // 1-row model. cx = 0.0 simulates a column whose left edge
        // sits under the gutter.
        let mut frame = make_frame();
        let col = make_col();
        let sel = SelectionState::default();
        let t = Theme::light();
        emit_cell(
            &mut frame,
            &col,
            &make_model(&col),
            0,
            0,
            0.0,
            50.0,
            71.0,
            42.0,
            CellStatus::Ready("hello".into()),
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        match &frame.primitives[0] {
            ScenePrimitive::Text(txt) => {
                let clip = txt.clip.expect("text should carry a clip rect");
                assert_eq!(
                    clip[0], 40.0,
                    "clip.x must clamp to row_number_width, not cx"
                );
            }
            _ => panic!("expected Text primitive"),
        }
    }

    // ── locked cell overlay ──────────────────────────────────

    #[test]
    fn emit_cell_locked_static_editable_false_emits_overlay() {
        let mut frame = make_frame();
        let col = make_col().read_only();
        let model = make_model(&col);
        let sel = SelectionState::default();
        let t = Theme::light();
        emit_cell(
            &mut frame,
            &col,
            &model,
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Ready("hello".into()),
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        assert_eq!(frame.primitive_count(), 2);
        match &frame.primitives[0] {
            ScenePrimitive::Rect(r) => assert_eq!(r.fill, t.locked_cell_bg),
            _ => panic!("expected the locked-cell overlay Rect first"),
        }
    }

    #[test]
    fn emit_cell_locked_predicate_false_emits_overlay() {
        let mut frame = make_frame();
        let col = make_col().editable_when(|_, _| false);
        let model = make_model(&col);
        let sel = SelectionState::default();
        let t = Theme::light();
        emit_cell(
            &mut frame,
            &col,
            &model,
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Ready("hello".into()),
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        assert_eq!(frame.primitive_count(), 2);
        match &frame.primitives[0] {
            ScenePrimitive::Rect(r) => assert_eq!(r.fill, t.locked_cell_bg),
            _ => panic!("expected the locked-cell overlay Rect first"),
        }
    }

    #[test]
    fn emit_cell_editable_emits_no_overlay() {
        let mut frame = make_frame();
        let col = make_col();
        let model = make_model(&col);
        let sel = SelectionState::default();
        let t = Theme::light();
        emit_cell(
            &mut frame,
            &col,
            &model,
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Ready("hello".into()),
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        // Only the text primitive — no locked-cell overlay.
        assert_eq!(frame.primitive_count(), 1);
        assert!(matches!(frame.primitives[0], ScenePrimitive::Text(_)));
    }

    #[test]
    fn emit_cell_locked_uses_locked_cell_text_color() {
        let mut frame = make_frame();
        let col = make_col().read_only();
        let model = make_model(&col);
        let sel = SelectionState::default();
        let t = Theme::light();
        emit_cell(
            &mut frame,
            &col,
            &model,
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Ready("hello".into()),
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        match &frame.primitives[1] {
            ScenePrimitive::Text(txt) => {
                assert_eq!(txt.color, t.locked_cell_text);
            }
            _ => panic!("expected Text primitive after the overlay"),
        }
    }

    #[test]
    fn emit_cell_locked_bg_skipped_when_theme_transparent() {
        let mut frame = make_frame();
        let col = make_col().read_only();
        let model = make_model(&col);
        let sel = SelectionState::default();
        let mut t = Theme::light();
        t.locked_cell_bg = crate::primitives::Color::rgba(0, 0, 0, 0);
        emit_cell(
            &mut frame,
            &col,
            &model,
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Ready("hello".into()),
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        // No overlay rect — only the text primitive.
        assert_eq!(frame.primitive_count(), 1);
        assert!(matches!(frame.primitives[0], ScenePrimitive::Text(_)));
    }

    // ── at-rest invalid-value background/border ──────────────

    #[test]
    fn emit_cell_invalid_value_emits_border_overlay() {
        let mut frame = make_frame();
        let col = make_col().required();
        let model = make_model(&col);
        let sel = SelectionState::default();
        let mut t = Theme::light();
        // Isolate the border: bg overlay covered separately below.
        t.invalid_cell_bg = crate::primitives::Color::rgba(0, 0, 0, 0);
        let mut invalid_borders = Vec::new();
        emit_cell(
            &mut frame,
            &col,
            &model,
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            // Empty value fails `.required()` even though the cell
            // was never edited.
            CellStatus::Ready(String::new()),
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut invalid_borders,
        );
        // The border itself isn't drawn here — it's collected so the
        // caller (SceneBuilder::build) can emit it once every grid
        // line / separator is already in the frame. See the push
        // site's comment for why.
        assert_eq!(frame.primitive_count(), 0);
        assert_eq!(invalid_borders, vec![(0.0, 0.0, col.width, 42.0)]);
    }

    #[test]
    fn emit_cell_invalid_value_emits_bg_overlay() {
        let mut frame = make_frame();
        let col = make_col().required();
        let model = make_model(&col);
        let sel = SelectionState::default();
        let mut t = Theme::light();
        // Isolate the bg: border overlay covered separately above.
        t.invalid_cell_border = crate::primitives::Color::rgba(0, 0, 0, 0);
        emit_cell(
            &mut frame,
            &col,
            &model,
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Ready(String::new()),
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        assert_eq!(frame.primitive_count(), 1);
        match &frame.primitives[0] {
            ScenePrimitive::Rect(r) => {
                assert_eq!(r.fill, t.invalid_cell_bg);
                assert!(r.stroke.is_none());
            }
            _ => panic!("expected the invalid-cell bg Rect"),
        }
    }

    #[test]
    fn emit_cell_valid_value_emits_no_border_overlay() {
        let mut frame = make_frame();
        let col = make_col().required();
        let model = make_model(&col);
        let sel = SelectionState::default();
        let t = Theme::light();
        emit_cell(
            &mut frame,
            &col,
            &model,
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Ready("ok".into()),
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        // Only the text primitive — no invalid-cell border.
        assert_eq!(frame.primitive_count(), 1);
        assert!(matches!(frame.primitives[0], ScenePrimitive::Text(_)));
    }

    #[test]
    fn emit_cell_invalid_border_skipped_when_theme_transparent() {
        let mut frame = make_frame();
        let col = make_col().required();
        let model = make_model(&col);
        let sel = SelectionState::default();
        let mut t = Theme::light();
        t.invalid_cell_border = crate::primitives::Color::rgba(0, 0, 0, 0);
        t.invalid_cell_bg = crate::primitives::Color::rgba(0, 0, 0, 0);
        emit_cell(
            &mut frame,
            &col,
            &model,
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Ready(String::new()),
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        // No bg/border rect — empty text yields no primitives at all.
        assert_eq!(frame.primitive_count(), 0);
    }

    #[test]
    fn emit_cell_invalid_and_locked_both_render() {
        let mut frame = make_frame();
        let col = make_col().read_only().with_rules(vec![
            rs_grid_core::validation::ValidationRule::required(),
        ]);
        let model = make_model(&col);
        let sel = SelectionState::default();
        let t = Theme::light();
        let mut invalid_borders = Vec::new();
        emit_cell(
            &mut frame,
            &col,
            &model,
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Ready(String::new()),
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut invalid_borders,
        );
        // Locked overlay (fill) + invalid bg — both conditions can hold
        // at once and don't suppress each other. The invalid border is
        // collected separately (see the border-overlay test above) so
        // it isn't a frame primitive here.
        assert_eq!(frame.primitive_count(), 2);
        match &frame.primitives[0] {
            ScenePrimitive::Rect(r) => assert_eq!(r.fill, t.locked_cell_bg),
            _ => panic!("expected the locked-cell overlay Rect first"),
        }
        match &frame.primitives[1] {
            ScenePrimitive::Rect(r) => assert_eq!(r.fill, t.invalid_cell_bg),
            _ => panic!("expected the invalid-cell bg Rect second"),
        }
        assert_eq!(invalid_borders, vec![(0.0, 0.0, col.width, 42.0)]);
    }

    // ── at-rest cell decoration ───────────────────────────────

    #[test]
    fn emit_cell_decoration_emits_tint_and_border() {
        let mut frame = make_frame();
        let col = make_col().decorated_when(|_, _| {
            Some(
                CellDecoration::default()
                    .with_border_color([239, 68, 68, 255])
                    .with_background_tint([255, 0, 0, 40]),
            )
        });
        let model = make_model(&col);
        let sel = SelectionState::default();
        let t = Theme::light();
        emit_cell(
            &mut frame,
            &col,
            &model,
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Ready("ok".into()),
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        // tint Rect + border Rect + text primitive.
        assert_eq!(frame.primitive_count(), 3);
        match &frame.primitives[0] {
            ScenePrimitive::Rect(r) => {
                assert_eq!(
                    r.fill,
                    crate::primitives::Color::rgba(255, 0, 0, 40)
                );
                assert!(r.stroke.is_none());
            }
            _ => panic!("expected the tint Rect first"),
        }
        match &frame.primitives[1] {
            ScenePrimitive::Rect(r) => {
                assert_eq!(
                    r.stroke,
                    Some(crate::primitives::Color::rgba(239, 68, 68, 255))
                );
                assert_eq!(r.stroke_width, t.decoration_border_width);
                assert_eq!(r.fill.a, 0);
            }
            _ => panic!("expected the border Rect second"),
        }
    }

    #[test]
    fn emit_cell_decoration_skipped_when_none() {
        let mut frame = make_frame();
        let col = make_col(); // no decorator attached
        let model = make_model(&col);
        let sel = SelectionState::default();
        let t = Theme::light();
        emit_cell(
            &mut frame,
            &col,
            &model,
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Ready("ok".into()),
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        assert_eq!(frame.primitive_count(), 1);
        assert!(matches!(frame.primitives[0], ScenePrimitive::Text(_)));
    }

    #[test]
    fn emit_cell_decoration_border_only_emits_single_rect() {
        let mut frame = make_frame();
        let col = make_col().decorated_when(|_, _| {
            Some(
                CellDecoration::default().with_border_color([239, 68, 68, 255]),
            )
        });
        let model = make_model(&col);
        let sel = SelectionState::default();
        let t = Theme::light();
        emit_cell(
            &mut frame,
            &col,
            &model,
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Ready("ok".into()),
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut Vec::new(),
        );
        // border Rect + text — no tint Rect since it wasn't set.
        assert_eq!(frame.primitive_count(), 2);
    }

    #[test]
    fn emit_cell_decoration_composes_with_locked_and_invalid() {
        let mut frame = make_frame();
        let col = make_col()
            .read_only()
            .with_rules(vec![
                rs_grid_core::validation::ValidationRule::required(),
            ])
            .decorated_when(|_, _| {
                Some(
                    CellDecoration::default()
                        .with_border_color([0, 0, 255, 255])
                        .with_background_tint([0, 0, 255, 30]),
                )
            });
        let model = make_model(&col);
        let sel = SelectionState::default();
        let t = Theme::light();
        let mut invalid_borders = Vec::new();
        emit_cell(
            &mut frame,
            &col,
            &model,
            0,
            0,
            0.0,
            0.0,
            21.0,
            42.0,
            CellStatus::Ready(String::new()),
            &sel,
            &no_search(),
            None,
            &t,
            None,
            None,
            &mut invalid_borders,
        );
        // locked fill + invalid bg + decoration tint + decoration
        // border — 4 overlay Rects, none suppressing others. The
        // invalid border is collected separately (not a frame
        // primitive here — see the border-overlay test above).
        assert_eq!(frame.primitive_count(), 4);
        match &frame.primitives[0] {
            ScenePrimitive::Rect(r) => assert_eq!(r.fill, t.locked_cell_bg),
            _ => panic!("expected the locked-cell overlay Rect first"),
        }
        match &frame.primitives[1] {
            ScenePrimitive::Rect(r) => assert_eq!(r.fill, t.invalid_cell_bg),
            _ => panic!("expected the invalid-cell bg Rect second"),
        }
        match &frame.primitives[2] {
            ScenePrimitive::Rect(r) => {
                assert_eq!(
                    r.fill,
                    crate::primitives::Color::rgba(0, 0, 255, 30)
                );
                assert!(r.stroke.is_none());
            }
            _ => panic!("expected the decoration tint Rect third"),
        }
        match &frame.primitives[3] {
            ScenePrimitive::Rect(r) => {
                assert_eq!(
                    r.stroke,
                    Some(crate::primitives::Color::rgba(0, 0, 255, 255))
                );
            }
            _ => panic!("expected the decoration border Rect fourth"),
        }
        assert_eq!(invalid_borders, vec![(0.0, 0.0, col.width, 42.0)]);
    }
}

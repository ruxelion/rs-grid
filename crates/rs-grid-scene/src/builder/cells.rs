use std::collections::HashSet;

use rs_grid_core::{
    column::{ButtonStyle, ColumnDef},
    datasource::CellStatus,
    format::{format_cell, CellAlign, CellElement, CellFormat},
    selection::SelectionState,
};

use crate::class_map::ClassResolver;

use crate::{
    frame::SceneFrame,
    primitives::{
        Color, ImagePrimitive, RectPrimitive, ScenePrimitive, TextAlign,
        TextPrimitive,
    },
    theme::Theme,
};

use super::FlashHint;

/// Emit selection fill, search highlight, and cell content
/// (text, image, or skeleton) for a single cell.
///
/// Shared by the scrollable-column and pinned-column render
/// loops to avoid duplicating logic.
#[allow(clippy::too_many_arguments)]
pub(super) fn emit_cell(
    frame: &mut SceneFrame,
    col: &ColumnDef,
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
        // Flash overlay — themed fade on paste
        if let Some(f) = flash {
            let a = (t.flash_fill.a as f64 * f.alpha_factor).round() as u8;
            frame.push(ScenePrimitive::Rect(RectPrimitive {
                x: cx,
                y: ry,
                width: col.width,
                height: row_height,
                fill: Color::rgba(
                    t.flash_fill.r,
                    t.flash_fill.g,
                    t.flash_fill.b,
                    a,
                ),
                stroke: None,
                stroke_width: 0.0,
                corner_radius: 0.0,
                clip: None,
            }));
        }
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
                    clip: Some([cx, ry, col.width, row_height]),
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
                    mid_y,
                    t,
                    base_url,
                    suffix,
                    *image_size,
                    *border_radius,
                    *gap,
                );
            } else {
                let (txt, align, bold, italic, color) = if let Some(fmt)
                    = &col.format
                {
                    let fc = format_cell(&raw, fmt);
                    let a = match fc.align.unwrap_or_default() {
                        CellAlign::Left => TextAlign::Left,
                        CellAlign::Right => TextAlign::Right,
                        CellAlign::Center => TextAlign::Center,
                        _ => TextAlign::Left,
                    };
                    let c = fc
                        .color
                        .map(|c| Color::rgba(c[0], c[1], c[2], c[3]))
                        .unwrap_or(t.cell_text);
                    (fc.text, a, fc.bold || col.bold, fc.italic, c)
                } else {
                    (raw, TextAlign::Left, col.bold, false, t.cell_text)
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
                    clip: Some([cx, ry, col.width, row_height]),
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
    emit_cell_buttons(frame, col, ri, ci, cx, ry, row_height, t);
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use rs_grid_core::{
        column::ColumnDef, datasource::CellStatus, format::CellFormat,
        selection::SelectionState,
    };

    use crate::{
        builder::FlashHint, frame::SceneFrame, primitives::ScenePrimitive,
        theme::Theme,
    };

    use super::emit_cell;

    // ── helpers ──────────────────────────────────────────────

    fn make_frame() -> SceneFrame {
        SceneFrame::new(800.0, 600.0, 1.0)
    }

    fn make_col() -> ColumnDef {
        ColumnDef::new("a", "Alpha", 100.0)
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
        let flash = FlashHint { alpha_factor: 0.5 };
        emit_cell(
            &mut frame,
            &col,
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
        );
        // selection fill + flash overlay = 2 Rect primitives
        assert_eq!(frame.primitive_count(), 2);
        assert!(frame
            .primitives
            .iter()
            .all(|p| matches!(p, ScenePrimitive::Rect(_))));
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
        use rs_grid_core::format::{CellAlign, CellElement};
        use std::rc::Rc;

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
        );
        // No background → only a Text primitive.
        assert_eq!(frame.primitive_count(), 1);
        assert!(matches!(
            frame.primitives[0],
            ScenePrimitive::Text(_)
        ));
    }

    #[test]
    fn emit_cell_styled_with_bg_emits_rect_and_text() {
        use crate::class_map::CellElementStyle;
        use crate::primitives::Color;
        use rs_grid_core::format::{CellAlign, CellElement};
        use std::rc::Rc;

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
        );
        // background rect + text
        assert_eq!(frame.primitive_count(), 2);
        assert!(matches!(frame.primitives[0], ScenePrimitive::Rect(_)));
        assert!(matches!(frame.primitives[1], ScenePrimitive::Text(_)));
    }

    #[test]
    fn emit_cell_styled_multiple_elements_emits_all() {
        use rs_grid_core::format::{CellAlign, CellElement};
        use std::rc::Rc;

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
        );
        // 2 elements → 2 Text primitives (no bg).
        assert_eq!(frame.primitive_count(), 2);
    }

    // ── emit_cell_buttons ────────────────────────────────

    #[test]
    fn emit_cell_button_primary_emits_rect_text_and_zone() {
        use rs_grid_core::column::{ButtonDef, ButtonStyle};

        let mut frame = make_frame();
        let col =
            ColumnDef::new("x", "X", 200.0).with_cell_buttons(vec![
                ButtonDef::new("save", "Save", ButtonStyle::Primary),
            ]);
        let sel = SelectionState::default();
        let t = Theme::light();
        emit_cell(
            &mut frame,
            &col,
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
        let col =
            ColumnDef::new("x", "X", 200.0).with_cell_buttons(vec![
                ButtonDef::new("g", "Ghost", ButtonStyle::Ghost),
            ]);
        let sel = SelectionState::default();
        let t = Theme::light();
        emit_cell(
            &mut frame,
            &col,
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
        );
        let has_stroke = frame.primitives.iter().any(|p| {
            matches!(p, ScenePrimitive::Rect(r) if r.stroke.is_some())
        });
        assert!(has_stroke, "Ghost button should have a stroke rect");
    }

    #[test]
    fn emit_cell_button_secondary_and_danger_emit_zones() {
        use rs_grid_core::column::{ButtonDef, ButtonStyle};

        let mut frame = make_frame();
        let col =
            ColumnDef::new("x", "X", 400.0).with_cell_buttons(vec![
                ButtonDef::new("d", "Del", ButtonStyle::Danger),
                ButtonDef::new("s", "Sec", ButtonStyle::Secondary),
            ]);
        let sel = SelectionState::default();
        let t = Theme::light();
        emit_cell(
            &mut frame,
            &col,
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
        );
        assert_eq!(frame.primitive_count(), 1);
        assert!(
            matches!(frame.primitives[0], ScenePrimitive::Image(_)),
            "expected Image only"
        );
    }
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
        clip: Some([cx, ry, col_width, row_height]),
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
            clip: Some([cx, ry, col_width, row_height]),
            align: TextAlign::Left,
            max_width: Some(
                (col_width - 2.0 * t.cell_padding - image_size - gap).max(0.0),
            ),
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
    t: &Theme,
    class_resolver: Option<&ClassResolver>,
) {
    let clip = [cx, ry, cell_w, row_height];
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
    let clip = [cx, ry, col.width, row_height];

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

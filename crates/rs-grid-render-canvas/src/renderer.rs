use rs_grid_scene::{
    frame::SceneFrame,
    primitives::{
        LinePrimitive, PolygonPrimitive, RectPrimitive, ScenePrimitive,
        TextAlign, TextPrimitive,
    },
};
use web_sys::CanvasRenderingContext2d;

/// Renders a `SceneFrame` onto a `CanvasRenderingContext2d`.
pub struct CanvasRenderer {
    ctx: CanvasRenderingContext2d,
}

impl CanvasRenderer {
    pub fn new(ctx: CanvasRenderingContext2d) -> Self {
        Self { ctx }
    }

    /// Clear the canvas and draw all primitives in the frame.
    pub fn render(&self, frame: &SceneFrame) {
        let ctx = &self.ctx;
        let dpr = frame.dpr;

        ctx.save();

        // Scale for device pixel ratio so all coordinates are in CSS pixels.
        ctx.scale(dpr, dpr).expect("canvas scale should not fail");

        ctx.clear_rect(0.0, 0.0, frame.viewport_width, frame.viewport_height);

        for prim in &frame.primitives {
            match prim {
                ScenePrimitive::Rect(r) => self.draw_rect(r),
                ScenePrimitive::Text(t) => self.draw_text(t),
                ScenePrimitive::Line(l) => self.draw_line(l),
                ScenePrimitive::Polygon(p) => self.draw_polygon(p),
            }
        }

        ctx.restore();
    }

    fn draw_rect(&self, r: &RectPrimitive) {
        let ctx = &self.ctx;
        if r.corner_radius > 0.0 {
            let rad = r.corner_radius.min(r.width / 2.0).min(r.height / 2.0);
            let (x, y, w, h) = (r.x, r.y, r.width, r.height);
            ctx.begin_path();
            ctx.move_to(x + rad, y);
            ctx.line_to(x + w - rad, y);
            ctx.arc_to(x + w, y, x + w, y + rad, rad).unwrap();
            ctx.line_to(x + w, y + h - rad);
            ctx.arc_to(x + w, y + h, x + w - rad, y + h, rad).unwrap();
            ctx.line_to(x + rad, y + h);
            ctx.arc_to(x, y + h, x, y + h - rad, rad).unwrap();
            ctx.line_to(x, y + rad);
            ctx.arc_to(x, y, x + rad, y, rad).unwrap();
            ctx.close_path();
            ctx.set_fill_style_str(&r.fill.to_css());
            ctx.fill();
        } else {
            ctx.set_fill_style_str(&r.fill.to_css());
            ctx.fill_rect(r.x, r.y, r.width, r.height);
        }

        if let Some(stroke) = r.stroke {
            ctx.save();
            ctx.set_stroke_style_str(&stroke.to_css());
            ctx.set_line_width(r.stroke_width);
            ctx.stroke_rect(r.x, r.y, r.width, r.height);
            ctx.restore();
        }
    }

    fn draw_text(&self, t: &TextPrimitive) {
        let ctx = &self.ctx;
        ctx.save();

        if let Some([cx, cy, cw, ch]) = t.clip {
            ctx.begin_path();
            ctx.rect(cx.round(), cy.round(), cw.round(), ch.round());
            ctx.clip();
        }

        let weight = if t.bold { "600" } else { "400" };
        ctx.set_fill_style_str(&t.color.to_css());
        // Round font size to integer: consistent glyph metrics across cells.
        ctx.set_font(&format!(
            "{} {}px system-ui, sans-serif",
            weight,
            t.font_size.round() as u32,
        ));
        ctx.set_text_baseline("alphabetic");
        ctx.set_text_align(match t.align {
            TextAlign::Left => "left",
            TextAlign::Right => "right",
            TextAlign::Center => "center",
        });
        // Round to integer CSS pixels: avoids sub-pixel blur on text.
        let _ = ctx.fill_text(&t.text, t.x.round(), t.y.round());

        ctx.restore();
    }

    fn draw_polygon(&self, p: &PolygonPrimitive) {
        let ctx = &self.ctx;
        let n = p.points.len();
        if n < 2 {
            return;
        }

        ctx.begin_path();

        if p.corner_radius <= 0.0 {
            ctx.move_to(p.points[0][0], p.points[0][1]);
            for pt in p.points.iter().skip(1) {
                ctx.line_to(pt[0], pt[1]);
            }
        } else {
            let r = p.corner_radius;
            // For each vertex, round the corner using arcTo.
            for i in 0..n {
                let prev = p.points[(i + n - 1) % n];
                let curr = p.points[i];
                let next = p.points[(i + 1) % n];

                // Unit vector from curr toward prev (incoming edge direction reversed).
                let dx_in = prev[0] - curr[0];
                let dy_in = prev[1] - curr[1];
                let len_in = (dx_in * dx_in + dy_in * dy_in).sqrt().max(1e-9);
                // Entry point: walk back from curr along the incoming edge by r.
                let px = curr[0] + dx_in / len_in * r;
                let py = curr[1] + dy_in / len_in * r;

                // Unit vector from curr toward next (outgoing edge).
                let dx_out = next[0] - curr[0];
                let dy_out = next[1] - curr[1];
                let len_out =
                    (dx_out * dx_out + dy_out * dy_out).sqrt().max(1e-9);
                // Exit point: walk forward from curr along the outgoing edge by r.
                let qx = curr[0] + dx_out / len_out * r;
                let qy = curr[1] + dy_out / len_out * r;

                if i == 0 {
                    ctx.move_to(px, py);
                } else {
                    ctx.line_to(px, py);
                }
                // arcTo rounds the corner between (px,py) → curr → (qx,qy).
                ctx.arc_to(curr[0], curr[1], qx, qy, r).unwrap();
            }
        }

        ctx.close_path();
        ctx.set_fill_style_str(&p.fill.to_css());
        ctx.fill();
    }

    fn draw_line(&self, l: &LinePrimitive) {
        let ctx = &self.ctx;
        ctx.save();
        ctx.set_stroke_style_str(&l.color.to_css());
        ctx.set_line_width(l.width);
        ctx.begin_path();
        ctx.move_to(l.x1, l.y1);
        ctx.line_to(l.x2, l.y2);
        ctx.stroke();
        ctx.restore();
    }
}

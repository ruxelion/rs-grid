use rs_grid_scene::{
    frame::SceneFrame,
    primitives::{LinePrimitive, RectPrimitive, ScenePrimitive, TextPrimitive},
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
        ctx.scale(dpr, dpr)
            .expect("canvas scale should not fail");

        ctx.clear_rect(0.0, 0.0, frame.viewport_width, frame.viewport_height);

        for prim in &frame.primitives {
            match prim {
                ScenePrimitive::Rect(r) => self.draw_rect(r),
                ScenePrimitive::Text(t) => self.draw_text(t),
                ScenePrimitive::Line(l) => self.draw_line(l),
            }
        }

        ctx.restore();
    }

    fn draw_rect(&self, r: &RectPrimitive) {
        let ctx = &self.ctx;
        ctx.set_fill_style_str(&r.fill.to_css());
        ctx.fill_rect(r.x, r.y, r.width, r.height);

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
            ctx.rect(cx, cy, cw, ch);
            ctx.clip();
        }

        ctx.set_fill_style_str(&t.color.to_css());
        ctx.set_font(&format!("{}px system-ui, sans-serif", t.font_size));
        ctx.set_text_baseline("alphabetic");
        // Ignore the Result — fill_text only fails on infinite coords.
        let _ = ctx.fill_text(&t.text, t.x, t.y);

        ctx.restore();
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

use std::{cell::RefCell, collections::HashMap};

use rs_grid_scene::{
    frame::SceneFrame,
    primitives::{
        ImagePrimitive, LinePrimitive, PolygonPrimitive, RectPrimitive,
        ScenePrimitive, TextAlign, TextPrimitive,
    },
};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlImageElement};

// ── image cache ─────────────────────────────────────────

/// Max loaded images kept in memory.
const MAX_CACHED: usize = 200;
/// Max concurrent image loads (matches browser connection
/// limit per hostname).
const MAX_PENDING: usize = 8;

struct ImageCache {
    entries: HashMap<String, HtmlImageElement>,
    /// Access order for LRU eviction (back = most recent).
    order: Vec<String>,
}

impl ImageCache {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            order: Vec::new(),
        }
    }

    /// Look up an image by URL. Returns `None` if not
    /// cached. Bumps the entry to most-recent on hit.
    fn get(&mut self, url: &str) -> Option<&HtmlImageElement> {
        if self.entries.contains_key(url) {
            self.touch(url);
            self.entries.get(url)
        } else {
            None
        }
    }

    /// Insert a new entry and evict old loaded entries if
    /// the cache is over capacity.
    fn insert(&mut self, url: String, el: HtmlImageElement) {
        self.order.push(url.clone());
        self.entries.insert(url, el);
        self.evict();
    }

    /// Count how many entries are still loading.
    fn pending_count(&self) -> usize {
        self.entries
            .values()
            .filter(|el| el.natural_width() == 0)
            .count()
    }

    /// Move a URL to the back (most recently used).
    fn touch(&mut self, url: &str) {
        if let Some(pos) = self.order.iter().position(|u| u == url) {
            self.order.remove(pos);
            self.order.push(url.to_owned());
        }
    }

    /// Evict oldest *loaded* entries until we're at
    /// capacity. Never evicts pending (loading) entries.
    fn evict(&mut self) {
        while self.entries.len() > MAX_CACHED {
            // Find the oldest loaded entry to evict.
            let evict_pos = self.order.iter().position(|url| {
                self.entries
                    .get(url)
                    .is_some_and(|el| el.natural_width() > 0)
            });
            match evict_pos {
                Some(pos) => {
                    let url = self.order.remove(pos);
                    self.entries.remove(&url);
                }
                None => break, // all entries pending
            }
        }
    }
}

// ── renderer ────────────────────────────────────────────

/// Renders a `SceneFrame` onto a `CanvasRenderingContext2d`.
pub struct CanvasRenderer {
    ctx: CanvasRenderingContext2d,
    image_cache: RefCell<ImageCache>,
}

impl CanvasRenderer {
    /// Wrap a Canvas2D context into a renderer.
    pub fn new(ctx: CanvasRenderingContext2d) -> Self {
        Self {
            ctx,
            image_cache: RefCell::new(ImageCache::new()),
        }
    }

    /// Clear the canvas and draw all primitives in the frame.
    pub fn render(&self, frame: &SceneFrame) {
        let ctx = &self.ctx;
        let dpr = frame.dpr;

        ctx.save();

        // Scale for device pixel ratio so all coordinates
        // are in CSS pixels.
        ctx.scale(dpr, dpr).expect("canvas scale should not fail");

        ctx.clear_rect(0.0, 0.0, frame.viewport_width, frame.viewport_height);

        for prim in &frame.primitives {
            match prim {
                ScenePrimitive::Rect(r) => self.draw_rect(r),
                ScenePrimitive::Text(t) => self.draw_text(t),
                ScenePrimitive::Line(l) => self.draw_line(l),
                ScenePrimitive::Polygon(p) => self.draw_polygon(p),
                ScenePrimitive::Image(img) => self.draw_image(img),
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
            ctx.arc_to(x + w, y, x + w, y + rad, rad).expect("arc_to");
            ctx.line_to(x + w, y + h - rad);
            ctx.arc_to(x + w, y + h, x + w - rad, y + h, rad)
                .expect("arc_to");
            ctx.line_to(x + rad, y + h);
            ctx.arc_to(x, y + h, x, y + h - rad, rad).expect("arc_to");
            ctx.line_to(x, y + rad);
            ctx.arc_to(x, y, x + rad, y, rad).expect("arc_to");
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
            for i in 0..n {
                let prev = p.points[(i + n - 1) % n];
                let curr = p.points[i];
                let next = p.points[(i + 1) % n];

                let dx_in = prev[0] - curr[0];
                let dy_in = prev[1] - curr[1];
                let len_in = (dx_in * dx_in + dy_in * dy_in).sqrt().max(1e-9);
                let px = curr[0] + dx_in / len_in * r;
                let py = curr[1] + dy_in / len_in * r;

                let dx_out = next[0] - curr[0];
                let dy_out = next[1] - curr[1];
                let len_out =
                    (dx_out * dx_out + dy_out * dy_out).sqrt().max(1e-9);
                let qx = curr[0] + dx_out / len_out * r;
                let qy = curr[1] + dy_out / len_out * r;

                if i == 0 {
                    ctx.move_to(px, py);
                } else {
                    ctx.line_to(px, py);
                }
                ctx.arc_to(curr[0], curr[1], qx, qy, r).expect("arc_to");
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

    fn draw_image(&self, img: &ImagePrimitive) {
        let ctx = &self.ctx;
        let mut cache = self.image_cache.borrow_mut();

        // Try to get from cache (bumps LRU).
        let cached = cache.get(&img.url).cloned();

        let el = if let Some(el) = cached {
            Some(el)
        } else {
            // Not cached — only create if pending load
            // slots are available.
            if cache.pending_count() < MAX_PENDING {
                let doc = web_sys::window()
                    .expect("no window")
                    .document()
                    .expect("no document");
                let el = doc
                    .create_element("img")
                    .expect("create img")
                    .dyn_into::<HtmlImageElement>()
                    .expect("cast");
                el.set_src(&img.url);
                cache.insert(img.url.clone(), el.clone());
                Some(el)
            } else {
                // Too many pending — just draw placeholder,
                // don't create element. Will retry next
                // frame when a slot frees up.
                None
            }
        };

        // Release borrow before drawing.
        drop(cache);

        ctx.save();

        // Cell clipping — round to integer pixels (same as text clipping)
        if let Some([cx, cy, cw, ch]) = img.clip {
            ctx.begin_path();
            ctx.rect(cx.round(), cy.round(), cw.round(), ch.round());
            ctx.clip();
        }

        match el {
            Some(ref el) if el.natural_width() > 0 => {
                self.draw_loaded_image(ctx, img, el);
            }
            _ => {
                self.draw_image_placeholder(ctx, img);
            }
        }

        ctx.restore();
    }

    fn draw_loaded_image(
        &self,
        ctx: &CanvasRenderingContext2d,
        img: &ImagePrimitive,
        el: &HtmlImageElement,
    ) {
        let nat_w = el.natural_width() as f64;
        let nat_h = el.natural_height() as f64;
        // Guard against degenerate images (zero-size would cause div/0)
        if nat_w <= 0.0 || nat_h <= 0.0 {
            return;
        }

        // object-fit: contain
        let scale = (img.width / nat_w).min(img.height / nat_h);
        let draw_w = nat_w * scale;
        let draw_h = nat_h * scale;
        let draw_x = img.x + (img.width - draw_w) / 2.0;
        let draw_y = img.y + (img.height - draw_h) / 2.0;

        // Rounded corners
        if img.corner_radius > 0.0 {
            let r = img.corner_radius.min(draw_w / 2.0).min(draw_h / 2.0);
            self.rounded_rect_path(ctx, draw_x, draw_y, draw_w, draw_h, r);
            ctx.clip();
        }

        let _ = ctx.draw_image_with_html_image_element_and_dw_and_dh(
            el, draw_x, draw_y, draw_w, draw_h,
        );
    }

    fn draw_image_placeholder(
        &self,
        ctx: &CanvasRenderingContext2d,
        img: &ImagePrimitive,
    ) {
        let bar_w = img.width * 0.5;
        let bar_h = img.height * 0.5;
        let bar_x = img.x + (img.width - bar_w) / 2.0;
        let bar_y = img.y + (img.height - bar_h) / 2.0;
        ctx.set_fill_style_str("rgba(200,200,200,0.4)");
        let r = if img.corner_radius > 0.0 {
            img.corner_radius.min(bar_w / 2.0).min(bar_h / 2.0)
        } else {
            3.0_f64.min(bar_w / 2.0).min(bar_h / 2.0)
        };
        self.rounded_rect_path(ctx, bar_x, bar_y, bar_w, bar_h, r);
        ctx.fill();
    }

    /// Build a rounded-rect path (reused by image
    /// drawing and placeholder).
    fn rounded_rect_path(
        &self,
        ctx: &CanvasRenderingContext2d,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        r: f64,
    ) {
        ctx.begin_path();
        ctx.move_to(x + r, y);
        ctx.line_to(x + w - r, y);
        ctx.arc_to(x + w, y, x + w, y + r, r).expect("arc_to");
        ctx.line_to(x + w, y + h - r);
        ctx.arc_to(x + w, y + h, x + w - r, y + h, r)
            .expect("arc_to");
        ctx.line_to(x + r, y + h);
        ctx.arc_to(x, y + h, x, y + h - r, r).expect("arc_to");
        ctx.line_to(x, y + r);
        ctx.arc_to(x, y, x + r, y, r).expect("arc_to");
        ctx.close_path();
    }
}

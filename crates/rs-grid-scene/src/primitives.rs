/// A simple RGBA color.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    /// Red channel (0–255).
    pub r: u8,
    /// Green channel (0–255).
    pub g: u8,
    /// Blue channel (0–255).
    pub b: u8,
    /// Alpha channel (0 = transparent, 255 = opaque).
    pub a: u8,
}

impl Color {
    /// Create a color from red, green, blue, and alpha.
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create a fully opaque color from red, green, and blue.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r, g, b, 255)
    }

    /// Produce a CSS `rgba(...)` string understood by Canvas2D.
    pub fn to_css(self) -> String {
        let a_f = self.a as f64 / 255.0;
        format!("rgba({},{},{},{:.4})", self.r, self.g, self.b, a_f)
    }

    /// Format as a CSS custom-property value.
    /// Opaque (`a == 255`) → `#rrggbb`; semi-transparent →
    /// `rgba(r, g, b, a)` with `a` as a 0–1 float (2 decimal places).
    pub fn to_css_var(self) -> String {
        if self.a == 255 {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            let a = self.a as f64 / 255.0;
            format!("rgba({}, {}, {}, {:.2})", self.r, self.g, self.b, a)
        }
    }
}

/// A filled (and optionally stroked) rectangle.
#[derive(Debug, Clone)]
pub struct RectPrimitive {
    /// Left edge in logical pixels.
    pub x: f64,
    /// Top edge in logical pixels.
    pub y: f64,
    /// Width in logical pixels.
    pub width: f64,
    /// Height in logical pixels.
    pub height: f64,
    /// Fill color.
    pub fill: Color,
    /// Optional stroke color (`None` = no stroke).
    pub stroke: Option<Color>,
    /// Stroke width in logical pixels.
    pub stroke_width: f64,
    /// Corner radius in logical pixels (0 = sharp corners).
    pub corner_radius: f64,
    /// Optional clipping rectangle `[x, y, w, h]` in logical pixels.
    pub clip: Option<[f64; 4]>,
}

/// Horizontal text alignment.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TextAlign {
    /// Left-aligned (default).
    #[default]
    Left,
    /// Centered.
    Center,
    /// Right-aligned.
    Right,
}

/// A clipped text run.
#[derive(Debug, Clone)]
pub struct TextPrimitive {
    /// Left edge in logical pixels.
    pub x: f64,
    /// Baseline y-position in logical pixels.
    pub y: f64,
    /// Text content to render.
    pub text: String,
    /// Text color.
    pub color: Color,
    /// Font size in logical pixels.
    pub font_size: f64,
    /// Render with font-weight 600 when `true`.
    pub bold: bool,
    /// Render with `font-style: italic` when `true`.
    pub italic: bool,
    /// Optional clipping rectangle `(x, y, width, height)`.
    pub clip: Option<[f64; 4]>,
    /// Horizontal text alignment.
    pub align: TextAlign,
    /// If set, text is truncated with "…" to fit within this
    /// logical-pixel width. Measurement happens in the renderer
    /// using Canvas2D `measureText()`.
    pub max_width: Option<f64>,
}

/// A straight line segment.
#[derive(Debug, Clone)]
pub struct LinePrimitive {
    /// Start x in logical pixels.
    pub x1: f64,
    /// Start y in logical pixels.
    pub y1: f64,
    /// End x in logical pixels.
    pub x2: f64,
    /// End y in logical pixels.
    pub y2: f64,
    /// Line color.
    pub color: Color,
    /// Line width in logical pixels.
    pub width: f64,
}

/// A filled convex polygon with optional rounded corners.
#[derive(Debug, Clone)]
pub struct PolygonPrimitive {
    /// Vertices as `[x, y]` pairs in logical pixels.
    pub points: Vec<[f64; 2]>,
    /// Fill color.
    pub fill: Color,
    /// Corner radius in logical pixels (0 = sharp corners).
    pub corner_radius: f64,
}

/// An image loaded from a URL.
///
/// The renderer resolves the URL to a loaded element,
/// caches it, and draws it with object-fit: contain
/// semantics. While loading, a placeholder is shown.
#[derive(Debug, Clone)]
pub struct ImagePrimitive {
    /// Fully resolved URL of the image.
    pub url: String,
    /// Left edge in logical pixels.
    pub x: f64,
    /// Top edge in logical pixels.
    pub y: f64,
    /// Available width in logical pixels.
    pub width: f64,
    /// Available height in logical pixels.
    pub height: f64,
    /// Corner radius for rounded clipping (0 = sharp).
    pub corner_radius: f64,
    /// Optional clipping rectangle `[x, y, w, h]`.
    pub clip: Option<[f64; 4]>,
    /// Color of the placeholder shown while the image is loading.
    pub placeholder_color: Color,
}

/// Sum type over all renderable primitives.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ScenePrimitive {
    /// Filled rectangle with optional stroke.
    Rect(RectPrimitive),
    /// Clipped text run.
    Text(TextPrimitive),
    /// Straight line segment.
    Line(LinePrimitive),
    /// Filled convex polygon.
    Polygon(PolygonPrimitive),
    /// Image loaded from a URL.
    Image(ImagePrimitive),
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Color ────────────────────────────────────────────────

    #[test]
    fn color_rgba_stores_all_channels() {
        let c = Color::rgba(10, 20, 30, 40);
        assert_eq!((c.r, c.g, c.b, c.a), (10, 20, 30, 40));
    }

    #[test]
    fn color_rgb_is_fully_opaque() {
        let c = Color::rgb(100, 150, 200);
        assert_eq!(c.a, 255);
        assert_eq!((c.r, c.g, c.b), (100, 150, 200));
    }

    #[test]
    fn color_to_css_opaque() {
        let c = Color::rgb(255, 128, 0);
        assert_eq!(c.to_css(), "rgba(255,128,0,1.0000)");
    }

    #[test]
    fn color_to_css_transparent() {
        let c = Color::rgba(10, 20, 30, 0);
        assert_eq!(c.to_css(), "rgba(10,20,30,0.0000)");
    }

    #[test]
    fn color_to_css_half_alpha() {
        let c = Color::rgba(0, 0, 0, 128);
        let css = c.to_css();
        // 128/255 ≈ 0.5020
        assert!(css.starts_with("rgba(0,0,0,0.50"));
    }

    #[test]
    fn color_equality() {
        let a = Color::rgb(1, 2, 3);
        let b = Color::rgba(1, 2, 3, 255);
        assert_eq!(a, b);
    }

    #[test]
    fn color_copy_semantics() {
        let a = Color::rgb(10, 20, 30);
        let b = a; // Copy
        assert_eq!(a, b);
    }

    // ── TextAlign ───────────────────────────────────────────

    #[test]
    fn text_align_default_is_left() {
        assert_eq!(TextAlign::default(), TextAlign::Left);
    }

    #[test]
    fn text_align_variants_distinct() {
        assert_ne!(TextAlign::Left, TextAlign::Center);
        assert_ne!(TextAlign::Center, TextAlign::Right);
        assert_ne!(TextAlign::Left, TextAlign::Right);
    }

    // ── RectPrimitive ───────────────────────────────────────

    #[test]
    fn rect_primitive_no_stroke() {
        let r = RectPrimitive {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
            fill: Color::rgb(255, 0, 0),
            stroke: None,
            stroke_width: 0.0,
            corner_radius: 0.0,
            clip: None,
        };
        assert!(r.stroke.is_none());
        assert_eq!(r.x, 10.0);
    }

    #[test]
    fn rect_primitive_with_stroke() {
        let r = RectPrimitive {
            x: 0.0,
            y: 0.0,
            width: 50.0,
            height: 50.0,
            fill: Color::rgb(0, 0, 0),
            stroke: Some(Color::rgb(255, 0, 0)),
            stroke_width: 2.0,
            corner_radius: 4.0,
            clip: None,
        };
        assert!(r.stroke.is_some());
        assert_eq!(r.stroke_width, 2.0);
        assert_eq!(r.corner_radius, 4.0);
    }

    // ── TextPrimitive ───────────────────────────────────────

    #[test]
    fn text_primitive_fields() {
        let t = TextPrimitive {
            x: 5.0,
            y: 15.0,
            text: "Hello".into(),
            color: Color::rgb(0, 0, 0),
            font_size: 14.0,
            bold: true,
            italic: false,
            clip: Some([0.0, 0.0, 100.0, 50.0]),
            align: TextAlign::Center,
            max_width: Some(80.0),
        };
        assert_eq!(t.text, "Hello");
        assert!(t.bold);
        assert_eq!(t.align, TextAlign::Center);
        assert_eq!(t.max_width, Some(80.0));
    }

    #[test]
    fn text_primitive_no_clip_no_max_width() {
        let t = TextPrimitive {
            x: 0.0,
            y: 0.0,
            text: String::new(),
            color: Color::rgb(0, 0, 0),
            font_size: 12.0,
            bold: false,
            italic: false,
            clip: None,
            align: TextAlign::Left,
            max_width: None,
        };
        assert!(t.clip.is_none());
        assert!(t.max_width.is_none());
    }

    // ── LinePrimitive ───────────────────────────────────────

    #[test]
    fn line_primitive_fields() {
        let l = LinePrimitive {
            x1: 0.0,
            y1: 0.0,
            x2: 100.0,
            y2: 100.0,
            color: Color::rgb(128, 128, 128),
            width: 1.5,
        };
        assert_eq!(l.x2, 100.0);
        assert_eq!(l.width, 1.5);
    }

    // ── PolygonPrimitive ────────────────────────────────────

    #[test]
    fn polygon_primitive_triangle() {
        let p = PolygonPrimitive {
            points: vec![[0.0, 0.0], [50.0, 100.0], [100.0, 0.0]],
            fill: Color::rgb(0, 255, 0),
            corner_radius: 0.0,
        };
        assert_eq!(p.points.len(), 3);
    }

    // ── ImagePrimitive ──────────────────────────────────────

    #[test]
    fn image_primitive_fields() {
        let img = ImagePrimitive {
            url: "https://example.com/img.png".into(),
            x: 10.0,
            y: 20.0,
            width: 32.0,
            height: 32.0,
            corner_radius: 4.0,
            clip: Some([10.0, 20.0, 32.0, 32.0]),
            placeholder_color: Color::rgba(200, 200, 200, 100),
        };
        assert_eq!(img.url, "https://example.com/img.png");
        assert!(img.clip.is_some());
    }

    // ── ScenePrimitive enum ─────────────────────────────────

    #[test]
    fn scene_primitive_rect_variant() {
        let p = ScenePrimitive::Rect(RectPrimitive {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
            fill: Color::rgb(0, 0, 0),
            stroke: None,
            stroke_width: 0.0,
            corner_radius: 0.0,
            clip: None,
        });
        assert!(matches!(p, ScenePrimitive::Rect(_)));
    }

    // ── Color::to_css_var ───────────────────────────────────

    #[test]
    fn color_to_css_var_opaque() {
        let c = Color::rgb(0xde, 0xad, 0xbe);
        assert_eq!(c.to_css_var(), "#deadbe");
    }

    #[test]
    fn color_to_css_var_semi_transparent() {
        let c = Color::rgba(255, 128, 0, 128);
        let s = c.to_css_var();
        // Should be rgba(r, g, b, a) format with 2 decimal places
        assert!(s.starts_with("rgba(255, 128, 0,"), "unexpected: {s}");
        // 128/255 ≈ 0.50
        assert!(s.contains("0.50"), "unexpected alpha: {s}");
    }

    #[test]
    fn color_to_css_var_fully_transparent() {
        let c = Color::rgba(0, 0, 0, 0);
        let s = c.to_css_var();
        assert!(s.starts_with("rgba("), "unexpected: {s}");
        assert!(s.contains("0.00"), "unexpected: {s}");
    }

    #[test]
    fn color_to_css_var_opaque_all_channels() {
        let c = Color::rgb(0, 0, 0);
        assert_eq!(c.to_css_var(), "#000000");
        let c2 = Color::rgb(255, 255, 255);
        assert_eq!(c2.to_css_var(), "#ffffff");
    }

    #[test]
    fn scene_primitive_clone() {
        let p = ScenePrimitive::Text(TextPrimitive {
            x: 0.0,
            y: 0.0,
            text: "clone me".into(),
            color: Color::rgb(0, 0, 0),
            font_size: 12.0,
            bold: false,
            italic: false,
            clip: None,
            align: TextAlign::Left,
            max_width: None,
        });
        let p2 = p.clone();
        if let ScenePrimitive::Text(t) = p2 {
            assert_eq!(t.text, "clone me");
        } else {
            panic!("expected Text variant");
        }
    }
}

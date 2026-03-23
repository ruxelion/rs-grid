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
    /// Optional clipping rectangle `(x, y, width, height)`.
    pub clip: Option<[f64; 4]>,
    /// Horizontal text alignment.
    pub align: TextAlign,
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
}

/// Sum type over all renderable primitives.
#[derive(Debug, Clone)]
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

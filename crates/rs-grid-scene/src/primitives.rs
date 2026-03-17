/// A simple RGBA color.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

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
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub fill: Color,
    pub stroke: Option<Color>,
    pub stroke_width: f64,
}

/// Horizontal text alignment.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TextAlign {
    #[default]
    Left,
    Right,
}

/// A clipped text run.
#[derive(Debug, Clone)]
pub struct TextPrimitive {
    pub x: f64,
    pub y: f64,
    pub text: String,
    pub color: Color,
    pub font_size: f64,
    /// Optional clipping rectangle `(x, y, width, height)`.
    pub clip: Option<[f64; 4]>,
    pub align: TextAlign,
}

/// A straight line segment.
#[derive(Debug, Clone)]
pub struct LinePrimitive {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub color: Color,
    pub width: f64,
}

/// Sum type over all renderable primitives.
#[derive(Debug, Clone)]
pub enum ScenePrimitive {
    Rect(RectPrimitive),
    Text(TextPrimitive),
    Line(LinePrimitive),
}

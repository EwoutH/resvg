// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt;
use std::path::PathBuf;

// external
pub use svgdom::{
    Align,
    AspectRatio,
    Color,
    FuzzyEq,
    FuzzyZero,
    Transform,
};

// self
use geom::*;
pub use super::numbers::*;


macro_rules! enum_default {
    ($name:ident, $def_value:ident) => {
        impl Default for $name {
            fn default() -> Self {
                $name::$def_value
            }
        }
    };
}


/// A line cap.
///
/// `stroke-linecap` attribute in the SVG.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LineCap {
    Butt,
    Round,
    Square,
}

enum_default!(LineCap, Butt);

impl ToString for LineCap {
    fn to_string(&self) -> String {
        match self {
            LineCap::Butt   => "butt",
            LineCap::Round  => "round",
            LineCap::Square => "square",
        }.to_string()
    }
}


/// A line join.
///
/// `stroke-linejoin` attribute in the SVG.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LineJoin {
    Miter,
    Round,
    Bevel,
}

enum_default!(LineJoin, Miter);

impl ToString for LineJoin {
    fn to_string(&self) -> String {
        match self {
            LineJoin::Miter => "miter",
            LineJoin::Round => "round",
            LineJoin::Bevel => "bevel",
        }.to_string()
    }
}


/// A fill rule.
///
/// `fill-rule` attribute in the SVG.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FillRule {
    NonZero,
    EvenOdd,
}

enum_default!(FillRule, NonZero);

impl ToString for FillRule {
    fn to_string(&self) -> String {
        match self {
            FillRule::NonZero => "nonzero",
            FillRule::EvenOdd => "evenodd",
        }.to_string()
    }
}


/// An element units.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Units {
    UserSpaceOnUse,
    ObjectBoundingBox,
}

impl ToString for Units {
    fn to_string(&self) -> String {
        match self {
            Units::UserSpaceOnUse       => "userSpaceOnUse",
            Units::ObjectBoundingBox    => "objectBoundingBox",
        }.to_string()
    }
}


/// A spread method.
///
/// `spreadMethod` attribute in the SVG.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SpreadMethod {
    Pad,
    Reflect,
    Repeat,
}

enum_default!(SpreadMethod, Pad);

impl ToString for SpreadMethod {
    fn to_string(&self) -> String {
        match self {
            SpreadMethod::Pad       => "pad",
            SpreadMethod::Reflect   => "reflect",
            SpreadMethod::Repeat    => "repeat",
        }.to_string()
    }
}


/// A visibility property.
///
/// `visibility` attribute in the SVG.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Visibility {
    Visible,
    Hidden,
    Collapse,
}

enum_default!(Visibility, Visible);

impl ToString for Visibility {
    fn to_string(&self) -> String {
        match self {
            Visibility::Visible     => "visible",
            Visibility::Hidden      => "hidden",
            Visibility::Collapse    => "collapse",
        }.to_string()
    }
}


/// A text decoration style.
///
/// Defines the style of the line that should be rendered.
#[allow(missing_docs)]
#[derive(Clone, Default, Debug)]
pub struct TextDecorationStyle {
    pub fill: Option<Fill>,
    pub stroke: Option<Stroke>,
}


/// A text decoration.
#[derive(Clone, Default, Debug)]
pub struct TextDecoration {
    /// Draw underline using specified style.
    ///
    /// Should be drawn before/under text.
    pub underline: Option<TextDecorationStyle>,

    /// Draw overline using specified style.
    ///
    /// Should be drawn before/under text.
    pub overline: Option<TextDecorationStyle>,

    /// Draw line-through using specified style.
    ///
    /// Should be drawn after/over text.
    pub line_through: Option<TextDecorationStyle>,
}


/// A text anchor.
///
/// `text-anchor` attribute in the SVG.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TextAnchor {
    Start,
    Middle,
    End,
}

enum_default!(TextAnchor, Start);

impl ToString for TextAnchor {
    fn to_string(&self) -> String {
        match self {
            TextAnchor::Start   => "start",
            TextAnchor::Middle  => "middle",
            TextAnchor::End     => "end",
        }.to_string()
    }
}


/// A font style.
///
/// `font-style` attribute in the SVG.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

enum_default!(FontStyle, Normal);

impl ToString for FontStyle {
    fn to_string(&self) -> String {
        match self {
            FontStyle::Normal   => "normal",
            FontStyle::Italic   => "italic",
            FontStyle::Oblique  => "oblique",
        }.to_string()
    }
}


/// A font variant.
///
/// `font-variant` attribute in the SVG.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FontVariant {
    Normal,
    SmallCaps,
}

enum_default!(FontVariant, Normal);

impl ToString for FontVariant {
    fn to_string(&self) -> String {
        match self {
            FontVariant::Normal     => "normal",
            FontVariant::SmallCaps  => "small-caps",
        }.to_string()
    }
}


/// A font weight.
///
/// `font-weight` attribute in the SVG.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FontWeight {
    W100,
    W200,
    W300,
    W400,
    W500,
    W600,
    W700,
    W800,
    W900,
}

enum_default!(FontWeight, W400);

impl ToString for FontWeight {
    fn to_string(&self) -> String {
        match self {
            FontWeight::W100 => "100",
            FontWeight::W200 => "200",
            FontWeight::W300 => "300",
            FontWeight::W400 => "400",
            FontWeight::W500 => "500",
            FontWeight::W600 => "600",
            FontWeight::W700 => "700",
            FontWeight::W800 => "800",
            FontWeight::W900 => "900",
        }.to_string()
    }
}


/// A font stretch.
///
/// `font-stretch` attribute in the SVG.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FontStretch {
    Normal,
    Wider,
    Narrower,
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
}

enum_default!(FontStretch, Normal);

impl ToString for FontStretch {
    fn to_string(&self) -> String {
        match self {
            FontStretch::Normal         => "normal",
            FontStretch::Wider          => "wider",
            FontStretch::Narrower       => "narrower",
            FontStretch::UltraCondensed => "ultra-condensed",
            FontStretch::ExtraCondensed => "extra-condensed",
            FontStretch::Condensed      => "condensed",
            FontStretch::SemiCondensed  => "semi-condensed",
            FontStretch::SemiExpanded   => "semi-expanded",
            FontStretch::Expanded       => "expanded",
            FontStretch::ExtraExpanded  => "extra-expanded",
            FontStretch::UltraExpanded  => "ultra-expanded",
        }.to_string()
    }
}


/// A paint style.
///
/// `paint` value type in the SVG.
#[allow(missing_docs)]
#[derive(Clone)]
pub enum Paint {
    /// Paint with a color.
    Color(Color),

    /// Paint using a referenced element.
    Link(String),
}

impl fmt::Debug for Paint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Paint::Color(c) => write!(f, "Color({})", c),
            Paint::Link(_) => write!(f, "Link"),
        }
    }
}


/// A fill style.
#[allow(missing_docs)]
#[derive(Clone, Debug)]
pub struct Fill {
    pub paint: Paint,
    pub opacity: Opacity,
    pub rule: FillRule,
}

impl Default for Fill {
    fn default() -> Self {
        Fill {
            paint: Paint::Color(Color::black()),
            opacity: Opacity::default(),
            rule: FillRule::default(),
        }
    }
}


/// A stroke style.
#[allow(missing_docs)]
#[derive(Clone, Debug)]
pub struct Stroke {
    pub paint: Paint,
    pub dasharray: Option<Vec<f64>>,
    pub dashoffset: f32, // f32 and not f64 to reduce the struct size.
    pub miterlimit: StrokeMiterlimit,
    pub opacity: Opacity,
    pub width: StrokeWidth,
    pub linecap: LineCap,
    pub linejoin: LineJoin,
}

impl Default for Stroke {
    fn default() -> Self {
        Stroke {
            // The actual default color is `none`,
            // but to simplify the `Stroke` object creation we use `black`.
            paint: Paint::Color(Color::black()),
            dasharray: None,
            dashoffset: 0.0,
            miterlimit: StrokeMiterlimit::default(),
            opacity: Opacity::default(),
            width: StrokeWidth::default(),
            linecap: LineCap::default(),
            linejoin: LineJoin::default(),
        }
    }
}


/// A font description.
#[allow(missing_docs)]
#[derive(Clone, Debug)]
pub struct Font {
    /// Font family.
    ///
    /// Currently, is exactly the same as in the `font-family` attribute.
    /// So it can look like `Verdana, 'Times New Roman', sans-serif`.
    pub family: String,
    pub size: FontSize,
    pub style: FontStyle,
    pub variant: FontVariant,
    pub weight: FontWeight,
    pub stretch: FontStretch,

    /// Letter spacing.
    ///
    /// None == `normal`
    pub letter_spacing: Option<f64>,

    /// Word spacing.
    ///
    /// None == `normal`
    pub word_spacing: Option<f64>,
}


/// View box.
#[derive(Clone, Copy, Debug)]
pub struct ViewBox {
    /// Value of the `viewBox` attribute.
    pub rect: Rect,

    /// Value of the `preserveAspectRatio` attribute.
    pub aspect: AspectRatio,
}


/// A path absolute segment.
///
/// Unlike the SVG spec, can contain only `M`, `L`, `C` and `Z` segments.
/// All other segments will be converted into this one.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug)]
pub enum PathSegment {
    MoveTo {
        x: f64,
        y: f64,
    },
    LineTo {
        x: f64,
        y: f64,
    },
    CurveTo {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        x: f64,
        y: f64,
    },
    ClosePath,
}


/// Identifies input for a filter primitive.
#[allow(missing_docs)]
#[derive(Clone, PartialEq, Debug)]
pub enum FilterInput {
    SourceGraphic,
    SourceAlpha,
    BackgroundImage,
    BackgroundAlpha,
    FillPaint,
    StrokePaint,
    Reference(String),
}

impl ToString for FilterInput {
    fn to_string(&self) -> String {
        match self {
            FilterInput::SourceGraphic      => "SourceGraphic",
            FilterInput::SourceAlpha        => "SourceAlpha",
            FilterInput::BackgroundImage    => "BackgroundImage",
            FilterInput::BackgroundAlpha    => "BackgroundAlpha",
            FilterInput::FillPaint          => "FillPaint",
            FilterInput::StrokePaint        => "StrokePaint",
            FilterInput::Reference(ref s)   => s,
        }.to_string()
    }
}


/// A color interpolation mode.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ColorInterpolation {
    SRGB,
    LinearRGB,
}

impl ToString for ColorInterpolation {
    fn to_string(&self) -> String {
        match self {
            ColorInterpolation::SRGB        => "sRGB",
            ColorInterpolation::LinearRGB   => "linearRGB",
        }.to_string()
    }
}


/// A raster image container.
#[derive(Clone, Debug)]
pub enum ImageData {
    /// Path to a PNG, JPEG or SVG(Z) image.
    ///
    /// Preprocessor will check that the file exist, but because it can be removed later,
    /// so there is no guarantee that this path is valid.
    ///
    /// The path may be relative.
    Path(PathBuf),

    /// Image raw data.
    ///
    /// It's not a decoded image data, but the data that was decoded from base64.
    /// So you still need a PNG, JPEG and SVG(Z) decoding libraries.
    Raw(Vec<u8>),
}


/// An image codec.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ImageFormat {
    PNG,
    JPEG,
    SVG,
}


/// An images blending mode.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FeBlendMode {
    Normal,
    Multiply,
    Screen,
    Darken,
    Lighten,
}

impl ToString for FeBlendMode {
    fn to_string(&self) -> String {
        match self {
            FeBlendMode::Normal     => "normal",
            FeBlendMode::Multiply   => "multiply",
            FeBlendMode::Screen     => "screen",
            FeBlendMode::Darken     => "darken",
            FeBlendMode::Lighten    => "lighten",
        }.to_string()
    }
}


/// An images compositing operation.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FeCompositeOperator {
    Over,
    In,
    Out,
    Atop,
    Xor,
    Arithmetic,
}

impl ToString for FeCompositeOperator {
    fn to_string(&self) -> String {
        match self {
            FeCompositeOperator::Over       => "over",
            FeCompositeOperator::In         => "in",
            FeCompositeOperator::Out        => "out",
            FeCompositeOperator::Atop       => "atop",
            FeCompositeOperator::Xor        => "xor",
            FeCompositeOperator::Arithmetic => "arithmetic",
        }.to_string()
    }
}


/// Kind of the `feImage` data.
#[derive(Clone, Debug)]
pub enum FeImageKind {
    /// Empty image.
    ///
    /// Unlike the `image` element, `feImage` can be without the `href` attribute.
    /// In this case the filter primitive is an empty canvas.
    /// And we can't remove it, because its `result` can be used.
    None,

    /// An image data.
    Image(ImageData, ImageFormat),

    /// A reference to an SVG object.
    ///
    /// `feImage` can reference any SVG object, just like `use` element.
    /// But we can't resolve `use` in this case.
    ///
    /// Not supported yet.
    Use(String),
}

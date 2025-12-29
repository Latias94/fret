use std::sync::Arc;

use fret_components_ui::declarative::style as decl_style;
use fret_components_ui::recipes::input::{InputTokenKeys, resolve_input_chrome};
use fret_components_ui::{ChromeRefinement, LayoutRefinement, Size as ComponentSize};
use fret_core::{Color, Corners, Edges, FontId, Px, TextStyle};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, Length, SizeStyle, TextAreaProps};
use fret_ui::{ElementCx, TextAreaStyle, Theme, UiHost};

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

#[derive(Clone)]
pub struct Textarea {
    model: Model<String>,
    a11y_label: Option<Arc<str>>,
    min_height: Px,
    size: ComponentSize,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for Textarea {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Textarea")
            .field("model", &"<model>")
            .field("a11y_label", &self.a11y_label.as_ref().map(|s| s.as_ref()))
            .field("min_height", &self.min_height)
            .field("size", &self.size)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl Textarea {
    pub fn new(model: Model<String>) -> Self {
        Self {
            model,
            a11y_label: None,
            min_height: Px(80.0),
            size: ComponentSize::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn min_height(mut self, min_height: Px) -> Self {
        self.min_height = min_height;
        self
    }

    pub fn size(mut self, size: ComponentSize) -> Self {
        self.size = size;
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        textarea(
            cx,
            self.model,
            self.a11y_label,
            self.min_height,
            self.size,
            self.chrome,
            self.layout,
        )
    }
}

pub fn textarea<H: UiHost>(
    cx: &mut ElementCx<'_, H>,
    model: Model<String>,
    a11y_label: Option<Arc<str>>,
    min_height: Px,
    size: ComponentSize,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();

    let resolved = resolve_input_chrome(&theme, size, &chrome, InputTokenKeys::none());

    let font_line_height = theme
        .metric_by_key("font.line_height")
        .unwrap_or(theme.metrics.font_line_height);
    let text_style = TextStyle {
        font: FontId::default(),
        size: resolved.text_px,
        line_height: Some(font_line_height),
        ..Default::default()
    };

    let mut chrome = TextAreaStyle::default();
    chrome.padding_x = resolved.padding.left;
    chrome.padding_y = resolved.padding.top;
    chrome.background = resolved.background;
    chrome.border = Edges::all(resolved.border_width);
    chrome.border_color = resolved.border_color;
    chrome.corner_radii = Corners::all(resolved.radius);
    chrome.text_color = resolved.text_color;
    chrome.selection_color = alpha_mul(resolved.selection_color, 0.65);
    chrome.caret_color = resolved.text_color;
    chrome.preedit_bg_color = alpha_mul(resolved.selection_color, 0.22);
    chrome.preedit_underline_color = theme.colors.accent;
    chrome.focus_ring = Some(decl_style::focus_ring(&theme, resolved.radius));

    let root_layout = decl_style::layout_style(&theme, layout.relative().w_full());

    let mut props = TextAreaProps::new(model);
    props.a11y_label = a11y_label;
    props.chrome = chrome;
    props.text_style = text_style;
    props.min_height = min_height;
    props.layout = root_layout;
    props.layout.size = SizeStyle {
        width: Length::Fill,
        height: Length::Auto,
        min_height: Some(min_height),
        ..Default::default()
    };

    cx.text_area(props)
}

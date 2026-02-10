use std::sync::Arc;

use fret_core::{Color, Corners, Edges, FontId, Px, TextStyle};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, Length, SizeStyle, TextAreaProps};
use fret_ui::{ElementContext, TextAreaStyle, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::recipes::input::{InputTokenKeys, resolve_input_chrome};
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, Size as ComponentSize};

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

#[derive(Clone)]
pub struct Textarea {
    model: Model<String>,
    a11y_label: Option<Arc<str>>,
    aria_invalid: bool,
    disabled: bool,
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
            aria_invalid: false,
            disabled: false,
            min_height: Px(64.0),
            size: ComponentSize::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn aria_invalid(mut self, aria_invalid: bool) -> Self {
        self.aria_invalid = aria_invalid;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        textarea(
            cx,
            self.model,
            self.a11y_label,
            self.aria_invalid,
            self.disabled,
            self.min_height,
            self.size,
            self.chrome,
            self.layout,
        )
    }
}

pub fn textarea<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<String>,
    a11y_label: Option<Arc<str>>,
    aria_invalid: bool,
    disabled: bool,
    min_height: Px,
    size: ComponentSize,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);

    let resolved = resolve_input_chrome(theme, size, &chrome, InputTokenKeys::none());

    let font_line_height = theme
        .metric_by_key("font.line_height")
        .unwrap_or_else(|| theme.metric_required("font.line_height"));
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
    chrome.preedit_underline_color = resolved.selection_color;
    chrome.focus_ring = Some(decl_style::focus_ring(theme, resolved.radius));

    if aria_invalid {
        let border_color = theme.color_required("destructive");
        chrome.border_color = border_color;
        if let Some(mut ring) = chrome.focus_ring.take() {
            let ring_key = if theme.name.contains("/dark") {
                "destructive/40"
            } else {
                "destructive/20"
            };
            ring.color = theme
                .color_by_key(ring_key)
                .or_else(|| theme.color_by_key("destructive/20"))
                .unwrap_or(border_color);
            chrome.focus_ring = Some(ring);
        }
    }

    let root_layout = decl_style::layout_style(theme, layout.relative().w_full());

    let mut props = TextAreaProps::new(model);
    props.enabled = !disabled;
    props.focusable = !disabled;
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

    if disabled {
        cx.opacity(0.5, move |cx| vec![cx.text_area(props)])
    } else {
        cx.text_area(props)
    }
}

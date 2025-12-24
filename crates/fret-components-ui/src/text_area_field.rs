use fret_core::{Corners, Edges, FontId, Px, TextStyle};
use fret_runtime::Model;
use fret_ui::{BoundTextArea, TextAreaStyle, Theme, UiHost, Widget};

use crate::style::StyleRefinement;
use crate::{Sizable, Size};

pub struct TextAreaField {
    inner: BoundTextArea,
    size: Size,
    style: StyleRefinement,
    min_height: Px,
    last_theme_revision: Option<u64>,
}

impl TextAreaField {
    pub fn new(model: Model<String>) -> Self {
        Self {
            inner: BoundTextArea::new(model),
            size: Size::Medium,
            style: StyleRefinement::default(),
            min_height: Px(0.0),
            last_theme_revision: None,
        }
    }

    pub fn with_size(mut self, size: Size) -> Self {
        self.size = size;
        self.last_theme_revision = None;
        self
    }

    pub fn refine_style(mut self, style: StyleRefinement) -> Self {
        self.style = style;
        self.last_theme_revision = None;
        self
    }

    pub fn with_min_height(mut self, min_height: Px) -> Self {
        self.min_height = min_height;
        self
    }

    fn sync_chrome(&mut self, theme: &Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        let mut chrome = TextAreaStyle {
            padding_x: self.size.input_px(theme),
            padding_y: self.size.input_py(theme),
            background: theme.colors.panel_background,
            border: Edges::all(Px(1.0)),
            border_color: theme.colors.panel_border,
            corner_radii: Corners::all(self.size.control_radius(theme)),
            text_color: theme.colors.text_primary,
            selection_color: theme.colors.selection_background,
            caret_color: theme.colors.text_primary,
            preedit_bg_color: fret_core::Color {
                a: 0.22,
                ..theme.colors.selection_background
            },
            preedit_underline_color: theme.colors.accent,
        };

        if let Some(px) = theme.metric_by_key("component.text_area.padding_x") {
            chrome.padding_x = px;
        }
        if let Some(px) = theme.metric_by_key("component.text_area.padding_y") {
            chrome.padding_y = px;
        }
        if let Some(px) = theme.metric_by_key("component.text_area.radius") {
            chrome.corner_radii = Corners::all(px);
        }
        if let Some(px) = theme.metric_by_key("component.text_area.border_width") {
            chrome.border = Edges::all(px);
        }
        if let Some(bg) = theme.color_by_key("component.text_area.bg") {
            chrome.background = bg;
        }
        if let Some(c) = theme.color_by_key("component.text_area.border") {
            chrome.border_color = c;
        }
        if let Some(c) = theme.color_by_key("component.text_area.fg") {
            chrome.text_color = c;
            chrome.caret_color = c;
        }
        if let Some(c) = theme.color_by_key("component.text_area.selection") {
            chrome.selection_color = c;
        }

        if let Some(padding_x) = self.style.padding_x.clone() {
            chrome.padding_x = padding_x.resolve(theme);
        }
        if let Some(padding_y) = self.style.padding_y.clone() {
            chrome.padding_y = padding_y.resolve(theme);
        }
        if let Some(radius) = self.style.radius.clone() {
            chrome.corner_radii = Corners::all(radius.resolve(theme));
        }
        if let Some(border_width) = self.style.border_width.clone() {
            chrome.border = Edges::all(border_width.resolve(theme));
        }
        if let Some(bg) = self.style.background.clone() {
            chrome.background = bg.resolve(theme);
        }
        if let Some(c) = self.style.border_color.clone() {
            chrome.border_color = c.resolve(theme);
        }
        if let Some(c) = self.style.text_color.clone() {
            let c = c.resolve(theme);
            chrome.text_color = c;
            chrome.caret_color = c;
        }

        let text_px = theme
            .metric_by_key("component.text_area.text_px")
            .unwrap_or_else(|| self.size.control_text_px(theme));
        self.inner.set_text_style(TextStyle {
            font: FontId::default(),
            size: text_px,
        });

        chrome.padding_x = Px(chrome.padding_x.0.max(0.0));
        chrome.padding_y = Px(chrome.padding_y.0.max(0.0));
        self.inner.set_min_height(self.min_height);
        self.inner.set_style(chrome);
    }
}

impl Sizable for TextAreaField {
    fn with_size(self, size: Size) -> Self {
        TextAreaField::with_size(self, size)
    }
}

impl<H: UiHost> Widget<H> for TextAreaField {
    fn is_focusable(&self) -> bool {
        true
    }

    fn is_text_input(&self) -> bool {
        true
    }

    fn event(&mut self, cx: &mut fret_ui::EventCx<'_, H>, event: &fret_core::Event) {
        self.sync_chrome(cx.theme());
        self.inner.event(cx, event);
    }

    fn layout(&mut self, cx: &mut fret_ui::LayoutCx<'_, H>) -> fret_core::Size {
        self.sync_chrome(cx.theme());
        self.inner.layout(cx)
    }

    fn paint(&mut self, cx: &mut fret_ui::PaintCx<'_, H>) {
        self.sync_chrome(cx.theme());
        self.inner.paint(cx);
    }

    fn semantics(&mut self, cx: &mut fret_ui::widget::SemanticsCx<'_, H>) {
        self.inner.semantics(cx);
    }
}

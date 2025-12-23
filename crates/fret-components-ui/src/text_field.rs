use fret_core::Px;
use fret_runtime::Model;
use fret_ui::{BoundTextInput, TextInputStyle, Theme, UiHost, Widget};

use crate::style::StyleRefinement;

pub struct TextField {
    inner: BoundTextInput,
    style: StyleRefinement,
    last_theme_revision: Option<u64>,
}

impl TextField {
    pub fn new(model: Model<String>) -> Self {
        Self {
            inner: BoundTextInput::new(model),
            style: StyleRefinement::default(),
            last_theme_revision: None,
        }
    }

    pub fn refine_style(mut self, style: StyleRefinement) -> Self {
        self.style = style;
        self.last_theme_revision = None;
        self
    }

    fn sync_chrome(&mut self, theme: &Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        let snap = theme.snapshot();
        let mut chrome = TextInputStyle::from_theme(snap);

        // Component namespace defaults (best-effort).
        if let Some(px) = theme.metric_by_key("component.text_field.padding_x") {
            chrome.padding_x = px;
        }
        if let Some(px) = theme.metric_by_key("component.text_field.padding_y") {
            chrome.padding_y = px;
        }
        if let Some(px) = theme.metric_by_key("component.text_field.radius") {
            chrome.corner_radii = fret_core::geometry::Corners::all(px);
        }
        if let Some(bg) = theme.color_by_key("component.text_field.bg") {
            chrome.background = bg;
        }
        if let Some(c) = theme.color_by_key("component.text_field.border") {
            chrome.border_color = c;
        }
        if let Some(c) = theme.color_by_key("component.text_field.border_focus") {
            chrome.border_color_focused = c;
        }
        if let Some(c) = theme.color_by_key("component.text_field.fg") {
            chrome.text_color = c;
            chrome.caret_color = c;
        }
        if let Some(c) = theme.color_by_key("component.text_field.selection") {
            chrome.selection_color = c;
        }

        // Tailwind-like typed refinements override tokens.
        if let Some(padding_x) = self.style.padding_x.clone() {
            chrome.padding_x = padding_x.resolve(theme);
        }
        if let Some(padding_y) = self.style.padding_y.clone() {
            chrome.padding_y = padding_y.resolve(theme);
        }
        if let Some(radius) = self.style.radius.clone() {
            chrome.corner_radii = fret_core::geometry::Corners::all(radius.resolve(theme));
        }
        if let Some(border_width) = self.style.border_width.clone() {
            chrome.border = fret_core::geometry::Edges::all(border_width.resolve(theme));
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

        // Keep a small minimum height so the field is usable even with empty text.
        chrome.padding_x = Px(chrome.padding_x.0.max(0.0));
        chrome.padding_y = Px(chrome.padding_y.0.max(0.0));

        self.inner.set_chrome_style(chrome);
    }
}

impl<H: UiHost> Widget<H> for TextField {
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

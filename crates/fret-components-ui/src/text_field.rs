use fret_core::{FontId, Px, TextStyle};
use fret_runtime::CommandId;
use fret_runtime::Model;
use fret_ui::{BoundTextInput, TextInputStyle, Theme, UiHost, Widget};

use crate::style::StyleRefinement;
use crate::{Sizable, Size};

pub struct TextField {
    inner: BoundTextInput,
    size: Size,
    style: StyleRefinement,
    last_theme_revision: Option<u64>,
}

impl TextField {
    pub fn new(model: Model<String>) -> Self {
        Self {
            inner: BoundTextInput::new(model),
            size: Size::Medium,
            style: StyleRefinement::default(),
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

    pub fn with_submit_command(mut self, command: CommandId) -> Self {
        self.inner.set_submit_command(Some(command));
        self
    }

    pub fn with_cancel_command(mut self, command: CommandId) -> Self {
        self.inner.set_cancel_command(Some(command));
        self
    }

    fn sync_chrome(&mut self, theme: &Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        let snap = theme.snapshot();
        let mut chrome = TextInputStyle::from_theme(snap);

        chrome.padding_x = self.size.input_px(theme);
        chrome.padding_y = self.size.input_py(theme);
        chrome.corner_radii = fret_core::geometry::Corners::all(self.size.control_radius(theme));

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

        let text_px = theme
            .metric_by_key("component.text_field.text_px")
            .unwrap_or_else(|| self.size.control_text_px(theme));
        self.inner.set_text_style(TextStyle {
            font: FontId::default(),
            size: text_px,
        });

        // Keep a small minimum height so the field is usable even with empty text.
        chrome.padding_x = Px(chrome.padding_x.0.max(0.0));
        chrome.padding_y = Px(chrome.padding_y.0.max(0.0));

        self.inner.set_chrome_style(chrome);
    }
}

impl Sizable for TextField {
    fn with_size(self, size: Size) -> Self {
        TextField::with_size(self, size)
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
        let inner = self.inner.layout(cx);
        let min_h = self.size.input_h(cx.theme()).0.max(0.0);
        let h = inner.height.0.max(min_h).min(cx.available.height.0);
        fret_core::Size::new(inner.width, Px(h))
    }

    fn paint(&mut self, cx: &mut fret_ui::PaintCx<'_, H>) {
        self.sync_chrome(cx.theme());
        self.inner.paint(cx);
    }

    fn semantics(&mut self, cx: &mut fret_ui::widget::SemanticsCx<'_, H>) {
        self.inner.semantics(cx);
    }
}

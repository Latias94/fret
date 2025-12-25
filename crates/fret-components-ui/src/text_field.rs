use fret_core::{FontId, Px, TextStyle};
use fret_runtime::CommandId;
use fret_runtime::Model;
use fret_ui::primitives::{BoundTextInput, TextInputStyle};
use fret_ui::{Theme, UiHost, Widget};

use crate::recipes::input::{InputTokenKeys, resolve_input_chrome};
use crate::style::StyleRefinement;
use crate::{Sizable, Size};

pub struct TextField {
    inner: BoundTextInput,
    size: Size,
    style: StyleRefinement,
    min_height: Px,
    last_theme_revision: Option<u64>,
}

impl TextField {
    pub fn new(model: Model<String>) -> Self {
        Self {
            inner: BoundTextInput::new(model),
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

        let resolved = resolve_input_chrome(
            theme,
            self.size,
            &self.style,
            InputTokenKeys {
                padding_x: Some("component.text_field.padding_x"),
                padding_y: Some("component.text_field.padding_y"),
                min_height: Some("component.text_field.min_height"),
                radius: Some("component.text_field.radius"),
                border_width: Some("component.text_field.border_width"),
                bg: Some("component.text_field.bg"),
                border: Some("component.text_field.border"),
                border_focus: Some("component.text_field.border_focus"),
                fg: Some("component.text_field.fg"),
                text_px: Some("component.text_field.text_px"),
                selection: Some("component.text_field.selection"),
            },
        );

        let snap = theme.snapshot();
        let mut chrome = TextInputStyle::from_theme(snap);

        chrome.padding_x = resolved.padding_x;
        chrome.padding_y = resolved.padding_y;
        chrome.corner_radii = fret_core::geometry::Corners::all(resolved.radius);
        chrome.border = fret_core::geometry::Edges::all(resolved.border_width);
        chrome.background = resolved.background;
        chrome.border_color = resolved.border_color;
        chrome.border_color_focused = resolved.border_color_focused;
        chrome.text_color = resolved.text_color;
        chrome.caret_color = resolved.text_color;
        chrome.selection_color = resolved.selection_color;

        let text_px = resolved.text_px;
        self.inner.set_text_style(TextStyle {
            font: FontId::default(),
            size: text_px,
        });

        // Keep a small minimum height so the field is usable even with empty text.
        self.min_height = resolved.min_height;

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
        let min_h = self.min_height.0.max(0.0);
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

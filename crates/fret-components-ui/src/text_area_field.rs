use fret_core::{Corners, Edges, FontId, Px, TextStyle};
use fret_runtime::Model;
use fret_ui::primitives::{BoundTextArea, TextAreaStyle};
use fret_ui::{Theme, UiHost, Widget};

use crate::recipes::input::{InputTokenKeys, resolve_input_chrome};
use crate::style::ChromeRefinement;
use crate::{Sizable, Size};

pub struct TextAreaField {
    inner: BoundTextArea,
    size: Size,
    style: ChromeRefinement,
    min_height: Px,
    last_theme_revision: Option<u64>,
}

impl TextAreaField {
    pub fn new(model: Model<String>) -> Self {
        Self {
            inner: BoundTextArea::new(model),
            size: Size::Medium,
            style: ChromeRefinement::default(),
            min_height: Px(0.0),
            last_theme_revision: None,
        }
    }

    pub fn with_size(mut self, size: Size) -> Self {
        self.size = size;
        self.last_theme_revision = None;
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
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

        let resolved = resolve_input_chrome(
            theme,
            self.size,
            &self.style,
            InputTokenKeys {
                padding_x: Some("component.text_area.padding_x"),
                padding_y: Some("component.text_area.padding_y"),
                min_height: Some("component.text_area.min_height"),
                radius: Some("component.text_area.radius"),
                border_width: Some("component.text_area.border_width"),
                bg: Some("component.text_area.bg"),
                border: Some("component.text_area.border"),
                border_focus: Some("component.text_area.border_focus"),
                fg: Some("component.text_area.fg"),
                text_px: Some("component.text_area.text_px"),
                selection: Some("component.text_area.selection"),
            },
        );

        let chrome = TextAreaStyle {
            padding_x: resolved.padding.left,
            padding_y: resolved.padding.top,
            background: resolved.background,
            border: Edges::all(resolved.border_width),
            border_color: resolved.border_color,
            focus_ring: Some(fret_ui::element::RingStyle {
                placement: fret_ui::element::RingPlacement::Outset,
                width: theme
                    .metric_by_key("component.ring.width")
                    .unwrap_or(Px(2.0)),
                offset: theme
                    .metric_by_key("component.ring.offset")
                    .unwrap_or(Px(2.0)),
                color: resolved.border_color_focused,
                offset_color: Some(
                    theme
                        .color_by_key("ring-offset-background")
                        .unwrap_or(theme.colors.surface_background),
                ),
                corner_radii: Corners::all(resolved.radius),
            }),
            corner_radii: Corners::all(resolved.radius),
            text_color: resolved.text_color,
            selection_color: resolved.selection_color,
            caret_color: resolved.text_color,
            preedit_bg_color: fret_core::Color {
                a: 0.22,
                ..resolved.selection_color
            },
            preedit_underline_color: theme.colors.accent,
        };

        let text_px = resolved.text_px;
        self.inner.set_text_style(TextStyle {
            font: FontId::default(),
            size: text_px,
            ..Default::default()
        });

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

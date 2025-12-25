use fret_components_icons::{IconGlyph, IconId, IconRegistry};
use fret_components_ui::recipes::input::{
    InputTokenKeys, default_text_input_style, resolve_input_chrome,
};
use fret_components_ui::{ChromeRefinement, MetricRef, Size as ComponentSize, Space};
use fret_core::{
    Color, DrawOrder, Event, Point, Px, Rect, SceneOp, Size, TextConstraints, TextMetrics,
    TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::Model;
use fret_ui::primitives::{BoundTextInput, TextInputStyle};
use fret_ui::{EventCx, LayoutCx, PaintCx, Theme, UiHost, Widget};

#[derive(Debug, Clone)]
struct PreparedIcon {
    icon: IconId,
    blob: fret_core::TextBlobId,
    metrics: TextMetrics,
    scale_factor_bits: u32,
    theme_revision: u64,
}

pub struct InputGroup {
    inner: BoundTextInput,
    disabled: bool,
    size: ComponentSize,
    style: ChromeRefinement,
    leading_icon: Option<IconId>,
    trailing_icon: Option<IconId>,
    min_height: Px,
    icon_size: Px,
    icon_gap: Px,
    icon_inset_left: Px,
    icon_inset_right: Px,
    prepared_leading: Option<PreparedIcon>,
    prepared_trailing: Option<PreparedIcon>,
    last_theme_revision: Option<u64>,
}

impl InputGroup {
    pub fn new(model: Model<String>) -> Self {
        Self {
            inner: BoundTextInput::new(model),
            disabled: false,
            size: ComponentSize::Medium,
            style: ChromeRefinement::default(),
            leading_icon: None,
            trailing_icon: None,
            min_height: Px(0.0),
            icon_size: Px(16.0),
            icon_gap: Px(8.0),
            icon_inset_left: Px(0.0),
            icon_inset_right: Px(0.0),
            prepared_leading: None,
            prepared_trailing: None,
            last_theme_revision: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn with_size(mut self, size: ComponentSize) -> Self {
        self.size = size;
        self.last_theme_revision = None;
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.style = self.style.merge(style);
        self.last_theme_revision = None;
        self
    }

    pub fn leading_icon(mut self, icon: IconId) -> Self {
        self.leading_icon = Some(icon);
        self.last_theme_revision = None;
        self
    }

    pub fn trailing_icon(mut self, icon: IconId) -> Self {
        self.trailing_icon = Some(icon);
        self.last_theme_revision = None;
        self
    }

    fn sync_chrome(&mut self, theme: &Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        let resolved = resolve_input_chrome(theme, self.size, &self.style, InputTokenKeys::none());

        let icon_size = theme
            .metric_by_key("component.input_group.icon_px")
            .unwrap_or(Px(16.0));
        let icon_gap = theme
            .metric_by_key("component.input_group.icon_gap")
            .unwrap_or_else(|| MetricRef::space(Space::N2).resolve(theme));

        let base_left = resolved.padding.left;
        let base_right = resolved.padding.right;

        let mut padding = resolved.padding;
        if self.leading_icon.is_some() {
            padding.left = Px((padding.left.0 + icon_size.0 + icon_gap.0).max(0.0));
        }
        if self.trailing_icon.is_some() {
            padding.right = Px((padding.right.0 + icon_size.0 + icon_gap.0).max(0.0));
        }

        let mut chrome: TextInputStyle = default_text_input_style(theme);
        chrome.padding = padding;
        chrome.corner_radii = fret_core::Corners::all(resolved.radius);
        chrome.border = fret_core::Edges::all(resolved.border_width);
        chrome.background = resolved.background;
        chrome.border_color = resolved.border_color;
        chrome.border_color_focused = resolved.border_color_focused;
        chrome.text_color = resolved.text_color;
        chrome.caret_color = resolved.text_color;
        chrome.selection_color = Color {
            a: 1.0,
            ..resolved.selection_color
        };

        self.inner.set_chrome_style(chrome);
        self.inner.set_text_style(TextStyle {
            size: resolved.text_px,
            ..Default::default()
        });

        self.min_height = resolved.min_height;
        self.icon_size = icon_size;
        self.icon_gap = icon_gap;
        self.icon_inset_left = base_left;
        self.icon_inset_right = base_right;
    }

    fn prepare_icon<H: UiHost>(
        &mut self,
        cx: &mut LayoutCx<'_, H>,
        which: IconSlot,
        icon: &IconId,
    ) {
        let theme_rev = cx.theme().revision();
        let scale_bits = cx.scale_factor.to_bits();

        let slot_ref = match which {
            IconSlot::Leading => &mut self.prepared_leading,
            IconSlot::Trailing => &mut self.prepared_trailing,
        };

        if let Some(p) = slot_ref.as_ref()
            && p.theme_revision == theme_rev
            && p.scale_factor_bits == scale_bits
            && &p.icon == icon
        {
            return;
        }

        if let Some(p) = slot_ref.take() {
            cx.text.release(p.blob);
        }

        let glyph: IconGlyph = cx
            .app
            .with_global_mut(IconRegistry::default, |icons, _app| {
                icons.ensure_builtin_glyphs();
                icons
                    .glyph(icon)
                    .cloned()
                    .unwrap_or_else(|| IconGlyph::new("?"))
            });

        let size = self.icon_size;
        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };
        let style = TextStyle {
            font: glyph.font,
            size,
            line_height: Some(size),
            ..Default::default()
        };
        let (blob, metrics) = cx.text.prepare(glyph.text.as_ref(), style, constraints);

        *slot_ref = Some(PreparedIcon {
            icon: icon.clone(),
            blob,
            metrics,
            scale_factor_bits: scale_bits,
            theme_revision: theme_rev,
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IconSlot {
    Leading,
    Trailing,
}

impl<H: UiHost> Widget<H> for InputGroup {
    fn cleanup_resources(&mut self, text: &mut dyn fret_core::TextService) {
        self.inner.cleanup_resources(text);
        if let Some(p) = self.prepared_leading.take() {
            text.release(p.blob);
        }
        if let Some(p) = self.prepared_trailing.take() {
            text.release(p.blob);
        }
    }

    fn is_focusable(&self) -> bool {
        !self.disabled
    }

    fn is_text_input(&self) -> bool {
        !self.disabled
    }

    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        !self.disabled
    }

    fn hit_test_children(&self, _bounds: Rect, _position: Point) -> bool {
        !self.disabled
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        self.sync_chrome(cx.theme());

        if self.disabled {
            return;
        }

        self.inner.event(cx, event);
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.sync_chrome(cx.theme());

        if let Some(icon) = self.leading_icon.clone() {
            self.prepare_icon(cx, IconSlot::Leading, &icon);
        }
        if let Some(icon) = self.trailing_icon.clone() {
            self.prepare_icon(cx, IconSlot::Trailing, &icon);
        }

        let inner = self.inner.layout(cx);
        let min_h = self.min_height.0.max(0.0);
        let h = inner.height.0.max(min_h).min(cx.available.height.0);
        Size::new(inner.width, Px(h))
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_chrome(cx.theme());

        self.inner.paint(cx);

        let theme = cx.theme();
        let mut icon_color = theme
            .color_by_key("muted.foreground")
            .or_else(|| theme.color_by_key("muted-foreground"))
            .unwrap_or(theme.colors.text_muted);
        if self.disabled {
            icon_color = theme.colors.text_disabled;
            icon_color.a *= 0.5;
        }

        let icon_size = self.icon_size.0.max(0.0);
        if icon_size <= 0.0 {
            return;
        }

        let bounds = cx.bounds;
        let center_y = bounds.origin.y.0 + (bounds.size.height.0 - icon_size) * 0.5;

        if let Some(icon) = self.leading_icon.as_ref()
            && let Some(p) = self.prepared_leading.as_ref().filter(|p| &p.icon == icon)
        {
            let x = bounds.origin.x.0 + self.icon_inset_left.0.max(0.0);
            let top = center_y + ((icon_size - p.metrics.size.height.0) * 0.5).max(0.0);
            let y = top + p.metrics.baseline.0;
            cx.scene.push(SceneOp::Text {
                order: DrawOrder(10),
                origin: Point::new(Px(x), Px(y)),
                text: p.blob,
                color: icon_color,
            });
        }

        if let Some(icon) = self.trailing_icon.as_ref()
            && let Some(p) = self.prepared_trailing.as_ref().filter(|p| &p.icon == icon)
        {
            let right = bounds.origin.x.0 + bounds.size.width.0;
            let x = right - self.icon_inset_right.0.max(0.0) - icon_size;
            let top = center_y + ((icon_size - p.metrics.size.height.0) * 0.5).max(0.0);
            let y = top + p.metrics.baseline.0;
            cx.scene.push(SceneOp::Text {
                order: DrawOrder(10),
                origin: Point::new(Px(x), Px(y)),
                text: p.blob,
                color: icon_color,
            });
        }
    }

    fn semantics(&mut self, cx: &mut fret_ui::widget::SemanticsCx<'_, H>) {
        self.inner.semantics(cx);
        cx.set_disabled(self.disabled);
    }
}

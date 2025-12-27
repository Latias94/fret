use fret_components_icons::{IconId, IconRegistry, MISSING_ICON_SVG, ResolvedSvgOwned};
use fret_components_ui::recipes::input::{
    InputTokenKeys, default_text_input_style, resolve_input_chrome,
};
use fret_components_ui::{ChromeRefinement, MetricRef, Size as ComponentSize, Space};
use fret_core::{Color, DrawOrder, Event, Px, Rect, SceneOp, Size, SvgFit};
use fret_runtime::Model;
use fret_ui::{BoundTextInput, TextInputStyle};
use fret_ui::{EventCx, LayoutCx, PaintCx, Theme, UiHost, Widget};

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
        self.inner.set_text_style(fret_core::TextStyle {
            size: resolved.text_px,
            ..Default::default()
        });

        self.min_height = resolved.min_height;
        self.icon_size = icon_size;
        self.icon_gap = icon_gap;
        self.icon_inset_left = base_left;
        self.icon_inset_right = base_right;
    }

    fn register_icon_svg<H: UiHost>(
        &self,
        cx: &mut PaintCx<'_, H>,
        icon: &IconId,
    ) -> fret_core::SvgId {
        let resolved = cx
            .app
            .with_global_mut(IconRegistry::default, |icons, _app| {
                icons.resolve_svg_owned(icon)
            });

        match resolved {
            Some(ResolvedSvgOwned::Static(bytes)) => cx.services.svg().register_svg(bytes),
            Some(ResolvedSvgOwned::Bytes(bytes)) => cx.services.svg().register_svg(bytes.as_ref()),
            None => cx.services.svg().register_svg(MISSING_ICON_SVG),
        }
    }
}

impl<H: UiHost> Widget<H> for InputGroup {
    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        self.inner.cleanup_resources(services);
    }

    fn is_focusable(&self) -> bool {
        !self.disabled
    }

    fn is_text_input(&self) -> bool {
        !self.disabled
    }

    fn hit_test(&self, _bounds: Rect, _position: fret_core::Point) -> bool {
        !self.disabled
    }

    fn hit_test_children(&self, _bounds: Rect, _position: fret_core::Point) -> bool {
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

        let inner = self.inner.layout(cx);
        let min_h = self.min_height.0.max(0.0);
        let h = inner.height.0.max(min_h);
        Size::new(inner.width, Px(h))
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_chrome(cx.theme());
        self.inner.paint(cx);

        let theme = cx.theme();
        let icon_color = if self.disabled {
            theme.colors.text_disabled
        } else {
            theme
                .color_by_key("muted-foreground")
                .unwrap_or(theme.colors.text_muted)
        };

        let y =
            cx.bounds.origin.y.0 + ((cx.bounds.size.height.0 - self.icon_size.0) * 0.5).max(0.0);

        if let Some(icon) = self.leading_icon.clone() {
            let svg = self.register_icon_svg(cx, &icon);
            let x = cx.bounds.origin.x.0 + self.icon_inset_left.0;
            let rect = Rect::new(
                fret_core::Point::new(Px(x), Px(y)),
                Size::new(self.icon_size, self.icon_size),
            );
            cx.scene.push(SceneOp::SvgMaskIcon {
                order: DrawOrder(10),
                rect,
                svg,
                fit: SvgFit::Contain,
                color: icon_color,
                opacity: 1.0,
            });
        }

        if let Some(icon) = self.trailing_icon.clone() {
            let svg = self.register_icon_svg(cx, &icon);
            let x = cx.bounds.origin.x.0
                + (cx.bounds.size.width.0 - self.icon_inset_right.0 - self.icon_size.0).max(0.0);
            let rect = Rect::new(
                fret_core::Point::new(Px(x), Px(y)),
                Size::new(self.icon_size, self.icon_size),
            );
            cx.scene.push(SceneOp::SvgMaskIcon {
                order: DrawOrder(10),
                rect,
                svg,
                fit: SvgFit::Contain,
                color: icon_color,
                opacity: 1.0,
            });
        }
    }
}

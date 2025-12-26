use std::sync::Arc;

use fret_core::{
    Color, Corners, CursorIcon, DrawOrder, Edges, Event, KeyCode, MouseButton, Point, Px, Rect,
    SceneOp, SemanticsRole, Size, TextConstraints, TextMetrics, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::Model;
use fret_ui::{EventCx, Invalidation, LayoutCx, PaintCx, Theme, UiHost, Widget};

use crate::style::{ColorFallback, MetricFallback, component_color, component_metric};
use crate::{Sizable, Size as ComponentSize};

#[derive(Debug, Clone)]
struct PreparedTab {
    label: Arc<str>,
    blob: Option<fret_core::TextBlobId>,
    metrics: TextMetrics,
    rect: Rect,
}

#[derive(Debug, Clone)]
struct ResolvedTabsStyle {
    height: Px,
    padding_x: Px,
    gap: Px,
    radius: Px,
    border_width: Px,
    text_size: Px,
    bg: Color,
    border: Color,
    tab_bg_hover: Color,
    tab_bg_active: Color,
    fg: Color,
    fg_muted: Color,
}

impl Default for ResolvedTabsStyle {
    fn default() -> Self {
        Self {
            height: Px(32.0),
            padding_x: Px(10.0),
            gap: Px(6.0),
            radius: Px(8.0),
            border_width: Px(1.0),
            text_size: Px(13.0),
            bg: Color::TRANSPARENT,
            border: Color::TRANSPARENT,
            tab_bg_hover: Color::TRANSPARENT,
            tab_bg_active: Color::TRANSPARENT,
            fg: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            fg_muted: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.8,
            },
        }
    }
}

pub struct Tabs {
    model: Model<usize>,
    tabs: Vec<Arc<str>>,
    disabled: bool,
    size: ComponentSize,

    hovered: Option<usize>,
    pressed: Option<usize>,
    last_bounds: Rect,
    last_theme_revision: Option<u64>,
    prepared: Vec<PreparedTab>,
    prepared_scale_factor_bits: Option<u32>,
    prepared_theme_revision: Option<u64>,
    resolved: ResolvedTabsStyle,
}

impl Tabs {
    pub fn new<T>(model: Model<usize>, tabs: Vec<T>) -> Self
    where
        T: Into<Arc<str>>,
    {
        Self {
            model,
            tabs: tabs.into_iter().map(Into::into).collect(),
            disabled: false,
            size: ComponentSize::Medium,
            hovered: None,
            pressed: None,
            last_bounds: Rect::default(),
            last_theme_revision: None,
            prepared: Vec::new(),
            prepared_scale_factor_bits: None,
            prepared_theme_revision: None,
            resolved: ResolvedTabsStyle::default(),
        }
    }

    pub fn with_size(mut self, size: ComponentSize) -> Self {
        self.size = size;
        self.last_theme_revision = None;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    fn selected_index<H: UiHost>(&self, app: &H) -> usize {
        app.models()
            .get(self.model)
            .copied()
            .unwrap_or_default()
            .min(self.tabs.len().saturating_sub(1))
    }

    fn set_selected<H: UiHost>(&self, app: &mut H, index: usize) {
        let index = index.min(self.tabs.len().saturating_sub(1));
        let _ = app.models_mut().update(self.model, |v| *v = index);
    }

    fn translate_prepared(&mut self, delta: Point) {
        if delta.x.0 == 0.0 && delta.y.0 == 0.0 {
            return;
        }
        for tab in &mut self.prepared {
            tab.rect.origin = Point::new(tab.rect.origin.x + delta.x, tab.rect.origin.y + delta.y);
        }
    }

    fn sync_bounds(&mut self, bounds: Rect) {
        let prev = self.last_bounds;
        if prev != Rect::default() && prev.origin != bounds.origin && !self.prepared.is_empty() {
            let delta = Point::new(
                bounds.origin.x - prev.origin.x,
                bounds.origin.y - prev.origin.y,
            );
            self.translate_prepared(delta);
        }
        self.last_bounds = bounds;
    }

    fn sync_style_from_theme(&mut self, theme: &Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        let height = component_metric(
            "component.tabs.height",
            MetricFallback::Px(self.size.button_h(theme)),
        )
        .resolve(theme);
        let padding_x = component_metric(
            "component.tabs.padding_x",
            MetricFallback::Px(self.size.button_px(theme)),
        )
        .resolve(theme);
        let gap = component_metric(
            "component.tabs.gap",
            MetricFallback::Px(Px((self.size.button_px(theme).0 * 0.5).round().max(0.0))),
        )
        .resolve(theme);
        let radius =
            component_metric("component.tabs.radius", MetricFallback::ThemeRadiusMd).resolve(theme);
        let border_width =
            component_metric("component.tabs.border_width", MetricFallback::Px(Px(1.0)))
                .resolve(theme);
        let text_size = theme
            .metric_by_key("component.tabs.text_px")
            .unwrap_or_else(|| self.size.control_text_px(theme));

        let bg = component_color("component.tabs.bg", ColorFallback::ThemePanelBackground)
            .resolve(theme);
        let border = component_color("component.tabs.border", ColorFallback::ThemePanelBorder)
            .resolve(theme);
        let tab_bg_hover = component_color(
            "component.tabs.tab_bg_hover",
            ColorFallback::ThemeHoverBackground,
        )
        .resolve(theme);
        let tab_bg_active = component_color(
            "component.tabs.tab_bg_active",
            ColorFallback::ThemeSelectionBackground,
        )
        .resolve(theme);
        let fg =
            component_color("component.tabs.fg", ColorFallback::ThemeTextPrimary).resolve(theme);
        let fg_muted = component_color("component.tabs.fg_inactive", ColorFallback::ThemeTextMuted)
            .resolve(theme);

        self.resolved = ResolvedTabsStyle {
            height,
            padding_x,
            gap,
            radius,
            border_width,
            text_size,
            bg,
            border,
            tab_bg_hover,
            tab_bg_active,
            fg,
            fg_muted,
        };
    }

    fn hit_test_tab(&self, point: Point) -> Option<usize> {
        self.prepared.iter().position(|t| t.rect.contains(point))
    }

    fn rebuild_tabs<H: UiHost>(&mut self, cx: &mut LayoutCx<'_, H>) {
        let prev = std::mem::take(&mut self.prepared);

        let text_style = TextStyle {
            font: fret_core::FontId::default(),
            size: self.resolved.text_size,
            ..Default::default()
        };
        let text_constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };

        let height = self.resolved.height.0.max(0.0);
        let pad_x = self.resolved.padding_x.0.max(0.0);
        let gap = self.resolved.gap.0.max(0.0);

        let mut x = cx.bounds.origin.x.0;
        for (i, label) in self.tabs.iter().cloned().enumerate() {
            let metrics = cx
                .services
                .text()
                .measure(label.as_ref(), text_style, text_constraints);
            let w = (metrics.size.width.0 + pad_x * 2.0).max(0.0);
            let rect = Rect::new(
                Point::new(Px(x), cx.bounds.origin.y),
                Size::new(Px(w), Px(height)),
            );
            let blob = prev
                .get(i)
                .and_then(|t| (t.label == label).then_some(t.blob))
                .flatten();
            self.prepared.push(PreparedTab {
                label,
                blob,
                metrics,
                rect,
            });
            x += w + gap;
        }
    }
}

impl Sizable for Tabs {
    fn with_size(self, size: ComponentSize) -> Self {
        Tabs::with_size(self, size)
    }
}

impl<H: UiHost> Widget<H> for Tabs {
    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        for tab in self.prepared.drain(..) {
            if let Some(blob) = tab.blob {
                services.text().release(blob);
            }
        }
        self.prepared_scale_factor_bits = None;
        self.prepared_theme_revision = None;
    }

    fn is_focusable(&self) -> bool {
        true
    }

    fn semantics(&mut self, cx: &mut fret_ui::widget::SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Panel);
        cx.set_disabled(self.disabled);
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        self.sync_style_from_theme(cx.theme());
        self.sync_bounds(cx.bounds);

        let selected = self.selected_index(cx.app);

        match event {
            Event::Pointer(pe) => match pe {
                fret_core::PointerEvent::Move { position, .. } => {
                    let hovered = self.hit_test_tab(*position);
                    if hovered != self.hovered {
                        self.hovered = hovered;
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                    }
                    if !self.disabled && (hovered.is_some() || cx.captured == Some(cx.node)) {
                        cx.set_cursor_icon(CursorIcon::Pointer);
                    }
                }
                fret_core::PointerEvent::Down {
                    position, button, ..
                } => {
                    if *button != MouseButton::Left || self.disabled {
                        return;
                    }
                    let hit = self.hit_test_tab(*position);
                    if hit.is_none() {
                        return;
                    }
                    self.pressed = hit;
                    cx.capture_pointer(cx.node);
                    cx.request_focus(cx.node);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                }
                fret_core::PointerEvent::Up {
                    position, button, ..
                } => {
                    if *button != MouseButton::Left {
                        return;
                    }
                    cx.release_pointer_capture();
                    let pressed = self.pressed.take();
                    let hit = self.hit_test_tab(*position);
                    if pressed.is_some() && pressed == hit && !self.disabled {
                        if let Some(i) = hit {
                            self.set_selected(cx.app, i);
                        }
                    }
                    self.hovered = hit;
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                }
                _ => {}
            },
            Event::KeyDown { key, repeat, .. } => {
                if *repeat || self.disabled {
                    return;
                }
                if cx.focus != Some(cx.node) {
                    return;
                }

                match key {
                    KeyCode::ArrowLeft => {
                        let next = selected.saturating_sub(1);
                        self.set_selected(cx.app, next);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    KeyCode::ArrowRight => {
                        let next = (selected + 1).min(self.tabs.len().saturating_sub(1));
                        self.set_selected(cx.app, next);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.sync_style_from_theme(cx.theme());
        cx.observe_model(self.model, Invalidation::Paint);

        self.last_bounds = cx.bounds;

        // Rebuild on any layout pass; tabs are small and this keeps behavior predictable for now.
        self.rebuild_tabs(cx);

        let h = self.resolved.height.0.max(0.0).min(cx.available.height.0);
        Size::new(cx.available.width, Px(h))
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_style_from_theme(cx.theme());
        cx.observe_model(self.model, Invalidation::Paint);
        self.sync_bounds(cx.bounds);

        let scale_bits = cx.scale_factor.to_bits();
        let theme_rev = cx.theme().revision();
        if self.prepared_scale_factor_bits != Some(scale_bits)
            || self.prepared_theme_revision != Some(theme_rev)
        {
            for tab in &mut self.prepared {
                if let Some(blob) = tab.blob.take() {
                    cx.services.text().release(blob);
                }
            }
            self.prepared_scale_factor_bits = Some(scale_bits);
            self.prepared_theme_revision = Some(theme_rev);
        }

        let border_w = Px(self.resolved.border_width.0.max(0.0));
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: self.resolved.bg,
            border: Edges::all(border_w),
            border_color: self.resolved.border,
            corner_radii: Corners::all(self.resolved.radius),
        });

        if self.prepared.is_empty() {
            return;
        }

        let selected = self.selected_index(cx.app);
        for (i, tab) in self.prepared.iter_mut().enumerate() {
            if tab.blob.is_none() {
                let text_style = TextStyle {
                    font: fret_core::FontId::default(),
                    size: self.resolved.text_size,
                    ..Default::default()
                };
                let text_constraints = TextConstraints {
                    max_width: None,
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                    scale_factor: cx.scale_factor,
                };
                let (blob, metrics) =
                    cx.services
                        .text()
                        .prepare(tab.label.as_ref(), text_style, text_constraints);
                tab.blob = Some(blob);
                tab.metrics = metrics;
            }

            let selected = i == selected;
            let hovered = self.hovered == Some(i);
            let pressed = self.pressed == Some(i);

            let bg = if selected {
                self.resolved.tab_bg_active
            } else if hovered || pressed {
                self.resolved.tab_bg_hover
            } else {
                Color {
                    a: 0.0,
                    ..self.resolved.bg
                }
            };

            if selected || hovered || pressed {
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(1),
                    rect: tab.rect,
                    background: bg,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: Corners::all(self.resolved.radius),
                });
            }

            let pad_x = self.resolved.padding_x.0.max(0.0);
            let text_x = Px(tab.rect.origin.x.0 + pad_x);
            let inner_y = tab.rect.origin.y.0
                + ((tab.rect.size.height.0 - tab.metrics.size.height.0) * 0.5).max(0.0);
            let text_y = Px(inner_y + tab.metrics.baseline.0);

            let color = if selected {
                self.resolved.fg
            } else {
                self.resolved.fg_muted
            };

            let Some(blob) = tab.blob else {
                continue;
            };
            cx.scene.push(SceneOp::Text {
                order: DrawOrder(2),
                origin: Point::new(text_x, text_y),
                text: blob,
                color,
            });
        }

        if cx.focus == Some(cx.node) && fret_ui::focus_visible::is_focus_visible(cx.app, cx.window)
        {
            let focus_ring = cx.theme().colors.focus_ring;
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(3),
                rect: cx.bounds,
                background: Color {
                    a: 0.0,
                    ..focus_ring
                },
                border: Edges::all(Px(2.0)),
                border_color: focus_ring,
                corner_radii: Corners::all(self.resolved.radius),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{
        AppWindowId, Event, Modifiers, Point, PointerEvent, Px, Rect, Scene, SceneOp, Size,
        PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle, TextBlobId,
        TextConstraints, TextMetrics, TextService, TextStyle,
    };
    use fret_ui::primitives::{Column, Scroll};
    use fret_ui::{EventCx, LayoutCx, PaintCx, UiHost, UiTree, Widget};

    #[derive(Default)]
    struct FakeText;

    impl TextService for FakeText {
        fn prepare(
            &mut self,
            text: &str,
            _style: TextStyle,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            let w = Px((text.chars().count() as f32) * 7.0);
            let h = Px(10.0);
            (
                TextBlobId::default(),
                TextMetrics {
                    size: fret_core::Size::new(w, h),
                    baseline: Px(8.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for FakeText {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    fn min_text_y(scene: &Scene) -> Option<Px> {
        scene
            .ops()
            .iter()
            .filter_map(|op| match op {
                SceneOp::Text { origin, .. } => Some(origin.y),
                _ => None,
            })
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
    }

    #[test]
    fn tabs_paint_tracks_scroll_translation_without_layout() {
        let mut app = App::new();
        let mut ui = UiTree::<App>::new();
        ui.set_window(AppWindowId::default());
        ui.set_paint_cache_enabled(false);

        let scroll = ui.create_node(Scroll::new());
        ui.set_root(scroll);
        let col = ui.create_node(Column::new());
        ui.add_child(scroll, col);

        let model = app.models_mut().insert(0usize);
        let tabs = ui.create_node(Tabs::new(
            model,
            vec!["Scene", "Game", "Inspector", "Console"],
        ));
        ui.add_child(col, tabs);

        // Make the column tall enough to scroll.
        #[derive(Default)]
        struct Spacer {
            height: Px,
        }

        impl Spacer {
            fn new(height: Px) -> Self {
                Self { height }
            }
        }

        impl<H: UiHost> Widget<H> for Spacer {
            fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
                Size::new(Px(1.0), self.height)
            }

            fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}

            fn event(&mut self, _cx: &mut EventCx<'_, H>, _event: &Event) {}
        }

        let spacer = ui.create_node(Spacer::new(Px(800.0)));
        ui.add_child(col, spacer);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );
        let mut text = FakeText::default();
        let mut scene = Scene::default();

        ui.layout_all(&mut app, &mut text, bounds, 1.0);
        ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);
        let y0 = min_text_y(&scene).expect("text drawn");

        // Scroll down (content moves up).
        ui.dispatch_event(
            &mut app,
            &mut text,
            &Event::Pointer(PointerEvent::Wheel {
                position: Point::new(Px(10.0), Px(10.0)),
                delta: Point::new(Px(0.0), Px(-50.0)),
                modifiers: Modifiers::default(),
            }),
        );

        scene.clear();
        ui.layout_all(&mut app, &mut text, bounds, 1.0);
        ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);
        let y1 = min_text_y(&scene).expect("text drawn after scroll");

        assert!(
            y1.0 < y0.0 - 1.0,
            "expected tabs text to move with scroll: before={y0:?} after={y1:?}"
        );
    }
}

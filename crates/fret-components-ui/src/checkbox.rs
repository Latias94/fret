use std::sync::Arc;

use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, KeyCode, MouseButton, Point, Px, Rect, SceneOp,
    SemanticsRole, Size, TextConstraints, TextMetrics, TextStyle, TextWrap,
};
use fret_runtime::Model;
use fret_ui::{EventCx, Invalidation, LayoutCx, PaintCx, Theme, UiHost, Widget};

use crate::style::{ColorFallback, MetricFallback, component_color, component_metric};

#[derive(Debug, Clone)]
struct PreparedText {
    blob: fret_core::TextBlobId,
    metrics: TextMetrics,
}

#[derive(Debug, Clone)]
struct ResolvedCheckboxStyle {
    box_size: Px,
    gap: Px,
    radius: Px,
    border_width: Px,
    min_height: Px,
    bg: Color,
    bg_hover: Color,
    border: Color,
    fg: Color,
    fg_disabled: Color,
    indicator: Color,
    indicator_disabled: Color,
}

impl Default for ResolvedCheckboxStyle {
    fn default() -> Self {
        Self {
            box_size: Px(16.0),
            gap: Px(8.0),
            radius: Px(4.0),
            border_width: Px(1.0),
            min_height: Px(28.0),
            bg: Color::TRANSPARENT,
            bg_hover: Color::TRANSPARENT,
            border: Color::TRANSPARENT,
            fg: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            fg_disabled: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.5,
            },
            indicator: Color {
                r: 0.4,
                g: 0.6,
                b: 1.0,
                a: 1.0,
            },
            indicator_disabled: Color {
                r: 0.4,
                g: 0.6,
                b: 1.0,
                a: 0.4,
            },
        }
    }
}

pub struct Checkbox {
    model: Model<bool>,
    label: Arc<str>,
    disabled: bool,

    hovered: bool,
    pressed: bool,
    last_bounds: Rect,
    last_theme_revision: Option<u64>,
    prepared: Option<PreparedText>,
    prepared_scale_factor_bits: Option<u32>,
    resolved: ResolvedCheckboxStyle,
}

impl Checkbox {
    pub fn new(model: Model<bool>, label: impl Into<Arc<str>>) -> Self {
        Self {
            model,
            label: label.into(),
            disabled: false,
            hovered: false,
            pressed: false,
            last_bounds: Rect::default(),
            last_theme_revision: None,
            prepared: None,
            prepared_scale_factor_bits: None,
            resolved: ResolvedCheckboxStyle::default(),
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    fn checked<H: UiHost>(&self, app: &H) -> bool {
        app.models().get(self.model).copied().unwrap_or(false)
    }

    fn toggle<H: UiHost>(&self, app: &mut H) {
        let _ = app.models_mut().update(self.model, |v| *v = !*v);
    }

    fn sync_style_from_theme(&mut self, theme: &Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        let box_size =
            component_metric("component.checkbox.box_size", MetricFallback::Px(Px(16.0)))
                .resolve(theme);
        let gap =
            component_metric("component.checkbox.gap", MetricFallback::Px(Px(8.0))).resolve(theme);
        let radius = component_metric("component.checkbox.radius", MetricFallback::Px(Px(4.0)))
            .resolve(theme);
        let border_width = component_metric(
            "component.checkbox.border_width",
            MetricFallback::Px(Px(1.0)),
        )
        .resolve(theme);
        let min_height = component_metric(
            "component.checkbox.min_height",
            MetricFallback::Px(Px(28.0)),
        )
        .resolve(theme);

        let bg = component_color("component.checkbox.bg", ColorFallback::ThemePanelBackground)
            .resolve(theme);
        let bg_hover = component_color(
            "component.checkbox.bg_hover",
            ColorFallback::ThemeHoverBackground,
        )
        .resolve(theme);
        let border = component_color("component.checkbox.border", ColorFallback::ThemePanelBorder)
            .resolve(theme);
        let fg = component_color("component.checkbox.fg", ColorFallback::ThemeTextPrimary)
            .resolve(theme);
        let fg_disabled = component_color(
            "component.checkbox.fg_disabled",
            ColorFallback::ThemeTextDisabled,
        )
        .resolve(theme);
        let indicator = component_color("component.checkbox.indicator", ColorFallback::ThemeAccent)
            .resolve(theme);
        let indicator_disabled = component_color(
            "component.checkbox.indicator_disabled",
            ColorFallback::ThemeTextDisabled,
        )
        .resolve(theme);

        self.resolved = ResolvedCheckboxStyle {
            box_size,
            gap,
            radius,
            border_width,
            min_height,
            bg,
            bg_hover,
            border,
            fg,
            fg_disabled,
            indicator,
            indicator_disabled,
        };
    }
}

impl<H: UiHost> Widget<H> for Checkbox {
    fn cleanup_resources(&mut self, text: &mut dyn fret_core::TextService) {
        if let Some(p) = self.prepared.take() {
            text.release(p.blob);
        }
        self.prepared_scale_factor_bits = None;
    }

    fn is_focusable(&self) -> bool {
        true
    }

    fn semantics(&mut self, cx: &mut fret_ui::widget::SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Button);
        cx.set_disabled(self.disabled);
        cx.set_selected(self.checked(cx.app));
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        self.sync_style_from_theme(cx.theme());

        match event {
            Event::Pointer(pe) => match pe {
                fret_core::PointerEvent::Move { position, .. } => {
                    let hovered = self.last_bounds.contains(*position);
                    if hovered != self.hovered {
                        self.hovered = hovered;
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                    }
                }
                fret_core::PointerEvent::Down {
                    position, button, ..
                } => {
                    if *button != MouseButton::Left || self.disabled {
                        return;
                    }
                    if !self.last_bounds.contains(*position) {
                        return;
                    }
                    self.pressed = true;
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

                    let was_pressed = self.pressed;
                    self.pressed = false;
                    cx.release_pointer_capture();

                    let hovered = self.last_bounds.contains(*position);
                    self.hovered = hovered;

                    if was_pressed && hovered && !self.disabled {
                        self.toggle(cx.app);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                    }

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
                if matches!(key, KeyCode::Space | KeyCode::Enter) {
                    self.toggle(cx.app);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                }
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.sync_style_from_theme(cx.theme());
        cx.observe_model(self.model, Invalidation::Paint);

        self.last_bounds = cx.bounds;

        let text_style = TextStyle {
            font: fret_core::FontId::default(),
            size: Px(13.0),
        };
        let text_constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            scale_factor: cx.scale_factor,
        };
        let metrics = cx.text.measure(&self.label, text_style, text_constraints);

        let box_w = self.resolved.box_size.0.max(0.0);
        let gap = self.resolved.gap.0.max(0.0);
        let text_w = metrics.size.width.0.max(0.0);
        let text_h = metrics.size.height.0.max(0.0);

        let min_h = self.resolved.min_height.0.max(0.0);
        let height = Px(text_h.max(box_w).max(min_h).min(cx.available.height.0));
        let width = Px((box_w + gap + text_w).min(cx.available.width.0).max(0.0));
        Size::new(width, height)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_style_from_theme(cx.theme());
        cx.observe_model(self.model, Invalidation::Paint);
        self.last_bounds = cx.bounds;

        let scale_bits = cx.scale_factor.to_bits();
        if self.prepared_scale_factor_bits != Some(scale_bits) {
            if let Some(p) = self.prepared.take() {
                cx.text.release(p.blob);
            }
            self.prepared_scale_factor_bits = None;
        }

        if self.prepared.is_none() {
            let text_style = TextStyle {
                font: fret_core::FontId::default(),
                size: Px(13.0),
            };
            let text_constraints = TextConstraints {
                max_width: None,
                wrap: TextWrap::None,
                scale_factor: cx.scale_factor,
            };
            let (blob, metrics) = cx.text.prepare(&self.label, text_style, text_constraints);
            self.prepared = Some(PreparedText { blob, metrics });
            self.prepared_scale_factor_bits = Some(scale_bits);
        }

        let checked = self.checked(cx.app);

        let box_w = self.resolved.box_size.0.max(0.0);
        let gap = self.resolved.gap.0.max(0.0);

        let box_x = cx.bounds.origin.x.0;
        let box_y = cx.bounds.origin.y.0 + ((cx.bounds.size.height.0 - box_w) * 0.5).max(0.0);
        let box_rect = Rect::new(
            Point::new(Px(box_x), Px(box_y)),
            Size::new(Px(box_w), Px(box_w)),
        );

        let bg = if self.disabled {
            self.resolved.bg
        } else if self.hovered || self.pressed {
            self.resolved.bg_hover
        } else {
            self.resolved.bg
        };

        let border_w = Px(self.resolved.border_width.0.max(0.0));
        let border = Edges::all(border_w);

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: box_rect,
            background: bg,
            border,
            border_color: self.resolved.border,
            corner_radii: Corners::all(self.resolved.radius),
        });

        if checked {
            let inset = (border_w.0 + 3.0).max(0.0);
            let inner_w = (box_w - inset * 2.0).max(0.0);
            let inner = Rect::new(
                Point::new(Px(box_x + inset), Px(box_y + inset)),
                Size::new(Px(inner_w), Px(inner_w)),
            );
            let c = if self.disabled {
                self.resolved.indicator_disabled
            } else {
                self.resolved.indicator
            };
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(1),
                rect: inner,
                background: c,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px((self.resolved.radius.0 - inset).max(0.0))),
            });
        }

        if cx.focus == Some(cx.node) {
            let focus_ring = cx.theme().colors.focus_ring;
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(2),
                rect: cx.bounds,
                background: Color {
                    a: 0.0,
                    ..focus_ring
                },
                border: Edges::all(Px(2.0)),
                border_color: focus_ring,
                corner_radii: Corners::all(cx.theme().metrics.radius_md),
            });
        }

        let Some(prepared) = self.prepared.as_ref() else {
            return;
        };
        let text_x = Px(cx.bounds.origin.x.0 + box_w + gap);
        let inner_y = cx.bounds.origin.y.0
            + ((cx.bounds.size.height.0 - prepared.metrics.size.height.0) * 0.5);
        let text_y = Px(inner_y + prepared.metrics.baseline.0);
        let color = if self.disabled {
            self.resolved.fg_disabled
        } else {
            self.resolved.fg
        };
        cx.scene.push(SceneOp::Text {
            order: DrawOrder(3),
            origin: Point::new(text_x, text_y),
            text: prepared.blob,
            color,
        });
    }
}

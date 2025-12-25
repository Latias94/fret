use fret_core::{
    Color, Corners, CursorIcon, DrawOrder, Edges, Event, KeyCode, MouseButton, Px, Rect, SceneOp,
    SemanticsRole, Size,
};
use fret_runtime::Model;
use fret_ui::{EventCx, Invalidation, LayoutCx, PaintCx, Theme, UiHost, Widget};

use crate::style::{ColorFallback, MetricFallback, component_color, component_metric};
use crate::{Sizable, Size as ComponentSize};

#[derive(Debug, Clone)]
struct ResolvedSwitchStyle {
    width: Px,
    height: Px,
    border_width: Px,
    radius: Px,
    padding: Px,
    bg_off: Color,
    bg_on: Color,
    border: Color,
    knob: Color,
    knob_disabled: Color,
    focus_ring: Color,
}

impl Default for ResolvedSwitchStyle {
    fn default() -> Self {
        Self {
            width: Px(40.0),
            height: Px(22.0),
            border_width: Px(1.0),
            radius: Px(999.0),
            padding: Px(2.0),
            bg_off: Color::TRANSPARENT,
            bg_on: Color::TRANSPARENT,
            border: Color::TRANSPARENT,
            knob: Color::TRANSPARENT,
            knob_disabled: Color::TRANSPARENT,
            focus_ring: Color::TRANSPARENT,
        }
    }
}

pub struct Switch {
    model: Model<bool>,
    disabled: bool,
    size: ComponentSize,
    hovered: bool,
    pressed: bool,
    last_bounds: Rect,
    last_theme_revision: Option<u64>,
    resolved: ResolvedSwitchStyle,
}

impl Switch {
    pub fn new(model: Model<bool>) -> Self {
        Self {
            model,
            disabled: false,
            size: ComponentSize::Medium,
            hovered: false,
            pressed: false,
            last_bounds: Rect::default(),
            last_theme_revision: None,
            resolved: ResolvedSwitchStyle::default(),
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

    fn checked<H: UiHost>(&self, app: &H) -> bool {
        app.models().get(self.model).copied().unwrap_or(false)
    }

    fn toggle<H: UiHost>(&self, app: &mut H) {
        let _ = app.models_mut().update(self.model, |v| *v = !*v);
    }

    fn sync_from_theme(&mut self, theme: &Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        let (width_default, height_default) = match self.size {
            ComponentSize::XSmall => (Px(34.0), Px(18.0)),
            ComponentSize::Small => (Px(38.0), Px(20.0)),
            ComponentSize::Medium => (Px(40.0), Px(22.0)),
            ComponentSize::Large => (Px(46.0), Px(24.0)),
        };
        let width = component_metric("component.switch.width", MetricFallback::Px(width_default))
            .resolve(theme);
        let height = component_metric(
            "component.switch.height",
            MetricFallback::Px(height_default),
        )
        .resolve(theme);
        let border_width =
            component_metric("component.switch.border_width", MetricFallback::Px(Px(1.0)))
                .resolve(theme);
        let radius = component_metric("component.switch.radius", MetricFallback::Px(Px(999.0)))
            .resolve(theme);
        let padding = component_metric("component.switch.padding", MetricFallback::Px(Px(2.0)))
            .resolve(theme);

        let bg_off = component_color(
            "component.switch.bg_off",
            // On dark themes, `panel_background`/`surface_background` can be too close to the
            // surrounding panel. `hover_background` is typically a subtle but visible overlay.
            ColorFallback::ThemeHoverBackground,
        )
        .resolve(theme);
        let bg_on =
            component_color("component.switch.bg_on", ColorFallback::ThemeAccent).resolve(theme);
        let border = component_color("component.switch.border", ColorFallback::ThemePanelBorder)
            .resolve(theme);
        let knob = component_color(
            "component.switch.knob",
            // The knob should stay visible even when `surface_background` and `panel_background`
            // are close; `text_primary` is a robust fallback across themes.
            ColorFallback::ThemeTextPrimary,
        )
        .resolve(theme);
        let knob_disabled = component_color(
            "component.switch.knob_disabled",
            ColorFallback::ThemeTextDisabled,
        )
        .resolve(theme);

        self.resolved = ResolvedSwitchStyle {
            width,
            height,
            border_width,
            radius,
            padding,
            bg_off,
            bg_on,
            border,
            knob,
            knob_disabled,
            focus_ring: theme.colors.focus_ring,
        };
    }
}

impl Sizable for Switch {
    fn with_size(self, size: ComponentSize) -> Self {
        Switch::with_size(self, size)
    }
}

impl<H: UiHost> Widget<H> for Switch {
    fn is_focusable(&self) -> bool {
        true
    }

    fn semantics(&mut self, cx: &mut fret_ui::widget::SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Button);
        cx.set_disabled(self.disabled);
        cx.set_selected(self.checked(cx.app));
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        self.sync_from_theme(cx.theme());
        self.last_bounds = cx.bounds;

        match event {
            Event::Pointer(pe) => match pe {
                fret_core::PointerEvent::Move { position, .. } => {
                    let hovered = self.last_bounds.contains(*position);
                    if hovered != self.hovered {
                        self.hovered = hovered;
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                    }
                    if !self.disabled && (hovered || cx.captured == Some(cx.node)) {
                        cx.set_cursor_icon(CursorIcon::Pointer);
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
        self.sync_from_theme(cx.theme());
        cx.observe_model(self.model, Invalidation::Paint);
        self.last_bounds = cx.bounds;

        let w = self.resolved.width.0.max(0.0).min(cx.available.width.0);
        let h = self.resolved.height.0.max(0.0).min(cx.available.height.0);
        Size::new(Px(w), Px(h))
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_from_theme(cx.theme());
        cx.observe_model(self.model, Invalidation::Paint);
        self.last_bounds = cx.bounds;

        let checked = self.checked(cx.app);

        let bg = if checked {
            self.resolved.bg_on
        } else if self.hovered || self.pressed {
            cx.theme().colors.hover_background
        } else {
            self.resolved.bg_off
        };

        let border_w = Px(self.resolved.border_width.0.max(0.0));
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: bg,
            border: Edges::all(border_w),
            border_color: self.resolved.border,
            corner_radii: Corners::all(self.resolved.radius),
        });

        let pad = self.resolved.padding.0.max(0.0);
        let inner_w = (cx.bounds.size.width.0 - pad * 2.0).max(0.0);
        let inner_h = (cx.bounds.size.height.0 - pad * 2.0).max(0.0);
        let knob_d = inner_h.min(inner_w).max(0.0);

        let knob_x = if checked {
            cx.bounds.origin.x.0 + cx.bounds.size.width.0 - pad - knob_d
        } else {
            cx.bounds.origin.x.0 + pad
        };
        let knob_y = cx.bounds.origin.y.0 + (cx.bounds.size.height.0 - knob_d) * 0.5;
        let knob_rect = Rect::new(
            fret_core::Point::new(Px(knob_x), Px(knob_y)),
            Size::new(Px(knob_d), Px(knob_d)),
        );

        let knob_color = if self.disabled {
            self.resolved.knob_disabled
        } else {
            self.resolved.knob
        };
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(1),
            rect: knob_rect,
            background: knob_color,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(Px(knob_d * 0.5)),
        });

        if cx.focus == Some(cx.node) && fret_ui::focus_visible::is_focus_visible(cx.app, cx.window)
        {
            let focus = self.resolved.focus_ring;
            let inset = Px(1.0);
            let w = (cx.bounds.size.width.0 - inset.0 * 2.0).max(0.0);
            let h = (cx.bounds.size.height.0 - inset.0 * 2.0).max(0.0);
            let rect = Rect::new(
                fret_core::Point::new(
                    Px(cx.bounds.origin.x.0 + inset.0),
                    Px(cx.bounds.origin.y.0 + inset.0),
                ),
                Size::new(Px(w), Px(h)),
            );
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(2),
                rect,
                background: Color { a: 0.0, ..focus },
                border: Edges::all(Px(2.0)),
                border_color: focus,
                corner_radii: Corners::all(self.resolved.radius),
            });
        }
    }
}

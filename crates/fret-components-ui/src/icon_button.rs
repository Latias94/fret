use fret_components_icons::{IconGlyph, IconId, IconRegistry};
use fret_core::{
    Color, Corners, CursorIcon, DrawOrder, Edges, Event, MouseButton, Point, Px, Rect, SceneOp,
    SemanticsRole, Size, TextConstraints, TextMetrics, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::CommandId;
use fret_ui::{EventCx, Invalidation, LayoutCx, PaintCx, Theme, UiHost, Widget};

use crate::style::{ColorFallback, MetricFallback, component_color, component_metric};
use crate::{Sizable, Size as ComponentSize};

#[derive(Debug, Clone)]
struct PreparedText {
    blob: fret_core::TextBlobId,
    metrics: TextMetrics,
}

#[derive(Debug, Clone)]
struct ResolvedIconButtonStyle {
    size: Px,
    radius: Px,
    border_width: Px,
    bg: Color,
    bg_hover: Color,
    bg_active: Color,
    border: Color,
    fg: Color,
    fg_disabled: Color,
}

impl Default for ResolvedIconButtonStyle {
    fn default() -> Self {
        Self {
            size: Px(32.0),
            radius: Px(8.0),
            border_width: Px(1.0),
            bg: Color::TRANSPARENT,
            bg_hover: Color::TRANSPARENT,
            bg_active: Color::TRANSPARENT,
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
        }
    }
}

pub struct IconButton {
    icon: IconId,
    command: Option<CommandId>,
    disabled: bool,
    size: ComponentSize,

    hovered: bool,
    pressed: bool,
    last_bounds: Rect,
    last_theme_revision: Option<u64>,
    prepared: Option<PreparedText>,
    prepared_scale_bits: Option<u32>,
    prepared_theme_revision: Option<u64>,
    prepared_icon_key: Option<String>,
    resolved: ResolvedIconButtonStyle,
}

impl IconButton {
    pub fn new(icon: IconId) -> Self {
        Self {
            icon,
            command: None,
            disabled: false,
            size: ComponentSize::Medium,
            hovered: false,
            pressed: false,
            last_bounds: Rect::default(),
            last_theme_revision: None,
            prepared: None,
            prepared_scale_bits: None,
            prepared_theme_revision: None,
            prepared_icon_key: None,
            resolved: ResolvedIconButtonStyle::default(),
        }
    }

    pub fn on_click(mut self, command: impl Into<CommandId>) -> Self {
        self.command = Some(command.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn with_size(mut self, size: ComponentSize) -> Self {
        self.size = size;
        self
    }

    fn sync_style_from_theme(&mut self, theme: &Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        let size_default = self.size.icon_button_size(theme);

        let size = component_metric(
            "component.icon_button.size",
            MetricFallback::Px(size_default),
        )
        .resolve(theme);
        let radius = component_metric(
            "component.icon_button.radius",
            MetricFallback::ThemeRadiusMd,
        )
        .resolve(theme);
        let border_width = component_metric(
            "component.icon_button.border_width",
            MetricFallback::Px(Px(1.0)),
        )
        .resolve(theme);

        let bg = component_color(
            "component.icon_button.bg",
            ColorFallback::Color(Color::TRANSPARENT),
        )
        .resolve(theme);
        let bg_hover = component_color(
            "component.icon_button.bg_hover",
            ColorFallback::ThemeHoverBackground,
        )
        .resolve(theme);
        let bg_active = component_color(
            "component.icon_button.bg_active",
            ColorFallback::ThemeSelectionBackground,
        )
        .resolve(theme);
        let border = component_color(
            "component.icon_button.border",
            ColorFallback::ThemePanelBorder,
        )
        .resolve(theme);
        let fg = component_color("component.icon_button.fg", ColorFallback::ThemeTextPrimary)
            .resolve(theme);
        let fg_disabled = component_color(
            "component.icon_button.fg_disabled",
            ColorFallback::ThemeTextDisabled,
        )
        .resolve(theme);

        self.resolved = ResolvedIconButtonStyle {
            size,
            radius,
            border_width,
            bg,
            bg_hover,
            bg_active,
            border,
            fg,
            fg_disabled,
        };
    }

    fn icon_glyph<H: UiHost>(&self, app: &mut H) -> IconGlyph {
        app.with_global_mut(IconRegistry::default, |icons, _app| {
            icons.ensure_builtin_glyphs();
            icons
                .glyph(&self.icon)
                .cloned()
                .unwrap_or_else(|| IconGlyph::new("?"))
        })
    }
}

impl Sizable for IconButton {
    fn with_size(self, size: ComponentSize) -> Self {
        IconButton::with_size(self, size)
    }
}

impl<H: UiHost> Widget<H> for IconButton {
    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        if let Some(p) = self.prepared.take() {
            services.text().release(p.blob);
        }
        self.prepared_scale_bits = None;
        self.prepared_theme_revision = None;
        self.prepared_icon_key = None;
    }

    fn is_focusable(&self) -> bool {
        true
    }

    fn semantics(&mut self, cx: &mut fret_ui::widget::SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Button);
        cx.set_disabled(self.disabled);
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        self.sync_style_from_theme(cx.theme());
        self.last_bounds = cx.bounds;

        let Event::Pointer(pe) = event else {
            return;
        };
        match pe {
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
                    if let Some(cmd) = self.command.clone() {
                        cx.dispatch_command(cmd);
                    }
                }

                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.sync_style_from_theme(cx.theme());
        self.last_bounds = cx.bounds;

        let size = self.resolved.size.0.max(0.0);
        let w = size.min(cx.available.width.0);
        let h = size.min(cx.available.height.0);
        Size::new(Px(w), Px(h))
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_style_from_theme(cx.theme());
        self.last_bounds = cx.bounds;

        let bg = if self.disabled {
            self.resolved.bg
        } else if self.pressed {
            self.resolved.bg_active
        } else if self.hovered {
            self.resolved.bg_hover
        } else {
            self.resolved.bg
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

        let scale_bits = cx.scale_factor.to_bits();
        let theme_rev = cx.theme().revision();
        let icon_glyph = self.icon_glyph(cx.app);
        let icon_key = format!(
            "{}|{}|{}|{:?}",
            self.icon.as_str(),
            icon_glyph.text.as_ref(),
            icon_glyph.size.0,
            icon_glyph.font
        );

        let needs_prepare = self.prepared.is_none()
            || self.prepared_scale_bits != Some(scale_bits)
            || self.prepared_theme_revision != Some(theme_rev)
            || self.prepared_icon_key.as_deref() != Some(icon_key.as_str());

        if needs_prepare {
            if let Some(p) = self.prepared.take() {
                cx.services.text().release(p.blob);
            }
            let style = TextStyle {
                font: icon_glyph.font,
                size: icon_glyph.size,
                ..Default::default()
            };
            let constraints = TextConstraints {
                max_width: None,
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                scale_factor: cx.scale_factor,
            };
            let (blob, metrics) =
                cx.services
                    .text()
                    .prepare(icon_glyph.text.as_ref(), style, constraints);
            self.prepared = Some(PreparedText { blob, metrics });
            self.prepared_scale_bits = Some(scale_bits);
            self.prepared_theme_revision = Some(theme_rev);
            self.prepared_icon_key = Some(icon_key);
        }

        let Some(prepared) = self.prepared.as_ref() else {
            return;
        };

        let text_x = cx.bounds.origin.x.0
            + ((cx.bounds.size.width.0 - prepared.metrics.size.width.0) * 0.5).max(0.0);
        let text_top = cx.bounds.origin.y.0
            + ((cx.bounds.size.height.0 - prepared.metrics.size.height.0) * 0.5).max(0.0);
        let text_y = text_top + prepared.metrics.baseline.0;

        let color = if self.disabled {
            self.resolved.fg_disabled
        } else {
            self.resolved.fg
        };

        cx.scene.push(SceneOp::Text {
            order: DrawOrder(1),
            origin: Point::new(Px(text_x), Px(text_y)),
            text: prepared.blob,
            color,
        });

        if cx.focus == Some(cx.node) && fret_ui::focus_visible::is_focus_visible(cx.app, cx.window)
        {
            let focus = cx.theme().colors.focus_ring;
            let inset = Px(1.0);
            let w = (cx.bounds.size.width.0 - inset.0 * 2.0).max(0.0);
            let h = (cx.bounds.size.height.0 - inset.0 * 2.0).max(0.0);
            let rect = Rect::new(
                Point::new(
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

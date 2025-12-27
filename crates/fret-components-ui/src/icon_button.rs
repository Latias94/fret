use fret_components_icons::{IconId, IconRegistry, MISSING_ICON_SVG, ResolvedSvgOwned};
use fret_core::{
    Color, Corners, CursorIcon, DrawOrder, Edges, Event, MouseButton, Px, Rect, SceneOp,
    SemanticsRole, Size, SvgFit,
};
use fret_runtime::CommandId;
use fret_ui::{EventCx, Invalidation, LayoutCx, PaintCx, Theme, UiHost, Widget};

use crate::style::{ColorFallback, MetricFallback, component_color, component_metric};
use crate::{Sizable, Size as ComponentSize};

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
}

impl Sizable for IconButton {
    fn with_size(self, size: ComponentSize) -> Self {
        IconButton::with_size(self, size)
    }
}

impl<H: UiHost> Widget<H> for IconButton {
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
            }
            fret_core::PointerEvent::Up { button, .. } => {
                if *button != MouseButton::Left || self.disabled {
                    return;
                }
                let was_pressed = self.pressed;
                self.pressed = false;
                cx.release_pointer_capture();
                if was_pressed {
                    if let Some(cmd) = self.command.clone() {
                        cx.dispatch_command(cmd);
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.sync_style_from_theme(cx.theme());
        Size::new(self.resolved.size, self.resolved.size)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_style_from_theme(cx.theme());

        let bg = if self.disabled {
            self.resolved.bg
        } else if self.pressed {
            self.resolved.bg_active
        } else if self.hovered {
            self.resolved.bg_hover
        } else {
            self.resolved.bg
        };

        let border_w = self.resolved.border_width;
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: bg,
            border: Edges::all(border_w),
            border_color: self.resolved.border,
            corner_radii: Corners::all(self.resolved.radius),
        });

        let color = if self.disabled {
            self.resolved.fg_disabled
        } else {
            self.resolved.fg
        };

        let resolved = cx
            .app
            .with_global_mut(IconRegistry::default, |icons, _app| {
                icons.resolve_svg_owned(&self.icon)
            });

        let svg_id = match resolved {
            Some(ResolvedSvgOwned::Static(bytes)) => cx.services.svg().register_svg(bytes),
            Some(ResolvedSvgOwned::Bytes(bytes)) => cx.services.svg().register_svg(bytes.as_ref()),
            None => cx.services.svg().register_svg(MISSING_ICON_SVG),
        };

        let icon_px = Px((self.resolved.size.0 * 0.5).max(0.0));
        let icon_px = Px(icon_px
            .0
            .min(cx.bounds.size.width.0)
            .min(cx.bounds.size.height.0));
        let x = cx.bounds.origin.x.0 + ((cx.bounds.size.width.0 - icon_px.0) * 0.5).max(0.0);
        let y = cx.bounds.origin.y.0 + ((cx.bounds.size.height.0 - icon_px.0) * 0.5).max(0.0);
        let rect = Rect::new(
            fret_core::Point::new(Px(x), Px(y)),
            Size::new(icon_px, icon_px),
        );

        cx.scene.push(SceneOp::SvgMaskIcon {
            order: DrawOrder(1),
            rect,
            svg: svg_id,
            fit: SvgFit::Contain,
            color,
            opacity: 1.0,
        });

        if cx.focus == Some(cx.node) && fret_ui::focus_visible::is_focus_visible(cx.app, cx.window)
        {
            let focus = cx.theme().colors.focus_ring;
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

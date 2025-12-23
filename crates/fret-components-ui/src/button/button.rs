use std::sync::Arc;

use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, FontId, MouseButton, Point, Px, Rect, SceneOp, Size,
    TextConstraints, TextMetrics, TextStyle, TextWrap,
};
use fret_runtime::CommandId;
use fret_ui::{EventCx, Invalidation, LayoutCx, PaintCx, UiHost, Widget};

use crate::style::{
    ColorFallback, MetricFallback, MetricRef, StyleRefinement, component_color, component_metric,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Default,
    Ghost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonIntent {
    Default,
    Primary,
    Danger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    Sm,
    Md,
    Lg,
}

#[derive(Debug, Clone)]
struct PreparedText {
    blob: fret_core::TextBlobId,
    metrics: TextMetrics,
}

pub struct Button {
    label: Arc<str>,
    command: Option<CommandId>,
    disabled: bool,
    variant: ButtonVariant,
    intent: ButtonIntent,
    size: ButtonSize,
    style: StyleRefinement,
    hovered: bool,
    pressed: bool,
    last_bounds: Rect,
    prepared: Option<PreparedText>,
    last_theme_revision: Option<u64>,
    resolved: ResolvedButtonStyle,
}

#[derive(Debug, Clone)]
struct ResolvedButtonStyle {
    padding_x: Px,
    padding_y: Px,
    min_height: Px,
    radius: Px,
    border_width: Px,
    bg: Color,
    bg_hover: Color,
    bg_active: Color,
    border: Color,
    fg: Color,
    fg_disabled: Color,
}

impl Default for ResolvedButtonStyle {
    fn default() -> Self {
        Self {
            padding_x: Px(10.0),
            padding_y: Px(6.0),
            min_height: Px(28.0),
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
                a: 1.0,
            },
        }
    }
}

impl Button {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            command: None,
            disabled: false,
            variant: ButtonVariant::Default,
            intent: ButtonIntent::Default,
            size: ButtonSize::Md,
            style: StyleRefinement::default(),
            hovered: false,
            pressed: false,
            last_bounds: Rect::default(),
            prepared: None,
            last_theme_revision: None,
            resolved: ResolvedButtonStyle::default(),
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

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn intent(mut self, intent: ButtonIntent) -> Self {
        self.intent = intent;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn refine_style(mut self, style: StyleRefinement) -> Self {
        self.style = style;
        self
    }

    fn sync_style_from_theme(&mut self, theme: &fret_ui::Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        let (default_px, default_py, min_h) = match self.size {
            ButtonSize::Sm => (Px(8.0), Px(5.0), Px(24.0)),
            ButtonSize::Md => (Px(10.0), Px(6.0), Px(28.0)),
            ButtonSize::Lg => (Px(12.0), Px(7.0), Px(32.0)),
        };

        let padding_x = self
            .style
            .padding_x
            .clone()
            .unwrap_or_else(|| {
                component_metric("component.button.padding_x", MetricFallback::Px(default_px))
            })
            .resolve(theme);
        let padding_y = self
            .style
            .padding_y
            .clone()
            .unwrap_or_else(|| {
                component_metric("component.button.padding_y", MetricFallback::Px(default_py))
            })
            .resolve(theme);
        let min_height = self
            .style
            .min_height
            .clone()
            .unwrap_or_else(|| {
                component_metric("component.button.min_height", MetricFallback::Px(min_h))
            })
            .resolve(theme);
        let radius = self
            .style
            .radius
            .clone()
            .unwrap_or_else(|| {
                component_metric("component.button.radius", MetricFallback::ThemeRadiusMd)
            })
            .resolve(theme);
        let border_width = self
            .style
            .border_width
            .clone()
            .unwrap_or(MetricRef::Px(Px(1.0)))
            .resolve(theme);

        let border = self
            .style
            .border_color
            .clone()
            .unwrap_or(component_color(
                "component.button.border",
                ColorFallback::ThemePanelBorder,
            ))
            .resolve(theme);

        let (bg, bg_hover, bg_active, fg) = resolve_button_colors(theme, self.variant, self.intent);
        let fg_disabled = component_color(
            "component.button.fg_disabled",
            ColorFallback::ThemeTextDisabled,
        )
        .resolve(theme);

        self.resolved = ResolvedButtonStyle {
            padding_x,
            padding_y,
            min_height,
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

fn resolve_button_colors(
    theme: &fret_ui::Theme,
    variant: ButtonVariant,
    intent: ButtonIntent,
) -> (Color, Color, Color, Color) {
    // Defaults are intentionally conservative: they must look acceptable even without `component.*` keys.
    let base_fg = theme.colors.text_primary;

    let fg = match intent {
        ButtonIntent::Default => {
            component_color("component.button.fg", ColorFallback::ThemeTextPrimary).resolve(theme)
        }
        ButtonIntent::Primary => component_color(
            "component.button.fg_primary",
            ColorFallback::ThemeTextPrimary,
        )
        .resolve(theme),
        ButtonIntent::Danger => component_color(
            "component.button.fg_danger",
            ColorFallback::ThemeTextPrimary,
        )
        .resolve(theme),
    };

    let (bg, bg_hover, bg_active) = match (variant, intent) {
        (ButtonVariant::Ghost, _) => (
            Color {
                a: 0.0,
                ..theme.colors.panel_background
            },
            theme.colors.hover_background,
            theme.colors.selection_background,
        ),
        (ButtonVariant::Default, ButtonIntent::Default) => (
            component_color("component.button.bg", ColorFallback::ThemePanelBackground)
                .resolve(theme),
            component_color(
                "component.button.bg_hover",
                ColorFallback::ThemeHoverBackground,
            )
            .resolve(theme),
            component_color(
                "component.button.bg_active",
                ColorFallback::ThemeSelectionBackground,
            )
            .resolve(theme),
        ),
        (ButtonVariant::Default, ButtonIntent::Primary) => (
            component_color("component.button.bg_primary", ColorFallback::ThemeAccent)
                .resolve(theme),
            component_color(
                "component.button.bg_primary_hover",
                ColorFallback::ThemeAccent,
            )
            .resolve(theme),
            component_color(
                "component.button.bg_primary_active",
                ColorFallback::ThemeAccent,
            )
            .resolve(theme),
        ),
        (ButtonVariant::Default, ButtonIntent::Danger) => (
            component_color(
                "component.button.bg_danger",
                ColorFallback::ThemeSelectionBackground,
            )
            .resolve(theme),
            component_color(
                "component.button.bg_danger_hover",
                ColorFallback::ThemeHoverBackground,
            )
            .resolve(theme),
            component_color(
                "component.button.bg_danger_active",
                ColorFallback::ThemeSelectionBackground,
            )
            .resolve(theme),
        ),
    };

    (
        bg,
        bg_hover,
        bg_active,
        if fg == Color::TRANSPARENT {
            base_fg
        } else {
            fg
        },
    )
}

impl<H: UiHost> Widget<H> for Button {
    fn is_focusable(&self) -> bool {
        true
    }

    fn semantics(&mut self, cx: &mut fret_ui::widget::SemanticsCx<'_, H>) {
        cx.set_role(fret_core::SemanticsRole::Button);
        cx.set_disabled(self.disabled);
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        self.sync_style_from_theme(cx.theme());

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

        if let Some(prepared) = self.prepared.take() {
            cx.text.release(prepared.blob);
        }

        let text_style = TextStyle {
            font: FontId::default(),
            size: Px(13.0),
        };
        let text_constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            scale_factor: cx.scale_factor,
        };
        let (blob, metrics) = cx.text.prepare(&self.label, text_style, text_constraints);
        self.prepared = Some(PreparedText { blob, metrics });

        let pad_x = self.resolved.padding_x.0.max(0.0);
        let pad_y = self.resolved.padding_y.0.max(0.0);
        let text_w = metrics.size.width.0.max(0.0);
        let text_h = metrics.size.height.0.max(0.0);

        let min_h = self.resolved.min_height.0.max(0.0);
        let height = Px((text_h + pad_y * 2.0).max(min_h).min(cx.available.height.0));
        let width = Px((text_w + pad_x * 2.0).min(cx.available.width.0).max(0.0));
        Size::new(width, height)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_style_from_theme(cx.theme());
        self.last_bounds = cx.bounds;

        let Some(prepared) = self.prepared.as_ref() else {
            return;
        };

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
        let border = Edges::all(border_w);

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: bg,
            border,
            border_color: self.resolved.border,
            corner_radii: Corners::all(self.resolved.radius),
        });

        if cx.focus == Some(cx.node) {
            let focus_ring = cx.theme().colors.focus_ring;
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(1),
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

        let pad_x = self.resolved.padding_x.0.max(0.0);
        let inner_w = (cx.bounds.size.width.0 - pad_x * 2.0).max(0.0);
        let text_x = cx.bounds.origin.x.0
            + pad_x
            + ((inner_w - prepared.metrics.size.width.0) * 0.5).max(0.0);

        let pad_y = self.resolved.padding_y.0.max(0.0);
        let inner_h = (cx.bounds.size.height.0 - pad_y * 2.0).max(0.0);
        let text_top = cx.bounds.origin.y.0
            + pad_y
            + ((inner_h - prepared.metrics.size.height.0) * 0.5).max(0.0);
        let text_y = text_top + prepared.metrics.baseline.0;

        let color = if self.disabled {
            self.resolved.fg_disabled
        } else {
            self.resolved.fg
        };

        cx.scene.push(SceneOp::Text {
            order: DrawOrder(2),
            origin: Point::new(Px(text_x), Px(text_y)),
            text: prepared.blob,
            color,
        });
    }
}

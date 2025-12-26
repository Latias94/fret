use std::sync::Arc;

use fret_core::{
    Color, Corners, CursorIcon, DrawOrder, Edges, Event, FontId, KeyCode, MouseButton, Point, Px,
    Rect, SceneOp, Size, TextConstraints, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::CommandId;
use fret_ui::{EventCx, Invalidation, LayoutCx, PaintCx, Theme, UiHost, Widget};

use fret_components_ui::{ChromeRefinement, MetricRef};
use fret_components_ui::{Sizable, Size as ComponentSize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Default,
    Destructive,
    Outline,
    Secondary,
    Ghost,
    Link,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    Default,
    Sm,
    Lg,
    Icon,
}

impl ButtonSize {
    fn component_size(self) -> ComponentSize {
        match self {
            Self::Default => ComponentSize::Medium,
            Self::Sm => ComponentSize::Small,
            Self::Lg => ComponentSize::Large,
            Self::Icon => ComponentSize::Medium,
        }
    }
}

#[derive(Debug, Clone)]
struct PreparedText {
    blob: fret_core::TextBlobId,
    metrics: fret_core::TextMetrics,
}

#[derive(Debug, Clone)]
struct ResolvedButtonStyle {
    padding_x: Px,
    padding_y: Px,
    min_height: Px,
    radius: Px,
    border_width: Px,
    text_size: Px,
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
            padding_x: Px(12.0),
            padding_y: Px(6.0),
            min_height: Px(32.0),
            radius: Px(8.0),
            border_width: Px(1.0),
            text_size: Px(13.0),
            bg: Color::TRANSPARENT,
            bg_hover: Color::TRANSPARENT,
            bg_active: Color::TRANSPARENT,
            border: Color::TRANSPARENT,
            fg: Color::TRANSPARENT,
            fg_disabled: Color::TRANSPARENT,
        }
    }
}

pub struct Button {
    label: Arc<str>,
    command: Option<CommandId>,
    disabled: bool,
    variant: ButtonVariant,
    size: ButtonSize,
    style: ChromeRefinement,
    hovered: bool,
    pressed: bool,
    last_bounds: Rect,
    prepared: Option<PreparedText>,
    prepared_scale_factor_bits: Option<u32>,
    prepared_theme_revision: Option<u64>,
    last_theme_revision: Option<u64>,
    resolved: ResolvedButtonStyle,
}

impl Button {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            command: None,
            disabled: false,
            variant: ButtonVariant::Default,
            size: ButtonSize::Default,
            style: ChromeRefinement::default(),
            hovered: false,
            pressed: false,
            last_bounds: Rect::default(),
            prepared: None,
            prepared_scale_factor_bits: None,
            prepared_theme_revision: None,
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
        self.last_theme_revision = None;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self.last_theme_revision = None;
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.style = style;
        self.last_theme_revision = None;
        self
    }

    fn sync_style_from_theme(&mut self, theme: &Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        let size = self.size.component_size();

        let default_px = size.button_px(theme);
        let default_py = size.button_py(theme);
        let default_h = size.button_h(theme);
        let default_radius = size.control_radius(theme);

        let padding_x = self
            .style
            .padding
            .as_ref()
            .and_then(|p| p.left.as_ref())
            .map(|m| m.resolve(theme))
            .or_else(|| theme.metric_by_key("component.button.padding_x"))
            .unwrap_or(default_px);
        let padding_y = self
            .style
            .padding
            .as_ref()
            .and_then(|p| p.top.as_ref())
            .map(|m| m.resolve(theme))
            .or_else(|| theme.metric_by_key("component.button.padding_y"))
            .unwrap_or(default_py);
        let min_height = self
            .style
            .min_height
            .clone()
            .map(|m| m.resolve(theme))
            .or_else(|| theme.metric_by_key("component.button.min_height"))
            .unwrap_or(default_h);
        let radius = self
            .style
            .radius
            .clone()
            .map(|m| m.resolve(theme))
            .or_else(|| theme.metric_by_key("component.button.radius"))
            .unwrap_or(default_radius);
        let border_width = self
            .style
            .border_width
            .clone()
            .unwrap_or(MetricRef::Px(Px(1.0)))
            .resolve(theme);

        let text_size = theme
            .metric_by_key("component.button.text_px")
            .unwrap_or_else(|| size.control_text_px(theme));

        let fg_disabled = theme
            .color_by_key("component.button.fg_disabled")
            .unwrap_or(theme.colors.text_disabled);

        let (bg, bg_hover, bg_active, border, fg) =
            resolve_shadcn_button_colors(theme, self.variant);

        self.resolved = ResolvedButtonStyle {
            padding_x,
            padding_y,
            min_height: Px(min_height.0.max(0.0)),
            radius: Px(radius.0.max(0.0)),
            border_width: Px(border_width.0.max(0.0)),
            text_size,
            bg,
            bg_hover,
            bg_active,
            border,
            fg,
            fg_disabled,
        };
    }
}

impl Sizable for Button {
    fn with_size(self, size: ComponentSize) -> Self {
        let mapped = match size {
            ComponentSize::XSmall | ComponentSize::Small => ButtonSize::Sm,
            ComponentSize::Medium => ButtonSize::Default,
            ComponentSize::Large => ButtonSize::Lg,
        };
        self.size(mapped)
    }
}

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn resolve_shadcn_button_colors(
    theme: &Theme,
    variant: ButtonVariant,
) -> (Color, Color, Color, Color, Color) {
    let transparent = Color::TRANSPARENT;

    let bg_primary = theme.color_by_key("primary").unwrap_or(theme.colors.accent);
    let fg_primary = theme
        .color_by_key("primary-foreground")
        .or_else(|| theme.color_by_key("primary.foreground"))
        .unwrap_or(theme.colors.text_primary);

    let bg_secondary = theme
        .color_by_key("secondary")
        .unwrap_or(theme.colors.panel_background);
    let fg_secondary = theme
        .color_by_key("secondary-foreground")
        .or_else(|| theme.color_by_key("secondary.foreground"))
        .unwrap_or(theme.colors.text_primary);

    let bg_destructive = theme
        .color_by_key("destructive")
        .unwrap_or(theme.colors.selection_background);
    let fg_destructive = theme
        .color_by_key("destructive-foreground")
        .or_else(|| theme.color_by_key("destructive.foreground"))
        .unwrap_or(theme.colors.text_primary);

    let fg_default = theme
        .color_by_key("foreground")
        .unwrap_or(theme.colors.text_primary);

    let bg_accent = theme
        .color_by_key("accent")
        .or_else(|| theme.color_by_key("accent.background"))
        .unwrap_or(theme.colors.hover_background);

    let border = theme
        .color_by_key("border")
        .unwrap_or(theme.colors.panel_border);

    match variant {
        ButtonVariant::Default => (
            bg_primary,
            alpha_mul(bg_primary, 0.9),
            alpha_mul(bg_primary, 0.8),
            transparent,
            fg_primary,
        ),
        ButtonVariant::Destructive => (
            bg_destructive,
            alpha_mul(bg_destructive, 0.9),
            alpha_mul(bg_destructive, 0.8),
            transparent,
            fg_destructive,
        ),
        ButtonVariant::Secondary => (
            bg_secondary,
            alpha_mul(bg_secondary, 0.9),
            alpha_mul(bg_secondary, 0.8),
            transparent,
            fg_secondary,
        ),
        ButtonVariant::Outline => (
            transparent,
            bg_accent,
            theme.colors.selection_background,
            border,
            fg_default,
        ),
        ButtonVariant::Ghost => (
            transparent,
            bg_accent,
            theme.colors.selection_background,
            transparent,
            fg_default,
        ),
        ButtonVariant::Link => (
            transparent,
            transparent,
            transparent,
            transparent,
            bg_primary,
        ),
    }
}

impl<H: UiHost> Widget<H> for Button {
    fn cleanup_resources(&mut self, text: &mut dyn fret_core::TextService) {
        if let Some(prepared) = self.prepared.take() {
            text.release(prepared.blob);
        }
        self.prepared_scale_factor_bits = None;
        self.prepared_theme_revision = None;
    }

    fn is_focusable(&self) -> bool {
        !self.disabled
    }

    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        !self.disabled
    }

    fn hit_test_children(&self, _bounds: Rect, _position: Point) -> bool {
        !self.disabled
    }

    fn semantics(&mut self, cx: &mut fret_ui::widget::SemanticsCx<'_, H>) {
        cx.set_role(fret_core::SemanticsRole::Button);
        cx.set_disabled(self.disabled);
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        self.sync_style_from_theme(cx.theme());
        self.last_bounds = cx.bounds;

        if self.disabled {
            return;
        }

        match event {
            Event::KeyDown { key, repeat, .. } => {
                if *repeat {
                    return;
                }
                if cx.focus != Some(cx.node) {
                    return;
                }
                if *key != KeyCode::Enter && *key != KeyCode::Space {
                    return;
                }

                if !self.pressed {
                    self.pressed = true;
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                }
                cx.stop_propagation();
            }
            Event::KeyUp { key, .. } => {
                if cx.focus != Some(cx.node) {
                    return;
                }
                if *key != KeyCode::Enter && *key != KeyCode::Space {
                    return;
                }

                let was_pressed = self.pressed;
                self.pressed = false;
                if was_pressed {
                    if let Some(command) = self.command.clone() {
                        cx.dispatch_command(command);
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                }
                cx.stop_propagation();
            }
            Event::Pointer(pe) => match pe {
                fret_core::PointerEvent::Move { position, .. } => {
                    cx.set_cursor_icon(CursorIcon::Pointer);
                    let hovered = cx.bounds.contains(*position);
                    if hovered != self.hovered {
                        self.hovered = hovered;
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                    }
                }
                fret_core::PointerEvent::Down {
                    button, position, ..
                } => {
                    if *button != MouseButton::Left {
                        return;
                    }
                    if !cx.bounds.contains(*position) {
                        return;
                    }
                    self.pressed = true;
                    cx.request_focus(cx.node);
                    cx.capture_pointer(cx.node);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                }
                fret_core::PointerEvent::Up {
                    button, position, ..
                } => {
                    if *button != MouseButton::Left {
                        return;
                    }
                    let was_pressed = self.pressed;
                    self.pressed = false;
                    cx.release_pointer_capture();

                    if was_pressed && cx.bounds.contains(*position) {
                        if let Some(command) = self.command.clone() {
                            cx.dispatch_command(command);
                        }
                    }

                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.sync_style_from_theme(cx.theme());
        self.last_bounds = cx.bounds;

        let text_style = TextStyle {
            font: FontId::default(),
            size: self.resolved.text_size,
            weight: fret_core::FontWeight::MEDIUM,
            ..Default::default()
        };
        let text_constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };
        let metrics = cx.text.measure(&self.label, text_style, text_constraints);

        let pad_x = self.resolved.padding_x.0.max(0.0);
        let pad_y = self.resolved.padding_y.0.max(0.0);
        let text_w = metrics.size.width.0.max(0.0);
        let text_h = metrics.size.height.0.max(0.0);

        let min_h = self.resolved.min_height.0.max(0.0);
        let height = Px((text_h + pad_y * 2.0).max(min_h).min(cx.available.height.0));
        let width = if self.size == ButtonSize::Icon {
            Px(height.0.min(cx.available.width.0).max(0.0))
        } else {
            Px((text_w + pad_x * 2.0).min(cx.available.width.0).max(0.0))
        };
        Size::new(width, height)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_style_from_theme(cx.theme());
        self.last_bounds = cx.bounds;

        let scale_bits = cx.scale_factor.to_bits();
        if self.prepared_scale_factor_bits != Some(scale_bits)
            || self.prepared_theme_revision != Some(cx.theme().revision())
        {
            if let Some(prepared) = self.prepared.take() {
                cx.text.release(prepared.blob);
            }
            self.prepared_scale_factor_bits = None;
            self.prepared_theme_revision = None;
        }

        if self.prepared.is_none() {
            let text_style = TextStyle {
                font: FontId::default(),
                size: self.resolved.text_size,
                weight: fret_core::FontWeight::MEDIUM,
                ..Default::default()
            };
            let text_constraints = TextConstraints {
                max_width: None,
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                scale_factor: cx.scale_factor,
            };
            let (blob, metrics) = cx.text.prepare(&self.label, text_style, text_constraints);
            self.prepared = Some(PreparedText { blob, metrics });
            self.prepared_scale_factor_bits = Some(scale_bits);
            self.prepared_theme_revision = Some(cx.theme().revision());
        }

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
        let border_color = if self.resolved.border == Color::TRANSPARENT {
            Color::TRANSPARENT
        } else {
            self.resolved.border
        };

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: bg,
            border,
            border_color,
            corner_radii: Corners::all(self.resolved.radius),
        });

        if cx.focus == Some(cx.node) && fret_ui::focus_visible::is_focus_visible(cx.app, cx.window)
        {
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

        let fg = if self.disabled {
            self.resolved.fg_disabled
        } else if self.resolved.fg == Color::TRANSPARENT {
            cx.theme().colors.text_primary
        } else {
            self.resolved.fg
        };

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
        let text_origin = Point::new(Px(text_x), Px(text_y));

        cx.scene.push(SceneOp::Text {
            order: DrawOrder(2),
            origin: text_origin,
            text: prepared.blob,
            color: fg,
        });

        if self.variant == ButtonVariant::Link && self.hovered && !self.pressed {
            let underline_h = Px(1.0);
            let underline_y = Px(text_y + 1.0);
            let underline_w = prepared.metrics.size.width;
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(3),
                rect: Rect::new(
                    Point::new(Px(text_x), underline_y),
                    Size::new(underline_w, underline_h),
                ),
                background: fg,
                border: Edges::default(),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::default(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Button, ButtonVariant};
    use crate::test_host::TestHost;
    use fret_ui::{Theme, ThemeConfig};

    #[test]
    fn button_variant_uses_semantic_palette_keys() {
        let mut theme = Theme::global(&TestHost::default()).clone();
        let mut cfg = ThemeConfig::default();
        cfg.name = "Semantic".to_string();
        cfg.colors
            .insert("primary".to_string(), "#0000ff".to_string());
        cfg.colors
            .insert("primary-foreground".to_string(), "#ffffff".to_string());
        cfg.colors
            .insert("destructive".to_string(), "#ff0000".to_string());
        cfg.colors
            .insert("destructive-foreground".to_string(), "#ffffff".to_string());
        cfg.colors
            .insert("border".to_string(), "#00ff00".to_string());
        cfg.colors
            .insert("foreground".to_string(), "#cccccc".to_string());
        cfg.colors
            .insert("accent".to_string(), "#333333".to_string());
        theme.apply_config(&cfg);

        let mut b = Button::new("x").variant(ButtonVariant::Destructive);
        b.sync_style_from_theme(&theme);
        assert!(b.resolved.bg.r > 0.0, "expected destructive bg");

        let mut o = Button::new("x").variant(ButtonVariant::Outline);
        o.sync_style_from_theme(&theme);
        assert!(o.resolved.border.g > 0.0, "expected outline border");
        assert_eq!(o.resolved.bg, fret_core::Color::TRANSPARENT);

        let mut d = Button::new("x").variant(ButtonVariant::Default);
        d.sync_style_from_theme(&theme);
        assert!(d.resolved.bg.b > 0.0, "expected primary bg");
        assert!(d.resolved.fg.r > 0.0, "expected primary fg");

        let mut g = Button::new("x").variant(ButtonVariant::Ghost);
        g.sync_style_from_theme(&theme);
        assert_eq!(g.resolved.bg, fret_core::Color::TRANSPARENT);
        // hover bg should be non-transparent (accent fallback)
        assert!(g.resolved.bg_hover.a >= 0.0);

        // Ensure size mapping doesn't produce negative metrics.
        let mut s = Button::new("x");
        s.sync_style_from_theme(&theme);
        assert!(s.resolved.min_height.0 >= 0.0);
        assert!(s.resolved.padding_x.0 >= 0.0);
        assert!(s.resolved.padding_y.0 >= 0.0);
        assert!(s.resolved.radius.0 >= 0.0);
        assert!(s.resolved.border_width.0 >= 0.0);
        assert!(s.resolved.text_size.0 >= 0.0);
    }
}

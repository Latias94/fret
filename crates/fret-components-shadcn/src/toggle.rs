use std::sync::Arc;

use fret_core::{
    Color, Corners, CursorIcon, DrawOrder, Edges, Event, KeyCode, MouseButton, Point, Px, Rect,
    SceneOp, SemanticsRole, Size, TextConstraints, TextMetrics, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::Model;
use fret_ui::{EventCx, Invalidation, LayoutCx, PaintCx, Theme, UiHost, Widget};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleVariant {
    Default,
    Outline,
}

impl Default for ToggleVariant {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleSize {
    Default,
    Sm,
    Lg,
}

impl Default for ToggleSize {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Debug, Clone)]
struct PreparedText {
    blob: fret_core::TextBlobId,
    metrics: TextMetrics,
}

#[derive(Debug, Clone)]
struct ResolvedToggleStyle {
    padding_x: Px,
    min_width: Px,
    min_height: Px,
    radius: Px,
    border_width: Px,
    text_style: TextStyle,
    fg: Color,
    fg_disabled: Color,
    fg_hover: Color,
    fg_on: Color,
    border: Color,
    border_on: Color,
    bg: Color,
    bg_hover: Color,
    bg_on: Color,
}

impl Default for ResolvedToggleStyle {
    fn default() -> Self {
        Self {
            padding_x: Px(8.0),
            min_width: Px(36.0),
            min_height: Px(36.0),
            radius: Px(8.0),
            border_width: Px(1.0),
            text_style: TextStyle::default(),
            fg: Color::TRANSPARENT,
            fg_disabled: Color::TRANSPARENT,
            fg_hover: Color::TRANSPARENT,
            fg_on: Color::TRANSPARENT,
            border: Color::TRANSPARENT,
            border_on: Color::TRANSPARENT,
            bg: Color::TRANSPARENT,
            bg_hover: Color::TRANSPARENT,
            bg_on: Color::TRANSPARENT,
        }
    }
}

pub struct Toggle {
    model: Model<bool>,
    label: Arc<str>,
    disabled: bool,
    variant: ToggleVariant,
    size: ToggleSize,
    hovered: bool,
    pressed: bool,
    last_bounds: Rect,
    prepared: Option<PreparedText>,
    prepared_scale_factor_bits: Option<u32>,
    prepared_theme_revision: Option<u64>,
    last_theme_revision: Option<u64>,
    resolved: ResolvedToggleStyle,
}

impl Toggle {
    pub fn new(model: Model<bool>, label: impl Into<Arc<str>>) -> Self {
        Self {
            model,
            label: label.into(),
            disabled: false,
            variant: ToggleVariant::Default,
            size: ToggleSize::Default,
            hovered: false,
            pressed: false,
            last_bounds: Rect::default(),
            prepared: None,
            prepared_scale_factor_bits: None,
            prepared_theme_revision: None,
            last_theme_revision: None,
            resolved: ResolvedToggleStyle::default(),
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn variant(mut self, variant: ToggleVariant) -> Self {
        self.variant = variant;
        self.last_theme_revision = None;
        self
    }

    pub fn size(mut self, size: ToggleSize) -> Self {
        self.size = size;
        self.last_theme_revision = None;
        self
    }

    fn is_on<H: UiHost>(&self, app: &H) -> bool {
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

        let (min_h, min_w, px) = match self.size {
            ToggleSize::Default => (Px(36.0), Px(36.0), Px(8.0)),
            ToggleSize::Sm => (Px(32.0), Px(32.0), Px(6.0)),
            ToggleSize::Lg => (Px(40.0), Px(40.0), Px(10.0)),
        };

        let radius = theme.metrics.radius_md;
        let border_w = Px(1.0);

        let text_px = theme
            .metric_by_key("component.toggle.text_px")
            .unwrap_or(Px(14.0));
        let line_height = theme
            .metric_by_key("component.toggle.line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or(theme.metrics.font_line_height);

        let fg = theme
            .color_by_key("foreground")
            .unwrap_or(theme.colors.text_primary);
        let fg_disabled = theme.colors.text_disabled;
        let fg_hover = theme
            .color_by_key("muted.foreground")
            .or_else(|| theme.color_by_key("muted-foreground"))
            .unwrap_or(theme.colors.text_muted);
        let fg_on = theme
            .color_by_key("accent-foreground")
            .or_else(|| theme.color_by_key("accent.foreground"))
            .unwrap_or(theme.colors.text_primary);

        let border = theme
            .color_by_key("input")
            .or_else(|| theme.color_by_key("border"))
            .unwrap_or(theme.colors.panel_border);
        let border_on = border;

        let transparent = Color::TRANSPARENT;
        let bg_on = theme
            .color_by_key("accent")
            .unwrap_or(theme.colors.hover_background);
        let bg_hover = match self.variant {
            ToggleVariant::Default => theme
                .color_by_key("muted")
                .unwrap_or(theme.colors.hover_background),
            ToggleVariant::Outline => bg_on,
        };

        let bg = transparent;

        self.resolved = ResolvedToggleStyle {
            padding_x: px,
            min_width: min_w,
            min_height: min_h,
            radius,
            border_width: border_w,
            text_style: TextStyle {
                font: fret_core::FontId::default(),
                size: text_px,
                line_height: Some(line_height),
                ..Default::default()
            },
            fg,
            fg_disabled,
            fg_hover,
            fg_on,
            border: match self.variant {
                ToggleVariant::Default => transparent,
                ToggleVariant::Outline => border,
            },
            border_on: match self.variant {
                ToggleVariant::Default => transparent,
                ToggleVariant::Outline => border_on,
            },
            bg,
            bg_hover,
            bg_on,
        };

        self.prepared_theme_revision = None;
        self.prepared_scale_factor_bits = None;
    }

    fn prepare_text<H: UiHost>(&mut self, cx: &mut LayoutCx<'_, H>) {
        let theme_rev = cx.theme().revision();
        let scale_bits = cx.scale_factor.to_bits();
        if self.prepared_theme_revision == Some(theme_rev)
            && self.prepared_scale_factor_bits == Some(scale_bits)
            && self.prepared.is_some()
        {
            return;
        }

        if let Some(p) = self.prepared.take() {
            cx.text.release(p.blob);
        }

        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };
        let (blob, metrics) = cx
            .text
            .prepare(&self.label, self.resolved.text_style, constraints);
        self.prepared = Some(PreparedText { blob, metrics });
        self.prepared_theme_revision = Some(theme_rev);
        self.prepared_scale_factor_bits = Some(scale_bits);
    }
}

impl<H: UiHost> Widget<H> for Toggle {
    fn cleanup_resources(&mut self, text: &mut dyn fret_core::TextService) {
        if let Some(p) = self.prepared.take() {
            text.release(p.blob);
        }
        self.prepared_scale_factor_bits = None;
        self.prepared_theme_revision = None;
    }

    fn is_focusable(&self) -> bool {
        true
    }

    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        !self.disabled
    }

    fn hit_test_children(&self, _bounds: Rect, _position: Point) -> bool {
        !self.disabled
    }

    fn semantics(&mut self, cx: &mut fret_ui::widget::SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Button);
        cx.set_disabled(self.disabled);
        cx.set_selected(self.is_on(cx.app));
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        self.sync_style_from_theme(cx.theme());
        self.last_bounds = cx.bounds;

        if self.disabled {
            return;
        }

        match event {
            Event::Pointer(pe) => match pe {
                fret_core::PointerEvent::Move { position, .. } => {
                    let hovered = self.last_bounds.contains(*position);
                    if hovered != self.hovered {
                        self.hovered = hovered;
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                    }
                    if hovered || cx.captured == Some(cx.node) {
                        cx.set_cursor_icon(CursorIcon::Pointer);
                    }
                }
                fret_core::PointerEvent::Down {
                    position, button, ..
                } => {
                    if *button != MouseButton::Left {
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
                    if was_pressed && hovered {
                        self.toggle(cx.app);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                    } else {
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                    }
                    cx.stop_propagation();
                }
                _ => {}
            },
            Event::KeyDown { key, repeat, .. } => {
                if *repeat {
                    return;
                }
                if cx.focus != Some(cx.node) {
                    return;
                }
                if !matches!(key, KeyCode::Enter | KeyCode::Space) {
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
                if !matches!(key, KeyCode::Enter | KeyCode::Space) {
                    return;
                }
                if self.pressed {
                    self.pressed = false;
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

        self.prepare_text(cx);
        let Some(prepared) = self.prepared.as_ref() else {
            return Size::new(Px(0.0), Px(0.0));
        };

        let pad_x = self.resolved.padding_x.0.max(0.0);
        let desired_w =
            (prepared.metrics.size.width.0 + pad_x * 2.0).max(self.resolved.min_width.0);
        let desired_h = self.resolved.min_height.0.max(0.0);

        let w = desired_w.min(cx.available.width.0).max(0.0);
        let h = desired_h.min(cx.available.height.0).max(0.0);
        Size::new(Px(w), Px(h))
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_style_from_theme(cx.theme());
        cx.observe_model(self.model, Invalidation::Paint);
        self.last_bounds = cx.bounds;

        let on = self.is_on(cx.app);
        let Some(prepared) = self.prepared.as_ref() else {
            return;
        };

        let (bg, border_color, fg) = if on {
            (
                self.resolved.bg_on,
                self.resolved.border_on,
                self.resolved.fg_on,
            )
        } else if self.pressed || self.hovered {
            (
                self.resolved.bg_hover,
                self.resolved.border,
                self.resolved.fg_hover,
            )
        } else {
            (self.resolved.bg, self.resolved.border, self.resolved.fg)
        };

        let mut bg = bg;
        let mut border_color = border_color;
        let mut fg = fg;
        if self.disabled {
            bg.a *= 0.5;
            border_color.a *= 0.5;
            fg = self.resolved.fg_disabled;
            fg.a *= 0.5;
        }

        let border_w = Px(self.resolved.border_width.0.max(0.0));
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: bg,
            border: Edges::all(border_w),
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

        let pad_x = self.resolved.padding_x.0.max(0.0);
        let inner_w = (cx.bounds.size.width.0 - pad_x * 2.0).max(0.0);
        let text_x = cx.bounds.origin.x.0
            + pad_x
            + ((inner_w - prepared.metrics.size.width.0) * 0.5).max(0.0);

        let inner_h = cx.bounds.size.height.0.max(0.0);
        let text_top =
            cx.bounds.origin.y.0 + ((inner_h - prepared.metrics.size.height.0) * 0.5).max(0.0);
        let text_y = text_top + prepared.metrics.baseline.0;

        cx.scene.push(SceneOp::Text {
            order: DrawOrder(2),
            origin: Point::new(Px(text_x), Px(text_y)),
            text: prepared.blob,
            color: fg,
        });
    }
}

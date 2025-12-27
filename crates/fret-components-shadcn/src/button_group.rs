use std::sync::Arc;

use fret_components_ui::Size as ComponentSize;
use fret_core::{
    Color, Corners, CursorIcon, DrawOrder, Edges, Event, KeyCode, MouseButton, Point, Px, Rect,
    SceneOp, SemanticsRole, Size, TextConstraints, TextMetrics, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::CommandId;
use fret_ui::{EventCx, Invalidation, LayoutCx, PaintCx, Theme, UiHost, Widget};

use crate::button::{ButtonSize, ButtonVariant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonGroupOrientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone)]
pub struct ButtonGroupItem {
    pub label: Arc<str>,
    pub command: Option<CommandId>,
    pub disabled: bool,
    pub variant: ButtonVariant,
    pub size: ButtonSize,
}

impl ButtonGroupItem {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            command: None,
            disabled: false,
            variant: ButtonVariant::Default,
            size: ButtonSize::Default,
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

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }
}

#[derive(Debug, Clone)]
struct PreparedText {
    blob: fret_core::TextBlobId,
    metrics: TextMetrics,
}

#[derive(Debug, Clone)]
struct ResolvedButtonGroupStyle {
    gap: Px,
    radius: Px,
    border_width: Px,
    ring_width: Px,
    fg_disabled: Color,
    text_px: Px,
    line_height: Px,
}

impl Default for ResolvedButtonGroupStyle {
    fn default() -> Self {
        Self {
            gap: Px(0.0),
            radius: Px(8.0),
            border_width: Px(1.0),
            ring_width: Px(2.0),
            fg_disabled: Color::TRANSPARENT,
            text_px: Px(13.0),
            line_height: Px(16.0),
        }
    }
}

pub struct ButtonGroup {
    orientation: ButtonGroupOrientation,
    items: Vec<ButtonGroupItem>,
    disabled: bool,
    hovered_index: Option<usize>,
    pressed_index: Option<usize>,
    active_index: usize,
    last_bounds: Rect,
    item_bounds: Vec<Rect>,
    prepared: Vec<Option<PreparedText>>,
    prepared_scale_factor_bits: Option<u32>,
    prepared_theme_revision: Option<u64>,
    last_theme_revision: Option<u64>,
    resolved: ResolvedButtonGroupStyle,
}

impl ButtonGroup {
    pub fn new() -> Self {
        Self {
            orientation: ButtonGroupOrientation::Horizontal,
            items: Vec::new(),
            disabled: false,
            hovered_index: None,
            pressed_index: None,
            active_index: 0,
            last_bounds: Rect::default(),
            item_bounds: Vec::new(),
            prepared: Vec::new(),
            prepared_scale_factor_bits: None,
            prepared_theme_revision: None,
            last_theme_revision: None,
            resolved: ResolvedButtonGroupStyle::default(),
        }
    }

    pub fn orientation(mut self, orientation: ButtonGroupOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn item(mut self, item: ButtonGroupItem) -> Self {
        self.items.push(item);
        self.prepared_theme_revision = None;
        self.prepared_scale_factor_bits = None;
        self
    }

    fn is_item_enabled(&self, index: usize) -> bool {
        self.items
            .get(index)
            .is_some_and(|it| !self.disabled && !it.disabled)
    }

    fn item_at(&self, position: Point) -> Option<usize> {
        self.item_bounds.iter().position(|r| r.contains(position))
    }

    fn hit_test_position(&self, position: Point) -> bool {
        if self.disabled {
            return false;
        }
        let Some(idx) = self.item_at(position) else {
            return false;
        };
        self.is_item_enabled(idx)
    }

    fn move_active(&mut self, delta: isize) {
        let n = self.items.len();
        if n == 0 {
            self.active_index = 0;
            return;
        }

        let mut idx = self.active_index as isize;
        for _ in 0..n {
            idx = (idx + delta).rem_euclid(n as isize);
            let i = idx as usize;
            if self.is_item_enabled(i) {
                self.active_index = i;
                return;
            }
        }
    }

    fn sync_style_from_theme(&mut self, theme: &Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        let radius = theme.metrics.radius_md;

        let fg_disabled = theme.colors.text_disabled;

        // Keep these aligned with the shadcn Button defaults (until Button exposes a shared style
        // resolver for reuse).
        let text_px = theme
            .metric_by_key("component.button.text_px")
            .unwrap_or_else(|| ComponentSize::Medium.control_text_px(theme));
        let line_height = theme
            .metric_by_key("font.line_height")
            .unwrap_or(theme.metrics.font_line_height);

        self.resolved = ResolvedButtonGroupStyle {
            gap: Px(0.0),
            radius,
            border_width: Px(1.0),
            ring_width: Px(2.0),
            fg_disabled,
            text_px,
            line_height,
        };

        self.prepared_theme_revision = None;
        self.prepared_scale_factor_bits = None;
    }

    fn resolve_item_colors(
        theme: &Theme,
        variant: ButtonVariant,
    ) -> (Color, Color, Color, Color, Color) {
        // Duplicate of the logic in `button.rs`, kept local to avoid cross-module churn for now.
        fn alpha_mul(mut c: Color, mul: f32) -> Color {
            c.a = (c.a * mul).clamp(0.0, 1.0);
            c
        }

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

    fn prepare_texts<H: UiHost>(&mut self, cx: &mut LayoutCx<'_, H>) {
        let theme_rev = cx.theme().revision();
        let scale_bits = cx.scale_factor.to_bits();
        if self.prepared_theme_revision == Some(theme_rev)
            && self.prepared_scale_factor_bits == Some(scale_bits)
            && self.prepared.len() == self.items.len()
            && self.prepared.iter().all(|p| p.is_some())
        {
            return;
        }

        for p in self.prepared.drain(..).flatten() {
            cx.services.text().release(p.blob);
        }
        self.prepared.clear();

        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };

        for item in &self.items {
            let text_style = TextStyle {
                font: fret_core::FontId::default(),
                size: self.resolved.text_px,
                weight: fret_core::FontWeight::MEDIUM,
                line_height: Some(self.resolved.line_height),
                letter_spacing_em: None,
            };
            let (blob, metrics) = cx
                .services
                .text()
                .prepare(&item.label, text_style, constraints);
            self.prepared.push(Some(PreparedText { blob, metrics }));
        }

        self.prepared_theme_revision = Some(theme_rev);
        self.prepared_scale_factor_bits = Some(scale_bits);
    }
}

impl Default for ButtonGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: UiHost> Widget<H> for ButtonGroup {
    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        for p in self.prepared.drain(..).flatten() {
            services.text().release(p.blob);
        }
        self.prepared_scale_factor_bits = None;
        self.prepared_theme_revision = None;
    }

    fn is_focusable(&self) -> bool {
        !self.disabled && self.items.iter().any(|it| !it.disabled)
    }

    fn hit_test(&self, _bounds: Rect, position: Point) -> bool {
        self.hit_test_position(position)
    }

    fn hit_test_children(&self, _bounds: Rect, position: Point) -> bool {
        self.hit_test_position(position)
    }

    fn semantics(&mut self, cx: &mut fret_ui::widget::SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Generic);
        cx.set_disabled(self.disabled);
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
                    let mut hovered = self.item_at(*position);
                    if hovered.is_some_and(|i| !self.is_item_enabled(i)) {
                        hovered = None;
                    }
                    if hovered != self.hovered_index {
                        self.hovered_index = hovered;
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                    }
                    if hovered.is_some() || cx.captured == Some(cx.node) {
                        cx.set_cursor_icon(CursorIcon::Pointer);
                    }
                }
                fret_core::PointerEvent::Down {
                    position, button, ..
                } => {
                    if *button != MouseButton::Left {
                        return;
                    }
                    let Some(idx) = self.item_at(*position) else {
                        return;
                    };
                    if !self.is_item_enabled(idx) {
                        return;
                    }
                    self.active_index = idx;
                    self.pressed_index = Some(idx);
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
                    let pressed = self.pressed_index.take();
                    cx.release_pointer_capture();

                    let hovered = self.item_at(*position);
                    self.hovered_index = hovered.filter(|i| self.is_item_enabled(*i));

                    if let (Some(idx), Some(hov)) = (pressed, hovered)
                        && idx == hov
                        && let Some(item) = self.items.get(idx)
                        && let Some(cmd) = item.command.clone()
                    {
                        cx.dispatch_command(cmd);
                    }

                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
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

                match (self.orientation, key) {
                    (ButtonGroupOrientation::Horizontal, KeyCode::ArrowLeft)
                    | (ButtonGroupOrientation::Vertical, KeyCode::ArrowUp) => {
                        self.move_active(-1);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    (ButtonGroupOrientation::Horizontal, KeyCode::ArrowRight)
                    | (ButtonGroupOrientation::Vertical, KeyCode::ArrowDown) => {
                        self.move_active(1);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    (_, KeyCode::Home) => {
                        self.active_index = 0;
                        self.move_active(0);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    (_, KeyCode::End) => {
                        self.active_index = self.items.len().saturating_sub(1);
                        self.move_active(0);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    (_, KeyCode::Enter | KeyCode::Space) => {
                        if self.pressed_index.is_none() && self.is_item_enabled(self.active_index) {
                            self.pressed_index = Some(self.active_index);
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                        }
                        cx.stop_propagation();
                    }
                    _ => {}
                }
            }
            Event::KeyUp { key, .. } => {
                if cx.focus != Some(cx.node) {
                    return;
                }
                if !matches!(key, KeyCode::Enter | KeyCode::Space) {
                    return;
                }
                let pressed = self.pressed_index.take();
                if pressed != Some(self.active_index) {
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }
                if let Some(item) = self.items.get(self.active_index)
                    && self.is_item_enabled(self.active_index)
                    && let Some(cmd) = item.command.clone()
                {
                    cx.dispatch_command(cmd);
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

        self.prepare_texts(cx);

        let n = self.items.len();
        if n == 0 {
            self.item_bounds.clear();
            return Size::new(Px(0.0), Px(0.0));
        }

        let gap = self.resolved.gap.0.max(0.0);

        let mut item_ws: Vec<f32> = Vec::with_capacity(n);
        let mut item_hs: Vec<f32> = Vec::with_capacity(n);
        for i in 0..n {
            let Some(item) = self.items.get(i) else {
                item_ws.push(0.0);
                item_hs.push(0.0);
                continue;
            };

            let comp_size = match item.size {
                ButtonSize::Default | ButtonSize::Icon => ComponentSize::Medium,
                ButtonSize::Sm => ComponentSize::Small,
                ButtonSize::Lg => ComponentSize::Large,
            };
            let default_px = comp_size.button_px(cx.theme());
            let default_py = comp_size.button_py(cx.theme());
            let default_h = comp_size.button_h(cx.theme());

            let pad_x = cx
                .theme()
                .metric_by_key("component.button.padding_x")
                .unwrap_or(default_px)
                .0
                .max(0.0);
            let pad_y = cx
                .theme()
                .metric_by_key("component.button.padding_y")
                .unwrap_or(default_py)
                .0
                .max(0.0);
            let min_h = cx
                .theme()
                .metric_by_key("component.button.min_height")
                .unwrap_or(default_h)
                .0
                .max(0.0);

            let Some(Some(p)) = self.prepared.get(i) else {
                item_ws.push(0.0);
                item_hs.push(min_h);
                continue;
            };

            let text_w = p.metrics.size.width.0.max(0.0);
            let text_h = p.metrics.size.height.0.max(0.0);
            let h = (text_h + pad_y * 2.0).max(min_h);
            let w = if item.size == ButtonSize::Icon {
                h
            } else {
                (text_w + pad_x * 2.0).max(0.0)
            };
            item_ws.push(w);
            item_hs.push(h);
        }

        let available_w = cx.available.width.0.max(0.0);
        let available_h = cx.available.height.0.max(0.0);

        match self.orientation {
            ButtonGroupOrientation::Horizontal => {
                let desired_w = item_ws.iter().sum::<f32>() + gap * (n.saturating_sub(1) as f32);
                let desired_h = item_hs.iter().copied().fold(0.0, f32::max);

                let total_w = desired_w.min(available_w);
                let total_h = desired_h.min(available_h);

                if desired_w > 0.0 && total_w < desired_w {
                    let each = (total_w - gap * (n.saturating_sub(1) as f32)).max(0.0) / (n as f32);
                    item_ws.fill(each);
                }

                self.item_bounds.clear();
                self.item_bounds.reserve(n);
                let mut x = cx.bounds.origin.x.0;
                let y = cx.bounds.origin.y.0;
                for (i, w) in item_ws.iter().enumerate() {
                    let rect = Rect {
                        origin: Point::new(Px(x), Px(y)),
                        size: Size::new(Px(*w), Px(total_h)),
                    };
                    self.item_bounds.push(rect);
                    x += *w;
                    if i + 1 < n {
                        x += gap;
                    }
                }
                Size::new(Px(total_w), Px(total_h))
            }
            ButtonGroupOrientation::Vertical => {
                let desired_w = item_ws.iter().copied().fold(0.0, f32::max);
                let desired_h = item_hs.iter().sum::<f32>() + gap * (n.saturating_sub(1) as f32);

                let total_w = desired_w.min(available_w);
                let total_h = desired_h.min(available_h);

                if desired_h > 0.0 && total_h < desired_h {
                    let each = (total_h - gap * (n.saturating_sub(1) as f32)).max(0.0) / (n as f32);
                    item_hs.fill(each);
                }

                self.item_bounds.clear();
                self.item_bounds.reserve(n);
                let x = cx.bounds.origin.x.0;
                let mut y = cx.bounds.origin.y.0;
                for (i, h) in item_hs.iter().enumerate() {
                    let rect = Rect {
                        origin: Point::new(Px(x), Px(y)),
                        size: Size::new(Px(total_w), Px(*h)),
                    };
                    self.item_bounds.push(rect);
                    y += *h;
                    if i + 1 < n {
                        y += gap;
                    }
                }
                Size::new(Px(total_w), Px(total_h))
            }
        }
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_style_from_theme(cx.theme());
        self.last_bounds = cx.bounds;

        if self.disabled {
            self.hovered_index = None;
            self.pressed_index = None;
        }

        let n = self.items.len();
        if n == 0 {
            return;
        }

        let focused = cx.focus == Some(cx.node)
            && fret_ui::focus_visible::is_focus_visible(cx.app, cx.window);

        for i in 0..n {
            let Some(item) = self.items.get(i) else {
                continue;
            };
            let Some(rect) = self.item_bounds.get(i).copied() else {
                continue;
            };
            let Some(Some(prepared)) = self.prepared.get(i) else {
                continue;
            };

            let enabled = !self.disabled && !item.disabled;
            let hovered = self.hovered_index == Some(i);
            let pressed = self.pressed_index == Some(i);

            let (bg, bg_hover, bg_active, border, fg) =
                Self::resolve_item_colors(cx.theme(), item.variant);

            let (mut bg, mut border_color, mut fg) = if pressed {
                (bg_active, border, fg)
            } else if hovered {
                (bg_hover, border, fg)
            } else {
                (bg, border, fg)
            };

            if !enabled {
                bg.a *= 0.5;
                border_color.a *= 0.5;
                fg = self.resolved.fg_disabled;
                fg.a *= 0.5;
            }

            let border_w = Px(self.resolved.border_width.0.max(0.0));
            let mut border_edges = Edges::all(border_w);

            let radius = self.resolved.radius;
            let corner_radii = match (self.orientation, self.resolved.gap == Px(0.0), n) {
                (_, false, _) | (_, _, 1) => Corners::all(radius),
                (ButtonGroupOrientation::Horizontal, true, _) => {
                    if i == 0 {
                        Corners {
                            top_left: radius,
                            bottom_left: radius,
                            top_right: Px(0.0),
                            bottom_right: Px(0.0),
                        }
                    } else if i + 1 == n {
                        Corners {
                            top_left: Px(0.0),
                            bottom_left: Px(0.0),
                            top_right: radius,
                            bottom_right: radius,
                        }
                    } else {
                        Corners::all(Px(0.0))
                    }
                }
                (ButtonGroupOrientation::Vertical, true, _) => {
                    if i == 0 {
                        Corners {
                            top_left: radius,
                            top_right: radius,
                            bottom_left: Px(0.0),
                            bottom_right: Px(0.0),
                        }
                    } else if i + 1 == n {
                        Corners {
                            top_left: Px(0.0),
                            top_right: Px(0.0),
                            bottom_left: radius,
                            bottom_right: radius,
                        }
                    } else {
                        Corners::all(Px(0.0))
                    }
                }
            };

            if self.resolved.gap == Px(0.0) {
                match self.orientation {
                    ButtonGroupOrientation::Horizontal => {
                        if i > 0 {
                            border_edges.left = Px(0.0);
                        }
                    }
                    ButtonGroupOrientation::Vertical => {
                        if i > 0 {
                            border_edges.top = Px(0.0);
                        }
                    }
                }
            }

            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(0),
                rect,
                background: bg,
                border: border_edges,
                border_color,
                corner_radii,
            });

            if focused && i == self.active_index {
                let ring = cx
                    .theme()
                    .color_by_key("ring")
                    .unwrap_or(cx.theme().colors.focus_ring);
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(1),
                    rect,
                    background: Color { a: 0.0, ..ring },
                    border: Edges::all(Px(self.resolved.ring_width.0.max(0.0))),
                    border_color: ring,
                    corner_radii,
                });
            }

            let comp_size = match item.size {
                ButtonSize::Default | ButtonSize::Icon => ComponentSize::Medium,
                ButtonSize::Sm => ComponentSize::Small,
                ButtonSize::Lg => ComponentSize::Large,
            };
            let default_px = comp_size.button_px(cx.theme());
            let default_py = comp_size.button_py(cx.theme());

            let pad_x = cx
                .theme()
                .metric_by_key("component.button.padding_x")
                .unwrap_or(default_px)
                .0
                .max(0.0);
            let inner_w = (rect.size.width.0 - pad_x * 2.0).max(0.0);
            let text_x = rect.origin.x.0
                + pad_x
                + ((inner_w - prepared.metrics.size.width.0) * 0.5).max(0.0);

            let pad_y = cx
                .theme()
                .metric_by_key("component.button.padding_y")
                .unwrap_or(default_py)
                .0
                .max(0.0);
            let inner_h = (rect.size.height.0 - pad_y * 2.0).max(0.0);
            let text_top = rect.origin.y.0
                + pad_y
                + ((inner_h - prepared.metrics.size.height.0) * 0.5).max(0.0);
            let text_y = text_top + prepared.metrics.baseline.0;
            let origin = Point::new(Px(text_x), Px(text_y));

            cx.scene.push(SceneOp::Text {
                order: DrawOrder(2),
                origin,
                text: prepared.blob,
                color: fg,
            });
        }
    }
}

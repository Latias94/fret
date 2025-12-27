use std::sync::Arc;

use fret_components_ui::{MetricRef, Space};
use fret_core::{
    Color, Corners, CursorIcon, DrawOrder, Edges, Event, KeyCode, MouseButton, Point, Px, Rect,
    SceneOp, SemanticsRole, Size, TextConstraints, TextMetrics, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::Model;
use fret_ui::{EventCx, Invalidation, LayoutCx, PaintCx, Theme, UiHost, Widget};

use crate::toggle::{ToggleSize, ToggleVariant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleGroupType {
    Single,
    Multiple,
}

#[derive(Debug, Clone)]
pub struct ToggleGroupItem {
    pub value: Arc<str>,
    pub label: Arc<str>,
    pub disabled: bool,
}

impl ToggleGroupItem {
    pub fn new(value: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Clone, Copy)]
enum SelectionModel {
    Single(Model<Option<Arc<str>>>),
    Multiple(Model<Vec<Arc<str>>>),
}

#[derive(Debug, Clone)]
struct PreparedText {
    blob: fret_core::TextBlobId,
    metrics: TextMetrics,
}

#[derive(Debug, Clone)]
struct ResolvedToggleGroupStyle {
    spacing: Px,
    padding_x: Px,
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

impl Default for ResolvedToggleGroupStyle {
    fn default() -> Self {
        Self {
            spacing: Px(0.0),
            padding_x: Px(12.0),
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

pub struct ToggleGroup {
    selection: SelectionModel,
    items: Vec<ToggleGroupItem>,
    disabled: bool,
    variant: ToggleVariant,
    size: ToggleSize,
    spacing: Space,
    hovered_index: Option<usize>,
    pressed_index: Option<usize>,
    active_index: usize,
    last_bounds: Rect,
    item_bounds: Vec<Rect>,
    prepared: Vec<Option<PreparedText>>,
    prepared_scale_factor_bits: Option<u32>,
    prepared_theme_revision: Option<u64>,
    last_theme_revision: Option<u64>,
    resolved: ResolvedToggleGroupStyle,
}

impl ToggleGroup {
    pub fn single(model: Model<Option<Arc<str>>>) -> Self {
        Self::new(SelectionModel::Single(model))
    }

    pub fn multiple(model: Model<Vec<Arc<str>>>) -> Self {
        Self::new(SelectionModel::Multiple(model))
    }

    fn new(selection: SelectionModel) -> Self {
        Self {
            selection,
            items: Vec::new(),
            disabled: false,
            variant: ToggleVariant::Default,
            size: ToggleSize::Default,
            spacing: Space::N0,
            hovered_index: None,
            pressed_index: None,
            active_index: 0,
            last_bounds: Rect::default(),
            item_bounds: Vec::new(),
            prepared: Vec::new(),
            prepared_scale_factor_bits: None,
            prepared_theme_revision: None,
            last_theme_revision: None,
            resolved: ResolvedToggleGroupStyle::default(),
        }
    }

    pub fn item(mut self, item: ToggleGroupItem) -> Self {
        self.items.push(item);
        self.prepared_theme_revision = None;
        self.prepared_scale_factor_bits = None;
        self
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

    pub fn spacing(mut self, spacing: Space) -> Self {
        self.spacing = spacing;
        self.last_theme_revision = None;
        self
    }

    fn observe_selection_model<H: UiHost>(&self, cx: &mut impl ObserveModelCx<H>) {
        match self.selection {
            SelectionModel::Single(m) => cx.observe_model(m, Invalidation::Paint),
            SelectionModel::Multiple(m) => cx.observe_model(m, Invalidation::Paint),
        }
    }

    fn is_selected<H: UiHost>(&self, app: &H, value: &Arc<str>) -> bool {
        match self.selection {
            SelectionModel::Single(m) => {
                app.models().get(m).and_then(|v| v.as_ref()) == Some(value)
            }
            SelectionModel::Multiple(m) => app
                .models()
                .get(m)
                .map(|v| v.iter().any(|x| x == value))
                .unwrap_or(false),
        }
    }

    fn toggle_value<H: UiHost>(&self, app: &mut H, value: Arc<str>) {
        match self.selection {
            SelectionModel::Single(m) => {
                let _ = app.models_mut().update(m, |v| {
                    if v.as_ref() == Some(&value) {
                        *v = None;
                    } else {
                        *v = Some(value);
                    }
                });
            }
            SelectionModel::Multiple(m) => {
                let _ = app.models_mut().update(m, |v| {
                    if let Some(pos) = v.iter().position(|x| x == &value) {
                        v.remove(pos);
                    } else {
                        v.push(value);
                    }
                });
            }
        }
    }

    fn sync_style_from_theme(&mut self, theme: &Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        let spacing = MetricRef::space(self.spacing).resolve(theme);

        let (min_h, default_padding_space) = match self.size {
            ToggleSize::Default => (Px(36.0), Space::N3),
            ToggleSize::Sm => (Px(32.0), Space::N2p5),
            ToggleSize::Lg => (Px(40.0), Space::N3p5),
        };
        let px = theme
            .metric_by_key("component.toggle_group.padding_x")
            .unwrap_or_else(|| MetricRef::space(default_padding_space).resolve(theme));

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

        self.resolved = ResolvedToggleGroupStyle {
            spacing,
            padding_x: px,
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
            let (blob, metrics) =
                cx.services
                    .text()
                    .prepare(&item.label, self.resolved.text_style, constraints);
            self.prepared.push(Some(PreparedText { blob, metrics }));
        }

        self.prepared_theme_revision = Some(theme_rev);
        self.prepared_scale_factor_bits = Some(scale_bits);
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

    fn is_item_enabled(&self, index: usize) -> bool {
        self.items
            .get(index)
            .is_some_and(|it| !self.disabled && !it.disabled)
    }

    fn sync_active_index<H: UiHost>(&mut self, app: &H) {
        if self.items.is_empty() {
            self.active_index = 0;
            return;
        }
        if self.active_index >= self.items.len() {
            self.active_index = 0;
        }

        let Some(_cur) = self.items.get(self.active_index) else {
            self.active_index = 0;
            return;
        };
        if self.is_item_enabled(self.active_index) {
            return;
        }

        if let Some((idx, _)) = self
            .items
            .iter()
            .enumerate()
            .find(|(i, it)| self.is_item_enabled(*i) && self.is_selected(app, &it.value))
        {
            self.active_index = idx;
            return;
        }

        if let Some((idx, _)) = self
            .items
            .iter()
            .enumerate()
            .find(|(i, _)| self.is_item_enabled(*i))
        {
            self.active_index = idx;
        }
    }

    fn move_active(&mut self, delta: i32) {
        if self.items.is_empty() {
            return;
        }
        let len = self.items.len() as i32;
        let mut idx = self.active_index as i32;
        for _ in 0..len {
            idx = (idx + delta + len) % len;
            let u = idx as usize;
            if self.is_item_enabled(u) {
                self.active_index = u;
                return;
            }
        }
    }
}

trait ObserveModelCx<H: UiHost> {
    fn observe_model<T: std::any::Any>(&mut self, model: Model<T>, inv: Invalidation);
}

impl<'a, H: UiHost> ObserveModelCx<H> for LayoutCx<'a, H> {
    fn observe_model<T: std::any::Any>(&mut self, model: Model<T>, inv: Invalidation) {
        LayoutCx::observe_model(self, model, inv)
    }
}

impl<'a, H: UiHost> ObserveModelCx<H> for PaintCx<'a, H> {
    fn observe_model<T: std::any::Any>(&mut self, model: Model<T>, inv: Invalidation) {
        PaintCx::observe_model(self, model, inv)
    }
}

impl<H: UiHost> Widget<H> for ToggleGroup {
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
        self.sync_active_index(cx.app);

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
                    {
                        self.toggle_value(cx.app, item.value.clone());
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

                match key {
                    KeyCode::ArrowLeft => {
                        self.move_active(-1);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    KeyCode::ArrowRight => {
                        self.move_active(1);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    KeyCode::Home => {
                        self.active_index = 0;
                        self.move_active(0);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    KeyCode::End => {
                        self.active_index = self.items.len().saturating_sub(1);
                        self.move_active(0);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    KeyCode::Enter | KeyCode::Space => {
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
                {
                    self.toggle_value(cx.app, item.value.clone());
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
        self.observe_selection_model(cx);
        self.last_bounds = cx.bounds;

        self.prepare_texts(cx);

        let n = self.items.len();
        if n == 0 {
            self.item_bounds.clear();
            return Size::new(Px(0.0), Px(0.0));
        }

        let pad_x = self.resolved.padding_x.0.max(0.0);
        let desired_h = self.resolved.min_height.0.max(0.0);

        let mut item_ws: Vec<f32> = Vec::with_capacity(n);
        for i in 0..n {
            let Some(Some(p)) = self.prepared.get(i) else {
                item_ws.push(0.0);
                continue;
            };
            let w = (p.metrics.size.width.0 + pad_x * 2.0).max(0.0);
            item_ws.push(w);
        }

        let gap = self.resolved.spacing.0.max(0.0);
        let desired_total_w = item_ws.iter().sum::<f32>() + gap * (n.saturating_sub(1) as f32);

        let available_w = cx.available.width.0.max(0.0);
        let total_w = desired_total_w.min(available_w);
        let total_h = desired_h.min(cx.available.height.0.max(0.0));

        if desired_total_w > 0.0 && total_w < desired_total_w && n > 0 {
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

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_style_from_theme(cx.theme());
        self.observe_selection_model(cx);
        self.last_bounds = cx.bounds;
        self.sync_active_index(cx.app);

        if self.disabled {
            self.hovered_index = None;
            self.pressed_index = None;
        }

        let n = self.items.len();
        if n == 0 {
            return;
        }

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
            let selected = self.is_selected(cx.app, &item.value);
            let hovered = self.hovered_index == Some(i);
            let pressed = self.pressed_index == Some(i);

            let (bg, border_color, fg) = if selected {
                (
                    self.resolved.bg_on,
                    self.resolved.border_on,
                    self.resolved.fg_on,
                )
            } else if pressed || hovered {
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
            if !enabled {
                bg.a *= 0.5;
                border_color.a *= 0.5;
                fg = self.resolved.fg_disabled;
                fg.a *= 0.5;
            }

            let border_w = Px(self.resolved.border_width.0.max(0.0));
            let mut border = Edges::all(border_w);
            if self.resolved.spacing == Px(0.0) && i > 0 {
                border.left = Px(0.0);
            }

            let radius = self.resolved.radius;
            let corner_radii = if self.resolved.spacing == Px(0.0) && n > 1 {
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
            } else {
                Corners::all(radius)
            };

            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(0),
                rect,
                background: bg,
                border,
                border_color,
                corner_radii,
            });

            let pad_x = self.resolved.padding_x.0.max(0.0);
            let inner_w = (rect.size.width.0 - pad_x * 2.0).max(0.0);
            let text_x = rect.origin.x.0
                + pad_x
                + ((inner_w - prepared.metrics.size.width.0) * 0.5).max(0.0);

            let inner_h = rect.size.height.0.max(0.0);
            let text_top =
                rect.origin.y.0 + ((inner_h - prepared.metrics.size.height.0) * 0.5).max(0.0);
            let text_y = text_top + prepared.metrics.baseline.0;

            cx.scene.push(SceneOp::Text {
                order: DrawOrder(1),
                origin: Point::new(Px(text_x), Px(text_y)),
                text: prepared.blob,
                color: fg,
            });
        }

        if cx.focus == Some(cx.node)
            && fret_ui::focus_visible::is_focus_visible(cx.app, cx.window)
            && let Some(rect) = self.item_bounds.get(self.active_index).copied()
        {
            let focus_ring = cx.theme().colors.focus_ring;
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(10),
                rect,
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

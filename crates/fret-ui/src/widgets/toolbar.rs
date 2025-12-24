use crate::ThemeSnapshot;
use crate::{
    UiHost,
    widget::{EventCx, Invalidation, LayoutCx, PaintCx, Widget},
};
use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, MouseButton, Point, Px, Rect, SceneOp, Size,
    TextConstraints, TextMetrics, TextStyle, TextWrap,
};
use fret_runtime::CommandId;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ToolbarItem {
    pub label: Arc<str>,
    pub command: CommandId,
    pub selected: bool,
}

impl ToolbarItem {
    pub fn new(label: impl Into<Arc<str>>, command: impl Into<CommandId>) -> Self {
        Self {
            label: label.into(),
            command: command.into(),
            selected: false,
        }
    }

    pub fn with_selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

#[derive(Debug)]
struct PreparedItem {
    selected: bool,
    label: Arc<str>,
    blob: Option<fret_core::TextBlobId>,
    metrics: TextMetrics,
    bounds: Rect,
}

pub struct Toolbar {
    items: Vec<ToolbarItem>,
    prepared: Vec<PreparedItem>,
    pending_release: Vec<fret_core::TextBlobId>,
    prepared_scale_factor_bits: Option<u32>,
    hovered: Option<usize>,
    pressed: Option<usize>,
    style: TextStyle,
    padding_x: Px,
    padding_y: Px,
    gap: Px,
    corner_radius: Px,
    last_theme_revision: Option<u64>,
}

impl Toolbar {
    pub fn new(items: Vec<ToolbarItem>) -> Self {
        Self {
            items,
            prepared: Vec::new(),
            pending_release: Vec::new(),
            prepared_scale_factor_bits: None,
            hovered: None,
            pressed: None,
            style: TextStyle {
                font: fret_core::FontId::default(),
                size: Px(13.0),
            },
            padding_x: Px(10.0),
            padding_y: Px(6.0),
            gap: Px(8.0),
            corner_radius: Px(8.0),
            last_theme_revision: None,
        }
    }

    pub fn set_items(&mut self, items: Vec<ToolbarItem>) {
        self.items = items;
        for item in self.prepared.drain(..) {
            if let Some(blob) = item.blob {
                self.pending_release.push(blob);
            }
        }
        self.hovered = None;
        self.pressed = None;
    }

    fn sync_style_from_theme(&mut self, theme: ThemeSnapshot) {
        if self.last_theme_revision == Some(theme.revision) {
            return;
        }
        self.last_theme_revision = Some(theme.revision);
        self.padding_x = theme.metrics.padding_md;
        self.padding_y = theme.metrics.padding_sm;
        self.gap = theme.metrics.padding_sm;
        self.corner_radius = theme.metrics.radius_md;
    }

    fn item_index_at(&self, pos: Point) -> Option<usize> {
        for (i, item) in self.prepared.iter().enumerate() {
            if item.bounds.contains(pos) {
                return Some(i);
            }
        }
        None
    }
}

impl<H: UiHost> Widget<H> for Toolbar {
    fn cleanup_resources(&mut self, text: &mut dyn fret_core::TextService) {
        for blob in self.pending_release.drain(..) {
            text.release(blob);
        }
        for item in self.prepared.drain(..) {
            if let Some(blob) = item.blob {
                text.release(blob);
            }
        }
        self.prepared_scale_factor_bits = None;
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        self.sync_style_from_theme(cx.theme().snapshot());

        if self.items.is_empty() {
            return;
        }

        let Event::Pointer(pe) = event else {
            return;
        };
        match pe {
            fret_core::PointerEvent::Move { position, .. } => {
                let hovered = self.item_index_at(*position);
                if hovered != self.hovered {
                    self.hovered = hovered;
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                }
            }
            fret_core::PointerEvent::Down {
                position, button, ..
            } => {
                if *button != MouseButton::Left {
                    return;
                }
                let Some(i) = self.item_index_at(*position) else {
                    return;
                };
                self.pressed = Some(i);
                cx.capture_pointer(cx.node);
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
                let pressed = self.pressed.take();
                cx.release_pointer_capture();

                let hovered = self.item_index_at(*position);
                if pressed == hovered
                    && let Some(i) = pressed
                    && let Some(item) = self.items.get(i)
                {
                    cx.dispatch_command(item.command.clone());
                }

                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.sync_style_from_theme(cx.theme().snapshot());

        if self.items.is_empty() {
            self.hovered = None;
            self.pressed = None;
            return Size::new(cx.available.width, Px(0.0));
        }

        let text_constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            scale_factor: cx.scale_factor,
        };

        let prev = std::mem::take(&mut self.prepared);

        let mut max_metrics_h = Px(0.0);
        for item in &self.items {
            let metrics = cx
                .text
                .measure(item.label.as_ref(), self.style, text_constraints);
            max_metrics_h = Px(max_metrics_h.0.max(metrics.size.height.0));
        }

        let row_h = Px(max_metrics_h.0 + self.padding_y.0 * 2.0);
        let height = Px(row_h.0.max(24.0).min(cx.available.height.0));

        let mut x = cx.bounds.origin.x.0 + self.gap.0.max(0.0);
        let y = cx.bounds.origin.y.0;

        for item in &self.items {
            let label = item.label.clone();
            let metrics = cx
                .text
                .measure(label.as_ref(), self.style, text_constraints);
            let w = Px(metrics.size.width.0 + self.padding_x.0 * 2.0);
            let bounds = Rect::new(Point::new(Px(x), Px(y)), Size::new(w, height));
            x += w.0 + self.gap.0.max(0.0);

            let blob = prev.iter().find(|p| p.label == label).and_then(|p| p.blob);
            self.prepared.push(PreparedItem {
                selected: item.selected,
                label,
                blob,
                metrics,
                bounds,
            });
        }

        for item in prev {
            if let Some(blob) = item.blob
                && !self.prepared.iter().any(|p| p.blob == Some(blob))
            {
                self.pending_release.push(blob);
            }
        }

        let end_x = (x - self.gap.0.max(0.0)).max(cx.bounds.origin.x.0);
        let width = Px((end_x - cx.bounds.origin.x.0)
            .min(cx.available.width.0)
            .max(0.0));
        Size::new(width, height)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        for blob in self.pending_release.drain(..) {
            cx.text.release(blob);
        }

        if self.items.is_empty() || cx.bounds.size.height.0 <= 0.0 {
            return;
        }

        let theme = cx.theme().snapshot();
        self.sync_style_from_theme(theme);

        let scale_bits = cx.scale_factor.to_bits();
        if self.prepared_scale_factor_bits != Some(scale_bits) {
            for item in &mut self.prepared {
                if let Some(blob) = item.blob.take() {
                    cx.text.release(blob);
                }
            }
            self.prepared_scale_factor_bits = Some(scale_bits);
        }

        let text_constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            scale_factor: cx.scale_factor,
        };

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: theme.colors.panel_background,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(Px(0.0)),
        });

        for (i, item) in self.prepared.iter_mut().enumerate() {
            if item.blob.is_none() {
                let (blob, metrics) =
                    cx.text
                        .prepare(item.label.as_ref(), self.style, text_constraints);
                item.blob = Some(blob);
                item.metrics = metrics;
            }

            let hovered = self.hovered == Some(i);
            let pressed = self.pressed == Some(i);

            let bg = if item.selected {
                theme.colors.selection_background
            } else if pressed || hovered {
                theme.colors.hover_background
            } else {
                Color {
                    a: 0.0,
                    ..theme.colors.panel_background
                }
            };

            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(1),
                rect: item.bounds,
                background: bg,
                border: Edges::all(Px(1.0)),
                border_color: theme.colors.panel_border,
                corner_radii: Corners::all(self.corner_radius),
            });

            let text_x = Px(item.bounds.origin.x.0 + self.padding_x.0);
            let inner_y = item.bounds.origin.y.0
                + ((item.bounds.size.height.0 - item.metrics.size.height.0) * 0.5);
            let text_y = Px(inner_y + item.metrics.baseline.0);
            let Some(blob) = item.blob else { continue };
            cx.scene.push(SceneOp::Text {
                order: DrawOrder(2),
                origin: Point::new(text_x, text_y),
                text: blob,
                color: theme.colors.text_primary,
            });
        }
    }
}

use std::sync::Arc;

use fret_core::{
    Color, Corners, CursorIcon, DrawOrder, Edges, Event, MouseButton, Point, Px, Rect, SceneOp,
    SemanticsRole, Size, TextConstraints, TextMetrics, TextStyle, TextWrap,
};
use fret_runtime::Menu;
use fret_ui::{EventCx, Invalidation, LayoutCx, PaintCx, UiHost, Widget};

use crate::style::{
    ColorFallback, MetricFallback, StyleRefinement, component_color, component_metric,
};
use crate::{Sizable, Size as ComponentSize};

#[derive(Debug, Clone)]
struct PreparedText {
    blob: fret_core::TextBlobId,
    metrics: TextMetrics,
}

pub struct DropdownMenuButton {
    label: Arc<str>,
    menu: Menu,
    style: StyleRefinement,
    size: ComponentSize,
    hovered: bool,
    pressed: bool,
    last_bounds: Rect,
    last_theme_revision: Option<u64>,
    label_prepared: Option<PreparedText>,
    chevron_prepared: Option<PreparedText>,
    prepared_scale_factor_bits: Option<u32>,
    prepared_theme_revision: Option<u64>,
    resolved: ResolvedStyle,
}

#[derive(Debug, Clone)]
struct ResolvedStyle {
    padding_x: Px,
    padding_y: Px,
    min_height: Px,
    radius: Px,
    border_width: Px,
    bg: Color,
    bg_hover: Color,
    bg_active: Color,
    border: Color,
    text: Color,
    text_size: Px,
}

impl Default for ResolvedStyle {
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
            text: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            text_size: Px(13.0),
        }
    }
}

impl DropdownMenuButton {
    pub fn new(label: impl Into<Arc<str>>, menu: Menu) -> Self {
        Self {
            label: label.into(),
            menu,
            style: StyleRefinement::default(),
            size: ComponentSize::Medium,
            hovered: false,
            pressed: false,
            last_bounds: Rect::default(),
            last_theme_revision: None,
            label_prepared: None,
            chevron_prepared: None,
            prepared_scale_factor_bits: None,
            prepared_theme_revision: None,
            resolved: ResolvedStyle::default(),
        }
    }

    pub fn with_size(mut self, size: ComponentSize) -> Self {
        self.size = size;
        self.last_theme_revision = None;
        self.prepared_scale_factor_bits = None;
        self.prepared_theme_revision = None;
        self
    }

    pub fn refine_style(mut self, style: StyleRefinement) -> Self {
        self.style = style;
        self.last_theme_revision = None;
        self.prepared_scale_factor_bits = None;
        self.prepared_theme_revision = None;
        self
    }

    fn sync_style_from_theme(&mut self, theme: &fret_ui::Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        let padding_x = self
            .style
            .padding_x
            .clone()
            .unwrap_or_else(|| {
                component_metric(
                    "component.dropdown_menu.padding_x",
                    MetricFallback::Px(self.size.button_px(theme)),
                )
            })
            .resolve(theme);
        let padding_y = self
            .style
            .padding_y
            .clone()
            .unwrap_or_else(|| {
                component_metric(
                    "component.dropdown_menu.padding_y",
                    MetricFallback::Px(self.size.button_py(theme)),
                )
            })
            .resolve(theme);
        let min_height = self
            .style
            .min_height
            .clone()
            .unwrap_or_else(|| {
                component_metric(
                    "component.dropdown_menu.min_height",
                    MetricFallback::Px(self.size.button_h(theme)),
                )
            })
            .resolve(theme);
        let radius = self
            .style
            .radius
            .clone()
            .unwrap_or_else(|| {
                component_metric(
                    "component.dropdown_menu.radius",
                    MetricFallback::ThemeRadiusMd,
                )
            })
            .resolve(theme);
        let border_width = self
            .style
            .border_width
            .clone()
            .unwrap_or_else(|| {
                component_metric(
                    "component.dropdown_menu.border_width",
                    MetricFallback::Px(Px(1.0)),
                )
            })
            .resolve(theme);

        let bg = self
            .style
            .background
            .clone()
            .unwrap_or(component_color(
                "component.dropdown_menu.bg",
                ColorFallback::ThemePanelBackground,
            ))
            .resolve(theme);
        let bg_hover = component_color(
            "component.dropdown_menu.bg_hover",
            ColorFallback::ThemeHoverBackground,
        )
        .resolve(theme);
        let bg_active = component_color(
            "component.dropdown_menu.bg_active",
            ColorFallback::ThemeSelectionBackground,
        )
        .resolve(theme);

        let border = self
            .style
            .border_color
            .clone()
            .unwrap_or(component_color(
                "component.dropdown_menu.border",
                ColorFallback::ThemePanelBorder,
            ))
            .resolve(theme);
        let text = self
            .style
            .text_color
            .clone()
            .unwrap_or(component_color(
                "component.dropdown_menu.text",
                ColorFallback::ThemeTextPrimary,
            ))
            .resolve(theme);

        let text_size = theme
            .metric_by_key("component.dropdown_menu.text_px")
            .unwrap_or_else(|| self.size.control_text_px(theme));
        self.resolved = ResolvedStyle {
            padding_x,
            padding_y,
            min_height,
            radius,
            border_width,
            bg,
            bg_hover,
            bg_active,
            border,
            text,
            text_size,
        };

        self.prepared_scale_factor_bits = None;
        self.prepared_theme_revision = None;
    }

    fn close_menu<H: UiHost>(&self, cx: &mut EventCx<'_, H>) {
        cx.dispatch_command(fret_runtime::CommandId::from("context_menu.close"));
    }

    fn open_menu<H: UiHost>(&self, cx: &mut EventCx<'_, H>) {
        let position = Point::new(
            self.last_bounds.origin.x,
            Px(self.last_bounds.origin.y.0 + self.last_bounds.size.height.0 + 2.0),
        );
        cx.open_context_menu(position, self.menu.clone());
    }

    fn ensure_prepared<H: UiHost>(&mut self, cx: &mut PaintCx<'_, H>) {
        let scale_bits = cx.scale_factor.to_bits();
        let theme_rev = cx.theme().revision();
        if self.prepared_scale_factor_bits == Some(scale_bits)
            && self.prepared_theme_revision == Some(theme_rev)
            && self.label_prepared.is_some()
            && self.chevron_prepared.is_some()
        {
            return;
        }

        self.cleanup_prepared(cx.text);
        self.prepared_scale_factor_bits = Some(scale_bits);
        self.prepared_theme_revision = Some(theme_rev);

        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            scale_factor: cx.scale_factor,
        };

        let style = TextStyle {
            font: fret_core::FontId::default(),
            size: self.resolved.text_size,
        };

        let (label_blob, label_metrics) = cx.text.prepare(self.label.as_ref(), style, constraints);
        let (chev_blob, chev_metrics) = cx.text.prepare("▾", style, constraints);

        self.label_prepared = Some(PreparedText {
            blob: label_blob,
            metrics: label_metrics,
        });
        self.chevron_prepared = Some(PreparedText {
            blob: chev_blob,
            metrics: chev_metrics,
        });
    }

    fn cleanup_prepared(&mut self, text: &mut dyn fret_core::TextService) {
        if let Some(p) = self.label_prepared.take() {
            text.release(p.blob);
        }
        if let Some(p) = self.chevron_prepared.take() {
            text.release(p.blob);
        }
        self.prepared_scale_factor_bits = None;
        self.prepared_theme_revision = None;
    }
}

impl Sizable for DropdownMenuButton {
    fn with_size(self, size: ComponentSize) -> Self {
        DropdownMenuButton::with_size(self, size)
    }
}

impl<H: UiHost> Widget<H> for DropdownMenuButton {
    fn cleanup_resources(&mut self, text: &mut dyn fret_core::TextService) {
        self.cleanup_prepared(text);
    }

    fn is_focusable(&self) -> bool {
        true
    }

    fn semantics(&mut self, cx: &mut fret_ui::widget::SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Button);
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
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
                cx.release_pointer_capture();
                let was_pressed = self.pressed;
                self.pressed = false;
                let hovered = self.last_bounds.contains(*position);
                self.hovered = hovered;
                if was_pressed && hovered {
                    self.open_menu(cx);
                } else if was_pressed {
                    self.close_menu(cx);
                }
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.last_bounds = cx.bounds;
        self.sync_style_from_theme(cx.theme());

        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            scale_factor: cx.scale_factor,
        };
        let style = TextStyle {
            font: fret_core::FontId::default(),
            size: self.resolved.text_size,
        };
        let label_metrics = cx.text.measure(self.label.as_ref(), style, constraints);
        let chevron_metrics = cx.text.measure("▾", style, constraints);

        let pad_y = self.resolved.padding_y.0.max(0.0);
        let content_h = label_metrics
            .size
            .height
            .0
            .max(chevron_metrics.size.height.0);
        let h = (content_h + pad_y * 2.0).max(self.resolved.min_height.0.max(0.0));

        Size::new(cx.available.width, Px(h))
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_style_from_theme(cx.theme());
        self.ensure_prepared(cx);

        let bg = if self.pressed {
            self.resolved.bg_active
        } else if self.hovered {
            self.resolved.bg_hover
        } else {
            self.resolved.bg
        };

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: bg,
            border: Edges::all(self.resolved.border_width),
            border_color: self.resolved.border,
            corner_radii: Corners::all(self.resolved.radius),
        });

        let Some(label) = self.label_prepared.as_ref() else {
            return;
        };
        let Some(chev) = self.chevron_prepared.as_ref() else {
            return;
        };

        let inner_w = (cx.bounds.size.width.0 - self.resolved.padding_x.0 * 2.0).max(0.0);
        let center_y = cx.bounds.origin.y.0 + cx.bounds.size.height.0 * 0.5;

        let label_y = center_y - label.metrics.size.height.0 * 0.5 + label.metrics.baseline.0;
        let x = cx.bounds.origin.x.0 + self.resolved.padding_x.0;

        let chev_x = cx.bounds.origin.x.0
            + self.resolved.padding_x.0
            + (inner_w - chev.metrics.size.width.0).max(0.0);
        let chev_y = center_y - chev.metrics.size.height.0 * 0.5 + chev.metrics.baseline.0;

        cx.scene.push(SceneOp::Text {
            order: DrawOrder(1),
            origin: Point::new(Px(x), Px(label_y)),
            text: label.blob,
            color: self.resolved.text,
        });
        cx.scene.push(SceneOp::Text {
            order: DrawOrder(1),
            origin: Point::new(Px(chev_x), Px(chev_y)),
            text: chev.blob,
            color: self.resolved.text,
        });
    }
}

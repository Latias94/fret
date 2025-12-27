use crate::{ContextMenuRequest, ContextMenuService, MenuBarContextMenu, MenuBarContextMenuEntry};
use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, MouseButton, Point, Px, Rect, SceneOp, SemanticsRole,
    Size, TextConstraints, TextMetrics, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::{CommandId, InputContext, Menu, MenuBar};
use fret_ui::{
    ThemeSnapshot, UiHost,
    widget::{EventCx, Invalidation, LayoutCx, PaintCx, SemanticsCx, Widget},
};

#[derive(Debug)]
struct PreparedMenu {
    index: usize,
    title: std::sync::Arc<str>,
    blob: Option<fret_core::TextBlobId>,
    metrics: TextMetrics,
    bounds: Rect,
}

pub struct AppMenuBar {
    menu_bar: MenuBar,
    prepared: Vec<PreparedMenu>,
    pending_release: Vec<fret_core::TextBlobId>,
    prepared_scale_factor_bits: Option<u32>,
    hovered: Option<usize>,
    open_index: Option<usize>,
    open_serial: Option<u64>,
    style: TextStyle,
    padding_x: Px,
    padding_y: Px,
    gap: Px,
    corner_radius: Px,
    height: Px,
    last_theme_revision: Option<u64>,
}

impl AppMenuBar {
    pub fn new(menu_bar: MenuBar) -> Self {
        Self {
            menu_bar,
            prepared: Vec::new(),
            pending_release: Vec::new(),
            prepared_scale_factor_bits: None,
            hovered: None,
            open_index: None,
            open_serial: None,
            style: TextStyle {
                font: fret_core::FontId::default(),
                size: Px(13.0),
                ..Default::default()
            },
            padding_x: Px(10.0),
            padding_y: Px(6.0),
            gap: Px(6.0),
            corner_radius: Px(8.0),
            height: Px(30.0),
            last_theme_revision: None,
        }
    }

    pub fn set_menu_bar(&mut self, menu_bar: MenuBar) {
        self.menu_bar = menu_bar;
        for item in self.prepared.drain(..) {
            if let Some(blob) = item.blob {
                self.pending_release.push(blob);
            }
        }
        self.hovered = None;
        self.open_index = None;
        self.open_serial = None;
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
        self.style.size = theme.metrics.font_size;
        let computed_h = theme.metrics.font_size + self.padding_y * 2.0;
        self.height = Px(30.0_f32.max(computed_h.0));
    }

    fn current_menu_serial<H: UiHost>(app: &H, window: fret_core::AppWindowId) -> Option<u64> {
        app.global::<ContextMenuService>()
            .and_then(|s| s.request(window))
            .map(|(serial, _)| serial)
    }

    fn sync_open_state<H: UiHost>(&mut self, app: &H, window: fret_core::AppWindowId) {
        let current = app
            .global::<ContextMenuService>()
            .and_then(|s| s.request(window));
        let Some((serial, request)) = current else {
            self.open_serial = None;
            self.open_index = None;
            return;
        };
        let Some(menu_bar) = request.menu_bar.as_ref() else {
            self.open_serial = None;
            self.open_index = None;
            return;
        };
        self.open_serial = Some(serial);
        self.open_index = Some(menu_bar.open_index);
    }

    fn menu_index_at(&self, pos: Point) -> Option<usize> {
        for m in &self.prepared {
            if m.bounds.contains(pos) {
                return Some(m.index);
            }
        }
        None
    }

    fn open_menu<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        window: fret_core::AppWindowId,
        index: usize,
    ) {
        let Some(prepared) = self.prepared.iter().find(|m| m.index == index) else {
            return;
        };
        let Some(menu) = self.menu_bar.menus.get(index) else {
            return;
        };

        let position = Point::new(
            prepared.bounds.origin.x,
            Px(prepared.bounds.origin.y.0 + prepared.bounds.size.height.0 + 2.0),
        );

        let inv_ctx = InputContext {
            platform: cx.input_ctx.platform,
            caps: cx.input_ctx.caps.clone(),
            ui_has_modal: cx.input_ctx.ui_has_modal,
            focus_is_text_input: cx.input_ctx.focus_is_text_input,
        };

        cx.app
            .with_global_mut(ContextMenuService::default, |service, _app| {
                let entries: Vec<MenuBarContextMenuEntry> = self
                    .prepared
                    .iter()
                    .filter_map(|m| {
                        self.menu_bar
                            .menus
                            .get(m.index)
                            .map(|menu| MenuBarContextMenuEntry {
                                index: m.index,
                                bounds: m.bounds,
                                menu: Menu {
                                    title: menu.title.clone(),
                                    items: menu.items.clone(),
                                },
                            })
                    })
                    .collect();

                service.set_request(
                    window,
                    ContextMenuRequest {
                        position,
                        menu: Menu {
                            title: menu.title.clone(),
                            items: menu.items.clone(),
                        },
                        input_ctx: inv_ctx,
                        menu_bar: Some(MenuBarContextMenu {
                            open_index: index,
                            entries,
                        }),
                    },
                );
            });

        self.open_index = Some(index);
        self.open_serial = Self::current_menu_serial(&*cx.app, window);

        cx.dispatch_command(CommandId::from("context_menu.open"));
        cx.stop_propagation();
    }

    fn close_menu<H: UiHost>(&mut self, cx: &mut EventCx<'_, H>) {
        self.open_index = None;
        self.open_serial = None;
        cx.dispatch_command(CommandId::from("context_menu.close"));
        cx.stop_propagation();
    }
}

impl<H: UiHost> Widget<H> for AppMenuBar {
    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        for blob in self.pending_release.drain(..) {
            services.text().release(blob);
        }
        for item in self.prepared.drain(..) {
            if let Some(blob) = item.blob {
                services.text().release(blob);
            }
        }
        self.prepared_scale_factor_bits = None;
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::MenuBar);
        cx.set_label("Menu bar");
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        self.sync_style_from_theme(cx.theme().snapshot());
        let Some(window) = cx.window else {
            return;
        };

        self.sync_open_state(cx.app, window);

        let Event::Pointer(pe) = event else {
            return;
        };
        match pe {
            fret_core::PointerEvent::Move { position, .. } => {
                let hovered = self.menu_index_at(*position);
                if hovered != self.hovered {
                    self.hovered = hovered;
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                }

                if let (Some(open), Some(h)) = (self.open_index, hovered)
                    && open != h
                {
                    self.open_menu(cx, window, h);
                }
            }
            fret_core::PointerEvent::Down {
                position, button, ..
            } => {
                if *button != MouseButton::Left {
                    return;
                }
                let Some(i) = self.menu_index_at(*position) else {
                    return;
                };

                if self.open_index == Some(i) {
                    self.close_menu(cx);
                    return;
                }

                self.open_menu(cx, window, i);
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.sync_style_from_theme(cx.theme().snapshot());
        if let Some(window) = cx.window {
            self.sync_open_state(cx.app, window);
        }

        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };

        let prev = std::mem::take(&mut self.prepared);

        let mut max_metrics_h = Px(0.0);
        for menu in &self.menu_bar.menus {
            let metrics = cx
                .services
                .text()
                .measure(menu.title.as_ref(), self.style, constraints);
            max_metrics_h = Px(max_metrics_h.0.max(metrics.size.height.0));
        }

        let row_h = Px(max_metrics_h.0 + self.padding_y.0 * 2.0);
        self.height = Px(row_h.0.max(24.0).min(cx.available.height.0));

        let y = cx.bounds.origin.y.0;
        let mut x = cx.bounds.origin.x.0 + self.gap.0.max(0.0);

        for (index, menu) in self.menu_bar.menus.iter().enumerate() {
            let title: std::sync::Arc<str> = menu.title.clone();
            let metrics = cx
                .services
                .text()
                .measure(title.as_ref(), self.style, constraints);
            let w = Px(metrics.size.width.0 + self.padding_x.0 * 2.0);
            let bounds = Rect::new(Point::new(Px(x), Px(y)), Size::new(w, row_h));
            x += w.0 + self.gap.0.max(0.0);

            let blob = prev
                .iter()
                .find(|m| m.index == index && m.title == title)
                .and_then(|m| m.blob);
            self.prepared.push(PreparedMenu {
                index,
                title,
                blob,
                metrics,
                bounds,
            });
        }

        for item in prev {
            if let Some(blob) = item.blob
                && !self.prepared.iter().any(|m| m.blob == Some(blob))
            {
                self.pending_release.push(blob);
            }
        }

        Size::new(cx.available.width, self.height)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        for blob in self.pending_release.drain(..) {
            cx.services.text().release(blob);
        }

        let theme = cx.theme().snapshot();
        self.sync_style_from_theme(theme);
        let Some(window) = cx.window else {
            return;
        };

        self.sync_open_state(cx.app, window);

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: theme.colors.panel_background,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(Px(0.0)),
        });

        let scale_bits = cx.scale_factor.to_bits();
        if self.prepared_scale_factor_bits != Some(scale_bits) {
            for item in &mut self.prepared {
                if let Some(blob) = item.blob.take() {
                    cx.services.text().release(blob);
                }
            }
            self.prepared_scale_factor_bits = Some(scale_bits);
        }

        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };

        for item in &mut self.prepared {
            let blob = match item.blob {
                Some(b) => b,
                None => {
                    // Paint-time preparation ensures compatibility with subtree replay caching.
                    let (b, m) =
                        cx.services
                            .text()
                            .prepare(item.title.as_ref(), self.style, constraints);
                    item.blob = Some(b);
                    item.metrics = m;
                    b
                }
            };

            let hovered = self.hovered == Some(item.index);
            let active = self.open_index == Some(item.index);

            let bg = if active {
                theme.colors.selection_background
            } else if hovered {
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
            cx.scene.push(SceneOp::Text {
                order: DrawOrder(2),
                origin: Point::new(text_x, text_y),
                text: blob,
                color: theme.colors.text_primary,
            });
        }
    }
}

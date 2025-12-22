use crate::ThemeSnapshot;
use crate::{
    UiHost,
    widget::{EventCx, Invalidation, LayoutCx, PaintCx, Widget},
};
use fret_app::{InputContext, Menu, MenuBar};
use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, MouseButton, Point, Px, Rect, SceneOp, Size,
    TextConstraints, TextMetrics, TextStyle, TextWrap,
};

use super::context_menu::{
    ContextMenuRequest, ContextMenuService, MenuBarContextMenu, MenuBarContextMenuEntry,
};

#[derive(Debug)]
struct PreparedMenu {
    index: usize,
    blob: fret_core::TextBlobId,
    metrics: TextMetrics,
    bounds: Rect,
}

pub struct AppMenuBar {
    menu_bar: MenuBar,
    prepared: Vec<PreparedMenu>,
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
            hovered: None,
            open_index: None,
            open_serial: None,
            style: TextStyle {
                font: fret_core::FontId::default(),
                size: Px(13.0),
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
        self.prepared.clear();
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

        cx.dispatch_command(fret_app::CommandId::from("context_menu.open"));
        cx.stop_propagation();
    }

    fn close_menu<H: UiHost>(&mut self, cx: &mut EventCx<'_, H>) {
        self.open_index = None;
        self.open_serial = None;
        cx.dispatch_command(fret_app::CommandId::from("context_menu.close"));
        cx.stop_propagation();
    }
}

impl<H: UiHost> Widget<H> for AppMenuBar {
    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        self.sync_style_from_theme(cx.theme().snapshot());
        let Some(window) = cx.window else {
            return;
        };

        self.sync_open_state(cx.app, window);

        match event {
            Event::Pointer(pe) => match pe {
                fret_core::PointerEvent::Move { position, .. } => {
                    let hovered = self.menu_index_at(*position);
                    if hovered != self.hovered {
                        self.hovered = hovered;
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                    }

                    if let (Some(open), Some(h)) = (self.open_index, hovered) {
                        if open != h {
                            self.open_menu(cx, window, h);
                        }
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
            },
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.sync_style_from_theme(cx.theme().snapshot());
        if let Some(window) = cx.window {
            self.sync_open_state(cx.app, window);
        }

        for item in self.prepared.drain(..) {
            cx.text.release(item.blob);
        }

        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            scale_factor: cx.scale_factor,
        };

        let mut max_metrics_h = Px(0.0);
        for menu in &self.menu_bar.menus {
            let (_, metrics) = cx
                .text
                .prepare(menu.title.as_ref(), self.style, constraints);
            max_metrics_h = Px(max_metrics_h.0.max(metrics.size.height.0));
        }

        let row_h = Px(max_metrics_h.0 + self.padding_y.0 * 2.0);
        self.height = Px(row_h.0.max(24.0).min(cx.available.height.0));

        let y = cx.bounds.origin.y.0;
        let mut x = cx.bounds.origin.x.0 + self.gap.0.max(0.0);

        for (index, menu) in self.menu_bar.menus.iter().enumerate() {
            let (blob, metrics) = cx
                .text
                .prepare(menu.title.as_ref(), self.style, constraints);
            let w = Px(metrics.size.width.0 + self.padding_x.0 * 2.0);
            let bounds = Rect::new(Point::new(Px(x), Px(y)), Size::new(w, row_h));
            x += w.0 + self.gap.0.max(0.0);

            self.prepared.push(PreparedMenu {
                index,
                blob,
                metrics,
                bounds,
            });
        }

        Size::new(cx.available.width, self.height)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
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

        for item in &self.prepared {
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
                text: item.blob,
                color: theme.colors.text_primary,
            });
        }
    }
}

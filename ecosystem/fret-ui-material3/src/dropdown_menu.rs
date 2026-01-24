//! Material 3 dropdown menu (overlay MVP).
//!
//! This is an outcome-oriented wrapper:
//! - anchors a `Menu` panel to a trigger element using the shared overlay controller,
//! - provides menu-like dismissal (Escape / outside press, non click-through),
//! - best-effort initial focus on the first enabled item.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{Edges, Px, Rect, Size};
use fret_runtime::Model;
use fret_ui::action::OnDismissRequest;
use fret_ui::element::{AnyElement, Overflow};
use fret_ui::elements::GlobalElementId;
use fret_ui::overlay_placement::{Align, Side};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::overlay;
use fret_ui_kit::overlay_controller;
use fret_ui_kit::primitives::direction as direction_prim;
use fret_ui_kit::primitives::popper;
use fret_ui_kit::primitives::popper_content;
use fret_ui_kit::{OverlayController, OverlayPresence};

use crate::menu::{Menu, MenuEntry};
use crate::motion::ms_to_frames;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DropdownMenuAlign {
    #[default]
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DropdownMenuSide {
    Top,
    Right,
    #[default]
    Bottom,
    Left,
}

#[derive(Clone)]
pub struct DropdownMenu {
    open: Model<bool>,
    align: DropdownMenuAlign,
    align_offset: Px,
    side: DropdownMenuSide,
    side_offset: Px,
    window_margin: Px,
    min_width: Px,
    close_on_select: bool,
    on_dismiss_request: Option<OnDismissRequest>,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for DropdownMenu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DropdownMenu")
            .field("open", &"<model>")
            .field("align", &self.align)
            .field("side", &self.side)
            .field("side_offset", &self.side_offset)
            .field("window_margin", &self.window_margin)
            .field("min_width", &self.min_width)
            .field("close_on_select", &self.close_on_select)
            .field("on_dismiss_request", &self.on_dismiss_request.is_some())
            .finish()
    }
}

impl DropdownMenu {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            open,
            align: DropdownMenuAlign::default(),
            align_offset: Px(0.0),
            side: DropdownMenuSide::default(),
            side_offset: Px(4.0),
            window_margin: Px(0.0),
            min_width: Px(128.0),
            close_on_select: true,
            on_dismiss_request: None,
            a11y_label: None,
            test_id: None,
        }
    }

    pub fn align(mut self, align: DropdownMenuAlign) -> Self {
        self.align = align;
        self
    }

    pub fn align_offset(mut self, offset: Px) -> Self {
        self.align_offset = offset;
        self
    }

    pub fn side(mut self, side: DropdownMenuSide) -> Self {
        self.side = side;
        self
    }

    pub fn side_offset(mut self, offset: Px) -> Self {
        self.side_offset = offset;
        self
    }

    pub fn window_margin(mut self, margin: Px) -> Self {
        self.window_margin = margin;
        self
    }

    pub fn min_width(mut self, min_width: Px) -> Self {
        self.min_width = min_width;
        self
    }

    pub fn close_on_select(mut self, close: bool) -> Self {
        self.close_on_select = close;
        self
    }

    pub fn on_dismiss_request(mut self, on_dismiss_request: Option<OnDismissRequest>) -> Self {
        self.on_dismiss_request = on_dismiss_request;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        entries: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<MenuEntry>,
    ) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();

            let is_open = cx
                .get_model_copied(&self.open, Invalidation::Layout)
                .unwrap_or(false);

            let open_ticks = ms_to_frames(
                theme.duration_ms_by_key("md.sys.motion.duration.short4")
                    .unwrap_or(200),
            );
            let close_ticks = ms_to_frames(
                theme.duration_ms_by_key("md.sys.motion.duration.short2")
                    .unwrap_or(100),
            );
            let easing = theme
                .easing_by_key("md.sys.motion.easing.emphasized")
                .or_else(|| theme.easing_by_key("md.sys.motion.easing.standard"))
                .unwrap_or(fret_ui::theme::CubicBezier {
                    x1: 0.0,
                    y1: 0.0,
                    x2: 1.0,
                    y2: 1.0,
                });
            let motion = OverlayController::transition_with_durations_and_cubic_bezier(
                cx,
                is_open,
                open_ticks,
                close_ticks,
                easing,
            );
            let overlay_presence = OverlayPresence {
                present: motion.present,
                interactive: is_open,
            };
            let trigger = trigger(cx);
            let trigger_id = trigger.id;

            if overlay_presence.present {
                let direction = direction_prim::use_direction_in_scope(cx, None);

                let Some(anchor) = overlay::anchor_bounds_for_element(cx, trigger_id) else {
                    return trigger;
                };
                let outer = overlay::outer_bounds_with_window_margin(cx.bounds, self.window_margin);

                let menu_item_height = theme
                    .metric_by_key("md.comp.menu.list-item.container.height")
                    .unwrap_or(Px(48.0));
                let divider_height = theme
                    .metric_by_key("md.comp.menu.divider.height")
                    .unwrap_or(Px(1.0));
                let divider_margin = Px(8.0);

                let mut menu_entries = entries(cx);
                if self.close_on_select {
                    menu_entries = wrap_close_on_select(menu_entries, self.open.clone());
                }

                let estimated = estimated_menu_panel_size(
                    anchor,
                    self.min_width,
                    &menu_entries,
                    menu_item_height,
                    divider_height,
                    divider_margin,
                );

                let align = match self.align {
                    DropdownMenuAlign::Start => Align::Start,
                    DropdownMenuAlign::Center => Align::Center,
                    DropdownMenuAlign::End => Align::End,
                };
                let side = match self.side {
                    DropdownMenuSide::Top => Side::Top,
                    DropdownMenuSide::Right => Side::Right,
                    DropdownMenuSide::Bottom => Side::Bottom,
                    DropdownMenuSide::Left => Side::Left,
                };

                let placement =
                    popper::PopperContentPlacement::new(direction, side, align, self.side_offset)
                        .with_align_offset(self.align_offset)
                        .with_collision_padding(Edges::all(Px(8.0)));
                let layout =
                    popper::popper_content_layout_sized(outer, anchor, estimated, placement);

                let initial_focus_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
                let initial_focus_id_for_menu = initial_focus_id.clone();

                let a11y_label = self
                    .a11y_label
                    .clone()
                    .unwrap_or_else(|| Arc::<str>::from("Menu"));
                let test_id = self.test_id.clone().unwrap_or_else(|| {
                    Arc::<str>::from(format!("material3-menu-{}", trigger_id.0))
                });

                let overlay_root = popper_content::popper_wrapper_panel_at(
                    cx,
                    layout.rect,
                    Edges::all(Px(0.0)),
                    Overflow::Visible,
                    move |cx| {
                        vec![
                            Menu::new()
                                .a11y_label(a11y_label)
                                .test_id(test_id)
                                .entries(menu_entries)
                                .into_element_with_initial_focus_id(cx, initial_focus_id_for_menu),
                        ]
                    },
                );

                let opacity = motion.progress;
                let scale = 0.95 + 0.05 * motion.progress;
                let origin = popper::popper_content_transform_origin(&layout, anchor, None);
                let origin_inv = fret_core::Point::new(Px(-origin.x.0), Px(-origin.y.0));
                let transform = fret_core::Transform2D::translation(origin)
                    * fret_core::Transform2D::scale_uniform(scale)
                    * fret_core::Transform2D::translation(origin_inv);
                let overlay_root =
                    fret_ui_kit::declarative::overlay_motion::wrap_opacity_and_render_transform_gated(
                        cx,
                        opacity,
                        transform,
                        overlay_presence.interactive,
                        vec![overlay_root],
                    );

                let mut request = overlay_controller::OverlayRequest::dismissible_menu(
                    trigger_id,
                    trigger_id,
                    self.open.clone(),
                    overlay_presence,
                    vec![overlay_root],
                );
                request.root_name = Some(format!("material3.dropdown_menu.{}", trigger_id.0));
                request.close_on_window_focus_lost = true;
                request.close_on_window_resize = true;
                request.dismissible_on_dismiss_request = self.on_dismiss_request.clone();
                request.initial_focus = initial_focus_id.get();

                OverlayController::request(cx, request);
            }

            trigger
        })
    }
}

fn wrap_close_on_select(entries: Vec<MenuEntry>, open: Model<bool>) -> Vec<MenuEntry> {
    entries
        .into_iter()
        .map(|e| match e {
            MenuEntry::Separator => MenuEntry::Separator,
            MenuEntry::Item(mut item) => {
                if item.disabled {
                    return MenuEntry::Item(item);
                }
                let Some(prev) = item.on_select.clone() else {
                    return MenuEntry::Item(item);
                };
                let open = open.clone();
                item.on_select = Some(Arc::new(move |host, cx, reason| {
                    prev(host, cx, reason);
                    let _ = host.models_mut().update(&open, |v| *v = false);
                    host.request_redraw(cx.window);
                }));
                MenuEntry::Item(item)
            }
        })
        .collect()
}

fn estimated_menu_panel_size(
    anchor: Rect,
    min_width: Px,
    entries: &[MenuEntry],
    item_height: Px,
    divider_height: Px,
    divider_margin_total: Px,
) -> Size {
    let mut h = 0.0f32;
    for e in entries {
        match e {
            MenuEntry::Item(_) => h += item_height.0.max(0.0),
            MenuEntry::Separator => {
                h += divider_height.0.max(0.0) + divider_margin_total.0.max(0.0)
            }
        }
    }

    let w = anchor.size.width.0.max(min_width.0).max(0.0);
    Size::new(Px(w), Px(h.max(1.0)))
}

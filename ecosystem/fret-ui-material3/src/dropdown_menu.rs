//! Material 3 dropdown menu (overlay MVP).
//!
//! This is an outcome-oriented wrapper:
//! - anchors a `Menu` panel to a trigger element using the shared overlay controller,
//! - provides menu-like dismissal (Escape / outside press, non click-through),
//! - best-effort initial focus on the first enabled item.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::OnceLock;

use fret_core::{Edges, Px, Rect, Size};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, LayoutStyle, Length, Overflow, ScrollAxis, ScrollProps};
use fret_ui::elements::GlobalElementId;
use fret_ui::overlay_placement::{Align, Side};
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::{controllable_state, model_watch::ModelWatchExt as _};
use fret_ui_kit::overlay;
use fret_ui_kit::overlay_controller;
use fret_ui_kit::primitives::dismissable_layer::OnDismissRequest;
use fret_ui_kit::primitives::menu as menu_primitive;
use fret_ui_kit::primitives::popper;
use fret_ui_kit::primitives::popper_content;
use fret_ui_kit::primitives::portal_inherited;
use fret_ui_kit::{OverlayController, OverlayPresence, WidgetStateProperty};

use crate::foundation::context::material_layout_direction_in_scope;
use crate::foundation::overlay_motion::drive_overlay_open_close_motion;
use crate::foundation::test_id::part_test_id;
use crate::menu::{
    MaterialMenuSubmenuContext, Menu, MenuEntry, MenuStyle, material_menu_submenu_panel_tree,
    menu_submenu_entries_by_value,
};
use crate::motion::ms_to_frames;
use crate::tokens::dropdown_menu as dropdown_menu_tokens;
use crate::tokens::menu as menu_tokens;

fn default_dropdown_menu_a11y_label() -> Arc<str> {
    static LABEL: OnceLock<Arc<str>> = OnceLock::new();
    LABEL.get_or_init(|| Arc::<str>::from("Menu")).clone()
}

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
    submenu_min_width: Px,
    max_height: Option<Px>,
    close_on_select: bool,
    on_dismiss_request: Option<OnDismissRequest>,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    menu_style: MenuStyle,
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
            .field("submenu_min_width", &self.submenu_min_width)
            .field("max_height", &self.max_height)
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
            min_width: menu_tokens::ITEM_MIN_WIDTH_FALLBACK,
            submenu_min_width: menu_tokens::ITEM_MIN_WIDTH_FALLBACK,
            max_height: None,
            close_on_select: true,
            on_dismiss_request: None,
            a11y_label: None,
            test_id: None,
            menu_style: MenuStyle::default(),
        }
    }

    /// Creates a menu with a controlled/uncontrolled open model.
    ///
    /// When `open` is `None`, the menu stores its internal open model at the root call site and
    /// initializes it from `default_open`.
    pub fn new_controllable<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        open: Option<Model<bool>>,
        default_open: bool,
    ) -> Self {
        let open = controllable_state::use_controllable_model(cx, open, || default_open).model();
        Self::new(open)
    }

    /// Default teaching-surface constructor for a menu that owns its open model.
    pub fn uncontrolled<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Self {
        Self::new_controllable(cx, None, false)
    }

    /// Returns the resolved open model, including the internally owned model for uncontrolled use.
    pub fn open_model(&self) -> Model<bool> {
        self.open.clone()
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

    pub fn submenu_min_width(mut self, min_width: Px) -> Self {
        self.submenu_min_width = min_width;
        self
    }

    pub fn max_height(mut self, max_height: Px) -> Self {
        self.max_height = Some(max_height);
        self
    }

    pub fn menu_style(mut self, style: MenuStyle) -> Self {
        self.menu_style = self.menu_style.merged(style);
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

    #[track_caller]
    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        entries: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<MenuEntry>,
    ) -> AnyElement {
        cx.scope(|cx| {
            let is_open = cx
                .get_model_copied(&self.open, Invalidation::Layout)
                .unwrap_or(false);

            let close_grace_frames = Some({
                let theme = Theme::global(&*cx.app);
                ms_to_frames(dropdown_menu_tokens::close_duration_ms(theme))
            });
            let motion = drive_overlay_open_close_motion(cx, is_open, close_grace_frames);
            let overlay_presence = OverlayPresence {
                present: motion.present,
                interactive: is_open,
            };
            let trigger = trigger(cx);
            let trigger_id = trigger.id;
            let overlay_root_name = format!("material3.dropdown_menu.{}", trigger_id.0);
            let submenu_cfg = menu_primitive::sub::MenuSubmenuConfig::default();
            let submenu_models =
                menu_primitive::root::with_root_name_sync_root_open_and_ensure_submenu(
                    cx,
                    &overlay_root_name,
                    is_open,
                    submenu_cfg,
                );

            if overlay_presence.present {
                let direction = material_layout_direction_in_scope(cx);

                let Some(anchor) = overlay::anchor_bounds_for_element(cx, trigger_id) else {
                    return trigger;
                };
                let outer = overlay::outer_bounds_with_window_margin_for_environment(
                    cx,
                    fret_ui::Invalidation::Layout,
                    self.window_margin,
                );

                let (
                    menu_item_height,
                    menu_item_two_line_height,
                    menu_section_label_height,
                    menu_item_min_width,
                    menu_item_max_width,
                    menu_vertical_padding,
                    divider_height,
                    divider_margin,
                    collision_padding,
                    default_max_height,
                ) = {
                    let theme = Theme::global(&*cx.app);
                    (
                        menu_tokens::list_item_height_for_supporting(theme, false),
                        menu_tokens::list_item_height_for_supporting(theme, true),
                        menu_tokens::section_label_height(theme),
                        menu_tokens::item_min_width(theme),
                        menu_tokens::item_max_width(theme),
                        menu_tokens::container_vertical_padding(theme),
                        menu_tokens::divider_height(theme),
                        dropdown_menu_tokens::divider_margin_total(theme),
                        dropdown_menu_tokens::collision_padding(theme),
                        dropdown_menu_tokens::max_height(theme),
                    )
                };

                let mut menu_entries = entries(cx);
                if self.close_on_select {
                    menu_entries = wrap_close_on_select(menu_entries, self.open.clone());
                }
                let menu_entries_for_submenu = menu_entries.clone();

                let max_height = self
                    .max_height
                    .unwrap_or(default_max_height)
                    .min(Px(outer.size.height.0.max(1.0)));
                let estimated = estimated_menu_panel_size(
                    anchor,
                    self.min_width.max(menu_item_min_width),
                    menu_item_max_width,
                    max_height,
                    &menu_entries,
                    menu_item_height,
                    menu_item_two_line_height,
                    menu_section_label_height,
                    divider_height,
                    divider_margin,
                    menu_vertical_padding,
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
                        .with_collision_padding(collision_padding);
                let layout =
                    popper::popper_content_layout_sized(outer, anchor, estimated, placement);

                let initial_focus_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
                let initial_focus_id_for_menu = initial_focus_id.clone();

                let a11y_label = self
                    .a11y_label
                    .clone()
                    .unwrap_or_else(default_dropdown_menu_a11y_label);

                #[derive(Default)]
                struct DerivedDefaultTestId {
                    trigger: u64,
                    test_id: Option<Arc<str>>,
                }

                let default_test_id = cx.slot_state(DerivedDefaultTestId::default, |st| {
                    if st.test_id.is_none() || st.trigger != trigger_id.0 {
                        st.trigger = trigger_id.0;
                        st.test_id = Some(Arc::<str>::from(format!(
                            "material3-menu-{}",
                            trigger_id.0
                        )));
                    }
                    st.test_id.as_ref().expect("test_id").clone()
                });

                let test_id = self.test_id.clone().unwrap_or(default_test_id);
                let viewport_test_id = part_test_id(&test_id, "viewport");
                let menu_width = layout.rect.size.width;
                let menu_height = layout.rect.size.height;
                let menu_style = self
                    .menu_style
                    .clone()
                    .item_min_width(WidgetStateProperty::new(Some(menu_width)))
                    .item_max_width(WidgetStateProperty::new(Some(menu_width)));

                let overlay_root_name_for_controls = Arc::<str>::from(overlay_root_name.clone());
                let submenu_min_width = self.submenu_min_width.max(menu_item_min_width);
                let submenu_panel_style = self.menu_style.clone();
                let submenu_ctx = MaterialMenuSubmenuContext::root(
                    submenu_models.clone(),
                    submenu_cfg,
                    outer,
                    submenu_min_width,
                    max_height,
                    overlay_root_name_for_controls.clone(),
                    Some(test_id.clone()),
                );
                let submenu_ctx_for_root = submenu_ctx.clone();

                let portal_ctx = portal_inherited::PortalInherited::capture(cx);
                let overlay_children = portal_inherited::with_root_name_inheriting(
                    cx,
                    &overlay_root_name,
                    portal_ctx,
                    |cx| {
                        cx.provide(direction, |cx| {
                            let root_panel = popper_content::popper_wrapper_panel_at(
                                cx,
                                layout.rect,
                                Edges::all(Px(0.0)),
                                Overflow::Visible,
                                move |cx| {
                                    let scroll_handle =
                                        cx.slot_state(ScrollHandle::default, |h| h.clone());
                                    let mut scroll_layout = LayoutStyle::default();
                                    scroll_layout.size.width = Length::Fill;
                                    scroll_layout.size.height = Length::Px(menu_height);
                                    scroll_layout.overflow = Overflow::Clip;

                                    let mut viewport = cx.scroll(
                                        ScrollProps {
                                            layout: scroll_layout,
                                            axis: ScrollAxis::Y,
                                            scroll_handle: Some(scroll_handle),
                                            ..Default::default()
                                        },
                                        move |cx| {
                                            vec![
                                                Menu::new()
                                                    .a11y_label(a11y_label)
                                                    .test_id(test_id)
                                                    .entries(menu_entries)
                                                    .style(menu_style)
                                                    .into_element_with_submenu_context(
                                                        cx,
                                                        initial_focus_id_for_menu,
                                                        Some(submenu_ctx_for_root.clone()),
                                                    ),
                                            ]
                                        },
                                    );
                                    viewport = viewport.test_id(viewport_test_id);
                                    vec![viewport]
                                },
                            );

                            let opacity = motion.alpha;
                            let scale = motion.scale;
                            let origin =
                                popper::popper_content_transform_origin(&layout, anchor, None);
                            let origin_inv =
                                fret_core::Point::new(Px(-origin.x.0), Px(-origin.y.0));
                            let transform = fret_core::Transform2D::translation(origin)
                                * fret_core::Transform2D::scale_uniform(scale)
                                * fret_core::Transform2D::translation(origin_inv);
                            let root_panel = fret_ui_kit::declarative::overlay_motion::wrap_opacity_and_render_transform_gated(
                                cx,
                                opacity,
                                transform,
                                overlay_presence.interactive,
                                vec![root_panel],
                            );
                            let mut children = vec![root_panel];

                            let submenu_open_value = cx
                                .watch_model(&submenu_models.open_value)
                                .layout()
                                .cloned()
                                .unwrap_or(None);
                            if let Some(open_value) = submenu_open_value {
                                let submenu_entries = menu_submenu_entries_by_value(
                                    &menu_entries_for_submenu,
                                    open_value.as_ref(),
                                );
                                let desired = submenu_entries
                                    .as_ref()
                                    .map(|entries| {
                                        let desired_h = estimated_menu_panel_height_for_entries(
                                            entries,
                                            menu_item_height,
                                            menu_item_two_line_height,
                                            menu_section_label_height,
                                            divider_height,
                                            divider_margin,
                                            menu_vertical_padding,
                                            max_height,
                                        );
                                        Size::new(submenu_min_width, desired_h)
                                    })
                                    .unwrap_or_else(|| Size::new(submenu_min_width, max_height));
                                let open_submenu = menu_primitive::sub::with_open_submenu_synced(
                                    cx,
                                    &submenu_models,
                                    outer,
                                    desired,
                                    |_cx, open_value, geometry| (open_value, geometry),
                                );
                                if let (Some((open_value, geometry)), Some(entries)) =
                                    (open_submenu, submenu_entries)
                                {
                                    children.push(material_menu_submenu_panel_tree(
                                        cx,
                                        entries,
                                        open_value,
                                        geometry,
                                        submenu_models.clone(),
                                        submenu_panel_style,
                                        submenu_ctx,
                                    ));
                                }
                            }

                            children
                        })
                    },
                );

                let mut request = overlay_controller::OverlayRequest::dismissible_menu(
                    trigger_id,
                    trigger_id,
                    self.open.clone(),
                    overlay_presence,
                    overlay_children,
                );
                request.root_name = Some(overlay_root_name);
                request.close_on_window_focus_lost = true;
                request.close_on_window_resize = true;
                request.dismissible_on_dismiss_request = self.on_dismiss_request.clone();
                request.dismissible_on_pointer_move = Some(
                    menu_primitive::root::submenu_pointer_move_handler(submenu_models, submenu_cfg),
                );
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
                if let Some(submenu) = item.submenu.take() {
                    item.submenu = Some(wrap_close_on_select(submenu, open.clone()));
                    return MenuEntry::Item(item);
                }
                if item.disabled {
                    return MenuEntry::Item(item);
                }
                let open = open.clone();
                item.append_on_select(Arc::new(move |host, cx, _reason| {
                    let _ = host.models_mut().update(&open, |v| *v = false);
                    host.request_redraw(cx.window);
                }));
                MenuEntry::Item(item)
            }
            MenuEntry::Label(label) => MenuEntry::Label(label),
            MenuEntry::Group(mut group) => {
                group.entries = wrap_close_on_select(group.entries, open.clone());
                MenuEntry::Group(group)
            }
        })
        .collect()
}

fn estimated_menu_panel_size(
    anchor: Rect,
    min_width: Px,
    max_width: Px,
    max_height: Px,
    entries: &[MenuEntry],
    item_height: Px,
    two_line_item_height: Px,
    section_label_height: Px,
    divider_height: Px,
    divider_margin_total: Px,
    vertical_padding: Px,
) -> Size {
    let mut h = (vertical_padding.0.max(0.0) * 2.0).max(0.0);
    for e in entries {
        match e {
            MenuEntry::Item(item) => {
                let height = if item.has_supporting_text() {
                    two_line_item_height
                } else {
                    item_height
                };
                h += height.0.max(0.0);
            }
            MenuEntry::Label(_) => h += section_label_height.0.max(0.0),
            MenuEntry::Group(group) => {
                h += estimated_menu_entries_height(
                    &group.entries,
                    item_height,
                    two_line_item_height,
                    section_label_height,
                    divider_height,
                    divider_margin_total,
                );
            }
            MenuEntry::Separator => {
                h += divider_height.0.max(0.0) + divider_margin_total.0.max(0.0)
            }
        }
    }

    let max_width = max_width.0.max(min_width.0).max(0.0);
    let w = anchor.size.width.0.max(min_width.0).min(max_width).max(0.0);
    let h = h.min(max_height.0.max(1.0));
    Size::new(Px(w), Px(h.max(1.0)))
}

fn estimated_menu_entries_height(
    entries: &[MenuEntry],
    item_height: Px,
    two_line_item_height: Px,
    section_label_height: Px,
    divider_height: Px,
    divider_margin_total: Px,
) -> f32 {
    let mut h = 0.0;
    for e in entries {
        match e {
            MenuEntry::Item(item) => {
                let height = if item.has_supporting_text() {
                    two_line_item_height
                } else {
                    item_height
                };
                h += height.0.max(0.0);
            }
            MenuEntry::Label(_) => h += section_label_height.0.max(0.0),
            MenuEntry::Group(group) => {
                h += estimated_menu_entries_height(
                    &group.entries,
                    item_height,
                    two_line_item_height,
                    section_label_height,
                    divider_height,
                    divider_margin_total,
                );
            }
            MenuEntry::Separator => {
                h += divider_height.0.max(0.0) + divider_margin_total.0.max(0.0)
            }
        }
    }
    h
}

fn estimated_menu_panel_height_for_entries(
    entries: &[MenuEntry],
    item_height: Px,
    two_line_item_height: Px,
    section_label_height: Px,
    divider_height: Px,
    divider_margin_total: Px,
    vertical_padding: Px,
    max_height: Px,
) -> Px {
    let h = vertical_padding.0.max(0.0) * 2.0
        + estimated_menu_entries_height(
            entries,
            item_height,
            two_line_item_height,
            section_label_height,
            divider_height,
            divider_margin_total,
        );
    Px(h.clamp(1.0, max_height.0.max(1.0)))
}

#[cfg(test)]
mod tests {
    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size};
    use fret_ui::elements::with_element_cx;
    use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;

    use super::*;

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        )
    }

    #[test]
    fn dropdown_menu_new_controllable_uses_controlled_model_when_provided() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let controlled = app.models_mut().insert(true);

        with_element_cx(
            &mut app,
            window,
            bounds(),
            "material3-dropdown-menu-controlled",
            |cx| {
                let menu = DropdownMenu::new_controllable(cx, Some(controlled.clone()), false);
                assert_eq!(menu.open_model(), controlled);
            },
        );
    }

    #[test]
    fn dropdown_menu_new_controllable_applies_default_open() {
        let window = AppWindowId::default();
        let mut app = App::new();

        with_element_cx(
            &mut app,
            window,
            bounds(),
            "material3-dropdown-menu-default-open",
            |cx| {
                let menu = DropdownMenu::new_controllable(cx, None, true);
                let open = cx
                    .watch_model(&menu.open_model())
                    .layout()
                    .copied()
                    .unwrap_or(false);
                assert!(open);
            },
        );
    }

    #[test]
    fn dropdown_menu_uncontrolled_multiple_instances_do_not_share_open_model() {
        let window = AppWindowId::default();
        let mut app = App::new();

        with_element_cx(
            &mut app,
            window,
            bounds(),
            "material3-dropdown-menu-uncontrolled-scope",
            |cx| {
                let a = DropdownMenu::uncontrolled(cx);
                let b = DropdownMenu::uncontrolled(cx);
                assert_ne!(a.open_model(), b.open_model());
            },
        );
    }

    #[test]
    fn estimated_panel_size_uses_material_menu_intrinsic_bounds_and_padding() {
        let entries = vec![
            MenuEntry::Label(crate::menu::MenuLabel::new("Actions")),
            MenuEntry::Item(crate::menu::MenuItem::new("Alpha")),
            MenuEntry::Separator,
            MenuEntry::Item(crate::menu::MenuItem::new("Beta")),
        ];
        let wide_anchor = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(360.0), Px(40.0)));
        let narrow_anchor = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(72.0), Px(40.0)));

        let wide = estimated_menu_panel_size(
            wide_anchor,
            Px(112.0),
            Px(280.0),
            Px(1000.0),
            &entries,
            Px(48.0),
            Px(64.0),
            Px(32.0),
            Px(1.0),
            Px(8.0),
            Px(8.0),
        );
        assert_eq!(wide.width, Px(280.0));
        assert_eq!(wide.height, Px(153.0));

        let narrow = estimated_menu_panel_size(
            narrow_anchor,
            Px(112.0),
            Px(280.0),
            Px(1000.0),
            &entries,
            Px(48.0),
            Px(64.0),
            Px(32.0),
            Px(1.0),
            Px(8.0),
            Px(8.0),
        );
        assert_eq!(narrow.width, Px(112.0));
        assert_eq!(narrow.height, Px(153.0));
    }
}

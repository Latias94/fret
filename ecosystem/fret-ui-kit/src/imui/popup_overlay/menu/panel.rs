use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{Corners, Edges, Px, SemanticsRole};
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, InsetStyle, LayoutStyle, Length, Overflow,
    PositionStyle, SpacingLength,
};
use fret_ui::{GlobalElementId, UiHost};

use super::policy::{ImUiMenuNavState, ImUiPopupMenuPolicyState};
use crate::imui::menu_family_controls::ImUiMenubarPolicyState;
use crate::imui::{ImUiFacade, PopupMenuOptions, UiWriterImUiFacadeExt};
use crate::primitives::popper;

pub(super) struct PopupMenuBuilt {
    pub(super) children: Vec<AnyElement>,
    pub(super) first_item: Option<GlobalElementId>,
    pub(super) content_focus: Option<GlobalElementId>,
}

pub(super) fn build_popup_menu<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    root_name: &str,
    options: PopupMenuOptions,
    popup_policy: ImUiPopupMenuPolicyState,
    menubar_policy: Option<ImUiMenubarPolicyState>,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> Option<PopupMenuBuilt> {
    ui.with_cx_mut(|cx| {
        let (open, anchor_model, panel_id) =
            super::super::super::with_popup_store_for_id(cx, id, |st, _app| {
                (st.open.clone(), st.anchor.clone(), st.panel_id)
            });
        let is_open = cx
            .read_model(&open, fret_ui::Invalidation::Paint, |_app, v| *v)
            .unwrap_or(false);
        if !is_open {
            return None;
        }

        let anchor = cx
            .read_model(&anchor_model, fret_ui::Invalidation::Paint, |_app, v| *v)
            .unwrap_or(None);
        let Some(anchor) = anchor else {
            let _ = cx.app.models_mut().update(&open, |v| *v = false);
            let _ = cx.app.models_mut().update(&anchor_model, |v| *v = None);
            super::super::super::with_popup_store_for_id(cx, id, |st, _app| {
                st.panel_id = None;
                st.keep_alive_generation = None;
            });
            cx.app.request_redraw(cx.window);
            return None;
        };

        let keep_alive_generation = super::super::super::popup_render_generation_for_window(cx);
        super::super::super::with_popup_store_for_id(cx, id, move |st, _app| {
            st.keep_alive_generation = Some(keep_alive_generation);
        });

        let desired = panel_id
            .and_then(|id| cx.last_bounds_for_element(id).map(|r| r.size))
            .unwrap_or(options.estimated_size);
        let layout = popper::popper_content_layout_sized(
            cx.environment_viewport_bounds(fret_ui::Invalidation::Layout),
            anchor,
            desired,
            options.placement,
        );

        let (popover, border) = {
            let theme = fret_ui::Theme::global(&*cx.app);
            (theme.color_token("popover"), theme.color_token("border"))
        };

        let nav_items = Rc::new(RefCell::new(Vec::<GlobalElementId>::new()));
        let nav_items_for_state = nav_items.clone();
        let mut menu_id_for_focus: Option<GlobalElementId> = None;
        let mut build = Some(f);
        let popup_policy_for_panel = popup_policy.clone();
        let menubar_policy_for_panel = menubar_policy.clone();
        let panel = cx.with_root_name(root_name, |cx| {
            cx.named("fret-ui-kit.imui.popup.panel", |cx| {
                let mut semantics = fret_ui::element::SemanticsProps::default();
                semantics.role = SemanticsRole::Menu;
                semantics.test_id = Some(Arc::from(format!("imui-popup-{id}")));
                semantics.layout = LayoutStyle {
                    position: PositionStyle::Absolute,
                    inset: InsetStyle {
                        left: Some(layout.rect.origin.x).into(),
                        top: Some(layout.rect.origin.y).into(),
                        ..Default::default()
                    },
                    overflow: Overflow::Visible,
                    ..Default::default()
                };

                let menu = cx.semantics_with_id(semantics, move |cx, menu_id| {
                    cx.state_for(
                        menu_id,
                        || ImUiMenuNavState {
                            items: nav_items_for_state.clone(),
                        },
                        |st| st.items.borrow_mut().clear(),
                    );

                    let mut panel_props = ContainerProps::default();
                    panel_props.background = Some(popover);
                    panel_props.border = Edges::all(Px(1.0));
                    panel_props.border_color = Some(border);
                    panel_props.corner_radii =
                        Corners::all(super::super::super::control_chrome::PANEL_RADIUS);
                    panel_props.padding = Edges::all(Px(4.0)).into();

                    vec![cx.container(panel_props, move |cx| {
                        let mut col = ColumnProps::default();
                        col.gap = SpacingLength::Px(Px(2.0));
                        col.layout.size.width = Length::Auto;
                        col.layout.size.height = Length::Auto;
                        vec![cx.column(col, move |cx| {
                            let render_children = move |cx: &mut fret_ui::ElementContext<'_, H>| {
                                let mut out: Vec<AnyElement> = Vec::new();
                                let mut ui = ImUiFacade {
                                    cx,
                                    out: &mut out,
                                    build_focus: None,
                                };
                                if let Some(f) = build.take() {
                                    f(&mut ui);
                                }
                                out
                            };
                            if let Some(menubar_policy) = menubar_policy_for_panel.clone() {
                                cx.provide(menubar_policy, move |cx| {
                                    cx.provide(popup_policy_for_panel.clone(), render_children)
                                })
                            } else {
                                cx.provide(popup_policy_for_panel.clone(), render_children)
                            }
                        })]
                    })]
                });
                menu_id_for_focus = Some(menu.id);
                super::super::super::with_popup_store_for_id(cx, id, |st, _app| {
                    st.panel_id = Some(menu.id)
                });
                menu
            })
        });

        let first_item = nav_items.borrow().first().copied();
        Some(PopupMenuBuilt {
            children: vec![panel],
            first_item,
            content_focus: menu_id_for_focus,
        })
    })
}

use super::*;

pub(crate) fn render_imui_disabled_scope_overlay_scene(
    cx: &mut ElementContext<'_, TestHost>,
    under_clicked: Rc<Cell<bool>>,
    over_clicked: Rc<Cell<bool>>,
    over_hovered: Rc<Cell<bool>>,
    over_hovered_like_imgui: Rc<Cell<bool>>,
    over_hovered_allow_when_disabled: Rc<Cell<bool>>,
    over_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
) -> crate::Elements {
    let mut stack = fret_ui::element::StackProps::default();
    stack.layout.size.width = Length::Fill;
    let element = cx.stack_props(stack, |cx| {
        crate::imui_raw(cx, |ui| {
            let under = ui.menu_item_with_options(
                "Underlay",
                MenuItemOptions {
                    test_id: Some(Arc::from("imui-underlay-item")),
                    ..Default::default()
                },
            );
            under_clicked.set(under.clicked());

            ui.disabled_scope(true, |ui| {
                let over = ui.menu_item_with_options(
                    "Overlay",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-overlay-item")),
                        ..Default::default()
                    },
                );
                over_id.set(over.id);
                over_clicked.set(over.clicked());
                over_hovered.set(over.core.hovered);
                over_hovered_like_imgui.set(over.hovered_like_imgui());
                over_hovered_allow_when_disabled
                    .set(over.is_hovered(ImUiHoveredFlags::ALLOW_WHEN_DISABLED));
            });
        })
    });
    vec![element].into()
}

pub(crate) fn render_imui_disabled_scope_tooltip_hover_scene(
    cx: &mut ElementContext<'_, TestHost>,
    hovered_for_tooltip: Rc<Cell<bool>>,
    hovered_raw: Rc<Cell<bool>>,
    stationary_met: Rc<Cell<bool>>,
    delay_short_met: Rc<Cell<bool>>,
    delay_normal_met: Rc<Cell<bool>>,
) -> crate::Elements {
    let mut stack = fret_ui::element::StackProps::default();
    stack.layout.size.width = Length::Fill;
    let element = cx.stack_props(stack, |cx| {
        crate::imui_raw(cx, |ui| {
            ui.disabled_scope(true, |ui| {
                let resp = ui.menu_item_with_options(
                    "Tooltip target",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-tooltip-target")),
                        ..Default::default()
                    },
                );
                hovered_for_tooltip.set(resp.is_hovered(ImUiHoveredFlags::FOR_TOOLTIP));
                hovered_raw.set(resp.pointer_hovered_raw);
                stationary_met.set(resp.hover_stationary_met);
                delay_short_met.set(resp.hover_delay_short_met);
                delay_normal_met.set(resp.hover_delay_normal_met);
            });
        })
    });
    vec![element].into()
}

pub(crate) fn render_imui_popup_modal_barrier_hover_scene(
    cx: &mut ElementContext<'_, TestHost>,
    popup_id: &'static str,
    open_popup: bool,
    popup_opened: Rc<Cell<bool>>,
    under_hovered_default: Rc<Cell<bool>>,
    under_hovered_allow_when_blocked: Rc<Cell<bool>>,
    under_hovered_raw: Rc<Cell<bool>>,
    under_hovered_raw_below_barrier: Rc<Cell<bool>>,
) -> crate::Elements {
    let anchor = Rect::new(
        Point::new(Px(280.0), Px(160.0)),
        Size::new(Px(1.0), Px(1.0)),
    );
    let mut stack = fret_ui::element::StackProps::default();
    stack.layout.size.width = Length::Fill;
    let element = cx.stack_props(stack, |cx| {
        crate::imui_raw(cx, |ui| {
            let under = ui.menu_item_with_options(
                "Underlay",
                MenuItemOptions {
                    test_id: Some(Arc::from("imui-underlay-item")),
                    ..Default::default()
                },
            );
            under_hovered_default.set(under.core.hovered);
            under_hovered_allow_when_blocked
                .set(under.is_hovered(ImUiHoveredFlags::ALLOW_WHEN_BLOCKED_BY_POPUP));
            under_hovered_raw.set(under.pointer_hovered_raw);
            under_hovered_raw_below_barrier.set(under.pointer_hovered_raw_below_barrier);

            if open_popup {
                ui.open_popup_at(popup_id, anchor);
            }
            popup_opened.set(ui.begin_popup_menu(popup_id, None, |ui| {
                ui.menu_item_with_options(
                    "Popup item",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-popup-item")),
                        ..Default::default()
                    },
                );
            }));
        })
    });
    vec![element].into()
}

pub(crate) fn render_imui_shared_hover_delay_scene(
    cx: &mut ElementContext<'_, TestHost>,
    id_a: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
    hovered_b_shared: Rc<Cell<bool>>,
    hovered_b_no_shared: Rc<Cell<bool>>,
    b_stationary_met: Rc<Cell<bool>>,
    b_delay_short_met: Rc<Cell<bool>>,
    b_delay_short_shared_met: Rc<Cell<bool>>,
    id_b: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
) -> crate::Elements {
    let mut stack = fret_ui::element::StackProps::default();
    stack.layout.size.width = Length::Fill;
    let element = cx.stack_props(stack, |cx| {
        crate::imui_raw(cx, |ui| {
            let a = ui.menu_item_with_options(
                "A",
                MenuItemOptions {
                    test_id: Some(Arc::from("imui-shared-delay-a")),
                    ..Default::default()
                },
            );
            id_a.set(a.id);

            let b = ui.menu_item_with_options(
                "B",
                MenuItemOptions {
                    test_id: Some(Arc::from("imui-shared-delay-b")),
                    ..Default::default()
                },
            );
            id_b.set(b.id);
            b_stationary_met.set(b.hover_stationary_met);
            b_delay_short_met.set(b.hover_delay_short_met);
            b_delay_short_shared_met.set(b.hover_delay_short_shared_met);
            let flags = ImUiHoveredFlags::DELAY_SHORT | ImUiHoveredFlags::NO_NAV_OVERRIDE;
            hovered_b_shared.set(b.is_hovered(flags));
            hovered_b_no_shared.set(b.is_hovered(flags | ImUiHoveredFlags::NO_SHARED_DELAY));
        })
    });
    vec![element].into()
}

pub(crate) fn render_imui_active_item_blocks_hover_scene(
    cx: &mut ElementContext<'_, TestHost>,
    a_hovered: Rc<Cell<bool>>,
    a_focused: Rc<Cell<bool>>,
    b_core_hovered: Rc<Cell<bool>>,
    b_blocked_by_active_item: Rc<Cell<bool>>,
    b_hovered_default: Rc<Cell<bool>>,
    b_hovered_allow_when_blocked: Rc<Cell<bool>>,
) -> crate::Elements {
    let mut stack = fret_ui::element::StackProps::default();
    stack.layout.size.width = Length::Fill;
    let element = cx.stack_props(stack, |cx| {
        crate::imui_raw(cx, |ui| {
            ui.vertical(|ui| {
                let a = ui.menu_item_with_options(
                    "A",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-active-item-a")),
                        ..Default::default()
                    },
                );
                a_hovered.set(a.core.hovered);
                a_focused.set(a.core.focused);

                let b = ui.menu_item_with_options(
                    "B",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-active-item-b")),
                        ..Default::default()
                    },
                );

                b_core_hovered.set(b.core.hovered);
                b_blocked_by_active_item.set(b.hover_blocked_by_active_item);
                let flags = ImUiHoveredFlags::NO_NAV_OVERRIDE;
                b_hovered_default.set(b.is_hovered(flags));
                b_hovered_allow_when_blocked
                    .set(b.is_hovered(flags | ImUiHoveredFlags::ALLOW_WHEN_BLOCKED_BY_ACTIVE_ITEM));
            });
        })
    });
    vec![element].into()
}

pub(crate) fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for b in bytes {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3u64);
    }
    hash
}

pub(crate) fn hover_timer_token_for(
    kind: u64,
    element: fret_ui::elements::GlobalElementId,
) -> TimerToken {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for b in kind.to_le_bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3u64);
    }
    for b in element.0.to_le_bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3u64);
    }
    TimerToken(hash)
}

use super::super::super::super::super::*;

use super::OverlayModels;

pub(super) struct OverlayWidgets {
    pub(super) overlay_reset: AnyElement,
    pub(super) dropdown: AnyElement,
    pub(super) context_menu: AnyElement,
    pub(super) context_menu_edge: AnyElement,
    pub(super) underlay: AnyElement,
    pub(super) tooltip: AnyElement,
    pub(super) hover_card: AnyElement,
    pub(super) popover: AnyElement,
    pub(super) dialog: AnyElement,
    pub(super) alert_dialog: AnyElement,
    pub(super) sheet: AnyElement,
    pub(super) portal_geometry: AnyElement,
}

pub(super) fn build(cx: &mut ElementContext<'_, App>, models: &OverlayModels) -> OverlayWidgets {
    OverlayWidgets {
        overlay_reset: overlay_reset(cx, models),
        dropdown: dropdown(cx, models),
        context_menu: context_menu(cx, models),
        context_menu_edge: context_menu_edge(cx, models),
        underlay: underlay(cx),
        tooltip: tooltip(cx),
        hover_card: hover_card(cx),
        popover: popover(cx, models),
        dialog: dialog(cx, models),
        alert_dialog: alert_dialog(cx, models),
        sheet: sheet(cx, models),
        portal_geometry: portal_geometry(cx, models),
    }
}

fn overlay_reset(cx: &mut ElementContext<'_, App>, models: &OverlayModels) -> AnyElement {
    use fret_ui::action::OnActivate;

    let dropdown_open = models.dropdown_open.clone();
    let context_menu_open = models.context_menu_open.clone();
    let context_menu_edge_open = models.context_menu_edge_open.clone();
    let popover_open = models.popover_open.clone();
    let dialog_open = models.dialog_open.clone();
    let alert_dialog_open = models.alert_dialog_open.clone();
    let sheet_open = models.sheet_open.clone();
    let portal_geometry_popover_open = models.portal_geometry_popover_open.clone();
    let last_action = models.last_action.clone();

    let on_activate: OnActivate = Arc::new(move |host, _cx, _reason| {
        let _ = host.models_mut().update(&dropdown_open, |v| *v = false);
        let _ = host.models_mut().update(&context_menu_open, |v| *v = false);
        let _ = host
            .models_mut()
            .update(&context_menu_edge_open, |v| *v = false);
        let _ = host.models_mut().update(&popover_open, |v| *v = false);
        let _ = host.models_mut().update(&dialog_open, |v| *v = false);
        let _ = host.models_mut().update(&alert_dialog_open, |v| *v = false);
        let _ = host.models_mut().update(&sheet_open, |v| *v = false);
        let _ = host
            .models_mut()
            .update(&portal_geometry_popover_open, |v| *v = false);
        let _ = host.models_mut().update(&last_action, |v| {
            *v = Arc::<str>::from("overlay:reset");
        });
    });

    shadcn::Button::new("Reset overlays")
        .variant(shadcn::ButtonVariant::Secondary)
        .test_id("ui-gallery-overlay-reset")
        .on_activate(on_activate)
        .into_element(cx)
}

fn dropdown(cx: &mut ElementContext<'_, App>, models: &OverlayModels) -> AnyElement {
    let dropdown_open = models.dropdown_open.clone();

    shadcn::DropdownMenu::new(dropdown_open.clone())
        .modal(false)
        .into_element(
            cx,
            |cx| {
                shadcn::Button::new("DropdownMenu")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-dropdown-trigger")
                    .toggle_model(dropdown_open.clone())
                    .into_element(cx)
            },
            |_cx| {
                vec![
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Apple")
                            .test_id("ui-gallery-dropdown-item-apple")
                            .on_select(CMD_MENU_DROPDOWN_APPLE),
                    ),
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("More")
                            .test_id("ui-gallery-dropdown-item-more")
                            .close_on_select(false)
                            .submenu(vec![
                                shadcn::DropdownMenuEntry::Item(
                                    shadcn::DropdownMenuItem::new("Nested action")
                                        .test_id("ui-gallery-dropdown-submenu-item-nested")
                                        .on_select(CMD_MENU_CONTEXT_ACTION),
                                ),
                                shadcn::DropdownMenuEntry::Separator,
                                shadcn::DropdownMenuEntry::Item(
                                    shadcn::DropdownMenuItem::new("Nested disabled").disabled(true),
                                ),
                            ]),
                    ),
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Orange").on_select(CMD_MENU_DROPDOWN_ORANGE),
                    ),
                    shadcn::DropdownMenuEntry::Separator,
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Disabled").disabled(true),
                    ),
                ]
            },
        )
}

fn context_menu(cx: &mut ElementContext<'_, App>, models: &OverlayModels) -> AnyElement {
    let context_menu_open = models.context_menu_open.clone();

    shadcn::ContextMenu::new(context_menu_open).into_element(
        cx,
        |cx| {
            shadcn::Button::new("ContextMenu (right click)")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-context-trigger")
                .into_element(cx)
        },
        |_cx| {
            vec![
                shadcn::ContextMenuEntry::Item(
                    shadcn::ContextMenuItem::new("Action")
                        .test_id("ui-gallery-context-item-action")
                        .on_select(CMD_MENU_CONTEXT_ACTION),
                ),
                shadcn::ContextMenuEntry::Separator,
                shadcn::ContextMenuEntry::Item(
                    shadcn::ContextMenuItem::new("Disabled").disabled(true),
                ),
            ]
        },
    )
}

fn context_menu_edge(cx: &mut ElementContext<'_, App>, models: &OverlayModels) -> AnyElement {
    let context_menu_edge_open = models.context_menu_edge_open.clone();

    shadcn::ContextMenu::new(context_menu_edge_open).into_element(
        cx,
        |cx| {
            shadcn::Button::new("ContextMenu (edge, right click)")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-context-trigger-edge")
                .into_element(cx)
        },
        |_cx| {
            vec![
                shadcn::ContextMenuEntry::Item(
                    shadcn::ContextMenuItem::new("Action")
                        .test_id("ui-gallery-context-edge-item-action")
                        .on_select(CMD_MENU_CONTEXT_ACTION),
                ),
                shadcn::ContextMenuEntry::Separator,
                shadcn::ContextMenuEntry::Item(
                    shadcn::ContextMenuItem::new("Disabled").disabled(true),
                ),
            ]
        },
    )
}

fn underlay(cx: &mut ElementContext<'_, App>) -> AnyElement {
    shadcn::Button::new("Underlay (outside-press target)")
        .variant(shadcn::ButtonVariant::Secondary)
        .test_id("ui-gallery-overlay-underlay")
        .into_element(cx)
}

fn tooltip(cx: &mut ElementContext<'_, App>) -> AnyElement {
    shadcn::Tooltip::new(
        shadcn::Button::new("Tooltip (hover)")
            .variant(shadcn::ButtonVariant::Outline)
            .test_id("ui-gallery-tooltip-trigger")
            .into_element(cx),
        shadcn::TooltipContent::new(vec![shadcn::TooltipContent::text(
            cx,
            "Tooltip: hover intent + placement",
        )])
        .into_element(cx)
        .test_id("ui-gallery-tooltip-content"),
    )
    .arrow(true)
    .arrow_test_id("ui-gallery-tooltip-arrow")
    .panel_test_id("ui-gallery-tooltip-panel")
    .open_delay_frames(10)
    .close_delay_frames(10)
    .side(shadcn::TooltipSide::Top)
    .into_element(cx)
}

fn hover_card(cx: &mut ElementContext<'_, App>) -> AnyElement {
    shadcn::HoverCard::new(
        shadcn::Button::new("HoverCard (hover)")
            .variant(shadcn::ButtonVariant::Outline)
            .test_id("ui-gallery-hovercard-trigger")
            .into_element(cx),
        shadcn::HoverCardContent::new(vec![
            cx.text("HoverCard content (overlay-root)"),
            cx.text("Move pointer from trigger to content."),
        ])
        .into_element(cx)
        .test_id("ui-gallery-hovercard-content"),
    )
    .open_delay_frames(10)
    .close_delay_frames(10)
    .into_element(cx)
}

fn popover(cx: &mut ElementContext<'_, App>, models: &OverlayModels) -> AnyElement {
    use fret_ui::action::OnDismissRequest;

    let popover_open = models.popover_open.clone();
    let dialog_open = models.dialog_open.clone();
    let last_action = models.last_action.clone();

    let popover_open_for_dismiss = popover_open.clone();
    let last_action_for_dismiss = last_action.clone();
    let popover_on_dismiss: OnDismissRequest = Arc::new(move |host, _cx, _reason| {
        let _ = host
            .models_mut()
            .update(&popover_open_for_dismiss, |open| *open = false);
        let _ = host.models_mut().update(&last_action_for_dismiss, |cur| {
            *cur = Arc::<str>::from("popover:dismissed");
        });
    });

    shadcn::Popover::new(popover_open.clone())
        .auto_focus(true)
        .on_dismiss_request(Some(popover_on_dismiss))
        .into_element(
            cx,
            |cx| {
                shadcn::Button::new("Popover")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-popover-trigger")
                    .toggle_model(popover_open.clone())
                    .into_element(cx)
            },
            |cx| {
                let open_dialog = shadcn::Button::new("Open dialog")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-popover-dialog-trigger")
                    .toggle_model(dialog_open.clone())
                    .into_element(cx);

                let close = shadcn::Button::new("Close")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .test_id("ui-gallery-popover-close")
                    .toggle_model(popover_open.clone())
                    .into_element(cx);

                shadcn::PopoverContent::new(vec![cx.text("Popover content"), open_dialog, close])
                    .into_element(cx)
                    .test_id("ui-gallery-popover-content")
            },
        )
}

fn dialog(cx: &mut ElementContext<'_, App>, models: &OverlayModels) -> AnyElement {
    let dialog_open = models.dialog_open.clone();

    shadcn::Dialog::new(dialog_open.clone()).into_element(
        cx,
        |cx| {
            shadcn::Button::new("Dialog")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-dialog-trigger")
                .toggle_model(dialog_open.clone())
                .into_element(cx)
        },
        |cx| {
            shadcn::DialogContent::new(vec![
                shadcn::DialogHeader::new(vec![
                    shadcn::DialogTitle::new("Dialog").into_element(cx),
                    shadcn::DialogDescription::new("Escape / overlay click closes")
                        .into_element(cx),
                ])
                .into_element(cx),
                {
                    let body = stack::vstack(
                        cx,
                        stack::VStackProps::default()
                            .gap(Space::N2)
                            .layout(LayoutRefinement::default().w_full().min_w_0().min_h_0()),
                        |cx| {
                            (0..64)
                                .map(|i| cx.text(format!("Scrollable content line {}", i + 1)))
                                .collect::<Vec<_>>()
                        },
                    );

                    shadcn::ScrollArea::new([body])
                        .refine_layout(
                            LayoutRefinement::default()
                                .w_full()
                                .h_px(Px(240.0))
                                .min_w_0()
                                .min_h_0(),
                        )
                        .viewport_test_id("ui-gallery-dialog-scroll-viewport")
                        .into_element(cx)
                },
                shadcn::DialogFooter::new(vec![
                    shadcn::Button::new("Close")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .test_id("ui-gallery-dialog-close")
                        .toggle_model(dialog_open.clone())
                        .into_element(cx),
                    shadcn::Button::new("Confirm")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-dialog-confirm")
                        .into_element(cx),
                ])
                .into_element(cx),
            ])
            .into_element(cx)
            .test_id("ui-gallery-dialog-content")
        },
    )
}

fn alert_dialog(cx: &mut ElementContext<'_, App>, models: &OverlayModels) -> AnyElement {
    let alert_dialog_open = models.alert_dialog_open.clone();

    shadcn::AlertDialog::new(alert_dialog_open.clone()).into_element(
        cx,
        |cx| {
            shadcn::Button::new("AlertDialog")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-alert-dialog-trigger")
                .toggle_model(alert_dialog_open.clone())
                .into_element(cx)
        },
        |cx| {
            shadcn::AlertDialogContent::new(vec![
                shadcn::AlertDialogHeader::new(vec![
                    shadcn::AlertDialogTitle::new("Are you absolutely sure?").into_element(cx),
                    shadcn::AlertDialogDescription::new("This is non-closable by overlay click.")
                        .into_element(cx),
                ])
                .into_element(cx),
                shadcn::AlertDialogFooter::new(vec![
                    shadcn::AlertDialogCancel::new("Cancel", alert_dialog_open.clone())
                        .test_id("ui-gallery-alert-dialog-cancel")
                        .into_element(cx),
                    shadcn::AlertDialogAction::new("Continue", alert_dialog_open.clone())
                        .test_id("ui-gallery-alert-dialog-action")
                        .into_element(cx),
                ])
                .into_element(cx),
            ])
            .into_element(cx)
            .test_id("ui-gallery-alert-dialog-content")
        },
    )
}

fn sheet(cx: &mut ElementContext<'_, App>, models: &OverlayModels) -> AnyElement {
    let sheet_open = models.sheet_open.clone();

    shadcn::Sheet::new(sheet_open.clone())
        .side(shadcn::SheetSide::Right)
        .size(Px(360.0))
        .into_element(
            cx,
            |cx| {
                shadcn::Button::new("Sheet")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-sheet-trigger")
                    .toggle_model(sheet_open.clone())
                    .into_element(cx)
            },
            |cx| {
                shadcn::SheetContent::new(vec![
                    shadcn::SheetHeader::new(vec![
                        shadcn::SheetTitle::new("Sheet").into_element(cx),
                        shadcn::SheetDescription::new("A modal side panel.").into_element(cx),
                    ])
                    .into_element(cx),
                    {
                        let body = stack::vstack(
                            cx,
                            stack::VStackProps::default()
                                .gap(Space::N2)
                                .layout(LayoutRefinement::default().w_full().min_w_0().min_h_0()),
                            |cx| {
                                (0..96)
                                    .map(|i| cx.text(format!("Sheet body line {}", i + 1)))
                                    .collect::<Vec<_>>()
                            },
                        );

                        shadcn::ScrollArea::new([body])
                            .refine_layout(
                                LayoutRefinement::default()
                                    .flex_1()
                                    .w_full()
                                    .min_w_0()
                                    .min_h_0(),
                            )
                            .viewport_test_id("ui-gallery-sheet-scroll-viewport")
                            .into_element(cx)
                    },
                    shadcn::SheetFooter::new(vec![
                        shadcn::Button::new("Close")
                            .variant(shadcn::ButtonVariant::Secondary)
                            .test_id("ui-gallery-sheet-close")
                            .toggle_model(sheet_open.clone())
                            .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx)
                .test_id("ui-gallery-sheet-content")
            },
        )
}

fn portal_geometry(cx: &mut ElementContext<'_, App>, models: &OverlayModels) -> AnyElement {
    let portal_geometry_popover_open = models.portal_geometry_popover_open.clone();

    let popover = shadcn::Popover::new(portal_geometry_popover_open.clone())
        .side(shadcn::PopoverSide::Right)
        .align(shadcn::PopoverAlign::Start)
        .side_offset(Px(8.0))
        .window_margin(Px(8.0))
        .arrow(true)
        .into_element(
            cx,
            |cx| {
                shadcn::Button::new("Portal geometry (scroll + clamp)")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-portal-geometry-trigger")
                    .toggle_model(portal_geometry_popover_open.clone())
                    .into_element(cx)
            },
            |cx| {
                let close = shadcn::Button::new("Close")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .test_id("ui-gallery-portal-geometry-popover-close")
                    .toggle_model(portal_geometry_popover_open.clone())
                    .into_element(cx);

                shadcn::PopoverContent::new(vec![
                    cx.text("Popover content (placement + clamp)"),
                    cx.text("Wheel-scroll the viewport while open."),
                    close,
                ])
                .refine_layout(LayoutRefinement::default().w_px(Px(360.0)).h_px(Px(220.0)))
                .into_element(cx)
                .attach_semantics(
                    SemanticsDecoration::default()
                        .test_id("ui-gallery-portal-geometry-popover-content"),
                )
            },
        );

    let items = (1..=48)
        .map(|i| cx.text(format!("Scroll item {i:02}")))
        .collect::<Vec<_>>();

    let body = stack::vstack(cx, stack::VStackProps::default().gap(Space::N2), |_cx| {
        let mut out: Vec<AnyElement> = Vec::with_capacity(items.len() + 2);
        out.push(popover);
        out.extend(items);
        out
    });

    let scroll = shadcn::ScrollArea::new(vec![body])
        .refine_layout(LayoutRefinement::default().w_px(Px(240.0)).h_px(Px(160.0)))
        .into_element(cx);

    let scroll = scroll.attach_semantics(
        SemanticsDecoration::default().test_id("ui-gallery-portal-geometry-scroll-area"),
    );

    shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Portal geometry").into_element(cx),
            shadcn::CardDescription::new(
                "Validates floating placement under scroll + window clamp.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![scroll]).into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full())
    .into_element(cx)
}

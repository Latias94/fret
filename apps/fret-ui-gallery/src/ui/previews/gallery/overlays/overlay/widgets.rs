use super::super::super::super::super::*;
use fret::UiChild;
use fret::UiCx;

use super::OverlayModels;
use fret_core::Color;

// Typed helper shells: these helpers may still lower to overlay/provider roots internally because
// the current shadcn root APIs land concrete elements, but the gallery preview no longer stores a
// landed widget inventory just to lay them out.
pub(super) fn overlay_reset(_cx: &mut UiCx<'_>, models: &OverlayModels) -> impl UiChild + use<> {
    use fret_ui::action::OnActivate;

    let dropdown_open = models.dropdown_open.clone();
    let context_menu_open = models.context_menu_open.clone();
    let context_menu_edge_open = models.context_menu_edge_open.clone();
    let popover_open = models.popover_open.clone();
    let dialog_open = models.dialog_open.clone();
    let dialog_glass_open = models.dialog_glass_open.clone();
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
        let _ = host.models_mut().update(&dialog_glass_open, |v| *v = false);
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
}

pub(super) fn dropdown(_cx: &mut UiCx<'_>, models: &OverlayModels) -> impl UiChild + use<> {
    let dropdown_open = models.dropdown_open.clone();

    shadcn::DropdownMenu::from_open(dropdown_open.clone())
        .modal(false)
        .compose()
        .trigger(
            shadcn::Button::new("DropdownMenu")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-dropdown-trigger"),
        )
        .entries_with(move |_cx| {
            vec![
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Apple")
                        .test_id("ui-gallery-dropdown-item-apple")
                        .action(CMD_MENU_DROPDOWN_APPLE),
                ),
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("More")
                        .test_id("ui-gallery-dropdown-item-more")
                        .close_on_select(false)
                        .submenu(vec![
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Nested action")
                                    .test_id("ui-gallery-dropdown-submenu-item-nested")
                                    .action(CMD_MENU_CONTEXT_ACTION),
                            ),
                            shadcn::DropdownMenuEntry::Separator,
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Nested disabled").disabled(true),
                            ),
                        ]),
                ),
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Orange").action(CMD_MENU_DROPDOWN_ORANGE),
                ),
                shadcn::DropdownMenuEntry::Separator,
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Disabled").disabled(true),
                ),
            ]
        })
}

pub(super) fn context_menu(_cx: &mut UiCx<'_>, models: &OverlayModels) -> impl UiChild + use<> {
    let context_menu_open = models.context_menu_open.clone();

    shadcn::ContextMenu::from_open(context_menu_open)
        .content_test_id("ui-gallery-context-content")
        .compose()
        .trigger(
            shadcn::Button::new("ContextMenu (right click)")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-context-trigger"),
        )
        .entries_with(|_cx| {
            vec![
                shadcn::ContextMenuEntry::Item(
                    shadcn::ContextMenuItem::new("Action")
                        .test_id("ui-gallery-context-item-action")
                        .action(CMD_MENU_CONTEXT_ACTION),
                ),
                shadcn::ContextMenuEntry::Separator,
                shadcn::ContextMenuEntry::Item(
                    shadcn::ContextMenuItem::new("Disabled").disabled(true),
                ),
            ]
        })
}

pub(super) fn context_menu_edge(
    _cx: &mut UiCx<'_>,
    models: &OverlayModels,
) -> impl UiChild + use<> {
    let context_menu_edge_open = models.context_menu_edge_open.clone();

    // Keep this trigger near the window edge so the default `side=Right` placement is forced to flip.
    shadcn::ContextMenu::from_open(context_menu_edge_open)
        .content_test_id("ui-gallery-context-edge-content")
        .compose()
        .trigger(
            shadcn::Button::new("ContextMenu (edge)")
                .variant(shadcn::ButtonVariant::Outline)
                .refine_layout(LayoutRefinement::default().w_px(Px(200.0)))
                .test_id("ui-gallery-context-trigger-edge"),
        )
        .content(
            shadcn::ContextMenuContent::new()
                .min_width(Px(240.0))
                .window_margin(Px(8.0)),
        )
        .entries_with(|_cx| {
            vec![
                shadcn::ContextMenuEntry::Item(
                    shadcn::ContextMenuItem::new("Action (long label to force edge flip)")
                        .test_id("ui-gallery-context-edge-item-action")
                        .action(CMD_MENU_CONTEXT_ACTION),
                ),
                shadcn::ContextMenuEntry::Separator,
                shadcn::ContextMenuEntry::Item(
                    shadcn::ContextMenuItem::new("Disabled").disabled(true),
                ),
            ]
        })
}

pub(super) fn underlay(_cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::Button::new("Underlay (outside-press target)")
        .variant(shadcn::ButtonVariant::Secondary)
        .test_id("ui-gallery-overlay-underlay")
}

pub(super) fn tooltip(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    use std::time::Duration;

    let tooltip_a_content = shadcn::TooltipContent::build(cx, |_cx| {
        [shadcn::TooltipContent::text(
            "Tooltip: hover intent + placement",
        )]
    })
    .test_id("ui-gallery-tooltip-content")
    .into_element(cx);
    let tooltip_a = shadcn::Tooltip::new(
        cx,
        shadcn::Button::new("Tooltip A (delay)")
            .variant(shadcn::ButtonVariant::Outline)
            .test_id("ui-gallery-tooltip-trigger"),
        tooltip_a_content,
    )
    .arrow(true)
    .arrow_test_id("ui-gallery-tooltip-arrow")
    .panel_test_id("ui-gallery-tooltip-panel")
    .side(shadcn::TooltipSide::Top)
    .into_element(cx);

    let tooltip_b_content = shadcn::TooltipContent::build(cx, |_cx| {
        [shadcn::TooltipContent::text(
            "Skip-delay window should open this immediately after closing A.",
        )]
    })
    .test_id("ui-gallery-tooltip-skip-content")
    .into_element(cx);
    let tooltip_b = shadcn::Tooltip::new(
        cx,
        shadcn::Button::new("Tooltip B (skip delay)")
            .variant(shadcn::ButtonVariant::Outline)
            .test_id("ui-gallery-tooltip-skip-trigger"),
        tooltip_b_content,
    )
    .arrow(true)
    .arrow_test_id("ui-gallery-tooltip-skip-arrow")
    .panel_test_id("ui-gallery-tooltip-skip-panel")
    .side(shadcn::TooltipSide::Top)
    .into_element(cx);

    let row = ui::h_row(|_cx| vec![tooltip_a, tooltip_b])
        .gap(Space::N2)
        .items_center()
        .into_element(cx)
        .test_id("ui-gallery-tooltip-delay-group-row");

    shadcn::TooltipProvider::new()
        .delay(Duration::from_millis(700))
        .close_delay(Duration::ZERO)
        .timeout_duration(Duration::from_millis(300))
        .with(cx, |_cx| vec![row])
        .into_iter()
        .next()
        .expect("tooltip provider returns one root element")
}

pub(super) fn hover_card(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    use std::time::Duration;

    let hover_card_content = shadcn::HoverCardContent::new(vec![
        cx.text("HoverCard content (overlay-root)"),
        cx.text("Move pointer from trigger to content."),
    ])
    .test_id("ui-gallery-hovercard-content")
    .into_element(cx);

    shadcn::HoverCard::new(
        cx,
        shadcn::Button::new("HoverCard (hover)")
            .variant(shadcn::ButtonVariant::Outline)
            .test_id("ui-gallery-hovercard-trigger"),
        hover_card_content,
    )
    .open_delay(Duration::from_millis(700))
    .close_delay(Duration::from_millis(300))
    .into_element(cx)
}

pub(super) fn popover(cx: &mut UiCx<'_>, models: &OverlayModels) -> impl UiChild + use<> {
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

    shadcn::Popover::from_open(popover_open.clone())
        .auto_focus(true)
        .on_dismiss_request(Some(popover_on_dismiss))
        .into_element_with(
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

                let header = shadcn::PopoverHeader::new(vec![
                    shadcn::PopoverTitle::new("Popover").into_element(cx),
                    shadcn::PopoverDescription::new("Escape / outside click closes.")
                        .into_element(cx),
                ])
                .into_element(cx);

                let actions = ui::v_flex(move |_cx| vec![open_dialog, close])
                    .gap(Space::N2)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full())
                    .into_element(cx);

                shadcn::PopoverContent::new(vec![header, actions])
                    .into_element(cx)
                    .test_id("ui-gallery-popover-content")
            },
        )
}

pub(super) fn dialog(_cx: &mut UiCx<'_>, models: &OverlayModels) -> impl UiChild + use<> {
    let dialog_open = models.dialog_open.clone();
    let dialog_open_for_trigger = dialog_open.clone();

    shadcn::Dialog::new(dialog_open.clone())
        .compose()
        .trigger(shadcn::DialogTrigger::build(
            shadcn::Button::new("Dialog")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-dialog-trigger")
                .toggle_model(dialog_open_for_trigger),
        ))
        .content_with(move |cx| {
            shadcn::DialogContent::new(vec![
                shadcn::DialogHeader::new(vec![
                    shadcn::DialogTitle::new("Dialog").into_element(cx),
                    shadcn::DialogDescription::new("Escape / overlay click closes")
                        .into_element(cx),
                ])
                .into_element(cx),
                {
                    let body = ui::v_flex(|cx| {
                        (0..64)
                            .map(|i| cx.text(format!("Scrollable content line {}", i + 1)))
                            .collect::<Vec<_>>()
                    })
                    .gap(Space::N2)
                    .layout(LayoutRefinement::default().w_full().min_w_0().min_h_0())
                    .into_element(cx);

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
                shadcn::DialogClose::new(dialog_open.clone())
                    .into_element(cx)
                    .test_id("ui-gallery-dialog-x-close"),
            ])
            .show_close_button(false)
            .into_element(cx)
            .test_id("ui-gallery-dialog-content")
        })
}

pub(super) fn dialog_glass(_cx: &mut UiCx<'_>, models: &OverlayModels) -> impl UiChild + use<> {
    let dialog_open = models.dialog_glass_open.clone();
    let dialog_open_for_trigger = dialog_open.clone();

    let overlay_tint = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.25,
    };

    shadcn::Dialog::new(dialog_open.clone())
        .overlay_color(overlay_tint)
        .overlay_glass_backdrop(true)
        .compose()
        .trigger(shadcn::DialogTrigger::build(
            shadcn::Button::new("Dialog (Glass)")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-dialog-glass-trigger")
                .toggle_model(dialog_open_for_trigger),
        ))
        .content_with(move |cx| {
            shadcn::DialogContent::new(vec![
                shadcn::DialogHeader::new(vec![
                    shadcn::DialogTitle::new("Dialog (Glass)").into_element(cx),
                    shadcn::DialogDescription::new(
                        "Backdrop blur variant (reduced-transparency aware).",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
                {
                    let body = ui::v_flex(|cx| {
                        (0..64)
                            .map(|i| cx.text(format!("Scrollable content line {}", i + 1)))
                            .collect::<Vec<_>>()
                    })
                    .gap(Space::N2)
                    .layout(LayoutRefinement::default().w_full().min_w_0().min_h_0())
                    .into_element(cx);

                    shadcn::ScrollArea::new([body])
                        .refine_layout(
                            LayoutRefinement::default()
                                .w_full()
                                .h_px(Px(240.0))
                                .min_w_0()
                                .min_h_0(),
                        )
                        .viewport_test_id("ui-gallery-dialog-glass-scroll-viewport")
                        .into_element(cx)
                },
                shadcn::DialogFooter::new(vec![
                    shadcn::Button::new("Close")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .test_id("ui-gallery-dialog-glass-close")
                        .toggle_model(dialog_open.clone())
                        .into_element(cx),
                    shadcn::Button::new("Confirm")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-dialog-glass-confirm")
                        .into_element(cx),
                ])
                .into_element(cx),
                shadcn::DialogClose::new(dialog_open.clone())
                    .into_element(cx)
                    .test_id("ui-gallery-dialog-glass-x-close"),
            ])
            .show_close_button(false)
            .into_element(cx)
            .test_id("ui-gallery-dialog-glass-content")
        })
}

pub(super) fn alert_dialog(_cx: &mut UiCx<'_>, models: &OverlayModels) -> impl UiChild + use<> {
    let alert_dialog_open = models.alert_dialog_open.clone();
    let alert_dialog_open_for_trigger = alert_dialog_open.clone();

    shadcn::AlertDialog::new(alert_dialog_open.clone())
        .compose()
        .trigger(shadcn::AlertDialogTrigger::build(
            shadcn::Button::new("AlertDialog")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-alert-dialog-trigger")
                .toggle_model(alert_dialog_open_for_trigger),
        ))
        .content_with(move |cx| {
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
        })
}

pub(super) fn sheet(_cx: &mut UiCx<'_>, models: &OverlayModels) -> impl UiChild + use<> {
    let sheet_open = models.sheet_open.clone();
    let sheet_open_for_trigger = sheet_open.clone();

    shadcn::Sheet::new(sheet_open.clone())
        .side(shadcn::SheetSide::Right)
        .size(Px(360.0))
        .compose()
        .trigger(shadcn::SheetTrigger::build(
            shadcn::Button::new("Sheet")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-sheet-trigger")
                .toggle_model(sheet_open_for_trigger),
        ))
        .content_with(move |cx| {
            shadcn::SheetContent::new(vec![
                shadcn::SheetHeader::new(vec![
                    shadcn::SheetTitle::new("Sheet").into_element(cx),
                    shadcn::SheetDescription::new("A modal side panel.").into_element(cx),
                ])
                .into_element(cx),
                {
                    let body = ui::v_flex(|cx| {
                        (0..96)
                            .map(|i| cx.text(format!("Sheet body line {}", i + 1)))
                            .collect::<Vec<_>>()
                    })
                    .gap(Space::N2)
                    .layout(LayoutRefinement::default().w_full().min_w_0().min_h_0())
                    .into_element(cx);
                    let body = {
                        let props = decl_style::container_props(
                            Theme::global(&*cx.app),
                            ChromeRefinement::default().px(Space::N4),
                            LayoutRefinement::default().w_full().min_w_0().min_h_0(),
                        );
                        cx.container(props, move |_cx| vec![body])
                    };

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
        })
}

pub(super) fn portal_geometry(cx: &mut UiCx<'_>, models: &OverlayModels) -> impl UiChild + use<> {
    let portal_geometry_popover_open = models.portal_geometry_popover_open.clone();

    let popover = shadcn::Popover::from_open(portal_geometry_popover_open.clone())
        .side(shadcn::PopoverSide::Right)
        .align(shadcn::PopoverAlign::Start)
        .side_offset(Px(8.0))
        .window_margin(Px(8.0))
        .arrow(true)
        .into_element_with(
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

    let body = ui::v_stack(|_cx| {
        let mut out: Vec<AnyElement> = Vec::with_capacity(items.len() + 2);
        out.push(popover);
        out.extend(items);
        out
    })
    .gap(Space::N2)
    .into_element(cx);

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

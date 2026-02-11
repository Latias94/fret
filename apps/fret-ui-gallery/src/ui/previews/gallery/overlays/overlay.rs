use super::super::super::super::*;

pub(in crate::ui) fn preview_overlay(
    cx: &mut ElementContext<'_, App>,
    popover_open: Model<bool>,
    dialog_open: Model<bool>,
    alert_dialog_open: Model<bool>,
    sheet_open: Model<bool>,
    portal_geometry_popover_open: Model<bool>,
    dropdown_open: Model<bool>,
    context_menu_open: Model<bool>,
    context_menu_edge_open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_ui::action::OnDismissRequest;

    let last_action_status = {
        let last = cx
            .app
            .models()
            .get_cloned(&last_action)
            .unwrap_or_else(|| Arc::<str>::from("<none>"));
        let text = format!("last action: {last}");
        cx.text(text).test_id("ui-gallery-overlay-last-action")
    };

    let overlays =
        cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
            let overlay_reset = {
                use fret_ui::action::OnActivate;

                let dropdown_open = dropdown_open.clone();
                let context_menu_open = context_menu_open.clone();
                let context_menu_edge_open = context_menu_edge_open.clone();
                let popover_open = popover_open.clone();
                let dialog_open = dialog_open.clone();
                let alert_dialog_open = alert_dialog_open.clone();
                let sheet_open = sheet_open.clone();
                let portal_geometry_popover_open = portal_geometry_popover_open.clone();
                let last_action = last_action.clone();

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
            };

            let dropdown = shadcn::DropdownMenu::new(dropdown_open.clone())
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
                                            shadcn::DropdownMenuItem::new("Nested disabled")
                                                .disabled(true),
                                        ),
                                    ]),
                            ),
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Orange")
                                    .on_select(CMD_MENU_DROPDOWN_ORANGE),
                            ),
                            shadcn::DropdownMenuEntry::Separator,
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Disabled").disabled(true),
                            ),
                        ]
                    },
                );

            let context_menu = shadcn::ContextMenu::new(context_menu_open.clone()).into_element(
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
            );

            let context_menu_edge = shadcn::ContextMenu::new(context_menu_edge_open.clone())
                .into_element(
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
                );

            let underlay = shadcn::Button::new("Underlay (outside-press target)")
                .variant(shadcn::ButtonVariant::Secondary)
                .test_id("ui-gallery-overlay-underlay")
                .into_element(cx);

            let tooltip = shadcn::Tooltip::new(
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
            .into_element(cx);

            let hover_card = shadcn::HoverCard::new(
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
            .into_element(cx);

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

            let popover = shadcn::Popover::new(popover_open.clone())
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

                        shadcn::PopoverContent::new(vec![
                            cx.text("Popover content"),
                            open_dialog,
                            close,
                        ])
                        .into_element(cx)
                        .test_id("ui-gallery-popover-content")
                    },
                );

            let dialog = shadcn::Dialog::new(dialog_open.clone()).into_element(
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
                                stack::VStackProps::default().gap(Space::N2).layout(
                                    LayoutRefinement::default().w_full().min_w_0().min_h_0(),
                                ),
                                |cx| {
                                    (0..64)
                                        .map(|i| {
                                            cx.text(format!("Scrollable content line {}", i + 1))
                                        })
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
            );

            let alert_dialog = shadcn::AlertDialog::new(alert_dialog_open.clone()).into_element(
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
                            shadcn::AlertDialogTitle::new("Are you absolutely sure?")
                                .into_element(cx),
                            shadcn::AlertDialogDescription::new(
                                "This is non-closable by overlay click.",
                            )
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
            );

            let sheet = shadcn::Sheet::new(sheet_open.clone())
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
                                shadcn::SheetDescription::new("A modal side panel.")
                                    .into_element(cx),
                            ])
                            .into_element(cx),
                            {
                                let body = stack::vstack(
                                    cx,
                                    stack::VStackProps::default().gap(Space::N2).layout(
                                        LayoutRefinement::default().w_full().min_w_0().min_h_0(),
                                    ),
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
                );

            let portal_geometry = {
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
                            .refine_layout(
                                LayoutRefinement::default().w_px(Px(360.0)).h_px(Px(220.0)),
                            )
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
                    SemanticsDecoration::default()
                        .test_id("ui-gallery-portal-geometry-scroll-area"),
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
            };

            let body = stack::vstack(
                cx,
                stack::VStackProps::default().layout(LayoutRefinement::default().w_full()),
                |cx| {
                    let gap = cx.with_theme(|theme| {
                        fret_ui_kit::MetricRef::space(Space::N2).resolve(theme)
                    });

                    let row = |cx: &mut ElementContext<'_, App>, children: Vec<AnyElement>| {
                        let layout = cx.with_theme(|theme| {
                            decl_style::layout_style(
                                theme,
                                LayoutRefinement::default().w_full().min_w_0(),
                            )
                        });
                        cx.flex(
                            fret_ui::element::FlexProps {
                                layout,
                                direction: fret_core::Axis::Horizontal,
                                gap,
                                padding: Edges::all(Px(0.0)),
                                justify: fret_ui::element::MainAlign::Start,
                                align: fret_ui::element::CrossAlign::Center,
                                wrap: true,
                            },
                            |_cx| children,
                        )
                    };

                    let row_end = |cx: &mut ElementContext<'_, App>, children: Vec<AnyElement>| {
                        let layout = cx.with_theme(|theme| {
                            decl_style::layout_style(
                                theme,
                                LayoutRefinement::default().w_full().min_w_0(),
                            )
                        });
                        cx.flex(
                            fret_ui::element::FlexProps {
                                layout,
                                direction: fret_core::Axis::Horizontal,
                                gap,
                                padding: Edges::all(Px(0.0)),
                                justify: fret_ui::element::MainAlign::End,
                                align: fret_ui::element::CrossAlign::Center,
                                wrap: false,
                            },
                            |_cx| children,
                        )
                    };

                    vec![
                        row(cx, vec![dropdown, context_menu, overlay_reset]),
                        row_end(cx, vec![context_menu_edge]),
                        row(cx, vec![tooltip, hover_card, popover, underlay, dialog]),
                        row(cx, vec![alert_dialog, sheet]),
                        portal_geometry,
                    ]
                },
            );

            vec![body]
        });

    let dialog_open_flag = {
        let open = cx
            .get_model_copied(&dialog_open, Invalidation::Layout)
            .unwrap_or(false);
        if open {
            Some(cx.text("Dialog open").test_id("ui-gallery-dialog-open"))
        } else {
            None
        }
    };

    let alert_dialog_open_flag = {
        let open = cx
            .get_model_copied(&alert_dialog_open, Invalidation::Layout)
            .unwrap_or(false);
        if open {
            Some(
                cx.text("AlertDialog open")
                    .test_id("ui-gallery-alert-dialog-open"),
            )
        } else {
            None
        }
    };

    let popover_dismissed_flag = {
        let last = cx
            .get_model_cloned(&last_action, Invalidation::Layout)
            .unwrap_or_else(|| Arc::<str>::from("<none>"));
        if last.as_ref() == "popover:dismissed" {
            Some(
                cx.text("Popover dismissed")
                    .test_id("ui-gallery-popover-dismissed"),
            )
        } else {
            None
        }
    };

    let mut out: Vec<AnyElement> = vec![overlays, last_action_status];

    if let Some(flag) = popover_dismissed_flag {
        out.push(flag);
    }
    if let Some(flag) = dialog_open_flag {
        out.push(flag);
    }
    if let Some(flag) = alert_dialog_open_flag {
        out.push(flag);
    }

    out
}

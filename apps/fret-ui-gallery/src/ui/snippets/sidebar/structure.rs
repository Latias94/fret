pub const SOURCE: &str = include_str!("structure.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_icons::IconId;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate, UiActionHost};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

fn sidebar_icon(cx: &mut AppComponentCx<'_>, id: &'static str) -> AnyElement {
    shadcn::raw::icon::icon(cx, IconId::new_static(id)).into_element(cx)
}

fn set_text_model(model: Model<Arc<str>>, value: &'static str) -> OnActivate {
    let value = Arc::<str>::from(value);
    Arc::new(
        move |host: &mut dyn UiActionHost, action_cx: ActionCx, _reason: ActivateReason| {
            let _ = host
                .models_mut()
                .update(&model, |current| *current = value.clone());
            host.request_redraw(action_cx.window);
        },
    )
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let search = cx.local_model_keyed("search", String::new);
    let selected = cx.local_model_keyed("selected", || Arc::<str>::from("overview"));
    let last_action = cx.local_model_keyed("last_action", || Arc::<str>::from("none"));
    let help_group_open = cx.local_model_keyed("help_group_open", || true);

    let content = shadcn::SidebarProvider::new()
        .width(Px(288.0))
        .width_icon(Px(56.0))
        .width_mobile(Px(320.0))
        .with(cx, |cx| {
            let sidebar_ctx = shadcn::use_sidebar(cx).expect("sidebar context");
            let collapsed = !sidebar_ctx.is_mobile && sidebar_ctx.collapsed();
            let search_value = cx
                .get_model_cloned(&search, Invalidation::Layout)
                .unwrap_or_default();
            let selected_value = cx
                .get_model_cloned(&selected, Invalidation::Layout)
                .unwrap_or_else(|| Arc::<str>::from("overview"));
            let last_action_value = cx
                .get_model_cloned(&last_action, Invalidation::Layout)
                .unwrap_or_else(|| Arc::<str>::from("none"));

            let workspace_switcher = shadcn::SidebarMenu::new([shadcn::SidebarMenuItem::new(
                shadcn::SidebarMenuButton::new("Acme Inc.")
                    .icon(IconId::new_static("lucide.building-2"))
                    .on_activate(set_text_model(last_action.clone(), "switch-workspace"))
                    .into_element(cx),
            )
            .extend_children([shadcn::SidebarMenuAction::new([sidebar_icon(
                cx,
                "lucide.chevrons-up-down",
            )])
            .a11y_label("Switch workspace")
            .on_activate(set_text_model(last_action.clone(), "switch-workspace"))
            .into_element(cx)])
            .into_element(cx)])
            .into_element(cx);

            let overview_item = shadcn::SidebarMenuItem::new(
                shadcn::SidebarMenuButton::new("Overview")
                    .icon(IconId::new_static("lucide.layout-dashboard"))
                    .active(selected_value.as_ref() == "overview")
                    .on_activate(set_text_model(selected.clone(), "overview"))
                    .into_element(cx),
            )
            .into_element(cx);

            let reports_item = shadcn::SidebarMenuItem::new(
                shadcn::SidebarMenuButton::new("Reports")
                    .active(selected_value.as_ref() == "reports")
                    .on_activate(set_text_model(selected.clone(), "reports"))
                    .children([
                        sidebar_icon(cx, "lucide.file-chart-column"),
                        shadcn::raw::typography::small("Reports").into_element(cx),
                        shadcn::raw::typography::muted("Beta").into_element(cx),
                    ])
                    .into_element(cx),
            )
            .extend_children([
                shadcn::SidebarMenuBadge::new("12")
                    .test_id("ui-gallery-sidebar-structure-badge")
                    .into_element(cx),
            ])
            .into_element(cx);

            let projects_item = shadcn::SidebarMenuItem::new(
                shadcn::SidebarMenuButton::new("Projects")
                    .icon(IconId::new_static("lucide.folder-kanban"))
                    .active(selected_value.as_ref() == "projects")
                    .on_activate(set_text_model(selected.clone(), "projects"))
                    .into_element(cx),
            )
            .extend_children([
                shadcn::SidebarMenuAction::new([sidebar_icon(cx, "lucide.plus")])
                    .a11y_label("Add project")
                    .on_activate(set_text_model(last_action.clone(), "add-project"))
                    .test_id("ui-gallery-sidebar-structure-menu-action")
                    .into_element(cx),
            ])
            .into_element(cx);

            let settings_submenu = shadcn::SidebarMenuSub::new([
                shadcn::SidebarMenuSubItem::new(
                    shadcn::SidebarMenuSubButton::new("Billing")
                        .active(selected_value.as_ref() == "billing")
                        .on_activate(set_text_model(selected.clone(), "billing"))
                        .into_element(cx),
                )
                .into_element(cx),
                shadcn::SidebarMenuSubItem::new(
                    shadcn::SidebarMenuSubButton::new("Usage")
                        .active(selected_value.as_ref() == "usage")
                        .on_activate(set_text_model(selected.clone(), "usage"))
                        .into_element(cx),
                )
                .into_element(cx),
            ])
            .into_element(cx);
            let settings_item = shadcn::SidebarMenuItem::new(
                shadcn::SidebarMenuButton::new("Settings")
                    .icon(IconId::new_static("lucide.settings-2"))
                    .active(selected_value.as_ref() == "settings")
                    .on_activate(set_text_model(selected.clone(), "settings"))
                    .into_element(cx),
            )
            .extend_children([settings_submenu])
            .into_element(cx);

            let workspace_group = shadcn::SidebarGroup::new([
                shadcn::SidebarGroupLabel::new("Workspace").into_element(cx),
                shadcn::SidebarGroupAction::new([sidebar_icon(cx, "lucide.plus")])
                    .a11y_label("Create workspace item")
                    .on_activate(set_text_model(last_action.clone(), "create-workspace-item"))
                    .test_id("ui-gallery-sidebar-structure-group-action")
                    .into_element(cx),
                shadcn::SidebarGroupContent::new([shadcn::SidebarMenu::new([
                    overview_item,
                    reports_item,
                    projects_item,
                    settings_item,
                ])
                .into_element(cx)])
                .into_element(cx),
            ])
            .into_element(cx);

            let help_group = shadcn::CollapsibleRoot::new()
                .open(help_group_open.clone())
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx, |cx| {
                    let label_row = ui::h_flex(|cx| {
                        vec![
                            shadcn::raw::typography::small("Help").into_element(cx),
                            sidebar_icon(cx, "lucide.chevron-down"),
                        ]
                    })
                    .gap(Space::N2)
                    .items_center()
                    .justify_between()
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .test_id("ui-gallery-sidebar-structure-group-label-row")
                    .into_element(cx);

                    let trigger = shadcn::CollapsibleTriggerPart::new([label_row])
                        .as_child(true)
                        .into_element(cx);

                    let label = shadcn::SidebarGroupLabel::new("Help")
                        .as_child(true)
                        .children([trigger])
                        .into_element(cx);

                    let content = shadcn::CollapsibleContentPart::new([
                        shadcn::SidebarGroupContent::new([shadcn::SidebarMenu::new([
                            shadcn::SidebarMenuItem::new(
                                shadcn::SidebarMenuButton::new("Documentation")
                                    .icon(IconId::new_static("lucide.book-open"))
                                    .on_activate(set_text_model(
                                        last_action.clone(),
                                        "open-help-docs",
                                    ))
                                    .into_element(cx),
                            )
                            .into_element(cx),
                            shadcn::SidebarMenuItem::new(
                                shadcn::SidebarMenuButton::new("Keyboard shortcuts")
                                    .icon(IconId::new_static("lucide.keyboard"))
                                    .on_activate(set_text_model(
                                        last_action.clone(),
                                        "open-help-shortcuts",
                                    ))
                                    .into_element(cx),
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx)])
                        .into_element(cx),
                    ])
                    .test_id("ui-gallery-sidebar-structure-group-label-content")
                    .into_element(cx);

                    vec![shadcn::SidebarGroup::new([label, content]).into_element(cx)]
                });

            let footer = shadcn::SidebarFooter::new([shadcn::SidebarMenu::new([
                shadcn::SidebarMenuItem::new(
                    shadcn::SidebarMenuButton::new("frank@example.com")
                        .icon(IconId::new_static("lucide.user-round"))
                        .on_activate(set_text_model(last_action.clone(), "open-account"))
                        .into_element(cx),
                )
                .into_element(cx),
            ])
            .into_element(cx)])
            .into_element(cx);

            let sidebar = shadcn::Sidebar::new([
                shadcn::SidebarHeader::new([
                    shadcn::SidebarInput::new(search.clone())
                        .a11y_label("Search sidebar")
                        .placeholder("Search")
                        .into_element(cx),
                    shadcn::SidebarSeparator::new().into_element(cx),
                    workspace_switcher,
                ])
                .into_element(cx),
                shadcn::SidebarContent::new([workspace_group, help_group]).into_element(cx),
                footer,
                shadcn::SidebarRail::new()
                    .test_id("ui-gallery-sidebar-structure-rail")
                    .into_element(cx),
            ])
            .collapsible(shadcn::SidebarCollapsible::Icon)
            .refine_layout(LayoutRefinement::default().h_full())
            .into_element(cx);

            let summary = shadcn::card(|cx| {
                ui::children![
                    cx;
                    shadcn::card_header(
                        |cx| ui::children![cx; shadcn::card_title("Structure / composable parts")],
                    ),
                    shadcn::card_content(|cx| {
                        vec![
                            shadcn::SidebarTrigger::new().into_element(cx),
                            cx.text(format!("search={search_value:?}")),
                            cx.text(format!("selected={}", selected_value.as_ref())),
                            cx.text(format!("last_action={}", last_action_value.as_ref())),
                            cx.text(format!("collapsed={collapsed}")),
                            cx.text("This example consolidates the upstream Header/Footer/Content/Group/Menu/Action/Sub/Rail docs into one copyable Fret snippet."),
                            cx.text("It also keeps the official SidebarGroup collapsible-label lane copyable via SidebarGroupLabel::as_child(true) + CollapsibleTriggerPart."),
                        ]
                    }),
                ]
            })
            .refine_layout(LayoutRefinement::default().w_full().h_full().min_w_0())
            .into_element(cx);

            let inset = shadcn::SidebarInset::new([summary]).into_element(cx);
            let frame = ui::h_flex(|_cx| vec![sidebar, inset])
                .gap(Space::N4)
                .items_start()
                .layout(LayoutRefinement::default().w_full().h_px(Px(360.0)))
                .into_element(cx);

            vec![frame]
        });

    content
        .into_iter()
        .next()
        .unwrap_or_else(|| cx.text("Missing sidebar structure content."))
}

// endregion: example

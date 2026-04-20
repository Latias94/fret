pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::app::AppRenderActionsExt as _;
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui::element::SemanticsDecoration;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

fn resolve_selected<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: &Model<Arc<str>>,
    fallback: &'static str,
) -> Arc<str> {
    cx.get_model_cloned(model, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from(fallback))
}

fn menu_button(
    cx: &mut AppComponentCx<'_>,
    selected_model: Model<Arc<str>>,
    active_value: &Arc<str>,
    value: &'static str,
    label: &'static str,
    icon: &'static str,
    test_id: Arc<str>,
) -> shadcn::SidebarMenuButton {
    let collapsed = shadcn::use_sidebar(cx)
        .is_some_and(|ctx| ctx.device_shell_mode.is_desktop() && ctx.collapsed());
    let is_active = active_value.as_ref() == value;
    let selected_for_activate = selected_model.clone();
    let value_for_activate: Arc<str> = Arc::from(value);

    shadcn::SidebarMenuButton::new(label)
        .icon(fret_icons::IconId::new_static(icon))
        .active(is_active)
        .collapsed(collapsed)
        .on_activate(cx.actions().listen(move |host, action_cx| {
            let _ = host
                .models_mut()
                .update(&selected_for_activate, |v| *v = value_for_activate.clone());
            host.request_redraw(action_cx.window);
        }))
        .test_id(test_id)
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let selected_model = cx.local_model_keyed("selected", || Arc::<str>::from("playground"));

    let content = shadcn::SidebarProvider::new().with(cx, |cx| {
        let selected_value = resolve_selected(cx, &selected_model, "playground");
        let collapsed = shadcn::use_sidebar(cx)
            .is_some_and(|ctx| ctx.device_shell_mode.is_desktop() && ctx.collapsed());

        let header = ui::h_row(|cx| {
            vec![
                shadcn::SidebarTrigger::new()
                    .test_id("ui-gallery-sidebar-demo-toggle")
                    .into_element(cx),
                shadcn::Button::new("Focus")
                    .variant(shadcn::ButtonVariant::Ghost)
                    .size(shadcn::ButtonSize::Sm)
                    .test_id("ui-gallery-sidebar-demo-focus")
                    .into_element(cx),
                shadcn::raw::typography::muted(if collapsed {
                    "Collapsed to icon rail"
                } else {
                    "Expanded"
                })
                .into_element(cx),
                shadcn::raw::typography::muted(format!("active={}", selected_value.as_ref()))
                    .into_element(cx),
            ]
        })
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

        let platform = shadcn::SidebarGroup::new([
            shadcn::SidebarGroupLabel::new("Platform").into_element(cx),
            shadcn::SidebarMenu::new([
                shadcn::SidebarMenuItem::new(
                    menu_button(
                        cx,
                        selected_model.clone(),
                        &selected_value,
                        "playground",
                        "Playground",
                        "lucide.square-terminal",
                        Arc::from("ui-gallery-sidebar-demo-item-playground"),
                    )
                    .into_element(cx),
                )
                .into_element(cx),
                shadcn::SidebarMenuItem::new(
                    menu_button(
                        cx,
                        selected_model.clone(),
                        &selected_value,
                        "models",
                        "Models",
                        "lucide.bot",
                        Arc::from("ui-gallery-sidebar-demo-item-models"),
                    )
                    .into_element(cx),
                )
                .into_element(cx),
                shadcn::SidebarMenuItem::new(
                    menu_button(
                        cx,
                        selected_model.clone(),
                        &selected_value,
                        "documentation",
                        "Documentation",
                        "lucide.book-open",
                        Arc::from("ui-gallery-sidebar-demo-item-documentation"),
                    )
                    .into_element(cx),
                )
                .into_element(cx),
                shadcn::SidebarMenuItem::new(
                    menu_button(
                        cx,
                        selected_model.clone(),
                        &selected_value,
                        "settings",
                        "Settings",
                        "lucide.settings-2",
                        Arc::from("ui-gallery-sidebar-demo-item-settings"),
                    )
                    .into_element(cx),
                )
                .into_element(cx),
            ])
            .into_element(cx),
        ])
        .into_element(cx);

        let projects = shadcn::SidebarGroup::new([
            shadcn::SidebarGroupLabel::new("Projects").into_element(cx),
            shadcn::SidebarMenu::new([
                shadcn::SidebarMenuItem::new(
                    menu_button(
                        cx,
                        selected_model.clone(),
                        &selected_value,
                        "design-engineering",
                        "Design Engineering",
                        "lucide.frame",
                        Arc::from("ui-gallery-sidebar-demo-item-design-engineering"),
                    )
                    .into_element(cx),
                )
                .into_element(cx),
                shadcn::SidebarMenuItem::new(
                    menu_button(
                        cx,
                        selected_model.clone(),
                        &selected_value,
                        "sales-marketing",
                        "Sales & Marketing",
                        "lucide.chart-pie",
                        Arc::from("ui-gallery-sidebar-demo-item-sales-marketing"),
                    )
                    .into_element(cx),
                )
                .into_element(cx),
                shadcn::SidebarMenuItem::new(
                    menu_button(
                        cx,
                        selected_model.clone(),
                        &selected_value,
                        "travel",
                        "Travel",
                        "lucide.map",
                        Arc::from("ui-gallery-sidebar-demo-item-travel"),
                    )
                    .into_element(cx),
                )
                .into_element(cx),
            ])
            .into_element(cx),
        ])
        .into_element(cx);

        let sidebar = shadcn::Sidebar::new([
            shadcn::SidebarHeader::new([
                shadcn::raw::typography::small("Acme Inc.").into_element(cx)
            ])
            .into_element(cx),
            shadcn::SidebarContent::new([platform, projects]).into_element(cx),
            shadcn::SidebarFooter::new([shadcn::raw::typography::small("shadcn").into_element(cx)])
                .into_element(cx),
        ])
        .collapsible(shadcn::SidebarCollapsible::Icon)
        .refine_layout(LayoutRefinement::default().h_full())
        .into_element(cx)
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sidebar-demo-sidebar"),
        );

        let content = shadcn::card(|cx| {
            ui::children![
                cx;
                shadcn::card_header(|cx| ui::children![cx; shadcn::card_title("Content")]),
                shadcn::card_content(|cx| {
                    vec![
                        cx.text("A sidebar that collapses to icon mode."),
                        cx.text("Select any menu item to verify active and hover states."),
                    ]
                }),
            ]
        })
        .refine_layout(LayoutRefinement::default().w_full().h_full().min_w_0())
        .into_element(cx);

        let inset = shadcn::SidebarInset::new([ui::v_flex(|_cx| vec![header, content])
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full().h_full())
            .into_element(cx)])
        .into_element(cx);

        let frame = ui::h_flex(|_cx| vec![sidebar, inset])
            .gap(Space::N4)
            .items_start()
            .layout(LayoutRefinement::default().w_full().h_px(Px(360.0)))
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(fret_core::SemanticsRole::Group)
                    .test_id("ui-gallery-sidebar-demo"),
            );

        vec![frame]
    });

    content
        .into_iter()
        .next()
        .unwrap_or_else(|| cx.text("Missing sidebar demo content."))
}

// endregion: example

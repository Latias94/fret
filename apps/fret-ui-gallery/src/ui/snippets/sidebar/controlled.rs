pub const SOURCE: &str = include_str!("controlled.rs");

// region: example
use fret::{UiChild, UiCx};
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

fn menu_button<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    selected_model: Model<Arc<str>>,
    active_value: &Arc<str>,
    value: &'static str,
    label: &'static str,
    icon: &'static str,
    test_id: Arc<str>,
) -> impl IntoUiElement<H> + use<H> {
    let collapsed = shadcn::use_sidebar(cx).is_some_and(|ctx| !ctx.is_mobile && ctx.collapsed());
    let is_active = active_value.as_ref() == value;
    let selected_for_activate = selected_model.clone();
    let value_for_activate: Arc<str> = Arc::from(value);
    let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
        let _ = host
            .models_mut()
            .update(&selected_for_activate, |v| *v = value_for_activate.clone());
        host.request_redraw(action_cx.window);
    });

    shadcn::SidebarMenuButton::new(label)
        .icon(fret_icons::IconId::new_static(icon))
        .active(is_active)
        .collapsed(collapsed)
        .on_activate(on_activate)
        .test_id(test_id)
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("open", || true);
    let selected = cx.local_model_keyed("selected", || Arc::<str>::from("design-engineering"));

    let content = shadcn::SidebarProvider::new()
        .open(Some(open.clone()))
        .with(cx, |cx| {
            let open_now = cx
                .get_model_cloned(&open, Invalidation::Paint)
                .unwrap_or(true);
            let selected_value = resolve_selected(cx, &selected, "design-engineering");

            let open_for_toggle = open.clone();
            let on_toggle_open: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    let _ = host.models_mut().update(&open_for_toggle, |v| *v = !*v);
                    host.request_redraw(action_cx.window);
                });

            let header = ui::h_flex(|cx| {
                vec![
                    shadcn::Button::new(if open_now {
                        "Close Sidebar"
                    } else {
                        "Open Sidebar"
                    })
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(on_toggle_open.clone())
                    .test_id("ui-gallery-sidebar-controlled-toggle")
                    .into_element(cx),
                    shadcn::raw::typography::muted(
                        "Controlled via SidebarProvider.open(Some(model)).",
                    )
                    .into_element(cx),
                ]
            })
            .gap(Space::N2)
            .items_center()
            .layout(LayoutRefinement::default().w_full())
            .into_element(cx);

            let projects = shadcn::SidebarGroup::new([
                shadcn::SidebarGroupLabel::new("Projects").into_element(cx),
                shadcn::SidebarMenu::new([
                    shadcn::SidebarMenuItem::new(
                        menu_button(
                            cx,
                            selected.clone(),
                            &selected_value,
                            "design-engineering",
                            "Design Engineering",
                            "lucide.frame",
                            Arc::from("ui-gallery-sidebar-controlled-item-design-engineering"),
                        )
                        .into_element(cx),
                    )
                    .into_element(cx),
                    shadcn::SidebarMenuItem::new(
                        menu_button(
                            cx,
                            selected.clone(),
                            &selected_value,
                            "sales-marketing",
                            "Sales & Marketing",
                            "lucide.chart-pie",
                            Arc::from("ui-gallery-sidebar-controlled-item-sales-marketing"),
                        )
                        .into_element(cx),
                    )
                    .into_element(cx),
                    shadcn::SidebarMenuItem::new(
                        menu_button(
                            cx,
                            selected.clone(),
                            &selected_value,
                            "travel",
                            "Travel",
                            "lucide.map",
                            Arc::from("ui-gallery-sidebar-controlled-item-travel"),
                        )
                        .into_element(cx),
                    )
                    .into_element(cx),
                    shadcn::SidebarMenuItem::new(
                        menu_button(
                            cx,
                            selected.clone(),
                            &selected_value,
                            "support",
                            "Support",
                            "lucide.life-buoy",
                            Arc::from("ui-gallery-sidebar-controlled-item-support"),
                        )
                        .into_element(cx),
                    )
                    .into_element(cx),
                    shadcn::SidebarMenuItem::new(
                        menu_button(
                            cx,
                            selected.clone(),
                            &selected_value,
                            "feedback",
                            "Feedback",
                            "lucide.send",
                            Arc::from("ui-gallery-sidebar-controlled-item-feedback"),
                        )
                        .into_element(cx),
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
            ])
            .into_element(cx);

            let sidebar =
                shadcn::Sidebar::new([shadcn::SidebarContent::new([projects]).into_element(cx)])
                    .collapsible(shadcn::SidebarCollapsible::Offcanvas)
                    .refine_layout(LayoutRefinement::default().h_full())
                    .into_element(cx)
                    .attach_semantics(
                        SemanticsDecoration::default()
                            .role(fret_core::SemanticsRole::Group)
                            .test_id("ui-gallery-sidebar-controlled-sidebar"),
                    );

            let trigger = shadcn::SidebarTrigger::new()
                .test_id("ui-gallery-sidebar-controlled-trigger")
                .into_element(cx);

            let inset = shadcn::SidebarInset::new([shadcn::card(|cx| {
                ui::children![
                    cx;
                    shadcn::card_header(
                        |cx| ui::children![cx; shadcn::card_title("Sidebar Inset")],
                    ),
                    shadcn::card_content(|cx| {
                        vec![
                            cx.text("Use a main content panel next to Sidebar when controlled."),
                            cx.text(format!("open={}", open_now)),
                            cx.text(format!("selected={}", selected_value.as_ref())),
                        ]
                    }),
                ]
            })
            .refine_layout(LayoutRefinement::default().w_full().h_full().min_w_0())
            .into_element(cx)])
            .into_element(cx);

            let frame = ui::h_flex(|_cx| vec![sidebar, inset])
                .gap(Space::N4)
                .items_start()
                .layout(LayoutRefinement::default().w_full().h_px(Px(320.0)))
                .into_element(cx)
                .attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Group)
                        .test_id("ui-gallery-sidebar-controlled"),
                );

            let body = ui::v_flex(|_cx| vec![header, trigger, frame])
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full())
                .into_element(cx);

            vec![body]
        });

    content
        .into_iter()
        .next()
        .unwrap_or_else(|| cx.text("Missing sidebar controlled content."))
}

// endregion: example

pub const SOURCE: &str = include_str!("mobile.rs");

// region: example
use fret::app::AppActivateExt as _;
use fret::{UiChild, UiCx};
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

    shadcn::SidebarMenuButton::new(label)
        .icon(fret_icons::IconId::new_static(icon))
        .active(is_active)
        .collapsed(collapsed)
        .listen(move |host, action_cx| {
            let _ = host
                .models_mut()
                .update(&selected_for_activate, |v| *v = value_for_activate.clone());
            host.request_redraw(action_cx.window);
        })
        .test_id(test_id)
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open_mobile = cx.local_model_keyed("open_mobile", || false);
    let selected = cx.local_model_keyed("selected", || Arc::<str>::from("playground"));

    let content = shadcn::SidebarProvider::new()
        .default_open(false)
        .open_mobile(Some(open_mobile.clone()))
        .is_mobile(true)
        .with(cx, |cx| {
            let open_mobile_now = cx
                .get_model_cloned(&open_mobile, Invalidation::Paint)
                .unwrap_or(false);
            let selected_value = resolve_selected(cx, &selected, "playground");

            let open_mobile_for_toggle = open_mobile.clone();

            let header = ui::h_flex(|cx| {
                    vec![
                        shadcn::SidebarTrigger::new()
                            .test_id("ui-gallery-sidebar-mobile-trigger")
                            .into_element(cx),
                        shadcn::Button::new("Toggle open_mobile")
                            .variant(shadcn::ButtonVariant::Ghost)
                            .size(shadcn::ButtonSize::Sm)
                            .listen({
                                let open_mobile_for_toggle = open_mobile_for_toggle.clone();
                                move |host, action_cx| {
                                    let _ = host
                                        .models_mut()
                                        .update(&open_mobile_for_toggle, |v| *v = !*v);
                                    host.request_redraw(action_cx.window);
                                }
                            })
                            .test_id("ui-gallery-sidebar-mobile-external-toggle")
                            .into_element(cx),
                        shadcn::raw::typography::muted(
                            "Forced mobile mode via SidebarProvider.is_mobile(true).",
                        ).into_element(cx),
                        shadcn::raw::typography::muted( format!("open_mobile={open_mobile_now}")).into_element(cx),
                        shadcn::raw::typography::muted( format!("selected={}", selected_value.as_ref())).into_element(cx),
                    ]
                })
                    .gap(Space::N2)
                    .items_center()
                    .layout(LayoutRefinement::default().w_full()).into_element(cx);

            let projects = shadcn::SidebarGroup::new([
                shadcn::SidebarGroupLabel::new("Projects").into_element(cx),
                shadcn::SidebarMenu::new([
                    shadcn::SidebarMenuItem::new(menu_button(
                        cx,
                        selected.clone(),
                        &selected_value,
                        "playground",
                        "Playground",
                        "lucide.square-terminal",
                        Arc::from("ui-gallery-sidebar-mobile-item-playground"),
                    )
                    .into_element(cx))
                    .into_element(cx),
                    shadcn::SidebarMenuItem::new(menu_button(
                        cx,
                        selected.clone(),
                        &selected_value,
                        "documentation",
                        "Documentation",
                        "lucide.book-open",
                        Arc::from("ui-gallery-sidebar-mobile-item-documentation"),
                    )
                    .into_element(cx))
                    .into_element(cx),
                ])
                .into_element(cx),
            ])
            .into_element(cx);

            let sidebar = shadcn::Sidebar::new([shadcn::SidebarContent::new([projects]).into_element(cx)])
                .collapsible(shadcn::SidebarCollapsible::Offcanvas)
                .refine_layout(LayoutRefinement::default().h_full())
                .into_element(cx);

            let main = shadcn::card(|cx| {
                ui::children![
                    cx;
                    shadcn::card_header(|cx| ui::children![cx; shadcn::card_title("Mobile Sheet")]),
                    shadcn::card_content(|cx| {
                        vec![
                            cx.text(
                                "Open the sidebar via SidebarTrigger. Escape should close and restore focus.",
                            ),
                            shadcn::Button::new("Focus")
                                .variant(shadcn::ButtonVariant::Ghost)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-sidebar-mobile-focus")
                                .into_element(cx),
                        ]
                    }),
                ]
            })
            .refine_layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(fret_core::SemanticsRole::Group)
                    .test_id("ui-gallery-sidebar-mobile-main"),
            );

            let body = ui::v_flex(|_cx| vec![header, sidebar, main])
                    .gap(Space::N3)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full()).into_element(cx);

            vec![body]
        });

    content
        .into_iter()
        .next()
        .unwrap_or_else(|| cx.text("Missing sidebar mobile content."))
}

// endregion: example

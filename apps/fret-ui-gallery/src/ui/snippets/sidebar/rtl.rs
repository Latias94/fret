pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::app::AppActivateExt as _;
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui::element::SemanticsDecoration;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

fn resolve_selected(
    cx: &mut UiCx<'_>,
    model: &Model<Arc<str>>,
    fallback: &'static str,
) -> Arc<str> {
    cx.get_model_cloned(model, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from(fallback))
}

fn menu_button(
    cx: &mut UiCx<'_>,
    selected_model: Model<Arc<str>>,
    active_value: &Arc<str>,
    value: &'static str,
    label: &'static str,
    icon: &'static str,
    test_id: Arc<str>,
) -> impl IntoUiElement<fret_app::App> + use<> {
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
    let selected_model = cx.local_model_keyed("selected", || Arc::<str>::from("playground"));
    let selected_value = resolve_selected(cx, &selected_model, "playground");

    shadcn::DirectionProvider::new(shadcn::LayoutDirection::Rtl).into_element(cx, |cx| {
        let platform = shadcn::SidebarGroup::new([
            shadcn::SidebarGroupLabel::new("المنصة")
                .collapsed(false)
                .into_element(cx),
            shadcn::SidebarMenu::new([
                shadcn::SidebarMenuItem::new(
                    menu_button(
                        cx,
                        selected_model.clone(),
                        &selected_value,
                        "playground",
                        "ساحة اللعب",
                        "lucide.square-terminal",
                        Arc::from("ui-gallery-sidebar-rtl-item-playground"),
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
                        "الوثائق",
                        "lucide.book-open",
                        Arc::from("ui-gallery-sidebar-rtl-item-documentation"),
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
                        "الإعدادات",
                        "lucide.settings-2",
                        Arc::from("ui-gallery-sidebar-rtl-item-settings"),
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
                shadcn::raw::typography::small("مؤسسة مثال").into_element(cx)
            ])
            .into_element(cx),
            shadcn::SidebarContent::new([platform])
                .collapsed(false)
                .into_element(cx),
            shadcn::SidebarFooter::new([shadcn::raw::typography::small("الدعم").into_element(cx)])
                .into_element(cx),
        ])
        .collapsed(false)
        .refine_layout(LayoutRefinement::default().h_full())
        .into_element(cx);

        let content = shadcn::card(|cx| {
            ui::children![
                cx;
                shadcn::card_header(|cx| ui::children![cx; shadcn::card_title("RTL")]),
                shadcn::card_content(|cx| {
                    vec![
                        cx.text("This section validates RTL direction + icon alignment."),
                        cx.text(format!("active={}", selected_value.as_ref())),
                    ]
                }),
            ]
        })
        .refine_layout(LayoutRefinement::default().w_full().h_full().min_w_0())
        .into_element(cx);

        ui::h_flex(|_cx| vec![content, sidebar])
            .gap(Space::N4)
            .items_start()
            .layout(LayoutRefinement::default().w_full().h_px(Px(320.0)))
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(fret_core::SemanticsRole::Group)
                    .test_id("ui-gallery-sidebar-rtl"),
            )
    })
}

// endregion: example

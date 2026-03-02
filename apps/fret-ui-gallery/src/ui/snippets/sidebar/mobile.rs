pub const SOURCE: &str = include_str!("mobile.rs");

// region: example
use fret_ui::element::SemanticsDecoration;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default, Clone)]
struct SidebarModels {
    open_mobile: Option<Model<bool>>,
    selected: Option<Model<Arc<str>>>,
}

fn ensure_models<H: UiHost>(cx: &mut ElementContext<'_, H>) -> (Model<bool>, Model<Arc<str>>) {
    let state = cx.with_state(SidebarModels::default, |st| st.clone());
    match (state.open_mobile, state.selected) {
        (Some(open_mobile), Some(selected)) => (open_mobile, selected),
        _ => {
            let models = cx.app.models_mut();
            let open_mobile = models.insert(false);
            let selected = models.insert(Arc::<str>::from("playground"));
            cx.with_state(SidebarModels::default, |st| {
                st.open_mobile = Some(open_mobile.clone());
                st.selected = Some(selected.clone());
            });
            (open_mobile, selected)
        }
    }
}

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
) -> AnyElement {
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
        .into_element(cx)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let (open_mobile, selected) = ensure_models(cx);

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
            let on_toggle_open_mobile: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    let _ = host.models_mut().update(&open_mobile_for_toggle, |v| *v = !*v);
                    host.request_redraw(action_cx.window);
                });

            let header = stack::hstack(
                cx,
                stack::HStackProps::default()
                    .gap(Space::N2)
                    .items_center()
                    .layout(LayoutRefinement::default().w_full()),
                |cx| {
                    vec![
                        shadcn::SidebarTrigger::new()
                            .test_id("ui-gallery-sidebar-mobile-trigger")
                            .into_element(cx),
                        shadcn::Button::new("Toggle open_mobile")
                            .variant(shadcn::ButtonVariant::Ghost)
                            .size(shadcn::ButtonSize::Sm)
                            .on_activate(on_toggle_open_mobile)
                            .test_id("ui-gallery-sidebar-mobile-external-toggle")
                            .into_element(cx),
                        shadcn::typography::muted(
                            cx,
                            "Forced mobile mode via SidebarProvider.is_mobile(true).",
                        ),
                        shadcn::typography::muted(cx, format!("open_mobile={open_mobile_now}")),
                        shadcn::typography::muted(cx, format!("selected={}", selected_value.as_ref())),
                    ]
                },
            );

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
                    ))
                    .into_element(cx),
                    shadcn::SidebarMenuItem::new(menu_button(
                        cx,
                        selected.clone(),
                        &selected_value,
                        "documentation",
                        "Documentation",
                        "lucide.book-open",
                        Arc::from("ui-gallery-sidebar-mobile-item-documentation"),
                    ))
                    .into_element(cx),
                ])
                .into_element(cx),
            ])
            .into_element(cx);

            let sidebar = shadcn::Sidebar::new([shadcn::SidebarContent::new([projects]).into_element(cx)])
                .collapsible(shadcn::SidebarCollapsible::Offcanvas)
                .refine_layout(LayoutRefinement::default().h_full())
                .into_element(cx);

            let main = shadcn::Card::new(vec![
                shadcn::CardHeader::new(vec![
                    shadcn::CardTitle::new("Mobile Sheet").into_element(cx),
                ])
                .into_element(cx),
                shadcn::CardContent::new(vec![
                    cx.text(
                        "Open the sidebar via SidebarTrigger. Escape should close and restore focus.",
                    ),
                    shadcn::Button::new("Focus")
                        .variant(shadcn::ButtonVariant::Ghost)
                        .size(shadcn::ButtonSize::Sm)
                        .test_id("ui-gallery-sidebar-mobile-focus")
                        .into_element(cx),
                ])
                .into_element(cx),
            ])
            .refine_layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(fret_core::SemanticsRole::Group)
                    .test_id("ui-gallery-sidebar-mobile-main"),
            );

            let body = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N3)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full()),
                |_cx| vec![header, sidebar, main],
            );

            vec![body]
        });

    content
        .into_iter()
        .next()
        .unwrap_or_else(|| cx.text("Missing sidebar mobile content."))
}

// endregion: example

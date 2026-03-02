// region: example
use fret_core::Px;
use fret_ui::element::SemanticsDecoration;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default, Clone)]
struct SidebarModels {
    selected: Option<Model<Arc<str>>>,
}

fn ensure_selected<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<Arc<str>> {
    let state = cx.with_state(SidebarModels::default, |st| st.clone());
    match state.selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Arc::<str>::from("playground"));
            cx.with_state(SidebarModels::default, |st| st.selected = Some(model.clone()));
            model
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
    let selected_model = ensure_selected(cx);

    let content = shadcn::SidebarProvider::new().with(cx, |cx| {
        let selected_value = resolve_selected(cx, &selected_model, "playground");
        let collapsed = shadcn::use_sidebar(cx).is_some_and(|ctx| !ctx.is_mobile && ctx.collapsed());

        let header = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |cx| {
                vec![
                    shadcn::SidebarTrigger::new()
                        .test_id("ui-gallery-sidebar-demo-toggle")
                        .into_element(cx),
                    shadcn::Button::new("Focus")
                        .variant(shadcn::ButtonVariant::Ghost)
                        .size(shadcn::ButtonSize::Sm)
                        .test_id("ui-gallery-sidebar-demo-focus")
                        .into_element(cx),
                    shadcn::typography::muted(
                        cx,
                        if collapsed {
                            "Collapsed to icon rail"
                        } else {
                            "Expanded"
                        },
                    ),
                    shadcn::typography::muted(
                        cx,
                        format!("active={}", selected_value.as_ref()),
                    ),
                ]
            },
        );

        let platform = shadcn::SidebarGroup::new([
            shadcn::SidebarGroupLabel::new("Platform").into_element(cx),
            shadcn::SidebarMenu::new([
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    selected_model.clone(),
                    &selected_value,
                    "playground",
                    "Playground",
                    "lucide.square-terminal",
                    Arc::from("ui-gallery-sidebar-demo-item-playground"),
                ))
                .into_element(cx),
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    selected_model.clone(),
                    &selected_value,
                    "models",
                    "Models",
                    "lucide.bot",
                    Arc::from("ui-gallery-sidebar-demo-item-models"),
                ))
                .into_element(cx),
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    selected_model.clone(),
                    &selected_value,
                    "documentation",
                    "Documentation",
                    "lucide.book-open",
                    Arc::from("ui-gallery-sidebar-demo-item-documentation"),
                ))
                .into_element(cx),
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    selected_model.clone(),
                    &selected_value,
                    "settings",
                    "Settings",
                    "lucide.settings-2",
                    Arc::from("ui-gallery-sidebar-demo-item-settings"),
                ))
                .into_element(cx),
            ])
            .into_element(cx),
        ])
        .into_element(cx);

        let projects = shadcn::SidebarGroup::new([
            shadcn::SidebarGroupLabel::new("Projects").into_element(cx),
            shadcn::SidebarMenu::new([
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    selected_model.clone(),
                    &selected_value,
                    "design-engineering",
                    "Design Engineering",
                    "lucide.frame",
                    Arc::from("ui-gallery-sidebar-demo-item-design-engineering"),
                ))
                .into_element(cx),
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    selected_model.clone(),
                    &selected_value,
                    "sales-marketing",
                    "Sales & Marketing",
                    "lucide.chart-pie",
                    Arc::from("ui-gallery-sidebar-demo-item-sales-marketing"),
                ))
                .into_element(cx),
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    selected_model.clone(),
                    &selected_value,
                    "travel",
                    "Travel",
                    "lucide.map",
                    Arc::from("ui-gallery-sidebar-demo-item-travel"),
                ))
                .into_element(cx),
            ])
            .into_element(cx),
        ])
        .into_element(cx);

        let sidebar = shadcn::Sidebar::new([
            shadcn::SidebarHeader::new([shadcn::typography::small(cx, "Acme Inc.")])
                .into_element(cx),
            shadcn::SidebarContent::new([platform, projects]).into_element(cx),
            shadcn::SidebarFooter::new([shadcn::typography::small(cx, "shadcn")]).into_element(cx),
        ])
        .collapsible(shadcn::SidebarCollapsible::Icon)
        .refine_layout(LayoutRefinement::default().h_full())
        .into_element(cx)
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sidebar-demo-sidebar"),
        );

        let content = shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![shadcn::CardTitle::new("Content").into_element(cx)])
                .into_element(cx),
            shadcn::CardContent::new(vec![
                cx.text("A sidebar that collapses to icon mode."),
                cx.text("Select any menu item to verify active and hover states."),
            ])
            .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().h_full().min_w_0())
        .into_element(cx);

        let inset = shadcn::SidebarInset::new([stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full().h_full()),
            |_cx| vec![header, content],
        )])
        .into_element(cx);

        let frame = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N4)
                .items_start()
                .layout(LayoutRefinement::default().w_full().h_px(Px(360.0))),
            |_cx| vec![sidebar, inset],
        )
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


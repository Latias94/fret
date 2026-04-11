pub const SOURCE: &str = include_str!("responsive.rs");

// region: example
use fret::children::UiElementSinkExt;
use fret::component::prelude::Model;
use fret::{UiChild, UiCx};
use fret_ui::element::AnyElement;
use fret_ui::Invalidation;
use fret_ui_kit::adaptive::{device_shell_mode, device_shell_switch, DeviceShellSwitchPolicy};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use shadcn::raw::breadcrumb::primitives as bc;
use std::sync::Arc;

const ITEMS_TO_DISPLAY: usize = 3;

fn toggle_model_on_activate(open: Model<bool>) -> fret_ui::action::OnActivate {
    Arc::new(move |host, acx, _reason| {
        let _ = host.models_mut().update(&open, |v| *v = !*v);
        host.request_redraw(acx.window);
    })
}

fn close_model_on_activate(open: Model<bool>) -> fret_ui::action::OnActivate {
    Arc::new(move |host, acx, _reason| {
        let _ = host.models_mut().update(&open, |v| *v = false);
        host.request_redraw(acx.window);
    })
}

fn breadcrumb_trigger_props(trigger_test_id: Arc<str>) -> fret_ui::element::PressableProps {
    let mut props = fret_ui::element::PressableProps::default();
    props.a11y.role = Some(fret_core::SemanticsRole::Button);
    props.a11y.label = Some(Arc::from("Toggle menu"));
    props.a11y.test_id = Some(trigger_test_id);
    props
}

fn render_breadcrumb_overflow_trigger(cx: &mut UiCx<'_>, trigger_test_id: Arc<str>) -> AnyElement {
    cx.pressable(breadcrumb_trigger_props(trigger_test_id), move |cx, _st| {
        vec![bc::BreadcrumbEllipsis::new()
            .size(fret_core::Px(16.0))
            .into_element(cx)]
    })
}

fn render_breadcrumb_overflow_drawer_trigger(
    cx: &mut UiCx<'_>,
    trigger_test_id: Arc<str>,
    toggle: fret_ui::action::OnActivate,
) -> AnyElement {
    cx.pressable(breadcrumb_trigger_props(trigger_test_id), move |cx, _st| {
        cx.pressable_on_activate(toggle.clone());
        vec![bc::BreadcrumbEllipsis::new()
            .size(fret_core::Px(16.0))
            .into_element(cx)]
    })
}

fn breadcrumb_dropdown_entries(
    overflow_items: &[(&str, Option<&str>)],
) -> Vec<shadcn::DropdownMenuEntry> {
    overflow_items
        .iter()
        .map(|(label, _href)| {
            if *label == "Documentation" {
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new(*label)
                        .on_activate(Arc::new(|_host, _acx, _reason| {}))
                        .test_id("ui-gallery-breadcrumb-responsive-menu-docs"),
                )
            } else {
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new(*label)
                        .on_activate(Arc::new(|_host, _acx, _reason| {})),
                )
            }
        })
        .collect::<Vec<_>>()
}

fn render_breadcrumb_drawer_content(
    cx: &mut UiCx<'_>,
    overflow_items: &[(&str, Option<&str>)],
    close: fret_ui::action::OnActivate,
) -> AnyElement {
    let body = cx.container(
        fret_ui::element::ContainerProps {
            layout: Default::default(),
            padding: fret_core::Edges::all(fret_core::Px(16.0)).into(),
            ..Default::default()
        },
        move |cx| {
            vec![ui::v_stack(move |cx| {
                overflow_items
                    .iter()
                    .map(|(label, _href)| fret_ui_kit::ui::text(*label).into_element(cx))
                    .collect::<Vec<_>>()
            })
            .gap(Space::N1)
            .items_stretch()
            .into_element(cx)]
        },
    );

    shadcn::DrawerContent::build(|cx, out| {
        out.push_ui(
            cx,
            shadcn::DrawerHeader::build(|cx, out| {
                out.push_ui(cx, shadcn::DrawerTitle::new("Navigate to"));
                out.push_ui(
                    cx,
                    shadcn::DrawerDescription::new("Select a page to navigate to."),
                );
            }),
        );
        out.push(body);
        out.push_ui(
            cx,
            shadcn::DrawerFooter::build(|cx, out| {
                out.push_ui(
                    cx,
                    shadcn::Button::new("Close")
                        .variant(shadcn::ButtonVariant::Outline)
                        .on_activate(close.clone())
                        .test_id("ui-gallery-breadcrumb-responsive-drawer-close"),
                );
            }),
        );
    })
    .into_element(cx)
    .test_id("ui-gallery-breadcrumb-responsive-drawer-content")
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model(|| false);
    let shell_policy = DeviceShellSwitchPolicy::default();
    let is_desktop = device_shell_mode(cx, Invalidation::Layout, shell_policy).is_desktop();

    // Matches shadcn/ui v4 `breadcrumb-responsive` structure.
    let items: [(&str, Option<&str>); 5] = [
        ("Home", Some("#")),
        ("Documentation", Some("#")),
        ("Building Your Application", Some("#")),
        ("Data Fetching", Some("#")),
        ("Caching and Revalidating", None),
    ];

    let overflow_items = &items[1..items.len().saturating_sub(2)];
    let tail_items = &items[items.len().saturating_sub(ITEMS_TO_DISPLAY - 1)..];

    let tail_max_w = if is_desktop {
        LayoutRefinement::default()
    } else {
        LayoutRefinement::default().max_w(fret_core::Px(80.0))
    };

    let trigger_test_id: Arc<str> = Arc::from("ui-gallery-breadcrumb-responsive-trigger");

    let crumb = bc::Breadcrumb::new().into_element(cx, |cx| {
        vec![bc::BreadcrumbList::new().into_element(cx, |cx| {
            let mut out: Vec<AnyElement> = Vec::new();

            out.push(bc::BreadcrumbItem::new().into_element(cx, |cx| {
                vec![bc::BreadcrumbLink::new(items[0].0)
                    .href(items[0].1.unwrap_or("#"))
                    .on_activate(Arc::new(|_host, _acx, _reason| {}))
                    .into_element(cx)]
            }));
            out.push(bc::BreadcrumbSeparator::new().into_element(cx));

            if items.len() > ITEMS_TO_DISPLAY {
                out.push(bc::BreadcrumbItem::new().into_element(cx, |cx| {
                    let desktop_trigger_test_id = trigger_test_id.clone();
                    let mobile_trigger_test_id = trigger_test_id.clone();
                    vec![device_shell_switch(
                        cx,
                        Invalidation::Layout,
                        shell_policy,
                        |cx| {
                            let dropdown = shadcn::DropdownMenu::from_open(open.clone())
                                .align(shadcn::DropdownMenuAlign::Start);
                            dropdown.into_element(
                                cx,
                                move |cx| {
                                    render_breadcrumb_overflow_trigger(
                                        cx,
                                        desktop_trigger_test_id.clone(),
                                    )
                                },
                                |_cx| breadcrumb_dropdown_entries(overflow_items),
                            )
                        },
                        |cx| {
                            let drawer = shadcn::Drawer::new(open.clone());
                            let toggle = toggle_model_on_activate(open.clone());
                            let close = close_model_on_activate(open.clone());
                            drawer.into_element(
                                cx,
                                move |cx| {
                                    render_breadcrumb_overflow_drawer_trigger(
                                        cx,
                                        mobile_trigger_test_id.clone(),
                                        toggle.clone(),
                                    )
                                },
                                move |cx| {
                                    render_breadcrumb_drawer_content(
                                        cx,
                                        overflow_items,
                                        close.clone(),
                                    )
                                },
                            )
                        },
                    )]
                }));
                out.push(bc::BreadcrumbSeparator::new().into_element(cx));
            }

            // Tail items (last `ITEMS_TO_DISPLAY - 1` entries).
            for (i, (label, href)) in tail_items.iter().enumerate() {
                let is_last = i + 1 == tail_items.len();
                let tail_layout = tail_max_w.clone();
                out.push(bc::BreadcrumbItem::new().into_element(cx, |cx| {
                    if is_last || href.is_none() {
                        vec![bc::BreadcrumbPage::new(*label)
                            .truncate(true)
                            .refine_layout(tail_layout.clone())
                            .into_element(cx)]
                    } else {
                        vec![bc::BreadcrumbLink::new(*label)
                            .href(href.unwrap_or("#"))
                            .truncate(true)
                            .refine_layout(tail_layout.clone())
                            .on_activate(Arc::new(|_host, _acx, _reason| {}))
                            .into_element(cx)]
                    }
                }));

                if !is_last {
                    out.push(bc::BreadcrumbSeparator::new().into_element(cx));
                }
            }

            out
        })]
    });

    crumb.test_id("ui-gallery-breadcrumb-responsive")
}
// endregion: example

pub const SOURCE: &str = include_str!("responsive.rs");

// region: example
use fret_app::App;
use fret_ui::Invalidation;
use fret_ui_shadcn::breadcrumb::primitives as bc;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

const ITEMS_TO_DISPLAY: usize = 3;

#[derive(Default, Clone)]
struct Models {
    open: Option<Model<bool>>,
}

fn open_model(cx: &mut ElementContext<'_, App>) -> Model<bool> {
    let state = cx.with_state(Models::default, |st| st.clone());
    match state.open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| st.open = Some(model.clone()));
            model
        }
    }
}

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

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let open = open_model(cx);

    let is_desktop = fret_ui_kit::declarative::viewport_queries::viewport_width_at_least(
        cx,
        Invalidation::Layout,
        fret_ui_kit::declarative::viewport_queries::tailwind::MD,
        fret_ui_kit::declarative::viewport_queries::ViewportQueryHysteresis::default(),
    );

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
                vec![
                    bc::BreadcrumbLink::new(items[0].0)
                        .href(items[0].1.unwrap_or("#"))
                        .on_activate(Arc::new(|_host, _acx, _reason| {}))
                        .into_element(cx),
                ]
            }));
            out.push(bc::BreadcrumbSeparator::new().into_element(cx));

            if items.len() > ITEMS_TO_DISPLAY {
                out.push(bc::BreadcrumbItem::new().into_element(cx, |cx| {
                    if is_desktop {
                        let dropdown = shadcn::DropdownMenu::new(open.clone())
                            .align(shadcn::DropdownMenuAlign::Start);
                        vec![dropdown.into_element(
                            cx,
                            |cx| {
                                let mut props = fret_ui::element::PressableProps::default();
                                props.a11y.role = Some(fret_core::SemanticsRole::Button);
                                props.a11y.label = Some(Arc::from("Toggle menu"));
                                props.a11y.test_id = Some(trigger_test_id.clone());

                                cx.pressable(props, move |cx, _st| {
                                    vec![
                                        bc::BreadcrumbEllipsis::new()
                                            .size(fret_core::Px(16.0))
                                            .into_element(cx),
                                    ]
                                })
                            },
                            |_cx| {
                                overflow_items
                                    .iter()
                                    .map(|(label, _href)| {
                                        if *label == "Documentation" {
                                            shadcn::DropdownMenuEntry::Item(
                                                shadcn::DropdownMenuItem::new(*label)
                                                    .on_activate(Arc::new(
                                                        |_host, _acx, _reason| {},
                                                    ))
                                                    .test_id(
                                                        "ui-gallery-breadcrumb-responsive-menu-docs",
                                                    ),
                                            )
                                        } else {
                                            shadcn::DropdownMenuEntry::Item(
                                                shadcn::DropdownMenuItem::new(*label).on_activate(
                                                    Arc::new(|_host, _acx, _reason| {}),
                                                ),
                                            )
                                        }
                                    })
                                    .collect::<Vec<_>>()
                            },
                        )]
                    } else {
                        let drawer = shadcn::Drawer::new(open.clone());
                        let toggle = toggle_model_on_activate(open.clone());
                        let close = close_model_on_activate(open.clone());
                        vec![drawer.into_element(
                            cx,
                            move |cx| {
                                let mut props = fret_ui::element::PressableProps::default();
                                props.a11y.role = Some(fret_core::SemanticsRole::Button);
                                props.a11y.label = Some(Arc::from("Toggle menu"));
                                props.a11y.test_id = Some(trigger_test_id.clone());

                                cx.pressable(props, move |cx, _st| {
                                    cx.pressable_on_activate(toggle.clone());
                                    vec![
                                        bc::BreadcrumbEllipsis::new()
                                            .size(fret_core::Px(16.0))
                                            .into_element(cx),
                                    ]
                                })
                            },
                            move |cx| {
                                shadcn::DrawerContent::new([
                                    shadcn::DrawerHeader::new([
                                        shadcn::DrawerTitle::new("Navigate to").into_element(cx),
                                        shadcn::DrawerDescription::new(
                                            "Select a page to navigate to.",
                                        )
                                        .into_element(cx),
                                    ])
                                    .into_element(cx),
                                    cx.container(
                                        fret_ui::element::ContainerProps {
                                            layout: Default::default(),
                                            padding: fret_core::Edges::all(fret_core::Px(16.0))
                                                .into(),
                                            ..Default::default()
                                        },
                                        move |cx| {
                                            vec![stack::vstack(
                                                cx,
                                                stack::VStackProps::default()
                                                    .gap(Space::N1)
                                                    .items_stretch(),
                                                move |cx| {
                                                    overflow_items
                                                        .iter()
                                                        .map(|(label, _href)| {
                                                            fret_ui_kit::ui::text(cx, *label)
                                                                .into_element(cx)
                                                        })
                                                        .collect::<Vec<_>>()
                                                },
                                            )]
                                        },
                                    ),
                                    shadcn::DrawerFooter::new([shadcn::Button::new("Close")
                                        .variant(shadcn::ButtonVariant::Outline)
                                        .on_activate(close.clone())
                                        .test_id(
                                            "ui-gallery-breadcrumb-responsive-drawer-close",
                                        )
                                        .into_element(cx)])
                                    .into_element(cx),
                                ])
                                .into_element(cx)
                                .test_id("ui-gallery-breadcrumb-responsive-drawer-content")
                            },
                        )]
                    }
                }));
                out.push(bc::BreadcrumbSeparator::new().into_element(cx));
            }

            // Tail items (last `ITEMS_TO_DISPLAY - 1` entries).
            for (i, (label, href)) in tail_items.iter().enumerate() {
                let is_last = i + 1 == tail_items.len();
                let tail_layout = tail_max_w.clone();
                out.push(bc::BreadcrumbItem::new().into_element(cx, |cx| {
                    if is_last || href.is_none() {
                        vec![
                            bc::BreadcrumbPage::new(*label)
                                .truncate(true)
                                .refine_layout(tail_layout.clone())
                                .into_element(cx),
                        ]
                    } else {
                        vec![
                            bc::BreadcrumbLink::new(*label)
                                .href(href.unwrap_or("#"))
                                .truncate(true)
                                .refine_layout(tail_layout.clone())
                                .on_activate(Arc::new(|_host, _acx, _reason| {}))
                                .into_element(cx),
                        ]
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

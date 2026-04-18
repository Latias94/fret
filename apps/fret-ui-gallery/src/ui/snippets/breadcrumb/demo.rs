pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
#[allow(unused_imports)]
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use shadcn::raw::breadcrumb::primitives as bc;
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model(|| false);
    let dropdown =
        shadcn::DropdownMenu::from_open(open.clone()).align(shadcn::DropdownMenuAlign::Start);

    let crumb = bc::Breadcrumb::new().into_element(cx, |cx| {
        vec![bc::BreadcrumbList::new().into_element(cx, |cx| {
            vec![
                bc::BreadcrumbItem::new().into_element(cx, |cx| {
                    vec![
                        bc::BreadcrumbLink::new("Home")
                            .href("/")
                            .on_activate(Arc::new(|_host, _acx, _reason| {}))
                            .into_element(cx),
                    ]
                }),
                bc::BreadcrumbSeparator::new().into_element(cx),
                bc::BreadcrumbItem::new().into_element(cx, |cx| {
                    vec![dropdown.into_element(
                        cx,
                        |cx| {
                            let mut props = fret_ui::element::PressableProps::default();
                            props.a11y.role = Some(fret_core::SemanticsRole::Button);
                            props.a11y.label = Some(Arc::from("Toggle menu"));
                            props.a11y.test_id =
                                Some(Arc::from("ui-gallery-breadcrumb-demo-ellipsis-trigger"));

                            cx.pressable(props, move |cx, _st| {
                                vec![
                                    bc::BreadcrumbEllipsis::new()
                                        .size(fret_core::Px(16.0))
                                        .into_element(cx),
                                ]
                            })
                        },
                        |_cx| {
                            vec![
                                shadcn::DropdownMenuEntry::Item(
                                    shadcn::DropdownMenuItem::new("Documentation")
                                        .on_activate(Arc::new(|_host, _acx, _reason| {}))
                                        .test_id("ui-gallery-breadcrumb-demo-menu-docs"),
                                ),
                                shadcn::DropdownMenuEntry::Item(
                                    shadcn::DropdownMenuItem::new("Themes")
                                        .on_activate(Arc::new(|_host, _acx, _reason| {})),
                                ),
                                shadcn::DropdownMenuEntry::Item(
                                    shadcn::DropdownMenuItem::new("GitHub")
                                        .on_activate(Arc::new(|_host, _acx, _reason| {})),
                                ),
                            ]
                        },
                    )]
                }),
                bc::BreadcrumbSeparator::new().into_element(cx),
                bc::BreadcrumbItem::new().into_element(cx, |cx| {
                    vec![
                        bc::BreadcrumbLink::new("Components")
                            .href("/docs/components")
                            .on_activate(Arc::new(|_host, _acx, _reason| {}))
                            .into_element(cx),
                    ]
                }),
                bc::BreadcrumbSeparator::new().into_element(cx),
                bc::BreadcrumbItem::new().into_element(cx, |cx| {
                    vec![bc::BreadcrumbPage::new("Breadcrumb").into_element(cx)]
                }),
            ]
        })]
    });

    crumb.test_id("ui-gallery-breadcrumb-demo")
}
// endregion: example

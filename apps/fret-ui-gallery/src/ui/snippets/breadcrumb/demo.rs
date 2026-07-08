pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
#[allow(unused_imports)]
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model(|| false);
    let dropdown = shadcn::DropdownMenu::from_open(open.clone())
        .align(shadcn::DropdownMenuAlign::Start)
        .test_id_prefix("ui-gallery-breadcrumb-demo-menu");

    let crumb = shadcn::BreadcrumbRoot::new().into_element(cx, |cx| {
        vec![shadcn::BreadcrumbList::new().into_element(cx, |cx| {
            vec![
                shadcn::BreadcrumbItemPart::new().into_element(cx, |cx| {
                    vec![
                        shadcn::BreadcrumbLink::new("Home")
                            .href("/")
                            .on_activate(Arc::new(|_host, _acx, _reason| {}))
                            .into_element(cx),
                    ]
                }),
                shadcn::BreadcrumbSeparatorPart::new().into_element(cx),
                shadcn::BreadcrumbItemPart::new().into_element(cx, |cx| {
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
                                    shadcn::BreadcrumbEllipsis::new()
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
                                        .on_activate(Arc::new(|_host, _acx, _reason| {}))
                                        .test_id("ui-gallery-breadcrumb-demo-menu-themes"),
                                ),
                                shadcn::DropdownMenuEntry::Item(
                                    shadcn::DropdownMenuItem::new("GitHub")
                                        .on_activate(Arc::new(|_host, _acx, _reason| {}))
                                        .test_id("ui-gallery-breadcrumb-demo-menu-github"),
                                ),
                            ]
                        },
                    )]
                }),
                shadcn::BreadcrumbSeparatorPart::new().into_element(cx),
                shadcn::BreadcrumbItemPart::new().into_element(cx, |cx| {
                    vec![
                        shadcn::BreadcrumbLink::new("Components")
                            .href("/docs/components")
                            .on_activate(Arc::new(|_host, _acx, _reason| {}))
                            .into_element(cx),
                    ]
                }),
                shadcn::BreadcrumbSeparatorPart::new().into_element(cx),
                shadcn::BreadcrumbItemPart::new().into_element(cx, |cx| {
                    vec![shadcn::BreadcrumbPage::new("Breadcrumb").into_element(cx)]
                }),
            ]
        })]
    });

    crumb.test_id("ui-gallery-breadcrumb-demo")
}
// endregion: example

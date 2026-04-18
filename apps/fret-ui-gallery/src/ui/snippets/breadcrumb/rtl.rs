pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui::Theme;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use shadcn::raw::breadcrumb::primitives as bc;
use std::sync::Arc;

fn dot_separator<H: UiHost>(cx: &mut ElementContext<'_, H>) -> impl IntoUiElement<H> + use<H> {
    bc::BreadcrumbSeparator::new()
        .children(|cx| {
            [shadcn::raw::icon::icon_with(
                cx,
                fret_icons::IconId::new_static("lucide.dot"),
                Some(Px(14.0)),
                None,
            )]
        })
        .into_element(cx)
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model(|| false);

    with_direction_provider(cx, LayoutDirection::Rtl, move |cx| {
        let crumb = bc::Breadcrumb::new().into_element(cx, |cx| {
            let list = bc::BreadcrumbList::new().into_element(cx, |cx| {
                let home = bc::BreadcrumbItem::new().into_element(cx, |cx| {
                    vec![
                        bc::BreadcrumbLink::new("الرئيسية")
                            .on_activate(Arc::new(|_host, _acx, _reason| {}))
                            .into_element(cx),
                    ]
                });

                let components_dropdown = bc::BreadcrumbItem::new().into_element(cx, |cx| {
                    let menu = shadcn::DropdownMenu::from_open(open.clone())
                        .align(shadcn::DropdownMenuAlign::End)
                        .into_element(
                            cx,
                            move |cx| {
                                let theme = Theme::global(&*cx.app);
                                let fg = theme.color_token("foreground");
                                let muted = theme.color_token("muted-foreground");
                                let mut props = fret_ui::element::PressableProps::default();
                                props.a11y.label = Some(Arc::<str>::from("المكونات"));
                                props.a11y.test_id = Some(Arc::<str>::from(
                                    "ui-gallery-breadcrumb-rtl-dropdown-trigger",
                                ));

                                cx.pressable(props, move |cx, st| {
                                    let color = if st.hovered { fg } else { muted };
                                    let label = fret_ui_kit::ui::text("المكونات")
                                        .text_color(fret_ui_kit::ColorRef::Color(color))
                                        .nowrap()
                                        .into_element(cx);
                                    let chevron = shadcn::raw::icon::icon_with(
                                        cx,
                                        fret_icons::IconId::new_static("lucide.chevron-down"),
                                        Some(Px(14.0)),
                                        Some(fret_ui_kit::ColorRef::Color(color)),
                                    );

                                    vec![
                                        ui::h_row(move |_cx| vec![chevron, label])
                                            .gap(Space::N1)
                                            .items_center()
                                            .into_element(cx),
                                    ]
                                })
                            },
                            |_cx| {
                                vec![
                                    shadcn::DropdownMenuGroup::new([
                                        shadcn::DropdownMenuItem::new("التوثيق")
                                            .on_activate(Arc::new(|_host, _acx, _reason| {}))
                                            .test_id("ui-gallery-breadcrumb-rtl-dropdown-docs")
                                            .into(),
                                        shadcn::DropdownMenuItem::new("السمات")
                                            .on_activate(Arc::new(|_host, _acx, _reason| {}))
                                            .into(),
                                        shadcn::DropdownMenuItem::new("جيت هاب")
                                            .on_activate(Arc::new(|_host, _acx, _reason| {}))
                                            .into(),
                                    ])
                                    .into(),
                                ]
                            },
                        );
                    vec![menu]
                });

                let page = bc::BreadcrumbItem::new().into_element(cx, |cx| {
                    vec![bc::BreadcrumbPage::new("مسار التنقل").into_element(cx)]
                });

                vec![
                    home,
                    dot_separator(cx).into_element(cx),
                    components_dropdown,
                    dot_separator(cx).into_element(cx),
                    page,
                ]
            });

            vec![list]
        });

        crumb.test_id("ui-gallery-breadcrumb-rtl")
    })
}
// endregion: example

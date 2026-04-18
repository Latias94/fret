pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui::Theme;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn repository_item<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    test_id: &'static str,
    name: &'static str,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).snapshot();
    let props = decl_style::container_props(
        &theme,
        ChromeRefinement::default()
            .border_1()
            .rounded(Radius::Md)
            .px(Space::N4)
            .py(Space::N2),
        LayoutRefinement::default().w_full().min_w_0(),
    );

    cx.container(props, move |cx| {
        vec![shadcn::raw::typography::small(name).into_element(cx)]
    })
    .test_id(test_id)
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    // Mirrors the official shadcn/ui v4 `collapsible-demo.tsx` repository list example.
    cx.scope(|cx| {
        let open = cx.local_model_keyed("demo_open", || false);

        shadcn::CollapsibleRoot::new()
            .open(open.clone())
            .gap(Space::N2)
            .refine_layout(LayoutRefinement::default().w_px(Px(350.0)).min_w_0())
            .into_element(cx, move |cx| {
                let header = {
                    let title = shadcn::raw::typography::small("@peduarte starred 3 repositories")
                        .into_element(cx);
                    let button = shadcn::Button::new("")
                        .a11y_label("Toggle details")
                        .variant(shadcn::ButtonVariant::Ghost)
                        .size(shadcn::ButtonSize::Icon)
                        .refine_layout(LayoutRefinement::default().w_px(Px(32.0)).h_px(Px(32.0)))
                        .icon(fret_icons::IconId::new_static("lucide.chevrons-up-down"))
                        .test_id("ui-gallery-collapsible-demo-trigger")
                        .into_element(cx);
                    let trigger = shadcn::CollapsibleTriggerPart::new([button])
                        .as_child(true)
                        .into_element(cx);

                    let row = ui::h_flex(move |_cx| vec![title, trigger])
                        .gap(Space::N4)
                        .items_center()
                        .justify_between()
                        .layout(LayoutRefinement::default().w_full().min_w_0())
                        .into_element(cx);
                    let theme = Theme::global(&*cx.app).snapshot();
                    let props = decl_style::container_props(
                        &theme,
                        ChromeRefinement::default().px(Space::N4),
                        LayoutRefinement::default().w_full().min_w_0(),
                    );
                    cx.container(props, move |_cx| vec![row])
                };

                let repository_primitives = repository_item(
                    cx,
                    "ui-gallery-collapsible-demo-repo-primitives",
                    "@radix-ui/primitives",
                );

                let content = shadcn::CollapsibleContentPart::new([
                    repository_item(
                        cx,
                        "ui-gallery-collapsible-demo-repo-colors",
                        "@radix-ui/colors",
                    ),
                    repository_item(
                        cx,
                        "ui-gallery-collapsible-demo-repo-stitches",
                        "@stitches/react",
                    ),
                ])
                .gap(Space::N2)
                .test_id("ui-gallery-collapsible-demo-content")
                .into_element(cx);

                vec![header, repository_primitives, content]
            })
            .test_id("ui-gallery-collapsible-demo")
    })
}
// endregion: example

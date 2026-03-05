pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_ui_kit::declarative::controllable_state;
use fret_ui_shadcn::collapsible_primitives as shadcn_col;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    // Mirrors upstream shadcn/ui v4 `collapsible-demo.tsx` composition: free structure with
    // `Trigger(asChild)` in the header and `Content` later in the tree.
    cx.scope(|cx| {
        let open =
            controllable_state::use_controllable_model(cx, None::<Model<bool>>, || false).model();

        shadcn_col::Collapsible::new()
            .open(open)
            .gap(Space::N2)
            .refine_layout(LayoutRefinement::default().w_px(Px(350.0)))
            .into_element(cx, move |cx| {
                let header = {
                    let title = shadcn::typography::small(cx, "@peduarte starred 3 repositories");
                    let button = shadcn::Button::new("")
                        .a11y_label("Toggle")
                        .variant(shadcn::ButtonVariant::Ghost)
                        .size(shadcn::ButtonSize::Icon)
                        .refine_layout(LayoutRefinement::default().w_px(Px(32.0)).h_px(Px(32.0)))
                        .icon(fret_icons::IconId::new_static("lucide.chevrons-up-down"))
                        .test_id("ui-gallery-collapsible-demo-trigger")
                        .into_element(cx);
                    let trigger = shadcn_col::CollapsibleTrigger::new([button])
                        .as_child(true)
                        .into_element(cx);

                    let row = stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .gap_x(Space::N4)
                            .items_center()
                            .justify_between()
                            .layout(LayoutRefinement::default().w_full().min_w_0()),
                        move |_cx| vec![title, trigger],
                    );
                    let theme = Theme::global(&*cx.app).snapshot();
                    let props = shadcn::decl_style::container_props(
                        &theme,
                        ChromeRefinement::default().px(Space::N4),
                        LayoutRefinement::default().w_full().min_w_0(),
                    );
                    cx.container(props, move |_cx| vec![row])
                };

                let item = |cx: &mut ElementContext<'_, H>, text: &'static str| {
                    let theme = Theme::global(&*cx.app).snapshot();
                    let props = shadcn::decl_style::container_props(
                        &theme,
                        ChromeRefinement::default()
                            .border_1()
                            .rounded(Radius::Md)
                            .px(Space::N4)
                            .py(Space::N2),
                        LayoutRefinement::default().w_full(),
                    );
                    cx.container(props, move |cx| vec![cx.text(text)])
                };

                let content = shadcn_col::CollapsibleContent::new([
                    item(cx, "@radix-ui/colors"),
                    item(cx, "@stitches/react"),
                ])
                .gap(Space::N2)
                .test_id("ui-gallery-collapsible-demo-content")
                .into_element(cx);

                vec![header, item(cx, "@radix-ui/primitives"), content]
            })
            .test_id("ui-gallery-collapsible-demo")
    })
}
// endregion: example

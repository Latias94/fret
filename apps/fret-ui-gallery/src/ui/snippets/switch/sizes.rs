// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default)]
struct Models {
    size_small: Option<Model<bool>>,
    size_default: Option<Model<bool>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let (size_small, size_default) = cx.with_state(Models::default, |st| {
        (st.size_small.clone(), st.size_default.clone())
    });

    let (size_small, size_default) = match (size_small, size_default) {
        (Some(size_small), Some(size_default)) => (size_small, size_default),
        _ => {
            let size_small = cx.app.models_mut().insert(false);
            let size_default = cx.app.models_mut().insert(true);
            cx.with_state(Models::default, |st| {
                st.size_small = Some(size_small.clone());
                st.size_default = Some(size_default.clone());
            });
            (size_small, size_default)
        }
    };

    let small = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Switch::new(size_small)
                    .a11y_label("Small switch")
                    .size(shadcn::SwitchSize::Sm)
                    .test_id("ui-gallery-switch-size-small")
                    .into_element(cx),
                shadcn::Label::new("Small").into_element(cx),
            ]
        },
    )
    .test_id("ui-gallery-switch-sizes-sm");

    let default = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Switch::new(size_default)
                    .a11y_label("Default switch")
                    .test_id("ui-gallery-switch-size-default")
                    .into_element(cx),
                shadcn::Label::new("Default").into_element(cx),
            ]
        },
    )
    .test_id("ui-gallery-switch-sizes-default");

    stack::hstack(
        cx,
        stack::HStackProps::default()
            .gap(Space::N4)
            .items_center()
            .layout(LayoutRefinement::default().w_full().min_w_0().max_w(Px(520.0))),
        |_cx| vec![small, default],
    )
    .test_id("ui-gallery-switch-sizes")
}

// endregion: example


pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let email = cx.local_model(String::new);
    let max_w = LayoutRefinement::default().w_full().max_w(Px(420.0));
    let id = ControlId::from("ui-gallery-label-email");

    ui::v_stack(|cx| {
        vec![
            shadcn::Label::new("Your email address")
                .for_control(id.clone())
                .test_id("ui-gallery-label-demo-label")
                .into_element(cx),
            shadcn::Input::new(email)
                .placeholder("you@example.com")
                .control_id(id)
                .test_id("ui-gallery-label-demo-input")
                .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_start()
    .layout(max_w)
    .into_element(cx)
    .test_id("ui-gallery-label-demo")
}
// endregion: example

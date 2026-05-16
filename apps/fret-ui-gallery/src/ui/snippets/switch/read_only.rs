pub const SOURCE: &str = include_str!("read_only.rs");

// region: example
use fret::app::AppRenderActionsExt;
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_kit::declarative::ModelWatchExt;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let checked = cx.local_model(|| true);
    let read_only = cx.local_model(|| true);
    let read_only_now = cx.watch_model(&read_only).copied().unwrap_or(true);
    let control_id = ControlId::from("ui-gallery-switch-read-only");

    let field = shadcn::Field::new([
        shadcn::FieldContent::new([
            shadcn::FieldLabel::new("Managed by policy")
                .for_control(control_id.clone())
                .test_id("ui-gallery-switch-read-only-label")
                .into_element(cx),
            shadcn::FieldDescription::new(
                "This setting is visible and focusable, but cannot be changed here.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::Switch::new(checked)
            .control_id(control_id)
            .read_only(read_only_now)
            .a11y_label("Managed by policy")
            .test_id("ui-gallery-switch-read-only-control")
            .into_element(cx),
    ])
    .orientation(shadcn::FieldOrientation::Horizontal)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(360.0)))
    .into_element(cx)
    .test_id("ui-gallery-switch-read-only-field");

    let read_only_for_toggle = read_only.clone();
    let toggle = shadcn::Button::new(if read_only_now {
        "Allow changes"
    } else {
        "Lock changes"
    })
    .variant(shadcn::ButtonVariant::Outline)
    .size(shadcn::ButtonSize::Sm)
    .on_activate(cx.actions().listen(move |host, action_cx| {
        let _ = host
            .models_mut()
            .update(&read_only_for_toggle, |value| *value = !*value);
        host.request_redraw(action_cx.window);
    }))
    .test_id("ui-gallery-switch-read-only-policy-toggle")
    .into_element(cx);

    ui::v_stack(|_cx| vec![field, toggle])
        .gap(Space::N2)
        .items_start()
        .layout(LayoutRefinement::default().w_full().max_w(Px(360.0)))
        .into_element(cx)
        .test_id("ui-gallery-switch-read-only")
}
// endregion: example

pub const SOURCE: &str = include_str!("checked_state.rs");

// region: example
use fret::app::UiCxActionsExt as _;
use fret::{AppComponentCx, UiChild};
use fret_ui::Invalidation;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

mod act {
    fret::actions!([ToggleSnapshot = "ui-gallery.checkbox.toggle_snapshot.v1"]);
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let checked_controlled = cx.local_model_keyed("checked_controlled", || true);
    let checked_optional = cx.local_model_keyed("checked_optional", || None::<bool>);
    let checked_snapshot = cx.local_model_keyed("checked_snapshot", || false);

    let checked_snapshot_now = cx
        .get_model_copied(&checked_snapshot, Invalidation::Layout)
        .unwrap_or(false);

    ui::v_flex(|cx| {
        cx.actions().models::<act::ToggleSnapshot>({
            let checked_snapshot = checked_snapshot.clone();
            move |models| {
                models
                    .update(&checked_snapshot, |value| *value = !*value)
                    .is_ok()
            }
        });

        vec![
            ui::h_flex(|cx| {
                vec![
                    shadcn::Checkbox::new(checked_controlled)
                        .control_id("ui-gallery-checkbox-controlled")
                        .a11y_label("Controlled checkbox")
                        .test_id("ui-gallery-checkbox-controlled")
                        .into_element(cx),
                    shadcn::FieldLabel::new("Controlled checked state")
                        .for_control("ui-gallery-checkbox-controlled")
                        .test_id("ui-gallery-checkbox-controlled-label")
                        .into_element(cx),
                ]
            })
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N3)
            .items_center()
            .into_element(cx),
            ui::h_flex(|cx| {
                vec![
                    shadcn::Checkbox::new_optional(checked_optional)
                        .control_id("ui-gallery-checkbox-optional")
                        .a11y_label("Optional checkbox")
                        .test_id("ui-gallery-checkbox-optional")
                        .into_element(cx),
                    shadcn::FieldLabel::new("Optional / indeterminate state")
                        .for_control("ui-gallery-checkbox-optional")
                        .test_id("ui-gallery-checkbox-optional-label")
                        .into_element(cx),
                ]
            })
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N3)
            .items_center()
            .into_element(cx),
            shadcn::Field::new([
                shadcn::Checkbox::from_checked(checked_snapshot_now)
                    .control_id("ui-gallery-checkbox-snapshot")
                    .a11y_label("Snapshot checkbox")
                    .action(act::ToggleSnapshot)
                    .test_id("ui-gallery-checkbox-snapshot")
                    .into_element(cx),
                shadcn::FieldContent::new([
                    shadcn::FieldLabel::new("Snapshot / action-first state")
                        .for_control("ui-gallery-checkbox-snapshot")
                        .test_id("ui-gallery-checkbox-snapshot-label")
                        .into_element(cx),
                    shadcn::FieldDescription::new(
                        "Render from a plain bool and dispatch a typed action on toggle.",
                    )
                    .test_id("ui-gallery-checkbox-snapshot-description")
                    .into_element(cx),
                ])
                .into_element(cx),
            ])
            .orientation(shadcn::FieldOrientation::Horizontal)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
        ]
    })
    .gap(Space::N3)
    .items_start()
    .layout(LayoutRefinement::default().w_full().max_w(Px(420.0)))
    .into_element(cx)
    .test_id("ui-gallery-checkbox-checked-state")
}
// endregion: example

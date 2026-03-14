pub const SOURCE: &str = include_str!("time_picker.rs");

// region: example
use std::sync::Arc;

use fret::{UiChild, UiCx};
use fret_ui::action::OnActivate;
use fret_ui_material3 as material3;
use fret_ui_shadcn::prelude::*;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let dialog = material3::TimePickerDialog::uncontrolled(cx);
    let open = dialog.open_model();
    let selected = dialog.selected_model();

    let open_dialog: OnActivate = {
        let open = open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = true);
            host.request_redraw(action_cx.window);
        })
    };

    let selected_value = cx
        .get_model_copied(&selected, Invalidation::Layout)
        .unwrap_or_else(|| time::Time::from_hms(9, 41, 0).expect("valid time"));
    let selected_label: Arc<str> = Arc::from(format!(
        "Selected: {:02}:{:02}",
        selected_value.hour(),
        selected_value.minute()
    ));

    dialog
        .test_id("ui-gallery-material3-time-picker")
        .into_element(cx, move |cx| {
            ui::v_stack(move |cx| {
                    let docked = material3::DockedTimePicker::new(selected.clone())
                        .test_id("ui-gallery-material3-time-picker-docked")
                        .into_element(cx);

                    vec![
                        cx.text(
                            "Material 3 Time Picker: primitives driven by md.comp.time-picker.* tokens.",
                        ),
                        cx.text(selected_label.clone()),
                        material3::Button::new("Open time picker dialog")
                            .variant(material3::ButtonVariant::Filled)
                            .on_activate(open_dialog.clone())
                            .test_id("ui-gallery-material3-time-picker-open")
                            .into_element(cx),
                        material3::Button::new("Underlay focus probe")
                            .variant(material3::ButtonVariant::Outlined)
                            .test_id("ui-gallery-material3-time-picker-underlay-probe")
                            .into_element(cx),
                        docked,
                    ]
                }).gap(Space::N4).into_element(cx)
        })
}

// endregion: example

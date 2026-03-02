pub const SOURCE: &str = include_str!("date_picker.rs");

// region: example
use std::sync::Arc;

use fret_ui::action::OnActivate;
use fret_ui_headless::calendar::CalendarMonth;
use fret_ui_material3 as material3;
use fret_ui_shadcn::prelude::*;

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    month: Model<CalendarMonth>,
    selected: Model<Option<time::Date>>,
) -> AnyElement {
    let open_dialog: OnActivate = {
        let open = open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = true);
            host.request_redraw(action_cx.window);
        })
    };

    let selected_value = cx
        .get_model_cloned(&selected, Invalidation::Layout)
        .unwrap_or(None);
    let selected_label: Arc<str> = match selected_value {
        Some(date) => Arc::from(format!("Selected: {date}")),
        None => Arc::<str>::from("Selected: <none>"),
    };

    material3::DatePickerDialog::new(open, month.clone(), selected.clone())
        .test_id("ui-gallery-material3-date-picker")
        .into_element(cx, move |cx| {
            stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N4),
                move |cx| {
                    let docked = material3::DockedDatePicker::new(month.clone(), selected.clone())
                        .test_id("ui-gallery-material3-date-picker-docked")
                        .into_element(cx);

                    vec![
                        cx.text(
                            "Material 3 Date Picker: primitives driven by md.comp.date-picker.* tokens.",
                        ),
                        cx.text(selected_label.clone()),
                        material3::Button::new("Open date picker dialog")
                            .variant(material3::ButtonVariant::Filled)
                            .on_activate(open_dialog.clone())
                            .test_id("ui-gallery-material3-date-picker-open")
                            .into_element(cx),
                        material3::Button::new("Underlay focus probe")
                            .variant(material3::ButtonVariant::Outlined)
                            .test_id("ui-gallery-material3-date-picker-underlay-probe")
                            .into_element(cx),
                        docked,
                    ]
                },
            )
        })
}

// endregion: example

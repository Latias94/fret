use super::super::super::super::*;

pub(in crate::ui) fn preview_material3_date_picker(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    month: Model<fret_ui_headless::calendar::CalendarMonth>,
    selected: Model<Option<time::Date>>,
) -> Vec<AnyElement> {
    use fret_ui::action::OnActivate;
    use fret_ui_material3::{Button, ButtonVariant, DatePickerDialog, DockedDatePicker};

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

    let dialog = DatePickerDialog::new(open.clone(), month.clone(), selected.clone())
        .test_id("ui-gallery-material3-date-picker")
        .into_element(cx, move |cx| {
            stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N4),
                move |cx| {
                    let docked = DockedDatePicker::new(month.clone(), selected.clone())
                        .test_id("ui-gallery-material3-date-picker-docked")
                        .into_element(cx);

                    vec![
                        cx.text(
                            "Material 3 Date Picker: primitives driven by md.comp.date-picker.* tokens.",
                        ),
                        cx.text(selected_label.clone()),
                        Button::new("Open date picker dialog")
                            .variant(ButtonVariant::Filled)
                            .on_activate(open_dialog.clone())
                            .test_id("ui-gallery-material3-date-picker-open")
                            .into_element(cx),
                        Button::new("Underlay focus probe")
                            .variant(ButtonVariant::Outlined)
                            .test_id("ui-gallery-material3-date-picker-underlay-probe")
                            .into_element(cx),
                        docked,
                    ]
                },
            )
        });

    vec![dialog]
}

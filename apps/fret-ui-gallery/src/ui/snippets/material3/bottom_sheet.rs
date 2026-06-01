pub const SOURCE: &str = include_str!("bottom_sheet.rs");

// region: example
use std::sync::Arc;

use fret::{AppComponentCx, UiChild};
use fret_ui::action::OnActivate;
use fret_ui_material3 as material3;
use fret_ui_shadcn::prelude::*;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let sheet =
        material3::ModalBottomSheet::uncontrolled(cx).test_id("ui-gallery-material3-bottom-sheet");
    let open = sheet.open_model();
    let sheet_text_field = material3::TextField::uncontrolled(cx);
    let sheet_select = material3::Select::uncontrolled(cx);
    let sheet_autocomplete = material3::Autocomplete::uncontrolled(cx);
    let sheet_autocomplete_selected =
        cx.local_model_keyed("bottom_sheet_autocomplete_selected", || None::<Arc<str>>);

    let select_items: Arc<[material3::SelectItem]> = Arc::from(vec![
        material3::SelectItem::new("alpha", "Alpha")
            .test_id("ui-gallery-material3-bottom-sheet-select-alpha"),
        material3::SelectItem::new("beta", "Beta")
            .test_id("ui-gallery-material3-bottom-sheet-select-beta"),
        material3::SelectItem::new("gamma", "Gamma")
            .test_id("ui-gallery-material3-bottom-sheet-select-gamma"),
    ]);
    let autocomplete_items: Arc<[material3::AutocompleteItem]> = Arc::from(vec![
        material3::AutocompleteItem::new("alpha", "Alpha"),
        material3::AutocompleteItem::new("beta", "Beta"),
        material3::AutocompleteItem::new("gamma", "Gamma"),
        material3::AutocompleteItem::new("delta", "Delta"),
    ]);

    let open_sheet: OnActivate = {
        let open = open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = true);
            host.request_redraw(action_cx.window);
        })
    };
    let close_sheet: OnActivate = {
        let open = open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = false);
            host.request_redraw(action_cx.window);
        })
    };

    let underlay = move |cx: &mut AppComponentCx<'_>| {
        ui::v_stack(move |cx| {
            let docked = material3::DockedBottomSheet::new()
                .variant(material3::DockedBottomSheetVariant::Standard)
                .test_id("ui-gallery-material3-bottom-sheet-docked")
                .into_element(cx, |cx| {
                    vec![
                        cx.text("Docked (standard) sheet: token-driven container + drag handle."),
                        material3::Button::new("Primary action")
                            .variant(material3::ButtonVariant::Filled)
                            .test_id("ui-gallery-material3-bottom-sheet-docked-primary")
                            .into_element(cx),
                        material3::Button::new("Secondary action")
                            .variant(material3::ButtonVariant::Outlined)
                            .test_id("ui-gallery-material3-bottom-sheet-docked-secondary")
                            .into_element(cx),
                    ]
                });

            vec![
                cx.text(
                    "Material 3 Bottom Sheet: primitives driven by md.comp.sheet.bottom.* tokens.",
                ),
                material3::Button::new("Open modal bottom sheet")
                    .variant(material3::ButtonVariant::Filled)
                    .on_activate(open_sheet.clone())
                    .test_id("ui-gallery-material3-bottom-sheet-open")
                    .into_element(cx),
                material3::Button::new("Underlay focus probe")
                    .variant(material3::ButtonVariant::Outlined)
                    .test_id("ui-gallery-material3-bottom-sheet-underlay-probe")
                    .into_element(cx),
                cx.text(
                    "Tip: click the scrim to dismiss; Tab should stay inside the sheet while open.",
                ),
                docked,
            ]
        })
        .gap(Space::N4)
        .into_element(cx)
    };

    sheet.into_element(cx, underlay, move |cx| {
        let text_field = sheet_text_field
            .variant(material3::TextFieldVariant::Outlined)
            .label("Project name")
            .placeholder("Type a name")
            .a11y_label("Bottom sheet project name")
            .test_id("ui-gallery-material3-bottom-sheet-text-field")
            .into_element(cx);

        let select = sheet_select
            .label("Project")
            .placeholder("Pick one")
            .items(select_items.clone())
            .a11y_label("Bottom sheet select")
            .test_id("ui-gallery-material3-bottom-sheet-select")
            .into_element(cx);

        let autocomplete = sheet_autocomplete
            .selected_value(sheet_autocomplete_selected.clone())
            .label("Assignee")
            .placeholder("Type to filter")
            .items(autocomplete_items.clone())
            .a11y_label("Bottom sheet autocomplete")
            .test_id("ui-gallery-material3-bottom-sheet-autocomplete")
            .into_element(cx);

        vec![
            ui::v_stack(move |cx| {
                vec![
                    text_field,
                    select,
                    autocomplete,
                    material3::Button::new("Close")
                        .variant(material3::ButtonVariant::Filled)
                        .on_activate(close_sheet.clone())
                        .test_id("ui-gallery-material3-bottom-sheet-close")
                        .into_element(cx),
                ]
            })
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4)
            .into_element(cx),
        ]
    })
}

// endregion: example

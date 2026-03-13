pub const SOURCE: &str = include_str!("bottom_sheet.rs");

// region: example
use std::sync::Arc;

use fret_ui::action::OnActivate;
use fret_ui_material3 as material3;
use fret_ui_shadcn::prelude::*;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let sheet =
        material3::ModalBottomSheet::uncontrolled(cx).test_id("ui-gallery-material3-bottom-sheet");
    let open = sheet.open_model();
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

    let underlay = move |cx: &mut ElementContext<'_, H>| {
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
        vec![
            ui::v_stack(move |cx| {
                vec![
                    cx.text("Modal bottom sheet content."),
                    material3::Button::new("Close")
                        .variant(material3::ButtonVariant::Filled)
                        .on_activate(close_sheet.clone())
                        .test_id("ui-gallery-material3-bottom-sheet-close")
                        .into_element(cx),
                ]
            })
            .gap(Space::N4)
            .into_element(cx),
        ]
    })
}

// endregion: example

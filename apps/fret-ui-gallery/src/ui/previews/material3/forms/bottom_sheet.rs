use super::super::super::super::*;

pub(in crate::ui) fn preview_material3_bottom_sheet(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
) -> Vec<AnyElement> {
    use fret_ui::action::OnActivate;
    use fret_ui_material3::{
        Button, ButtonVariant, DockedBottomSheet, DockedBottomSheetVariant, ModalBottomSheet,
    };

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

    let underlay = move |cx: &mut ElementContext<'_, App>| {
        stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N4),
            move |cx| {
                let docked =
                    DockedBottomSheet::new()
                        .variant(DockedBottomSheetVariant::Standard)
                        .test_id("ui-gallery-material3-bottom-sheet-docked")
                        .into_element(cx, |cx| {
                            vec![
                        cx.text("Docked (standard) sheet: token-driven container + drag handle."),
                        Button::new("Primary action")
                            .variant(ButtonVariant::Filled)
                            .test_id("ui-gallery-material3-bottom-sheet-docked-primary")
                            .into_element(cx),
                        Button::new("Secondary action")
                            .variant(ButtonVariant::Outlined)
                            .test_id("ui-gallery-material3-bottom-sheet-docked-secondary")
                            .into_element(cx),
                    ]
                        });

                vec![
                cx.text(
                    "Material 3 Bottom Sheet: primitives driven by md.comp.sheet.bottom.* tokens.",
                ),
                Button::new("Open modal bottom sheet")
                    .variant(ButtonVariant::Filled)
                    .on_activate(open_sheet.clone())
                    .test_id("ui-gallery-material3-bottom-sheet-open")
                    .into_element(cx),
                Button::new("Underlay focus probe")
                    .variant(ButtonVariant::Outlined)
                    .test_id("ui-gallery-material3-bottom-sheet-underlay-probe")
                    .into_element(cx),
                cx.text(
                    "Tip: click the scrim to dismiss; Tab should stay inside the sheet while open.",
                ),
                docked,
            ]
            },
        )
    };

    let sheet = ModalBottomSheet::new(open)
        .test_id("ui-gallery-material3-bottom-sheet")
        .into_element(cx, underlay, move |cx| {
            vec![stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N4),
                move |cx| {
                    vec![
                        cx.text("Modal bottom sheet content."),
                        Button::new("Close")
                            .variant(ButtonVariant::Filled)
                            .on_activate(close_sheet.clone())
                            .test_id("ui-gallery-material3-bottom-sheet-close")
                            .into_element(cx),
                    ]
                },
            )]
        });

    vec![sheet]
}

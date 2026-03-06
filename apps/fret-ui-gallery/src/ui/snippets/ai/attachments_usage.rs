pub const SOURCE: &str = include_str!("attachments_usage.rs");

// region: example
use crate::ui::snippets::aspect_ratio::landscape_image_id;
use fret_core::Px;
use fret_ui::Theme;
use fret_ui::element::{ContainerProps, InteractivityGateProps};
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;

fn render_grid_attachment<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    data: ui_ai::AttachmentData,
) -> AnyElement {
    ui_ai::Attachment::new(data)
        .variant(ui_ai::AttachmentVariant::Grid)
        .into_element_with_children(cx, move |cx, _parts| {
            let theme = Theme::global(&*cx.app).clone();
            let preview = ui_ai::AttachmentPreview::from_context().into_element(cx);
            let remove = ui_ai::AttachmentRemove::from_context().into_element(cx);

            let mut overlay = ContainerProps::default();
            overlay.layout = fret_ui_kit::declarative::style::layout_style(
                &theme,
                LayoutRefinement::default().relative().w_full().h_full(),
            );
            let remove = cx.interactivity_gate_props(
                InteractivityGateProps {
                    present: true,
                    interactive: true,
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &theme,
                        LayoutRefinement::default()
                            .absolute()
                            .top_px(Px(8.0))
                            .right_px(Px(8.0)),
                    ),
                },
                move |_cx| vec![remove],
            );

            vec![cx.container(overlay, move |_cx| vec![preview, remove])]
        })
}

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let mut image = ui_ai::AttachmentFileData::new("usage-image")
        .filename("mountain-landscape.jpg")
        .media_type("image/jpeg");
    if let Some(preview) = landscape_image_id(cx) {
        image = image.preview_image(preview);
    }

    let doc = ui_ai::AttachmentFileData::new("usage-doc")
        .filename("quarterly-report.pdf")
        .media_type("application/pdf");

    let children = vec![
        render_grid_attachment(cx, ui_ai::AttachmentData::File(image)),
        render_grid_attachment(cx, ui_ai::AttachmentData::File(doc)),
    ];

    ui::v_flex(move |cx| {
        vec![
            cx.text(
                "Display uploaded files in a message surface with a shared Attachments container.",
            ),
            ui::h_flex(move |cx| {
                vec![
                    ui_ai::Attachments::new(children)
                        .variant(ui_ai::AttachmentVariant::Grid)
                        .refine_layout(LayoutRefinement::default().min_w_0())
                        .into_element(cx),
                ]
            })
            .layout(LayoutRefinement::default().w_full())
            .justify_center()
            .into_element(cx),
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N3)
    .into_element(cx)
}
// endregion: example

pub const SOURCE: &str = include_str!("attachments_usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{ImageId, Px};
use fret_ui::Theme;
use fret_ui::element::{ContainerProps, InteractivityGateProps};
use fret_ui_ai as ui_ai;
use fret_ui_assets::ui::ImageSourceElementContextExt as _;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;

fn landscape_image_id(cx: &mut UiCx<'_>) -> Option<ImageId> {
    let request = crate::driver::demo_assets::ui_gallery_ai_attachment_landscape_request();
    cx.use_image_source_state_from_asset_request(&request).image
}

#[rustfmt::skip]
fn render_grid_attachment(
    cx: &mut UiCx<'_>,
    data: ui_ai::AttachmentData,
) -> impl UiChild + use<> {
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

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
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
        render_grid_attachment(cx, ui_ai::AttachmentData::File(image)).into_element(cx),
        render_grid_attachment(cx, ui_ai::AttachmentData::File(doc)).into_element(cx),
    ];

    ui::v_flex(move |cx| {
        vec![
            cx.text(
                "Display uploaded files in a message surface with a shared Attachments container. The image preview resolves through a gallery-owned logical bundle asset request.",
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

pub const SOURCE: &str = include_str!("attachments_usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{ImageColorSpace, ImageId, Px};
use fret_ui::Theme;
use fret_ui::element::{ContainerProps, InteractivityGateProps};
use fret_ui_ai as ui_ai;
use fret_ui_assets::{ImageSource, ui::ImageSourceElementContextExt as _};
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::OnceLock;

fn attachment_preview_rgba8(width: u32, height: u32, accent: [u8; 3]) -> Vec<u8> {
    let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
    let width_f = (width.saturating_sub(1)).max(1) as f32;
    let height_f = (height.saturating_sub(1)).max(1) as f32;

    for y in 0..height {
        for x in 0..width {
            let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
            let fx = x as f32 / width_f;
            let fy = y as f32 / height_f;

            let mut r =
                (18.0 + 42.0 * (1.0 - fy) + (accent[0] as f32) * (0.30 + 0.40 * fx)).min(255.0);
            let mut g = (22.0
                + 56.0 * (1.0 - fy.sin().abs())
                + (accent[1] as f32) * (0.24 + 0.38 * (1.0 - fy)))
                .min(255.0);
            let mut b =
                (32.0 + 58.0 * fy + (accent[2] as f32) * (0.35 + 0.42 * (1.0 - fx))).min(255.0);

            if x < 6 || y < 6 || x + 6 >= width || y + 6 >= height {
                r = 236.0;
                g = 239.0;
                b = 244.0;
            } else if y > height / 3 && y < (height * 2) / 3 && x > width / 5 && x < (width * 4) / 5
            {
                r = (r + 28.0).min(255.0);
                g = (g + 26.0).min(255.0);
                b = (b + 18.0).min(255.0);
            }

            out[idx] = r as u8;
            out[idx + 1] = g as u8;
            out[idx + 2] = b as u8;
            out[idx + 3] = 255;
        }
    }

    out
}

fn landscape_source() -> &'static ImageSource {
    static SOURCE: OnceLock<ImageSource> = OnceLock::new();
    SOURCE.get_or_init(|| {
        ImageSource::rgba8(
            320,
            180,
            attachment_preview_rgba8(320, 180, [112, 170, 226]),
            ImageColorSpace::Srgb,
        )
    })
}

fn landscape_image_id(cx: &mut UiCx<'_>) -> Option<ImageId> {
    cx.use_image_source_state(landscape_source()).image
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
                "Display uploaded files in a message surface with a shared Attachments container. The image preview comes from a deterministic in-memory RGBA source so the snippet stays self-contained.",
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

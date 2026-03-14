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

fn landscape_image_id(cx: &mut UiCx<'_>) -> Option<ImageId> {
    static SOURCE: OnceLock<ImageSource> = OnceLock::new();
    let source = SOURCE.get_or_init(|| {
        // Keep the snippet self-contained instead of depending on repo-relative demo assets.
        ImageSource::rgba8(
            320,
            192,
            demo_preview_rgba8(320, 192, (92, 168, 255)),
            ImageColorSpace::Srgb,
        )
    });
    cx.use_image_source_state(source).image
}

fn demo_preview_rgba8(width: u32, height: u32, accent: (u8, u8, u8)) -> Vec<u8> {
    let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
    let width_f = (width.saturating_sub(1)).max(1) as f32;
    let height_f = (height.saturating_sub(1)).max(1) as f32;

    for y in 0..height {
        for x in 0..width {
            let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
            let fx = x as f32 / width_f;
            let fy = y as f32 / height_f;

            let mut r = (24.0 + accent.0 as f32 * (0.30 + 0.70 * fx)) as u8;
            let mut g = (20.0 + accent.1 as f32 * (0.40 + 0.60 * (1.0 - fy))) as u8;
            let mut b = (28.0 + accent.2 as f32 * (0.35 + 0.65 * (0.5 + 0.5 * (fx - fy)))) as u8;

            let border = x < 3 || y < 3 || x + 3 >= width || y + 3 >= height;
            let horizon = y > height / 2 - 3 && y < height / 2 + 3;
            let badge = x > width / 10 && x < width / 5 && y > height / 8 && y < height / 4;

            if border {
                r = 245;
                g = 245;
                b = 245;
            } else if horizon {
                r = r.saturating_add(22);
                g = g.saturating_add(22);
                b = b.saturating_add(16);
            } else if badge {
                r = 250;
                g = 250;
                b = 250;
            }

            out[idx] = r;
            out[idx + 1] = g;
            out[idx + 2] = b;
            out[idx + 3] = 255;
        }
    }

    out
}

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

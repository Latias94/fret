pub const SOURCE: &str = include_str!("attachments_list.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{ImageColorSpace, ImageId, Px};
use fret_ui::Invalidation;
use fret_ui_ai as ui_ai;
use fret_ui_assets::{ImageSource, ui::ImageSourceElementContextExt as _};
use fret_ui_kit::{LayoutRefinement, MetricRef, Space, ui};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;
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

fn demo_items(cx: &mut UiCx<'_>) -> Vec<ui_ai::AttachmentData> {
    let mut image = ui_ai::AttachmentFileData::new("att-image")
        .filename("mountain-landscape.jpg")
        .media_type("image/jpeg");
    if let Some(preview) = landscape_image_id(cx) {
        image = image.preview_image(preview);
    }

    let pdf = ui_ai::AttachmentFileData::new("att-pdf")
        .filename("quarterly-report-2024.pdf")
        .media_type("application/pdf");
    let video = ui_ai::AttachmentFileData::new("att-video")
        .filename("product-demo.mp4")
        .media_type("video/mp4");
    let source = ui_ai::AttachmentSourceDocumentData::new("att-source")
        .title("API Documentation")
        .filename("api-reference")
        .url("https://docs.example.com/api");
    let audio = ui_ai::AttachmentFileData::new("att-audio")
        .filename("meeting-recording.mp3")
        .media_type("audio/mpeg");

    vec![
        ui_ai::AttachmentData::File(image),
        ui_ai::AttachmentData::File(pdf),
        ui_ai::AttachmentData::File(video),
        ui_ai::AttachmentData::SourceDocument(source),
        ui_ai::AttachmentData::File(audio),
    ]
}

fn render_list_attachment(
    cx: &mut UiCx<'_>,
    data: ui_ai::AttachmentData,
    on_remove: ui_ai::OnAttachmentRemove,
    test_id: Option<&'static str>,
) -> impl UiChild + use<> {
    let mut attachment = ui_ai::Attachment::new(data)
        .variant(ui_ai::AttachmentVariant::List)
        .show_media_type(true)
        .on_remove(on_remove);
    if let Some(test_id) = test_id {
        attachment = attachment.test_id(test_id);
    }

    attachment.into_element_with_children(cx, move |cx, _parts| {
        let preview = ui_ai::AttachmentPreview::from_context().into_element(cx);
        let info = ui_ai::AttachmentInfo::from_context()
            .show_media_type(true)
            .into_element(cx);
        let remove = ui_ai::AttachmentRemove::from_context().into_element(cx);

        let row = ui::h_row(move |_cx| vec![preview, info, remove])
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N3)
            .items_center()
            .into_element(cx);
        vec![row]
    })
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let removed_ids = cx.local_model_keyed("removed_ids", Vec::<Arc<str>>::new);

    let hidden = cx
        .get_model_cloned(&removed_ids, Invalidation::Layout)
        .unwrap_or_default();

    let on_remove: ui_ai::OnAttachmentRemove = Arc::new({
        let removed_ids = removed_ids.clone();
        move |host, _action_cx, id| {
            let _ = host.models_mut().update(&removed_ids, |v| {
                if !v.iter().any(|existing| existing.as_ref() == id.as_ref()) {
                    v.push(id.clone());
                }
            });
        }
    });

    let children = demo_items(cx)
        .into_iter()
        .filter(|item| !hidden.iter().any(|id| id.as_ref() == item.id().as_ref()))
        .map(|item| {
            let item_id = item.id().clone();
            let on_remove = on_remove.clone();
            let key = Arc::<str>::from(format!("attachments-list-{}", item_id.as_ref()));
            cx.keyed(key, move |cx| {
                render_list_attachment(
                    cx,
                    item.clone(),
                    on_remove.clone(),
                    (item_id.as_ref() == "att-image").then_some("ui-ai-attachment-list-att-image"),
                )
                .into_element(cx)
            })
        })
        .collect::<Vec<_>>();

    ui::h_flex(move |cx| {
        vec![
            ui_ai::Attachments::new(children)
                .variant(ui_ai::AttachmentVariant::List)
                .test_id("ui-ai-attachments-list-root")
                .refine_layout(
                    LayoutRefinement::default()
                        .w_full()
                        .max_w(MetricRef::Px(Px(440.0))),
                )
                .into_element(cx),
        ]
    })
    .layout(LayoutRefinement::default().w_full())
    .justify_center()
    .into_element(cx)
}
// endregion: example

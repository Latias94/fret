pub const SOURCE: &str = include_str!("attachments_inline.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{ImageColorSpace, ImageId, Px};
use fret_ui::Invalidation;
use fret_ui::Theme;
use fret_ui::element::{ContainerProps, InteractivityGateProps};
use fret_ui_ai as ui_ai;
use fret_ui_assets::{ImageSource, ui::ImageSourceElementContextExt as _};
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, MetricRef, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::{Arc, OnceLock};

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
                (18.0 + 44.0 * (1.0 - fy) + (accent[0] as f32) * (0.30 + 0.40 * fx)).min(255.0);
            let mut g =
                (24.0 + 42.0 * fx + (accent[1] as f32) * (0.24 + 0.40 * (1.0 - fy))).min(255.0);
            let mut b =
                (32.0 + 54.0 * fy + (accent[2] as f32) * (0.34 + 0.36 * (1.0 - fx))).min(255.0);

            if x < 6 || y < 6 || x + 6 >= width || y + 6 >= height {
                r = 236.0;
                g = 239.0;
                b = 244.0;
            } else if y > height / 3 && y < (height * 2) / 3 && x > width / 5 && x < (width * 4) / 5
            {
                r = (r + 24.0).min(255.0);
                g = (g + 20.0).min(255.0);
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

fn demo_items(cx: &mut UiCx<'_>) -> Vec<ui_ai::AttachmentData> {
    let mut image = ui_ai::AttachmentFileData::new("att-image")
        .filename("mountain-landscape.jpg")
        .media_type("image/jpeg");
    if let Some(preview) = landscape_image_id(cx) {
        image = image.preview_image(preview);
    }

    let pdf = ui_ai::AttachmentFileData::new("att-pdf")
        .filename("quarterly-report.pdf")
        .media_type("application/pdf");
    let source = ui_ai::AttachmentSourceDocumentData::new("att-source")
        .title("React Documentation")
        .filename("react.dev")
        .url("https://react.dev");
    let audio = ui_ai::AttachmentFileData::new("att-audio")
        .filename("podcast-episode.mp3")
        .media_type("audio/mp3");

    vec![
        ui_ai::AttachmentData::File(image),
        ui_ai::AttachmentData::File(pdf),
        ui_ai::AttachmentData::SourceDocument(source),
        ui_ai::AttachmentData::File(audio),
    ]
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
            let key = Arc::<str>::from(format!("attachments-inline-{}", item_id.as_ref()));
            cx.keyed(key, move |cx| {
                let open = cx.local_model(|| false);
                let hover_card_test_id = (item_id.as_ref() == "att-image")
                    .then_some("ui-ai-attachment-inline-att-image-hover-card");
                let mut attachment = ui_ai::Attachment::new(item.clone())
                    .variant(ui_ai::AttachmentVariant::Inline)
                    .on_remove(on_remove.clone())
                    .hovered_model(open.clone());
                if item_id.as_ref() == "att-image" {
                    attachment = attachment.test_id("ui-ai-attachment-inline-att-image");
                }
                let trigger = attachment.into_element_with_children(cx, move |cx, parts| {
                    let theme = Theme::global(&*cx.app).clone();
                    let preview = ui_ai::AttachmentPreview::from_context().into_element(cx);
                    let info = ui_ai::AttachmentInfo::from_context().into_element(cx);
                    let remove = ui_ai::AttachmentRemove::from_context().into_element(cx);

                    let mut affordance_props = ContainerProps::default();
                    affordance_props.layout = fret_ui_kit::declarative::style::layout_style(
                        &theme,
                        LayoutRefinement::default()
                            .relative()
                            .w_px(MetricRef::Px(Px(20.0)))
                            .h_px(MetricRef::Px(Px(20.0)))
                            .min_w(MetricRef::Px(Px(20.0)))
                            .min_h(MetricRef::Px(Px(20.0)))
                            .flex_shrink_0(),
                    );
                    let hovered = parts.hovered();
                    let remove = cx.interactivity_gate_props(
                        InteractivityGateProps {
                            present: true,
                            interactive: hovered,
                            layout: fret_ui_kit::declarative::style::layout_style(
                                &theme,
                                LayoutRefinement::default()
                                    .absolute()
                                    .top_px(Px(0.0))
                                    .left_px(Px(0.0))
                                    .w_px(MetricRef::Px(Px(20.0)))
                                    .h_px(MetricRef::Px(Px(20.0))),
                            ),
                        },
                        move |_cx| vec![remove],
                    );
                    let affordance = cx.container(affordance_props, move |cx| {
                        vec![
                            cx.opacity(if hovered { 0.0 } else { 1.0 }, move |_cx| vec![preview]),
                            remove,
                        ]
                    });

                    let row = ui::h_row(move |_cx| vec![affordance, info])
                        .layout(LayoutRefinement::default().min_w_0())
                        .gap(Space::N2)
                        .items_center()
                        .into_element(cx);
                    vec![row]
                });

                let preview = ui_ai::AttachmentPreview::new(item.clone())
                    .variant(ui_ai::AttachmentVariant::Grid)
                    .into_element(cx);
                let label = ui::text(ui_ai::get_attachment_label(&item))
                    .text_sm()
                    .into_element(cx);
                let media_type = item
                    .media_type()
                    .cloned()
                    .map(|media_type| ui::text(media_type).text_xs().into_element(cx));

                let hover_content = ui::v_flex(move |_cx| {
                    let mut out = vec![preview, label];
                    if let Some(media_type) = media_type {
                        out.push(media_type);
                    }
                    out
                })
                .layout(LayoutRefinement::default().min_w_0())
                .gap(Space::N2)
                .items_start()
                .into_element(cx);
                let trigger = ui_ai::AttachmentHoverCardTrigger::new(trigger).into_element(cx);
                let mut hover_content = ui_ai::AttachmentHoverCardContent::new(vec![hover_content]);
                if let Some(test_id) = hover_card_test_id {
                    hover_content = hover_content.test_id(test_id);
                }

                ui_ai::AttachmentHoverCard::new(trigger, hover_content)
                    .open_model(open)
                    .into_element(cx)
            })
        })
        .collect::<Vec<_>>();

    ui::h_flex(move |cx| {
        vec![
            ui_ai::Attachments::new(children)
                .variant(ui_ai::AttachmentVariant::Inline)
                .test_id("ui-ai-attachments-inline-root")
                .into_element(cx),
        ]
    })
    .layout(LayoutRefinement::default().w_full())
    .justify_center()
    .into_element(cx)
}
// endregion: example

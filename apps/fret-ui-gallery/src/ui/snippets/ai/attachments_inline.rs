pub const SOURCE: &str = include_str!("attachments_inline.rs");

// region: example
use fret_core::{ImageColorSpace, ImageId, Px};
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui::Theme;
use fret_ui::element::{ContainerProps, InteractivityGateProps};
use fret_ui_ai as ui_ai;
use fret_ui_assets::{ImageSource, ui::ImageSourceElementContextExt as _};
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, MetricRef, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;
use std::sync::OnceLock;

#[derive(Default)]
struct DemoModels {
    removed_ids: Option<Model<Vec<Arc<str>>>>,
}

#[derive(Default)]
struct HoverCardModels {
    open: Option<Model<bool>>,
}

fn landscape_image_id<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Option<ImageId> {
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

fn demo_items<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> Vec<ui_ai::AttachmentData> {
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

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let removed_ids = cx.with_state(DemoModels::default, |st| st.removed_ids.clone());
    let removed_ids = match removed_ids {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Vec::<Arc<str>>::new());
            cx.with_state(DemoModels::default, |st| {
                st.removed_ids = Some(model.clone())
            });
            model
        }
    };

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
                let open = cx.with_state(HoverCardModels::default, |st| st.open.clone());
                let open = match open {
                    Some(model) => model,
                    None => {
                        let model = cx.app.models_mut().insert(false);
                        cx.with_state(HoverCardModels::default, |st| st.open = Some(model.clone()));
                        model
                    }
                };
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

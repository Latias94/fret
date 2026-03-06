pub const SOURCE: &str = include_str!("attachments_inline.rs");

// region: example
use crate::ui::snippets::aspect_ratio::landscape_image_id;
use fret_core::Px;
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui::Theme;
use fret_ui::element::{ContainerProps, InteractivityGateProps};
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, MetricRef, Space};
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default)]
struct DemoModels {
    removed_ids: Option<Model<Vec<Arc<str>>>>,
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
                let mut attachment = ui_ai::Attachment::new(item.clone())
                    .variant(ui_ai::AttachmentVariant::Inline)
                    .on_remove(on_remove.clone());
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
                let hover_content = shadcn::HoverCardContent::new(vec![hover_content])
                    .refine_style(ChromeRefinement::default().p(Space::N2))
                    .into_element(cx);

                shadcn::HoverCard::new(trigger, hover_content)
                    .open_delay_frames(0)
                    .close_delay_frames(0)
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

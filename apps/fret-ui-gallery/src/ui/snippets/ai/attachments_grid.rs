pub const SOURCE: &str = include_str!("attachments_grid.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{ImageId, Px};
use fret_ui::Invalidation;
use fret_ui::Theme;
use fret_ui::element::{ContainerProps, InteractivityGateProps};
use fret_ui_ai as ui_ai;
use fret_ui_assets::ui::ImageSourceElementContextExt as _;
use fret_ui_kit::LayoutRefinement;
use fret_ui_kit::ui;
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

fn landscape_image_id(cx: &mut UiCx<'_>) -> Option<ImageId> {
    let request = crate::driver::demo_assets::ui_gallery_ai_attachment_landscape_request();
    cx.use_image_source_state_from_asset_request(&request).image
}

fn portrait_image_id(cx: &mut UiCx<'_>) -> Option<ImageId> {
    let request = crate::driver::demo_assets::ui_gallery_ai_attachment_portrait_request();
    cx.use_image_source_state_from_asset_request(&request).image
}

fn demo_items(cx: &mut UiCx<'_>) -> Vec<ui_ai::AttachmentData> {
    let mut image_one = ui_ai::AttachmentFileData::new("att-image")
        .filename("mountain-landscape.jpg")
        .media_type("image/jpeg");
    if let Some(preview) = landscape_image_id(cx) {
        image_one = image_one.preview_image(preview);
    }

    let mut image_two = ui_ai::AttachmentFileData::new("att-image-2")
        .filename("ocean-portrait.jpg")
        .media_type("image/jpeg");
    if let Some(preview) = portrait_image_id(cx) {
        image_two = image_two.preview_image(preview);
    }

    let doc = ui_ai::AttachmentFileData::new("att-doc")
        .filename("document.pdf")
        .media_type("application/pdf");
    let video = ui_ai::AttachmentFileData::new("att-video")
        .filename("video.mp4")
        .media_type("video/mp4");

    vec![
        ui_ai::AttachmentData::File(image_one),
        ui_ai::AttachmentData::File(image_two),
        ui_ai::AttachmentData::File(doc),
        ui_ai::AttachmentData::File(video),
    ]
}

fn render_grid_attachment(
    cx: &mut UiCx<'_>,
    data: ui_ai::AttachmentData,
    on_remove: ui_ai::OnAttachmentRemove,
    test_id: Option<&'static str>,
    remove_test_id: Option<&'static str>,
) -> impl UiChild + use<> {
    let mut attachment = ui_ai::Attachment::new(data)
        .variant(ui_ai::AttachmentVariant::Grid)
        .on_remove(on_remove);
    if let Some(test_id) = test_id {
        attachment = attachment.test_id(test_id);
    }
    if let Some(remove_test_id) = remove_test_id {
        attachment = attachment.remove_test_id(remove_test_id);
    }

    attachment.into_element_with_children(cx, move |cx, _parts| {
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
            let key = Arc::<str>::from(format!("attachments-grid-{}", item_id.as_ref()));
            cx.keyed(key, move |cx| {
                render_grid_attachment(
                    cx,
                    item.clone(),
                    on_remove.clone(),
                    (item_id.as_ref() == "att-image").then_some("ui-ai-attachment-grid-att-image"),
                    (item_id.as_ref() == "att-image")
                        .then_some("ui-ai-attachment-grid-att-image-remove"),
                )
                .into_element(cx)
            })
        })
        .collect::<Vec<_>>();

    ui::h_flex(move |cx| {
        vec![
            ui_ai::Attachments::new(children)
                .variant(ui_ai::AttachmentVariant::Grid)
                .test_id("ui-ai-attachments-grid-root")
                .into_element(cx),
        ]
    })
    .layout(LayoutRefinement::default().w_full())
    .justify_center()
    .into_element(cx)
}
// endregion: example

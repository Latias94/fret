pub const SOURCE: &str = include_str!("attachments_grid.rs");

// region: example
use fret_core::{ImageColorSpace, ImageId, Px};
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui::Theme;
use fret_ui::element::{ContainerProps, InteractivityGateProps};
use fret_ui_ai as ui_ai;
use fret_ui_assets::{ImageSource, ui::ImageSourceElementContextExt as _};
use fret_ui_kit::LayoutRefinement;
use fret_ui_kit::ui;
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;
use std::sync::OnceLock;

#[derive(Default)]
struct DemoModels {
    removed_ids: Option<Model<Vec<Arc<str>>>>,
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

fn portrait_image_id<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Option<ImageId> {
    static SOURCE: OnceLock<ImageSource> = OnceLock::new();
    let source = SOURCE.get_or_init(|| {
        ImageSource::rgba8(
            192,
            320,
            demo_preview_rgba8(192, 320, (255, 164, 118)),
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

            let mut r = (20.0 + accent.0 as f32 * (0.35 + 0.65 * fx)) as u8;
            let mut g = (24.0 + accent.1 as f32 * (0.28 + 0.72 * (1.0 - fy))) as u8;
            let mut b = (24.0 + accent.2 as f32 * (0.30 + 0.70 * fy)) as u8;

            let border = x < 3 || y < 3 || x + 3 >= width || y + 3 >= height;
            let horizon = y > height / 2 - 3 && y < height / 2 + 3;
            let badge = x > width / 8 && x < width / 4 && y > height / 10 && y < height / 5;

            if border {
                r = 245;
                g = 245;
                b = 245;
            } else if horizon {
                r = r.saturating_add(18);
                g = g.saturating_add(18);
                b = b.saturating_add(12);
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

fn render_grid_attachment<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    data: ui_ai::AttachmentData,
    on_remove: ui_ai::OnAttachmentRemove,
    test_id: Option<&'static str>,
    remove_test_id: Option<&'static str>,
) -> AnyElement {
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

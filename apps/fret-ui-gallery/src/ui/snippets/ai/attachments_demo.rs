pub const SOURCE: &str = include_str!("attachments_demo.rs");

// region: example
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default)]
struct DemoModels {
    items: Option<Model<Vec<ui_ai::AttachmentData>>>,
}

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let items = cx.with_state(DemoModels::default, |st| st.items.clone());
    let items = match items {
        Some(model) => model,
        None => {
            let image = ui_ai::AttachmentFileData::new("att-image")
                .filename("diagram.png")
                .media_type("image/png")
                .size_bytes(123_456);
            let doc = ui_ai::AttachmentFileData::new("att-doc")
                .filename("notes.txt")
                .media_type("text/plain")
                .size_bytes(4_321);
            let src = ui_ai::AttachmentSourceDocumentData::new("att-src")
                .title("Spec: Overlay Policy")
                .filename("docs/adr/0066.md")
                .url("https://example.com/spec");

            let seed = vec![
                ui_ai::AttachmentData::File(image),
                ui_ai::AttachmentData::File(doc),
                ui_ai::AttachmentData::SourceDocument(src),
            ];
            let model = cx.app.models_mut().insert(seed);
            cx.with_state(DemoModels::default, |st| st.items = Some(model.clone()));
            model
        }
    };

    let on_remove: ui_ai::OnAttachmentRemove = Arc::new({
        let items = items.clone();
        move |host, _action_cx, id| {
            let _ = host.models_mut().update(&items, |v| {
                v.retain(|item| item.id().as_ref() != id.as_ref());
            });
        }
    });

    let now = cx
        .get_model_cloned(&items, Invalidation::Layout)
        .unwrap_or_default();

    let mut grid_children = Vec::new();
    let mut inline_children = Vec::new();
    let mut list_children = Vec::new();

    for item in &now {
        let item_id = item.id().clone();

        let on_remove_grid = on_remove.clone();
        let key = Arc::<str>::from(format!("ai-attachments-demo-grid-{}", item_id.as_ref()));
        let item_grid = item.clone();
        let item_id_grid = item_id.clone();
        grid_children.push(cx.keyed(key, move |cx| {
            let mut el =
                ui_ai::Attachment::new(item_grid.clone()).variant(ui_ai::AttachmentVariant::Grid);
            el = el.on_remove(on_remove_grid.clone());
            if item_id_grid.as_ref() == "att-image" {
                el = el
                    .test_id("ui-ai-attachment-grid-att-image")
                    .remove_test_id("ui-ai-attachment-grid-att-image-remove");
            }
            el.into_element(cx)
        }));

        let on_remove_inline = on_remove.clone();
        let key = Arc::<str>::from(format!("ai-attachments-demo-inline-{}", item_id.as_ref()));
        let item_inline = item.clone();
        let item_id_inline = item_id.clone();
        inline_children.push(cx.keyed(key, move |cx| {
            let mut el = ui_ai::Attachment::new(item_inline.clone())
                .variant(ui_ai::AttachmentVariant::Inline)
                .on_remove(on_remove_inline.clone());
            if item_id_inline.as_ref() == "att-image" {
                el = el.test_id("ui-ai-attachment-inline-att-image");
            }
            let trigger = el.into_element(cx);

            let hover_preview = ui_ai::AttachmentPreview::new(item_inline.clone())
                .variant(ui_ai::AttachmentVariant::Grid)
                .into_element(cx);
            let hover_label = ui::text(cx, ui_ai::get_attachment_label(&item_inline))
                .text_sm()
                .into_element(cx);
            let hover_content = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().min_w_0())
                    .gap(Space::N2)
                    .items_start(),
                move |_cx| vec![hover_preview, hover_label],
            );
            let hover_content = shadcn::HoverCardContent::new(vec![hover_content])
                .refine_style(ChromeRefinement::default().p(Space::N2))
                .into_element(cx);

            shadcn::HoverCard::new(trigger, hover_content)
                .open_delay_frames(0)
                .close_delay_frames(0)
                .into_element(cx)
        }));

        let on_remove_list = on_remove.clone();
        let key = Arc::<str>::from(format!("ai-attachments-demo-list-{}", item_id.as_ref()));
        let item_list = item.clone();
        let item_id_list = item_id.clone();
        list_children.push(cx.keyed(key, move |cx| {
            let mut el = ui_ai::Attachment::new(item_list.clone())
                .variant(ui_ai::AttachmentVariant::List)
                .show_media_type(true)
                .on_remove(on_remove_list.clone());
            if item_id_list.as_ref() == "att-image" {
                el = el.test_id("ui-ai-attachment-list-att-image");
            }
            el.into_element(cx)
        }));
    }

    let grid = ui_ai::Attachments::new(grid_children)
        .variant(ui_ai::AttachmentVariant::Grid)
        .test_id("ui-ai-attachments-grid-root")
        .into_element(cx);

    let inline = ui_ai::Attachments::new(inline_children)
        .variant(ui_ai::AttachmentVariant::Inline)
        .test_id("ui-ai-attachments-inline-root")
        .into_element(cx);

    let list = ui_ai::Attachments::new(list_children)
        .variant(ui_ai::AttachmentVariant::List)
        .test_id("ui-ai-attachments-list-root")
        .into_element(cx);

    let empty = ui_ai::AttachmentEmpty::new(Vec::new())
        .test_id("ui-ai-attachments-empty-root")
        .into_element(cx);

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |cx| {
            vec![
                cx.text("Attachments (AI Elements)"),
                cx.text("Grid, inline, and list variants. Hover to reveal remove controls; remove mutates the attachment list."),
                cx.text("Grid"),
                grid,
                cx.text("Inline (with hover preview)"),
                inline,
                cx.text("List"),
                list,
                cx.text("Empty state"),
                empty,
            ]
        },
    )
}
// endregion: example

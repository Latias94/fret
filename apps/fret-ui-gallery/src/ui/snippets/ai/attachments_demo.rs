pub const SOURCE: &str = include_str!("attachments_demo.rs");

// region: example
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
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

    let mut children = Vec::new();
    for item in now {
        let item_id = item.id().clone();
        let key = Arc::<str>::from(format!("ai-attachments-demo-{}", item_id.as_ref()));
        let on_remove = on_remove.clone();
        children.push(cx.keyed(key, move |cx| {
            let mut el =
                ui_ai::Attachment::new(item.clone()).variant(ui_ai::AttachmentVariant::Grid);
            el = el.on_remove(on_remove.clone());
            if item_id.as_ref() == "att-image" {
                el = el
                    .test_id("ui-ai-attachment-grid-att-image")
                    .remove_test_id("ui-ai-attachment-grid-att-image-remove");
            }
            el.into_element(cx)
        }));
    }

    let grid = ui_ai::Attachments::new(children)
        .variant(ui_ai::AttachmentVariant::Grid)
        .test_id("ui-ai-attachments-grid-root")
        .into_element(cx);

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |cx| {
            vec![
                cx.text("Attachments (AI Elements)"),
                cx.text("Hover to reveal remove controls; remove mutates the attachment list."),
                grid,
            ]
        },
    )
}
// endregion: example

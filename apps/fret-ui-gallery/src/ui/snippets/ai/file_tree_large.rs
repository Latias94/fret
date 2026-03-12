pub const SOURCE: &str = include_str!("file_tree_large.rs");

// region: example
use fret_core::{Px, TextAlign, TextOverflow, TextWrap};
use fret_ui::action::ActionCx;
use fret_ui::element::{AnyElement, Length, SemanticsDecoration, SizeStyle, TextProps};
use fret_ui::{ElementContext, UiHost};
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::ModelWatchExt;
use fret_ui_kit::{LayoutRefinement, Space, ui};
use std::sync::Arc;

fn invisible_marker<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    test_id: &'static str,
) -> AnyElement {
    cx.text_props(TextProps {
        layout: fret_ui::element::LayoutStyle {
            size: SizeStyle {
                width: Length::Px(Px(0.0)),
                height: Length::Px(Px(0.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        text: Arc::<str>::from(""),
        style: None,
        color: None,
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: TextAlign::Start,
        ink_overflow: Default::default(),
    })
    .attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id(Arc::<str>::from(test_id)),
    )
}

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let selected = cx.local_model_keyed("selected", || None::<Arc<str>>);

    let selected_value = cx.watch_model(&selected).layout().cloned().flatten();

    let big_files = (0..1000u32).map(|i| {
        let path: Arc<str> = Arc::from(format!("big/{i:04}.txt"));
        let name: Arc<str> = Arc::from(format!("{i:04}.txt"));
        let mut file = ui_ai::FileTreeFile::new(path, name);
        if i == 0 {
            file = file.test_id("ui-ai-file-tree-large-file-0000");
        }
        if i == 500 {
            file = file.test_id("ui-ai-file-tree-large-file-500");
        }
        file.into()
    });

    let tree = ui_ai::FileTree::new([ui_ai::FileTreeFolder::new("big", "big")
        .test_id("ui-ai-file-tree-large-folder-big")
        .children(big_files)
        .into()])
    .selected_path(selected_value.clone())
    .on_select(Arc::new({
        let selected = selected.clone();
        move |host, _action_cx: ActionCx, path| {
            let _ = host.models_mut().update(&selected, |v| *v = Some(path));
        }
    }))
    .test_id_root("ui-ai-file-tree-large-root")
    .refine_layout(
        LayoutRefinement::default()
            .w_full()
            .min_w_0()
            .max_h(Px(320.0)),
    )
    .into_element(cx);

    let mut out = vec![tree];
    if selected_value.as_deref() == Some("big/0500.txt") {
        out.push(invisible_marker(
            cx,
            "ui-ai-file-tree-large-selected-marker",
        ));
    }

    ui::v_flex(move |_cx| out)
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .gap(Space::N0)
        .into_element(cx)
}

pub fn preview<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    render(cx)
}
// endregion: example

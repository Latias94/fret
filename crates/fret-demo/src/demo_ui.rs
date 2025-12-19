use crate::command_palette::{CommandPalette, OverlayBackdrop, OverlayPanelLayout};
use crate::dnd_probe::DndProbe;
use crate::elements_mvp2::ElementsMvp2Demo;
use crate::ime_probe::ImeProbe;
use crate::property_row::PropertyRow;
use fret_core::{AppWindowId, Axis, Color, Px};
use fret_ui::{
    ColoredPanel, Column, DockSpace, FixedPanel, Scroll, Split, Text, TextArea, TextInput,
    TreeNode, TreeView, UiLayerId, UiTree, VirtualList, VirtualListDataSource, VirtualListRow,
};
use std::borrow::Cow;

#[derive(Debug, Clone)]
struct LazyEntityList {
    count: usize,
}

impl VirtualListDataSource for LazyEntityList {
    type Key = u64;

    fn len(&self) -> usize {
        self.count
    }

    fn key_at(&self, index: usize) -> Self::Key {
        index as u64
    }

    fn row_at(&self, index: usize) -> VirtualListRow<'_> {
        VirtualListRow::new(Cow::Owned(format!("Entity {index:06}")))
    }

    fn index_of_key(&self, key: Self::Key) -> Option<usize> {
        let index = key as usize;
        if index < self.count {
            Some(index)
        } else {
            None
        }
    }
}

pub struct DemoUiConfig {
    pub split_fraction: f32,
    pub property_count: usize,
}

impl Default for DemoUiConfig {
    fn default() -> Self {
        Self {
            split_fraction: 0.72,
            property_count: 28,
        }
    }
}

pub struct DemoLayers {
    pub modal: UiLayerId,
    pub external_dnd: UiLayerId,
    pub command_palette: UiLayerId,
    pub command_palette_node: fret_core::NodeId,
}

pub fn build_demo_ui(window: AppWindowId, config: DemoUiConfig) -> (UiTree, DemoLayers) {
    let mut ui = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(Split::new(Axis::Horizontal, config.split_fraction));
    ui.set_root(root);

    let dock = ui.create_node(DockSpace::new(window));
    ui.add_child(root, dock);

    let scroll = ui.create_node(Scroll::new());
    ui.add_child(root, scroll);

    let column = ui.create_node(Column::new().with_padding(Px(10.0)).with_spacing(Px(8.0)));
    ui.add_child(scroll, column);

    let dnd_probe = ui.create_node(DndProbe::new());
    ui.add_child(column, dnd_probe);

    let text_header = ui.create_node(Text::new("Text MVP (labels + single-line TextInput)"));
    ui.add_child(column, text_header);

    let text_input =
        ui.create_node(TextInput::new().with_text("Click here, then type (IME supported)"));
    ui.add_child(column, text_input);

    let text_input2 =
        ui.create_node(TextInput::new().with_text("Another TextInput (Tab to switch focus)"));
    ui.add_child(column, text_input2);

    let ime_probe = ui.create_node(ImeProbe::new());
    ui.add_child(column, ime_probe);

    let multiline_header = ui.create_node(Text::new(
        "Multiline MVP (wrap + hit test + caret rect + selection rects)",
    ));
    ui.add_child(column, multiline_header);

    let multiline = ui.create_node(
        TextArea::new(
            "Multiline text: click/drag to place caret and select.\n\
This is wrapped text (TextWrap::Word) and exercises:\n\
- TextService::hit_test_point\n\
- TextService::caret_rect\n\
- TextService::selection_rects\n\
\n\
Goal: foundation for Console/Inspector/code editor.",
        )
        .with_min_height(Px(220.0)),
    );
    ui.add_child(column, multiline);

    let list_header = ui.create_node(Text::new(
        "VirtualList MVP (Hierarchy/Project-scale list: scroll + selection + virtualization)",
    ));
    ui.add_child(column, list_header);

    let list_panel = ui.create_node(FixedPanel::new(
        Px(260.0),
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        },
    ));
    ui.add_child(column, list_panel);

    let list = ui.create_node(VirtualList::new(LazyEntityList { count: 100_000 }));
    ui.add_child(list_panel, list);

    let tree_header = ui.create_node(Text::new(
        "TreeView MVP (Hierarchy-style tree: expand/collapse + selection + virtualization)",
    ));
    ui.add_child(column, tree_header);

    let tree_panel = ui.create_node(FixedPanel::new(
        Px(260.0),
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        },
    ));
    ui.add_child(column, tree_panel);

    let mut next_id: u64 = 1;
    let mut expand: Vec<u64> = Vec::new();
    let mut roots: Vec<TreeNode> = Vec::new();
    for r in 0..200u64 {
        let root_id = next_id;
        next_id += 1;
        if r < 3 {
            expand.push(root_id);
        }

        let mut children: Vec<TreeNode> = Vec::new();
        for c in 0..20u64 {
            let child_id = next_id;
            next_id += 1;

            let mut grandchildren: Vec<TreeNode> = Vec::new();
            if c < 3 {
                for g in 0..5u64 {
                    let grand_id = next_id;
                    next_id += 1;
                    grandchildren.push(TreeNode::new(
                        grand_id,
                        format!("Grandchild {r:03}-{c:02}-{g:02}"),
                    ));
                }
            }

            children.push(
                TreeNode::new(child_id, format!("Child {r:03}-{c:02}"))
                    .with_children(grandchildren),
            );
        }
        roots.push(TreeNode::new(root_id, format!("Root {r:03}")).with_children(children));
    }

    let tree = ui.create_node(TreeView::new(roots).with_expanded(expand));
    ui.add_child(tree_panel, tree);

    let elements_demo = ui.create_node(ElementsMvp2Demo::new());
    ui.add_child(column, elements_demo);

    populate_property_panel(&mut ui, column, config.property_count);

    let modal_root = ui.create_node(ColoredPanel::new(Color {
        r: 0.02,
        g: 0.02,
        b: 0.02,
        a: 0.45,
    }));
    let modal = ui.push_overlay_root(modal_root, true);
    ui.set_layer_visible(modal, false);

    let dnd_root = ui.create_node(ColoredPanel::new(Color {
        r: 0.08,
        g: 0.20,
        b: 0.10,
        a: 0.22,
    }));
    let external_dnd = ui.push_overlay_root_ex(dnd_root, false, false);
    ui.set_layer_visible(external_dnd, false);

    let palette_root =
        ui.create_node(OverlayPanelLayout::new(Px(640.0), Px(360.0)).with_top(Px(64.0)));
    let command_palette = ui.push_overlay_root(palette_root, true);
    ui.set_layer_visible(command_palette, false);

    let backdrop = ui.create_node(OverlayBackdrop::new(
        Color {
            r: 0.02,
            g: 0.02,
            b: 0.02,
            a: 0.55,
        },
        fret_app::CommandId::from("command_palette.close"),
    ));
    ui.add_child(palette_root, backdrop);

    let command_palette_node = ui.create_node(CommandPalette::new());
    ui.add_child(palette_root, command_palette_node);

    (
        ui,
        DemoLayers {
            modal,
            external_dnd,
            command_palette,
            command_palette_node,
        },
    )
}

fn populate_property_panel(ui: &mut UiTree, parent: fret_core::NodeId, count: usize) {
    for i in 0..count {
        let shade = 0.14 + (i % 2) as f32 * 0.02;
        let height = if i % 7 == 0 { Px(72.0) } else { Px(44.0) };
        let item = ui.create_node(PropertyRow::new(
            format!("Property {i}"),
            height,
            Color {
                r: shade,
                g: shade + 0.01,
                b: shade + 0.02,
                a: 1.0,
            },
        ));
        ui.add_child(parent, item);
    }
}

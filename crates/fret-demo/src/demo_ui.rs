use crate::command_palette::{CommandPalette, OverlayBackdrop, OverlayPanelLayout};
use crate::dnd_probe::DndProbe;
use crate::editor_shell::{DemoSelection, HierarchyPanel, InspectorPanel};
use crate::elements_mvp2::ElementsMvp2Demo;
use crate::hierarchy::DemoHierarchy;
use crate::ime_probe::ImeProbe;
use fret_app::Model;
use fret_core::{AppWindowId, Axis, Color, PanelKey, Px};
use fret_ui::{
    BoundTextInput, ColoredPanel, Column, ContextMenu, DockSpace, FixedPanel, Scroll, Split, Stack,
    Text, TextArea, TextInput, UiLayerId, UiTree, VirtualList, VirtualListDataSource,
    VirtualListRow,
};
use std::borrow::Cow;

use crate::world::DemoWorld;

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
}

impl Default for DemoUiConfig {
    fn default() -> Self {
        Self {
            split_fraction: 0.72,
        }
    }
}

pub struct DemoLayers {
    pub modal: UiLayerId,
    pub external_dnd: UiLayerId,
    pub command_palette: UiLayerId,
    pub command_palette_node: fret_core::NodeId,
    pub context_menu: UiLayerId,
    pub context_menu_node: fret_core::NodeId,
    pub inspector_edit: UiLayerId,
    pub inspector_edit_input_node: fret_core::NodeId,
    pub dockspace_node: fret_core::NodeId,
}

pub fn build_demo_ui(
    window: AppWindowId,
    config: DemoUiConfig,
    selection: Model<DemoSelection>,
    hierarchy: Model<DemoHierarchy>,
    world: Model<DemoWorld>,
    inspector_edit_buffer: Model<String>,
) -> (UiTree, DemoLayers) {
    let mut ui = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(Split::new(Axis::Horizontal, config.split_fraction));
    ui.set_root(root);

    let key_hierarchy = PanelKey::new("core.hierarchy");
    let key_inspector = PanelKey::new("core.inspector");

    let hierarchy = ui.create_node(HierarchyPanel::new(selection, hierarchy));
    let inspector = ui.create_node(InspectorPanel::new(selection, world));

    let dock = ui.create_node(
        DockSpace::new(window)
            .with_panel_content(key_hierarchy, hierarchy)
            .with_panel_content(key_inspector, inspector),
    );
    ui.add_child(root, dock);
    ui.add_child(dock, hierarchy);
    ui.add_child(dock, inspector);

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

    let editor_header = ui.create_node(Text::new(
        "Editor Shell MVP (Hierarchy → Inspector) is mounted into DockSpace panels",
    ));
    ui.add_child(column, editor_header);

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

    let elements_demo = ui.create_node(ElementsMvp2Demo::new());
    ui.add_child(column, elements_demo);

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

    let context_menu_node = ui.create_node(ContextMenu::new());
    let context_menu = ui.push_overlay_root(context_menu_node, true);
    ui.set_layer_visible(context_menu, false);

    let inspector_root =
        ui.create_node(OverlayPanelLayout::new(Px(520.0), Px(160.0)).with_top(Px(96.0)));
    let inspector_edit = ui.push_overlay_root(inspector_root, true);
    ui.set_layer_visible(inspector_edit, false);

    let inspector_backdrop = ui.create_node(OverlayBackdrop::new(
        Color {
            r: 0.02,
            g: 0.02,
            b: 0.02,
            a: 0.55,
        },
        fret_app::CommandId::from("inspector_edit.close"),
    ));
    ui.add_child(inspector_root, inspector_backdrop);

    let inspector_panel = ui.create_node(Stack::new());
    ui.add_child(inspector_root, inspector_panel);

    let inspector_panel_bg = ui.create_node(ColoredPanel::new(Color {
        r: 0.10,
        g: 0.10,
        b: 0.12,
        a: 1.0,
    }));
    ui.add_child(inspector_panel, inspector_panel_bg);

    let inspector_column =
        ui.create_node(Column::new().with_padding(Px(12.0)).with_spacing(Px(8.0)));
    ui.add_child(inspector_panel, inspector_column);

    let inspector_label = ui.create_node(Text::new("Edit value (Enter=commit, Esc=cancel)"));
    ui.add_child(inspector_column, inspector_label);

    let inspector_edit_input_node = ui.create_node(
        BoundTextInput::new(inspector_edit_buffer)
            .with_submit_command(fret_app::CommandId::from("inspector_edit.commit"))
            .with_cancel_command(fret_app::CommandId::from("inspector_edit.close")),
    );
    ui.add_child(inspector_column, inspector_edit_input_node);

    (
        ui,
        DemoLayers {
            modal,
            external_dnd,
            command_palette,
            command_palette_node,
            context_menu,
            context_menu_node,
            inspector_edit,
            inspector_edit_input_node,
            dockspace_node: dock,
        },
    )
}

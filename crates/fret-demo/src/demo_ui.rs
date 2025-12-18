use fret_core::{AppWindowId, Axis, Color, Px};
use fret_ui::{Column, DockSpace, FixedPanel, Scroll, Split, UiTree};

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

pub fn build_demo_ui(window: AppWindowId, config: DemoUiConfig) -> (UiTree, fret_core::NodeId) {
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

    populate_property_panel(&mut ui, column, config.property_count);

    (ui, root)
}

fn populate_property_panel(ui: &mut UiTree, parent: fret_core::NodeId, count: usize) {
    for i in 0..count {
        let shade = 0.14 + (i % 2) as f32 * 0.02;
        let height = if i % 7 == 0 { Px(72.0) } else { Px(44.0) };
        let item = ui.create_node(FixedPanel::new(
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


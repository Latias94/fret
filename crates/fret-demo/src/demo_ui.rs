use crate::DemoPlayState;
use crate::asset_drop::CurrentSceneService;
use crate::command_palette::{CommandPalette, OverlayBackdrop, OverlayPanelLayout};
use crate::dnd_probe::DndProbe;
use crate::elements_mvp2::ElementsMvp2Demo;
use crate::ime_probe::ImeProbe;
use crate::overlay_layouts::CenteredOverlayLayout;
use crate::scene_document::SceneDocumentService;
use fret_app::Model;
use fret_core::{AppWindowId, Axis, Color, Px};
use fret_editor::{InspectorEditHint, InspectorEditLayout};
use fret_editor::{ViewportToolManager, ViewportToolMode};
use fret_ui::{
    AppMenuBar, Bar, BoundTextInput, ColoredPanel, Column, ContextMenu, DockSpace, FixedPanel,
    HeaderBody, PanelThemeBackground, Row, Scroll, Split, Stack, Text, TextArea, TextInput,
    Toolbar, ToolbarItem, UiLayerId, UiTree, VirtualList, VirtualListDataSource, VirtualListRow,
};
use std::borrow::Cow;
use std::sync::Arc;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoUiKind {
    Main,
    DockFloating,
}

struct DemoToolbar {
    tools: Model<ViewportToolManager>,
    toolbar: Toolbar,
    last_mode: Option<ViewportToolMode>,
    last_playing: Option<bool>,
}

impl DemoToolbar {
    pub fn new(tools: Model<ViewportToolManager>) -> Self {
        Self {
            tools,
            toolbar: Toolbar::new(Vec::new()),
            last_mode: None,
            last_playing: None,
        }
    }

    fn rebuild_items(&mut self, app: &mut fret_app::App) -> bool {
        let mode = self.tools.get(app).map(|t| t.active).unwrap_or_default();
        let playing = app.global::<DemoPlayState>().is_some_and(|s| s.playing);
        if self.last_mode == Some(mode) && self.last_playing == Some(playing) {
            return false;
        }
        self.last_mode = Some(mode);
        self.last_playing = Some(playing);

        let play_label: Arc<str> = if playing { "Stop" } else { "Play" }.into();
        let items = vec![
            ToolbarItem::new("New", "scene.new"),
            ToolbarItem::new("Save", "scene.save"),
            ToolbarItem::new("Save As", "scene.save_as"),
            ToolbarItem::new("Select (Q)", "viewport.tool.select")
                .with_selected(mode == ViewportToolMode::Select),
            ToolbarItem::new("Move (W)", "viewport.tool.move")
                .with_selected(mode == ViewportToolMode::Move),
            ToolbarItem::new("Rotate (E)", "viewport.tool.rotate")
                .with_selected(mode == ViewportToolMode::Rotate),
            ToolbarItem::new(play_label, "demo.play.toggle").with_selected(playing),
            ToolbarItem::new("Save Layout", "dock.layout.preset.save_last"),
            ToolbarItem::new("Load Layout", "dock.layout.preset.load_last"),
            ToolbarItem::new("Reset Layout", "dock.layout.reset_default"),
        ];
        self.toolbar.set_items(items);
        true
    }
}

impl fret_ui::Widget for DemoToolbar {
    fn event(&mut self, cx: &mut fret_ui::EventCx<'_>, event: &fret_core::Event) {
        if self.rebuild_items(cx.app) {
            cx.invalidate_self(fret_ui::Invalidation::Layout);
            cx.invalidate_self(fret_ui::Invalidation::Paint);
            cx.request_redraw();
        }
        self.toolbar.event(cx, event);
    }

    fn layout(&mut self, cx: &mut fret_ui::LayoutCx<'_>) -> fret_core::Size {
        let _ = self.rebuild_items(cx.app);
        self.toolbar.layout(cx)
    }

    fn paint(&mut self, cx: &mut fret_ui::PaintCx<'_>) {
        self.toolbar.paint(cx);
    }
}

struct DemoTopBarStatus {
    text: String,
    text_blob: Option<fret_core::TextBlobId>,
    text_metrics: Option<fret_core::TextMetrics>,
    last_key: Option<(u64, u64, bool)>,
    last_scale_factor: Option<f32>,
    style: fret_core::TextStyle,
}

impl DemoTopBarStatus {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            text_blob: None,
            text_metrics: None,
            last_key: None,
            last_scale_factor: None,
            style: fret_core::TextStyle {
                font: fret_core::FontId::default(),
                size: Px(13.0),
            },
        }
    }

    fn compute_text(app: &fret_app::App) -> String {
        let playing = app.global::<DemoPlayState>().is_some_and(|s| s.playing);
        let play = if playing { "Play" } else { "Edit" };

        let dirty = app
            .global::<SceneDocumentService>()
            .is_some_and(|s| s.dirty());

        let title = app
            .global::<CurrentSceneService>()
            .and_then(|s| s.guid())
            .and_then(|guid| {
                let project = app.global::<fret_editor::ProjectService>()?;
                let id = project.id_for_guid(guid)?;
                let path = project.path_for_id(id)?;
                path.file_stem().map(|s| s.to_string_lossy().to_string())
            })
            .unwrap_or_else(|| "No Scene".to_string());

        let suffix = if dirty { "*" } else { "" };
        format!("{play} | {title}{suffix}")
    }

    fn sync_text(&mut self, cx: &mut fret_ui::LayoutCx<'_>) -> bool {
        let playing = cx.app.global::<DemoPlayState>().is_some_and(|s| s.playing);
        let scene_rev = cx
            .app
            .global::<CurrentSceneService>()
            .map(|s| s.revision())
            .unwrap_or(0);
        let doc_rev = cx
            .app
            .global::<SceneDocumentService>()
            .map(|s| s.revision())
            .unwrap_or(0);
        let key = (scene_rev, doc_rev, playing);

        if self.last_key == Some(key) && self.last_scale_factor == Some(cx.scale_factor) {
            return false;
        }
        self.last_key = Some(key);
        self.last_scale_factor = Some(cx.scale_factor);

        if let Some(blob) = self.text_blob.take() {
            cx.text.release(blob);
        }
        self.text = Self::compute_text(cx.app);

        let constraints = fret_core::TextConstraints {
            max_width: Some(cx.available.width),
            wrap: fret_core::TextWrap::None,
            scale_factor: cx.scale_factor,
        };
        let (blob, metrics) = cx.text.prepare(&self.text, self.style, constraints);
        self.text_blob = Some(blob);
        self.text_metrics = Some(metrics);
        true
    }
}

impl fret_ui::Widget for DemoTopBarStatus {
    fn event(&mut self, _cx: &mut fret_ui::EventCx<'_>, _event: &fret_core::Event) {}

    fn layout(&mut self, cx: &mut fret_ui::LayoutCx<'_>) -> fret_core::Size {
        let _ = self.sync_text(cx);
        self.text_metrics.map(|m| m.size).unwrap_or_default()
    }

    fn paint(&mut self, cx: &mut fret_ui::PaintCx<'_>) {
        let theme = cx.theme().snapshot();
        let Some(blob) = self.text_blob else {
            return;
        };
        let Some(metrics) = self.text_metrics else {
            return;
        };

        let pad = theme.metrics.padding_md.0.max(0.0);
        let content_w = metrics.size.width.0;
        let x = (cx.bounds.origin.x.0 + cx.bounds.size.width.0 - pad - content_w)
            .max(cx.bounds.origin.x.0 + pad);

        let inner_y =
            cx.bounds.origin.y.0 + ((cx.bounds.size.height.0 - metrics.size.height.0) * 0.5);
        let y = inner_y + metrics.baseline.0;

        cx.scene.push(fret_core::SceneOp::Text {
            order: fret_core::DrawOrder(0),
            origin: fret_core::geometry::Point::new(Px(x), Px(y)),
            text: blob,
            color: theme.colors.text_muted,
        });
    }
}

pub fn build_demo_ui(
    window: AppWindowId,
    kind: DemoUiKind,
    config: DemoUiConfig,
    inspector_edit_buffer: Model<String>,
    viewport_tools: Model<ViewportToolManager>,
) -> (UiTree, DemoLayers) {
    let mut ui = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(Stack::new());
    ui.set_root(root);

    let bg = ui.create_node(ColoredPanel::themed(PanelThemeBackground::Surface, 1.0));
    ui.add_child(root, bg);

    let dock = match kind {
        DemoUiKind::Main => {
            let frame = ui.create_node(HeaderBody::auto());
            ui.add_child(root, frame);

            let header = ui.create_node(Column::new());
            ui.add_child(frame, header);

            let menu_bar = fret_app::MenuBar {
                menus: vec![
                    fret_app::Menu {
                        title: Arc::<str>::from("File"),
                        items: vec![
                            fret_app::MenuItem::Command {
                                command: fret_app::CommandId::from("scene.new"),
                                when: None,
                            },
                            fret_app::MenuItem::Command {
                                command: fret_app::CommandId::from("scene.save"),
                                when: None,
                            },
                            fret_app::MenuItem::Command {
                                command: fret_app::CommandId::from("scene.save_as"),
                                when: None,
                            },
                            fret_app::MenuItem::Separator,
                            fret_app::MenuItem::Submenu {
                                title: Arc::<str>::from("Layout"),
                                when: None,
                                items: vec![
                                    fret_app::MenuItem::Command {
                                        command: fret_app::CommandId::from(
                                            "dock.layout.reset_default",
                                        ),
                                        when: None,
                                    },
                                    fret_app::MenuItem::Separator,
                                    fret_app::MenuItem::Command {
                                        command: fret_app::CommandId::from(
                                            "dock.layout.preset.save_last",
                                        ),
                                        when: None,
                                    },
                                    fret_app::MenuItem::Command {
                                        command: fret_app::CommandId::from(
                                            "dock.layout.preset.load_last",
                                        ),
                                        when: None,
                                    },
                                ],
                            },
                            fret_app::MenuItem::Separator,
                            fret_app::MenuItem::Command {
                                command: fret_app::CommandId::from("command_palette.toggle"),
                                when: None,
                            },
                            fret_app::MenuItem::Separator,
                            fret_app::MenuItem::Command {
                                command: fret_app::CommandId::from("project.refresh"),
                                when: None,
                            },
                        ],
                    },
                    fret_app::Menu {
                        title: Arc::<str>::from("Edit"),
                        items: vec![
                            fret_app::MenuItem::Command {
                                command: fret_app::CommandId::from("edit.undo"),
                                when: None,
                            },
                            fret_app::MenuItem::Command {
                                command: fret_app::CommandId::from("edit.redo"),
                                when: None,
                            },
                        ],
                    },
                    fret_app::Menu {
                        title: Arc::<str>::from("View"),
                        items: vec![
                            fret_app::MenuItem::Command {
                                command: fret_app::CommandId::from("viewport.tool.select"),
                                when: None,
                            },
                            fret_app::MenuItem::Command {
                                command: fret_app::CommandId::from("viewport.tool.move"),
                                when: None,
                            },
                            fret_app::MenuItem::Command {
                                command: fret_app::CommandId::from("viewport.tool.rotate"),
                                when: None,
                            },
                            fret_app::MenuItem::Separator,
                            fret_app::MenuItem::Submenu {
                                title: Arc::<str>::from("Theme"),
                                when: None,
                                items: vec![
                                    fret_app::MenuItem::Command {
                                        command: fret_app::CommandId::from(
                                            "theme.set.fret_default_dark",
                                        ),
                                        when: None,
                                    },
                                    fret_app::MenuItem::Command {
                                        command: fret_app::CommandId::from(
                                            "theme.set.hardhacker_dark",
                                        ),
                                        when: None,
                                    },
                                    fret_app::MenuItem::Command {
                                        command: fret_app::CommandId::from(
                                            "theme.set.godot_default_dark",
                                        ),
                                        when: None,
                                    },
                                    fret_app::MenuItem::Separator,
                                    fret_app::MenuItem::Command {
                                        command: fret_app::CommandId::from("theme.reload"),
                                        when: None,
                                    },
                                    fret_app::MenuItem::Command {
                                        command: fret_app::CommandId::from("theme.reset_override"),
                                        when: None,
                                    },
                                ],
                            },
                        ],
                    },
                ],
            };

            let menu_bar_node = ui.create_node(AppMenuBar::new(menu_bar));
            ui.add_child(header, menu_bar_node);

            let top_bar = ui.create_node(Bar::new(PanelThemeBackground::Panel, 1.0));
            ui.add_child(header, top_bar);

            let top_bar_row = ui.create_node(Row::new());
            ui.add_child(top_bar, top_bar_row);

            let toolbar = ui.create_node(DemoToolbar::new(viewport_tools));
            ui.add_child(top_bar_row, toolbar);

            let status = ui.create_node(DemoTopBarStatus::new());
            ui.add_child(top_bar_row, status);

            let split = ui.create_node(Split::new(Axis::Horizontal, config.split_fraction));
            ui.add_child(frame, split);

            let dock = ui.create_node(DockSpace::new(window));
            ui.add_child(split, dock);

            let scroll = ui.create_node(Scroll::new());
            ui.add_child(split, scroll);

            let column = ui.create_node(Column::new().with_padding(Px(10.0)).with_spacing(Px(8.0)));
            ui.add_child(scroll, column);

            let dnd_probe = ui.create_node(DndProbe::new());
            ui.add_child(column, dnd_probe);

            let text_header =
                ui.create_node(Text::new("Text MVP (labels + single-line TextInput)"));
            ui.add_child(column, text_header);

            let text_input =
                ui.create_node(TextInput::new().with_text("Click here, then type (IME supported)"));
            ui.add_child(column, text_input);

            let text_input2 = ui
                .create_node(TextInput::new().with_text("Another TextInput (Tab to switch focus)"));
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

            dock
        }
        DemoUiKind::DockFloating => {
            let dock = ui.create_node(DockSpace::new(window));
            ui.add_child(root, dock);
            dock
        }
    };

    let modal_root = ui.create_node(CenteredOverlayLayout::new(Px(520.0), Px(170.0)));
    let modal = ui.push_overlay_root(modal_root, true);
    ui.set_layer_visible(modal, false);

    let modal_backdrop = ui.create_node(OverlayBackdrop::new(
        PanelThemeBackground::Surface,
        0.55,
        fret_app::CommandId::from("unsaved_dialog.cancel"),
    ));
    ui.add_child(modal_root, modal_backdrop);

    let modal_panel = ui.create_node(Stack::new());
    ui.add_child(modal_root, modal_panel);

    let modal_bg = ui.create_node(ColoredPanel::themed(PanelThemeBackground::Panel, 1.0));
    ui.add_child(modal_panel, modal_bg);

    let modal_col = ui.create_node(Column::new().with_padding(Px(14.0)).with_spacing(Px(10.0)));
    ui.add_child(modal_panel, modal_col);

    let modal_title = ui.create_node(Text::new("Unsaved changes"));
    ui.add_child(modal_col, modal_title);

    let modal_msg = ui.create_node(Text::new(
        "The current scene has unsaved changes.\nDo you want to save before continuing?",
    ));
    ui.add_child(modal_col, modal_msg);

    let modal_actions = ui.create_node(Toolbar::new(vec![
        ToolbarItem::new("Save", "unsaved_dialog.save"),
        ToolbarItem::new("Don't Save", "unsaved_dialog.discard"),
        ToolbarItem::new("Cancel", "unsaved_dialog.cancel"),
    ]));
    ui.add_child(modal_col, modal_actions);

    let dnd_root = ui.create_node(ColoredPanel::themed(PanelThemeBackground::Panel, 0.22));
    let external_dnd = ui.push_overlay_root_ex(dnd_root, false, false);
    ui.set_layer_visible(external_dnd, false);

    let palette_root =
        ui.create_node(OverlayPanelLayout::new(Px(640.0), Px(360.0)).with_top(Px(64.0)));
    let command_palette = ui.push_overlay_root(palette_root, true);
    ui.set_layer_visible(command_palette, false);

    let backdrop = ui.create_node(OverlayBackdrop::new(
        PanelThemeBackground::Surface,
        0.55,
        fret_app::CommandId::from("command_palette.close"),
    ));
    ui.add_child(palette_root, backdrop);

    let command_palette_node = ui.create_node(CommandPalette::new());
    ui.add_child(palette_root, command_palette_node);

    let context_menu_node = ui.create_node(ContextMenu::new());
    let context_menu = ui.push_overlay_root(context_menu_node, true);
    ui.set_layer_visible(context_menu, false);

    let inspector_root = ui.create_node(InspectorEditLayout::new(Px(420.0), Px(110.0)));
    let inspector_edit = ui.push_overlay_root(inspector_root, true);
    ui.set_layer_visible(inspector_edit, false);

    let inspector_backdrop = ui.create_node(OverlayBackdrop::new(
        PanelThemeBackground::Surface,
        0.55,
        fret_app::CommandId::from("inspector_edit.commit"),
    ));
    ui.add_child(inspector_root, inspector_backdrop);

    let inspector_panel = ui.create_node(Stack::new());
    ui.add_child(inspector_root, inspector_panel);

    let inspector_panel_bg = ui.create_node(ColoredPanel::themed(PanelThemeBackground::Panel, 1.0));
    ui.add_child(inspector_panel, inspector_panel_bg);

    let inspector_column =
        ui.create_node(Column::new().with_padding(Px(12.0)).with_spacing(Px(8.0)));
    ui.add_child(inspector_panel, inspector_column);

    let inspector_hint = ui.create_node(InspectorEditHint::new(window));
    ui.add_child(inspector_column, inspector_hint);

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

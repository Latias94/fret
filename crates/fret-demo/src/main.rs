mod command_palette;
mod demo_ui;
mod dnd_probe;
mod editor_shell;
mod elements_mvp2;
mod ime_probe;

use demo_ui::{DemoLayers, DemoUiConfig, build_demo_ui};
use editor_shell::DemoSelection;

use fret_app::{
    App, CommandId, CommandMeta, CommandScope, CreateWindowKind, CreateWindowRequest, Effect,
    Keymap, KeymapFileV1, KeymapService, Model, WindowRequest,
    keymap::{BindingV1, KeySpecV1},
};
use fret_core::{
    Axis, Color, DockLayoutNodeV1, DockLayoutV1, DockNode, DockOp, PanelKey, Rect, RenderTargetId,
    Scene,
};
use fret_render::{RenderTargetColorSpace, RenderTargetDescriptor, Renderer, WgpuContext};
use fret_runner_winit_wgpu::{WindowCreateSpec, WinitDriver, WinitRunner, WinitRunnerConfig};
use fret_ui::{ContextMenuService, DockManager, DockPanel, UiTree, ViewportPanel};
use std::{collections::HashMap, fs::File, path::Path};
use winit::event_loop::EventLoop;

struct DemoWindowState {
    ui: UiTree,
    layers: DemoLayers,
    palette_previous_focus: Option<fret_core::NodeId>,
    context_menu_previous_focus: Option<fret_core::NodeId>,
}

#[derive(Default)]
struct DemoDriver {
    main_window: Option<fret_core::AppWindowId>,
    scene_target: Option<RenderTargetId>,
    scene_target_size: Option<(u32, u32)>,
    scene_texture: Option<wgpu::Texture>,
    scene_pixels: Option<Vec<u8>>,
    queue: Option<wgpu::Queue>,
    logical_windows: HashMap<fret_core::AppWindowId, String>,
    window_placements: HashMap<fret_core::AppWindowId, fret_core::DockWindowPlacementV1>,
    next_floating_index: u32,
    loaded_layout: Option<DockLayoutV1>,
    selection: Option<Model<DemoSelection>>,
}

impl DemoDriver {
    fn layout_path() -> &'static Path {
        Path::new("./.fret/layout.json")
    }

    fn keymap_path() -> &'static Path {
        Path::new("./.fret/keymap.json")
    }

    fn load_layout_file() -> Option<DockLayoutV1> {
        let path = Self::layout_path();
        let file = File::open(path).ok()?;
        serde_json::from_reader(file).ok()
    }

    fn load_keymap_file() -> Result<Keymap, fret_app::KeymapError> {
        Keymap::from_file(Self::keymap_path())
    }

    fn save_layout_file(layout: &DockLayoutV1) -> std::io::Result<()> {
        if let Some(parent) = Self::layout_path().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = File::create(Self::layout_path())?;
        serde_json::to_writer_pretty(file, layout)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(())
    }

    fn ensure_main_tabs(
        dock: &mut DockManager,
        main: fret_core::AppWindowId,
    ) -> fret_core::DockNodeId {
        dock.graph.first_tabs_in_window(main).unwrap_or_else(|| {
            let tabs = dock.graph.insert_node(DockNode::Tabs {
                tabs: Vec::new(),
                active: 0,
            });
            dock.graph.set_window_root(main, tabs);
            tabs
        })
    }

    fn stamp_scene(&mut self, target: RenderTargetId, target_px: (u32, u32)) {
        let (Some(scene_target), Some((w, h)), Some(texture), Some(queue), Some(pixels)) = (
            self.scene_target,
            self.scene_target_size,
            self.scene_texture.as_ref(),
            self.queue.as_ref(),
            self.scene_pixels.as_mut(),
        ) else {
            return;
        };
        if target != scene_target {
            return;
        }

        let (x, y) = target_px;
        let cx = x.min(w.saturating_sub(1));
        let cy = y.min(h.saturating_sub(1));

        let mark = [240u8, 240u8, 245u8, 255u8];
        let ring = [255u8, 90u8, 70u8, 255u8];
        let r: i32 = 7;
        let r2 = r * r;
        let r_inner = (r - 1).max(0);
        let r_inner2 = r_inner * r_inner;

        for dy in -r..=r {
            for dx in -r..=r {
                let d2 = dx * dx + dy * dy;
                if d2 > r2 {
                    continue;
                }

                let px = cx as i32 + dx;
                let py = cy as i32 + dy;
                if px < 0 || py < 0 || px >= w as i32 || py >= h as i32 {
                    continue;
                }

                let is_cross = dx == 0 || dy == 0;
                let is_ring = d2 >= r_inner2;
                if is_cross || is_ring {
                    let rgba = if is_ring { ring } else { mark };
                    let idx = ((py as u32 * w + px as u32) * 4) as usize;
                    pixels[idx..idx + 4].copy_from_slice(&rgba);
                }
            }
        }

        queue.write_texture(
            texture.as_image_copy(),
            pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * w),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
    }

    fn next_floating_logical_id(&mut self) -> String {
        let n = self.next_floating_index.max(1);
        self.next_floating_index = n.saturating_add(1);
        format!("floating-{n}")
    }

    fn window_list_for_export(
        &mut self,
        dock: &DockManager,
    ) -> Vec<(fret_core::AppWindowId, String)> {
        let mut out: Vec<(fret_core::AppWindowId, String)> = Vec::new();
        for (&window, logical) in &self.logical_windows {
            if dock.graph.window_root(window).is_some() {
                out.push((window, logical.clone()));
            }
        }
        out
    }

    fn ensure_layout_panels(dock: &mut DockManager, layout: &DockLayoutV1) {
        for node in &layout.nodes {
            if let DockLayoutNodeV1::Tabs { tabs, .. } = node {
                for key in tabs {
                    dock.ensure_panel(key, || DockPanel {
                        title: format!("Missing: {}", key.kind.0),
                        color: Color {
                            r: 0.18,
                            g: 0.18,
                            b: 0.20,
                            a: 1.0,
                        },
                        viewport: None,
                    });
                }
            }
        }
    }
}

impl WinitDriver for DemoDriver {
    type WindowState = DemoWindowState;

    fn gpu_ready(&mut self, _app: &mut App, context: &WgpuContext, renderer: &mut Renderer) {
        let size = 512u32;
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        self.queue = Some(context.queue.clone());
        let texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("fret-demo scene render target"),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let mut pixels: Vec<u8> = vec![0; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let idx = ((y * size + x) * 4) as usize;
                let check = ((x / 32) ^ (y / 32)) & 1;
                let (r, g, b) = if check == 0 {
                    (24u8, 28u8, 40u8)
                } else {
                    (42u8, 55u8, 90u8)
                };
                pixels[idx] = r;
                pixels[idx + 1] = g;
                pixels[idx + 2] = b;
                pixels[idx + 3] = 255u8;
            }
        }

        context.queue.write_texture(
            texture.as_image_copy(),
            &pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * size),
                rows_per_image: Some(size),
            },
            wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let target = renderer.register_render_target(RenderTargetDescriptor {
            view,
            size: (size, size),
            format,
            color_space: RenderTargetColorSpace::Srgb,
        });

        self.scene_target = Some(target);
        self.scene_target_size = Some((size, size));
        self.scene_texture = Some(texture);
        self.scene_pixels = Some(pixels);
    }

    fn init(&mut self, app: &mut App, main_window: fret_core::AppWindowId) {
        self.main_window = Some(main_window);
        self.logical_windows.insert(main_window, "main".to_string());

        app.commands_mut().register(
            CommandId::from("command_palette.toggle"),
            CommandMeta::new("Toggle Command Palette")
                .with_description("Opens/closes the command palette overlay")
                .with_category("View")
                .with_keywords(["palette", "command", "search"]),
        );
        app.commands_mut().register(
            CommandId::from("command_palette.close"),
            CommandMeta::new("Close Command Palette")
                .with_description("Closes the command palette overlay")
                .with_category("View")
                .with_keywords(["palette", "command"]),
        );

        app.commands_mut().register(
            CommandId::from("context_menu.open"),
            CommandMeta::new("Open Context Menu")
                .with_description("Internal: opens a context menu overlay")
                .with_category("View")
                .hidden(),
        );
        app.commands_mut().register(
            CommandId::from("context_menu.close"),
            CommandMeta::new("Close Context Menu")
                .with_description("Closes the context menu overlay")
                .with_category("View")
                .hidden(),
        );

        app.commands_mut().register(
            CommandId::from("tree_view.expand"),
            CommandMeta::new("Expand")
                .with_description("Expand the selected tree node")
                .with_category("Hierarchy")
                .with_scope(CommandScope::Widget),
        );
        app.commands_mut().register(
            CommandId::from("tree_view.collapse"),
            CommandMeta::new("Collapse")
                .with_description("Collapse the selected tree node")
                .with_category("Hierarchy")
                .with_scope(CommandScope::Widget),
        );
        app.commands_mut().register(
            CommandId::from("tree_view.expand_all"),
            CommandMeta::new("Expand All")
                .with_description("Expand all tree nodes")
                .with_category("Hierarchy")
                .with_scope(CommandScope::Widget),
        );
        app.commands_mut().register(
            CommandId::from("tree_view.collapse_all"),
            CommandMeta::new("Collapse All")
                .with_description("Collapse all tree nodes")
                .with_category("Hierarchy")
                .with_scope(CommandScope::Widget),
        );

        app.commands_mut().register(
            CommandId::from("virtual_list.copy_label"),
            CommandMeta::new("Copy Label")
                .with_description("Copies the label of the selected list row")
                .with_category("List")
                .with_scope(CommandScope::Widget)
                .hidden(),
        );
        app.commands_mut().register(
            CommandId::from("virtual_list.clear_selection"),
            CommandMeta::new("Clear Selection")
                .with_description("Clears the list selection")
                .with_category("List")
                .with_scope(CommandScope::Widget)
                .hidden(),
        );

        app.commands_mut().register(
            CommandId::from("viewport.copy_uv"),
            CommandMeta::new("Copy Viewport UV")
                .with_description("Copies the last viewport cursor UV to clipboard")
                .with_category("Viewport")
                .with_scope(CommandScope::Widget)
                .hidden(),
        );
        app.commands_mut().register(
            CommandId::from("viewport.copy_target_px"),
            CommandMeta::new("Copy Viewport Target Px")
                .with_description("Copies the last viewport cursor target pixel coordinates")
                .with_category("Viewport")
                .with_scope(CommandScope::Widget)
                .hidden(),
        );

        app.commands_mut().register(
            CommandId::from("demo.toggle_modal"),
            CommandMeta::new("Toggle Modal Overlay")
                .with_description("Demo-only: toggles the modal overlay layer")
                .with_category("Demo"),
        );
        app.commands_mut().register(
            CommandId::from("demo.toggle_dnd_overlay"),
            CommandMeta::new("Toggle DnD Overlay")
                .with_description("Demo-only: toggles the external drag overlay layer")
                .with_category("Demo"),
        );
        app.commands_mut().register(
            CommandId::from("text.clear"),
            CommandMeta::new("Clear Text Input")
                .with_description("Clears the focused text input")
                .with_category("Edit")
                .with_keywords(["text", "input"])
                .with_scope(CommandScope::Widget),
        );
        app.commands_mut().register(
            CommandId::from("text.select_all"),
            CommandMeta::new("Select All")
                .with_description("Select all text in the focused text input")
                .with_category("Edit")
                .with_scope(CommandScope::Widget),
        );
        app.commands_mut().register(
            CommandId::from("text.copy"),
            CommandMeta::new("Copy")
                .with_description("Copy selected text")
                .with_category("Edit")
                .with_scope(CommandScope::Widget),
        );
        app.commands_mut().register(
            CommandId::from("text.cut"),
            CommandMeta::new("Cut")
                .with_description("Cut selected text")
                .with_category("Edit")
                .with_scope(CommandScope::Widget),
        );
        app.commands_mut().register(
            CommandId::from("text.paste"),
            CommandMeta::new("Paste")
                .with_description("Paste clipboard text")
                .with_category("Edit")
                .with_scope(CommandScope::Widget),
        );

        for (id, title, desc) in [
            (
                "text.move_left",
                "Move Left",
                "Move caret left by one character",
            ),
            (
                "text.move_right",
                "Move Right",
                "Move caret right by one character",
            ),
            (
                "text.move_word_left",
                "Move Word Left",
                "Move caret left by one word",
            ),
            (
                "text.move_word_right",
                "Move Word Right",
                "Move caret right by one word",
            ),
            ("text.move_home", "Move Home", "Move caret to the start"),
            ("text.move_end", "Move End", "Move caret to the end"),
            ("text.move_up", "Move Up", "Move caret up by one line"),
            ("text.move_down", "Move Down", "Move caret down by one line"),
            (
                "text.select_left",
                "Select Left",
                "Extend selection left by one character",
            ),
            (
                "text.select_right",
                "Select Right",
                "Extend selection right by one character",
            ),
            (
                "text.select_word_left",
                "Select Word Left",
                "Extend selection left by one word",
            ),
            (
                "text.select_word_right",
                "Select Word Right",
                "Extend selection right by one word",
            ),
            (
                "text.select_home",
                "Select Home",
                "Extend selection to the start",
            ),
            (
                "text.select_end",
                "Select End",
                "Extend selection to the end",
            ),
            (
                "text.select_up",
                "Select Up",
                "Extend selection up by one line",
            ),
            (
                "text.select_down",
                "Select Down",
                "Extend selection down by one line",
            ),
            (
                "text.delete_backward",
                "Delete Backward",
                "Delete backward (or delete selection)",
            ),
            (
                "text.delete_forward",
                "Delete Forward",
                "Delete forward (or delete selection)",
            ),
            (
                "text.delete_word_backward",
                "Delete Word Backward",
                "Delete backward by one word (or delete selection)",
            ),
            (
                "text.delete_word_forward",
                "Delete Word Forward",
                "Delete forward by one word (or delete selection)",
            ),
        ] {
            let repeatable = id.starts_with("text.move")
                || id.starts_with("text.select")
                || id.starts_with("text.delete");
            let mut meta = CommandMeta::new(title)
                .with_description(desc)
                .with_category("Edit")
                .with_scope(CommandScope::Widget);
            if repeatable {
                meta = meta.repeatable();
            }
            app.commands_mut().register(CommandId::from(id), meta);
        }
        app.commands_mut().register(
            CommandId::from("focus.next"),
            CommandMeta::new("Focus Next")
                .with_description("Move focus to the next focusable control")
                .with_category("View"),
        );
        app.commands_mut().register(
            CommandId::from("focus.previous"),
            CommandMeta::new("Focus Previous")
                .with_description("Move focus to the previous focusable control")
                .with_category("View"),
        );

        let default_keymap = Keymap::from_v1(KeymapFileV1 {
            keymap_version: 1,
            bindings: vec![
                BindingV1 {
                    command: Some("focus.next".into()),
                    platform: Some("all".into()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "Tab".into(),
                    },
                },
                BindingV1 {
                    command: Some("focus.previous".into()),
                    platform: Some("all".into()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec!["shift".into()],
                        key: "Tab".into(),
                    },
                },
                BindingV1 {
                    command: Some("command_palette.toggle".into()),
                    platform: Some("macos".into()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec!["meta".into()],
                        key: "KeyP".into(),
                    },
                },
                BindingV1 {
                    command: Some("command_palette.toggle".into()),
                    platform: Some("windows".into()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyP".into(),
                    },
                },
                BindingV1 {
                    command: Some("command_palette.toggle".into()),
                    platform: Some("linux".into()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyP".into(),
                    },
                },
                BindingV1 {
                    command: Some("demo.toggle_modal".into()),
                    platform: Some("all".into()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "F1".into(),
                    },
                },
                BindingV1 {
                    command: Some("demo.toggle_dnd_overlay".into()),
                    platform: Some("all".into()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "F2".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.clear".into()),
                    platform: Some("all".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyL".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_all".into()),
                    platform: Some("macos".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["meta".into()],
                        key: "KeyA".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.copy".into()),
                    platform: Some("macos".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["meta".into()],
                        key: "KeyC".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.cut".into()),
                    platform: Some("macos".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["meta".into()],
                        key: "KeyX".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.paste".into()),
                    platform: Some("macos".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["meta".into()],
                        key: "KeyV".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_all".into()),
                    platform: Some("windows".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyA".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.copy".into()),
                    platform: Some("windows".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyC".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.cut".into()),
                    platform: Some("windows".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyX".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.paste".into()),
                    platform: Some("windows".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyV".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_all".into()),
                    platform: Some("linux".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyA".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.copy".into()),
                    platform: Some("linux".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyC".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.cut".into()),
                    platform: Some("linux".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyX".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.paste".into()),
                    platform: Some("linux".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyV".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.move_left".into()),
                    platform: Some("all".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "ArrowLeft".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.move_right".into()),
                    platform: Some("all".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "ArrowRight".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.move_home".into()),
                    platform: Some("all".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "Home".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.move_end".into()),
                    platform: Some("all".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "End".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.move_up".into()),
                    platform: Some("all".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "ArrowUp".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.move_down".into()),
                    platform: Some("all".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "ArrowDown".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_left".into()),
                    platform: Some("all".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["shift".into()],
                        key: "ArrowLeft".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_right".into()),
                    platform: Some("all".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["shift".into()],
                        key: "ArrowRight".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_up".into()),
                    platform: Some("all".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["shift".into()],
                        key: "ArrowUp".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_down".into()),
                    platform: Some("all".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["shift".into()],
                        key: "ArrowDown".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_home".into()),
                    platform: Some("all".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["shift".into()],
                        key: "Home".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_end".into()),
                    platform: Some("all".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["shift".into()],
                        key: "End".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.delete_backward".into()),
                    platform: Some("all".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "Backspace".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.delete_forward".into()),
                    platform: Some("all".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "Delete".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.move_word_left".into()),
                    platform: Some("macos".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["alt".into()],
                        key: "ArrowLeft".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.move_word_right".into()),
                    platform: Some("macos".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["alt".into()],
                        key: "ArrowRight".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_word_left".into()),
                    platform: Some("macos".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["alt".into(), "shift".into()],
                        key: "ArrowLeft".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_word_right".into()),
                    platform: Some("macos".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["alt".into(), "shift".into()],
                        key: "ArrowRight".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.delete_word_backward".into()),
                    platform: Some("macos".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["alt".into()],
                        key: "Backspace".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.delete_word_forward".into()),
                    platform: Some("macos".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["alt".into()],
                        key: "Delete".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.move_word_left".into()),
                    platform: Some("windows".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "ArrowLeft".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.move_word_right".into()),
                    platform: Some("windows".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "ArrowRight".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_word_left".into()),
                    platform: Some("windows".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into(), "shift".into()],
                        key: "ArrowLeft".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_word_right".into()),
                    platform: Some("windows".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into(), "shift".into()],
                        key: "ArrowRight".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.delete_word_backward".into()),
                    platform: Some("windows".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "Backspace".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.delete_word_forward".into()),
                    platform: Some("windows".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "Delete".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.move_word_left".into()),
                    platform: Some("linux".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "ArrowLeft".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.move_word_right".into()),
                    platform: Some("linux".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "ArrowRight".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_word_left".into()),
                    platform: Some("linux".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into(), "shift".into()],
                        key: "ArrowLeft".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_word_right".into()),
                    platform: Some("linux".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into(), "shift".into()],
                        key: "ArrowRight".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.delete_word_backward".into()),
                    platform: Some("linux".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "Backspace".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.delete_word_forward".into()),
                    platform: Some("linux".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "Delete".into(),
                    },
                },
            ],
        })
        .expect("default keymap must parse");

        let mut merged = default_keymap;
        match Self::load_keymap_file() {
            Ok(user) => merged.extend(user),
            Err(e) => {
                tracing::info!(error = ?e, path = %Self::keymap_path().display(), "no user keymap loaded");
            }
        }
        app.set_global(KeymapService { keymap: merged });

        let mut dock = DockManager::default();
        let key_scene = PanelKey::new("core.scene");
        dock.insert_panel(
            key_scene.clone(),
            DockPanel {
                title: "Scene".to_string(),
                color: Color {
                    r: 0.12,
                    g: 0.16,
                    b: 0.22,
                    a: 1.0,
                },
                viewport: self.scene_target.zip(self.scene_target_size).map(
                    |(target, target_px_size)| ViewportPanel {
                        target,
                        target_px_size,
                        fit: fret_core::ViewportFit::Contain,
                    },
                ),
            },
        );
        let key_inspector = PanelKey::new("core.inspector");
        dock.insert_panel(
            key_inspector.clone(),
            DockPanel {
                title: "Inspector".to_string(),
                color: Color {
                    r: 0.16,
                    g: 0.14,
                    b: 0.20,
                    a: 1.0,
                },
                viewport: None,
            },
        );
        let key_hierarchy = PanelKey::new("core.hierarchy");
        dock.insert_panel(
            key_hierarchy.clone(),
            DockPanel {
                title: "Hierarchy".to_string(),
                color: Color {
                    r: 0.15,
                    g: 0.18,
                    b: 0.14,
                    a: 1.0,
                },
                viewport: None,
            },
        );

        if let Some(layout) = Self::load_layout_file() {
            Self::ensure_layout_panels(&mut dock, &layout);
            self.loaded_layout = Some(layout.clone());

            if let Some(main_entry) = layout
                .windows
                .iter()
                .find(|w| w.logical_window_id == "main")
            {
                if let Some(root) = dock
                    .graph
                    .import_subtree_from_layout_v1(&layout, main_entry.root)
                {
                    dock.graph.set_window_root(main_window, root);
                }
            }

            for w in &layout.windows {
                if w.logical_window_id == "main" {
                    continue;
                }
                app.push_effect(Effect::Window(WindowRequest::Create(CreateWindowRequest {
                    kind: CreateWindowKind::DockRestore {
                        logical_window_id: w.logical_window_id.clone(),
                    },
                    anchor: None,
                })));
            }
        } else {
            let tabs_left = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![key_hierarchy],
                active: 0,
            });
            let tabs_scene = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![key_scene],
                active: 0,
            });
            let tabs_inspector = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![key_inspector],
                active: 0,
            });
            let right = dock.graph.insert_node(DockNode::Split {
                axis: Axis::Vertical,
                children: vec![tabs_scene, tabs_inspector],
                fractions: vec![0.72, 0.28],
            });
            let root_dock = dock.graph.insert_node(DockNode::Split {
                axis: Axis::Horizontal,
                children: vec![tabs_left, right],
                fractions: vec![0.26, 0.74],
            });
            dock.graph.set_window_root(main_window, root_dock);
        }

        app.set_global(dock);

        if self.selection.is_none() {
            self.selection = Some(app.models_mut().insert(DemoSelection::default()));
        }
    }

    fn create_window_state(
        &mut self,
        _app: &mut App,
        window: fret_core::AppWindowId,
    ) -> Self::WindowState {
        let selection = self.selection.expect("selection model initialized");
        let (ui, layers) = build_demo_ui(window, DemoUiConfig::default(), selection);
        Self::WindowState {
            ui,
            layers,
            palette_previous_focus: None,
            context_menu_previous_focus: None,
        }
    }

    fn handle_event(
        &mut self,
        app: &mut App,
        text: &mut dyn fret_core::TextService,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        event: &fret_core::Event,
    ) {
        if let fret_core::Event::ExternalDrag(drag) = event {
            tracing::info!(window = ?window, ?drag, "external drag event received");
            match &drag.kind {
                fret_core::ExternalDragKind::EnterFiles(_)
                | fret_core::ExternalDragKind::OverFiles(_) => {
                    if !state.ui.is_layer_visible(state.layers.external_dnd) {
                        state.ui.set_layer_visible(state.layers.external_dnd, true);
                        app.request_redraw(window);
                    }
                }
                fret_core::ExternalDragKind::DropFiles(_) | fret_core::ExternalDragKind::Leave => {
                    if state.ui.is_layer_visible(state.layers.external_dnd) {
                        state.ui.set_layer_visible(state.layers.external_dnd, false);
                        app.request_redraw(window);
                    }
                }
            }
        }

        match event {
            fret_core::Event::WindowResized { width, height } => {
                let entry = self.window_placements.entry(window).or_insert(
                    fret_core::DockWindowPlacementV1 {
                        width: 640,
                        height: 480,
                        x: None,
                        y: None,
                        monitor_hint: None,
                    },
                );
                entry.width = width.0.max(1.0).round() as u32;
                entry.height = height.0.max(1.0).round() as u32;
            }
            fret_core::Event::WindowMoved { x, y } => {
                let entry = self.window_placements.entry(window).or_insert(
                    fret_core::DockWindowPlacementV1 {
                        width: 640,
                        height: 480,
                        x: None,
                        y: None,
                        monitor_hint: None,
                    },
                );
                entry.x = Some(*x);
                entry.y = Some(*y);
            }
            _ => {}
        }

        if let fret_core::Event::Pointer(pe) = event {
            if let fret_core::PointerEvent::Down { .. } = pe {
                if state.ui.is_layer_visible(state.layers.command_palette) {
                    // Command palette uses its own backdrop to dismiss; avoid demo-only right-click modal.
                    state.ui.dispatch_event(app, text, event);
                    return;
                }
                if state.ui.is_layer_visible(state.layers.modal) {
                    state.ui.set_layer_visible(state.layers.modal, false);
                    app.request_redraw(window);
                    return;
                }
            }
        }
        state.ui.dispatch_event(app, text, event);
    }

    fn handle_command(
        &mut self,
        app: &mut App,
        text: &mut dyn fret_core::TextService,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        command: CommandId,
    ) {
        if state.ui.dispatch_command(app, text, &command) {
            return;
        }

        match command.as_str() {
            "command_palette.toggle" => {
                let vis = state.ui.is_layer_visible(state.layers.command_palette);
                if vis {
                    state
                        .ui
                        .set_layer_visible(state.layers.command_palette, false);
                    if let Some(prev) = state.palette_previous_focus.take() {
                        state.ui.set_focus(Some(prev));
                    }
                } else {
                    state.palette_previous_focus = state.ui.focus();
                    state
                        .ui
                        .set_layer_visible(state.layers.command_palette, true);
                    state.ui.set_focus(Some(state.layers.command_palette_node));
                }
                app.request_redraw(window);
            }
            "command_palette.close" => {
                if state.ui.is_layer_visible(state.layers.command_palette) {
                    state
                        .ui
                        .set_layer_visible(state.layers.command_palette, false);
                    if let Some(prev) = state.palette_previous_focus.take() {
                        state.ui.set_focus(Some(prev));
                    }
                    app.request_redraw(window);
                }
            }
            "context_menu.open" => {
                if state.ui.is_layer_visible(state.layers.command_palette) {
                    return;
                }

                let has_request = app
                    .global::<ContextMenuService>()
                    .and_then(|s| s.request(window))
                    .is_some();
                if !has_request {
                    return;
                }

                state.context_menu_previous_focus = state.ui.focus();
                state.ui.set_layer_visible(state.layers.context_menu, true);
                state.ui.set_focus(Some(state.layers.context_menu_node));
                app.request_redraw(window);
            }
            "context_menu.close" => {
                if state.ui.is_layer_visible(state.layers.context_menu) {
                    state.ui.set_layer_visible(state.layers.context_menu, false);
                }

                app.with_global_mut(ContextMenuService::default, |service, app| {
                    let action = service.take_pending_action(window);
                    service.clear(window);
                    if let Some(command) = action {
                        app.push_effect(Effect::Command {
                            window: Some(window),
                            command,
                        });
                    }
                });

                if let Some(prev) = state.context_menu_previous_focus.take() {
                    state.ui.set_focus(Some(prev));
                }

                app.request_redraw(window);
            }
            "demo.toggle_modal" => {
                let vis = state.ui.is_layer_visible(state.layers.modal);
                state.ui.set_layer_visible(state.layers.modal, !vis);
                app.request_redraw(window);
            }
            "demo.toggle_dnd_overlay" => {
                let vis = state.ui.is_layer_visible(state.layers.external_dnd);
                state.ui.set_layer_visible(state.layers.external_dnd, !vis);
                app.request_redraw(window);
            }
            _ => {}
        }
    }

    fn dock_op(&mut self, app: &mut App, op: DockOp) {
        if let DockOp::RequestFloatPanelToNewWindow {
            source_window,
            panel,
            anchor,
        } = &op
        {
            app.push_effect(Effect::Window(WindowRequest::Create(CreateWindowRequest {
                kind: CreateWindowKind::DockFloating {
                    source_window: *source_window,
                    panel: panel.clone(),
                },
                anchor: *anchor,
            })));
            return;
        }

        let mut close_if_empty: Option<fret_core::AppWindowId> = None;
        let mut redraw: Vec<fret_core::AppWindowId> = Vec::new();

        {
            let Some(dock) = app.global_mut::<DockManager>() else {
                return;
            };

            let _ = dock.graph.apply_op(&op);

            if let DockOp::MovePanel { source_window, .. } = &op {
                if dock
                    .graph
                    .collect_panels_in_window(*source_window)
                    .is_empty()
                    && Some(*source_window) != self.main_window
                {
                    close_if_empty = Some(*source_window);
                }
            }
            if let DockOp::FloatPanelToWindow { source_window, .. } = &op {
                if dock
                    .graph
                    .collect_panels_in_window(*source_window)
                    .is_empty()
                    && Some(*source_window) != self.main_window
                {
                    close_if_empty = Some(*source_window);
                }
            }

            for (&w, _) in &self.logical_windows {
                if dock.graph.window_root(w).is_some() {
                    redraw.push(w);
                }
            }
        }

        if let Some(window) = close_if_empty {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
        }
        for w in redraw {
            app.request_redraw(w);
        }
    }

    fn viewport_input(&mut self, app: &mut App, event: fret_core::ViewportInputEvent) {
        match event.kind {
            fret_core::ViewportInputKind::PointerDown { button, .. } => {
                println!("viewport_input: {event:?}");
                if button == fret_core::MouseButton::Left {
                    self.stamp_scene(event.target, event.target_px);
                    app.request_redraw(event.window);
                }
            }
            fret_core::ViewportInputKind::PointerUp { .. }
            | fret_core::ViewportInputKind::Wheel { .. } => {
                println!("viewport_input: {event:?}");
            }
            fret_core::ViewportInputKind::PointerMove { .. } => {}
        }
    }

    fn render(
        &mut self,
        app: &mut App,
        _window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        bounds: Rect,
        scale_factor: f32,
        text: &mut dyn fret_core::TextService,
        scene: &mut Scene,
    ) {
        scene.clear();
        state.ui.layout_all(app, text, bounds, scale_factor);
        state.ui.paint_all(app, text, bounds, scene, scale_factor);
    }

    fn window_create_spec(
        &mut self,
        app: &mut App,
        request: &CreateWindowRequest,
    ) -> Option<WindowCreateSpec> {
        match &request.kind {
            CreateWindowKind::DockFloating { panel, .. } => {
                let title = app
                    .global::<DockManager>()
                    .and_then(|dock| dock.panel(panel))
                    .map(|p| p.title.clone())
                    .unwrap_or_else(|| "Floating".to_string());
                Some(WindowCreateSpec::new(
                    format!("fret-demo - {title}"),
                    winit::dpi::LogicalSize::new(640.0, 480.0),
                ))
            }
            CreateWindowKind::DockRestore { logical_window_id } => {
                let mut spec = WindowCreateSpec::new(
                    format!("fret-demo - {logical_window_id}"),
                    winit::dpi::LogicalSize::new(640.0, 480.0),
                );

                if let Some(layout) = self.loaded_layout.as_ref() {
                    if let Some(entry) = layout
                        .windows
                        .iter()
                        .find(|w| w.logical_window_id == *logical_window_id)
                    {
                        if let Some(p) = entry.placement.as_ref() {
                            spec.size =
                                winit::dpi::LogicalSize::new(p.width as f64, p.height as f64);
                            if let (Some(x), Some(y)) = (p.x, p.y) {
                                spec.position = Some(winit::dpi::Position::Logical(
                                    winit::dpi::LogicalPosition::new(x as f64, y as f64),
                                ));
                            }
                        }
                    }
                }

                Some(spec)
            }
        }
    }

    fn window_created(
        &mut self,
        app: &mut App,
        request: &CreateWindowRequest,
        new_window: fret_core::AppWindowId,
    ) {
        match &request.kind {
            CreateWindowKind::DockFloating {
                source_window,
                panel,
            } => {
                self.dock_op(
                    app,
                    DockOp::FloatPanelToWindow {
                        source_window: *source_window,
                        panel: panel.clone(),
                        new_window,
                    },
                );

                if !self.logical_windows.contains_key(&new_window) {
                    let id = self.next_floating_logical_id();
                    self.logical_windows.insert(new_window, id);
                }

                app.request_redraw(*source_window);
                app.request_redraw(new_window);
            }
            CreateWindowKind::DockRestore { logical_window_id } => {
                self.logical_windows
                    .insert(new_window, logical_window_id.clone());

                let Some(layout) = self.loaded_layout.as_ref() else {
                    return;
                };
                let Some(entry) = layout
                    .windows
                    .iter()
                    .find(|w| w.logical_window_id == *logical_window_id)
                else {
                    return;
                };

                let Some(dock) = app.global_mut::<DockManager>() else {
                    return;
                };
                if let Some(root) = dock.graph.import_subtree_from_layout_v1(layout, entry.root) {
                    dock.graph.set_window_root(new_window, root);
                    app.request_redraw(new_window);
                }
            }
        }
    }

    fn before_close_window(&mut self, app: &mut App, window: fret_core::AppWindowId) -> bool {
        let Some(main) = self.main_window else {
            return true;
        };
        if window == main {
            if let Some(dock) = app.global::<DockManager>() {
                let windows = self.window_list_for_export(dock);
                let layout = dock.graph.export_layout_v1_with_placement(&windows, |w| {
                    self.window_placements.get(&w).cloned()
                });
                if let Err(e) = Self::save_layout_file(&layout) {
                    tracing::error!(error = ?e, "failed to save layout.json");
                }
            }
            return true;
        }

        let Some(dock) = app.global_mut::<DockManager>() else {
            return true;
        };

        let target_tabs = Self::ensure_main_tabs(dock, main);
        let _ = dock.graph.apply_op(&DockOp::MergeWindowInto {
            source_window: window,
            target_window: main,
            target_tabs,
        });
        self.logical_windows.remove(&window);

        app.request_redraw(main);
        true
    }
}

fn main() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_platform=info".parse().unwrap())
                .add_directive("fret_render=info".parse().unwrap()),
        )
        .try_init();

    let event_loop = EventLoop::new()?;
    let mut config = WinitRunnerConfig {
        main_window_title: "fret-demo".to_string(),
        ..Default::default()
    };

    if let Some(layout) = DemoDriver::load_layout_file() {
        if let Some(main_entry) = layout
            .windows
            .iter()
            .find(|w| w.logical_window_id == "main")
        {
            if let Some(p) = main_entry.placement.as_ref() {
                config.main_window_size =
                    winit::dpi::LogicalSize::new(p.width as f64, p.height as f64);
                if let (Some(x), Some(y)) = (p.x, p.y) {
                    config.main_window_position = Some(winit::dpi::Position::Logical(
                        winit::dpi::LogicalPosition::new(x as f64, y as f64),
                    ));
                }
            }
        }
    }

    let app = App::new();
    let driver = DemoDriver::default();
    let mut runner = WinitRunner::new(config, app, driver);
    event_loop.run_app(&mut runner)?;
    Ok(())
}

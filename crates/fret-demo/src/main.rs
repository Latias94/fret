mod demo_ui;

use demo_ui::{DemoUiConfig, build_demo_ui};

use fret_app::{App, CreateWindowKind, CreateWindowRequest, Effect, WindowRequest};
use fret_core::{Axis, Color, DockNode, DropZone, Rect, RenderTargetId, Scene};
use fret_platform::winit_runner::{WindowCreateSpec, WinitDriver, WinitRunner, WinitRunnerConfig};
use fret_render::{RenderTargetColorSpace, RenderTargetDescriptor, Renderer, WgpuContext};
use fret_ui::{DockManager, DockPanel, UiTree, ViewportPanel};
use winit::event_loop::EventLoop;

struct DemoWindowState {
    ui: UiTree,
    modal_layer: fret_ui::UiLayerId,
}

#[derive(Default)]
struct DemoDriver {
    main_window: Option<fret_core::AppWindowId>,
    scene_target: Option<RenderTargetId>,
    scene_target_size: Option<(u32, u32)>,
    scene_texture: Option<wgpu::Texture>,
    scene_pixels: Option<Vec<u8>>,
    queue: Option<wgpu::Queue>,
}

impl DemoDriver {
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

        let mut dock = DockManager::default();
        let panel_scene = dock.create_panel(DockPanel {
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
        });
        let panel_inspector = dock.create_panel(DockPanel {
            title: "Inspector".to_string(),
            color: Color {
                r: 0.16,
                g: 0.14,
                b: 0.20,
                a: 1.0,
            },
            viewport: None,
        });
        let panel_hierarchy = dock.create_panel(DockPanel {
            title: "Hierarchy".to_string(),
            color: Color {
                r: 0.15,
                g: 0.18,
                b: 0.14,
                a: 1.0,
            },
            viewport: None,
        });

        let tabs_left = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_hierarchy],
            active: 0,
        });
        let tabs_scene = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_scene],
            active: 0,
        });
        let tabs_inspector = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_inspector],
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
        app.set_global(dock);
    }

    fn create_window_state(
        &mut self,
        _app: &mut App,
        window: fret_core::AppWindowId,
    ) -> Self::WindowState {
        let (ui, modal_layer) = build_demo_ui(window, DemoUiConfig::default());
        Self::WindowState { ui, modal_layer }
    }

    fn handle_event(
        &mut self,
        app: &mut App,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        event: &fret_core::Event,
    ) {
        if let fret_core::Event::Pointer(pe) = event {
            if let fret_core::PointerEvent::Down { button, .. } = pe {
                if state.ui.is_layer_visible(state.modal_layer) {
                    state.ui.set_layer_visible(state.modal_layer, false);
                    app.request_redraw(window);
                    return;
                }

                if *button == fret_core::MouseButton::Right {
                    state.ui.set_layer_visible(state.modal_layer, true);
                    app.request_redraw(window);
                    return;
                }
            }
        }
        state.ui.dispatch_event(app, event);
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
        scene: &mut Scene,
    ) {
        scene.clear();
        state.ui.layout_all(app, bounds);
        state.ui.paint_all(app, bounds, scene);
    }

    fn window_create_spec(
        &mut self,
        app: &mut App,
        request: CreateWindowRequest,
    ) -> Option<WindowCreateSpec> {
        match request.kind {
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
        }
    }

    fn window_created(
        &mut self,
        app: &mut App,
        request: CreateWindowRequest,
        new_window: fret_core::AppWindowId,
    ) {
        match request.kind {
            CreateWindowKind::DockFloating {
                source_window,
                panel,
            } => {
                let empty = {
                    let Some(dock) = app.global_mut::<DockManager>() else {
                        return;
                    };
                    dock.graph
                        .float_panel_to_window(source_window, panel, new_window);
                    dock.graph
                        .collect_panels_in_window(source_window)
                        .is_empty()
                };

                app.request_redraw(source_window);
                app.request_redraw(new_window);

                if empty && Some(source_window) != self.main_window {
                    app.push_effect(Effect::Window(WindowRequest::Close(source_window)));
                }
            }
        }
    }

    fn before_close_window(&mut self, app: &mut App, window: fret_core::AppWindowId) -> bool {
        let Some(main) = self.main_window else {
            return true;
        };
        if window == main {
            return true;
        }

        let Some(dock) = app.global_mut::<DockManager>() else {
            return true;
        };

        let target_tabs = Self::ensure_main_tabs(dock, main);
        let panels = dock.graph.collect_panels_in_window(window);
        for panel in panels {
            dock.graph.move_panel_between_windows(
                window,
                panel,
                main,
                target_tabs,
                DropZone::Center,
                None,
            );
        }
        dock.graph.remove_window_root(window);

        app.request_redraw(main);
        true
    }
}

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    let config = WinitRunnerConfig {
        main_window_title: "fret-demo".to_string(),
        ..Default::default()
    };
    let app = App::new();
    let driver = DemoDriver::default();
    let mut runner = WinitRunner::new(config, app, driver);
    event_loop.run_app(&mut runner)?;
    Ok(())
}

use std::{collections::HashMap, sync::Arc};

use fret_app::App;
use fret_core::{
    Axis, Color, DockNode, DropZone, Modifiers, MouseButton, Point, Px, Rect, Scene, Size,
};
use fret_render::{ClearColor, Renderer, SurfaceState, WgpuContext};
use fret_ui::{DockManager, DockPanel, DockRequest, DockSpace, UiTree};
use slotmap::SlotMap;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, MouseButton as WinitMouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::ModifiersState,
    window::Window,
};

struct WindowState {
    window: Arc<Window>,
    surface: SurfaceState<'static>,
    ui: UiTree,
    root: fret_core::NodeId,
    scene: Scene,
    cursor_pos: Point,
}

#[derive(Default)]
struct DemoApp {
    context: Option<WgpuContext>,
    renderer: Option<Renderer>,

    app: App,
    windows: SlotMap<fret_core::AppWindowId, WindowState>,
    winit_to_app: HashMap<winit::window::WindowId, fret_core::AppWindowId>,

    main_window: Option<fret_core::AppWindowId>,
    modifiers: Modifiers,
}

impl DemoApp {
    fn create_window(
        &mut self,
        event_loop: &ActiveEventLoop,
        title: &str,
        size: LogicalSize<f64>,
    ) -> anyhow::Result<(Arc<Window>, wgpu::Surface<'static>)> {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title(title)
                        .with_inner_size(size),
                )
                .expect("create window"),
        );

        let Some(context) = self.context.as_ref() else {
            anyhow::bail!("wgpu context not initialized");
        };
        let surface = context.create_surface(window.clone())?;
        Ok((window, surface))
    }

    fn insert_window_state(
        &mut self,
        window: Arc<Window>,
        surface: wgpu::Surface<'static>,
    ) -> anyhow::Result<fret_core::AppWindowId> {
        let Some(context) = self.context.as_ref() else {
            anyhow::bail!("wgpu context not initialized");
        };

        let size = window.inner_size();
        let surface = SurfaceState::new(
            &context.adapter,
            &context.device,
            surface,
            size.width,
            size.height,
        )?;

        let id = self.windows.insert_with_key(|id| {
            let mut ui = UiTree::new();
            let root = ui.create_node(DockSpace::new(id));
            ui.set_root(root);
            WindowState {
                window,
                surface,
                ui,
                root,
                scene: Scene::default(),
                cursor_pos: Point::new(Px(0.0), Px(0.0)),
            }
        });

        let winit_id = self.windows[id].window.id();
        self.winit_to_app.insert(winit_id, id);
        Ok(id)
    }

    fn resize_surface(&mut self, window: fret_core::AppWindowId, width: u32, height: u32) {
        let Some(context) = self.context.as_ref() else {
            return;
        };
        let Some(state) = self.windows.get_mut(window) else {
            return;
        };
        state.surface.resize(&context.device, width, height);
    }

    fn dispatch_pointer_event(
        &mut self,
        window: fret_core::AppWindowId,
        pe: fret_core::PointerEvent,
    ) {
        let (app, windows) = (&mut self.app, &mut self.windows);
        let Some(state) = windows.get_mut(window) else {
            return;
        };
        state.ui.dispatch_event(app, &fret_core::Event::Pointer(pe));
        state.window.request_redraw();
    }

    fn process_dock_requests(&mut self, event_loop: &ActiveEventLoop) {
        let requests = match self.app.global_mut::<DockManager>() {
            Some(dock) => dock.take_requests(),
            None => return,
        };

        for request in requests {
            match request {
                DockRequest::CreateFloatingWindow {
                    source_window,
                    panel,
                } => {
                    let title = self
                        .app
                        .global::<DockManager>()
                        .and_then(|dock| dock.panel(panel))
                        .map(|p| p.title.clone())
                        .unwrap_or_else(|| "Floating".to_string());

                    let (window, surface) = match self.create_window(
                        event_loop,
                        &format!("fret-demo - {title}"),
                        LogicalSize::new(640.0, 480.0),
                    ) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };

                    let new_window = match self.insert_window_state(window, surface) {
                        Ok(id) => id,
                        Err(_) => continue,
                    };

                    if let Some(dock) = self.app.global_mut::<DockManager>() {
                        dock.graph
                            .float_panel_to_window(source_window, panel, new_window);
                    }

                    if let Some(state) = self.windows.get(source_window) {
                        state.window.request_redraw();
                    }
                    if let Some(state) = self.windows.get(new_window) {
                        state.window.request_redraw();
                    }
                }
            }
        }
    }

    fn close_window(&mut self, window: fret_core::AppWindowId) {
        let Some(main) = self.main_window else {
            return;
        };

        if let Some(dock) = self.app.global_mut::<DockManager>() {
            let target_tabs = dock.graph.first_tabs_in_window(main).unwrap_or_else(|| {
                let tabs = dock.graph.insert_node(DockNode::Tabs {
                    tabs: Vec::new(),
                    active: 0,
                });
                dock.graph.set_window_root(main, tabs);
                tabs
            });

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
        }

        if let Some(state) = self.windows.remove(window) {
            self.winit_to_app.remove(&state.window.id());
        }
    }
}

impl ApplicationHandler for DemoApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.context.is_some() {
            return;
        }

        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("fret-demo")
                        .with_inner_size(LogicalSize::new(1280.0, 720.0)),
                )
                .expect("create window"),
        );

        let (context, surface) =
            pollster::block_on(WgpuContext::new_with_surface(window.clone())).expect("wgpu init");
        let renderer = Renderer::new(&context.device);

        self.context = Some(context);
        self.renderer = Some(renderer);

        let main_window = self
            .insert_window_state(window, surface)
            .expect("insert window state");

        let mut dock = DockManager::default();
        let panel_scene = dock.create_panel(DockPanel {
            title: "Scene".to_string(),
            color: Color {
                r: 0.12,
                g: 0.16,
                b: 0.22,
                a: 1.0,
            },
        });
        let panel_inspector = dock.create_panel(DockPanel {
            title: "Inspector".to_string(),
            color: Color {
                r: 0.16,
                g: 0.14,
                b: 0.20,
                a: 1.0,
            },
        });
        let panel_hierarchy = dock.create_panel(DockPanel {
            title: "Hierarchy".to_string(),
            color: Color {
                r: 0.15,
                g: 0.18,
                b: 0.14,
                a: 1.0,
            },
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
        self.app.set_global(dock);
        self.main_window = Some(main_window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(app_window) = self.winit_to_app.get(&window_id).copied() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                if Some(app_window) == self.main_window {
                    event_loop.exit();
                } else {
                    self.close_window(app_window);
                }
            }
            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = map_modifiers(mods.state());
            }
            WindowEvent::Resized(size) => {
                self.resize_surface(app_window, size.width, size.height);
                if let Some(state) = self.windows.get(app_window) {
                    state.window.request_redraw();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let pos = {
                    let Some(state) = self.windows.get_mut(app_window) else {
                        return;
                    };
                    let logical = position.to_logical::<f32>(state.window.scale_factor());
                    state.cursor_pos = Point::new(Px(logical.x), Px(logical.y));
                    state.cursor_pos
                };
                self.dispatch_pointer_event(
                    app_window,
                    fret_core::PointerEvent::Move { position: pos },
                );
                self.process_dock_requests(event_loop);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let Some(button) = map_mouse_button(button) else {
                    return;
                };
                let Some(pos) = self.windows.get(app_window).map(|s| s.cursor_pos) else {
                    return;
                };
                let pe = match state {
                    ElementState::Pressed => fret_core::PointerEvent::Down {
                        position: pos,
                        button,
                        modifiers: self.modifiers,
                    },
                    ElementState::Released => fret_core::PointerEvent::Up {
                        position: pos,
                        button,
                        modifiers: self.modifiers,
                    },
                };
                self.dispatch_pointer_event(app_window, pe);
                self.process_dock_requests(event_loop);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let Some(state) = self.windows.get(app_window) else {
                    return;
                };
                let delta = match delta {
                    MouseScrollDelta::LineDelta(x, y) => Point::new(Px(x * 20.0), Px(y * 20.0)),
                    MouseScrollDelta::PixelDelta(p) => {
                        let logical = p.to_logical::<f32>(state.window.scale_factor());
                        Point::new(Px(logical.x), Px(logical.y))
                    }
                };
                let pos = state.cursor_pos;
                self.dispatch_pointer_event(
                    app_window,
                    fret_core::PointerEvent::Wheel {
                        position: pos,
                        delta,
                        modifiers: self.modifiers,
                    },
                );
                self.process_dock_requests(event_loop);
            }
            WindowEvent::RedrawRequested => {
                let (Some(context), Some(renderer)) =
                    (self.context.as_ref(), self.renderer.as_mut())
                else {
                    return;
                };
                let (app, windows) = (&mut self.app, &mut self.windows);
                let Some(state) = windows.get_mut(app_window) else {
                    return;
                };

                let (frame, view) = match state.surface.get_current_frame_view() {
                    Ok(v) => v,
                    Err(wgpu::SurfaceError::Lost) => {
                        let size = state.window.inner_size();
                        self.resize_surface(app_window, size.width, size.height);
                        return;
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        event_loop.exit();
                        return;
                    }
                    Err(
                        wgpu::SurfaceError::Outdated
                        | wgpu::SurfaceError::Timeout
                        | wgpu::SurfaceError::Other,
                    ) => return,
                };

                let scale_factor = state.window.scale_factor() as f32;
                let physical = state.window.inner_size();
                let logical: winit::dpi::LogicalSize<f32> =
                    physical.to_logical(state.window.scale_factor());

                state.scene.clear();

                let bounds = Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(logical.width), Px(logical.height)),
                );
                let _ = state.ui.layout(app, state.root, bounds.size);
                state.ui.paint(app, state.root, bounds, &mut state.scene);

                let cmd = renderer.render_scene(
                    &context.device,
                    &context.queue,
                    state.surface.format(),
                    &view,
                    &state.scene,
                    ClearColor::default(),
                    scale_factor,
                    state.surface.size(),
                );

                context.queue.submit([cmd]);
                frame.present();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Wait);
        for (_, state) in self.windows.iter() {
            state.window.request_redraw();
        }
    }
}

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    let mut app = DemoApp::default();
    event_loop.run_app(&mut app)?;
    Ok(())
}

fn map_modifiers(state: ModifiersState) -> Modifiers {
    Modifiers {
        shift: state.shift_key(),
        ctrl: state.control_key(),
        alt: state.alt_key(),
        meta: state.super_key(),
    }
}

fn map_mouse_button(button: WinitMouseButton) -> Option<MouseButton> {
    Some(match button {
        WinitMouseButton::Left => MouseButton::Left,
        WinitMouseButton::Right => MouseButton::Right,
        WinitMouseButton::Middle => MouseButton::Middle,
        WinitMouseButton::Back => MouseButton::Back,
        WinitMouseButton::Forward => MouseButton::Forward,
        WinitMouseButton::Other(v) => MouseButton::Other(v),
    })
}

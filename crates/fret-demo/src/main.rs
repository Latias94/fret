use fret_app::App;
use fret_core::{
    AppWindowId, Axis, Color, DockNode, Modifiers, MouseButton, Point, Px, Rect, Scene, Size,
};
use fret_render::{ClearColor, Renderer, SurfaceState, WgpuContext};
use fret_ui::{DockManager, DockPanel, DockSpace, UiTree};
use slotmap::SlotMap;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event::{ElementState, MouseButton as WinitMouseButton, MouseScrollDelta},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::ModifiersState,
    window::Window,
};

#[derive(Default)]
struct DemoApp {
    window: Option<Arc<Window>>,
    context: Option<WgpuContext>,
    surface: Option<SurfaceState<'static>>,
    renderer: Option<Renderer>,

    app: App,
    ui: UiTree,
    root: Option<fret_core::NodeId>,
    scene: Scene,

    window_ids: SlotMap<AppWindowId, winit::window::WindowId>,
    main_window: Option<AppWindowId>,
    modifiers: Modifiers,
    cursor_pos: Point,
}

impl DemoApp {
    fn resize_surface(&mut self, width: u32, height: u32) {
        let Some(context) = self.context.as_ref() else {
            return;
        };
        let Some(surface) = self.surface.as_mut() else {
            return;
        };
        surface.resize(&context.device, width, height);
    }

    fn dispatch_pointer_event(&mut self, pe: fret_core::PointerEvent) {
        let Some(_root) = self.root else {
            return;
        };
        self.ui
            .dispatch_event(&mut self.app, &fret_core::Event::Pointer(pe));
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

impl ApplicationHandler for DemoApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
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

        let size = window.inner_size();
        let surface = SurfaceState::new(
            &context.adapter,
            &context.device,
            surface,
            size.width,
            size.height,
        )
        .expect("configure surface");
        let renderer = Renderer::new(&context.device);

        let mut ui = UiTree::new();
        let main_window = self.window_ids.insert(window.id());

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

        let root = ui.create_node(DockSpace::new(main_window));
        ui.set_root(root);

        self.window = Some(window);
        self.context = Some(context);
        self.surface = Some(surface);
        self.renderer = Some(renderer);
        self.ui = ui;
        self.root = Some(root);
        self.main_window = Some(main_window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let window = match self.window.as_ref() {
            Some(window) if window.id() == window_id => window.clone(),
            _ => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = map_modifiers(mods.state());
            }
            WindowEvent::Resized(size) => {
                self.resize_surface(size.width, size.height);
                window.request_redraw();
            }
            WindowEvent::CursorMoved { position, .. } => {
                let logical = position.to_logical::<f32>(window.scale_factor());
                self.cursor_pos = Point::new(Px(logical.x), Px(logical.y));
                self.dispatch_pointer_event(fret_core::PointerEvent::Move {
                    position: self.cursor_pos,
                });
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let Some(button) = map_mouse_button(button) else {
                    return;
                };
                let pe = match state {
                    ElementState::Pressed => fret_core::PointerEvent::Down {
                        position: self.cursor_pos,
                        button,
                        modifiers: self.modifiers,
                    },
                    ElementState::Released => fret_core::PointerEvent::Up {
                        position: self.cursor_pos,
                        button,
                        modifiers: self.modifiers,
                    },
                };
                self.dispatch_pointer_event(pe);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let delta = match delta {
                    MouseScrollDelta::LineDelta(x, y) => Point::new(Px(x * 20.0), Px(y * 20.0)),
                    MouseScrollDelta::PixelDelta(p) => {
                        let logical = p.to_logical::<f32>(window.scale_factor());
                        Point::new(Px(logical.x), Px(logical.y))
                    }
                };
                self.dispatch_pointer_event(fret_core::PointerEvent::Wheel {
                    position: self.cursor_pos,
                    delta,
                    modifiers: self.modifiers,
                });
            }
            WindowEvent::RedrawRequested => {
                let (Some(context), Some(renderer)) =
                    (self.context.as_ref(), self.renderer.as_mut())
                else {
                    return;
                };
                let Some(surface) = self.surface.as_ref() else {
                    return;
                };
                let Some(root) = self.root else {
                    return;
                };

                let (frame, view) = match surface.get_current_frame_view() {
                    Ok(v) => v,
                    Err(wgpu::SurfaceError::Lost) => {
                        let size = window.inner_size();
                        self.resize_surface(size.width, size.height);
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

                let scale_factor = window.scale_factor() as f32;
                let physical = window.inner_size();
                let logical: winit::dpi::LogicalSize<f32> =
                    physical.to_logical(window.scale_factor());

                self.scene.clear();

                let bounds = Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(logical.width), Px(logical.height)),
                );
                let _ = self.ui.layout(&mut self.app, root, bounds.size);
                self.ui.paint(&mut self.app, root, bounds, &mut self.scene);

                let cmd = renderer.render_scene(
                    &context.device,
                    &context.queue,
                    surface.format(),
                    &view,
                    &self.scene,
                    ClearColor::default(),
                    scale_factor,
                    surface.size(),
                );

                context.queue.submit([cmd]);
                frame.present();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Wait);
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
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

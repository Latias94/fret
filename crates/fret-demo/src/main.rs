use fret_app::{App, CreateWindowKind, CreateWindowRequest, Effect, WindowRequest};
use fret_core::{Axis, Color, DockNode, DropZone, Rect, Scene};
use fret_platform::winit_runner::{WindowCreateSpec, WinitDriver, WinitRunner, WinitRunnerConfig};
use fret_ui::{Column, DockManager, DockPanel, DockSpace, FixedPanel, Scroll, Split, UiTree};
use winit::event_loop::EventLoop;

struct DemoWindowState {
    ui: UiTree,
    root: fret_core::NodeId,
}

#[derive(Default)]
struct DemoDriver {
    main_window: Option<fret_core::AppWindowId>,
}

impl DemoDriver {
    fn ensure_main_tabs(dock: &mut DockManager, main: fret_core::AppWindowId) -> fret_core::DockNodeId {
        dock.graph.first_tabs_in_window(main).unwrap_or_else(|| {
            let tabs = dock.graph.insert_node(DockNode::Tabs {
                tabs: Vec::new(),
                active: 0,
            });
            dock.graph.set_window_root(main, tabs);
            tabs
        })
    }
}

impl WinitDriver for DemoDriver {
    type WindowState = DemoWindowState;

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
        app.set_global(dock);
    }

    fn create_window_state(
        &mut self,
        _app: &mut App,
        window: fret_core::AppWindowId,
    ) -> Self::WindowState {
        let mut ui = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(Split::new(Axis::Horizontal, 0.72));
        ui.set_root(root);

        let dock = ui.create_node(DockSpace::new(window));
        ui.add_child(root, dock);

        let scroll = ui.create_node(Scroll::new());
        ui.add_child(root, scroll);

        let column = ui.create_node(Column::new().with_padding(fret_core::Px(10.0)).with_spacing(fret_core::Px(8.0)));
        ui.add_child(scroll, column);

        for i in 0..28 {
            let shade = 0.14 + (i % 2) as f32 * 0.02;
            let height = if i % 7 == 0 { fret_core::Px(72.0) } else { fret_core::Px(44.0) };
            let item = ui.create_node(FixedPanel::new(
                height,
                Color {
                    r: shade,
                    g: shade + 0.01,
                    b: shade + 0.02,
                    a: 1.0,
                },
            ));
            ui.add_child(column, item);
        }

        Self::WindowState { ui, root }
    }

    fn handle_event(
        &mut self,
        app: &mut App,
        _window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        event: &fret_core::Event,
    ) {
        state.ui.dispatch_event(app, event);
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
        let _ = state.ui.layout_in(app, state.root, bounds);
        state.ui.paint(app, state.root, bounds, scene);
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
            CreateWindowKind::DockFloating { source_window, panel } => {
                let empty = {
                    let Some(dock) = app.global_mut::<DockManager>() else {
                        return;
                    };
                    dock.graph.float_panel_to_window(source_window, panel, new_window);
                    dock.graph.collect_panels_in_window(source_window).is_empty()
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

use anyhow::Context as _;
use fret_app::{App, CommandId, Effect, Model, WindowRequest};
use fret_components_app::tree::AppTreeRowRenderer;
use fret_components_icons::IconRegistry;
use fret_components_shadcn as shadcn;
use fret_components_ui::tree::{TreeItem, TreeItemId, TreeState};
use fret_core::{
    AppWindowId, Edges, Event, KeyCode, PlatformCapabilities, Px, Rect, Scene, UiServices,
};
use fret_runner_winit_wgpu::{WindowCreateSpec, WinitDriver, WinitRunner, WinitRunnerConfig};
use fret_ui::declarative;
use fret_ui::element::{ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign};
use fret_ui::{Invalidation, Theme, UiTree};
use std::sync::Arc;
use winit::event_loop::EventLoop;

struct ComponentsGalleryWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    items: Model<Vec<TreeItem>>,
    tree_state: Model<TreeState>,
    progress: Model<f32>,
    checkbox: Model<bool>,
    switch: Model<bool>,
    radio: Model<Option<Arc<str>>>,
    select: Model<Option<Arc<str>>>,
    select_open: Model<bool>,
}

#[derive(Default)]
struct ComponentsGalleryDriver;

impl ComponentsGalleryDriver {
    fn sample_tree_items() -> Vec<TreeItem> {
        vec![
            TreeItem::new(1, "src")
                .child(TreeItem::new(2, "components").child(TreeItem::new(3, "tree.rs")))
                .child(TreeItem::new(4, "theme.rs"))
                .child(TreeItem::new(5, "virtual_list.rs")),
            TreeItem::new(10, "crates")
                .child(TreeItem::new(11, "fret-ui"))
                .child(TreeItem::new(12, "fret-components-ui"))
                .child(TreeItem::new(13, "fret-demo").disabled(true)),
            TreeItem::new(20, "docs").child(TreeItem::new(21, "adr")),
        ]
    }

    fn build_ui(app: &mut App, window: AppWindowId) -> ComponentsGalleryWindowState {
        let items = app.models_mut().insert(Self::sample_tree_items());

        let initial_state = TreeState {
            selected: Some(1),
            expanded: [1, 10, 20].into_iter().collect(),
        };
        let tree_state = app.models_mut().insert(initial_state);
        let progress = app.models_mut().insert(35.0f32);
        let checkbox = app.models_mut().insert(false);
        let switch = app.models_mut().insert(true);
        let radio = app
            .models_mut()
            .insert(Option::<Arc<str>>::Some(Arc::from("a")));
        let select = app
            .models_mut()
            .insert(Option::<Arc<str>>::Some(Arc::from("apple")));
        let select_open = app.models_mut().insert(false);

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        ComponentsGalleryWindowState {
            ui,
            root: None,
            items,
            tree_state,
            progress,
            checkbox,
            switch,
            radio,
            select,
            select_open,
        }
    }

    fn render_gallery(
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        state: &mut ComponentsGalleryWindowState,
        bounds: Rect,
    ) {
        fret_components_ui::window_overlays::begin_frame(app, window);

        let items = state.items;
        let tree_state = state.tree_state;
        let progress = state.progress;
        let checkbox = state.checkbox;
        let switch = state.switch;
        let radio = state.radio;
        let select = state.select;
        let select_open = state.select_open;

        let root = declarative::render_root(
            &mut state.ui,
            app,
            services,
            window,
            bounds,
            "components-gallery",
            |cx| {
                let theme = Theme::global(&*cx.app);
                let selected = cx.app.models().get(tree_state).and_then(|s| s.selected);

                let title: Arc<str> = Arc::from("components_gallery");
                let subtitle: Arc<str> = Arc::from(format!(
                    "Tree MVP (driver-owned): arrows navigate, left/right collapses, click selects. Selected: {}",
                    selected
                        .map(|id| id.to_string())
                        .unwrap_or_else(|| "<none>".to_string())
                ));

                let mut root_layout = LayoutStyle::default();
                root_layout.size.width = Length::Fill;
                root_layout.size.height = Length::Fill;

                let mut tree_slot_layout = LayoutStyle::default();
                tree_slot_layout.size.width = Length::Fill;
                tree_slot_layout.size.height = Length::Fill;
                tree_slot_layout.flex.grow = 1.0;
                tree_slot_layout.flex.basis = Length::Px(Px(0.0));

                let padding = theme.metrics.padding_md;
                let bg = theme.colors.surface_background;

                vec![cx.container(
                    ContainerProps {
                        layout: root_layout,
                        background: Some(bg),
                        ..Default::default()
                    },
                    |cx| {
                        vec![cx.flex(
                            FlexProps {
                                layout: root_layout,
                                direction: fret_core::Axis::Vertical,
                                gap: Px(12.0),
                                padding: Edges::all(padding),
                                justify: MainAlign::Start,
                                align: CrossAlign::Stretch,
                                wrap: false,
                            },
                            |cx| {
                                let mut renderer = AppTreeRowRenderer;
                                vec![
                                    cx.text(title),
                                    cx.text(subtitle),
                                    cx.flex(
                                        FlexProps {
                                            layout: LayoutStyle::default(),
                                            direction: fret_core::Axis::Horizontal,
                                            gap: Px(8.0),
                                            padding: Edges::all(Px(0.0)),
                                            justify: MainAlign::Start,
                                            align: CrossAlign::Center,
                                            wrap: true,
                                        },
                                        |cx| {
                                            vec![
                                                shadcn::Button::new("Primary")
                                                    .on_click(CommandId::from("gallery.progress.inc"))
                                                    .into_element(cx),
                                                shadcn::Button::new("Destructive")
                                                    .variant(shadcn::ButtonVariant::Destructive)
                                                    .on_click(CommandId::from("gallery.progress.dec"))
                                                    .into_element(cx),
                                                shadcn::Button::new("Outline")
                                                    .variant(shadcn::ButtonVariant::Outline)
                                                    .on_click(CommandId::from("gallery.progress.reset"))
                                                    .into_element(cx),
                                                shadcn::Button::new("Disabled")
                                                    .disabled(true)
                                                    .into_element(cx),
                                            ]
                                        },
                                    ),
                                    shadcn::Progress::new(progress)
                                        .range(0.0, 100.0)
                                        .into_element(cx),
                                    cx.flex(
                                        FlexProps {
                                            layout: LayoutStyle::default(),
                                            direction: fret_core::Axis::Horizontal,
                                            gap: Px(12.0),
                                            padding: Edges::all(Px(0.0)),
                                            justify: MainAlign::Start,
                                            align: CrossAlign::Center,
                                            wrap: true,
                                        },
                                        |cx| {
                                            let checkbox_value =
                                                cx.app.models().get(checkbox).copied().unwrap_or(false);
                                            let switch_value =
                                                cx.app.models().get(switch).copied().unwrap_or(false);

                                            vec![
                                                shadcn::Checkbox::new(checkbox)
                                                    .a11y_label("Demo checkbox")
                                                    .into_element(cx),
                                                cx.text(format!("checkbox: {checkbox_value}")),
                                                shadcn::Switch::new(switch)
                                                    .a11y_label("Demo switch")
                                                    .into_element(cx),
                                                cx.text(format!("switch: {switch_value}")),
                                            ]
                                        },
                                    ),
                                    cx.flex(
                                        FlexProps {
                                            layout: LayoutStyle::default(),
                                            direction: fret_core::Axis::Vertical,
                                            gap: Px(8.0),
                                            padding: Edges::all(Px(0.0)),
                                            justify: MainAlign::Start,
                                            align: CrossAlign::Stretch,
                                            wrap: false,
                                        },
                                        |cx| {
                                            let value = cx
                                                .app
                                                .models()
                                                .get(radio)
                                                .and_then(|v| v.as_deref())
                                                .unwrap_or("<none>");

                                            vec![
                                                cx.text(format!("radio: {value}")),
                                                shadcn::RadioGroup::new(radio)
                                                    .a11y_label("Demo radio group")
                                                    .item(shadcn::RadioGroupItem::new("a", "Alpha"))
                                                    .item(shadcn::RadioGroupItem::new("b", "Beta"))
                                                    .item(
                                                        shadcn::RadioGroupItem::new("c", "Disabled")
                                                            .disabled(true),
                                                    )
                                                    .into_element(cx),
                                            ]
                                        },
                                    ),
                                    cx.flex(
                                        FlexProps {
                                            layout: LayoutStyle::default(),
                                            direction: fret_core::Axis::Vertical,
                                            gap: Px(8.0),
                                            padding: Edges::all(Px(0.0)),
                                            justify: MainAlign::Start,
                                            align: CrossAlign::Stretch,
                                        wrap: false,
                                    },
                                    |cx| {
                                        let value = cx
                                            .app
                                            .models()
                                            .get(select)
                                            .cloned()
                                            .flatten()
                                            .as_deref()
                                            .unwrap_or("<none>")
                                            .to_owned();

                                        vec![
                                            shadcn::Select::new(select, select_open)
                                                .a11y_label("Demo select")
                                                    .placeholder("Pick a fruit")
                                                .items([
                                                    shadcn::SelectItem::new("apple", "Apple"),
                                                    shadcn::SelectItem::new("banana", "Banana"),
                                                    shadcn::SelectItem::new("cherry", "Cherry"),
                                                ])
                                                .into_element(cx),
                                            cx.text(format!("select: {value}")),
                                        ]
                                    },
                                ),
                                    cx.container(
                                        ContainerProps {
                                            layout: tree_slot_layout,
                                            ..Default::default()
                                        },
                                        |cx| {
                                            vec![fret_components_ui::declarative::tree::tree_view_with_renderer(
                                                cx,
                                                items,
                                                tree_state,
                                                fret_components_ui::Size::Medium,
                                                &mut renderer,
                                            )]
                                        },
                                    ),
                                ]
                            },
                        )]
                    },
                )]
            },
        );

        state.ui.set_root(root);
        fret_components_ui::window_overlays::render(&mut state.ui, app, services, window, bounds);
        state.root = Some(root);
    }

    fn handle_tree_command(
        app: &mut App,
        items: Model<Vec<TreeItem>>,
        state: Model<TreeState>,
        command: &CommandId,
    ) -> bool {
        if let Some(id) = command.as_str().strip_prefix("tree.select.") {
            let Ok(id) = id.parse::<TreeItemId>() else {
                return true;
            };
            let _ = app.models_mut().update(state, |s| s.selected = Some(id));
            return true;
        }

        if let Some(id) = command.as_str().strip_prefix("tree.toggle.") {
            let Ok(id) = id.parse::<TreeItemId>() else {
                return true;
            };
            let _ = app.models_mut().update(state, |s| {
                if !s.expanded.insert(id) {
                    s.expanded.remove(&id);
                }
            });
            return true;
        }

        let _ = items;
        false
    }

    fn handle_tree_key_event(
        app: &mut App,
        items: Model<Vec<TreeItem>>,
        state: Model<TreeState>,
        event: &Event,
    ) -> bool {
        let Event::KeyDown {
            key, repeat: false, ..
        } = event
        else {
            return false;
        };

        let items_value = app.models().get(items).cloned().unwrap_or_default();
        let tree_state_value = app.models().get(state).cloned().unwrap_or_default();
        let entries = fret_components_ui::flatten_tree(&items_value, &tree_state_value.expanded);
        if entries.is_empty() {
            return false;
        }

        let selected_id = tree_state_value.selected;
        let selected_index = selected_id
            .and_then(|id| entries.iter().position(|e| e.id == id))
            .unwrap_or(0);

        match key {
            KeyCode::ArrowUp => {
                let next = selected_index.saturating_sub(1);
                let id = entries[next].id;
                let _ = app.models_mut().update(state, |s| s.selected = Some(id));
                true
            }
            KeyCode::ArrowDown => {
                let next = (selected_index + 1).min(entries.len().saturating_sub(1));
                let id = entries[next].id;
                let _ = app.models_mut().update(state, |s| s.selected = Some(id));
                true
            }
            KeyCode::ArrowLeft => {
                let Some(cur) = entries.get(selected_index).cloned() else {
                    return true;
                };
                if tree_state_value.expanded.contains(&cur.id) {
                    let _ = app.models_mut().update(state, |s| {
                        s.expanded.remove(&cur.id);
                    });
                    return true;
                }
                if let Some(parent) = cur.parent {
                    let _ = app
                        .models_mut()
                        .update(state, |s| s.selected = Some(parent));
                    return true;
                }
                true
            }
            KeyCode::ArrowRight => {
                let Some(cur) = entries.get(selected_index).cloned() else {
                    return true;
                };
                if cur.has_children && !tree_state_value.expanded.contains(&cur.id) {
                    let _ = app.models_mut().update(state, |s| {
                        s.expanded.insert(cur.id);
                    });
                    return true;
                }
                if cur.has_children {
                    if let Some(next) = entries.get(selected_index + 1)
                        && next.depth > cur.depth
                    {
                        let _ = app
                            .models_mut()
                            .update(state, |s| s.selected = Some(next.id));
                    }
                    return true;
                }
                true
            }
            KeyCode::Home => {
                let id = entries[0].id;
                let _ = app.models_mut().update(state, |s| s.selected = Some(id));
                true
            }
            KeyCode::End => {
                let id = entries[entries.len().saturating_sub(1)].id;
                let _ = app.models_mut().update(state, |s| s.selected = Some(id));
                true
            }
            _ => false,
        }
    }
}

impl WinitDriver for ComponentsGalleryDriver {
    type WindowState = ComponentsGalleryWindowState;

    fn init(&mut self, _app: &mut App, _main_window: AppWindowId) {}

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        Self::build_ui(app, window)
    }

    fn handle_model_changes(
        &mut self,
        app: &mut App,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        changed: &[fret_app::ModelId],
    ) {
        state.ui.propagate_model_changes(app, changed);
    }

    fn handle_command(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        state: &mut Self::WindowState,
        command: CommandId,
    ) {
        if state.ui.dispatch_command(app, services, &command) {
            return;
        }

        if ComponentsGalleryDriver::handle_tree_command(
            app,
            state.items,
            state.tree_state,
            &command,
        ) {
            app.request_redraw(window);
            return;
        }

        if let Some(id) = command.as_str().strip_prefix("gallery.tree.action.") {
            tracing::info!(%id, "gallery tree row action");
            return;
        }

        if let Some(id) = command.as_str().strip_prefix("app.tree.action.") {
            tracing::info!(%id, "app tree row action");
            return;
        }

        if command.as_str() == "gallery.close" {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
            return;
        }

        if command.as_str() == "gallery.progress.inc" {
            let _ = app
                .models_mut()
                .update(state.progress, |v| *v = (*v + 10.0).min(100.0));
            app.request_redraw(window);
        }

        if command.as_str() == "gallery.progress.dec" {
            let _ = app
                .models_mut()
                .update(state.progress, |v| *v = (*v - 10.0).max(0.0));
            app.request_redraw(window);
        }

        if command.as_str() == "gallery.progress.reset" {
            let _ = app.models_mut().update(state.progress, |v| *v = 35.0);
            app.request_redraw(window);
        }
    }

    fn handle_event(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        state: &mut Self::WindowState,
        event: &Event,
    ) {
        if let Event::KeyDown {
            key: KeyCode::Escape,
            repeat: false,
            ..
        } = event
        {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
            return;
        }

        if matches!(event, Event::WindowCloseRequested) {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
            return;
        }

        if ComponentsGalleryDriver::handle_tree_key_event(app, state.items, state.tree_state, event)
        {
            app.request_redraw(window);
            return;
        }

        state.ui.dispatch_event(app, services, event);
    }

    fn render(
        &mut self,
        app: &mut App,
        window: AppWindowId,
        state: &mut Self::WindowState,
        bounds: Rect,
        scale_factor: f32,
        services: &mut dyn UiServices,
        scene: &mut Scene,
    ) {
        ComponentsGalleryDriver::render_gallery(app, services, window, state, bounds);

        state.ui.request_semantics_snapshot();
        state.ui.ingest_paint_cache_source(scene);
        scene.clear();
        state.ui.layout_all(app, services, bounds, scale_factor);
        state
            .ui
            .paint_all(app, services, bounds, scene, scale_factor);
    }

    fn invalidate_ui_layout(
        &mut self,
        _app: &mut App,
        _window: AppWindowId,
        state: &mut Self::WindowState,
    ) {
        if let Some(root) = state.root {
            state.ui.invalidate(root, Invalidation::Layout);
        }
    }

    fn window_create_spec(
        &mut self,
        _app: &mut App,
        _request: &fret_app::CreateWindowRequest,
    ) -> Option<WindowCreateSpec> {
        None
    }

    fn window_created(
        &mut self,
        _app: &mut App,
        _request: &fret_app::CreateWindowRequest,
        _new_window: AppWindowId,
    ) {
    }

    fn accessibility_snapshot(
        &mut self,
        _app: &mut App,
        _window: AppWindowId,
        state: &mut Self::WindowState,
    ) -> Option<Arc<fret_core::SemanticsSnapshot>> {
        state.ui.semantics_snapshot_arc()
    }

    fn accessibility_focus(
        &mut self,
        _app: &mut App,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
    ) {
        state.ui.set_focus(Some(target));
    }

    fn accessibility_invoke(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
    ) {
        fret_ui_app::accessibility_actions::invoke(&mut state.ui, app, services, target);
    }

    fn accessibility_set_value_text(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
        value: &str,
    ) {
        fret_ui_app::accessibility_actions::set_value_text(
            &mut state.ui,
            app,
            services,
            target,
            value,
        );
    }

    fn accessibility_set_value_numeric(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
        value: f64,
    ) {
        fret_ui_app::accessibility_actions::set_value_numeric(
            &mut state.ui,
            app,
            services,
            target,
            value,
        );
    }
}

pub fn run() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_render=info".parse().unwrap())
                .add_directive("fret_runner_winit_wgpu=info".parse().unwrap()),
        )
        .try_init();

    let event_loop = EventLoop::new().context("create winit event loop")?;
    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());
    app.with_global_mut(IconRegistry::default, |icons, _app| {
        fret_icons_lucide::register_icons(icons);
    });

    let mut config = WinitRunnerConfig {
        main_window_title: "fret-demo components_gallery".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(980.0, 720.0),
        ..Default::default()
    };

    if let Some(settings) = fret_app::SettingsFileV1::load_json_if_exists(".fret/settings.json")
        .context("load .fret/settings.json")?
    {
        config.text_font_families.ui_sans = settings.fonts.ui_sans;
        config.text_font_families.ui_serif = settings.fonts.ui_serif;
        config.text_font_families.ui_mono = settings.fonts.ui_mono;
    }

    let driver = ComponentsGalleryDriver;
    let mut runner = WinitRunner::new(config, app, driver);
    event_loop.run_app(&mut runner)?;
    Ok(())
}

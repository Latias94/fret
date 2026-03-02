use std::sync::Arc;

use fret::prelude::*;
use fret_app::{CommandMeta, CommandScope};
use fret_bootstrap::ui_app_driver::ViewElements;
use fret_core::DockOp;
use fret_core::dock::{Axis, DockNode};
use fret_core::{AppWindowId, Color, DockNodeId, PanelKey, Px, SemanticsRole};
use fret_docking::{
    DockManager, DockPanel, DockPanelRegistry, DockPanelRegistryService, DockingPolicy,
    DockingPolicyService, create_dock_space_node_with_test_id, render_and_bind_dock_panels,
    render_cached_panel_root,
};
use fret_runtime::CommandId;
use fret_ui::element::{LayoutStyle, Length, SemanticsDecoration, SemanticsProps};
use fret_ui::retained_bridge::{LayoutCx, PaintCx, SemanticsCx, UiTreeRetainedExt as _, Widget};
use fret_ui::{ElementContext, UiHost, UiTree};

const ROOT_NAME: &str = "cookbook-docking-basics";

const TEST_ID_ROOT: &str = "cookbook.docking_basics.root";
const TEST_ID_DOCK_SPACE: &str = "cookbook.docking_basics.dock_space";
const TEST_ID_RESET_LAYOUT: &str = "cookbook.docking_basics.reset_layout";
const TEST_ID_ACTIVATE_HIERARCHY: &str = "cookbook.docking_basics.activate_hierarchy";
const TEST_ID_ACTIVATE_INSPECTOR: &str = "cookbook.docking_basics.activate_inspector";
const TEST_ID_ACTIVATE_EDITOR: &str = "cookbook.docking_basics.activate_editor";
const TEST_ID_ACTIVATE_CONSOLE: &str = "cookbook.docking_basics.activate_console";
const TEST_ID_ACTIVE_LEFT: &str = "cookbook.docking_basics.active_left";
const TEST_ID_ACTIVE_RIGHT: &str = "cookbook.docking_basics.active_right";

const CMD_RESET_LAYOUT: &str = "cookbook.docking.reset_layout";
const CMD_ACTIVATE_HIERARCHY: &str = "cookbook.docking.activate_hierarchy";
const CMD_ACTIVATE_INSPECTOR: &str = "cookbook.docking.activate_inspector";
const CMD_ACTIVATE_EDITOR: &str = "cookbook.docking.activate_editor";
const CMD_ACTIVATE_CONSOLE: &str = "cookbook.docking.activate_console";

fn install_commands(app: &mut App) {
    let scope = CommandScope::Widget;

    app.commands_mut().register(
        CommandId::from(CMD_RESET_LAYOUT),
        CommandMeta::new("Reset dock layout")
            .with_description("Reset the window dock layout to the default split + tabs.")
            .with_category("Docking")
            .with_scope(scope),
    );

    app.commands_mut().register(
        CommandId::from(CMD_ACTIVATE_HIERARCHY),
        CommandMeta::new("Activate Hierarchy panel")
            .with_description("Activate the Hierarchy tab (best-effort).")
            .with_category("Docking")
            .with_scope(scope),
    );

    app.commands_mut().register(
        CommandId::from(CMD_ACTIVATE_INSPECTOR),
        CommandMeta::new("Activate Inspector panel")
            .with_description("Activate the Inspector tab (best-effort).")
            .with_category("Docking")
            .with_scope(scope),
    );

    app.commands_mut().register(
        CommandId::from(CMD_ACTIVATE_EDITOR),
        CommandMeta::new("Activate Editor panel")
            .with_description("Activate the Editor tab (best-effort).")
            .with_category("Docking")
            .with_scope(scope),
    );

    app.commands_mut().register(
        CommandId::from(CMD_ACTIVATE_CONSOLE),
        CommandMeta::new("Activate Console panel")
            .with_description("Activate the Console tab (best-effort).")
            .with_category("Docking")
            .with_scope(scope),
    );
}

fn panel_hierarchy() -> PanelKey {
    PanelKey::new("core.hierarchy")
}

fn panel_inspector() -> PanelKey {
    PanelKey::new("core.inspector")
}

fn panel_editor() -> PanelKey {
    PanelKey::new("core.editor")
}

fn panel_console() -> PanelKey {
    PanelKey::new("core.console")
}

#[derive(Debug, Clone, Copy)]
struct DockLayoutIds {
    left_tabs: DockNodeId,
    right_tabs: DockNodeId,
}

fn reset_dock_layout(dock: &mut DockManager, window: AppWindowId) -> DockLayoutIds {
    dock.graph = fret_core::DockGraph::new();
    dock.graph.remove_window_root(window);
    dock.graph.floating_windows_mut(window).clear();

    let hierarchy = panel_hierarchy();
    let inspector = panel_inspector();
    let editor = panel_editor();
    let console = panel_console();

    dock.ensure_panel(&hierarchy, || DockPanel {
        title: "Hierarchy".to_string(),
        color: Color::from_srgb_hex_rgb(0x3B82F6),
        viewport: None,
    });
    dock.ensure_panel(&inspector, || DockPanel {
        title: "Inspector".to_string(),
        color: Color::from_srgb_hex_rgb(0xA855F7),
        viewport: None,
    });
    dock.ensure_panel(&editor, || DockPanel {
        title: "Editor".to_string(),
        color: Color::from_srgb_hex_rgb(0x22C55E),
        viewport: None,
    });
    dock.ensure_panel(&console, || DockPanel {
        title: "Console".to_string(),
        color: Color::from_srgb_hex_rgb(0xF97316),
        viewport: None,
    });

    let left_tabs = dock.graph.insert_node(DockNode::Tabs {
        tabs: vec![hierarchy, inspector],
        active: 0,
    });
    let right_tabs = dock.graph.insert_node(DockNode::Tabs {
        tabs: vec![editor, console],
        active: 0,
    });
    let root = dock.graph.insert_node(DockNode::Split {
        axis: Axis::Horizontal,
        children: vec![left_tabs, right_tabs],
        fractions: vec![0.3, 0.7],
    });
    dock.graph.set_window_root(window, root);

    DockLayoutIds {
        left_tabs,
        right_tabs,
    }
}

struct DockingBasicsPolicy;

impl DockingPolicy for DockingBasicsPolicy {
    fn allow_tear_off(
        &self,
        _source_window: AppWindowId,
        _panel: &PanelKey,
        _info: Option<&DockPanel>,
    ) -> bool {
        false
    }
}

struct DockingBasicsPanelRegistry;

impl DockPanelRegistry<App> for DockingBasicsPanelRegistry {
    fn render_panel(
        &self,
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: fret_core::Rect,
        panel: &PanelKey,
    ) -> Option<fret_core::NodeId> {
        let title = match panel.kind.0.as_str() {
            "core.hierarchy" => "Hierarchy",
            "core.inspector" => "Inspector",
            "core.editor" => "Editor",
            "core.console" => "Console",
            _ => "Panel",
        };

        let root_name = format!("cookbook.docking.panel.{}", panel.kind.0);
        Some(render_cached_panel_root(
            ui,
            app,
            services,
            window,
            bounds,
            &root_name,
            |cx| {
                let body = shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new(title).into_element(cx),
                        shadcn::CardDescription::new(
                            "Dock content is app-owned (registry-driven), while docking UI/policy lives in fret-docking.",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![cx.text(
                        "Try: click tabs, drag tabs, drag the splitter, right-click a tab.",
                    )])
                    .into_element(cx),
                ])
                .ui()
                .w_full()
                .h_full()
                .into_element(cx);

                vec![body]
            },
        ))
    }
}

struct DockingBasicsDockHostRoot {
    window: AppWindowId,
    dock_space: fret_core::NodeId,
}

impl DockingBasicsDockHostRoot {
    fn new(window: AppWindowId, dock_space: fret_core::NodeId) -> Self {
        Self { window, dock_space }
    }
}

impl<H: UiHost + 'static> Widget<H> for DockingBasicsDockHostRoot {
    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Group);
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> fret_core::Size {
        if cx.pass_kind != fret_ui::layout_pass::LayoutPassKind::Probe {
            render_and_bind_dock_panels(
                cx.tree,
                cx.app,
                cx.services,
                self.window,
                cx.bounds,
                self.dock_space,
            );
        }

        let _ = cx.layout_in(self.dock_space, cx.bounds);
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        if let Some(bounds) = cx.child_bounds(self.dock_space) {
            cx.paint(self.dock_space, bounds);
        } else {
            cx.paint(self.dock_space, cx.bounds);
        }
    }
}

#[derive(Debug)]
struct DockingBasicsWindowState {
    window: AppWindowId,
    layout_ids: DockLayoutIds,
}

fn install_docking_services(app: &mut App) {
    app.with_global_mut(DockPanelRegistryService::<App>::default, |svc, _app| {
        svc.set(Arc::new(DockingBasicsPanelRegistry));
    });

    app.with_global_mut(DockingPolicyService::default, |svc, _app| {
        svc.set(Arc::new(DockingBasicsPolicy));
    });
}

fn init_window(app: &mut App, window: AppWindowId) -> DockingBasicsWindowState {
    let layout_ids = app.with_global_mut(DockManager::default, |dock, _app| {
        reset_dock_layout(dock, window)
    });

    DockingBasicsWindowState { window, layout_ids }
}

fn active_tab_title(app: &App, tabs: DockNodeId) -> Option<String> {
    let dock = app.global::<DockManager>()?;
    let DockNode::Tabs { tabs, active } = dock.graph.node(tabs)? else {
        return None;
    };
    let panel = tabs.get(*active)?;
    dock.panel(panel).map(|p| p.title.clone())
}

fn active_tab_state(app: &App, tabs: DockNodeId) -> Option<(u32, u32)> {
    let dock = app.global::<DockManager>()?;
    let DockNode::Tabs { tabs, active } = dock.graph.node(tabs)? else {
        return None;
    };

    Some((*active as u32, tabs.len() as u32))
}

fn view(cx: &mut ElementContext<'_, App>, st: &mut DockingBasicsWindowState) -> ViewElements {
    let active_left = active_tab_title(cx.app, st.layout_ids.left_tabs).unwrap_or("Unknown".into());
    let active_right =
        active_tab_title(cx.app, st.layout_ids.right_tabs).unwrap_or("Unknown".into());

    let (active_left_index, left_count) =
        active_tab_state(cx.app, st.layout_ids.left_tabs).unwrap_or((0, 0));
    let (active_right_index, right_count) =
        active_tab_state(cx.app, st.layout_ids.right_tabs).unwrap_or((0, 0));

    let header = shadcn::CardHeader::new(vec![
        shadcn::CardTitle::new("Docking basics").into_element(cx),
        shadcn::CardDescription::new(
            "Minimal retained dock host + app-owned panel registry + runner dock_op wiring.",
        )
        .into_element(cx),
    ])
    .into_element(cx);

    let toolbar = ui::h_flex(cx, |cx| {
        let left_max = (left_count.saturating_sub(1)) as f64;
        let right_max = (right_count.saturating_sub(1)) as f64;

        let active_left_badge = shadcn::Badge::new(format!("Left: {active_left}"))
            .variant(shadcn::BadgeVariant::Secondary)
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Generic)
                    .test_id(TEST_ID_ACTIVE_LEFT)
                    .numeric_value(active_left_index as f64)
                    .numeric_range(0.0, left_max),
            );

        let active_right_badge = shadcn::Badge::new(format!("Right: {active_right}"))
            .variant(shadcn::BadgeVariant::Secondary)
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Generic)
                    .test_id(TEST_ID_ACTIVE_RIGHT)
                    .numeric_value(active_right_index as f64)
                    .numeric_range(0.0, right_max),
            );

        vec![
            shadcn::Button::new("Reset layout")
                .variant(shadcn::ButtonVariant::Outline)
                .on_click(CMD_RESET_LAYOUT)
                .test_id(TEST_ID_RESET_LAYOUT)
                .into_element(cx),
            shadcn::Button::new("Activate Hierarchy")
                .variant(shadcn::ButtonVariant::Secondary)
                .on_click(CMD_ACTIVATE_HIERARCHY)
                .test_id(TEST_ID_ACTIVATE_HIERARCHY)
                .into_element(cx),
            shadcn::Button::new("Activate Inspector")
                .variant(shadcn::ButtonVariant::Secondary)
                .on_click(CMD_ACTIVATE_INSPECTOR)
                .test_id(TEST_ID_ACTIVATE_INSPECTOR)
                .into_element(cx),
            shadcn::Button::new("Activate Editor")
                .variant(shadcn::ButtonVariant::Secondary)
                .on_click(CMD_ACTIVATE_EDITOR)
                .test_id(TEST_ID_ACTIVATE_EDITOR)
                .into_element(cx),
            shadcn::Button::new("Activate Console")
                .variant(shadcn::ButtonVariant::Secondary)
                .on_click(CMD_ACTIVATE_CONSOLE)
                .test_id(TEST_ID_ACTIVATE_CONSOLE)
                .into_element(cx),
            active_left_badge,
            active_right_badge,
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx);

    let dock_host =
        cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
            let window = st.window;

            let mut layout = LayoutStyle::default();
            layout.size.width = Length::Fill;
            layout.size.height = Length::Fill;

            let props = fret_ui::retained_bridge::RetainedSubtreeProps::new::<App>(move |ui| {
                let dock_space =
                    create_dock_space_node_with_test_id(ui, window, TEST_ID_DOCK_SPACE);
                ui.create_node_retained(DockingBasicsDockHostRoot::new(window, dock_space))
            })
            .with_layout(layout);

            vec![cx.retained_subtree(props)]
        });

    let content = ui::v_flex(cx, |_cx| vec![toolbar, dock_host])
        .gap(Space::N3)
        .w_full()
        .h_full()
        .min_w_0()
        .into_element(cx);

    let card = shadcn::Card::new(vec![
        header,
        shadcn::CardContent::new(vec![content]).into_element(cx),
    ])
    .ui()
    .w_full()
    .h_full()
    .max_w(Px(1100.0))
    .into_element(cx);

    let root = fret_cookbook::scaffold::centered_page_muted(cx, TEST_ID_ROOT, card);

    vec![cx.semantics(
        SemanticsProps {
            role: SemanticsRole::Group,
            test_id: None,
            ..Default::default()
        },
        |_cx| vec![root],
    )]
    .into()
}

fn on_command(
    app: &mut App,
    _services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    _ui: &mut UiTree<App>,
    st: &mut DockingBasicsWindowState,
    command: &CommandId,
) {
    let cmd = command.as_str();

    if cmd == CMD_RESET_LAYOUT {
        let ids = app.with_global_mut(DockManager::default, |dock, _app| {
            reset_dock_layout(dock, window)
        });
        st.layout_ids = ids;
        fret_docking::runtime::request_dock_invalidation(app, [window]);
        return;
    }

    let panel = match cmd {
        CMD_ACTIVATE_HIERARCHY => Some(panel_hierarchy()),
        CMD_ACTIVATE_INSPECTOR => Some(panel_inspector()),
        CMD_ACTIVATE_EDITOR => Some(panel_editor()),
        CMD_ACTIVATE_CONSOLE => Some(panel_console()),
        _ => None,
    };

    let Some(panel) = panel else {
        return;
    };

    let Some((_, op)) = app
        .global::<DockManager>()
        .and_then(|dock| dock.activate_panel_tab_best_effort([window], &panel))
    else {
        return;
    };

    // Apply directly (not via Effect) to keep this example self-contained.
    let _ = fret_docking::handle_dock_op(app, op);
    fret_docking::runtime::request_dock_invalidation(app, [window]);
}

fn on_dock_op(app: &mut App, op: DockOp) {
    // DockSpace emits Effect::Dock(op); the runner routes it here.
    //
    // Notes:
    // - `handle_dock_op` applies pure graph ops and translates tear-off requests into window
    //   create requests.
    // - This cookbook example installs a policy that disables tear-off.
    let _ = fret_docking::handle_dock_op(app, op);
}

fn configure_driver(
    driver: fret_bootstrap::ui_app_driver::UiAppDriver<DockingBasicsWindowState>,
) -> fret_bootstrap::ui_app_driver::UiAppDriver<DockingBasicsWindowState> {
    driver.on_command(on_command).dock_op(on_dock_op)
}

fn main() -> anyhow::Result<()> {
    let builder = fret_bootstrap::ui_app_with_hooks(ROOT_NAME, init_window, view, configure_driver)
        .with_main_window("cookbook-docking-basics", (1120.0, 820.0))
        .with_command_default_keybindings()
        .install_app(install_commands)
        .install_app(install_docking_services)
        .install_app(shadcn::install_app)
        .install_app(fret_cookbook::install_cookbook_defaults)
        .with_ui_assets_budgets(64 * 1024 * 1024, 4096, 16 * 1024 * 1024, 4096)
        .with_lucide_icons()
        .with_default_diagnostics();

    builder.run().map_err(anyhow::Error::from)
}

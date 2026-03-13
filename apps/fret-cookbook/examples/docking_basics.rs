use std::sync::Arc;

use fret::docking::core::{Axis, DockNode, DockNodeId, DockOp, PanelKey};
use fret::shadcn::raw::prelude::{CachedSubtreeExt, CachedSubtreeProps};
use fret::{
    advanced::prelude::*,
    docking::{
        self, DockManager, DockPanel, DockPanelFactory, DockPanelFactoryCx,
        DockPanelRegistryBuilder, DockPanelRegistryService, DockingPolicy, DockingPolicyService,
        create_dock_space_node_with_test_id, render_and_bind_dock_panels,
    },
    integration::InstallIntoApp,
    shadcn,
};
use fret_app::{CommandMeta, CommandScope};
use fret_core::{Color, PanelKind, Px};
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
const PANEL_DESCRIPTION: &str = "Dock content is app-owned, while reusable panel contributions aggregate through fret::docking::DockPanelFactory over the fret-docking ecosystem layer.";

fn install_commands(app: &mut KernelApp) {
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

struct DockingBasicsBundle;

impl InstallIntoApp for DockingBasicsBundle {
    fn install_into_app(self, app: &mut fret::app::App) {
        install_commands(app);
        install_docking_services(app);
        shadcn::app::install(app);
        fret_cookbook::install_cookbook_defaults(app);
    }
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

struct DockingBasicsCardPanelFactory {
    kind: PanelKind,
    title: &'static str,
}

impl DockingBasicsCardPanelFactory {
    fn new(kind: &'static str, title: &'static str) -> Self {
        Self {
            kind: PanelKind::new(kind),
            title,
        }
    }
}

impl DockPanelFactory<KernelApp> for DockingBasicsCardPanelFactory {
    fn panel_kind(&self) -> PanelKind {
        self.kind.clone()
    }

    fn build_panel(
        &self,
        panel: &PanelKey,
        cx: &mut DockPanelFactoryCx<'_, KernelApp>,
    ) -> Option<fret_core::NodeId> {
        let root_name = format!("cookbook.docking.panel.{}", panel.kind.0);
        Some(cx.render_cached_panel_root(&root_name, |cx| {
            let body = shadcn::card(|cx| {
                ui::children![
                    cx;
                    shadcn::card_header(|cx| {
                        ui::children![
                            cx;
                            shadcn::card_title(self.title),
                            shadcn::card_description(PANEL_DESCRIPTION),
                        ]
                    }),
                    shadcn::card_content(|cx| {
                        ui::children![
                            cx;
                            cx.text("Try: click tabs, drag tabs, drag the splitter, right-click a tab."),
                        ]
                    }),
                ]
            })
            .ui()
            .w_full()
            .h_full()
            .into_element(cx);

            vec![body]
        }))
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

fn install_docking_services(app: &mut KernelApp) {
    let mut registry = DockPanelRegistryBuilder::new();
    registry
        .register(DockingBasicsCardPanelFactory::new(
            "core.hierarchy",
            "Hierarchy",
        ))
        .register(DockingBasicsCardPanelFactory::new(
            "core.inspector",
            "Inspector",
        ))
        .register(DockingBasicsCardPanelFactory::new("core.editor", "Editor"))
        .register(DockingBasicsCardPanelFactory::new(
            "core.console",
            "Console",
        ));

    app.with_global_mut(
        DockPanelRegistryService::<KernelApp>::default,
        |svc, _app| {
            svc.set(registry.build_arc());
        },
    );

    app.with_global_mut(DockingPolicyService::default, |svc, _app| {
        svc.set(Arc::new(DockingBasicsPolicy));
    });
}

fn init_window(app: &mut KernelApp, window: AppWindowId) -> DockingBasicsWindowState {
    let layout_ids = app.with_global_mut(DockManager::default, |dock, _app| {
        reset_dock_layout(dock, window)
    });

    DockingBasicsWindowState { window, layout_ids }
}

fn active_tab_title(app: &KernelApp, tabs: DockNodeId) -> Option<String> {
    let dock = app.global::<DockManager>()?;
    let DockNode::Tabs { tabs, active } = dock.graph.node(tabs)? else {
        return None;
    };
    let panel = tabs.get(*active)?;
    dock.panel(panel).map(|p| p.title.clone())
}

fn active_tab_state(app: &KernelApp, tabs: DockNodeId) -> Option<(u32, u32)> {
    let dock = app.global::<DockManager>()?;
    let DockNode::Tabs { tabs, active } = dock.graph.node(tabs)? else {
        return None;
    };

    Some((*active as u32, tabs.len() as u32))
}

fn view(cx: &mut ElementContext<'_, KernelApp>, st: &mut DockingBasicsWindowState) -> ViewElements {
    let active_left = active_tab_title(cx.app, st.layout_ids.left_tabs).unwrap_or("Unknown".into());
    let active_right =
        active_tab_title(cx.app, st.layout_ids.right_tabs).unwrap_or("Unknown".into());

    let (active_left_index, left_count) =
        active_tab_state(cx.app, st.layout_ids.left_tabs).unwrap_or((0, 0));
    let (active_right_index, right_count) =
        active_tab_state(cx.app, st.layout_ids.right_tabs).unwrap_or((0, 0));

    let toolbar = ui::h_flex(|cx| {
        let left_max = (left_count.saturating_sub(1)) as f64;
        let right_max = (right_count.saturating_sub(1)) as f64;

        let active_left_badge = shadcn::Badge::new(format!("Left: {active_left}"))
            .variant(shadcn::BadgeVariant::Secondary)
            .a11y(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Generic)
                    .test_id(TEST_ID_ACTIVE_LEFT)
                    .numeric_value(active_left_index as f64)
                    .numeric_range(0.0, left_max),
            );

        let active_right_badge = shadcn::Badge::new(format!("Right: {active_right}"))
            .variant(shadcn::BadgeVariant::Secondary)
            .a11y(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Generic)
                    .test_id(TEST_ID_ACTIVE_RIGHT)
                    .numeric_value(active_right_index as f64)
                    .numeric_range(0.0, right_max),
            );

        ui::children![
            cx;
            shadcn::Button::new("Reset layout")
                .variant(shadcn::ButtonVariant::Outline)
                .on_click(CMD_RESET_LAYOUT)
                .test_id(TEST_ID_RESET_LAYOUT),
            shadcn::Button::new("Activate Hierarchy")
                .variant(shadcn::ButtonVariant::Secondary)
                .on_click(CMD_ACTIVATE_HIERARCHY)
                .test_id(TEST_ID_ACTIVATE_HIERARCHY),
            shadcn::Button::new("Activate Inspector")
                .variant(shadcn::ButtonVariant::Secondary)
                .on_click(CMD_ACTIVATE_INSPECTOR)
                .test_id(TEST_ID_ACTIVATE_INSPECTOR),
            shadcn::Button::new("Activate Editor")
                .variant(shadcn::ButtonVariant::Secondary)
                .on_click(CMD_ACTIVATE_EDITOR)
                .test_id(TEST_ID_ACTIVATE_EDITOR),
            shadcn::Button::new("Activate Console")
                .variant(shadcn::ButtonVariant::Secondary)
                .on_click(CMD_ACTIVATE_CONSOLE)
                .test_id(TEST_ID_ACTIVATE_CONSOLE),
            active_left_badge,
            active_right_badge,
        ]
    })
    .gap(Space::N2)
    .items_center();

    let dock_host =
        cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
            let window = st.window;

            let mut layout = LayoutStyle::default();
            layout.size.width = Length::Fill;
            layout.size.height = Length::Fill;

            let props =
                fret_ui::retained_bridge::RetainedSubtreeProps::new::<KernelApp>(move |ui| {
                    let dock_space =
                        create_dock_space_node_with_test_id(ui, window, TEST_ID_DOCK_SPACE);
                    ui.create_node_retained(DockingBasicsDockHostRoot::new(window, dock_space))
                })
                .with_layout(layout);

            vec![cx.retained_subtree(props)]
        });

    let card = shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::card_title("Docking basics"),
                    shadcn::card_description(
                        "Minimal retained dock host + app-owned panel registry + runner dock_op wiring.",
                    ),
                ]
            }),
            shadcn::card_content(|cx| {
                ui::children![
                    cx;
                    ui::v_flex(|cx| ui::children![cx; toolbar, dock_host])
                        .gap(Space::N3)
                        .w_full()
                        .h_full()
                        .min_w_0(),
                ]
            }),
        ]
    })
    .ui()
    .w_full()
    .h_full()
    .max_w(Px(1100.0));

    let root = fret_cookbook::scaffold::centered_page_muted_ui(cx, TEST_ID_ROOT, card);

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
    app: &mut KernelApp,
    _services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    _ui: &mut UiTree<KernelApp>,
    st: &mut DockingBasicsWindowState,
    command: &CommandId,
) {
    let cmd = command.as_str();

    if cmd == CMD_RESET_LAYOUT {
        let ids = app.with_global_mut(DockManager::default, |dock, _app| {
            reset_dock_layout(dock, window)
        });
        st.layout_ids = ids;
        docking::request_dock_invalidation(app, [window]);
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
    let _ = docking::handle_dock_op(app, op);
    docking::request_dock_invalidation(app, [window]);
}

fn on_dock_op(app: &mut KernelApp, op: DockOp) {
    // DockSpace emits Effect::Dock(op); the runner routes it here.
    //
    // Notes:
    // - `handle_dock_op` applies pure graph ops and translates tear-off requests into window
    //   create requests.
    // - This cookbook example installs a policy that disables tear-off.
    let _ = docking::handle_dock_op(app, op);
}

fn configure_driver(
    driver: UiAppDriver<DockingBasicsWindowState>,
) -> UiAppDriver<DockingBasicsWindowState> {
    driver.on_command(on_command).dock_op(on_dock_op)
}

fn main() -> anyhow::Result<()> {
    let builder = ui_app_with_hooks(ROOT_NAME, init_window, view, configure_driver)
        .with_main_window("cookbook-docking-basics", (1120.0, 820.0))
        .with_command_default_keybindings()
        .setup(DockingBasicsBundle)
        .setup(fret_icons_lucide::app::install)
        .with_ui_assets_budgets(64 * 1024 * 1024, 4096, 16 * 1024 * 1024, 4096);

    #[cfg(feature = "cookbook-diag")]
    let builder = builder.with_default_diagnostics();

    builder.run().map_err(anyhow::Error::from)
}

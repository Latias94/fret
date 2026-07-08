use std::sync::Arc;

use fret::advanced::raw::UiTree;
use fret::{advanced::prelude::*, component::prelude::*, integration::InstallIntoApp, shadcn};
use fret_app::{CommandMeta, CommandScope};
use fret_core::{Axis, Color, DockLayout, DockLayoutNode, DockLayoutWindow, DockOp, PanelKey, Px};
use fret_docking::{
    DockHostOptions, DockPanel, DockPanelElementRegistry, DockSurface, DockSurfacePanelPlacement,
    DockSurfacePanelSnapshot, DockingPolicy,
};
use fret_runtime::CommandId;
use fret_ui::ElementContext;
use fret_ui::element::{AnyElement, SemanticsDecoration, SemanticsProps};
use fret_ui_kit::prelude::{CachedSubtreeExt, CachedSubtreeProps};

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
const PANEL_DESCRIPTION: &str = "Dock content is app-owned, while reusable panel contributions aggregate through fret_docking::DockPanelElementRegistry over the fret-docking ecosystem layer.";

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

fn ensure_dock_panels(app: &mut KernelApp, surface: DockSurface) {
    surface.ensure_panel(app, &panel_hierarchy(), || DockPanel {
        title: "Hierarchy".to_string(),
        color: Color::from_srgb_hex_rgb(0x3B82F6),
        viewport: None,
    });
    surface.ensure_panel(app, &panel_inspector(), || DockPanel {
        title: "Inspector".to_string(),
        color: Color::from_srgb_hex_rgb(0xA855F7),
        viewport: None,
    });
    surface.ensure_panel(app, &panel_editor(), || DockPanel {
        title: "Editor".to_string(),
        color: Color::from_srgb_hex_rgb(0x22C55E),
        viewport: None,
    });
    surface.ensure_panel(app, &panel_console(), || DockPanel {
        title: "Console".to_string(),
        color: Color::from_srgb_hex_rgb(0xF97316),
        viewport: None,
    });
}

fn default_dock_layout() -> DockLayout {
    DockLayout::new(
        vec![DockLayoutWindow {
            logical_window_id: "main".to_string(),
            root: 3,
            placement: None,
            floatings: Vec::new(),
        }],
        vec![
            DockLayoutNode::Tabs {
                id: 1,
                tabs: vec![panel_hierarchy(), panel_inspector()],
                active: 0,
            },
            DockLayoutNode::Tabs {
                id: 2,
                tabs: vec![panel_editor(), panel_console()],
                active: 0,
            },
            DockLayoutNode::Split {
                id: 3,
                axis: Axis::Horizontal,
                children: vec![1, 2],
                fractions: vec![0.3, 0.7],
            },
        ],
    )
}

fn reset_dock_layout(app: &mut KernelApp, window: AppWindowId) {
    let surface = DockSurface::new(window);
    ensure_dock_panels(app, surface);
    let layout = default_dock_layout();
    let windows = [(window, "main".to_string())];
    surface
        .try_import_layout_for_windows(app, &layout, &windows)
        .expect("default dock layout should import");
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

impl DockingBasicsPanelRegistry {
    fn title_for(panel: &PanelKey) -> Option<&'static str> {
        match panel.kind.0.as_str() {
            "core.hierarchy" => Some("Hierarchy"),
            "core.inspector" => Some("Inspector"),
            "core.editor" => Some("Editor"),
            "core.console" => Some("Console"),
            _ => None,
        }
    }
}

impl DockPanelElementRegistry<KernelApp> for DockingBasicsPanelRegistry {
    fn render_panel(
        &self,
        cx: &mut ElementContext<'_, KernelApp>,
        _window: AppWindowId,
        panel: &PanelKey,
    ) -> Option<AnyElement> {
        let title = Self::title_for(panel)?;
        Some(
            shadcn::card(|cx| {
                ui::children![
                    cx;
                    shadcn::card_header(|cx| {
                        ui::children![
                            cx;
                            shadcn::card_title(title),
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
            .into_element(cx),
        )
    }
}

#[derive(Debug)]
struct DockingBasicsWindowState {
    window: AppWindowId,
}

fn install_docking_services(app: &mut KernelApp) {
    let surface = DockSurface::new(AppWindowId::default());
    surface.install_panel_registry(app, Arc::new(DockingBasicsPanelRegistry));
    surface.install_policy(app, Arc::new(DockingBasicsPolicy));
}

fn init_window(app: &mut KernelApp, window: AppWindowId) -> DockingBasicsWindowState {
    reset_dock_layout(app, window);
    DockingBasicsWindowState { window }
}

#[derive(Debug, Clone)]
struct ActiveGroupStatus {
    title: String,
    index: u32,
    count: u32,
}

fn active_group_status(
    app: &KernelApp,
    window: AppWindowId,
    panels: &[PanelKey],
) -> ActiveGroupStatus {
    let surface = DockSurface::new(window);
    let snapshots = surface.panels_in_window(app, window);

    for panel in panels {
        let Some(snapshot) = snapshots.iter().find(|snapshot| &snapshot.key == panel) else {
            continue;
        };
        let Some(location) = snapshot.location.as_ref() else {
            continue;
        };
        if location.placement == DockSurfacePanelPlacement::Docked && location.active {
            return status_from_snapshot(snapshot);
        }
    }

    ActiveGroupStatus {
        title: "Unknown".to_string(),
        index: 0,
        count: panels.len() as u32,
    }
}

fn status_from_snapshot(snapshot: &DockSurfacePanelSnapshot) -> ActiveGroupStatus {
    let Some(location) = snapshot.location.as_ref() else {
        return ActiveGroupStatus {
            title: snapshot.title.clone(),
            index: 0,
            count: 1,
        };
    };

    ActiveGroupStatus {
        title: snapshot.title.clone(),
        index: location.tab_index as u32,
        count: location.tab_count as u32,
    }
}

fn view(cx: &mut ElementContext<'_, KernelApp>, st: &mut DockingBasicsWindowState) -> ViewElements {
    let active_left =
        active_group_status(cx.app, st.window, &[panel_hierarchy(), panel_inspector()]);
    let active_right = active_group_status(cx.app, st.window, &[panel_editor(), panel_console()]);

    let toolbar = ui::h_flex(|cx| {
        let left_max = (active_left.count.saturating_sub(1)) as f64;
        let right_max = (active_right.count.saturating_sub(1)) as f64;

        let active_left_badge = shadcn::Badge::new(format!("Left: {}", active_left.title))
            .variant(shadcn::BadgeVariant::Secondary)
            .a11y(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Generic)
                    .test_id(TEST_ID_ACTIVE_LEFT)
                    .numeric_value(active_left.index as f64)
                    .numeric_range(0.0, left_max),
            );

        let active_right_badge = shadcn::Badge::new(format!("Right: {}", active_right.title))
            .variant(shadcn::BadgeVariant::Secondary)
            .a11y(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Generic)
                    .test_id(TEST_ID_ACTIVE_RIGHT)
                    .numeric_value(active_right.index as f64)
                    .numeric_range(0.0, right_max),
            );

        ui::children![
            cx;
            shadcn::Button::new("Reset layout")
                .variant(shadcn::ButtonVariant::Outline)
                .action(CMD_RESET_LAYOUT)
                .test_id(TEST_ID_RESET_LAYOUT),
            shadcn::Button::new("Activate Hierarchy")
                .variant(shadcn::ButtonVariant::Secondary)
                .action(CMD_ACTIVATE_HIERARCHY)
                .test_id(TEST_ID_ACTIVATE_HIERARCHY),
            shadcn::Button::new("Activate Inspector")
                .variant(shadcn::ButtonVariant::Secondary)
                .action(CMD_ACTIVATE_INSPECTOR)
                .test_id(TEST_ID_ACTIVATE_INSPECTOR),
            shadcn::Button::new("Activate Editor")
                .variant(shadcn::ButtonVariant::Secondary)
                .action(CMD_ACTIVATE_EDITOR)
                .test_id(TEST_ID_ACTIVATE_EDITOR),
            shadcn::Button::new("Activate Console")
                .variant(shadcn::ButtonVariant::Secondary)
                .action(CMD_ACTIVATE_CONSOLE)
                .test_id(TEST_ID_ACTIVATE_CONSOLE),
            active_left_badge,
            active_right_badge,
        ]
    })
    .gap(Space::N2)
    .items_center();

    let dock_host = cx.cached_subtree_with(
        CachedSubtreeProps::default().contain_layout_when_bounds_known(true),
        |cx| {
            let window = st.window;
            vec![DockSurface::new(window).host(
                cx,
                window,
                DockHostOptions {
                    test_id: Some(TEST_ID_DOCK_SPACE),
                    ..Default::default()
                },
            )]
        },
    );

    let card = shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::card_title("Docking basics"),
                    shadcn::card_description(
                        "Minimal declarative dock host + app-owned panel registry + runner dock_op wiring.",
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

    let root = fret_cookbook::scaffold::centered_page_muted(cx, TEST_ID_ROOT, card);

    vec![cx.semantics(
        SemanticsProps {
            role: SemanticsRole::Group,
            test_id: None,
            ..Default::default()
        },
        |_cx| root,
    )]
    .into()
}

fn on_command(
    app: &mut KernelApp,
    _services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    _ui: &mut UiTree<KernelApp>,
    _st: &mut DockingBasicsWindowState,
    command: &CommandId,
) {
    let cmd = command.as_str();

    if cmd == CMD_RESET_LAYOUT {
        reset_dock_layout(app, window);
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

    let surface = DockSurface::new(window);
    let _ = surface.select_panel(app, &panel);
}

fn on_dock_op(app: &mut KernelApp, op: DockOp) {
    // DockSpace emits Effect::Dock(op); the runner routes it here.
    let surface = DockSurface::new(AppWindowId::default());
    let _ = surface.host_lifecycle().on_dock_op(app, op);
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
        .setup((DockingBasicsBundle, fret_icons_lucide::app::install))
        .with_ui_assets_budgets(64 * 1024 * 1024, 4096, 16 * 1024 * 1024, 4096);

    #[cfg(feature = "cookbook-diag")]
    let builder = builder.with_default_diagnostics();

    builder.run().map_err(anyhow::Error::from)
}

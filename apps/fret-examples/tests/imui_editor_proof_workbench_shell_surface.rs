#[test]
fn imui_editor_proof_demo_routes_dock_window_shell_through_demo_local_owner() {
    let demo_source = include_str!("../src/imui_editor_proof_demo.rs");
    let shell_source = include_str!("../src/imui_editor_proof_demo/workbench_shell.rs");

    for needle in [
        "mod workbench_shell;",
        ".dock_op(workbench_shell::on_dock_op)",
        ".window_create_spec(workbench_shell::window_create_spec)",
        ".window_created(workbench_shell::window_created)",
        ".before_close_window(workbench_shell::before_close_window)",
        "workbench_shell::install_dock_panel_registry(app)",
        "workbench_shell::ensure_aux_window_requested(app, window)",
        "workbench_shell::dock_test_id_for_window(cx.app, window)",
        "workbench_shell::reset_dock_graph(ui.cx_mut().app, window)",
        "workbench_shell::ensure_dock_graph(cx.app, cx.window)",
    ] {
        assert!(
            demo_source.contains(needle),
            "imui_editor_proof_demo should route supporting proof shell behavior through the demo-local workbench_shell owner; missing `{needle}`"
        );
    }

    for needle in [
        "struct WindowBootstrapService",
        "struct ImUiEditorProofControlsPanelRegistry",
        "impl DockPanelElementRegistry<KernelApp> for ImUiEditorProofControlsPanelRegistry",
        "pub(super) fn install_dock_panel_registry",
        "pub(super) fn dock_test_id_for_window",
        "pub(super) fn ensure_dock_graph",
        "pub(super) fn reset_dock_graph",
        "fn ensure_dock_graph_inner",
        "pub(super) fn ensure_aux_window_requested",
        "pub(super) fn on_dock_op",
        "pub(super) fn window_create_spec",
        "pub(super) fn window_created",
        "pub(super) fn before_close_window",
        "DockManager::default",
        "CreateWindowKind::DockRestore",
        "CreateWindowKind::DockFloating",
        "DockFloatingWindow",
        "ViewportPanel",
        "WindowRole::Auxiliary",
        "ActivationPolicy::NonActivating",
        "dock_runtime::handle_dock_window_created",
        "dock_runtime::handle_dock_before_close_window",
    ] {
        assert!(
            shell_source.contains(needle),
            "workbench_shell should own supporting proof dock graph and window lifecycle policy; missing `{needle}`"
        );
    }

    for unexpected in [
        "struct WindowBootstrapService",
        "struct ImUiEditorProofControlsPanelRegistry",
        "impl DockPanelElementRegistry<KernelApp> for ImUiEditorProofControlsPanelRegistry",
        "fn ensure_dock_graph_inner",
        "CreateWindowKind::DockRestore",
        "DockManager::default",
    ] {
        assert!(
            !demo_source.contains(unexpected),
            "imui_editor_proof_demo.rs should stay focused on proof rendering after shell owner split; unexpected `{unexpected}`"
        );
    }
}

#[test]
fn imui_editor_proof_workbench_shell_uses_explicit_public_surfaces() {
    let shell_source = include_str!("../src/imui_editor_proof_demo/workbench_shell.rs");
    let compact: String = shell_source.split_whitespace().collect();

    for needle in [
        "usefret::advanced::KernelApp;",
        "usefret::app::AppRenderDataExtas_;",
        "usefret_app::{CreateWindowKind,CreateWindowRequest,Effect,WindowRequest};",
        "usefret_core::{AppWindowId,Color,DockFloatingWindow,DockNode,Point,Px,Rect,Size};",
        "usefret_ui::ElementContext;",
        "usefret_ui_kit::{IntoUiElementas_,StyledExtas_,UiExtas_};",
    ] {
        assert!(
            compact.contains(needle),
            "workbench_shell should name required advanced/app/component surfaces explicitly; missing `{needle}`",
        );
    }

    for forbidden in ["advanced::prelude::*", "component::prelude::*"] {
        assert!(
            !shell_source.contains(forbidden),
            "workbench_shell should not reintroduce broad prelude imports: `{forbidden}`",
        );
    }
}

#[test]
fn imui_editor_proof_demo_uses_explicit_public_surfaces() {
    let demo_source = include_str!("../src/imui_editor_proof_demo.rs");
    let compact: String = demo_source.split_whitespace().collect();

    for needle in [
        "usefret::{AppUi,Defaults,FretApp,Ui,shadcn};",
        "usefret::advanced::KernelApp;",
        "usefret::advanced::driver::{UiAppDriver,ViewElements};",
        "EmbeddedViewportUiAppDriverExtas_",
        "usefret::advanced::view::ViewWindowState;",
        "usefret::app::{ElementContextAccess,View};",
        "UiWriteras_",
        "UiWriterImUiFacadeExtas_",
        "UiWriterUiKitExtas_",
        "kit::{self,ImUiMultiSelectState}",
        "usefret_ui_kit::IntoUiElementas_;",
    ] {
        assert!(
            compact.contains(needle),
            "imui_editor_proof_demo should name required app/advanced/imui/component surfaces explicitly; missing `{needle}`",
        );
    }

    for symbol in [
        "AppWindowId",
        "Color",
        "KeyCode",
        "Modifiers",
        "PanelKind",
        "Point",
        "PointerId",
        "Px",
        "Rect",
        "Size",
    ] {
        assert!(
            compact.contains(&format!("{symbol},")),
            "imui_editor_proof_demo should import `{symbol}` explicitly from fret_core",
        );
    }

    for forbidden in [
        "advanced::prelude::*",
        "component::prelude::*",
        "imui::prelude::*",
        "fret::UiAppDriver",
    ] {
        assert!(
            !demo_source.contains(forbidden),
            "imui_editor_proof_demo should not reintroduce broad or root-level advanced imports: `{forbidden}`",
        );
    }
}

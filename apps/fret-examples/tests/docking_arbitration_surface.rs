fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn docking_arbitration_demo_keeps_body_and_state_text_on_roles() {
    let source = include_str!("../src/docking_arbitration_demo.rs");
    let source = compact(source);

    for needle in [
        "usefret::app::{AppRenderContext,text};",
        "fndocking_arbitration_readout_text<'a,Cx>(",
        "fndocking_arbitration_paragraph_text<'a,Cx>(",
        "Cx:AppRenderContext<'a>,",
        "text::control_readout(cx,text)",
        "text::paragraph(cx,text)",
        "docking_arbitration_paragraph_text(cx,\"Non-modaloverlay(Popover).\")",
        ".map(|v|docking_arbitration_readout_text(cx,v))",
        "docking_arbitration_readout_text(cx,ifpopover_is_open{\"Popover:open\"}else{\"Popover:closed\"},)",
        "docking_arbitration_readout_text(cx,ifdialog_is_open{\"Dialog:open\"}else{\"Dialog:closed\"},)",
        "docking_arbitration_readout_text(cx,ifdrop_mask_left_disallowed{\"Dropmask:leftedgedockingdisallowed\"}else{\"Dropmask:leftedgedockingallowed\"},)",
    ] {
        assert!(
            source.contains(needle),
            "docking arbitration demo should keep fixed state readouts on shared text roles; missing `{needle}`"
        );
    }

    for needle in [
        "vec![cx.text(ifpopover_is_open",
        "vec![cx.text(ifdialog_is_open",
        "vec![cx.text(ifdrop_mask_left_disallowed",
        "cx.text(\"Non-modaloverlay(Popover).\")",
        ".map(|v|cx.text(v))",
        "fret_ui_kit::declarative::text::text_control_readout(",
        "fret_ui_kit::declarative::text::text_paragraph(",
    ] {
        assert!(
            !source.contains(needle),
            "docking arbitration demo should not render state readouts with bare wrapping text; unexpected `{needle}`"
        );
    }
}

#[test]
fn docking_arbitration_demo_model_writes_stay_behind_controls_binding() {
    let source = include_str!("../src/docking_arbitration_demo.rs");
    let production_source = source
        .split("#[cfg(test)]")
        .next()
        .expect("docking arbitration demo should have production source before tests");
    let compact_production = compact(production_source);

    for needle in [
        "usefret_runtime::{ModelStore,PlatformCapabilities};",
        "structDockingArbitrationControls{",
        "structDockingArbitrationControlsService{",
        "letcontrols=DockingArbitrationControls::new(app.models_mut());",
        "DockingArbitrationControlsService::default",
        "svc.set(window,controls);",
        "letnext=controls.toggle_drop_mask_disallow_left_edge(host);",
        "controls.set_synth_pointer_debug(app,msg);",
        "controls.set_last_viewport_input(app,msg);",
        "if!synth.enabled&&pressed{return;}",
    ] {
        assert!(
            compact_production.contains(needle),
            "docking arbitration demo should route diagnostic model writes through its controls binding; missing `{needle}`"
        );
    }

    for forbidden in [
        "structDockingArbitrationPanelModels",
        "DockingArbitrationPanelModelsService",
        "structViewportDebugService",
        "last_event:HashMap<AppWindowId,Model<Arc<str>>>",
        "structDockingArbitrationModelOwner",
        "DockingArbitrationModelOwner::new(host.models_mut()).toggle_drop_mask_disallow_left_edge(&drop_mask_disallow_left_edge);",
        "set_synth_pointer_debug(&models.synth_pointer_debug,msg)",
        "set_last_viewport_input(&model,msg.clone())",
        "models_mut().update(",
        "models_mut().update::<",
        "models_mut().update_any(",
        "models_mut().update_any::<",
        "ModelStore::update(",
        "ModelStore::update::<",
        "ModelStore::update_any(",
        "ModelStore::update_any::<",
        "<ModelStore>::update(",
        "<ModelStore>::update::<",
        "<ModelStore>::update_any(",
        "<ModelStore>::update_any::<",
    ] {
        assert!(
            !compact_production.contains(forbidden),
            "docking arbitration production code should not bypass the controls binding with `{forbidden}`"
        );
    }

    let mut model_store_aliases = Vec::new();
    for statement in production_source.split(';') {
        let compact_statement = compact(statement);
        let Some(rest) = compact_statement.strip_prefix("let") else {
            continue;
        };
        let Some((alias, rhs)) = rest.split_once('=') else {
            continue;
        };
        if rhs.contains("models_mut()") {
            model_store_aliases.push(alias.trim_start_matches("mut").to_string());
        }
    }

    for alias in model_store_aliases {
        for forbidden in [
            format!("{alias}.update("),
            format!("{alias}.update::<"),
            format!("{alias}.update_any("),
            format!("{alias}.update_any::<"),
        ] {
            assert!(
                !compact_production.contains(&forbidden),
                "docking arbitration production code should not bypass the controls binding through a ModelStore alias with `{forbidden}`"
            );
        }
    }
}

#[test]
fn docking_arbitration_demo_uses_dock_surface_for_common_docking_assembly() {
    let source = include_str!("../src/docking_arbitration_demo.rs");

    for needle in [
        "DockSurface",
        "DockSurface::new",
        "surface.install_policy",
        "surface.install_panel_registry",
        "surface.install_viewport_overlay_hooks",
        "surface.host",
        "surface.host_lifecycle().on_dock_op",
        ".host_lifecycle()",
        "DockManager",
        "advanced::{DockManager, request_dock_invalidation}",
    ] {
        assert!(
            source.contains(needle),
            "docking arbitration demo should route common docking setup through DockSurface while retaining diagnostic graph access; missing `{needle}`"
        );
    }

    for forbidden in [
        "DockingPolicyService",
        "DockViewportOverlayHooksService",
        "DockPanelElementRegistryService",
        "DockingRuntime::new",
        "dock_space_element_from_registry",
        "DockPanelFactory",
        "DockPanelFactoryCx",
        "DockPanelRegistryBuilder",
        "DockPanelRegistryService",
        "DockSpace::new",
        "create_dock_space_node",
        "mount_dock_space",
        "render_and_bind_dock_panels",
        "dock_space_with(",
        "DockSpaceImUiOptions",
        "surface.driver()",
        "flush_runtime_commands_to_effects",
    ] {
        assert!(
            !source.contains(forbidden),
            "docking arbitration demo should not regress to legacy low-level docking entry point `{forbidden}`"
        );
    }
}

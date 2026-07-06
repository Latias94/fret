fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn gizmo3d_demo_uses_app_facing_plot3d_binding() {
    let source = compact(include_str!("../src/gizmo3d_demo.rs"));

    for needle in [
        "usefret_plot3d::{Plot3dPanelBinding,Plot3dStyle,Plot3dViewport,plot3d_panel};",
        "plot:Plot3dPanelBinding,",
        "Plot3dPanelBinding::new(app,Plot3dViewport{",
        "state.plot.viewport_untracked(app).target_px_size",
        "state.plot.sync_viewport_target(app,id,size)",
        "state.plot.panel_props().style(style)",
    ] {
        assert!(
            source.contains(needle),
            "gizmo3d_demo should keep Plot3D panel state behind Plot3dPanelBinding; missing `{needle}`"
        );
    }

    for legacy in [
        "plot:fret_runtime::Model<Plot3dModel>",
        "usefret_plot3d::{Plot3dModel,Plot3dPanelProps,Plot3dStyle,Plot3dViewport,plot3d_panel};",
        "app.models_mut().insert(Plot3dModel{",
        "state.plot.read(app,",
        "state.plot.update(app,",
        "Plot3dPanelProps::new(state.plot.clone())",
    ] {
        assert!(
            !source.contains(legacy),
            "gizmo3d_demo should not expose raw Plot3D model handles in app code; unexpected `{legacy}`"
        );
    }
}

#[test]
fn gizmo3d_demo_hides_demo_model_handle_behind_binding() {
    let source = compact(include_str!("../src/gizmo3d_demo.rs"));

    for needle in [
        "structGizmo3dDemoModelBinding{",
        "model:fret_runtime::Model<Gizmo3dDemoModel>,",
        "per_window:HashMap<AppWindowId,Gizmo3dDemoModelBinding>,",
        "demo:Gizmo3dDemoModelBinding,",
        "letdemo=Gizmo3dDemoModelBinding::new(app);",
        "demo.apply_viewport_theme(app);",
        "svc.per_window.insert(window,demo.clone());",
        "state.demo.sync_viewport_target(app,id,size)",
    ] {
        assert!(
            source.contains(needle),
            "gizmo3d_demo should keep the shared demo model handle behind Gizmo3dDemoModelBinding; missing `{needle}`"
        );
    }

    for legacy in [
        "per_window:HashMap<AppWindowId,fret_runtime::Model<Gizmo3dDemoModel>>",
        "demo:fret_runtime::Model<Gizmo3dDemoModel>,",
        "letdemo=app.models_mut().insert(Gizmo3dDemoModel::default());",
        "let_=demo.update(app,|m,_cx|{apply_viewport_gizmo_theme(&theme,m);});",
        "let_=state.demo.update(app,|m,_cx|{m.viewport_target=id;m.viewport_px=size;});",
    ] {
        assert!(
            !source.contains(legacy),
            "gizmo3d_demo should not expose the raw Gizmo3dDemoModel handle outside the binding; unexpected `{legacy}`"
        );
    }
}

#[test]
fn gizmo3d_demo_routes_basic_keyboard_mutations_through_binding() {
    let source = compact(include_str!("../src/gizmo3d_demo.rs"));

    for needle in [
        "state.demo.cancel_active_or_in_progress(app)",
        "state.demo.set_transform_mode(app,GizmoMode::Rotate,GizmoOpMaskPreset::Rotate);",
        "state.demo.set_transform_mode(app,GizmoMode::Scale,GizmoOpMaskPreset::Scale);",
        "state.demo.set_transform_mode(app,GizmoMode::Translate,GizmoOpMaskPreset::Translate);",
        "state.demo.set_transform_mode(app,GizmoMode::Universal,GizmoOpMaskPreset::Universal);",
        "state.demo.toggle_help(app);",
        "state.demo.toggle_op_mask(app);",
        "state.demo.toggle_depth_mode(app);",
        "state.demo.toggle_universal_translate_depth(app);",
    ] {
        assert!(
            source.contains(needle),
            "gizmo3d_demo basic keyboard mutations should route through binding methods; missing `{needle}`"
        );
    }

    for legacy in [
        "key:fret_core::KeyCode::Escape,..}=>{letdid_cancel=state.demo.update(app,|m,_cx|{m.cancel_active_viewport_tool_interaction()||m.cancel_in_progress_interaction()})",
        "key:fret_core::KeyCode::KeyR,repeat:false,..}=>{let_=state.demo.update(app,|m,_cx|{ifm.is_busy(){return;}ifm.op_mask_enabled{m.set_op_mask_preset(GizmoOpMaskPreset::Rotate);}else{m.gizmo_mut().config.mode=GizmoMode::Rotate;}});",
        "key:fret_core::KeyCode::KeyH,repeat:false,..}=>{let_=state.demo.update(app,|m,_cx|{m.show_help=!m.show_help;});",
        "key:fret_core::KeyCode::KeyM,repeat:false,..}=>{let_=state.demo.update(app,|m,_cx|{ifm.is_busy(){return;}m.op_mask_enabled=!m.op_mask_enabled;",
        "key:fret_core::KeyCode::KeyO,repeat:false,..}=>{let_=state.demo.update(app,|m,_cx|{ifm.is_busy(){return;}m.gizmo_mut().config.depth_mode=matchm.gizmo().config.depth_mode",
        "key:fret_core::KeyCode::KeyD,repeat:false,..}=>{let_=state.demo.update(app,|m,_cx|{ifm.is_busy(){return;}m.gizmo_mut().config.universal_includes_translate_depth=!m.gizmo().config.universal_includes_translate_depth;});",
    ] {
        assert!(
            !source.contains(legacy),
            "gizmo3d_demo should not keep direct basic keyboard model writes in event branches; unexpected `{legacy}`"
        );
    }
}

#[test]
fn gizmo3d_demo_routes_visual_keyboard_mutations_through_binding() {
    let source = compact(include_str!("../src/gizmo3d_demo.rs"));

    for needle in [
        "state.demo.cycle_visual_preset(app,modifiers.shift);",
        "state.demo.cycle_size_policy(app);",
        "state.demo.adjust_size_policy_fraction(app,-step);",
        "state.demo.adjust_size_policy_fraction(app,step);",
        "state.demo.adjust_gizmo_size_px(app,-step_screen_px);",
        "state.demo.adjust_gizmo_size_px(app,step_screen_px);",
        "state.demo.adjust_gizmo_stroke_px(app,-step_screen_px);",
        "state.demo.adjust_gizmo_stroke_px(app,step_screen_px);",
    ] {
        assert!(
            source.contains(needle),
            "gizmo3d_demo visual keyboard mutations should route through binding methods; missing `{needle}`"
        );
    }

    for legacy in [
        "key:fret_core::KeyCode::KeyG,modifiers,repeat:false,..}=>{let_=state.demo.update(app,|m,_cx|{ifm.is_busy(){return;}ifmodifiers.shift{",
        "key:fret_core::KeyCode::KeyV,repeat:false,..}=>{let_=state.demo.update(app,|m,_cx|{ifm.is_busy(){return;}m.gizmo_mut().config.size_policy=",
        "key:fret_core::KeyCode::Semicolon,modifiers,repeat:false,..}=>{letstep=ifmodifiers.shift{0.25}else{0.05};let_=state.demo.update(app,|m,_cx|{ifm.is_busy(){return;}matchm.gizmo_mut().config.size_policy",
        "key:fret_core::KeyCode::Quote,modifiers,repeat:false,..}=>{letstep=ifmodifiers.shift{0.25}else{0.05};let_=state.demo.update(app,|m,_cx|{ifm.is_busy(){return;}matchm.gizmo_mut().config.size_policy",
        "key:fret_core::KeyCode::Minus,modifiers,repeat:false,..}=>{letstep_screen_px=ifmodifiers.shift{16.0}else{4.0};let_=state.demo.update(app,|m,_cx|{ifm.is_busy(){return;}letcursor_units_per_screen_px=",
        "key:fret_core::KeyCode::Equal,modifiers,repeat:false,..}=>{letstep_screen_px=ifmodifiers.shift{16.0}else{4.0};let_=state.demo.update(app,|m,_cx|{ifm.is_busy(){return;}letcursor_units_per_screen_px=",
        "key:fret_core::KeyCode::Comma,modifiers,repeat:false,..}=>{letstep_screen_px=ifmodifiers.shift{2.0}else{1.0};let_=state.demo.update(app,|m,_cx|{ifm.is_busy(){return;}letcursor_units_per_screen_px=",
        "key:fret_core::KeyCode::Period,modifiers,repeat:false,..}=>{letstep_screen_px=ifmodifiers.shift{2.0}else{1.0};let_=state.demo.update(app,|m,_cx|{ifm.is_busy(){return;}letcursor_units_per_screen_px=",
    ] {
        assert!(
            !source.contains(legacy),
            "gizmo3d_demo should not keep direct visual keyboard model writes in event branches; unexpected `{legacy}`"
        );
    }
}

#[test]
fn gizmo3d_demo_routes_interaction_keyboard_mutations_through_binding() {
    let source = compact(include_str!("../src/gizmo3d_demo.rs"));

    for needle in [
        "state.demo.cycle_op_mask_preset(app,-1);",
        "state.demo.cycle_op_mask_preset(app,1);",
        "state.demo.toggle_gizmo_orientation(app);",
        "state.demo.toggle_gizmo_pivot_mode(app);",
        "state.demo.cycle_active_target(app,1);",
        "state.demo.cycle_active_target(app,-1);",
        "state.demo.frame_targets(app,frame_all,smooth_time_s);",
        "state.demo.apply_select_all_shortcut(app,clear);",
        "state.demo.apply_target_selection_shortcut(app,id,op);",
    ] {
        assert!(
            source.contains(needle),
            "gizmo3d_demo interaction keyboard mutations should route through binding methods; missing `{needle}`"
        );
    }

    for legacy in [
        "key:fret_core::KeyCode::BracketLeft,repeat:false,..}=>{let_=state.demo.update(app,|m,_cx|{ifm.is_busy()||!m.op_mask_enabled{return;}letn=GizmoOpMaskPreset::ALL.len();",
        "key:fret_core::KeyCode::BracketRight,repeat:false,..}=>{let_=state.demo.update(app,|m,_cx|{ifm.is_busy()||!m.op_mask_enabled{return;}letn=GizmoOpMaskPreset::ALL.len();",
        "key:fret_core::KeyCode::KeyL,repeat:false,..}=>{let_=state.demo.update(app,|m,_cx|{ifm.input.dragging||m.gizmo_mgr.state.active.is_some(){return;}m.gizmo_mut().config.orientation=",
        "key:fret_core::KeyCode::KeyP,repeat:false,..}=>{let_=state.demo.update(app,|m,_cx|{ifm.input.dragging||m.gizmo_mgr.state.active.is_some(){return;}m.gizmo_mut().config.pivot_mode=",
        "key:fret_core::KeyCode::KeyN,repeat:false,..}=>{let_=state.demo.update(app,|m,_cx|{ifm.input.dragging||m.gizmo_mgr.state.active.is_some()||m.selection.is_empty(){return;}letSome(i)=m.selection.iter().position(|id|*id==m.active_target)else{",
        "key:fret_core::KeyCode::KeyB,repeat:false,..}=>{let_=state.demo.update(app,|m,_cx|{ifm.input.dragging||m.gizmo_mgr.state.active.is_some()||m.selection.is_empty(){return;}letSome(i)=m.selection.iter().position(|id|*id==m.active_target)else{",
        "key:fret_core::KeyCode::KeyF,modifiers,repeat:false,}=>{letframe_all=modifiers.shift;letsmooth_time_s=ifframe_all{0.32}else{0.18};let_=state.demo.update(app,|m,_cx|{ifm.input.dragging||m.gizmo_mgr.state.active.is_some(){return;}",
        "key:fret_core::KeyCode::KeyA,modifiers,repeat:false,}ifmodifiers.ctrl||modifiers.meta=>{letclear=modifiers.shift;let_=state.demo.update(app,|m,_cx|{letis_busy=m.input.dragging||m.gizmo_mgr.state.active.is_some()||m.pending_selection.is_some()||m.marquee.is_some();",
        "letop=selection_op(modifiers);let_=state.demo.update(app,|m,_cx|{ifm.input.dragging||m.gizmo_mgr.state.active.is_some(){return;}apply_click_selection_op(&mutm.selection,&mutm.active_target,Some(id),op);});",
    ] {
        assert!(
            !source.contains(legacy),
            "gizmo3d_demo should not keep direct interaction keyboard model writes in event branches; unexpected `{legacy}`"
        );
    }
}

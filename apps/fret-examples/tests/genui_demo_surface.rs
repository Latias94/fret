fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn genui_demo_keeps_tool_text_on_roles() {
    let source = include_str!("../src/genui_demo.rs");
    let compact_source = compact(source);

    for needle in [
        "usefret_ui::{ElementContext,UiHost};",
        "usefret::advanced::text;",
        "fngenui_code_line_text<H:UiHost>(",
        "fngenui_readout_text<H:UiHost>(",
        "fngenui_paragraph_text<H:UiHost>(",
        "text::code_block(cx,text)",
        "text::control_readout(cx,text)",
        "text::compact_paragraph(cx,text)",
        ".map(|line|genui_code_line_text(cx,line))",
        "items.push(genui_readout_text(cx,\"auto-apply\"));",
        "items.push(genui_readout_text(cx,\"auto-fixonapply\"));",
        "items.push(genui_readout_text(cx,count_label.clone()));",
        ".map(|s|genui_readout_text(cx,s)),",
        "vec![genui_readout_text(cx,\"Nospecissues.\")]",
        "genui_readout_text(",
        "\"patch-only:{}\",",
        "stream_children.push(genui_paragraph_text(",
        "stream_children.push(genui_readout_text(cx,summary));",
    ] {
        assert!(
            compact_source.contains(needle),
            "GenUI demo tool text should use shared text roles; missing `{needle}`"
        );
    }

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "decl_text::",
        "ui::text(",
        "ui::rich_text(",
        ".text_sm()",
        ".font_semibold()",
        ".font_medium()",
        ".truncate()",
        "editor_children.push(ui::text(\"\").text_sm().into_element(cx));",
    ] {
        assert!(
            !compact_source.contains(needle),
            "GenUI demo should not render tool text with local ui::text policy; unexpected `{needle}`"
        );
    }
}

#[test]
fn genui_demo_uses_explicit_public_surfaces() {
    let source = include_str!("../src/genui_demo.rs");
    let compact_source = compact(source);

    for needle in [
        "usefret::advanced::KernelApp;",
        "usefret::advanced::driver::ViewElements;",
        "usefret::advanced::text;",
        "usefret::app::LocalState;",
        "usefret::app::prelude::*;",
        "usefret::style::{ColorRef,Space,ThemeSnapshot};",
        "usefret::AppComponentCx;",
        "usefret_runtime::{Model,ModelStore};",
        "usefret_ui::action::UiActionHost;",
        "usefret_ui_kit::IntoUiElement;",
    ] {
        assert!(
            compact_source.contains(needle),
            "GenUI demo should name its required app/advanced/style surfaces explicitly; missing `{needle}`"
        );
    }

    for forbidden in [
        "LocalStateElementContextExt",
        "LocalStateRawModelExt",
        "advanced::prelude::*",
        "component::prelude::*",
    ] {
        assert!(
            !source.contains(forbidden),
            "GenUI demo should not reintroduce broad prelude imports: `{forbidden}`",
        );
    }
}

#[test]
fn genui_demo_model_writes_stay_behind_owner_helpers() {
    let source = include_str!("../src/genui_demo.rs");
    let production_source = source
        .split("#[cfg(test)]")
        .next()
        .expect("GenUI demo should have production source before tests");
    let compact_source = compact(source);
    let compact_production = compact(production_source);

    for needle in [
        "usefret_runtime::{Model,ModelStore};",
        "structGenUiModelOwner<'a>{",
        "models:&'amutModelStore,",
        "fnupdate<T:Any,R>(&mutself,model:&Model<T>,f:implFnOnce(&mutT)->R)->Option<R>{",
        "fnread<T:Any,R>(&mutself,model:&Model<T>,f:implFnOnce(&T)->R)->Option<R>{",
        "fnreset_runtime_models(&self,app:&mutKernelApp,seed:Value)",
        "state.reset_runtime_models(app,seed);",
        "letmutowner=GenUiModelOwner::new(app.models_mut());",
        "letmutowner=GenUiModelOwner::new(host.models_mut());",
        "owner.read(&state_model_for_confirm,",
        "owner.update(&state_model_for_confirm,",
        "owner.update(&validation_model,|v|*v=out);",
        "owner.update(&state_model_for_submit,",
    ] {
        assert!(
            compact_source.contains(needle),
            "GenUI demo should keep shared runtime-model access behind a named owner helper; missing `{needle}`"
        );
    }

    for forbidden in [
        "models_mut().update(",
        "models_mut().update::<",
        "models_mut().read(",
        "models_mut().read::<",
        "models_mut().update_any(",
        "models_mut().update_any::<",
        "ModelStore::update(",
        "ModelStore::update::<",
        "ModelStore::read(",
        "ModelStore::read::<",
        "ModelStore::update_any(",
        "ModelStore::update_any::<",
        "<ModelStore>::update(",
        "<ModelStore>::update::<",
        "<ModelStore>::read(",
        "<ModelStore>::read::<",
        "<ModelStore>::update_any(",
        "<ModelStore>::update_any::<",
        "fngenui_update_model",
        "fngenui_host_update_model",
        "fngenui_host_read_model",
    ] {
        assert!(
            !compact_production.contains(forbidden),
            "GenUI production code should not bypass the owner helper with `{forbidden}`"
        );
    }
}

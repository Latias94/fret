fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn external_texture_imports_visibility_writes_stay_behind_owner_helper() {
    let owner = compact(include_str!("../src/external_texture_imports_owner.rs"));
    let native = compact(include_str!("../src/external_texture_imports_demo.rs"));
    let web = compact(include_str!("../src/external_texture_imports_web_demo.rs"));

    for needle in [
        "usefret_runtime::{Model,ModelStore};",
        "pub(crate)structExternalTextureImportsModelOwner<'a>{",
        "models:&'amutModelStore,",
        "pub(crate)fntoggle_surface(&mutself,show:&Model<bool>)->bool{",
        "self.models.update(show,|show|{*show=!*show;true}).unwrap_or(false)",
    ] {
        assert!(
            owner.contains(needle),
            "external texture imports owner helper should own visibility writes; missing `{needle}`"
        );
    }

    for (label, source, target) in [
        (
            "native external texture imports",
            &native,
            "ExternalTextureImportsModelOwner::new(app.models_mut()).toggle_surface(&st.view.show);",
        ),
        (
            "web external texture imports",
            &web,
            "ExternalTextureImportsModelOwner::new(app.models_mut()).toggle_surface(&state.show);",
        ),
    ] {
        assert!(
            source.contains(
                "usecrate::external_texture_imports_owner::ExternalTextureImportsModelOwner;"
            ),
            "{label} should import the shared private owner helper"
        );
        assert!(
            source.contains(target),
            "{label} should route visibility toggles through the owner helper"
        );
        assert!(
            !source.contains("app.models_mut().update(&st.view.show,|v|*v=!*v);"),
            "{label} should not update native visibility directly"
        );
        assert!(
            !source.contains("app.models_mut().update(&state.show,|v|*v=!*v);"),
            "{label} should not update web visibility directly"
        );
    }
}

fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn external_imports_visibility_writes_stay_behind_owner_helper() {
    let owner = compact(include_str!("../src/external_imports_owner.rs"));
    let native_texture = compact(include_str!("../src/external_texture_imports_demo.rs"));
    let web_texture = compact(include_str!("../src/external_texture_imports_web_demo.rs"));
    let avf_video = compact(include_str!("../src/external_video_imports_avf_demo.rs"));
    let mf_video = compact(include_str!("../src/external_video_imports_mf_demo.rs"));

    for needle in [
        "usefret_runtime::{Model,ModelStore};",
        "pub(crate)structExternalImportsModelOwner<'a>{",
        "models:&'amutModelStore,",
        "pub(crate)fntoggle_surface(&mutself,show:&Model<bool>)->bool{",
        "self.models.update(show,|show|{*show=!*show;true}).unwrap_or(false)",
    ] {
        assert!(
            owner.contains(needle),
            "external imports owner helper should own visibility writes; missing `{needle}`"
        );
    }

    for (label, source, target) in [
        (
            "native external texture imports",
            &native_texture,
            "ExternalImportsModelOwner::new(app.models_mut()).toggle_surface(&st.view.show);",
        ),
        (
            "web external texture imports",
            &web_texture,
            "ExternalImportsModelOwner::new(app.models_mut()).toggle_surface(&state.show);",
        ),
        (
            "AVF external video imports",
            &avf_video,
            "ExternalImportsModelOwner::new(app.models_mut()).toggle_surface(&st.view.show);",
        ),
        (
            "MF external video imports",
            &mf_video,
            "ExternalImportsModelOwner::new(app.models_mut()).toggle_surface(&st.view.show);",
        ),
    ] {
        assert!(
            source.contains("usecrate::external_imports_owner::ExternalImportsModelOwner;"),
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

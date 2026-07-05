fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn api_workbench_lite_demo_keeps_fixed_text_on_roles() {
    let source = include_str!("../src/api_workbench_lite_demo.rs");
    let source = compact(source);

    for needle in [
        "usefret_ui_kit::declarative::ElementContextThemeExtas_;",
        "fnapi_workbench_section_text<'a,Cx,T>(",
        "fnapi_workbench_readout_text<'a,Cx,T>(",
        "fnapi_workbench_code_label_text<'a,Cx,T>(",
        "fnapi_workbench_paragraph_text<'a,Cx,T>(",
        "text::section_chrome_label(cx,label)",
        "text::control_readout(cx,readout)",
        "text::code_label(cx,code)",
        "text::paragraph(cx,body)",
        "api_workbench_section_text(cx,\"APIWorkbenchLite\")",
        "api_workbench_paragraph_text(cx,\"First-contacttask:buildaPostman-liketoolonlyfromFret'spublicappsurface.\",)",
        "api_workbench_section_text(cx,\"FretAPIProbe\")",
        "api_workbench_readout_text(cx,\"Postman-likefirstcontact\")",
        "api_workbench_section_text(cx,\"ActivebaseURL\")",
        "api_workbench_code_label_text(cx,base_url)",
        "api_workbench_readout_text(cx,\"SQLite-backedrequesthistory\")",
        "api_workbench_readout_text(cx,\"Loadingsavedrequests...\").test_id(TEST_ID_HISTORY_LOADING)",
        "api_workbench_section_text(cx,\"Savedhistoryfailedtoload.\")",
        "api_workbench_readout_text(cx,err.to_string()).test_id(TEST_ID_HISTORY_ERROR)",
        "api_workbench_readout_text(cx,\"Nosavedrequestsyet.\").test_id(TEST_ID_HISTORY_EMPTY)",
    ] {
        assert!(
            source.contains(needle),
            "api workbench lite demo should keep fixed chrome/readout text on shared roles; missing `{needle}`"
        );
    }

    for needle in [
        "ui::text(\"APIWorkbenchLite\").font_semibold().text_base()",
        "ui::text(\"FretAPIProbe\").font_semibold()",
        "ui::text(\"Postman-likefirstcontact\").text_sm()",
        "ui::text(\"ActivebaseURL\").text_xs().font_semibold()",
        "ui::text(base_url).text_xs()",
        "ui::text(\"SQLite-backedrequesthistory\").text_xs()",
        "ui::text(\"Loadingsavedrequests...\").text_sm()",
        "ui::text(\"Savedhistoryfailedtoload.\").text_sm().font_semibold()",
        "ui::text(err.to_string()).text_sm()",
        "ui::text(\"Nosavedrequestsyet.\").text_sm()",
        "cx.elements()",
        "textasdecl_text",
        "theme:fret::style::ThemeSnapshot,",
    ] {
        assert!(
            !source.contains(needle),
            "api workbench lite demo should not render fixed chrome/readouts with local text policy; unexpected `{needle}`"
        );
    }
}

#[test]
fn api_workbench_lite_demo_uses_app_local_state_and_explicit_shadcn_imports() {
    let source = include_str!("../src/api_workbench_lite_demo.rs");
    let compact = compact(source);

    for needle in [
        "usefret::app::prelude::*;",
        "usefret::app::{LocalState,LocalStateTxn};",
        "usefret_ui_shadcn::facadeasshadcn;",
        "structWorkbenchLocals{method:LocalState<Option<Arc<str>>>,",
        "typeApiWorkbenchModelStore=fret_runtime::ModelStore;",
        "ApiWorkbenchModelOwner::new(models).submit_request(",
        "ApiWorkbenchModelOwner::new(models).retry_last_request(",
        "ApiWorkbenchModelOwner::new(host.models_mut()).can_retry_last_request(",
        "ApiWorkbenchModelOwner::new(models).load_history(",
        "FretApp::new(\"api-workbench-lite\")",
    ] {
        assert!(
            compact.contains(needle),
            "api workbench lite demo should keep app local state and explicit shadcn imports; missing `{needle}`"
        );
    }

    for forbidden in [
        "advanced::prelude",
        "use fret::{FretApp",
        "use fret::{",
        "component::prelude",
        "KernelApp",
        "AppWindowId",
        "ViewElements",
        "LocalStateModelStoreExt",
        "LocalStateRawModelExt",
        "LocalStateElementContextExt",
    ] {
        assert!(
            !source.contains(forbidden),
            "api workbench lite demo should not reintroduce broad or kernel-facing imports: `{forbidden}`"
        );
    }

    let owner_start = source
        .find("type ApiWorkbenchModelStore = fret_runtime::ModelStore;")
        .expect("api workbench lite demo should name one local model-owner boundary");
    let owner_end = source
        .find("impl WorkbenchLocals")
        .expect("api workbench lite demo should define locals after the model owner");
    let owner_source = &source[owner_start..owner_end];
    let outside_owner = format!("{}{}", &source[..owner_start], &source[owner_end..]);

    for forbidden in [
        "fnsubmit_request(models:&mutfret_runtime::ModelStore",
        "fnretry_last_request(models:&mutfret_runtime::ModelStore",
        "fnload_history(models:&mutfret_runtime::ModelStore",
    ] {
        assert!(
            !compact.contains(forbidden),
            "api workbench lite demo should keep raw ModelStore signatures behind the owner alias: `{forbidden}`"
        );
    }

    for raw_boundary in [
        "fret_runtime::ModelStore",
        "ApiWorkbenchModelStore",
        "LocalStateTxn::with_model_store",
        "history_save_mutation.submit(self.models",
        "response_mutation.submit(self.models",
        "history_save_mutation.retry_last(self.models",
        "response_mutation.retry_last(self.models",
        ".read(response_mutation.model()",
        ".read(history_query.model()",
    ] {
        assert!(
            owner_source.contains(raw_boundary),
            "api workbench lite owner should explicitly contain raw bridge `{raw_boundary}`"
        );
        assert!(
            !outside_owner.contains(raw_boundary),
            "api workbench lite raw bridge `{raw_boundary}` should not appear outside the local owner"
        );
    }

    assert_eq!(
        source.matches("host.models_mut()").count(),
        1,
        "api workbench lite demo should keep command-availability ModelStore access behind the owner boundary"
    );
}

fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn sonner_page_keeps_docs_path_and_scope_decisions() {
    let source = include_str!("../src/ui/pages/sonner.rs");

    for needle in [
        "repo-ref/ui/apps/v4/content/docs/components/base/sonner.mdx",
        "repo-ref/ui/apps/v4/registry/new-york-v4/ui/sonner.tsx",
        "repo-ref/ui/apps/v4/registry/new-york-v4/examples/sonner-demo.tsx",
        "repo-ref/ui/apps/v4/registry/new-york-v4/examples/sonner-types.tsx",
        "repo-ref/primitives/packages/react/toast/src/toast.tsx",
        "repo-ref/base-ui/packages/react/src/toast/",
        "Docs path stays `Demo`, `About`, `Usage`, `Examples`, `Types`, `Description`, `Position`, and `API Reference`",
        "generic composable `children([...])` API is not warranted here",
        "this pass did not identify a missing `fret-ui` mechanism bug",
        "DocSection::build(cx, \"About\", about)",
        "DocSection::build(cx, \"Examples\", examples)",
        "DocSection::build(cx, \"API Reference\", api_reference)",
        "DocSection::build(cx, \"Mounting (Fret)\", setup)",
    ] {
        assert!(
            source.contains(needle),
            "sonner page should keep the docs-path source axes and scope decisions stable; missing `{needle}`"
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![
            demo,
            about,
            usage,
            examples,
            types,
            description,
            position,
            api_reference,
            setup,
            extras,
            notes,
        ]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "sonner page should keep the docs-path sections before the Fret-only follow-ups"
    );
}

#[test]
fn sonner_docs_snippets_stay_copyable_and_message_oriented() {
    let demo = include_str!("../src/ui/snippets/sonner/demo.docs.rs.txt");
    let usage = include_str!("../src/ui/snippets/sonner/usage.docs.rs.txt");
    let types = include_str!("../src/ui/snippets/sonner/types.docs.rs.txt");
    let description = include_str!("../src/ui/snippets/sonner/description.docs.rs.txt");
    let position = include_str!("../src/ui/snippets/sonner/position.docs.rs.txt");
    let extras = include_str!("../src/ui/snippets/sonner/extras.docs.rs.txt");
    let setup = include_str!("../src/ui/snippets/sonner/setup.docs.rs.txt");

    for needle in [
        "use fret::{UiChild, UiCx};",
        "use fret_ui_shadcn::{facade as shadcn, prelude::*};",
        "let sonner = shadcn::Sonner::global(&mut *cx.app);",
        "let toaster = shadcn::Toaster::new().into_element(cx);",
        "shadcn::ToastMessageOptions::new()",
    ] {
        assert!(
            demo.contains(needle),
            "sonner demo docs snippet should remain a complete copyable example; missing `{needle}`"
        );
    }

    for needle in [
        "sonner.toast_message(",
        "\"Event has been created.\"",
        "let toaster = shadcn::Toaster::new().into_element(cx);",
    ] {
        assert!(
            usage.contains(needle),
            "sonner usage docs snippet should keep the minimal message-style lane copyable; missing `{needle}`"
        );
    }

    for needle in [
        "sonner.toast_success_message(",
        "sonner.toast_info_message(",
        "sonner.toast_warning_message(",
        "sonner.toast_error_message(",
        "let promise = sonner.toast_promise(host, action_cx.window, \"Loading...\");",
        "let toaster = shadcn::Toaster::new().into_element(cx);",
    ] {
        assert!(
            types.contains(needle),
            "sonner types docs snippet should keep the variant + promise lane copyable; missing `{needle}`"
        );
    }

    for needle in [
        ".description(\"Monday, January 3rd at 6:00pm\")",
        "let toaster = shadcn::Toaster::new().into_element(cx);",
    ] {
        assert!(
            description.contains(needle),
            "sonner description docs snippet should keep the description example copyable; missing `{needle}`"
        );
    }

    for needle in [
        "let toaster_position = cx.local_model_keyed(\"docs_sonner_position\", || {",
        "shadcn::ToastPosition::TopCenter",
        "let toaster = shadcn::Toaster::new().position(current).into_element(cx);",
    ] {
        assert!(
            position.contains(needle),
            "sonner position docs snippet should keep the local-toaster placement demo copyable; missing `{needle}`"
        );
    }

    for needle in [
        ".action_id(\"Undo\", CMD_TOAST_ACTION)",
        ".cancel_id(\"Cancel\", CMD_TOAST_ACTION)",
        "shadcn::ToastRequest::new(\"Swipe to dismiss\")",
        "let toaster = shadcn::Toaster::new().into_element(cx);",
    ] {
        assert!(
            extras.contains(needle),
            "sonner extras docs snippet should keep the Fret follow-up examples copyable; missing `{needle}`"
        );
    }

    for needle in [
        "use fret::{UiChild, UiCx};",
        "shadcn::Toaster::new()",
        ".position(shadcn::ToastPosition::TopCenter)",
    ] {
        assert!(
            setup.contains(needle),
            "sonner setup docs snippet should remain the smallest mounting example; missing `{needle}`"
        );
    }

    let combined = [demo, usage, types, description, position, extras, setup].join("\n");
    assert!(
        !combined.contains(".children(["),
        "sonner docs snippets should not widen the shadcn lane into a generic children API"
    );
    assert!(
        !combined.contains("toast.custom"),
        "sonner docs snippets should stay on the message-style lane rather than teaching a custom-content API"
    );
}

#[test]
fn sonner_docs_diag_script_covers_docs_path_sections() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/sonner/ui-gallery-sonner-docs-screenshots.json"
    );

    for needle in [
        "\"ui-gallery-sonner-demo-content\"",
        "\"ui-gallery-sonner-about-content\"",
        "\"ui-gallery-sonner-usage-content\"",
        "\"ui-gallery-sonner-examples-content\"",
        "\"ui-gallery-sonner-types-content\"",
        "\"ui-gallery-sonner-description-content\"",
        "\"ui-gallery-sonner-position-content\"",
        "\"ui-gallery-sonner-api-reference-content\"",
        "\"ui-gallery-sonner-docs.08-api-reference\"",
    ] {
        assert!(
            script.contains(needle),
            "sonner docs diag script should cover the docs-path sections; missing `{needle}`"
        );
    }
}

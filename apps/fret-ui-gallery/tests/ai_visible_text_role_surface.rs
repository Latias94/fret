fn canonicalize_rust_fragment(fragment: &str) -> String {
    fragment.split_whitespace().collect()
}

#[test]
fn ai_message_demo_visible_text_uses_shared_roles() {
    let source = include_str!("../src/ui/snippets/ai/message_demo.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "decl_text::text_control_readout(cx, format!(\"last_action={last_action}\"))",
        "decl_text::text_paragraph",
        "\"User messages render as a bubble aligned to the right.\"",
        "decl_text::text_paragraph(cx, \"Bubble chrome is controlled by theme tokens.\")",
        "decl_text::text_section_chrome_label",
        "\"Message (AI Elements): alignment + bubble + actions + markdown response.\"",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "message_demo should route fixed visible text through shared text roles; missing `{marker}`"
        );
    }

    for forbidden in [
        "cx.text(format!(\"last_action={last_action}\"))",
        "cx.text(\"User messages render as a bubble aligned to the right.\")",
        "cx.text(\"Bubble chrome is controlled by theme tokens.\")",
        "cx.text(\"Message (AI Elements): alignment + bubble + actions + markdown response.\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "message_demo reintroduced visible bare text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_terminal_demo_visible_text_and_state_marker_use_shared_roles() {
    let source = include_str!("../src/ui/snippets/ai/terminal_demo.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "fn state_marker(cx: &mut AppComponentCx<'_>, test_id: &'static str) -> AnyElement",
        "role: fret_core::SemanticsRole::Generic",
        "cx.spacer(SpacerProps",
        "decl_text::text_section_chrome_label(cx, \"Terminal (AI Elements)\")",
        "decl_text::text_paragraph",
        "\"Chrome-only viewer: apps own streaming + clear behavior.\"",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "terminal_demo should route fixed visible text and state markers through shared non-bare roles; missing `{marker}`"
        );
    }

    for forbidden in [
        "role: fret_core::SemanticsRole::Text",
        "cx.text(\"Terminal (AI Elements)\")",
        "cx.text(\"Chrome-only viewer: apps own streaming + clear behavior.\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "terminal_demo reintroduced bare visible text/state marker semantics: `{forbidden}`"
        );
    }
}

#[test]
fn ai_artifact_demo_visible_text_uses_shared_roles() {
    let source = include_str!("../src/ui/snippets/ai/artifact_demo.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "decl_text::text_paragraph",
        "\"Artifacts are chrome-only: apps own rendering, export, and lifecycle.\"",
        "decl_text::text_control_readout(cx, \"Artifact closed.\")",
        "decl_text::text_section_chrome_label(cx, \"Artifact (AI Elements)\")",
        "decl_text::text_paragraph(cx, \"Close hides the artifact; reset re-mounts it.\")",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "artifact_demo should route visible text through shared roles; missing `{marker}`"
        );
    }

    for forbidden in [
        "cx.text(\"Artifacts are chrome-only: apps own rendering, export, and lifecycle.\")",
        "cx.text(\"Artifact closed.\")",
        "cx.text(\"Artifact (AI Elements)\")",
        "cx.text(\"Close hides the artifact; reset re-mounts it.\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "artifact_demo reintroduced visible bare text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_artifact_code_display_uses_non_text_status_marker() {
    let source = include_str!("../src/ui/snippets/ai/artifact_code_display.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui::element::{AnyElement, Length, SemanticsDecoration, SpacerProps};",
        "fn empty_spacer(cx: &mut AppComponentCx<'_>) -> AnyElement",
        "fn status_marker(cx: &mut AppComponentCx<'_>, status_text: Arc<str>) -> AnyElement",
        "role(fret_core::SemanticsRole::Generic)",
        "label(format!(\"Status: {}\", status_text.as_ref()))",
        "test_id(\"ui-ai-artifact-docs-status\")",
        "empty_spacer(cx).attach_semantics",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "artifact_code_display should expose status as a non-text semantics marker; missing `{marker}`"
        );
    }

    for forbidden in [
        "cx.opacity(0.0",
        "cx.text(format!(\"Status: {status_text}\"))",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "artifact_code_display reintroduced hidden bare text status marker: `{forbidden}`"
        );
    }
}

#[test]
fn ai_code_block_demo_visible_text_and_state_marker_use_shared_roles() {
    let source = include_str!("../src/ui/snippets/ai/code_block_demo.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "fn state_marker(cx: &mut AppComponentCx<'_>, test_id: String) -> AnyElement",
        "role: fret_core::SemanticsRole::Generic",
        "cx.spacer(SpacerProps",
        "decl_text::text_section_chrome_label(cx, \"CodeBlock (AI Elements)\")",
        "decl_text::text_paragraph",
        "\"Composable header/title/actions composition aligned with the official AI Elements language-selector example.\"",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "code_block_demo should route visible text and state markers through shared non-bare roles; missing `{marker}`"
        );
    }

    for forbidden in [
        "cx.text(\"\")",
        "cx.text(\"CodeBlock (AI Elements)\")",
        "cx.text(\"Composable header/title/actions composition aligned with the official AI Elements language-selector example.\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "code_block_demo reintroduced bare visible text/state marker text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_sandbox_demo_visible_text_uses_shared_roles() {
    let source = include_str!("../src/ui/snippets/ai/sandbox_demo.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "decl_text::text_section_chrome_label(cx, \"Sandbox console output (demo).\")",
        "decl_text::text_paragraph(cx, \"Apps own execution backends; this is UI-only.\")",
        "decl_text::text_section_chrome_label(cx, \"Sandbox files view (demo).\")",
        "decl_text::text_paragraph(cx, \"Tabs are composable; provide your own panels.\")",
        "decl_text::text_section_chrome_label(cx, \"Sandbox (AI Elements)\")",
        "decl_text::text_paragraph",
        "\"Collapsible + tabs chrome. Apps own the sandbox backend.\"",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "sandbox_demo should route visible text through shared text roles; missing `{marker}`"
        );
    }

    for forbidden in [
        "cx.text(\"Sandbox console output (demo).\")",
        "cx.text(\"Apps own execution backends; this is UI-only.\")",
        "cx.text(\"Sandbox files view (demo).\")",
        "cx.text(\"Tabs are composable; provide your own panels.\")",
        "cx.text(\"Sandbox (AI Elements)\")",
        "cx.text(\"Collapsible + tabs chrome. Apps own the sandbox backend.\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "sandbox_demo reintroduced visible bare text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_queue_demo_visible_text_and_state_marker_use_shared_roles() {
    let source = include_str!("../src/ui/snippets/ai/queue_demo.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "fn state_marker(cx: &mut AppComponentCx<'_>, test_id: &'static str) -> AnyElement",
        "role: fret_core::SemanticsRole::Generic",
        "cx.spacer(SpacerProps",
        "decl_text::text_section_chrome_label(cx, \"Queue (AI Elements)\")",
        "decl_text::text_paragraph",
        "\"Hover an item to reveal actions; actions increment a demo marker.\"",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "queue_demo should route visible text and state markers through shared non-bare roles; missing `{marker}`"
        );
    }

    for forbidden in [
        "unwrap_or_else(|| cx.text(\"\"))",
        "cx.text(\"Queue (AI Elements)\")",
        "cx.text(\"Hover an item to reveal actions; actions increment a demo marker.\")",
        "cx.container(fret_ui::element::ContainerProps::default()",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "queue_demo reintroduced bare visible text/state marker text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_checkpoint_demo_visible_text_uses_shared_roles() {
    let source = include_str!("../src/ui/snippets/ai/checkpoint_demo.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "decl_text::text_control_readout(cx, \"Preview restored to the checkpoint.\")",
        "decl_text::text_control_readout",
        "\"The preview currently shows the latest conversation state.\"",
        "decl_text::text_paragraph(cx, message.content)",
        "decl_text::text_button_label",
        "checkpoint.trigger_label",
        "decl_text::text_paragraph",
        "\"The `Checkpoint` component provides a way to mark specific points in a conversation history and restore the chat to that state.\"",
        "\"Docs-aligned composition: `Conversation` + `Message` + `Checkpoint`. Hover the trigger to preview the tooltip, then activate it to restore the conversation.\"",
        "decl_text::text_chrome_glyph(cx, \"⟲\")",
        "decl_text::text_chrome_glyph(cx, \"•\")",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "checkpoint_demo should route visible text through shared text roles; missing `{marker}`"
        );
    }

    for forbidden in [
        "cx.text(\"Preview restored to the checkpoint.\")",
        "cx.text(\"The preview currently shows the latest conversation state.\")",
        "cx.text(message.content)",
        "cx.text(checkpoint.trigger_label)",
        "cx.text(\"The `Checkpoint` component provides a way to mark specific points in a conversation history and restore the chat to that state.\")",
        "cx.text(\"Docs-aligned composition: `Conversation` + `Message` + `Checkpoint`. Hover the trigger to preview the tooltip, then activate it to restore the conversation.\")",
        "cx.text(\"⟲\")",
        "cx.text(\"•\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "checkpoint_demo reintroduced visible bare text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_simple_chrome_snippets_use_shared_title_and_paragraph_roles() {
    for (name, source, title, body) in [
        (
            "agent_demo",
            include_str!("../src/ui/snippets/ai/agent_demo.rs"),
            "Agent",
            "Composable agent chrome with model, instructions, expandable tool schemas, and structured output.",
        ),
        (
            "code_block_usage",
            include_str!("../src/ui/snippets/ai/code_block_usage.rs"),
            "CodeBlock usage",
            "Minimal compound-parts composition aligned with the official AI Elements usage block.",
        ),
        (
            "environment_variables_demo",
            include_str!("../src/ui/snippets/ai/environment_variables_demo.rs"),
            "Environment Variables (AI Elements)",
            "Toggle to reveal values; copy uses a clipboard effect.",
        ),
        (
            "open_in_chat_demo",
            include_str!("../src/ui/snippets/ai/open_in_chat_demo.rs"),
            "OpenIn (AI Elements)",
            "Selecting a provider emits Effect::OpenUrl (URLs match upstream).",
        ),
    ] {
        let canonical = canonicalize_rust_fragment(source);

        for marker in [
            "use fret_ui_kit::declarative::text as decl_text;",
            "decl_text::text_section_chrome_label",
            title,
            "decl_text::text_paragraph",
            body,
        ] {
            let marker = canonicalize_rust_fragment(marker);
            assert!(
                canonical.contains(&marker),
                "{name} should route fixed visible title/body text through shared roles; missing `{marker}`"
            );
        }

        for forbidden in [
            format!("cx.text(\"{title}\")"),
            format!("cx.text(\"{body}\")"),
        ] {
            let forbidden = canonicalize_rust_fragment(&forbidden);
            assert!(
                !canonical.contains(&forbidden),
                "{name} reintroduced bare fixed visible text: `{forbidden}`"
            );
        }
    }
}

#[test]
fn ai_image_demo_routes_visible_text_through_roles() {
    let source = include_str!("../src/ui/snippets/ai/image_demo.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "decl_text::text_paragraph",
        "Image (AI Elements): presentation surface backed by the shared gallery demo asset bundle.",
        "decl_text::text_control_readout(cx, format!(\"image_ready={}\", image_id.is_some()))",
        "decl_text::text_control_readout(cx, \"Loading image...\")",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "image_demo should route fixed visible text and status/loading readouts through shared roles; missing `{marker}`"
        );
    }

    for forbidden in [
        "cx.text(format!(\"image_ready={}\", image_id.is_some()))",
        "cx.text(\"Loading image...\")",
        "cx.text(\"Image (AI Elements): presentation surface backed by the shared gallery demo asset bundle.\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "image_demo reintroduced bare visible text/readout text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_chain_of_thought_composable_routes_child_text_through_roles() {
    let source = include_str!("../src/ui/snippets/ai/chain_of_thought_composable.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "decl_text::text_section_chrome_label(cx, \"Reasoning trace\")",
        "decl_text::text_section_chrome_label(cx, \"Collect evidence\")",
        "decl_text::text_paragraph",
        "Header text, step labels, and descriptions can all be composed from full child elements.",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "chain_of_thought_composable should route composed child text through shared roles; missing `{marker}`"
        );
    }

    for forbidden in [
        "cx.text(\"Reasoning trace\")",
        "cx.text(\"Collect evidence\")",
        "cx.text(\"Header text, step labels, and descriptions can all be composed from full child elements.\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "chain_of_thought_composable reintroduced bare composed child text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_test_results_composable_routes_custom_child_text_through_roles() {
    let source = include_str!("../src/ui/snippets/ai/test_results_composable.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "decl_text::text_control_readout(cx, \"12 of 15 checks are healthy\")",
        "decl_text::text_control_readout(cx, \"2 failures still need follow-up\")",
        "decl_text::text_list_row_label(cx, \"Authentication\")",
        "decl_text::text_control_readout(cx, \"2 pass / 1 fail\")",
        "decl_text::text_control_readout(cx, \"FAIL\")",
        "decl_text::text_control_readout(cx, \"PASS\")",
        "should reject stale refresh tokens",
        "should rotate keys after password reset",
        "decl_text::text_control_readout(cx, \"85ms cold cache\")",
        "decl_text::text_control_readout(cx, \"41ms warm path\")",
        "decl_text::text_control_readout(cx, \"12 passing\")",
        "decl_text::text_control_readout(cx, \"2 failing\")",
        "decl_text::text_control_readout(cx, \"1 skipped\")",
        "decl_text::text_control_readout(cx, \"3.25s wall time\")",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "test_results_composable should route fixed row/readout child text through shared roles; missing `{marker}`"
        );
    }

    assert!(
        canonical.contains(&canonicalize_rust_fragment(
            "decl_text::text_list_row_label"
        )),
        "test_results_composable should use list-row labels for fixed suite/test names"
    );

    for forbidden in [
        "cx.text(\"12 of 15 checks are healthy\")",
        "cx.text(\"2 failures still need follow-up\")",
        "cx.text(\"Authentication\")",
        "cx.text(\"2 pass / 1 fail\")",
        "cx.text(\"FAIL\")",
        "cx.text(\"should reject stale refresh tokens\")",
        "cx.text(\"85ms cold cache\")",
        "cx.text(\"PASS\")",
        "cx.text(\"should rotate keys after password reset\")",
        "cx.text(\"41ms warm path\")",
        "cx.text(\"12 passing\")",
        "cx.text(\"2 failing\")",
        "cx.text(\"1 skipped\")",
        "cx.text(\"3.25s wall time\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "test_results_composable reintroduced bare fixed row/readout text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_workflow_snippets_route_fixed_text_through_roles() {
    let snippets: &[(&str, &str, &[&str], &[&str])] = &[
        (
            "workflow_panel_demo",
            include_str!("../src/ui/snippets/ai/workflow_panel_demo.rs"),
            &[
                "use fret_ui_kit::declarative::text as decl_text;",
                "decl_text::text_section_chrome_label(cx, \"WorkflowPanel (AI Elements)\")",
                "decl_text::text_compact_paragraph",
                "Container chrome only. Apps own placement + interactions.",
                "WorkflowPanel (AI Elements): bordered container chrome.",
            ],
            &[
                "cx.text(\"WorkflowPanel (AI Elements)\")",
                "cx.text(\"Container chrome only. Apps own placement + interactions.\")",
                "cx.text(\"WorkflowPanel (AI Elements): bordered container chrome.\")",
            ],
        ),
        (
            "workflow_toolbar_demo",
            include_str!("../src/ui/snippets/ai/workflow_toolbar_demo.rs"),
            &[
                "use fret_ui_kit::declarative::text as decl_text;",
                "decl_text::text_section_chrome_label",
                "WorkflowToolbar (AI Elements): compact tool row chrome.",
            ],
            &["cx.text(\"WorkflowToolbar (AI Elements): compact tool row chrome.\")"],
        ),
        (
            "workflow_controls_demo",
            include_str!("../src/ui/snippets/ai/workflow_controls_demo.rs"),
            &[
                "use fret_ui_kit::declarative::text as decl_text;",
                "decl_text::text_control_readout(cx, format!(\"clicks={clicks}\"))",
                "WorkflowControls (AI Elements): button stack chrome.",
            ],
            &[
                "cx.text(format!(\"clicks={clicks}\"))",
                "cx.text(\"WorkflowControls (AI Elements): button stack chrome.\")",
            ],
        ),
        (
            "workflow_canvas_demo",
            include_str!("../src/ui/snippets/ai/workflow_canvas_demo.rs"),
            &[
                "use fret_ui_kit::declarative::text as decl_text;",
                "WorkflowCanvas (AI Elements): pan/zoom host + overlay slot.",
            ],
            &["cx.text(\"WorkflowCanvas (AI Elements): pan/zoom host + overlay slot.\")"],
        ),
        (
            "workflow_connection_demo",
            include_str!("../src/ui/snippets/ai/workflow_connection_demo.rs"),
            &[
                "use fret_ui_kit::declarative::text as decl_text;",
                "WorkflowConnection (AI Elements): in-progress connection line chrome.",
            ],
            &["cx.text(\"WorkflowConnection (AI Elements): in-progress connection line chrome.\")"],
        ),
        (
            "workflow_edge_demo",
            include_str!("../src/ui/snippets/ai/workflow_edge_demo.rs"),
            &[
                "use fret_ui_kit::declarative::text as decl_text;",
                "WorkflowEdge (AI Elements): dashed + animated stroke renderers.",
            ],
            &["cx.text(\"WorkflowEdge (AI Elements): dashed + animated stroke renderers.\")"],
        ),
        (
            "workflow_node_demo",
            include_str!("../src/ui/snippets/ai/workflow_node_demo.rs"),
            &[
                "use fret_ui_kit::declarative::text as decl_text;",
                "decl_text::text_compact_paragraph",
                "Node content slot: apps own interaction + state.",
                "Use handles as a styling seam (not an engine).",
                "decl_text::text_control_readout(cx, \"Footer slot\")",
                "WorkflowNode (AI Elements): header/content/footer chrome.",
            ],
            &[
                "cx.text(\"Node content slot: apps own interaction + state.\")",
                "cx.text(\"Use handles as a styling seam (not an engine).\")",
                "cx.text(\"Footer slot\")",
                "cx.text(\"WorkflowNode (AI Elements): header/content/footer chrome.\")",
            ],
        ),
        (
            "workflow_chrome_demo",
            include_str!("../src/ui/snippets/ai/workflow_chrome_demo.rs"),
            &[
                "use fret_ui_kit::declarative::text as decl_text;",
                "decl_text::text_compact_paragraph",
                "Node content is app-owned; this is the shadcn-aligned chrome surface.",
                "decl_text::text_control_readout",
                "Footer area (optional).",
                "Workflow panel (chrome-only).",
                "Apps own node/canvas engines and interaction policy.",
                "Workflow chrome (AI Elements)",
                "UI-only ports of @xyflow/react wrappers (Panel/Toolbar).",
            ],
            &[
                "cx.text(\"Node content is app-owned; this is the shadcn-aligned chrome surface.\")",
                "cx.text(\"Footer area (optional).\")",
                "cx.text(\"Workflow panel (chrome-only).\")",
                "cx.text(\"Apps own node/canvas engines and interaction policy.\")",
                "cx.text(\"Workflow chrome (AI Elements)\")",
                "cx.text(\"UI-only ports of @xyflow/react wrappers (Panel/Toolbar).\")",
            ],
        ),
        (
            "workflow_node_graph_demo",
            include_str!("../src/ui/snippets/ai/workflow_node_graph_demo.rs"),
            &[
                "use fret_ui_kit::declarative::text as decl_text;",
                "Workflow editor (engine-backed)",
                "Uses fret-node for graph interaction + fret-ui-ai for chrome wrappers.",
            ],
            &[
                "cx.text(\"Workflow editor (engine-backed)\")",
                "cx.text(\"Uses fret-node for graph interaction + fret-ui-ai for chrome wrappers.\")",
            ],
        ),
    ];

    for (name, source, required, forbidden) in snippets {
        let canonical = canonicalize_rust_fragment(source);
        for marker in *required {
            let marker = canonicalize_rust_fragment(marker);
            assert!(
                canonical.contains(&marker),
                "{name} should route fixed workflow text through shared roles; missing `{marker}`"
            );
        }
        for marker in *forbidden {
            let marker = canonicalize_rust_fragment(marker);
            assert!(
                !canonical.contains(&marker),
                "{name} reintroduced bare workflow text: `{marker}`"
            );
        }
    }
}

#[test]
fn ai_selector_branch_snippets_use_shared_text_roles_and_non_text_markers() {
    for (name, source, title, body) in [
        (
            "message_branch_demo",
            include_str!("../src/ui/snippets/ai/message_branch_demo.rs"),
            "MessageBranch (AI Elements)",
            "Prev/Next cycles through branches; only active branch is mounted.",
        ),
        (
            "mic_selector_demo",
            include_str!("../src/ui/snippets/ai/mic_selector_demo.rs"),
            "MicSelector (AI Elements)",
            "Docs-shaped compound example with typed item rows. Device inventory + permissions stay app-owned.",
        ),
        (
            "model_selector_demo",
            include_str!("../src/ui/snippets/ai/model_selector_demo.rs"),
            "ModelSelector (AI Elements)",
            "Dialog + Command surfaces; selection is app-owned.",
        ),
    ] {
        let canonical = canonicalize_rust_fragment(source);

        for marker in [
            "use fret_ui_kit::declarative::text as decl_text;",
            "fn state_marker",
            "SemanticsRole::Generic",
            "cx.spacer(SpacerProps",
            "decl_text::text_section_chrome_label",
            title,
            "decl_text::text_paragraph",
            body,
        ] {
            let marker = canonicalize_rust_fragment(marker);
            assert!(
                canonical.contains(&marker),
                "{name} should route fixed visible text and state markers through shared non-bare roles; missing `{marker}`"
            );
        }

        for forbidden in [
            "cx.text(\"\")",
            "role: fret_core::SemanticsRole::Text",
            &format!("cx.text(\"{title}\")"),
            &format!("cx.text(\"{body}\")"),
        ] {
            let forbidden = canonicalize_rust_fragment(forbidden);
            assert!(
                !canonical.contains(&forbidden),
                "{name} reintroduced bare visible text/state marker text: `{forbidden}`"
            );
        }
    }
}

#[test]
fn ai_prompt_and_plan_snippets_use_shared_outer_text_roles() {
    for (name, source, title, body) in [
        (
            "plan_demo",
            include_str!("../src/ui/snippets/ai/plan_demo.rs"),
            "Plan (AI Elements)",
            "Toggle the chevron button to expand/collapse.",
        ),
        (
            "prompt_input_action_menu_demo",
            include_str!("../src/ui/snippets/ai/prompt_input_action_menu_demo.rs"),
            "Prompt Input Action Menu (AI Elements)",
            "Use the + menu to add attachments.",
        ),
        (
            "prompt_input_tooltip_demo",
            include_str!("../src/ui/snippets/ai/prompt_input_tooltip_demo.rs"),
            "Prompt Input Button Tooltips (AI Elements)",
            "Hover the toolbar actions to preview a simple tooltip, a shortcut hint, and a bottom-positioned tooltip.",
        ),
    ] {
        let canonical = canonicalize_rust_fragment(source);

        for marker in [
            "use fret_ui_kit::declarative::text as decl_text;",
            "decl_text::text_section_chrome_label",
            title,
            "decl_text::text_paragraph",
            body,
        ] {
            let marker = canonicalize_rust_fragment(marker);
            assert!(
                canonical.contains(&marker),
                "{name} should route fixed outer title/body text through shared roles; missing `{marker}`"
            );
        }

        for forbidden in [
            format!("cx.text(\"{title}\")"),
            format!("cx.text(\"{body}\")"),
        ] {
            let forbidden = canonicalize_rust_fragment(&forbidden);
            assert!(
                !canonical.contains(&forbidden),
                "{name} reintroduced bare outer visible text: `{forbidden}`"
            );
        }
    }
}

#[test]
fn ai_commit_large_uses_shared_outer_text_roles_and_non_text_marker() {
    let source = include_str!("../src/ui/snippets/ai/commit_large_demo.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "fn state_marker(cx: &mut AppComponentCx<'_>, test_id: &'static str) -> AnyElement",
        "SemanticsRole::Generic",
        "cx.spacer(SpacerProps",
        "opened_now.is_some().then(|| state_marker(cx, \"ui-ai-commit-large-opened-marker\"))",
        "decl_text::text_section_chrome_label(cx, \"Commit (Large)\")",
        "decl_text::text_paragraph",
        "\"Scroll-heavy surface for hit testing + viewport scrolling.\"",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "commit_large_demo should route fixed visible text and state marker through shared non-bare roles; missing `{marker}`"
        );
    }

    for forbidden in [
        "role: fret_core::SemanticsRole::Text",
        "cx.text(\"\")",
        "cx.text(\"Commit (Large)\")",
        "cx.text(\"Scroll-heavy surface for hit testing + viewport scrolling.\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "commit_large_demo reintroduced bare visible text/state marker text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_large_stack_and_test_results_use_shared_text_roles_and_non_text_markers() {
    for (name, source, title, body, marker_id) in [
        (
            "stack_trace_large_demo",
            include_str!("../src/ui/snippets/ai/stack_trace_large_demo.rs"),
            "StackTrace (Large)",
            "Scroll in the frames viewport and click a file path.",
            "ui-ai-stack-trace-large-opened-marker",
        ),
        (
            "test_results_large_demo",
            include_str!("../src/ui/snippets/ai/test_results_large_demo.rs"),
            "Test Results Large (AI Elements)",
            "Scroll the page and click a deep row to set a marker.",
            "ui-ai-test-results-large-activated-marker",
        ),
    ] {
        let canonical = canonicalize_rust_fragment(source);

        for marker in [
            "use fret_ui_kit::declarative::text as decl_text;",
            "fn state_marker",
            "SemanticsRole::Generic",
            "cx.spacer(SpacerProps",
            marker_id,
            "decl_text::text_section_chrome_label",
            title,
            "decl_text::text_paragraph",
            body,
        ] {
            let marker = canonicalize_rust_fragment(marker);
            assert!(
                canonical.contains(&marker),
                "{name} should route fixed visible text and state markers through shared non-bare roles; missing `{marker}`"
            );
        }

        for forbidden in [
            "role: fret_core::SemanticsRole::Text",
            "cx.text(\"\")",
            &format!("cx.text(\"{title}\")"),
            &format!("cx.text(\"{body}\")"),
        ] {
            let forbidden = canonicalize_rust_fragment(forbidden);
            assert!(
                !canonical.contains(&forbidden),
                "{name} reintroduced bare visible text/state marker text: `{forbidden}`"
            );
        }
    }
}

#[test]
fn ai_tool_and_suggestions_use_shared_text_roles_and_non_text_markers() {
    for (name, source, title, body, marker_id) in [
        (
            "tool_demo",
            include_str!("../src/ui/snippets/ai/tool_demo.rs"),
            "Tool (AI Elements)",
            "Docs-shaped compound composition with the four official Tool states.",
            "ui-ai-tool-demo-content-marker",
        ),
        (
            "suggestions_demo",
            include_str!("../src/ui/snippets/ai/suggestions_demo.rs"),
            "Suggestions (AI Elements)",
            "Suggestion pills emit intents; apps own prompt insertion.",
            "ui-ai-suggestions-clicked-marker",
        ),
    ] {
        let canonical = canonicalize_rust_fragment(source);

        for marker in [
            "use fret_ui_kit::declarative",
            "text as decl_text",
            "fn state_marker",
            "SemanticsRole::Generic",
            "cx.spacer(SpacerProps",
            marker_id,
            "decl_text::text_section_chrome_label",
            title,
            "decl_text::text_paragraph",
            body,
        ] {
            let marker = canonicalize_rust_fragment(marker);
            assert!(
                canonical.contains(&marker),
                "{name} should route fixed visible text and state markers through shared non-bare roles; missing `{marker}`"
            );
        }

        for forbidden in [
            "role: SemanticsRole::Text",
            "role: fret_core::SemanticsRole::Text",
            "cx.text(\"\")",
            &format!("cx.text(\"{title}\")"),
            &format!("cx.text(\"{body}\")"),
        ] {
            let forbidden = canonicalize_rust_fragment(forbidden);
            assert!(
                !canonical.contains(&forbidden),
                "{name} reintroduced bare visible text/state marker text: `{forbidden}`"
            );
        }
    }

    let tool = canonicalize_rust_fragment(include_str!("../src/ui/snippets/ai/tool_demo.rs"));
    for section in [
        "Input Streaming (Pending)",
        "Input Available (Running)",
        "Output Available (Completed)",
        "Output Error",
    ] {
        let role_marker = canonicalize_rust_fragment(&format!(
            "decl_text::text_section_chrome_label(cx, \"{section}\")"
        ));
        assert!(
            tool.contains(&role_marker),
            "tool_demo should route `{section}` section labels through shared section chrome roles"
        );
        let bare_marker = canonicalize_rust_fragment(&format!("cx.text(\"{section}\")"));
        assert!(
            !tool.contains(&bare_marker),
            "tool_demo reintroduced bare section label text: `{bare_marker}`"
        );
    }

    let suggestions =
        canonicalize_rust_fragment(include_str!("../src/ui/snippets/ai/suggestions_demo.rs"));
    for marker in [
        "decl_text::text_button_label(cx, \"Summarize the release notes\")",
        "decl_text::text_button_label(cx, \"Draft a Tokyo travel brief\")",
        "decl_text::text_paragraph",
        "Composable children let callers add icons or extra inline structure while the suggestion payload stays app-owned.",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            suggestions.contains(&marker),
            "suggestions_demo should route custom children labels/body text through shared roles; missing `{marker}`"
        );
    }
    for forbidden in [
        "cx.text(\"Summarize the release notes\")",
        "cx.text(\"Draft a Tokyo travel brief\")",
        "cx.text(\"Composable children let callers add icons or extra inline structure while the suggestion payload stays app-owned.\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !suggestions.contains(&forbidden),
            "suggestions_demo reintroduced bare custom children text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_queue_prompt_input_and_transcription_use_shared_text_roles_and_non_text_markers() {
    for (name, source, title, body, marker_id) in [
        (
            "queue_prompt_input_demo",
            include_str!("../src/ui/snippets/ai/queue_prompt_input_demo.rs"),
            "Queue + PromptInput (AI Elements)",
            "Docs-aligned composition: content-only QueueSection above PromptInput tools.",
            "ui-ai-queue-prompt-input-sent-count-1",
        ),
        (
            "transcription_demo",
            include_str!("../src/ui/snippets/ai/transcription_demo.rs"),
            "App-owned timeline + interactive transcript",
            "Drag the scrubber or click a segment to seek. The transcript consumes app-owned current_time just like the official AI Elements example.",
            "ui-ai-transcription-demo-time-zero",
        ),
    ] {
        let canonical = canonicalize_rust_fragment(source);

        for marker in [
            "use fret_ui_kit::declarative::text as decl_text;",
            "fn state_marker",
            "SemanticsRole::Generic",
            "cx.spacer(SpacerProps",
            marker_id,
            "decl_text::text_section_chrome_label",
            title,
            "decl_text::text_paragraph",
            body,
        ] {
            let marker = canonicalize_rust_fragment(marker);
            assert!(
                canonical.contains(&marker),
                "{name} should route fixed visible text and state markers through shared non-bare roles; missing `{marker}`"
            );
        }

        for forbidden in [
            "role: fret_core::SemanticsRole::Text",
            "cx.text(\"\")",
            &format!("cx.text(\"{title}\")"),
            &format!("cx.text(\"{body}\")"),
        ] {
            let forbidden = canonicalize_rust_fragment(forbidden);
            assert!(
                !canonical.contains(&forbidden),
                "{name} reintroduced bare visible text/state marker text: `{forbidden}`"
            );
        }
    }

    let queue_prompt = canonicalize_rust_fragment(include_str!(
        "../src/ui/snippets/ai/queue_prompt_input_demo.rs"
    ));
    for marker in [
        "decl_text::text_button_label(cx, \"Search\")",
        "state_marker(cx, \"ui-ai-queue-prompt-input-sent-count-1\")",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            queue_prompt.contains(&marker),
            "queue_prompt_input_demo should route custom prompt-button text and sent marker through shared roles; missing `{marker}`"
        );
    }
    for forbidden in ["ui::text(\"Search\")", "cx.text(\"Search\")"] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !queue_prompt.contains(&forbidden),
            "queue_prompt_input_demo reintroduced a bare custom prompt-button label: `{forbidden}`"
        );
    }

    let transcription =
        canonicalize_rust_fragment(include_str!("../src/ui/snippets/ai/transcription_demo.rs"));
    for marker in [
        "ui-ai-transcription-demo-time-nonzero",
        "ui-ai-transcription-demo-active-{active_index}",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            transcription.contains(&marker),
            "transcription_demo should preserve diagnostics marker ids while keeping them non-text; missing `{marker}`"
        );
    }
}

#[test]
fn ai_web_preview_uses_shared_text_roles_and_non_text_markers() {
    let source = include_str!("../src/ui/snippets/ai/web_preview_demo.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "fn state_marker(cx: &mut AppComponentCx<'_>, test_id: &'static str) -> AnyElement",
        "role: fret_core::SemanticsRole::Generic",
        "cx.spacer(SpacerProps",
        "state_marker(cx, \"ui-ai-web-preview-demo-loading-false\")",
        "state_marker(cx, \"ui-ai-web-preview-demo-committed-true\")",
        "state_marker(cx, \"ui-ai-web-preview-demo-can-back-true\")",
        "state_marker(cx, \"ui-ai-web-preview-demo-can-forward-true\")",
        "state_marker(cx, \"ui-ai-web-preview-demo-can-forward-false\")",
        "decl_text::text_chrome_glyph(cx, \"←\")",
        "decl_text::text_chrome_glyph(cx, \"→\")",
        "decl_text::text_chrome_glyph(cx, \"↺\")",
        "decl_text::text_section_chrome_label(cx, \"Custom body content\")",
        "decl_text::text_paragraph",
        "\"Use this lane when preview chrome is enough for the current build.\"",
        "decl_text::text_section_chrome_label(cx, \"Custom console footer\")",
        "decl_text::text_paragraph(cx, \"Backend navigation is app-owned and optional in Fret.\")",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "web_preview_demo should route fixed visible text and state markers through shared non-bare roles; missing `{marker}`"
        );
    }

    for forbidden in [
        "role: fret_core::SemanticsRole::Text",
        "cx.text(\"\")",
        "cx.text(\"←\")",
        "cx.text(\"→\")",
        "cx.text(\"↺\")",
        "cx.text(\"Custom body content\")",
        "cx.text(\"Use this lane when preview chrome is enough for the current build.\")",
        "cx.text(\"Custom console footer\")",
        "cx.text(\"Backend navigation is app-owned and optional in Fret.\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "web_preview_demo reintroduced bare visible text/state marker text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_reasoning_hooks_and_transcript_torture_use_shared_text_roles() {
    let reasoning_hooks =
        canonicalize_rust_fragment(include_str!("../src/ui/snippets/ai/reasoning_hooks.rs"));
    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "decl_text::text_control_readout",
        "Reasoning controller unavailable",
        "decl_text::text_control_readout(cx, status)",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            reasoning_hooks.contains(&marker),
            "reasoning_hooks should route custom trigger status text through shared readout roles; missing `{marker}`"
        );
    }
    for forbidden in [
        "cx.text(\"Reasoning controller unavailable\")",
        "cx.text(status)",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !reasoning_hooks.contains(&forbidden),
            "reasoning_hooks reintroduced bare status text: `{forbidden}`"
        );
    }

    let transcript =
        canonicalize_rust_fragment(include_str!("../src/ui/snippets/ai/transcript_torture.rs"));
    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "decl_text::text_section_chrome_label",
        "Goal: baseline harness for long AI transcripts (scrolling + virtualization + caching).",
        "decl_text::text_paragraph",
        "Use scripted wheel-scroll to validate view-cache reuse stability and stale-paint safety.",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            transcript.contains(&marker),
            "transcript_torture should route fixed header copy through shared roles; missing `{marker}`"
        );
    }
    for forbidden in [
        "cx.text(\"Goal: baseline harness for long AI transcripts (scrolling + virtualization + caching).\")",
        "cx.text(\"Use scripted wheel-scroll to validate view-cache reuse stability and stale-paint safety.\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !transcript.contains(&forbidden),
            "transcript_torture reintroduced bare header text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_custom_children_snippets_use_shared_text_roles() {
    let environment = canonicalize_rust_fragment(include_str!(
        "../src/ui/snippets/ai/environment_variables_custom_children.rs"
    ));
    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "decl_text::text_section_chrome_label(cx, \"Runtime Secrets\")",
        "decl_text::text_code_label(cx, \"Primary API Key\")",
        "decl_text::text_control_readout(cx, \"Secret\")",
        "decl_text::text_code_label",
        "App-owned masked preview",
        "decl_text::text_paragraph(cx, \"Custom children take ownership of the visible content.\")",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            environment.contains(&marker),
            "environment_variables_custom_children should route custom child text through shared roles; missing `{marker}`"
        );
    }
    for forbidden in [
        "cx.text(\"Runtime Secrets\")",
        "cx.text(\"Primary API Key\")",
        "cx.text(\"Secret\")",
        "cx.text(\"App-owned masked preview\")",
        "cx.text(\"Custom children take ownership of the visible content.\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !environment.contains(&forbidden),
            "environment_variables_custom_children reintroduced bare text: `{forbidden}`"
        );
    }

    let package =
        canonicalize_rust_fragment(include_str!("../src/ui/snippets/ai/package_info_demo.rs"));
    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "decl_text::text_code_label",
        "pkg/react",
        "decl_text::text_button_label(cx, \"Breaking\")",
        "18.2.0 -> 19.0.0 (custom)",
        "decl_text::text_paragraph",
        "Custom summary supplied by the app.",
        "decl_text::text_code_label(cx, \"react-dom @ ^19.0.0\")",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            package.contains(&marker),
            "package_info_demo should route custom package text through shared roles; missing `{marker}`"
        );
    }
    for forbidden in [
        "cx.text(\"pkg/react\")",
        "cx.text(\"Breaking\")",
        "cx.text(\"18.2.0 -> 19.0.0 (custom)\")",
        "cx.text(\"Custom summary supplied by the app.\")",
        "cx.text(\"react-dom @ ^19.0.0\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !package.contains(&forbidden),
            "package_info_demo reintroduced bare custom package text: `{forbidden}`"
        );
    }

    let inline = canonicalize_rust_fragment(include_str!(
        "../src/ui/snippets/ai/inline_citation_demo.rs"
    ));
    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "decl_text::text_paragraph",
        "The technology continues to evolve rapidly, with new breakthroughs being announced regularly",
        "According to recent studies, artificial intelligence has shown remarkable progress in natural language processing.",
        "decl_text::text_paragraph(cx, \".\")",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            inline.contains(&marker),
            "inline_citation_demo should route citation prose through shared paragraph roles; missing `{marker}`"
        );
    }
    for forbidden in [
        "cx.text(\"The technology continues to evolve rapidly, with new breakthroughs being announced regularly\")",
        "cx.text(\"According to recent studies, artificial intelligence has shown remarkable progress in natural language processing.\")",
        "cx.text(\".\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !inline.contains(&forbidden),
            "inline_citation_demo reintroduced bare citation text: `{forbidden}`"
        );
    }

    let persona = canonicalize_rust_fragment(include_str!("../src/ui/snippets/ai/persona_demo.rs"));
    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "decl_text::text_button_label(cx, variant.label())",
        "decl_text::text_control_readout",
        "ui-ai-persona-demo-current-label",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            persona.contains(&marker),
            "persona_demo should route toggle labels/readouts through shared roles; missing `{marker}`"
        );
    }
    for forbidden in ["cx.text(variant.label())", "cx.text(format!("] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !persona.contains(&forbidden),
            "persona_demo reintroduced bare persona text: `{forbidden}`"
        );
    }

    let persona_custom = canonicalize_rust_fragment(include_str!(
        "../src/ui/snippets/ai/persona_custom_visual.rs"
    ));
    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "decl_text::text_control_readout(cx, \"Command\")",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            persona_custom.contains(&marker),
            "persona_custom_visual should route custom center label through shared roles; missing `{marker}`"
        );
    }
    let forbidden = canonicalize_rust_fragment("cx.text(\"Command\")");
    assert!(
        !persona_custom.contains(&forbidden),
        "persona_custom_visual reintroduced bare custom center text: `{forbidden}`"
    );

    let sources =
        canonicalize_rust_fragment(include_str!("../src/ui/snippets/ai/sources_custom_demo.rs"));
    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "decl_text::text_list_row_label(cx, title)",
        "decl_text::text_button_label(cx, format!(\"Using {count} citations\"))",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            sources.contains(&marker),
            "sources_custom_demo should route custom source labels through shared roles; missing `{marker}`"
        );
    }
    for forbidden in [
        "cx.text(title)",
        "cx.text(format!(\"Using {count} citations\"))",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !sources.contains(&forbidden),
            "sources_custom_demo reintroduced bare source text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_chat_demo_uses_shared_outer_text_roles_and_non_text_markers() {
    let source = include_str!("../src/ui/snippets/ai/chat_demo.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "fn empty_spacer(cx: &mut AppComponentCx<'_>) -> AnyElement",
        "fn state_marker(cx: &mut AppComponentCx<'_>, test_id: &'static str) -> AnyElement",
        "role: fret_core::SemanticsRole::Generic",
        "cx.spacer(SpacerProps",
        "state_marker(cx, \"ui-gallery-ai-chat-prompt-nonempty\")",
        "decl_text::text_paragraph",
        "\"Goal: interactive demo for PromptInput + transcript append.\"",
        "\"Send triggers a short \\\"loading\\\" window where Stop is available.\"",
        "decl_text::text_control_readout",
        "format!(\"Exported markdown: {len} chars\")",
        "test_id: Some(Arc::<str>::from(\"ui-gallery-ai-chat-exported-md-len\"))",
        "prompt_non_empty_marker.unwrap_or_else(|| empty_spacer(cx))",
        "exported.unwrap_or_else(|| empty_spacer(cx))",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "chat_demo should route fixed outer text, readouts, and state markers through shared roles; missing `{marker}`"
        );
    }

    for forbidden in [
        "role: fret_core::SemanticsRole::Text, test_id: Some(Arc::<str>::from(\"ui-gallery-ai-chat-prompt-nonempty\"))",
        "role: fret_core::SemanticsRole::Text, test_id: Some(Arc::<str>::from(\"ui-gallery-ai-chat-exported-md-len\"))",
        "cx.text(\"Goal: interactive demo for PromptInput + transcript append.\")",
        "cx.text(\"Send triggers a short \\\"loading\\\" window where Stop is available.\")",
        "vec![cx.text(format!(\"Exported markdown: {len} chars\"))]",
        "prompt_non_empty_marker.unwrap_or_else(|| cx.text(\"\"))",
        "exported.unwrap_or_else(|| cx.text(\"\"))",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "chat_demo reintroduced bare outer text/readout/marker text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_prompt_input_provider_and_docs_use_shared_text_roles_and_non_text_markers() {
    for (name, source, title, body) in [
        (
            "prompt_input_provider_demo",
            include_str!("../src/ui/snippets/ai/prompt_input_provider_demo.rs"),
            "Prompt Input Provider (AI Elements)",
            "External add mutates the provider attachments; send clears attachments.",
        ),
        (
            "prompt_input_docs_demo",
            include_str!("../src/ui/snippets/ai/prompt_input_docs_demo.rs"),
            "Prompt Input (AI Elements)",
            "Docs-aligned chat example: transcript + prompt composer, add attachments/screenshot actions, model picker, and upstream-like onSubmit(message).",
        ),
        (
            "prompt_input_referenced_sources_demo",
            include_str!("../src/ui/snippets/ai/prompt_input_referenced_sources_demo.rs"),
            "Prompt Input Referenced Sources (AI Elements)",
            "Add a source and remove it via the chip's hover affordance.",
        ),
    ] {
        let canonical = canonicalize_rust_fragment(source);

        for marker in [
            "use fret_ui_kit::declarative::text as decl_text;",
            "decl_text::text_section_chrome_label",
            title,
            "decl_text::text_paragraph",
            body,
        ] {
            let marker = canonicalize_rust_fragment(marker);
            assert!(
                canonical.contains(&marker),
                "{name} should route fixed title/body text through shared text roles; missing `{marker}`"
            );
        }

        for forbidden in [
            format!("cx.text(\"{title}\")"),
            format!("cx.text(\"{body}\")"),
        ] {
            let forbidden = canonicalize_rust_fragment(&forbidden);
            assert!(
                !canonical.contains(&forbidden),
                "{name} reintroduced bare fixed visible text: `{forbidden}`"
            );
        }
    }

    let provider = canonicalize_rust_fragment(include_str!(
        "../src/ui/snippets/ai/prompt_input_provider_demo.rs"
    ));
    for marker in [
        "fn state_marker(cx: &mut AppComponentCx<'_>, test_id: &'static str) -> AnyElement",
        "role: fret_core::SemanticsRole::Generic",
        "cx.spacer(SpacerProps",
        "state_marker(cx, \"ui-gallery-ai-prompt-input-provider-sent-count-1\")",
        "decl_text::text_button_label(cx, add_external_label.clone())",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            provider.contains(&marker),
            "prompt_input_provider_demo should route custom label and state marker through shared non-bare roles; missing `{marker}`"
        );
    }
    for forbidden in [
        "cx.text(\"\")",
        "ui::text(add_external_label.clone())",
        "role: fret_core::SemanticsRole::Text",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !provider.contains(&forbidden),
            "prompt_input_provider_demo reintroduced bare custom label or marker text: `{forbidden}`"
        );
    }

    let docs = canonicalize_rust_fragment(include_str!(
        "../src/ui/snippets/ai/prompt_input_docs_demo.rs"
    ));
    for marker in ["decl_text::text_button_label(cx, \"Search\")"] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            docs.contains(&marker),
            "prompt_input_docs_demo should route custom Search button text through shared button-label role"
        );
    }
    for forbidden in ["ui::text(\"Search\")", "cx.text(\"Search\")"] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !docs.contains(&forbidden),
            "prompt_input_docs_demo reintroduced bare custom Search label: `{forbidden}`"
        );
    }
}

#[test]
fn ai_reasoning_stack_trace_and_voice_selector_use_shared_chrome_text_roles() {
    for (name, source, title, body) in [
        (
            "reasoning_demo",
            include_str!("../src/ui/snippets/ai/reasoning_demo.rs"),
            "Reasoning (AI Elements)",
            "Start streaming to auto-open; stop to auto-close.",
        ),
        (
            "stack_trace_demo",
            include_str!("../src/ui/snippets/ai/stack_trace_demo.rs"),
            "StackTrace (AI Elements)",
            "Docs-aligned compound parts API with copy + file-open seams.",
        ),
        (
            "voice_selector_demo",
            include_str!("../src/ui/snippets/ai/voice_selector_demo.rs"),
            "VoiceSelector (AI Elements)",
            "Composable dialog + command recipe. Apps still own inventory and preview playback state.",
        ),
    ] {
        let canonical = canonicalize_rust_fragment(source);

        for marker in [
            "use fret_ui_kit::declarative::text as decl_text;",
            "decl_text::text_section_chrome_label",
            title,
            "decl_text::text_paragraph",
            body,
        ] {
            let marker = canonicalize_rust_fragment(marker);
            assert!(
                canonical.contains(&marker),
                "{name} should route fixed title/body text through shared chrome/prose roles; missing `{marker}`"
            );
        }

        for forbidden in [
            format!("cx.text(\"{title}\")"),
            format!("cx.text(\"{body}\")"),
        ] {
            let forbidden = canonicalize_rust_fragment(&forbidden);
            assert!(
                !canonical.contains(&forbidden),
                "{name} reintroduced bare fixed visible text: `{forbidden}`"
            );
        }
    }

    let stack_trace =
        canonicalize_rust_fragment(include_str!("../src/ui/snippets/ai/stack_trace_demo.rs"));
    for marker in [
        "decl_text::text_control_readout(cx, format!(\"Status: {status_text}\"))",
        "test_id(\"ui-ai-stack-trace-status\")",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            stack_trace.contains(&marker),
            "stack_trace_demo should keep status text on the shared control-readout role; missing `{marker}`"
        );
    }
    let forbidden = canonicalize_rust_fragment("cx.text(format!(\"Status: {status_text}\"))");
    assert!(
        !stack_trace.contains(&forbidden),
        "stack_trace_demo reintroduced bare status readout text: `{forbidden}`"
    );

    let voice_selector =
        canonicalize_rust_fragment(include_str!("../src/ui/snippets/ai/voice_selector_demo.rs"));
    for marker in [
        "decl_text::text_control_readout",
        "selected.as_deref().unwrap_or(\"<none>\")",
        "decl_text::text_control_readout(cx, format!(\"open={open_now}\"))",
        "ui-ai-voice-selector-demo-selected",
        "ui-ai-voice-selector-demo-open-true",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            voice_selector.contains(&marker),
            "voice_selector_demo should keep diagnostics readouts on the shared control-readout role; missing `{marker}`"
        );
    }
    for forbidden in [
        "cx.text(format!(\"selected={}\", selected.as_deref().unwrap_or(\"<none>\")))",
        "cx.text(format!(\"open={open_now}\"))",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !voice_selector.contains(&forbidden),
            "voice_selector_demo reintroduced bare diagnostics readout text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_confirmation_snippets_use_shared_content_text_roles() {
    for (name, source) in [
        (
            "confirmation_demo",
            include_str!("../src/ui/snippets/ai/confirmation_demo.rs"),
        ),
        (
            "confirmation_accepted",
            include_str!("../src/ui/snippets/ai/confirmation_accepted.rs"),
        ),
        (
            "confirmation_rejected",
            include_str!("../src/ui/snippets/ai/confirmation_rejected.rs"),
        ),
    ] {
        let canonical = canonicalize_rust_fragment(source);

        for marker in [
            "use fret_ui_kit::declarative::text as decl_text;",
            "decl_text::text_paragraph(cx, \"This tool wants to delete the file\")",
            "decl_text::text_code_wrap(cx, \"/tmp/example.txt\")",
            "decl_text::text_paragraph(cx, \". Do you approve this action?\")",
            "decl_text::text_control_readout(cx, \"You approved this tool execution\")",
            "decl_text::text_control_readout(cx, \"You rejected this tool execution\")",
        ] {
            let marker = canonicalize_rust_fragment(marker);
            assert!(
                canonical.contains(&marker),
                "{name} should route confirmation request/result content through shared roles; missing `{marker}`"
            );
        }

        for forbidden in [
            "cx.text(\"This tool wants to delete the file\")",
            "shadcn::raw::typography::inline_code(\"/tmp/example.txt\").into_element(cx)",
            "cx.text(\". Do you approve this action?\")",
            "cx.text(\"You approved this tool execution\")",
            "cx.text(\"You rejected this tool execution\")",
        ] {
            let forbidden = canonicalize_rust_fragment(forbidden);
            assert!(
                !canonical.contains(&forbidden),
                "{name} reintroduced bare confirmation content text: `{forbidden}`"
            );
        }
    }

    let demo =
        canonicalize_rust_fragment(include_str!("../src/ui/snippets/ai/confirmation_demo.rs"));
    for marker in [
        "decl_text::text_section_chrome_label(cx, \"Confirmation (AI Elements)\")",
        "decl_text::text_paragraph",
        "\"Docs-aligned tool approval workflow: request a destructive action, confirm or reject it, then inspect the final output state.\"",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            demo.contains(&marker),
            "confirmation_demo should route fixed outer title/body through shared roles; missing `{marker}`"
        );
    }
    for forbidden in [
        "cx.text(\"Confirmation (AI Elements)\")",
        "cx.text(\"Docs-aligned tool approval workflow: request a destructive action, confirm or reject it, then inspect the final output state.\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !demo.contains(&forbidden),
            "confirmation_demo reintroduced bare outer text: `{forbidden}`"
        );
    }

    let request = canonicalize_rust_fragment(include_str!(
        "../src/ui/snippets/ai/confirmation_request.rs"
    ));
    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "decl_text::text_paragraph",
        "\"This tool wants to execute a query on the production database:\"",
        "decl_text::text_code_wrap",
        "SELECT * FROM users WHERE role = 'admin'",
        "decl_text::text_control_readout(cx, \"You approved this tool execution\")",
        "decl_text::text_control_readout(cx, \"You rejected this tool execution\")",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            request.contains(&marker),
            "confirmation_request should route query request/result content through shared roles; missing `{marker}`"
        );
    }
    for forbidden in [
        "cx.text(\"This tool wants to execute a query on the production database:\")",
        "cx.text(\"You approved this tool execution\")",
        "cx.text(\"You rejected this tool execution\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !request.contains(&forbidden),
            "confirmation_request reintroduced bare confirmation content text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_task_demo_uses_shared_content_text_roles() {
    let source = include_str!("../src/ui/snippets/ai/task_demo.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::{icon, style as decl_style, text as decl_text};",
        "decl_text::text_list_row_label(cx, text)",
        "decl_text::text_code_wrap(cx, file_name)",
        "decl_text::text_section_chrome_label(cx, \"Task (AI Elements)\")",
        "decl_text::text_paragraph",
        "\"Collapsible task list demo aligned with the official AI Elements Task structure.\"",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "task_demo should route task rows, file labels, and fixed outer text through shared roles; missing `{marker}`"
        );
    }

    for forbidden in [
        "cx.text(text)",
        "cx.text(file_name)",
        "cx.text(\"Task (AI Elements)\")",
        "cx.text(\"Collapsible task list demo aligned with the official AI Elements Task structure.\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "task_demo reintroduced bare task content text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_conversation_demo_uses_non_text_instrumentation_and_button_label() {
    let source = include_str!("../src/ui/snippets/ai/conversation_demo.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "role: SemanticsRole::Generic",
        "test_id: Some(Arc::<str>::from(\"ui-ai-conversation-demo-exported-md-len\"))",
        "test_id: Some(Arc::<str>::from(\"ui-ai-conversation-demo-messages-len\"))",
        "numeric_value: Some(exported_md_len as f64)",
        "numeric_value: Some(messages.len() as f64)",
        "decl_text::text_button_label(cx, \"Latest\")",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "conversation_demo should keep diagnostics out of text layout and custom button text on the shared role; missing `{marker}`"
        );
    }

    for forbidden in [
        "role: SemanticsRole::Text",
        "role: fret_core::SemanticsRole::Text",
        "cx.text(\"Latest\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "conversation_demo reintroduced text-role diagnostics or bare button text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_usage_snippets_use_shared_chrome_and_paragraph_roles() {
    let attachments =
        canonicalize_rust_fragment(include_str!("../src/ui/snippets/ai/attachments_usage.rs"));
    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "decl_text::text_paragraph",
        "Display uploaded files in a message surface with a shared Attachments container. The image preview comes from the gallery demo asset bundle through a logical asset request so the snippet teaches shipped asset ownership.",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            attachments.contains(&marker),
            "attachments_usage should route fixed explanatory copy through paragraph text; missing `{marker}`"
        );
    }
    for forbidden in [
        "cx.text(\"Display uploaded files in a message surface with a shared Attachments container. The image preview comes from the gallery demo asset bundle through a logical asset request so the snippet teaches shipped asset ownership.\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !attachments.contains(&forbidden),
            "attachments_usage reintroduced bare fixed explanatory text: `{forbidden}`"
        );
    }

    let stack_trace =
        canonicalize_rust_fragment(include_str!("../src/ui/snippets/ai/stack_trace_usage.rs"));
    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "decl_text::text_section_chrome_label(cx, \"StackTrace usage\")",
        "decl_text::text_paragraph",
        "Minimal compound-parts composition aligned with the official AI Elements usage example.",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            stack_trace.contains(&marker),
            "stack_trace_usage should route fixed title/body text through shared roles; missing `{marker}`"
        );
    }
    for forbidden in [
        "cx.text(\"StackTrace usage\")",
        "cx.text(\"Minimal compound-parts composition aligned with the official AI Elements usage example.\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !stack_trace.contains(&forbidden),
            "stack_trace_usage reintroduced bare fixed text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_attachments_inline_hover_card_uses_shared_text_roles() {
    let source = include_str!("../src/ui/snippets/ai/attachments_inline.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "decl_text::text_list_row_label(cx, ui_ai::get_attachment_label(&item))",
        "decl_text::text_control_readout(cx, media_type)",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "attachments_inline should route hover-card label/readout text through shared roles; missing `{marker}`"
        );
    }

    for forbidden in [
        "ui::text(ui_ai::get_attachment_label(&item))",
        "ui::text(media_type).text_xs()",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "attachments_inline reintroduced default wrapping hover-card text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_message_usage_uses_shared_outer_and_user_text_roles() {
    let source = include_str!("../src/ui/snippets/ai/message_usage.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "decl_text::text_paragraph(cx, text.clone())",
        "decl_text::text_control_readout(cx, format!(\"last_action={last_action}\"))",
        "decl_text::text_section_chrome_label(cx, \"Message usage (AI Elements)\")",
        "decl_text::text_paragraph",
        "Docs-aligned composition: Conversation + Message + MessageActions + PromptInput.",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "message_usage should route user content, readout, and fixed outer copy through shared roles; missing `{marker}`"
        );
    }

    for forbidden in [
        "_ => Some(cx.text(text.clone()))",
        "cx.text(format!(\"last_action={last_action}\"))",
        "cx.text(\"Message usage (AI Elements)\")",
        "cx.text(\"Docs-aligned composition: Conversation + Message + MessageActions + PromptInput.\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "message_usage reintroduced bare user/readout/outer text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_canvas_world_spike_routes_visible_text_through_roles() {
    let source = include_str!("../src/ui/snippets/ai/canvas_world_layer_spike.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "decl_text::text_section_chrome_label(cx, \"Canvas world layer (spike)\")",
        "decl_text::text_paragraph",
        "Goal: nodes as element subtrees under a pan/zoom view transform.",
        "format!(\"Clicks: {node_clicks_value}\")",
        "decl_text::text_paragraph(cx, \"Try zooming/panning and click again.\")",
        "decl_text::text_control_readout(cx, \"Layout settled\")",
        "decl_text::text_control_readout(cx, \"Reset done\")",
        "format!(\"Connections: {}\", connections_value.len())",
        "decl_text::text_control_readout(cx, \"Marquee blocked (node hit)\")",
        "format!(\"Selected: {selected_count_value}\")",
        "decl_text::text_control_readout(cx, bounds_text)",
        "decl_text::text_control_readout(cx, debug_view_text)",
        "decl_text::text_control_readout(cx, debug_nodes_text)",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "canvas_world_layer_spike should route visible chrome/readouts through shared text roles; missing `{marker}`"
        );
    }

    for forbidden in [
        "cx.text(\"Canvas world layer (spike)\")",
        "cx.text(\"Goal: nodes as element subtrees under a pan/zoom view transform.\")",
        "cx.text(format!(\"Clicks: {node_clicks_value}\"))",
        "cx.text(\"Try zooming/panning and click again.\")",
        "cx.text(\"Layout settled\")",
        "cx.text(\"Reset done\")",
        "cx.text(format!(\"Connections: {}\", connections_value.len()))",
        "cx.text(\"Marquee blocked (node hit)\")",
        "cx.text(format!(\"Selected: {selected_count_value}\"))",
        "cx.text(bounds_text)",
        "cx.text(debug_view_text)",
        "cx.text(debug_nodes_text)",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "canvas_world_layer_spike reintroduced bare visible text: `{forbidden}`"
        );
    }
}

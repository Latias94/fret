use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::{UiChild, UiCx};

fn states_notes(_cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    crate::ui::doc_layout::notes_block([
        "`idle` is the resting state when the assistant is present but inactive.",
        "`listening` is the active intake state, typically paired with microphone capture or live input.",
        "`thinking` communicates processing / generation work before a response is emitted.",
        "`speaking` is the output state for playback or active response delivery.",
        "`asleep` is the dormant state used when the assistant is intentionally inactive.",
    ])
}

fn props_table(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["API", "Inputs", "Notes"],
        [
            [
                "Persona::new(state)",
                "PersonaState",
                "Required root state; matches the upstream state taxonomy.",
            ],
            [
                "variant(...)",
                "Obsidian | Mana | Opal | Halo | Glint | Command",
                "Visual shell selection; defaults to `Obsidian` like upstream docs.",
            ],
            [
                "size(...)",
                "Px",
                "Typed size override; default is 64px to match upstream `size-16`.",
            ],
            [
                "show_label(...)",
                "bool",
                "Gallery/documentation affordance; remains off by default for upstream-like output.",
            ],
            [
                "refine_layout / refine_style",
                "LayoutRefinement / ChromeRefinement",
                "Typed equivalent to upstream `className` customization.",
            ],
            [
                "children([...])",
                "IntoIterator<Item = AnyElement>",
                "Fret-specific eager custom-visual lane that replaces the default center indicator while keeping the Persona shell intact.",
            ],
            [
                "into_element_with_children(...)",
                "(cx, controller) -> Vec<AnyElement>",
                "Lower-level escape hatch when custom visuals need the current state / variant / size at build time.",
            ],
        ],
        false,
    )
}

fn features_notes(_cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    crate::ui::doc_layout::notes_block([
        "Six visual variants and five high-level states stay aligned with the official AI Elements taxonomy.",
        "Default output remains `obsidian` at 64px so the basic example stays close to upstream `<Persona state=\"...\" />` usage.",
        "Typed `size(...)`, `refine_layout(...)`, and `refine_style(...)` cover the same teaching space as upstream `className` sizing/chrome tweaks.",
        "Fret now also exposes a higher-level `children([...])` lane for custom center visuals, while preserving `into_element_with_children(...)` for controller-aware assembly.",
        "Existing UI Gallery diagnostics already cover preview + state/variant switching for this page, so docs-surface drift is easier to lock down than before.",
    ])
}

fn lifecycle_notes(_cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    crate::ui::doc_layout::notes_block([
        "Upstream AI Elements binds `Persona` to Rive/WebGL2, includes a React Strict Mode guard, and exposes load / ready / play / pause / stop callbacks.",
        "Fret intentionally keeps animation-engine IO and lifecycle ownership app-side, so `fret-ui-ai::Persona` stays a self-drawn shell instead of pretending the Rive callback surface already exists.",
        "Audit result: this is not a `crates/fret-ui` mechanism bug. The remaining drift for `persona` was public-surface and docs-surface parity in `ecosystem/fret-ui-ai` + UI Gallery.",
        "If we later add an app-owned Rive adapter, Strict Mode handling and callback parity should land there as ecosystem/runtime integration work rather than inside the base UI mechanism layer.",
    ])
}

pub(super) fn preview_ai_persona_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let interactive = snippets::persona_demo::render(cx);
    let variants = snippets::persona_variants::render(cx);
    let basic = snippets::persona_basic::render(cx);
    let state_management = snippets::persona_state_management::render(cx);
    let custom_styling = snippets::persona_custom_styling::render(cx);
    let custom_visual = snippets::persona_custom_visual::render(cx);
    let features = features_notes(cx);
    let states = states_notes(cx);
    let props = props_table(cx);
    let lifecycle = lifecycle_notes(cx);

    let body = doc_layout::render_doc_page_after(
        Some(
            "Docs-aligned AI Elements Persona coverage: preview, features/variants/props/states, runtime ownership notes, and copyable usage examples including a Fret-specific custom visual lane.",
        ),
        vec![
            DocSection::build(cx, "Preview", interactive)
                .description("Gallery-specific combined preview: one current persona with state controls plus variant switching so the upstream state/variant matrix is easy to compare on a single surface.")
                .test_id_prefix("ui-gallery-ai-persona-demo")
                .max_w(Px(980.0))
                .code_rust_from_file_region(snippets::persona_demo::SOURCE, "example"),
            DocSection::build(cx, "Features", features)
                .description("High-signal parity notes against the official Persona docs plus the new Fret custom-visual lane.")
                .max_w(Px(980.0))
                .no_shell(),
            DocSection::build(cx, "Variants", variants)
                .description("Six variant shells with the same idle state so visual drift is easy to compare at a glance.")
                .max_w(Px(980.0))
                .code_rust_from_file_region(snippets::persona_variants::SOURCE, "example"),
            DocSection::build(cx, "Props & Extensions", props)
                .description("Upstream-facing props plus the Fret-specific custom-visual surfaces.")
                .max_w(Px(980.0))
                .no_shell(),
            DocSection::build(cx, "States", states)
                .description("Persona responds to the same five high-level states described in the upstream docs.")
                .max_w(Px(980.0))
                .no_shell(),
            DocSection::build(cx, "Lifecycle & Ownership", lifecycle)
                .description("Why lifecycle/Strict-Mode parity is intentionally deferred to a future app-owned animation adapter.")
                .max_w(Px(980.0))
                .no_shell(),
            DocSection::build(cx, "Basic Usage", basic)
                .description("Minimal surface aligned to the upstream basic example: a single Persona with explicit state + variant.")
                .max_w(Px(980.0))
                .code_rust_from_file_region(snippets::persona_basic::SOURCE, "example"),
            DocSection::build(cx, "With State Management", state_management)
                .description("Copyable state-driven example matching the official icon-button control pattern while staying backend-agnostic in Rust.")
                .max_w(Px(980.0))
                .code_rust_from_file_region(snippets::persona_state_management::SOURCE, "example"),
            DocSection::build(cx, "Custom Styling", custom_styling)
                .description("Typed equivalent to the upstream `className=\"size-64 rounded-full border border-border\"` customization example.")
                .max_w(Px(980.0))
                .code_rust_from_file_region(snippets::persona_custom_styling::SOURCE, "example"),
            DocSection::build(cx, "Custom Visual Slot", custom_visual)
                .description("Fret-specific extension: prefer `.children([...])` for eager custom center content, and keep `into_element_with_children(...)` for controller-aware assembly.")
                .max_w(Px(980.0))
                .code_rust_from_file_region(snippets::persona_custom_visual::SOURCE, "example"),
        ],
        cx,
    );

    vec![body.into_element(cx)]
}

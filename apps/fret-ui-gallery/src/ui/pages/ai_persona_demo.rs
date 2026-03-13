use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::ai as snippets;
use fret::{UiChild, UiCx};
use fret_ui_kit::ui::UiElementSinkExt as _;

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
    let row = |api: &'static str, inputs: &'static str, notes: &'static str| {
        shadcn::TableRow::build(3, move |cx, out| {
            out.push_ui(cx, shadcn::TableCell::build(ui::text(api)));
            out.push_ui(cx, shadcn::TableCell::build(ui::text(inputs)));
            out.push_ui(cx, shadcn::TableCell::build(ui::text(notes)));
        })
    };

    shadcn::Table::build(|cx, out| {
        out.push_ui(
            cx,
            shadcn::TableHeader::build(|cx, out| {
                out.push(
                    shadcn::TableRow::build(3, |cx, out| {
                        out.push(shadcn::TableHead::new("API").into_element(cx));
                        out.push(shadcn::TableHead::new("Inputs").into_element(cx));
                        out.push(shadcn::TableHead::new("Notes").into_element(cx));
                    })
                    .into_element(cx),
                );
            }),
        );
        out.push_ui(
            cx,
            shadcn::TableBody::build(|cx, out| {
                out.push_ui(cx, row("Persona::new(state)", "PersonaState", "Required root state; matches the upstream state taxonomy."));
                out.push_ui(cx, row("variant(...)", "Obsidian | Mana | Opal | Halo | Glint | Command", "Visual shell selection; defaults to `Obsidian` like upstream docs."));
                out.push_ui(cx, row("size(...)", "Px", "Typed size override; default is 64px to match upstream `size-16`."));
                out.push_ui(cx, row("show_label(...)", "bool", "Gallery/documentation affordance; remains off by default for upstream-like output."));
                out.push_ui(cx, row("refine_layout / refine_style", "LayoutRefinement / ChromeRefinement", "Typed equivalent to upstream `className` customization."));
                out.push_ui(cx, row("into_element_with_children(...)", "custom center visual", "Fret-specific extension for composable custom visuals without forking the shell."));
            }),
        );
    })
    .into_element(cx)
}

fn lifecycle_notes(_cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    crate::ui::doc_layout::notes_block([
        "Upstream AI Elements binds `Persona` to Rive/WebGL2 and therefore exposes load / ready / play / pause / stop callbacks.",
        "Fret intentionally keeps runtime IO and animation engine ownership app-side, so `fret-ui-ai::Persona` does not pretend to expose callback hooks it cannot honor yet.",
        "If we later add an app-owned Rive adapter, callback parity should land as ecosystem policy/runtime integration work rather than inside `crates/fret-ui`.",
    ])
}

pub(super) fn preview_ai_persona_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let interactive = snippets::persona_demo::render(cx);
    let variants = snippets::persona_variants::render(cx);
    let basic = snippets::persona_basic::render(cx);
    let state_management = snippets::persona_state_management::render(cx);
    let custom_styling = snippets::persona_custom_styling::render(cx);
    let custom_visual = snippets::persona_custom_visual::render(cx);
    let states = states_notes(cx);
    let props = props_table(cx);
    let lifecycle = lifecycle_notes(cx);

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "Docs-aligned AI Elements Persona demo: interactive state controls, variant showcase, states/props reference, and a Fret-specific custom visual slot.",
        ),
        vec![
            DocSection::new("Interactive Demo", interactive)
                .description("Mirrors the official docs preview shape more closely: one current persona with state controls plus variant switching.")
                .test_id_prefix("ui-gallery-ai-persona-demo")
                .max_w(Px(980.0))
                .code_rust_from_file_region(snippets::persona_demo::SOURCE, "example"),
            DocSection::new("Variants", variants)
                .description("Six variant shells with the same idle state so visual drift is easy to compare at a glance.")
                .max_w(Px(980.0))
                .code_rust_from_file_region(snippets::persona_variants::SOURCE, "example"),
            DocSection::new("Basic Usage", basic)
                .description("Minimal surface aligned to the upstream basic example: a single Persona with explicit state + variant.")
                .max_w(Px(980.0))
                .code_rust_from_file_region(snippets::persona_basic::SOURCE, "example"),
            DocSection::new("With State Management", state_management)
                .description("Copyable state-driven example matching the official docs intent without relying on DOM buttons.")
                .max_w(Px(980.0))
                .code_rust_from_file_region(snippets::persona_state_management::SOURCE, "example"),
            DocSection::new("Custom Styling", custom_styling)
                .description("Equivalent to upstream `className`-driven styling, expressed as typed Fret chrome/layout refinements.")
                .max_w(Px(980.0))
                .code_rust_from_file_region(snippets::persona_custom_styling::SOURCE, "example"),
            DocSection::new("Custom Visual Slot", custom_visual)
                .description("Fret-specific extension: replace the center visual without forking the shell or losing stable selectors.")
                .max_w(Px(980.0))
                .code_rust_from_file_region(snippets::persona_custom_visual::SOURCE, "example"),
            DocSection::build(cx, "States", states)
                .description("Persona responds to the same five high-level states described in the upstream docs.")
                .max_w(Px(980.0))
                .no_shell(),
            DocSection::build(cx, "Props & Extensions", props)
                .description("Upstream-facing props plus Fret-specific typed customization seams.")
                .max_w(Px(980.0))
                .no_shell(),
            DocSection::build(cx, "Lifecycle & Ownership", lifecycle)
                .description("Why callback parity is intentionally deferred until a concrete runtime adapter exists.")
                .max_w(Px(980.0))
                .no_shell(),
        ],
    );

    vec![body.test_id("ui-gallery-page-ai-persona-demo")]
}

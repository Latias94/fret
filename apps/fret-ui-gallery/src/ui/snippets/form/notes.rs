pub const SOURCE: &str = include_str!("notes.rs");

// region: example
use fret_app::App;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N1)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Upstream reference: `repo-ref/ui/apps/v4/app/(internal)/sink/components/form-demo.tsx`.",
                ),
                shadcn::typography::muted(
                    cx,
                    "API reference: `ecosystem/fret-ui-shadcn/src/form.rs` (Form alias), `ecosystem/fret-ui-shadcn/src/field.rs` (FieldSet), and control primitives: `input.rs`, `textarea.rs`, `checkbox.rs`, `switch.rs`.",
                ),
                shadcn::typography::muted(
                    cx,
                    "The first section mirrors upstream `FormDemo` using `FormState` + `FormRegistry`; the remaining sections are gallery recipes (composition hub).",
                ),
                shadcn::typography::muted(
                    cx,
                    "Keep stable test IDs for each recipe so future diag automation can target composition surfaces.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Textarea supports placeholder text; the upstream Bio example uses a placeholder string.",
                ),
            ]
        },
    )
}
// endregion: example

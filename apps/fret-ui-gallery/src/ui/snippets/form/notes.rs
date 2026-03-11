pub const SOURCE: &str = include_str!("notes.rs");

// region: example
use fret_app::App;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    ui::v_flex(|cx| {
        vec![
            shadcn::raw::typography::muted(
                cx,
                "Upstream reference: `repo-ref/ui/apps/v4/app/(internal)/sink/components/form-demo.tsx`.",
            ),
            shadcn::raw::typography::muted(
                cx,
                "API reference: `ecosystem/fret-ui-shadcn/src/form.rs` (`FormControl` is slot-like for a single child), `ecosystem/fret-ui-shadcn/src/field.rs` (`Form`/`FormItem`/helpers), and control primitives: `input.rs`, `textarea.rs`, `checkbox.rs`, `switch.rs`.",
            ),
            shadcn::raw::typography::muted(
                cx,
                "The first section mirrors upstream `FormDemo` using `FormState` + `FormRegistry`; the remaining sections are gallery recipes (composition hub).",
            ),
            shadcn::raw::typography::muted(
                cx,
                "Fret keeps the shadcn taxonomy (`Form`, `FormField`, `FormItem`, etc.) but maps it onto framework-agnostic field primitives instead of mirroring `react-hook-form` literally; `FormControl` stays a transparent single-control wrapper rather than a layout column.",
            ),
            shadcn::raw::typography::muted(
                cx,
                "Keep stable test IDs for each recipe so future diag automation can target composition surfaces.",
            ),
            shadcn::raw::typography::muted(
                cx,
                "Textarea supports placeholder text; the upstream Bio example uses a placeholder string.",
            ),
        ]
    })
    .gap(Space::N1)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
}
// endregion: example

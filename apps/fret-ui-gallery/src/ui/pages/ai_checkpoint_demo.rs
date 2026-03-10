use super::super::*;

use crate::ui::doc_layout::{self, DocSection, notes};
use crate::ui::snippets::ai as snippets;
use fret_ui_shadcn as shadcn;

pub(super) fn preview_ai_checkpoint_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let demo = snippets::checkpoint_demo::render(cx);
    let features = notes(
        cx,
        [
            "Simple flex layout with icon, trigger, and separator",
            "Visual separator line for clear conversation breaks",
            "Clickable restore button for reverting to checkpoint",
            "Customizable icon (defaults to BookmarkIcon)",
            "Keyboard accessible with proper ARIA labels",
            "Responsive design that adapts to different screen sizes",
            "Seamless light/dark theme integration",
        ],
    );
    let customizable_icon = notes(
        cx,
        [
            "Swap the default bookmark for a product-specific icon while keeping the surrounding checkpoint recipe unchanged.",
            "Custom child content now inherits the same muted foreground baseline as the default bookmark icon.",
            "`CheckpointIcon::children_many(...)` and `into_element_with_children(...)` make move-only Fret trees feel closer to the official React children API.",
        ],
    );
    let manual_checkpoints = notes(
        cx,
        ["Allow users to manually create checkpoints at important conversation points."],
    );
    let automatic_checkpoints = notes(
        cx,
        ["Create checkpoints automatically after significant conversation milestones."],
    );
    let branching = notes(
        cx,
        [
            "Use checkpoints to enable branching conversations where users can explore different paths.",
        ],
    );
    let props = checkpoint_props_table(cx).test_id("ui-gallery-ai-checkpoint-props");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "The `Checkpoint` component provides a way to mark specific points in a conversation history and restore the chat to that state. Inspired by VSCode's Copilot checkpoint feature, it allows users to revert to an earlier conversation state while maintaining a clear visual separation between different conversation segments.",
        ),
        vec![
            DocSection::new("Usage with AI SDK", demo)
                .description(
                    "Build a chat interface with conversation checkpoints that allow users to restore to previous states.",
                )
                .test_id_prefix("ui-gallery-ai-checkpoint-demo")
                .max_w(Px(980.0))
                .code_rust_from_file_region(snippets::checkpoint_demo::SOURCE, "example"),
            DocSection::new("Features", features)
                .description("Official AI Elements defaults and outcomes to keep in view while porting.")
                .max_w(Px(980.0))
                .no_shell(),
            DocSection::new("Customizable Icon", customizable_icon)
                .description("The icon slot stays composable even though the default visual is a bookmark.")
                .max_w(Px(980.0))
                .code_rust_from_file_region(snippets::checkpoint_demo::SOURCE, "custom_icon")
                .no_shell(),
            DocSection::new("Manual Checkpoints", manual_checkpoints)
                .description("Create a checkpoint at an explicit user-visible milestone.")
                .max_w(Px(980.0))
                .code_rust_from_file_region(snippets::checkpoint_demo::SOURCE, "manual_checkpoints")
                .no_shell(),
            DocSection::new("Automatic Checkpoints", automatic_checkpoints)
                .description("Use a lightweight milestone rule when you want history snapshots without extra UI chrome.")
                .max_w(Px(980.0))
                .code_rust_from_file_region(snippets::checkpoint_demo::SOURCE, "automatic_checkpoints")
                .no_shell(),
            DocSection::new("Branching Conversations", branching)
                .description("Restore earlier context while preserving the truncated tail as a branch.")
                .max_w(Px(980.0))
                .code_rust_from_file_region(snippets::checkpoint_demo::SOURCE, "branching_conversations")
                .no_shell(),
            DocSection::new("Props", props)
                .description("Fret API surface for `fret_ui_ai::Checkpoint*` builders.")
                .max_w(Px(980.0)),
        ],
    );

    vec![body]
}

fn checkpoint_props_table(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let row = |cx: &mut ElementContext<'_, App>,
               part: &'static str,
               method: &'static str,
               ty: &'static str,
               default: &'static str,
               desc: &'static str| {
        shadcn::TableRow::build(5, move |cx, out| {
            out.push_ui(cx, shadcn::TableCell::build(ui::text(part)));
            out.push_ui(cx, shadcn::TableCell::build(ui::text(method)));
            out.push_ui(cx, shadcn::TableCell::build(ui::text(ty)));
            out.push_ui(cx, shadcn::TableCell::build(ui::text(default)));
            out.push_ui(cx, shadcn::TableCell::build(ui::text(desc)));
        })
        .border_bottom(true)
    };

    shadcn::Table::build(|cx, out| {
        out.push_ui(
            cx,
            shadcn::TableHeader::build(|cx, out| {
                out.push(
                    shadcn::TableRow::build(5, |cx, out| {
                        out.push(shadcn::TableHead::new("Part").into_element(cx));
                        out.push(shadcn::TableHead::new("Method").into_element(cx));
                        out.push(shadcn::TableHead::new("Type").into_element(cx));
                        out.push(shadcn::TableHead::new("Default").into_element(cx));
                        out.push(shadcn::TableHead::new("Description").into_element(cx));
                    })
                    .border_bottom(true)
                    .into_element(cx),
                );
            }),
        );
        out.push_ui(
            cx,
            shadcn::TableBody::build(|cx, out| {
                out.push_ui(cx, row(cx, "Checkpoint", "new(children)", "IntoIterator<Item = AnyElement>", "-", "Primary compound children; appends a trailing separator automatically."));
                out.push_ui(cx, row(cx, "Checkpoint", "into_element_with_children", "FnOnce(&mut ElementContext) -> Vec<AnyElement>", "-", "Builds the checkpoint row inside a live scope when compound children are easier to author lazily."));
                out.push_ui(cx, row(cx, "Checkpoint", "test_id", "impl Into<Arc<str>>", "None", "Stamps a group-level diagnostics id for automation and scripted repros."));
                out.push_ui(cx, row(cx, "Checkpoint", "refine_layout / refine_style", "builder methods", "w_full + min_w_0 + overflow_hidden", "Adjusts layout and chrome while preserving the checkpoint recipe defaults."));
                out.push_ui(cx, row(cx, "CheckpointIcon", "children", "AnyElement", "Bookmark icon", "Overrides the default icon with one composed subtree; muted foreground now inherits from the row by default."));
                out.push_ui(cx, row(cx, "CheckpointIcon", "children_many / into_element_with_children", "builder methods", "None", "Lets the icon slot host multiple composed nodes or build them lazily inside a live element scope."));
                out.push_ui(cx, row(cx, "CheckpointIcon", "icon_id / size / color", "builder methods", "bookmark / 16px / muted", "Tweaks the default bookmark visual without replacing the slot entirely."));
                out.push_ui(cx, row(cx, "CheckpointTrigger", "new(children)", "IntoIterator<Item = AnyElement>", "-", "Button label or richer trigger content."));
                out.push_ui(cx, row(cx, "CheckpointTrigger", "variant / size", "ButtonVariant / ButtonSize", "Ghost / Sm", "Matches the official AI Elements checkpoint trigger defaults."));
                out.push_ui(cx, row(cx, "CheckpointTrigger", "tooltip / tooltip_panel_test_id", "builder methods", "None", "Adds the docs-style tooltip and a stable test id for the floating panel."));
                out.push_ui(cx, row(cx, "CheckpointTrigger", "on_activate / muted_foreground / test_id", "builder methods", "None / true / None", "Wires restore behavior, keeps the idle ghost label muted, and exposes a stable trigger id."));
            }),
        );
    })
    .into_element(cx)
}

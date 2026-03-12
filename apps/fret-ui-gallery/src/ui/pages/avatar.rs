use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::avatar as snippets;

pub(super) fn preview_avatar(
    cx: &mut UiCx<'_>,
    avatar_image: Model<Option<ImageId>>,
) -> Vec<AnyElement> {
    #[derive(Default)]
    struct AvatarDropdownOpenState {
        model: Option<Model<bool>>,
    }

    let dropdown_open = cx.with_state(AvatarDropdownOpenState::default, |st| st.model.clone());
    let dropdown_open = if let Some(model) = dropdown_open {
        model
    } else {
        let model = cx.app.models_mut().insert(false);
        cx.with_state(AvatarDropdownOpenState::default, |st| {
            st.model = Some(model.clone());
        });
        model
    };

    let demo = snippets::demo::render(cx, avatar_image.clone());
    let usage = snippets::usage::render(cx);
    let basic = snippets::basic::render(cx, avatar_image.clone());
    let with_badge = snippets::with_badge::render(cx, avatar_image.clone());
    let badge_icon = snippets::badge_icon::render(cx, avatar_image.clone());
    let avatar_group = snippets::group::render(cx, avatar_image.clone());
    let group_count = snippets::group_count::render(cx, avatar_image.clone());
    let group_count_icon = snippets::group_count_icon::render(cx, avatar_image.clone());
    let sizes = snippets::sizes::render(cx, avatar_image.clone());
    let dropdown = snippets::dropdown::render(cx, avatar_image.clone(), dropdown_open.clone());
    let rtl = snippets::rtl::render(cx, avatar_image.clone());
    let fallback = snippets::fallback_only::render(cx);

    let api_reference = doc_layout::notes(
        cx,
        [
            "`Avatar`, `AvatarImage`, and `AvatarFallback` cover the base shadcn/Base UI surface, while `AvatarBadge`, `AvatarGroup`, and `AvatarGroupCount` stay as explicit typed recipe parts.",
            "`Avatar::new([..])` and `Avatar::children([..])` are already sufficient for composable avatar content; no extra generic children or slot-merge API is needed here.",
            "Dropdown composition remains recipe-owned: the authored pressable child button is the trigger, and the nested avatar stays presentational content inside it.",
        ],
    );

    let notes = doc_layout::notes(
        cx,
        [
            "Core parity is already in a good place: default size, circular clipping, fallback timing, overlap geometry, and dropdown trigger attribution all match the audited upstream outcomes.",
            "Gallery sections now mirror shadcn Avatar docs first: Demo, Usage, Basic, Badge, Badge with Icon, Avatar Group, Avatar Group Count, Avatar Group with Icon, Sizes, Dropdown, RTL, API Reference.",
            "Fret uses `ImageId` / `Model<Option<ImageId>>` instead of DOM `src`, so the minimal usage snippet intentionally shows the asset-ready composition shape rather than a browser-only image URL prop.",
            "`Fallback only` remains a Fret-specific follow-up section for compact regression coverage across sizes.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview now mirrors the shadcn Avatar docs order first, then appends a small Fret-specific fallback-only check.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Docs-aligned demo: basic avatar, avatar with badge, and avatar group with count.")
                .test_id_prefix("ui-gallery-avatar-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .description("Minimal usage mirroring the upstream docs shape, adapted to Fret's `ImageId` asset model.")
                .test_id_prefix("ui-gallery-avatar-usage")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Basic", basic)
                .description("Single avatar with an image + fallback, using a gallery-owned image model.")
                .test_id_prefix("ui-gallery-avatar-basic")
                .code_rust_from_file_region(snippets::basic::SOURCE, "example"),
            DocSection::new("Badge", with_badge)
                .description("`AvatarBadge` overlays a status dot at the avatar's bottom-right.")
                .test_id_prefix("ui-gallery-avatar-badge")
                .code_rust_from_file_region(snippets::with_badge::SOURCE, "example"),
            DocSection::new("Badge with Icon", badge_icon)
                .description("Use `AvatarBadge` children to render an icon badge.")
                .test_id_prefix("ui-gallery-avatar-badge-icon")
                .code_rust_from_file_region(snippets::badge_icon::SOURCE, "example"),
            DocSection::new("Avatar Group", avatar_group)
                .description("Overlapping avatar group (`-space-x-2`).")
                .test_id_prefix("ui-gallery-avatar-group")
                .code_rust_from_file_region(snippets::group::SOURCE, "example"),
            DocSection::new("Avatar Group Count", group_count)
                .description("Trailing count bubble that matches the group's size.")
                .test_id_prefix("ui-gallery-avatar-group-count")
                .code_rust_from_file_region(snippets::group_count::SOURCE, "example"),
            DocSection::new("Avatar Group with Icon", group_count_icon)
                .description("Use an icon inside `AvatarGroupCount` for add/invite affordances.")
                .test_id_prefix("ui-gallery-avatar-group-count-icon")
                .code_rust_from_file_region(snippets::group_count_icon::SOURCE, "example"),
            DocSection::new("Sizes", sizes)
                .description("Upstream: `size=\"sm\" | \"default\" | \"lg\"`.")
                .test_id_prefix("ui-gallery-avatar-sizes")
                .code_rust_from_file_region(snippets::sizes::SOURCE, "example"),
            DocSection::new("Dropdown", dropdown)
                .description(
                    "Use Avatar as a DropdownMenu trigger (shadcn `render` / `asChild` composition outcome).",
                )
                .test_id_prefix("ui-gallery-avatar-dropdown")
                .code_rust_from_file_region(snippets::dropdown::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Avatar should behave under an RTL direction provider.")
                .test_id_prefix("ui-gallery-avatar-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("API Reference", api_reference)
                .no_shell()
                .description("Public surface summary and composition ownership notes.")
                .test_id_prefix("ui-gallery-avatar-api-reference"),
            DocSection::new("Fallback only (Fret)", fallback)
                .description("Fallback-only avatars at each size.")
                .test_id_prefix("ui-gallery-avatar-fallback")
                .code_rust_from_file_region(snippets::fallback_only::SOURCE, "example"),
            DocSection::new("Notes", notes)
                .no_shell()
                .description("Usage notes.")
                .test_id_prefix("ui-gallery-avatar-notes"),
        ],
    );

    vec![body.test_id("ui-gallery-avatar")]
}

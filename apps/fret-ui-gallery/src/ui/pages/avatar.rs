use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::avatar as snippets;

pub(super) fn preview_avatar(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let basic = snippets::basic::render(cx);
    let with_badge = snippets::with_badge::render(cx);
    let badge_icon = snippets::badge_icon::render(cx);
    let avatar_group = snippets::group::render(cx);
    let group_count = snippets::group_count::render(cx);
    let group_count_icon = snippets::group_count_icon::render(cx);
    let sizes = snippets::sizes::render(cx);
    let dropdown = snippets::dropdown::render(cx);
    let rtl = snippets::rtl::render(cx);
    let fallback = snippets::fallback_only::render(cx);

    let api_reference = doc_layout::notes_block([
        "`Avatar`, `AvatarImage`, and `AvatarFallback` cover the base shadcn/Base UI surface, while `AvatarBadge`, `AvatarGroup`, and `AvatarGroupCount` stay as explicit typed recipe parts.",
        "`Avatar::new([..])` remains the base composable builder, while `avatar_sized(...)` is the preferred helper when size-dependent children should inherit the active avatar size before landing.",
        "Dropdown composition remains recipe-owned: the authored pressable child button is the trigger, and the nested avatar stays presentational content inside it.",
    ]);

    let notes = doc_layout::notes_block([
        "Core parity is already in a good place: default size, circular clipping, fallback timing, overlap geometry, and dropdown trigger attribution all match the audited upstream outcomes.",
        "Gallery sections now mirror shadcn Avatar docs first: Demo, Usage, Basic, Badge, Badge with Icon, Avatar Group, Avatar Group Count, Avatar Group with Icon, Sizes, Dropdown, RTL, API Reference.",
        "Gallery snippets now generate a self-contained `ImageSource::rgba8(...) -> ImageId` demo asset locally instead of relaying a page-owned `Model<Option<ImageId>>` through the docs shell.",
        "`Fallback only` remains a Fret-specific follow-up section for compact regression coverage across sizes.",
    ]);

    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .description("Public surface summary and composition ownership notes.")
        .test_id_prefix("ui-gallery-avatar-api-reference");
    let notes = DocSection::build(cx, "Notes", notes)
        .no_shell()
        .description("Usage notes.")
        .test_id_prefix("ui-gallery-avatar-notes");
    let demo = DocSection::build(cx, "Demo", demo)
        .description(
            "Docs-aligned demo: basic avatar, avatar with badge, and avatar group with count.",
        )
        .test_id_prefix("ui-gallery-avatar-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Minimal usage mirroring the upstream docs shape, adapted to Fret's `ImageId` asset model.")
        .test_id_prefix("ui-gallery-avatar-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let basic = DocSection::build(cx, "Basic", basic)
        .description("Single avatar with an image + fallback using a self-contained demo image.")
        .test_id_prefix("ui-gallery-avatar-basic")
        .code_rust_from_file_region(snippets::basic::SOURCE, "example");
    let with_badge = DocSection::build(cx, "Badge", with_badge)
        .description("`AvatarBadge` overlays a status dot at the avatar's bottom-right.")
        .test_id_prefix("ui-gallery-avatar-badge")
        .code_rust_from_file_region(snippets::with_badge::SOURCE, "example");
    let badge_icon = DocSection::build(cx, "Badge with Icon", badge_icon)
        .description("Use `AvatarBadge` children to render an icon badge.")
        .test_id_prefix("ui-gallery-avatar-badge-icon")
        .code_rust_from_file_region(snippets::badge_icon::SOURCE, "example");
    let avatar_group = DocSection::build(cx, "Avatar Group", avatar_group)
        .description("Overlapping avatar group (`-space-x-2`).")
        .test_id_prefix("ui-gallery-avatar-group")
        .code_rust_from_file_region(snippets::group::SOURCE, "example");
    let group_count = DocSection::build(cx, "Avatar Group Count", group_count)
        .description("Trailing count bubble that matches the group's size.")
        .test_id_prefix("ui-gallery-avatar-group-count")
        .code_rust_from_file_region(snippets::group_count::SOURCE, "example");
    let group_count_icon = DocSection::build(cx, "Avatar Group with Icon", group_count_icon)
        .description("Use an icon inside `AvatarGroupCount` for add/invite affordances.")
        .test_id_prefix("ui-gallery-avatar-group-count-icon")
        .code_rust_from_file_region(snippets::group_count_icon::SOURCE, "example");
    let sizes = DocSection::build(cx, "Sizes", sizes)
        .description("Upstream: `size=\"sm\" | \"default\" | \"lg\"`; first-party examples teach `avatar_sized(...)` for that lane.")
        .test_id_prefix("ui-gallery-avatar-sizes")
        .code_rust_from_file_region(snippets::sizes::SOURCE, "example");
    let dropdown = DocSection::build(cx, "Dropdown", dropdown)
        .description("Use Avatar as a DropdownMenu trigger (shadcn `render` / `asChild` composition outcome).")
        .test_id_prefix("ui-gallery-avatar-dropdown")
        .code_rust_from_file_region(snippets::dropdown::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("Avatar should behave under an RTL direction provider.")
        .test_id_prefix("ui-gallery-avatar-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let fallback = DocSection::build(cx, "Fallback only (Fret)", fallback)
        .description("Fallback-only avatars at each size.")
        .test_id_prefix("ui-gallery-avatar-fallback")
        .code_rust_from_file_region(snippets::fallback_only::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview now mirrors the shadcn Avatar docs order first, then appends a small Fret-specific fallback-only check.",
        ),
        vec![
            demo,
            usage,
            basic,
            with_badge,
            badge_icon,
            avatar_group,
            group_count,
            group_count_icon,
            sizes,
            dropdown,
            rtl,
            api_reference,
            fallback,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-avatar").into_element(cx)]
}

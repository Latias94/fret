use super::super::*;
use fret::UiCx;

use crate::spec::{
    BISECT_DISABLE_CARD_CODE_TABS, BISECT_DISABLE_CARD_PAGE_INTRO,
    BISECT_DISABLE_CARD_SECTION_CARD_CONTENT, BISECT_DISABLE_CARD_SECTION_COMPOSITIONS,
    BISECT_DISABLE_CARD_SECTION_DEMO, BISECT_DISABLE_CARD_SECTION_IMAGE,
    BISECT_DISABLE_CARD_SECTION_MEETING_NOTES, BISECT_DISABLE_CARD_SECTION_NOTES,
    BISECT_DISABLE_CARD_SECTION_RTL, BISECT_DISABLE_CARD_SECTION_SIZE,
    BISECT_DISABLE_CARD_SECTION_USAGE, ui_gallery_bisect_flags,
};
use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::card as snippets;

const CARD_PAGE_INTRO: &str = "Displays a card with header, content, and footer. Preview mirrors the upstream shadcn Card docs path after `Installation`: `Demo`, `Usage`, `Size`, `Image`, `RTL`, and `API Reference`; Fret-only regression sections stay afterwards.";
const CARD_CODE_REGION: &str = "example";

struct CardDocSectionDiagnostics {
    title: &'static str,
    description: &'static str,
    disable_flag: u32,
    code_source: Option<&'static str>,
    shell: bool,
}

fn card_doc_code_region_stats(code: &str, region: &str) -> (u64, u64) {
    let mut inside = false;
    let mut bytes = 0u64;
    let mut lines = 0u64;

    for line in code.lines() {
        let trimmed = line.trim();
        if let Some(name) = trimmed.strip_prefix("// region:") {
            inside = name.trim() == region;
            continue;
        }
        if trimmed == "// endregion" {
            if inside {
                break;
            }
            continue;
        }
        if let Some(name) = trimmed.strip_prefix("// endregion:") {
            if inside && (name.trim().is_empty() || name.trim() == region) {
                break;
            }
            continue;
        }
        if inside {
            bytes = bytes.saturating_add(line.len() as u64 + 1);
            lines = lines.saturating_add(1);
        }
    }

    if lines == 0 {
        (code.len() as u64, code.lines().count() as u64)
    } else {
        (bytes, lines)
    }
}

pub(crate) fn card_doc_scaffold_metrics_json(bisect: u32) -> serde_json::Value {
    let sections = [
        CardDocSectionDiagnostics {
            title: "Demo",
            description: "Login card layout (CardHeader + CardAction + CardContent + CardFooter).",
            disable_flag: BISECT_DISABLE_CARD_SECTION_DEMO,
            code_source: Some(snippets::demo::SOURCE),
            shell: false,
        },
        CardDocSectionDiagnostics {
            title: "Usage",
            description: "Basic structure (header/content/footer) with an optional action slot.",
            disable_flag: BISECT_DISABLE_CARD_SECTION_USAGE,
            code_source: Some(snippets::usage::SOURCE),
            shell: false,
        },
        CardDocSectionDiagnostics {
            title: "Size",
            description: "Compact small-card example aligned with the upstream docs.",
            disable_flag: BISECT_DISABLE_CARD_SECTION_SIZE,
            code_source: Some(snippets::size::SOURCE),
            shell: false,
        },
        CardDocSectionDiagnostics {
            title: "Image",
            description: "Media-first card with a featured badge, footer action, and a self-contained cover image.",
            disable_flag: BISECT_DISABLE_CARD_SECTION_IMAGE,
            code_source: Some(snippets::image::SOURCE),
            shell: false,
        },
        CardDocSectionDiagnostics {
            title: "RTL",
            description: "RTL login card aligned with the upstream translated example.",
            disable_flag: BISECT_DISABLE_CARD_SECTION_RTL,
            code_source: Some(snippets::rtl::SOURCE),
            shell: false,
        },
        CardDocSectionDiagnostics {
            title: "API Reference",
            description: "Public surface summary and ownership notes.",
            disable_flag: 0,
            code_source: None,
            shell: false,
        },
        CardDocSectionDiagnostics {
            title: "Rich Title (Fret)",
            description: "Copyable `card_title_children(...)` lane for attributed text and caller-owned title composition.",
            disable_flag: 0,
            code_source: Some(snippets::title_children::SOURCE),
            shell: false,
        },
        CardDocSectionDiagnostics {
            title: "Rich Description (Fret)",
            description: "Copyable `card_description_children(...)` lane for attributed text and caller-owned description composition.",
            disable_flag: 0,
            code_source: Some(snippets::description_children::SOURCE),
            shell: false,
        },
        CardDocSectionDiagnostics {
            title: "Compositions",
            description: "Fret-only slot matrix: docs-consistent header/content/footer/action permutations plus bordered-section follow-ups.",
            disable_flag: BISECT_DISABLE_CARD_SECTION_COMPOSITIONS,
            code_source: Some(snippets::compositions::SOURCE),
            shell: false,
        },
        CardDocSectionDiagnostics {
            title: "CardContent",
            description: "CardContent should preserve intrinsic sizes for inline children.",
            disable_flag: BISECT_DISABLE_CARD_SECTION_CARD_CONTENT,
            code_source: Some(snippets::card_content::SOURCE),
            shell: false,
        },
        CardDocSectionDiagnostics {
            title: "Meeting Notes",
            description: "Card with text content, self-contained avatar media, and a footer stack.",
            disable_flag: BISECT_DISABLE_CARD_SECTION_MEETING_NOTES,
            code_source: Some(snippets::meeting_notes::SOURCE),
            shell: false,
        },
        CardDocSectionDiagnostics {
            title: "Notes",
            description: "Implementation notes and pointers.",
            disable_flag: BISECT_DISABLE_CARD_SECTION_NOTES,
            code_source: None,
            shell: true,
        },
    ];

    let show_code_tabs = (bisect & BISECT_DISABLE_CARD_CODE_TABS) == 0;
    let show_intro = (bisect & BISECT_DISABLE_CARD_PAGE_INTRO) == 0;
    let mut visible_sections = 0u64;
    let mut visible_sections_with_code = 0u64;
    let mut visible_sections_with_shell = 0u64;
    let mut visible_title_bytes = 0u64;
    let mut visible_description_bytes = 0u64;
    let mut visible_code_bytes = 0u64;
    let mut visible_code_lines = 0u64;

    for section in sections {
        if (bisect & section.disable_flag) != 0 {
            continue;
        }
        visible_sections = visible_sections.saturating_add(1);
        visible_title_bytes = visible_title_bytes.saturating_add(section.title.len() as u64);
        visible_description_bytes =
            visible_description_bytes.saturating_add(section.description.len() as u64);
        if section.shell {
            visible_sections_with_shell = visible_sections_with_shell.saturating_add(1);
        }
        if show_code_tabs {
            if let Some(source) = section.code_source {
                let (code_bytes, code_lines) = card_doc_code_region_stats(source, CARD_CODE_REGION);
                visible_sections_with_code = visible_sections_with_code.saturating_add(1);
                visible_code_bytes = visible_code_bytes.saturating_add(code_bytes);
                visible_code_lines = visible_code_lines.saturating_add(code_lines);
            }
        }
    }

    let intro_len_bytes = show_intro
        .then_some(CARD_PAGE_INTRO.len() as u64)
        .unwrap_or(0);
    serde_json::json!({
        "card_doc_section_slots_total": 12u64,
        "card_doc_visible_sections_count": visible_sections,
        "card_doc_visible_sections_with_code_count": visible_sections_with_code,
        "card_doc_visible_sections_with_shell_count": visible_sections_with_shell,
        "card_doc_intro_len_bytes": intro_len_bytes,
        "card_doc_visible_static_text_bytes_estimate_total": intro_len_bytes
            .saturating_add(visible_title_bytes)
            .saturating_add(visible_description_bytes),
        "card_doc_visible_code_bytes_estimate_total": visible_code_bytes,
        "card_doc_visible_code_lines_estimate_total": visible_code_lines,
    })
}

pub(super) fn preview_card(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let bisect = ui_gallery_bisect_flags();
    let show_code_tabs = (bisect & BISECT_DISABLE_CARD_CODE_TABS) == 0;
    let show_intro = (bisect & BISECT_DISABLE_CARD_PAGE_INTRO) == 0;
    let api_reference = doc_layout::notes_block([
        "Reference baseline: shadcn/base + shadcn/radix Card docs.",
        "Visual/chrome baseline: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/card.tsx` plus the upstream `Demo`, `Size`, `Image`, and `RTL` examples.",
        "Base UI / Radix headless references do not add extra Card-specific interaction machinery here; the remaining drift is recipe/docs-surface work rather than a `fret-ui` mechanism bug.",
        "`card(...)` plus the slot helper family is the default copyable path; `Card::new([...])` remains the explicit raw/root collection surface.",
        "`Card`, `CardHeader`, `CardAction`, `CardContent`, and `CardFooter` already accept composable children via `...::new([...])` or the helper-family builders, so no extra generic root-level `children(...)` API is needed for shadcn parity.",
        "`CardTitle` and `CardDescription` keep compact text lanes by default, while `card_title_children(...)` / `card_description_children(...)` stay as the focused composable-children follow-ups instead of widening the whole family to a generic root `children(...)` / `compose()` API.",
        "`CardTitle::new(text)` stays the compact text lane, and `CardTitle::new_children(...)` / `card_title_children(...)` cover composable children when you need rich/selectable text or a caller-owned inline composition root.",
        "`CardDescription::new(text)` stays the compact supporting-text lane, and `CardDescription::new_children(...)` / `card_description_children(...)` cover matching composable children when the description needs attributed text or a caller-owned inline composition root.",
        "`CardAction` remains the recipe-owned top-right header slot, while card width (`w-full`, `max-w-sm`, fixed px width) stays caller-owned via `refine_layout(...)` on the card root.",
        "`CardFooter` direction/gap are the recipe-level knobs for the documented column/row outcomes; page/grid/container negotiation remains caller-owned and should not be baked into the Card recipe.",
        "This page is docs/public-surface parity work, not a mechanism-layer fix.",
    ]);
    let with_card_code = |section: DocSection, source: &'static str| {
        if show_code_tabs {
            section.code_rust_from_file_region(source, CARD_CODE_REGION)
        } else {
            section
        }
    };

    let mut sections: Vec<DocSection> = Vec::new();
    if (bisect & BISECT_DISABLE_CARD_SECTION_DEMO) == 0 {
        let login = snippets::demo::render(cx);
        let section = DocSection::build(cx, "Demo", login)
            .test_id_root("ui-gallery-card-section-demo")
            .test_id_prefix("ui-gallery-card-section-demo")
            .no_shell()
            .max_w(Px(980.0))
            .description("Login card layout (CardHeader + CardAction + CardContent + CardFooter).");
        sections.push(with_card_code(section, snippets::demo::SOURCE));
    }
    if (bisect & BISECT_DISABLE_CARD_SECTION_USAGE) == 0 {
        let usage = snippets::usage::render(cx);
        let section = DocSection::build(cx, "Usage", usage)
            .test_id_root("ui-gallery-card-section-usage")
            .test_id_prefix("ui-gallery-card-section-usage")
            .no_shell()
            .max_w(Px(980.0))
            .description("Basic structure (header/content/footer) with an optional action slot.");
        sections.push(with_card_code(section, snippets::usage::SOURCE));
    }
    if (bisect & BISECT_DISABLE_CARD_SECTION_SIZE) == 0 {
        let size = snippets::size::render(cx);
        let section = DocSection::build(cx, "Size", size)
            .test_id_root("ui-gallery-card-section-size")
            .test_id_prefix("ui-gallery-card-section-size")
            .no_shell()
            .max_w(Px(980.0))
            .description("Compact small-card example aligned with the upstream docs.");
        sections.push(with_card_code(section, snippets::size::SOURCE));
    }
    if (bisect & BISECT_DISABLE_CARD_SECTION_IMAGE) == 0 {
        let image = snippets::image::render(cx);
        let section = DocSection::build(cx, "Image", image)
            .test_id_root("ui-gallery-card-section-image")
            .test_id_prefix("ui-gallery-card-section-image")
            .no_shell()
            .max_w(Px(980.0))
            .description("Media-first card with a featured badge, footer action, and a self-contained cover image.");
        sections.push(with_card_code(section, snippets::image::SOURCE));
    }
    if (bisect & BISECT_DISABLE_CARD_SECTION_RTL) == 0 {
        let rtl = snippets::rtl::render(cx);
        let section = DocSection::build(cx, "RTL", rtl)
            .test_id_root("ui-gallery-card-section-rtl")
            .test_id_prefix("ui-gallery-card-section-rtl")
            .no_shell()
            .max_w(Px(980.0))
            .description("RTL login card aligned with the upstream translated example.");
        sections.push(with_card_code(section, snippets::rtl::SOURCE));
    }
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .test_id_root("ui-gallery-card-api-reference")
        .test_id_prefix("ui-gallery-card-api-reference")
        .no_shell()
        .description("Public surface summary and ownership notes.");
    sections.push(api_reference);
    let rich_title = snippets::title_children::render(cx);
    let rich_title = DocSection::build(cx, "Rich Title (Fret)", rich_title)
        .test_id_root("ui-gallery-card-section-rich-title")
        .test_id_prefix("ui-gallery-card-section-rich-title")
        .no_shell()
        .max_w(Px(980.0))
        .description(
            "Copyable `card_title_children(...)` lane for attributed text and caller-owned title composition.",
        );
    sections.push(with_card_code(rich_title, snippets::title_children::SOURCE));
    let rich_description = snippets::description_children::render(cx);
    let rich_description = DocSection::build(cx, "Rich Description (Fret)", rich_description)
        .test_id_root("ui-gallery-card-section-rich-description")
        .test_id_prefix("ui-gallery-card-section-rich-description")
        .no_shell()
        .max_w(Px(980.0))
        .description(
            "Copyable `card_description_children(...)` lane for attributed text and caller-owned description composition.",
        );
    sections.push(with_card_code(
        rich_description,
        snippets::description_children::SOURCE,
    ));
    if (bisect & BISECT_DISABLE_CARD_SECTION_COMPOSITIONS) == 0 {
        let compositions = snippets::compositions::render(cx);
        let section = DocSection::build(cx, "Compositions", compositions)
            .test_id_root("ui-gallery-card-section-compositions")
            .test_id_prefix("ui-gallery-card-section-compositions")
            .no_shell()
            .max_w(Px(980.0))
            .description("Fret-only slot matrix: docs-consistent header/content/footer/action permutations plus bordered-section follow-ups.");
        sections.push(with_card_code(section, snippets::compositions::SOURCE));
    }
    if (bisect & BISECT_DISABLE_CARD_SECTION_CARD_CONTENT) == 0 {
        let card_content_inline_button = snippets::card_content::render(cx);
        let section = DocSection::build(cx, "CardContent", card_content_inline_button)
            .test_id_root("ui-gallery-card-section-card-content")
            .test_id_prefix("ui-gallery-card-section-card-content")
            .no_shell()
            .max_w(Px(980.0))
            .description("CardContent should preserve intrinsic sizes for inline children.");
        sections.push(with_card_code(section, snippets::card_content::SOURCE));
    }
    if (bisect & BISECT_DISABLE_CARD_SECTION_MEETING_NOTES) == 0 {
        let meeting_notes = snippets::meeting_notes::render(cx);
        let section = DocSection::build(cx, "Meeting Notes", meeting_notes)
            .test_id_root("ui-gallery-card-section-meeting-notes")
            .test_id_prefix("ui-gallery-card-section-meeting-notes")
            .no_shell()
            .max_w(Px(980.0))
            .description(
                "Card with text content, self-contained avatar media, and a footer stack.",
            );
        sections.push(with_card_code(section, snippets::meeting_notes::SOURCE));
    }
    if (bisect & BISECT_DISABLE_CARD_SECTION_NOTES) == 0 {
        let notes = doc_layout::notes_block([
            "Card root width is caller-owned; express upstream `w-full max-w-sm` at the call site with `refine_layout(...)`.",
            "Grid/flex demo pages should put `w_full`, `min_w_0`, and `max_w(...)` on the page cell rather than baking them into the Card recipe.",
            "`CardFooter` now owns a fill-width + `min-w-0` inner row/column budget so footer-only text wraps against the card width instead of collapsing into one word per line.",
            "Default first-party teaching should prefer `card(...)` plus the slot helper family; `Card::build(...)` remains a lower-level builder option when call sites need explicit late child collection.",
            "`Rich Title (Fret)` and `Rich Description (Fret)` keep the `card_title_children(...)` / `card_description_children(...)` lanes copyable on the default app-facing surface instead of making callers drop to slot builders for rich text content.",
            "Gallery order now mirrors the upstream Card docs path through `API Reference` before appending Fret-only regression sections.",
            "`Compositions` is intentionally a Fret-only regression matrix, not an upstream docs-path example; it should stay honest about that and cover the optional `CardAction` lane when demonstrating slot permutations.",
            "The `Image` and `Meeting Notes` snippets now keep their demo media self-contained with inline RGBA sources, so the code tabs stay copyable without UI Gallery-only asset helpers.",
        ]);
        sections.push(
            DocSection::build(cx, "Notes", notes)
                .test_id_root("ui-gallery-card-section-notes")
                .test_id_prefix("ui-gallery-card-section-notes")
                .description("Implementation notes and pointers."),
        );
    }

    let body = doc_layout::render_doc_page(cx, show_intro.then_some(CARD_PAGE_INTRO), sections);

    vec![body.test_id("ui-gallery-card").into_element(cx)]
}

use super::*;
use fret::{UiChild, UiCx};
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::facade as shadcn;

pub(in crate::ui) struct DocSection {
    pub title: &'static str,
    pub title_test_id: Option<&'static str>,
    pub root_test_id: Option<Arc<str>>,
    pub description: Vec<&'static str>,
    // Intentionally stored as a landed value because the doc scaffold still decorates preview
    // roots, shells, and tab panels after section assembly.
    pub preview: AnyElement,
    pub code: Option<DocCodeBlock>,
    pub tabs_sizing: DocTabsSizing,
    pub max_w: Px,
    pub test_id_prefix: Option<Arc<str>>,
    pub shell: bool,
}

/// Layout contract for the docs scaffold's Preview/Code tab panels.
///
/// - `Intrinsic` keeps tab panels content-sized by default (shrink-wrap).
#[cfg_attr(
    feature = "gallery-ai",
    doc = "- `FillRemaining` allows `TabsContent` to fill remaining main-axis space when the tabs root is\n  laid out under a definite-size budget."
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(in crate::ui) enum DocTabsSizing {
    #[default]
    Intrinsic,
    #[cfg(feature = "gallery-ai")]
    FillRemaining,
}

pub(in crate::ui) struct DocCodeBlock {
    pub language: &'static str,
    pub code: Arc<str>,
}

impl DocSection {
    pub(in crate::ui) fn new(title: &'static str, preview: AnyElement) -> Self {
        Self {
            title,
            title_test_id: None,
            root_test_id: None,
            description: Vec::new(),
            preview,
            code: None,
            tabs_sizing: DocTabsSizing::default(),
            max_w: Px(760.0),
            test_id_prefix: None,
            shell: true,
        }
    }

    pub(in crate::ui) fn build<P>(cx: &mut UiCx<'_>, title: &'static str, preview: P) -> Self
    where
        P: IntoUiElement<fret_app::App>,
    {
        Self::new(title, preview.into_element(cx))
    }

    /// Marks a section as a Fret-only diagnostics harness while keeping the shared doc scaffold.
    pub(in crate::ui) fn build_diagnostics<P>(
        cx: &mut UiCx<'_>,
        title: &'static str,
        preview: P,
    ) -> Self
    where
        P: IntoUiElement<fret_app::App>,
    {
        Self::build(cx, title, preview)
    }

    pub(in crate::ui) fn title_test_id(mut self, title_test_id: &'static str) -> Self {
        self.title_test_id = Some(title_test_id);
        self
    }

    pub(in crate::ui) fn test_id_root(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.root_test_id = Some(test_id.into());
        self
    }

    pub(in crate::ui) fn description(mut self, description: &'static str) -> Self {
        self.description.push(description);
        self
    }

    pub(in crate::ui) fn descriptions(
        mut self,
        descriptions: impl IntoIterator<Item = &'static str>,
    ) -> Self {
        self.description.extend(descriptions);
        self
    }

    pub(in crate::ui) fn code(mut self, language: &'static str, code: impl Into<Arc<str>>) -> Self {
        self.code = Some(DocCodeBlock {
            language,
            code: code.into(),
        });
        self
    }

    pub(in crate::ui) fn code_from_file_region(
        self,
        language: &'static str,
        file: &'static str,
        region: &'static str,
    ) -> Self {
        let sliced = slice_code_region(file, region)
            .unwrap_or_else(|| format!("// region `{region}` not found\n{file}"));
        self.code(language, Arc::<str>::from(sliced))
    }

    pub(in crate::ui) fn code_rust_from_file_region(
        self,
        file: &'static str,
        region: &'static str,
    ) -> Self {
        self.code_from_file_region("rust", file, region)
    }

    /// Controls whether Preview/Code tabs should shrink-wrap their content (default) or fill any
    /// available main-axis space under definite-size ancestors (Tailwind-like `flex-1`).
    #[cfg(feature = "gallery-ai")]
    pub(in crate::ui) fn tabs_sizing(mut self, sizing: DocTabsSizing) -> Self {
        self.tabs_sizing = sizing;
        self
    }

    pub(in crate::ui) fn max_w(mut self, max_w: Px) -> Self {
        self.max_w = max_w;
        self
    }

    pub(in crate::ui) fn test_id_prefix(mut self, test_id_prefix: impl Into<Arc<str>>) -> Self {
        self.test_id_prefix = Some(test_id_prefix.into());
        self
    }

    pub(in crate::ui) fn no_shell(mut self) -> Self {
        self.shell = false;
        self
    }
}

// Typed page scaffold: the doc page still aggregates fully landed section bodies in
// `Vec<AnyElement>` internally before centering the final shell, but callers now keep the final
// landing explicit at the page surface.
pub(in crate::ui) fn render_doc_page(
    cx: &mut UiCx<'_>,
    intro: Option<&'static str>,
    sections: Vec<DocSection>,
) -> impl UiChild + use<> {
    render_doc_page_raw(cx as *mut _, intro, sections)
}

// Keeps page-body assembly borrow-friendly when section builders also need `cx`.
pub(in crate::ui) fn render_doc_page_after(
    intro: Option<&'static str>,
    sections: Vec<DocSection>,
    cx: &mut UiCx<'_>,
) -> impl UiChild + use<> {
    render_doc_page_raw(cx as *mut _, intro, sections)
}

fn render_doc_page_raw(
    cx: *mut UiCx<'_>,
    intro: Option<&'static str>,
    sections: Vec<DocSection>,
) -> impl UiChild + use<> {
    // SAFETY: callers pass the current page-building `UiCx`; this helper does not retain the
    // pointer beyond the call and only dereferences it after all other arguments are evaluated.
    let cx = unsafe { &mut *cx };
    let max_section_w = sections
        .iter()
        .map(|s| s.max_w)
        .fold(Px(0.0), |a, b| Px(a.0.max(b.0)));
    let page_max_w = Px(Px(760.0).0.max(max_section_w.0));

    ui::v_flex(move |cx| {
        let mut out: Vec<AnyElement> = Vec::with_capacity(sections.len() + 1);
        if let Some(intro) = intro {
            out.push(muted_full_width(cx, intro).into_element(cx));
        }
        out.extend(
            sections
                .into_iter()
                .map(|section| render_section(cx, section).into_element(cx)),
        );
        out
    })
    .gap(Space::N6)
    .items_start()
    .layout(
        // Center the doc column via auto margins instead of an extra horizontal flex wrapper.
        // The wrapper shape can over-inflate ancestor heights for long scrollable docs pages.
        LayoutRefinement::default()
            .w_full()
            .min_w_0()
            .max_w(page_max_w)
            .mx_auto(),
    )
    .into_element(cx)
}

#[cfg(any(feature = "gallery-dev", feature = "gallery-web-ime-harness"))]
// Typed preview-harness wrapper: preview harness pages still bridge pre-landed preview vectors
// into the shared doc page scaffold, but the explicit landing now lives at the preview-page call
// site rather than on this helper signature.
pub(in crate::ui) fn wrap_preview_page(
    cx: &mut UiCx<'_>,
    intro: Option<&'static str>,
    section_title: &'static str,
    elements: Vec<AnyElement>,
) -> impl UiChild + use<> {
    wrap_preview_page_raw(cx as *mut _, intro, section_title, elements)
}

// Keeps preview vectors borrow-friendly when call sites still land elements with `cx`.
#[cfg(any(feature = "gallery-dev", feature = "gallery-web-ime-harness"))]
pub(in crate::ui) fn wrap_preview_page_after(
    intro: Option<&'static str>,
    section_title: &'static str,
    elements: Vec<AnyElement>,
    cx: &mut UiCx<'_>,
) -> impl UiChild + use<> {
    wrap_preview_page_raw(cx as *mut _, intro, section_title, elements)
}

#[cfg(any(feature = "gallery-dev", feature = "gallery-web-ime-harness"))]
fn wrap_preview_page_raw(
    cx: *mut UiCx<'_>,
    intro: Option<&'static str>,
    section_title: &'static str,
    elements: Vec<AnyElement>,
) -> impl UiChild + use<> {
    // SAFETY: same contract as `render_doc_page`; the pointer is consumed immediately and never
    // stored.
    let cx = unsafe { &mut *cx };
    let preview = ui::v_flex(move |_cx| elements)
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .gap(Space::N4)
        .items_start()
        .into_element(cx);
    let preview_section = DocSection::build(cx, section_title, preview)
        .no_shell()
        .max_w(Px(980.0));

    render_doc_page(cx, intro, vec![preview_section])
}

/// A flex row that wraps on narrow widths.
///
/// Prefer this over the legacy stack-based hstack helper for "control bars" that can contain many
/// toggles/buttons.
#[cfg(feature = "gallery-dev")]
pub(in crate::ui) fn wrap_row<F>(
    cx: &mut UiCx<'_>,
    theme: &Theme,
    gap: Space,
    align: fret_ui::element::CrossAlign,
    children: F,
) -> impl UiChild + use<F>
where
    F: FnOnce(&mut UiCx<'_>) -> Vec<AnyElement>,
{
    let gap = fret_ui_kit::MetricRef::space(gap).resolve(theme);
    let layout = decl_style::layout_style(theme, LayoutRefinement::default().w_full().min_w_0());
    cx.flex(
        fret_ui::element::FlexProps {
            layout,
            direction: fret_core::Axis::Horizontal,
            gap: gap.into(),
            padding: Edges::all(Px(0.0)).into(),
            justify: fret_ui::element::MainAlign::Start,
            align,
            wrap: true,
        },
        children,
    )
    .into_element(cx)
}

#[cfg(feature = "gallery-dev")]
pub(in crate::ui) fn wrap_controls_row<F>(
    cx: &mut UiCx<'_>,
    theme: &Theme,
    gap: Space,
    children: F,
) -> impl UiChild + use<F>
where
    F: FnOnce(&mut UiCx<'_>) -> Vec<AnyElement>,
{
    wrap_row(
        cx,
        theme,
        gap,
        fret_ui::element::CrossAlign::Center,
        children,
    )
}

pub(in crate::ui) fn muted_full_width<T>(cx: &mut UiCx<'_>, text: T) -> impl UiChild + use<T>
where
    T: Into<Arc<str>>,
{
    let (style, color) = {
        let theme = Theme::global(&*cx.app);
        let style = fret_ui_kit::typography::control_text_style(
            theme,
            fret_ui_kit::typography::UiTextSize::Xs,
        );
        let color = theme
            .color_by_key("muted-foreground")
            .or_else(|| theme.color_by_key("muted_foreground"))
            .unwrap_or_else(|| theme.color_token("foreground"));
        (style, color)
    };

    cx.text_props(TextProps {
        layout: {
            let mut layout = fret_ui::element::LayoutStyle::default();
            layout.size.width = fret_ui::element::Length::Fill;
            layout
        },
        text: text.into(),
        style: Some(style),
        color: Some(color),
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        ink_overflow: fret_ui::element::TextInkOverflow::None,
    })
}

#[allow(dead_code)]
pub(in crate::ui) fn text_table<const N: usize, I>(
    cx: &mut UiCx<'_>,
    headers: [&'static str; N],
    rows: I,
    border_bottom: bool,
) -> impl UiChild + use<N, I>
where
    I: IntoIterator<Item = [&'static str; N]>,
{
    let rows = rows.into_iter().collect::<Vec<_>>();

    shadcn::table(move |cx| {
        let header = shadcn::table_header(move |_cx| {
            let mut header = shadcn::table_row(N as u16, move |_cx| {
                headers
                    .into_iter()
                    .map(shadcn::table_head)
                    .collect::<Vec<_>>()
            });
            if border_bottom {
                header = header.border_bottom(true);
            }
            vec![header]
        })
        .into_element(cx);

        let body = shadcn::table_body(move |_cx| {
            rows.into_iter()
                .map(move |cells| {
                    let mut row = shadcn::table_row(N as u16, move |_cx| {
                        cells
                            .into_iter()
                            .map(|cell| shadcn::table_cell(ui::text(cell)))
                            .collect::<Vec<_>>()
                    });
                    if border_bottom {
                        row = row.border_bottom(true);
                    }
                    row
                })
                .collect::<Vec<_>>()
        })
        .into_element(cx);

        vec![header, body]
    })
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
}

fn muted_inline<T>(cx: &mut UiCx<'_>, text: T) -> impl UiChild + use<T>
where
    T: Into<Arc<str>>,
{
    let (style, color) = {
        let theme = Theme::global(&*cx.app);
        let style = fret_ui_kit::typography::control_text_style(
            theme,
            fret_ui_kit::typography::UiTextSize::Xs,
        );
        let color = theme
            .color_by_key("muted-foreground")
            .or_else(|| theme.color_by_key("muted_foreground"))
            .unwrap_or_else(|| theme.color_token("foreground"));
        (style, color)
    };

    cx.text_props(TextProps {
        layout: fret_ui::element::LayoutStyle::default(),
        text: text.into(),
        style: Some(style),
        color: Some(color),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        ink_overflow: fret_ui::element::TextInkOverflow::None,
    })
}

pub(in crate::ui) fn notes_block<I, T>(lines: I) -> impl IntoUiElement<fret_app::App> + use<I, T>
where
    I: IntoIterator<Item = T>,
    T: Into<Arc<str>>,
{
    let lines = lines.into_iter().map(Into::into).collect::<Vec<Arc<str>>>();

    fn muted_flex_1_min_w_0<T>(cx: &mut UiCx<'_>, text: T) -> impl UiChild + use<T>
    where
        T: Into<Arc<str>>,
    {
        let (style, color) = {
            let theme = Theme::global(&*cx.app);
            let style = fret_ui_kit::typography::control_text_style(
                theme,
                fret_ui_kit::typography::UiTextSize::Xs,
            );
            let color = theme
                .color_by_key("muted-foreground")
                .or_else(|| theme.color_by_key("muted_foreground"))
                .unwrap_or_else(|| theme.color_token("foreground"));
            (style, color)
        };

        cx.text_props(TextProps {
            layout: {
                let mut layout = fret_ui::element::LayoutStyle::default();
                layout.flex.grow = 1.0;
                layout.flex.shrink = 1.0;
                layout.size.min_width = Some(fret_ui::element::Length::Px(Px(0.0)));
                layout
            },
            text: text.into(),
            style: Some(style),
            color: Some(color),
            wrap: TextWrap::WordBreak,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            ink_overflow: fret_ui::element::TextInkOverflow::None,
        })
    }

    ui::v_flex(move |cx| {
        lines
            .iter()
            .cloned()
            .map(|line| {
                ui::h_row(move |cx| {
                    [
                        muted_inline(cx, "•").into_element(cx),
                        muted_flex_1_min_w_0(cx, line).into_element(cx),
                    ]
                })
                .gap(Space::N1)
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .into_element(cx)
            })
            .collect::<Vec<_>>()
    })
    .gap(Space::N1)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
}

// Typed icon relay: the shared icon facade still lands a concrete icon leaf internally, but
// doc-layout callers no longer need to spell that raw detail on their own helper signatures.
pub(in crate::ui) fn icon(cx: &mut UiCx<'_>, id: &'static str) -> impl UiChild + use<> {
    shadcn::raw::icon::icon(cx, fret_icons::IconId::new_static(id))
}

#[allow(dead_code)]
// Intentional raw boundary: gap placeholders are assembled as concrete alert content because the
// helper returns a title/content pair for doc-page section registration.
pub(in crate::ui) fn gap_card(
    cx: &mut UiCx<'_>,
    title: &'static str,
    details: &'static str,
    test_id: &'static str,
) -> (&'static str, AnyElement) {
    let alert_content = shadcn::Alert::new([
        icon(cx, "lucide.info").into_element(cx),
        shadcn::AlertTitle::new("Guide-aligned placeholder").into_element(cx),
        shadcn::AlertDescription::new(details).into_element(cx),
    ])
    .variant(shadcn::AlertVariant::Default)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(700.0)))
    .into_element(cx)
    .test_id(test_id);
    (title, alert_content)
}

// Typed section wrapper: section assembly still adds test ids, shells, and optional semantics
// after the preview/code content lands, but callers now keep the final landing explicit.
fn render_section(cx: &mut UiCx<'_>, section: DocSection) -> impl UiChild + use<> {
    let DocSection {
        title,
        title_test_id,
        root_test_id,
        description,
        preview,
        code,
        tabs_sizing,
        max_w,
        test_id_prefix,
        shell,
    } = section;

    let has_code = code.is_some();
    let test_id_prefix = if has_code && test_id_prefix.is_none() {
        Some(auto_tabs_test_id_prefix(title, title_test_id))
    } else {
        test_id_prefix
    };

    let shell = shell && title != "Notes";
    let preview = if shell {
        demo_shell(cx, max_w, preview).into_element(cx)
    } else {
        preview
    };

    let content = match code {
        Some(code) => preview_code_tabs(
            cx,
            test_id_prefix.as_deref(),
            preview,
            max_w,
            code,
            tabs_sizing,
            shell,
        )
        .into_element(cx),
        None => preview,
    };

    let section_max_w = max_w;
    let section_body = ui::v_flex(move |cx| {
        let mut out: Vec<AnyElement> = Vec::with_capacity(3);
        let title_el = section_title(cx, title).into_element(cx);
        out.push(match (title_test_id, test_id_prefix.as_deref()) {
            (Some(test_id), _) => title_el.test_id(test_id),
            (None, Some(prefix)) => title_el.test_id(format!("{prefix}-title")),
            (None, None) => title_el,
        });
        if !description.is_empty() {
            let description_el = if description.len() == 1 {
                muted_full_width(cx, description[0]).into_element(cx)
            } else {
                ui::v_flex(move |cx| {
                    description
                        .into_iter()
                        .map(|line| muted_full_width(cx, line).into_element(cx))
                        .collect::<Vec<_>>()
                })
                .gap(Space::N1)
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .into_element(cx)
            };
            out.push(if let Some(prefix) = test_id_prefix.as_deref() {
                description_el.test_id(format!("{prefix}-description"))
            } else {
                description_el
            });
        }
        out.push(if let Some(prefix) = test_id_prefix.as_deref() {
            content.test_id(format!("{prefix}-content"))
        } else {
            content
        });
        out
    })
    .gap(Space::N2)
    .items_start()
    .layout(
        LayoutRefinement::default()
            .w_full()
            .min_w_0()
            .max_w(section_max_w),
    )
    .into_element(cx);

    if let Some(root_test_id) = root_test_id {
        let mut section_layout = fret_ui::element::LayoutStyle::default();
        section_layout.size.width = fret_ui::element::Length::Fill;
        cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: section_layout,
                role: fret_core::SemanticsRole::Group,
                test_id: Some(root_test_id),
                ..Default::default()
            },
            |_cx| [section_body],
        )
    } else {
        section_body
    }
}

fn demo_shell<B>(
    cx: &mut UiCx<'_>,
    max_w: Px,
    body: B,
) -> impl IntoUiElement<fret_app::App> + use<B>
where
    B: IntoUiElement<fret_app::App>,
{
    let props = cx.with_theme(|theme| {
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Md)
                .p(Space::N4),
            LayoutRefinement::default()
                .w_full()
                .min_w_0()
                .max_w(max_w)
                .overflow_visible(),
        )
    });
    cx.container(props, move |cx| [body.into_element(cx)])
}

fn auto_tabs_test_id_prefix(title: &'static str, title_test_id: Option<&'static str>) -> Arc<str> {
    if let Some(title_test_id) = title_test_id {
        let base = title_test_id
            .strip_suffix("-title")
            .unwrap_or(title_test_id);
        return Arc::from(base);
    }

    let slug = slugify_for_test_id(title);
    Arc::<str>::from(format!("docsec-{slug}"))
}

fn slugify_for_test_id(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut prev_dash = false;

    for ch in input.chars() {
        let ch = ch.to_ascii_lowercase();
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            prev_dash = false;
            continue;
        }

        if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }

    while out.ends_with('-') {
        out.pop();
    }

    if out.is_empty() {
        "section".to_string()
    } else {
        out
    }
}

// Typed tabs wrapper: `TabsItem` still stores concrete landed panel content after the preview/code
// surfaces receive test-id decoration, but callers no longer need to see a raw return type.
fn preview_code_tabs(
    cx: &mut UiCx<'_>,
    test_id_prefix: Option<&str>,
    preview: AnyElement,
    max_w: Px,
    code: DocCodeBlock,
    #[cfg(feature = "gallery-ai")] tabs_sizing: DocTabsSizing,
    #[cfg(not(feature = "gallery-ai"))] _tabs_sizing: DocTabsSizing,
    shell: bool,
) -> impl UiChild + use<> {
    let code_el = code_block_shell(cx, test_id_prefix, max_w, code, shell).into_element(cx);
    #[cfg(feature = "gallery-ai")]
    let fill_remaining = matches!(tabs_sizing, DocTabsSizing::FillRemaining);
    #[cfg(not(feature = "gallery-ai"))]
    let fill_remaining = false;

    let base = shadcn::Tabs::uncontrolled(Some("preview"))
        .content_fill_remaining(fill_remaining)
        .refine_layout(LayoutRefinement::default().w_full().min_w_0());

    let tabs = if let Some(prefix) = test_id_prefix {
        let tabs_test_id = format!("{prefix}-tabs");
        base.test_id(tabs_test_id.clone()).items([
            shadcn::TabsItem::new("preview", "Preview", [preview])
                .trigger_test_id(format!("{tabs_test_id}-trigger-preview")),
            shadcn::TabsItem::new("code", "Code", [code_el])
                .trigger_test_id(format!("{tabs_test_id}-trigger-code")),
        ])
    } else {
        base.items([
            shadcn::TabsItem::new("preview", "Preview", [preview]),
            shadcn::TabsItem::new("code", "Code", [code_el]),
        ])
    };

    tabs.into_element(cx)
}

// Typed code-block wrapper: the code-block shell still owns concrete child vectors for the copy
// affordance, optional header, and scroll area before the card wrapper lands.
fn code_block_shell(
    cx: &mut UiCx<'_>,
    test_id_prefix: Option<&str>,
    max_w: Px,
    block: DocCodeBlock,
    shell: bool,
) -> impl UiChild + use<> {
    let code: Arc<str> = block.code;

    let copy_on_activate: fret_ui::action::OnActivate = {
        let code = code.clone();
        Arc::new(move |host, acx, _reason| {
            let token = host.next_clipboard_token();
            host.push_effect(fret_runtime::Effect::ClipboardWriteText {
                window: acx.window,
                token,
                text: code.to_string(),
            });
        })
    };

    let mut copy = shadcn::Button::new("Copy")
        .variant(shadcn::ButtonVariant::Secondary)
        .size(shadcn::ButtonSize::Sm)
        .on_activate(copy_on_activate)
        .into_element(cx);
    if let Some(prefix) = test_id_prefix {
        copy = copy.test_id(format!("{prefix}-code-block-copy"));
    }

    let header = ui::h_flex(|cx| {
        [
            shadcn::Badge::new(block.language)
                .variant(shadcn::BadgeVariant::Secondary)
                .into_element(cx),
            copy,
        ]
    })
    .layout(LayoutRefinement::default().w_full())
    .items_center()
    .justify_between()
    .gap(Space::N2)
    .into_element(cx);

    let theme = Theme::global(&*cx.app);
    let monospace = fret_core::TextStyle {
        font: fret_core::FontId::monospace(),
        size: Px(12.0),
        weight: fret_core::FontWeight::NORMAL,
        slant: fret_core::TextSlant::Normal,
        line_height: theme.metric_by_key("font.line_height"),
        line_height_policy: fret_core::TextLineHeightPolicy::FixedFromStyle,
        letter_spacing_em: None,
        ..Default::default()
    };
    let code_text = cx.text_props(TextProps {
        layout: {
            let mut layout = fret_ui::element::LayoutStyle::default();
            layout.size.width = fret_ui::element::Length::Fill;
            layout
        },
        text: code.clone(),
        style: Some(monospace),
        color: Some(theme.color_token("foreground")),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        ink_overflow: fret_ui::element::TextInkOverflow::None,
    });

    let mut scroll = shadcn::ScrollArea::new([code_text])
        .axis(fret_ui::element::ScrollAxis::Both)
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .min_w_0()
                .max_h(Px(400.0))
                .overflow_visible(),
        )
        .into_element(cx);

    if let Some(prefix) = test_id_prefix {
        scroll = scroll.test_id(format!("{prefix}-code-block"));
    }

    let body = if shell {
        vec![header, scroll]
    } else {
        vec![scroll]
    };

    shadcn::Card::new(vec![shadcn::CardContent::new(body).into_element(cx)])
        .refine_layout(LayoutRefinement::default().w_full().min_w_0().max_w(max_w))
        .into_element(cx)
}

fn slice_code_region(code: &str, region: &str) -> Option<String> {
    let mut inside = false;
    let mut out: Vec<&str> = Vec::new();

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
            out.push(line);
        }
    }

    if out.is_empty() {
        return None;
    }

    let mut joined = out.join("\n");
    joined.push('\n');
    Some(joined)
}

// Typed title helper: section titles may still receive decoration after landing, but the helper
// itself stays on the typed lane.
fn section_title(cx: &mut UiCx<'_>, title: &'static str) -> impl UiChild + use<> {
    let theme = Theme::global(&*cx.app);
    let style = fret_core::TextStyle {
        font: fret_core::FontId::default(),
        size: Px(20.0),
        weight: fret_core::FontWeight::SEMIBOLD,
        slant: fret_core::TextSlant::Normal,
        line_height: theme.metric_by_key("font.line_height"),
        line_height_policy: fret_core::TextLineHeightPolicy::FixedFromStyle,
        letter_spacing_em: None,
        ..Default::default()
    };

    cx.text_props(TextProps {
        layout: {
            let mut layout = fret_ui::element::LayoutStyle::default();
            layout.size.width = fret_ui::element::Length::Fill;
            layout
        },
        text: Arc::from(title),
        style: Some(style),
        color: None,
        wrap: TextWrap::None,
        overflow: TextOverflow::Ellipsis,
        align: fret_core::TextAlign::Start,
        ink_overflow: fret_ui::element::TextInkOverflow::None,
    })
}

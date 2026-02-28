use fret_core::Px;
use fret_ui::Theme;

#[derive(Debug, Clone, Copy)]
pub(super) struct MarkdownTheme {
    pub(super) link: fret_core::Color,
    pub(super) muted: fret_core::Color,
    pub(super) hr: fret_core::Color,
    pub(super) blockquote_border: fret_core::Color,
    pub(super) blockquote_border_width: Px,
    pub(super) blockquote_padding: Px,
    pub(super) list_indent: Px,
    pub(super) inline_code_fg: fret_core::Color,
    pub(super) inline_code_bg: fret_core::Color,
    pub(super) inline_code_padding_x: Px,
    pub(super) inline_code_padding_y: Px,
    pub(super) task_checked: fret_core::Color,
    pub(super) task_unchecked: fret_core::Color,
    pub(super) table_border: fret_core::Color,
    pub(super) table_header_bg: fret_core::Color,
    pub(super) table_cell_padding_x: Px,
    pub(super) table_cell_padding_y: Px,
    pub(super) inline_math_fg: fret_core::Color,
    pub(super) inline_math_bg: fret_core::Color,
    pub(super) inline_math_padding_x: Px,
    pub(super) inline_math_padding_y: Px,
    #[cfg(feature = "mathjax-svg")]
    pub(super) inline_math_height: Px,
    pub(super) math_block_fg: fret_core::Color,
    pub(super) math_block_bg: fret_core::Color,
    pub(super) math_block_padding: Px,
    #[cfg(feature = "mathjax-svg")]
    pub(super) math_block_height: Px,
}

impl MarkdownTheme {
    pub(super) fn resolve(theme: &Theme) -> Self {
        fn color(theme: &Theme, suffix: &str) -> Option<fret_core::Color> {
            theme
                .color_by_key(&format!("component.markdown.{suffix}"))
                .or_else(|| theme.color_by_key(&format!("fret.markdown.{suffix}")))
                .or_else(|| theme.color_by_key(&format!("markdown.{suffix}")))
        }

        fn metric(theme: &Theme, suffix: &str) -> Option<Px> {
            theme
                .metric_by_key(&format!("component.markdown.{suffix}"))
                .or_else(|| theme.metric_by_key(&format!("fret.markdown.{suffix}")))
                .or_else(|| theme.metric_by_key(&format!("markdown.{suffix}")))
        }

        let link = color(theme, "link").unwrap_or_else(|| theme.color_token("primary"));
        let muted = color(theme, "muted").unwrap_or_else(|| theme.color_token("muted-foreground"));
        let hr = color(theme, "hr").unwrap_or_else(|| theme.color_token("border"));

        let blockquote_border =
            color(theme, "blockquote.border").unwrap_or_else(|| theme.color_token("border"));
        let blockquote_border_width = metric(theme, "blockquote.border_width").unwrap_or(Px(3.0));
        let blockquote_padding = metric(theme, "blockquote.padding")
            .unwrap_or_else(|| theme.metric_token("metric.padding.sm"));

        let list_indent =
            metric(theme, "list.indent").unwrap_or_else(|| theme.metric_token("metric.padding.md"));

        let inline_code_fg =
            color(theme, "inline_code.fg").unwrap_or_else(|| theme.color_token("foreground"));
        let inline_code_bg =
            // Upstream shadcn docs style inline code as `bg-muted`.
            // Source: `repo-ref/ui/apps/v4/components/component-preview.tsx`.
            color(theme, "inline_code.bg").unwrap_or_else(|| theme.color_token("muted"));
        let inline_code_padding_x = metric(theme, "inline_code.padding_x").unwrap_or(Px(3.0));
        let inline_code_padding_y = metric(theme, "inline_code.padding_y").unwrap_or(Px(1.0));

        let task_checked =
            color(theme, "task.checked").unwrap_or_else(|| theme.color_token("primary"));
        let task_unchecked =
            color(theme, "task.unchecked").unwrap_or_else(|| theme.color_token("muted-foreground"));

        let table_border =
            color(theme, "table.border").unwrap_or_else(|| theme.color_token("border"));
        let table_header_bg =
            color(theme, "table.header_bg").unwrap_or_else(|| theme.color_token("muted"));
        let table_cell_padding_x = metric(theme, "table.cell.padding_x")
            .unwrap_or_else(|| theme.metric_token("metric.padding.sm"));
        let table_cell_padding_y = metric(theme, "table.cell.padding_y")
            .unwrap_or_else(|| Px(theme.metric_token("metric.padding.sm").0 * 0.5));

        let inline_math_fg = color(theme, "math.inline.fg").unwrap_or(inline_code_fg);
        let inline_math_bg = color(theme, "math.inline.bg").unwrap_or(inline_code_bg);
        let inline_math_padding_x =
            metric(theme, "math.inline.padding_x").unwrap_or(inline_code_padding_x);
        let inline_math_padding_y =
            metric(theme, "math.inline.padding_y").unwrap_or(inline_code_padding_y);
        #[cfg(feature = "mathjax-svg")]
        let inline_math_height = metric(theme, "math.inline.height")
            .unwrap_or_else(|| theme.metric_token("metric.font.line_height"));

        let math_block_fg =
            color(theme, "math.block.fg").unwrap_or_else(|| theme.color_token("foreground"));
        let math_block_bg =
            color(theme, "math.block.bg").unwrap_or_else(|| theme.color_token("card"));
        let math_block_padding = metric(theme, "math.block.padding")
            .unwrap_or_else(|| theme.metric_token("metric.padding.md"));
        #[cfg(feature = "mathjax-svg")]
        let math_block_height = metric(theme, "math.block.height").unwrap_or_else(|| {
            let font_size = theme.metric_token("metric.font.size").0;
            let line_height = theme.metric_token("metric.font.line_height").0;
            Px((line_height * 3.25).max(font_size * 4.0))
        });

        Self {
            link,
            muted,
            hr,
            blockquote_border,
            blockquote_border_width,
            blockquote_padding,
            list_indent,
            inline_code_fg,
            inline_code_bg,
            inline_code_padding_x,
            inline_code_padding_y,
            task_checked,
            task_unchecked,
            table_border,
            table_header_bg,
            table_cell_padding_x,
            table_cell_padding_y,
            inline_math_fg,
            inline_math_bg,
            inline_math_padding_x,
            inline_math_padding_y,
            #[cfg(feature = "mathjax-svg")]
            inline_math_height,
            math_block_fg,
            math_block_bg,
            math_block_padding,
            #[cfg(feature = "mathjax-svg")]
            math_block_height,
        }
    }
}

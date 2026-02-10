use std::sync::Arc;

use fret_core::{FontWeight, Px};
use fret_ui::element::{AnyElement, CrossAlign, FlexProps, MainAlign};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Space};
use fret_ui_kit::{MetricRef, ui};

/// A small, keyboard-first hint row: a keycap (`Kbd`) followed by a text label.
///
/// This intentionally keeps both parts on the same sizing baseline (height, padding, typography)
/// to avoid mixed-script (e.g. Latin + CJK) alignment drift in compact footers/toolbars.
#[derive(Debug, Clone)]
pub struct ShortcutHint {
    keys: Arc<str>,
    label: Arc<str>,
    layout: LayoutRefinement,
}

impl ShortcutHint {
    pub fn new(keys: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            keys: keys.into(),
            label: label.into(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        shortcut_hint_with_patch(cx, self.keys, self.label, self.layout)
    }
}

fn shortcut_hint_with_patch<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    keys: Arc<str>,
    label: Arc<str>,
    layout_override: LayoutRefinement,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();

    let kbd = crate::Kbd::new(keys).into_element(cx);
    let label = shortcut_hint_label(cx, &theme, label);

    let base_h = Px(20.0);
    let layout_override = LayoutRefinement::default()
        .h_px(base_h)
        .min_h(base_h)
        .merge(layout_override);
    let mut layout = decl_style::layout_style(&theme, layout_override);
    // Default to `flex-none` so hint blocks wrap instead of squishing unpredictably.
    layout.flex.grow = 0.0;
    layout.flex.shrink = 0.0;

    cx.flex(
        FlexProps {
            layout,
            direction: fret_core::Axis::Horizontal,
            gap: MetricRef::space(Space::N1).resolve(&theme),
            padding: fret_core::Edges::all(Px(0.0)),
            justify: MainAlign::Start,
            align: CrossAlign::Center,
            wrap: false,
        },
        move |_cx| vec![kbd, label],
    )
}

fn shortcut_hint_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    label: Arc<str>,
) -> AnyElement {
    let fg = theme.color_required("muted-foreground");

    let px = theme
        .metric_by_key("component.kbd.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_required("font.size"));
    let line_height = theme
        .metric_by_key("component.kbd.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_required("font.line_height"));

    let chrome = ChromeRefinement::default().px(Space::N1).py(Space::N0p5);
    let layout = LayoutRefinement::default().h_px(Px(20.0)).min_h(Px(20.0));
    let props = decl_style::container_props(theme, chrome, layout);

    cx.container(props, |cx| {
        vec![
            ui::h_flex(cx, |cx| {
                vec![
                    ui::label(cx, label)
                        .text_size_px(px)
                        .line_height_px(line_height)
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(ColorRef::Color(fg))
                        .h_px(line_height)
                        .into_element(cx),
                ]
            })
            .w_full()
            .h_full()
            .justify_center()
            .items_center()
            .into_element(cx),
        ]
    })
}

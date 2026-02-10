//! Material 3 chip set (roving-focus-friendly container).
//!
//! This is modeled after Material Web's `md-chip-set` keyboard behavior:
//! - ArrowLeft/ArrowRight roving focus across chips (RTL-aware).
//! - Home/End jump to first/last enabled chip.
//!
//! Note: Fret's `SemanticsRole` does not currently include a dedicated "toolbar" role, so we
//! expose the set as `Group`.

use std::sync::Arc;

use fret_core::{Axis, KeyCode, LayoutDirection, Modifiers, Px, SemanticsRole};
use fret_ui::UiHost;
use fret_ui::action::RovingNavigateResult;
use fret_ui::element::{AnyElement, RovingFlexProps, SemanticsProps};
use fret_ui::elements::ElementContext;
use fret_ui_kit::declarative::ElementContextThemeExt as _;

use crate::chip::AssistChip;
use crate::filter_chip::FilterChip;
use crate::foundation::context::{resolved_layout_direction, theme_default_layout_direction};
use crate::input_chip::InputChip;
use crate::suggestion_chip::SuggestionChip;

#[derive(Debug, Clone)]
pub enum ChipSetItem {
    Assist(AssistChip),
    Suggestion(SuggestionChip),
    Filter(FilterChip),
    Input(InputChip),
}

impl From<AssistChip> for ChipSetItem {
    fn from(value: AssistChip) -> Self {
        Self::Assist(value)
    }
}

impl From<SuggestionChip> for ChipSetItem {
    fn from(value: SuggestionChip) -> Self {
        Self::Suggestion(value)
    }
}

impl From<FilterChip> for ChipSetItem {
    fn from(value: FilterChip) -> Self {
        Self::Filter(value)
    }
}

impl From<InputChip> for ChipSetItem {
    fn from(value: InputChip) -> Self {
        Self::Input(value)
    }
}

impl ChipSetItem {
    fn disabled_for_roving(&self) -> bool {
        match self {
            ChipSetItem::Assist(chip) => chip.disabled_for_roving(),
            ChipSetItem::Suggestion(chip) => chip.disabled_for_roving(),
            ChipSetItem::Filter(chip) => chip.disabled_for_roving(),
            ChipSetItem::Input(chip) => chip.disabled_for_roving(),
        }
    }

    fn into_element_with_tab_stop<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        tab_stop: bool,
    ) -> AnyElement {
        match self {
            ChipSetItem::Assist(chip) => chip.roving_tab_stop(tab_stop).into_element(cx),
            ChipSetItem::Suggestion(chip) => chip.roving_tab_stop(tab_stop).into_element(cx),
            ChipSetItem::Filter(chip) => chip.roving_tab_stop(tab_stop).into_element(cx),
            ChipSetItem::Input(chip) => chip.roving_tab_stop(tab_stop).into_element(cx),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChipSet {
    items: Vec<ChipSetItem>,
    disabled: bool,
    gap: Px,
    wrap_layout: bool,
    loop_navigation: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
}

impl Default for ChipSet {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            disabled: false,
            gap: Px(8.0),
            wrap_layout: false,
            loop_navigation: true,
            a11y_label: None,
            test_id: None,
        }
    }
}

impl ChipSet {
    pub fn new(items: Vec<ChipSetItem>) -> Self {
        Self {
            items,
            ..Default::default()
        }
    }

    pub fn items(mut self, items: Vec<ChipSetItem>) -> Self {
        self.items = items;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn gap(mut self, gap: Px) -> Self {
        self.gap = gap;
        self
    }

    pub fn wrap_layout(mut self, wrap_layout: bool) -> Self {
        self.wrap_layout = wrap_layout;
        self
    }

    pub fn loop_navigation(mut self, loop_navigation: bool) -> Self {
        self.loop_navigation = loop_navigation;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let ChipSet {
            items,
            disabled,
            gap,
            wrap_layout,
            loop_navigation,
            a11y_label,
            test_id,
        } = self;

        let default_layout_direction = cx.with_theme(|theme| theme_default_layout_direction(theme));
        let layout_direction = resolved_layout_direction(cx, default_layout_direction);

        let disabled_items: Arc<[bool]> = Arc::from(
            items
                .iter()
                .map(|it| disabled || it.disabled_for_roving())
                .collect::<Vec<_>>(),
        );

        let tab_stop = items
            .iter()
            .position(|it| !disabled && !it.disabled_for_roving());

        let sem = SemanticsProps {
            role: SemanticsRole::Group,
            label: a11y_label.clone(),
            test_id: test_id.clone(),
            disabled,
            ..Default::default()
        };

        let mut props = RovingFlexProps::default();
        props.flex.direction = Axis::Horizontal;
        props.flex.gap = gap;
        props.flex.wrap = wrap_layout;
        props.roving.enabled = !disabled;
        props.roving.wrap = loop_navigation;
        props.roving.disabled = disabled_items.clone();

        cx.semantics(sem, move |cx| {
            let items = items;
            vec![cx.roving_flex(props, move |cx| {
                cx.roving_on_navigate(Arc::new(move |_host, _cx, it| {
                    if it.repeat || it.modifiers != Modifiers::default() {
                        return RovingNavigateResult::NotHandled;
                    }
                    if it.axis != Axis::Horizontal {
                        return RovingNavigateResult::NotHandled;
                    }

                    let is_disabled =
                        |idx: usize| -> bool { it.disabled.get(idx).copied().unwrap_or(false) };

                    if it.key == KeyCode::Home {
                        let target = (0..it.len).find(|&i| !is_disabled(i));
                        return RovingNavigateResult::Handled { target };
                    }
                    if it.key == KeyCode::End {
                        let target = (0..it.len).rev().find(|&i| !is_disabled(i));
                        return RovingNavigateResult::Handled { target };
                    }

                    let forward = match (layout_direction, it.key) {
                        (LayoutDirection::Ltr, KeyCode::ArrowRight) => Some(true),
                        (LayoutDirection::Ltr, KeyCode::ArrowLeft) => Some(false),
                        (LayoutDirection::Rtl, KeyCode::ArrowLeft) => Some(true),
                        (LayoutDirection::Rtl, KeyCode::ArrowRight) => Some(false),
                        _ => None,
                    };
                    let Some(forward) = forward else {
                        return RovingNavigateResult::NotHandled;
                    };

                    let current = it
                        .current
                        .or_else(|| (0..it.len).find(|&i| !is_disabled(i)));
                    let Some(current) = current else {
                        return RovingNavigateResult::Handled { target: None };
                    };

                    let len = it.len;
                    let mut target: Option<usize> = None;
                    if it.wrap {
                        for step in 1..=len {
                            let idx = if forward {
                                (current + step) % len
                            } else {
                                (current + len - (step % len)) % len
                            };
                            if !is_disabled(idx) {
                                target = Some(idx);
                                break;
                            }
                        }
                    } else if forward {
                        target = ((current + 1)..len).find(|&i| !is_disabled(i));
                    } else if current > 0 {
                        target = (0..current).rev().find(|&i| !is_disabled(i));
                    }

                    RovingNavigateResult::Handled { target }
                }));

                items
                    .into_iter()
                    .enumerate()
                    .map(|(idx, it)| it.into_element_with_tab_stop(cx, tab_stop == Some(idx)))
                    .collect::<Vec<_>>()
            })]
        })
    }
}

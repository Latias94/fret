use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use fret_core::{Color, Corners, Edges, FontId, FontWeight, Px, SemanticsRole, TextStyle};
use fret_icons::ids;
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PressableA11y, PressableProps,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::combobox as kit_combobox;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, LayoutRefinement, MetricRef, Radius, ShadowPreset,
    Size, Space, WidgetState, WidgetStateProperty, WidgetStates, resolve_override_slot, ui,
};

use crate::combobox::{
    ComboboxChip, ComboboxChipsInput, ComboboxContent, ComboboxContentPart,
    ComboboxGroup as V4ComboboxGroup, ComboboxItem as V4ComboboxItem, ComboboxOpenChangeReason,
    ComboboxStyle, ComboboxValue,
};
use crate::combobox_data::{ComboboxOption, ComboboxOptionGroup};
use crate::command::CommandPaletteA11ySelectedMode;
use crate::{
    CommandEntry, CommandGroup, CommandItem, CommandPalette, CommandSeparator, Popover,
    PopoverContent,
};

/// Part-based authoring surface aligned with shadcn/ui v4 exports.
#[derive(Debug)]
pub enum ComboboxChipsPart {
    Value(ComboboxValue),
    ChipsInput(ComboboxChipsInput),
    Content(ComboboxContent),
}

impl From<ComboboxValue> for ComboboxChipsPart {
    fn from(value: ComboboxValue) -> Self {
        Self::Value(value)
    }
}

impl From<ComboboxChipsInput> for ComboboxChipsPart {
    fn from(value: ComboboxChipsInput) -> Self {
        Self::ChipsInput(value)
    }
}

impl From<ComboboxContent> for ComboboxChipsPart {
    fn from(value: ComboboxContent) -> Self {
        Self::Content(value)
    }
}

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn test_id_slug(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

#[derive(Default)]
struct ComboboxChipsState {
    query: Option<Model<String>>,
    open_change_reason: Option<Model<Option<ComboboxOpenChangeReason>>>,
    clear_query_on_close: kit_combobox::ClearQueryOnCloseState,
    focus_restore_target: Option<Arc<Mutex<Option<GlobalElementId>>>>,
}

#[derive(Clone)]
pub struct ComboboxChips {
    values: Model<Vec<Arc<str>>>,
    open: Model<bool>,
    query: Option<Model<String>>,
    items: Vec<ComboboxOption>,
    groups: Vec<ComboboxOptionGroup>,
    test_id_prefix: Option<Arc<str>>,
    trigger_test_id: Option<Arc<str>>,
    width: Option<Px>,
    placeholder: Arc<str>,
    search_placeholder: Arc<str>,
    empty_text: Arc<str>,
    disabled: bool,
    chip_show_remove: bool,
    a11y_label: Option<Arc<str>>,
    consume_outside_pointer_events: bool,
    close_on_commit: bool,
    clear_query_on_commit: bool,
    close_auto_focus_policy: kit_combobox::ComboboxCloseAutoFocusPolicy,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    style: ComboboxStyle,
}

impl ComboboxChips {
    pub fn new(values: Model<Vec<Arc<str>>>, open: Model<bool>) -> Self {
        Self {
            values,
            open,
            query: None,
            items: Vec::new(),
            groups: Vec::new(),
            test_id_prefix: None,
            trigger_test_id: None,
            width: None,
            placeholder: Arc::from("Select..."),
            search_placeholder: Arc::from("Search..."),
            empty_text: Arc::from("No results."),
            disabled: false,
            chip_show_remove: true,
            a11y_label: None,
            consume_outside_pointer_events: false,
            close_on_commit: false,
            clear_query_on_commit: true,
            close_auto_focus_policy: kit_combobox::ComboboxCloseAutoFocusPolicy::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: ComboboxStyle::default(),
        }
    }

    pub fn query_model(mut self, query: Model<String>) -> Self {
        self.query = Some(query);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = ComboboxOption>) -> Self {
        self.items.extend(items);
        self
    }

    /// Migration-friendly alias for [`ComboboxChips::items`].
    pub fn options(self, options: impl IntoIterator<Item = ComboboxOption>) -> Self {
        self.items(options)
    }

    pub fn group(mut self, group: ComboboxOptionGroup) -> Self {
        self.groups.push(group);
        self
    }

    /// Migration-friendly alias for [`ComboboxChips::group`].
    pub fn option_group(self, group: ComboboxOptionGroup) -> Self {
        self.group(group)
    }

    pub fn groups(mut self, groups: impl IntoIterator<Item = ComboboxOptionGroup>) -> Self {
        self.groups.extend(groups);
        self
    }

    /// Migration-friendly alias for [`ComboboxChips::groups`].
    pub fn option_groups(self, groups: impl IntoIterator<Item = ComboboxOptionGroup>) -> Self {
        self.groups(groups)
    }

    pub fn test_id_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        self.test_id_prefix = Some(prefix.into());
        self
    }

    pub fn trigger_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.trigger_test_id = Some(id.into());
        self
    }

    pub fn width(mut self, width: Px) -> Self {
        self.width = Some(width);
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn search_placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.search_placeholder = placeholder.into();
        self
    }

    pub fn empty_text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.empty_text = text.into();
        self
    }

    /// Controls whether selected value chips render an inline remove button.
    ///
    /// Upstream shadcn/ui v4 exposes this via the `ComboboxChip` `showRemove` prop.
    pub fn chip_show_remove(mut self, show_remove: bool) -> Self {
        self.chip_show_remove = show_remove;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn consume_outside_pointer_events(mut self, consume: bool) -> Self {
        self.consume_outside_pointer_events = consume;
        self
    }

    pub fn close_on_commit(mut self, close: bool) -> Self {
        self.close_on_commit = close;
        self
    }

    pub fn clear_query_on_commit(mut self, clear: bool) -> Self {
        self.clear_query_on_commit = clear;
        self
    }

    pub fn close_auto_focus_policy(
        mut self,
        policy: kit_combobox::ComboboxCloseAutoFocusPolicy,
    ) -> Self {
        self.close_auto_focus_policy = policy;
        self
    }

    pub fn refine_style(mut self, patch: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(patch);
        self
    }

    pub fn refine_layout(mut self, patch: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(patch);
        self
    }

    pub fn refine_recipe_style(mut self, style: ComboboxStyle) -> Self {
        self.style = self.style.clone().merged(style);
        self
    }

    /// Render the chips combobox using shadcn/ui v4 part-based composition.
    ///
    /// This is a compatibility adapter that maps v4-named parts onto Fret's `ComboboxChips`
    /// recipe. The adapter focuses on copy/paste parity for docs examples and does not attempt to
    /// emulate Base UI's DOM-first `ComboboxChipsInput` behavior.
    #[track_caller]
    pub fn into_element_parts<H: UiHost>(
        mut self,
        cx: &mut ElementContext<'_, H>,
        parts: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<ComboboxChipsPart>,
    ) -> AnyElement {
        apply_parts_patch_to_chips(&mut self, parts(cx));

        self.into_element(cx)
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        combobox_chips_with_patch(
            cx,
            self.values,
            self.open,
            self.query,
            &self.items,
            &self.groups,
            self.test_id_prefix,
            self.trigger_test_id,
            self.width,
            self.placeholder,
            self.search_placeholder,
            self.empty_text,
            self.disabled,
            self.chip_show_remove,
            self.a11y_label,
            self.consume_outside_pointer_events,
            self.close_on_commit,
            self.clear_query_on_commit,
            self.close_auto_focus_policy,
            self.chrome,
            self.layout,
            self.style,
        )
    }
}

fn apply_parts_patch_to_chips(chips: &mut ComboboxChips, parts: Vec<ComboboxChipsPart>) {
    for part in parts {
        match part {
            ComboboxChipsPart::Value(value) => {
                if value
                    .chips
                    .iter()
                    .any(|chip| !chip.show_remove && !chip.value.is_empty())
                {
                    chips.chip_show_remove = false;
                }
            }
            ComboboxChipsPart::ChipsInput(input) => {
                if let Some(placeholder) = input.placeholder {
                    // In Base UI, chips input is the editable surface used to filter items.
                    // Fret renders the filter input in the overlay, so we map the placeholder to
                    // `search_placeholder` for the closest outcome.
                    chips.search_placeholder = placeholder;
                }
            }
            ComboboxChipsPart::Content(content) => {
                apply_v4_content_patch_to_chips(chips, content);
            }
        }
    }
}

fn apply_v4_content_patch_to_chips(chips: &mut ComboboxChips, content: ComboboxContent) {
    for child in content.children {
        match child {
            ComboboxContentPart::Input(input) => {
                if let Some(placeholder) = input.placeholder {
                    chips.search_placeholder = placeholder;
                }
            }
            ComboboxContentPart::Empty(empty) => {
                chips.empty_text = empty.text;
            }
            ComboboxContentPart::Separator(_) => {}
            ComboboxContentPart::List(list) => {
                if !list.items.is_empty() {
                    chips.items = list
                        .items
                        .into_iter()
                        .map(v4_item_to_option)
                        .collect::<Vec<_>>();
                }

                if !list.groups.is_empty() {
                    chips.groups = list
                        .groups
                        .into_iter()
                        .filter_map(v4_group_to_option_group)
                        .collect::<Vec<_>>();
                }
            }
        }
    }
}

fn v4_item_to_option(item: V4ComboboxItem) -> ComboboxOption {
    let mut option = ComboboxOption::new(item.value.clone(), item.label.clone())
        .disabled(item.disabled)
        .keywords(item.keywords);
    if let Some(detail) = item.detail {
        option = option.detail(detail);
    }
    option
}

fn v4_group_to_option_group(group: V4ComboboxGroup) -> Option<ComboboxOptionGroup> {
    let heading = group.label.map(|label| label.text)?;
    let items = if !group.items.is_empty() {
        group.items
    } else {
        group.collection.map(|c| c.items).unwrap_or_default()
    };
    if items.is_empty() {
        return None;
    }
    let items = items.into_iter().map(v4_item_to_option).collect::<Vec<_>>();
    Some(ComboboxOptionGroup::new(heading, items))
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;

    #[test]
    fn combobox_chips_parts_patch_maps_search_placeholder_and_chip_remove() {
        let mut app = App::new();
        let values = app.models_mut().insert(Vec::<Arc<str>>::new());
        let open = app.models_mut().insert(false);
        let mut chips = ComboboxChips::new(values, open);

        let content = ComboboxContent::new([
            ComboboxContentPart::empty(crate::combobox::ComboboxEmpty::new("Nothing found."))
                .into(),
            ComboboxContentPart::list(
                crate::combobox::ComboboxList::new()
                    .items([crate::combobox::ComboboxItem::new("a", "Alpha")])
                    .groups([crate::combobox::ComboboxGroup::new()
                        .label(crate::combobox::ComboboxLabel::new("Group 1"))
                        .items([crate::combobox::ComboboxItem::new("b", "Beta")])]),
            )
            .into(),
        ]);

        apply_parts_patch_to_chips(
            &mut chips,
            vec![
                ComboboxValue::new([ComboboxChip::new("a").show_remove(false)]).into(),
                ComboboxChipsInput::new()
                    .placeholder("Add framework")
                    .into(),
                content.into(),
            ],
        );

        assert_eq!(chips.search_placeholder.as_ref(), "Add framework");
        assert_eq!(chips.empty_text.as_ref(), "Nothing found.");
        assert!(!chips.chip_show_remove);
        assert_eq!(chips.items.len(), 1);
        assert_eq!(chips.items[0].value.as_ref(), "a");
        assert_eq!(chips.items[0].label.as_ref(), "Alpha");
        assert_eq!(chips.groups.len(), 1);
        assert_eq!(chips.groups[0].heading.as_ref(), "Group 1");
        assert_eq!(chips.groups[0].items.len(), 1);
        assert_eq!(chips.groups[0].items[0].value.as_ref(), "b");
    }
}

#[allow(clippy::too_many_arguments)]
fn combobox_chips_with_patch<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    values: Model<Vec<Arc<str>>>,
    open: Model<bool>,
    query: Option<Model<String>>,
    items: &[ComboboxOption],
    groups: &[ComboboxOptionGroup],
    test_id_prefix: Option<Arc<str>>,
    trigger_test_id: Option<Arc<str>>,
    width: Option<Px>,
    placeholder: Arc<str>,
    search_placeholder: Arc<str>,
    empty_text: Arc<str>,
    disabled: bool,
    chip_show_remove: bool,
    a11y_label: Option<Arc<str>>,
    consume_outside_pointer_events: bool,
    close_on_commit: bool,
    clear_query_on_commit: bool,
    close_auto_focus_policy: kit_combobox::ComboboxCloseAutoFocusPolicy,
    chrome_patch: ChromeRefinement,
    layout_patch: LayoutRefinement,
    style_override: ComboboxStyle,
) -> AnyElement {
    cx.scope(|cx| {
        let theme = Theme::global(&*cx.app).snapshot();
        let open_change_reason_model = {
            let existing = cx.with_state(ComboboxChipsState::default, |st| {
                st.open_change_reason.clone()
            });
            if let Some(model) = existing {
                model
            } else {
                let model = cx.app.models_mut().insert(None::<ComboboxOpenChangeReason>);
                cx.with_state(ComboboxChipsState::default, |st| {
                    st.open_change_reason = Some(model.clone())
                });
                model
            }
        };
        let focus_restore_target = {
            let existing = cx.with_state(ComboboxChipsState::default, |st| {
                st.focus_restore_target.clone()
            });
            if let Some(cell) = existing {
                cell
            } else {
                let cell: Arc<Mutex<Option<GlobalElementId>>> = Arc::new(Mutex::new(None));
                cx.with_state(ComboboxChipsState::default, |st| {
                    st.focus_restore_target = Some(cell.clone());
                });
                cell
            }
        };
        let close_auto_focus = kit_combobox::on_close_auto_focus_with_reason(
            open_change_reason_model.clone(),
            focus_restore_target.clone(),
            close_auto_focus_policy,
        );

        let _selected_values = cx.watch_model(&values).cloned().unwrap_or_default();
        let is_open = cx.watch_model(&open).layout().copied().unwrap_or(false);

        let query_model = if let Some(q) = query {
            cx.with_state(ComboboxChipsState::default, |st| st.query = Some(q.clone()));
            q
        } else {
            let existing = cx.with_state(ComboboxChipsState::default, |st| st.query.clone());
            if let Some(m) = existing {
                m
            } else {
                let m = cx.app.models_mut().insert(String::new());
                cx.with_state(ComboboxChipsState::default, |st| st.query = Some(m.clone()));
                m
            }
        };

        let should_clear_query = cx.with_state(ComboboxChipsState::default, |st| {
            kit_combobox::should_clear_query_on_close(&mut st.clear_query_on_close, is_open)
        });
        if should_clear_query {
            let _ = cx.app.models_mut().update(&query_model, |v| v.clear());
        }

        let search_input_id = Rc::new(Cell::new(None));
        let popover = Popover::new(open.clone())
            .auto_focus(true)
            .consume_outside_pointer_events(consume_outside_pointer_events)
            .on_dismiss_request(Some(
                kit_combobox::set_open_change_reason_on_dismiss_request(
                    open_change_reason_model.clone(),
                ),
            ))
            .on_close_auto_focus(Some(close_auto_focus.clone()))
            .initial_focus_from_cell(search_input_id.clone());

        popover.into_element_with_anchor(
            cx,
            {
                let theme = theme.clone();
                let focus_restore_target = focus_restore_target.clone();
                let open_change_reason_model = open_change_reason_model.clone();
                let open_for_trigger = open.clone();
                let values_for_trigger = values.clone();
                let selected_values_for_trigger = _selected_values.clone();
                let items_for_trigger: Vec<ComboboxOption> = items.to_vec();
                let groups_for_trigger: Vec<ComboboxOptionGroup> = groups.to_vec();
                let test_id_prefix_for_trigger = test_id_prefix.clone();
                let trigger_test_id_for_trigger = trigger_test_id.clone();
                let placeholder_for_trigger = placeholder.clone();
                let a11y_label_for_trigger = a11y_label.clone();
                let chip_show_remove_for_trigger = chip_show_remove;

                let size = Size::default();
                let (control_radius, control_text_px, button_h, button_px, button_py) = {
                    let theme_full = Theme::global(&*cx.app);
                    (
                        size.control_radius(theme_full),
                        size.control_text_px(theme_full),
                        size.button_h(theme_full),
                        size.button_px(theme_full),
                        size.button_py(theme_full),
                    )
                };
                let radius = chrome_patch
                    .radius
                    .as_ref()
                    .map(|m| m.resolve(&theme))
                    .unwrap_or(control_radius);
                let ring = decl_style::focus_ring(&theme, radius);

                let text_style = TextStyle {
                    font: FontId::default(),
                    size: control_text_px,
                    weight: FontWeight::MEDIUM,
                    line_height: theme
                        .metric_by_key("font.line_height")
                        .or(Some(theme.metric_token("font.line_height"))),
                    ..Default::default()
                };

                let min_h = chrome_patch
                    .min_height
                    .as_ref()
                    .map(|m| m.resolve(&theme))
                    .unwrap_or(button_h);
                let pad_x = button_px;
                let pad_y = button_py;
                let border_w = chrome_patch
                    .border_width
                    .as_ref()
                    .map(|m| m.resolve(&theme))
                    .unwrap_or(Px(1.0));

                let mut trigger_layout = decl_style::layout_style(
                    &theme,
                    LayoutRefinement::default()
                        .min_h(min_h)
                        .merge(if let Some(w) = width {
                            LayoutRefinement::default().w_px(w)
                        } else {
                            LayoutRefinement::default().w_full()
                        })
                        .merge(layout_patch),
                );
                trigger_layout.size.height = Length::Auto;
                trigger_layout.size.min_height = Some(Length::Px(min_h));

                let bg_base = chrome_patch
                    .background
                    .as_ref()
                    .map(|c| c.resolve(&theme))
                    .unwrap_or_else(|| {
                        theme
                            .color_by_key("background")
                            .unwrap_or_else(|| theme.color_token("background"))
                    });
                let bg_hover = theme
                    .color_by_key("accent")
                    .or_else(|| theme.color_by_key("accent.background"))
                    .unwrap_or_else(|| theme.color_token("accent"));
                let bg_pressed = theme.color_token("accent");
                let fg_base = chrome_patch
                    .text_color
                    .as_ref()
                    .map(|c| c.resolve(&theme))
                    .unwrap_or_else(|| {
                        theme
                            .color_by_key("foreground")
                            .unwrap_or_else(|| theme.color_token("foreground"))
                    });
                let fg_hover = theme
                    .color_by_key("accent-foreground")
                    .or_else(|| theme.color_by_key("accent.foreground"))
                    .unwrap_or(fg_base);
                let muted_fg = theme
                    .color_by_key("muted-foreground")
                    .or_else(|| theme.color_by_key("muted_foreground"))
                    .unwrap_or(fg_base);
                let border_base = chrome_patch
                    .border_color
                    .as_ref()
                    .map(|c| c.resolve(&theme))
                    .unwrap_or_else(|| {
                        theme
                            .color_by_key("input")
                            .or_else(|| theme.color_by_key("border"))
                            .unwrap_or_else(|| theme.color_token("border"))
                    });
                let ring_border = theme.color_token("ring");

                let default_trigger_bg = WidgetStateProperty::new(ColorRef::Color(bg_base))
                    .when(WidgetStates::HOVERED, ColorRef::Color(bg_hover))
                    .when(WidgetStates::ACTIVE, ColorRef::Color(bg_pressed));
                let default_trigger_fg = WidgetStateProperty::new(ColorRef::Color(fg_base))
                    .when(WidgetStates::HOVERED, ColorRef::Color(fg_hover))
                    .when(WidgetStates::ACTIVE, ColorRef::Color(fg_hover));
                let default_trigger_border = WidgetStateProperty::new(ColorRef::Color(border_base))
                    .when(WidgetStates::FOCUS_VISIBLE, ColorRef::Color(ring_border));

                let enabled = !disabled;
                let trigger_gap = MetricRef::space(Space::N2).resolve(&theme);
                let chip_gap = MetricRef::space(Space::N1).resolve(&theme);

                let padding = chrome_patch.padding.clone().unwrap_or_default();
                let pad_top = padding.top.map(|m| m.resolve(&theme)).unwrap_or(pad_y);
                let pad_right = padding.right.map(|m| m.resolve(&theme)).unwrap_or(pad_x);
                let pad_bottom = padding.bottom.map(|m| m.resolve(&theme)).unwrap_or(pad_y);
                let pad_left = padding.left.map(|m| m.resolve(&theme)).unwrap_or(pad_x);

                move |cx| {
                    control_chrome_pressable_with_id_props(cx, |cx, st, trigger_id| {
                        *focus_restore_target
                            .lock()
                            .unwrap_or_else(|e| e.into_inner()) = Some(trigger_id);

                        let mut states = WidgetStates::from_pressable(cx, st, enabled);
                        states.set(WidgetState::Open, is_open);

                        let bg_ref = resolve_override_slot(
                            style_override.trigger_background.as_ref(),
                            &default_trigger_bg,
                            states,
                        );
                        let fg_ref = resolve_override_slot(
                            style_override.trigger_foreground.as_ref(),
                            &default_trigger_fg,
                            states,
                        );
                        let border_ref = resolve_override_slot(
                            style_override.trigger_border_color.as_ref(),
                            &default_trigger_border,
                            states,
                        );

                        let bg = bg_ref.resolve(&theme);
                        let fg = fg_ref.resolve(&theme);
                        let border = border_ref.resolve(&theme);
                        let icon_fg = alpha_mul(fg, 0.5);

                        cx.pressable_add_on_activate(
                            kit_combobox::set_open_change_reason_on_activate(
                                open_change_reason_model.clone(),
                                ComboboxOpenChangeReason::TriggerPress,
                            ),
                        );
                        cx.pressable_toggle_bool(&open_for_trigger);

                        let props = PressableProps {
                            layout: trigger_layout,
                            enabled,
                            focusable: true,
                            focus_ring: Some(ring),
                            a11y: PressableA11y {
                                role: Some(SemanticsRole::ComboBox),
                                label: a11y_label_for_trigger
                                    .clone()
                                    .or_else(|| Some(placeholder_for_trigger.clone())),
                                test_id: trigger_test_id_for_trigger.clone(),
                                expanded: Some(is_open),
                                ..Default::default()
                            },
                            ..Default::default()
                        };

                        let chrome_props = ContainerProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size = trigger_layout.size;
                                layout
                            },
                            padding: Edges {
                                top: pad_top,
                                right: pad_right,
                                bottom: pad_bottom,
                                left: pad_left,
                            }
                            .into(),
                            background: Some(bg),
                            shadow: None,
                            border: Edges::all(border_w),
                            border_color: Some(border),
                            corner_radii: Corners::all(radius),
                            ..Default::default()
                        };

                        (props, chrome_props, move |cx| {
                            let label_style = text_style.clone();
                            let chip_prefix = test_id_prefix_for_trigger
                                .clone()
                                .unwrap_or_else(|| Arc::from("combobox"));

                            let chips = cx.flex(
                                FlexProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Fill;
                                        layout
                                    },
                                    direction: fret_core::Axis::Horizontal,
                                    gap: chip_gap.into(),
                                    padding: Edges::all(Px(0.0)).into(),
                                    justify: MainAlign::Start,
                                    align: CrossAlign::Center,
                                    wrap: true,
                                },
                                move |cx| {
                                    let mut out: Vec<AnyElement> = Vec::new();
                                    if selected_values_for_trigger.is_empty() {
                                        out.push(
                                            ui::label(cx, placeholder_for_trigger.clone())
                                                .text_size_px(label_style.size)
                                                .font_weight(label_style.weight)
                                                .text_color(ColorRef::Color(muted_fg))
                                                .truncate()
                                                .into_element(cx),
                                        );
                                        return out;
                                    }

                                    let chip_bg = theme
                                        .color_by_key("muted")
                                        .or_else(|| theme.color_by_key("muted.background"))
                                        .unwrap_or_else(|| theme.color_token("muted"));
                                    let chip_fg = theme
                                        .color_by_key("foreground")
                                        .unwrap_or_else(|| theme.color_token("foreground"));

                                    for value in selected_values_for_trigger.iter().cloned() {
                                        let slug = test_id_slug(value.as_ref());
                                        let chip_test_id =
                                            format!("{chip_prefix}-chip-{slug}");
                                        let chip_remove_test_id =
                                            format!("{chip_prefix}-chip-{slug}-remove");
                                        let values_for_trigger_for_chip =
                                            values_for_trigger.clone();
                                        let label = items_for_trigger
                                            .iter()
                                            .chain(
                                                groups_for_trigger
                                                    .iter()
                                                    .flat_map(|g| g.items.iter()),
                                            )
                                            .find(|it| it.value.as_ref() == value.as_ref())
                                            .map(|it| it.label.clone())
                                            .unwrap_or_else(|| value.clone());

                                        let chip_props = ContainerProps {
                                            layout: LayoutStyle::default(),
                                            padding: Edges {
                                                top: Px(2.0),
                                                right: Px(4.0),
                                                bottom: Px(2.0),
                                                left: Px(6.0),
                                            }
                                            .into(),
                                            background: Some(chip_bg),
                                            corner_radii: Corners::all(Px(4.0)),
                                            ..Default::default()
                                        };
                                        out.push(
                                            cx.container(chip_props, move |cx| {
                                                vec![cx.flex(
                                                    FlexProps {
                                                        layout: LayoutStyle::default(),
                                                        direction: fret_core::Axis::Horizontal,
                                                        gap: Px(4.0).into(),
                                                        padding: Edges::all(Px(0.0)).into(),
                                                        justify: MainAlign::Start,
                                                        align: CrossAlign::Center,
                                                        wrap: false,
                                                    },
                                                    move |cx| {
                                                        let mut out = vec![
                                                            ui::label(cx, label.clone())
                                                                .text_size_px(Px(12.0))
                                                                .font_weight(FontWeight::MEDIUM)
                                                                .text_color(ColorRef::Color(
                                                                    chip_fg,
                                                                ))
                                                                .truncate()
                                                                .into_element(cx),
                                                        ];
                                                        if chip_show_remove_for_trigger {
                                                            let values_for_remove =
                                                                values_for_trigger_for_chip
                                                                    .clone();
                                                            let value_for_remove = value.clone();
                                                            out.push(
                                                                control_chrome_pressable_with_id_props(
                                                                    cx,
                                                                    move |cx, _st, _id| {
                                                                        cx.pressable_add_on_activate(
                                                                            Arc::new(
                                                                                move |host,
                                                                                      action_cx,
                                                                                      _reason| {
                                                                                    let _ = host
                                                                                        .models_mut()
                                                                                        .update(
                                                                                            &values_for_remove,
                                                                                            |values| {
                                                                                                values.retain(
                                                                                                    |v| {
                                                                                                        v.as_ref()
                                                                                                            != value_for_remove.as_ref()
                                                                                                    },
                                                                                                );
                                                                                            },
                                                                                        );
                                                                                    host.request_redraw(action_cx.window);
                                                                                },
                                                                            ),
                                                                        );

                                                                        let props = PressableProps {
                                                                            layout: {
                                                                                let mut layout =
                                                                                    LayoutStyle::default();
                                                                                layout.size.width =
                                                                                    Length::Px(Px(20.0));
                                                                                layout.size.height =
                                                                                    Length::Px(Px(20.0));
                                                                                layout
                                                                            },
                                                                            enabled,
                                                                            focusable: true,
                                                                            a11y: PressableA11y {
                                                                                role: Some(
                                                                                    SemanticsRole::Button,
                                                                                ),
                                                                                label: Some(
                                                                                    Arc::from("Remove"),
                                                                                ),
                                                                                ..Default::default()
                                                                            },
                                                                            ..Default::default()
                                                                        };
                                                                        let chrome_props =
                                                                            ContainerProps {
                                                                                layout:
                                                                                    LayoutStyle::default(),
                                                                                background: Some(
                                                                                    Color::TRANSPARENT,
                                                                                ),
                                                                                corner_radii:
                                                                                    Corners::all(Px(3.0)),
                                                                                ..Default::default()
                                                                            };
                                                                        let children =
                                                                            move |cx: &mut ElementContext<
                                                                                '_,
                                                                                H,
                                                                            >| {
                                                                                let icon =
                                                                                    decl_icon::icon_with(
                                                                                        cx,
                                                                                        ids::ui::CLOSE,
                                                                                        Some(Px(14.0)),
                                                                                        Some(ColorRef::Color(alpha_mul(
                                                                                            chip_fg,
                                                                                            0.6,
                                                                                        ))),
                                                                                    );
                                                                                vec![cx.flex(
                                                                                    FlexProps {
                                                                                        layout: LayoutStyle::default(),
                                                                                        direction:
                                                                                            fret_core::Axis::Horizontal,
                                                                                        gap: Px(0.0).into(),
                                                                                        padding: Edges::all(Px(0.0))
                                                                                            .into(),
                                                                                        justify: MainAlign::Center,
                                                                                        align: CrossAlign::Center,
                                                                                        wrap: false,
                                                                                    },
                                                                                    move |_cx| vec![icon],
                                                                                )]
                                                                            };
                                                                        (props, chrome_props, children)
                                                                    },
                                                                )
                                                                .test_id(chip_remove_test_id.clone()),
                                                            );
                                                        }
                                                        out
                                                    },
                                                )]
                                            })
                                            .test_id(chip_test_id),
                                        );
                                    }

                                    out
                                },
                            );

                            let right = decl_icon::icon_with(
                                cx,
                                ids::ui::CHEVRON_DOWN,
                                Some(Px(16.0)),
                                Some(ColorRef::Color(icon_fg)),
                            );

                            vec![cx.flex(
                                FlexProps {
                                    layout: LayoutStyle::default(),
                                    direction: fret_core::Axis::Horizontal,
                                    gap: trigger_gap.into(),
                                    padding: Edges::all(Px(0.0)).into(),
                                    justify: MainAlign::SpaceBetween,
                                    align: CrossAlign::Center,
                                    wrap: false,
                                },
                                move |_cx| vec![chips, right],
                            )]
                        })
                    })
                }
            },
            {
                let theme = theme.clone();
                let values = values.clone();
                let open = open.clone();
                let query_model = query_model.clone();
                let open_change_reason_model = open_change_reason_model.clone();
                let selected_values = _selected_values.clone();
                let items: Vec<ComboboxOption> = items.to_vec();
                let groups: Vec<ComboboxOptionGroup> = groups.to_vec();
                let test_id_prefix = test_id_prefix.clone();
                let search_placeholder = search_placeholder.clone();
                let empty_text = empty_text.clone();

                move |cx, anchor| {
                    let max_list_h = Px(theme
                        .metric_by_key("list.max_height")
                        .or_else(|| theme.metric_by_key("combobox.list.max_height"))
                        .unwrap_or(Px(280.0))
                        .0
                        .max(0.0));

                    let popover_surface = ChromeRefinement::default()
                        .rounded(Radius::Md)
                        .border_width(Px(1.0))
                        .border_color(ColorRef::Token {
                            key: "border",
                            fallback: ColorFallback::ThemePanelBorder,
                        })
                        .bg(ColorRef::Token {
                            key: "popover.background",
                            fallback: ColorFallback::ThemePanelBackground,
                        })
                        .shadow(ShadowPreset::Md);

                    let mut entries: Vec<CommandEntry> =
                        Vec::with_capacity(items.len() + groups.len());
                    let make_item = |item: ComboboxOption| -> CommandItem {
                        let item_disabled = disabled || item.disabled;
                        let is_selected = selected_values
                            .iter()
                            .any(|v| v.as_ref() == item.value.as_ref());
                        let value_for_select = item.value.clone();
                        let on_select = kit_combobox::commit_multi_selection_on_activate(
                            values.clone(),
                            open.clone(),
                            query_model.clone(),
                            open_change_reason_model.clone(),
                            value_for_select,
                            close_on_commit,
                            clear_query_on_commit,
                        );

                        let mut cmd_item = CommandItem::new(item.label.clone())
                            .value(item.value.clone())
                            .disabled(item_disabled)
                            .checkmark(is_selected)
                            .on_select_action(on_select);
                        if let Some(prefix) = test_id_prefix.as_deref() {
                            cmd_item = cmd_item.test_id(format!(
                                "{prefix}-item-{}",
                                test_id_slug(item.value.as_ref())
                            ));
                        }
                        cmd_item
                    };

                    for item in items.iter().cloned() {
                        entries.push(CommandEntry::Item(make_item(item)));
                    }

                    let non_empty_groups: Vec<ComboboxOptionGroup> = groups
                        .iter()
                        .cloned()
                        .filter(|group| !group.items.is_empty())
                        .collect();
                    let non_empty_groups_len = non_empty_groups.len();

                    if !items.is_empty() && non_empty_groups_len > 0 {
                        entries.push(CommandEntry::Separator(CommandSeparator::new()));
                    }

                    for (idx, group) in non_empty_groups.into_iter().enumerate() {
                        let group_items: Vec<CommandItem> = group
                            .items
                            .into_iter()
                            .map(|item| make_item(item))
                            .collect();
                        entries.push(CommandEntry::Group(
                            CommandGroup::new(group_items).heading(group.heading),
                        ));
                        if idx + 1 < non_empty_groups_len {
                            entries.push(CommandEntry::Separator(CommandSeparator::new()));
                        }
                    }

                    let mut palette = CommandPalette::new(query_model.clone(), [])
                        .entries(entries)
                        .a11y_label("Combobox list")
                        .input_role(SemanticsRole::ComboBox)
                        .input_expanded(true)
                        .input_id_out_cell(search_input_id.clone())
                        .a11y_selected_mode(CommandPaletteA11ySelectedMode::Checked)
                        .list_multiselectable(true)
                        .placeholder(search_placeholder.clone())
                        .disabled(disabled)
                        .empty_text(empty_text.clone())
                        .refine_style(popover_surface.clone())
                        .refine_scroll_layout(LayoutRefinement::default().max_h(max_list_h));

                    if let Some(prefix) = test_id_prefix.as_deref() {
                        palette = palette
                            .input_test_id(format!("{prefix}-input"))
                            .list_test_id(format!("{prefix}-listbox"));
                    }

                    let list = palette.into_element(cx);

                    let desired_w = width.unwrap_or_else(|| Px(anchor.size.width.0.max(180.0)));
                    let content_chrome = ChromeRefinement::default().p(Space::N0);
                    PopoverContent::new(vec![list])
                        .refine_style(content_chrome)
                        .refine_layout(LayoutRefinement::default().w_px(desired_w).min_w_0())
                        .into_element(cx)
                }
            },
        )
    })
}

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
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::combobox as kit_combobox;
use fret_ui_kit::primitives::controllable_state;
use fret_ui_kit::primitives::direction as direction_prim;
use fret_ui_kit::primitives::popover as radix_popover;
use fret_ui_kit::primitives::popper;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, LayoutRefinement, MetricRef, OverrideSlot, Radius,
    ShadowPreset, Size, Space, WidgetState, WidgetStateProperty, WidgetStates,
    resolve_override_slot, ui,
};

use crate::test_id::test_id_slug;
use crate::{
    CommandEntry, CommandGroup, CommandItem, CommandList, CommandPalette, CommandSeparator, Drawer,
    DrawerContent, Popover, PopoverAlign, PopoverAnchor, PopoverContent, PopoverSide,
};

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

pub(crate) fn combobox_group_items<'a>(group: &'a ComboboxGroup) -> &'a [ComboboxItem] {
    if !group.items.is_empty() {
        &group.items
    } else if let Some(collection) = group.collection.as_ref() {
        &collection.items
    } else {
        &[]
    }
}

#[derive(Debug, Clone, Default)]
pub struct ComboboxStyle {
    pub trigger_background: OverrideSlot<ColorRef>,
    pub trigger_foreground: OverrideSlot<ColorRef>,
    pub trigger_border_color: OverrideSlot<ColorRef>,
}

impl ComboboxStyle {
    pub fn trigger_background(mut self, background: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.trigger_background = Some(background);
        self
    }

    pub fn trigger_foreground(mut self, foreground: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.trigger_foreground = Some(foreground);
        self
    }

    pub fn trigger_border_color(mut self, border: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.trigger_border_color = Some(border);
        self
    }

    pub fn merged(mut self, other: Self) -> Self {
        if other.trigger_background.is_some() {
            self.trigger_background = other.trigger_background;
        }
        if other.trigger_foreground.is_some() {
            self.trigger_foreground = other.trigger_foreground;
        }
        if other.trigger_border_color.is_some() {
            self.trigger_border_color = other.trigger_border_color;
        }
        self
    }
}

/// Returns a layout-only anchor wrapper for combobox overlay placement.
///
/// Upstream shadcn/ui v4 returns a DOM ref (`useComboboxAnchor()`) used to anchor the popup. In
/// Fret, we model the same outcome by wrapping a child element and exposing a stable element ID.
pub fn use_combobox_anchor(child: AnyElement) -> PopoverAnchor {
    PopoverAnchor::new(child)
}

#[derive(Default)]
struct ComboboxState {
    query: Option<Model<String>>,
    open_change_reason: Option<Model<Option<ComboboxOpenChangeReason>>>,
    clear_query_on_close: kit_combobox::ClearQueryOnCloseState,
    focus_restore_target: Option<Arc<Mutex<Option<GlobalElementId>>>>,
}

pub use kit_combobox::ComboboxOpenChangeReason;

type OnOpenChange = kit_combobox::OnOpenChange;
type OnOpenChangeWithReason = kit_combobox::OnOpenChangeWithReason;
type OnValueChange = Arc<dyn Fn(Option<Arc<str>>) + Send + Sync + 'static>;

/// Trigger rendering preset for [`Combobox`].
///
/// This is a recipe-level knob used to align the upstream Base UI "Popup" example, where the
/// combobox is triggered from a button-like control.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ComboboxTriggerVariant {
    /// Default Fret shadcn combobox trigger (medium weight).
    #[default]
    Default,
    /// Button-like trigger (normal weight).
    Button,
}

/// shadcn/ui `ComboboxValue` (v4).
///
/// Upstream renders whatever children are passed (a string label in the simple case, chips in
/// `multiple` mode). In Fret, the `Combobox` recipe always renders the selected label itself, so
/// this type is primarily used by the chips adapter (`ComboboxChipsPart::Value`).
#[derive(Debug, Default)]
pub struct ComboboxValue {
    pub(crate) chips: Vec<ComboboxChip>,
}

impl ComboboxValue {
    pub fn new(chips: impl IntoIterator<Item = ComboboxChip>) -> Self {
        Self {
            chips: chips.into_iter().collect(),
        }
    }

    pub fn chip(mut self, chip: ComboboxChip) -> Self {
        self.chips.push(chip);
        self
    }

    pub fn chips(mut self, chips: impl IntoIterator<Item = ComboboxChip>) -> Self {
        self.chips.extend(chips);
        self
    }
}

/// shadcn/ui `ComboboxChip` (v4).
#[derive(Debug)]
pub struct ComboboxChip {
    pub(crate) value: Arc<str>,
    pub(crate) show_remove: bool,
}

impl ComboboxChip {
    pub fn new(value: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            show_remove: true,
        }
    }

    pub fn show_remove(mut self, show_remove: bool) -> Self {
        self.show_remove = show_remove;
        self
    }
}

/// shadcn/ui `ComboboxChipsInput` (v4).
#[derive(Debug, Default)]
pub struct ComboboxChipsInput {
    pub(crate) placeholder: Option<Arc<str>>,
}

impl ComboboxChipsInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }
}

/// shadcn/ui `ComboboxTrigger` (v4).
///
/// Upstream supports a `render` prop to use a button-like trigger. In Fret, this maps to
/// recipe-level knobs (`trigger_variant`, `width`, etc.).
#[derive(Debug, Default)]
pub struct ComboboxTrigger {
    pub(crate) variant: Option<ComboboxTriggerVariant>,
    pub(crate) width: Option<Px>,
}

impl ComboboxTrigger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn variant(mut self, variant: ComboboxTriggerVariant) -> Self {
        self.variant = Some(variant);
        self
    }

    pub fn width_px(mut self, width: Px) -> Self {
        self.width = Some(width);
        self
    }
}

/// shadcn/ui `ComboboxClear` (v4).
///
/// Upstream renders this inside an input-group addon. In Fret this is a configuration part that
/// enables the recipe clear affordance.
#[derive(Debug, Default)]
pub struct ComboboxClear {
    disabled: Option<bool>,
}

impl ComboboxClear {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = Some(disabled);
        self
    }
}

/// Part-based authoring surface aligned with shadcn/ui v4 exports.
///
/// Upstream uses Base UI render props to map `items` → list rows. In Fret we expose a structured
/// adapter that maps part configuration onto the existing Popover + Command recipe so upstream
/// “Usage” shapes remain expressible in Rust.
#[derive(Debug)]
pub enum ComboboxPart {
    Trigger(ComboboxTrigger),
    Input(ComboboxInput),
    Clear(ComboboxClear),
    Content(ComboboxContent),
}

impl ComboboxPart {
    pub fn trigger(trigger: ComboboxTrigger) -> Self {
        Self::Trigger(trigger)
    }

    pub fn input(input: ComboboxInput) -> Self {
        Self::Input(input)
    }

    pub fn clear(clear: ComboboxClear) -> Self {
        Self::Clear(clear)
    }

    pub fn content(content: ComboboxContent) -> Self {
        Self::Content(content)
    }
}

impl From<ComboboxInput> for ComboboxPart {
    fn from(value: ComboboxInput) -> Self {
        Self::Input(value)
    }
}

impl From<ComboboxContent> for ComboboxPart {
    fn from(value: ComboboxContent) -> Self {
        Self::Content(value)
    }
}

impl From<ComboboxTrigger> for ComboboxPart {
    fn from(value: ComboboxTrigger) -> Self {
        Self::Trigger(value)
    }
}

impl From<ComboboxClear> for ComboboxPart {
    fn from(value: ComboboxClear) -> Self {
        Self::Clear(value)
    }
}

/// shadcn/ui `ComboboxInput` (v4).
#[derive(Debug, Default)]
pub struct ComboboxInput {
    pub(crate) placeholder: Option<Arc<str>>,
    pub(crate) disabled: Option<bool>,
    pub(crate) aria_invalid: Option<bool>,
    pub(crate) show_trigger: Option<bool>,
    pub(crate) show_clear: Option<bool>,
}

impl ComboboxInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = Some(disabled);
        self
    }

    /// Apply the upstream `aria-invalid` error state chrome (border + focus ring color).
    pub fn aria_invalid(mut self, aria_invalid: bool) -> Self {
        self.aria_invalid = Some(aria_invalid);
        self
    }

    pub fn show_trigger(mut self, show_trigger: bool) -> Self {
        self.show_trigger = Some(show_trigger);
        self
    }

    pub fn show_clear(mut self, show_clear: bool) -> Self {
        self.show_clear = Some(show_clear);
        self
    }
}

/// shadcn/ui `ComboboxContent` (v4) (Base UI `Popup` + `Positioner`).
#[derive(Debug)]
pub struct ComboboxContent {
    pub(crate) side: Option<popper::Side>,
    pub(crate) align: Option<popper::Align>,
    pub(crate) side_offset: Option<Px>,
    pub(crate) align_offset: Option<Px>,
    pub(crate) anchor_element_id: Option<GlobalElementId>,
    pub(crate) children: Vec<ComboboxContentPart>,
}

impl ComboboxContent {
    pub fn new(children: impl IntoIterator<Item = ComboboxContentPart>) -> Self {
        Self {
            side: None,
            align: None,
            side_offset: None,
            align_offset: None,
            anchor_element_id: None,
            children: children.into_iter().collect(),
        }
    }

    pub fn side(mut self, side: popper::Side) -> Self {
        self.side = Some(side);
        self
    }

    pub fn align(mut self, align: popper::Align) -> Self {
        self.align = Some(align);
        self
    }

    pub fn side_offset_px(mut self, offset: Px) -> Self {
        self.side_offset = Some(offset);
        self
    }

    pub fn align_offset_px(mut self, offset: Px) -> Self {
        self.align_offset = Some(offset);
        self
    }

    /// Overrides which element the popup is anchored to (Base UI `Positioner.anchor`).
    pub fn anchor_element_id(mut self, anchor: GlobalElementId) -> Self {
        self.anchor_element_id = Some(anchor);
        self
    }
}

/// Part-based children inside `ComboboxContent`.
#[derive(Debug)]
pub enum ComboboxContentPart {
    Input(ComboboxInput),
    Empty(ComboboxEmpty),
    List(ComboboxList),
    Separator(ComboboxSeparator),
}

impl ComboboxContentPart {
    pub fn input(input: ComboboxInput) -> Self {
        Self::Input(input)
    }

    pub fn empty(empty: ComboboxEmpty) -> Self {
        Self::Empty(empty)
    }

    pub fn list(list: ComboboxList) -> Self {
        Self::List(list)
    }

    pub fn separator(sep: ComboboxSeparator) -> Self {
        Self::Separator(sep)
    }
}

impl From<ComboboxEmpty> for ComboboxContentPart {
    fn from(value: ComboboxEmpty) -> Self {
        Self::Empty(value)
    }
}

impl From<ComboboxInput> for ComboboxContentPart {
    fn from(value: ComboboxInput) -> Self {
        Self::Input(value)
    }
}

impl From<ComboboxList> for ComboboxContentPart {
    fn from(value: ComboboxList) -> Self {
        Self::List(value)
    }
}

impl From<ComboboxSeparator> for ComboboxContentPart {
    fn from(value: ComboboxSeparator) -> Self {
        Self::Separator(value)
    }
}

/// shadcn/ui `ComboboxEmpty` (v4).
#[derive(Debug)]
pub struct ComboboxEmpty {
    pub(crate) text: Arc<str>,
}

impl ComboboxEmpty {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }
}

/// shadcn/ui `ComboboxSeparator` (v4).
#[derive(Debug, Default)]
pub struct ComboboxSeparator;

impl ComboboxSeparator {
    pub fn new() -> Self {
        Self
    }
}

/// shadcn/ui `ComboboxList` (v4).
///
/// Upstream uses render props (`(item) => <ComboboxItem ... />`). In Rust, callers can prepare an
/// explicit list of items/groups, or leave it empty and continue using `Combobox::options(...)`.
#[derive(Debug, Default)]
pub struct ComboboxList {
    pub(crate) items: Vec<ComboboxItem>,
    pub(crate) groups: Vec<ComboboxGroup>,
    pub(crate) group_separators: bool,
}

impl ComboboxList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn items(mut self, items: impl IntoIterator<Item = ComboboxItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    pub fn groups(mut self, groups: impl IntoIterator<Item = ComboboxGroup>) -> Self {
        self.groups = groups.into_iter().collect();
        self
    }

    pub fn group_separators(mut self, enabled: bool) -> Self {
        self.group_separators = enabled;
        self
    }
}

/// shadcn/ui `ComboboxItem` (v4).
#[derive(Debug)]
pub struct ComboboxItem {
    pub(crate) value: Arc<str>,
    pub(crate) label: Arc<str>,
    pub(crate) detail: Option<Arc<str>>,
    pub(crate) disabled: bool,
    pub(crate) keywords: Vec<Arc<str>>,
    pub(crate) content: Option<AnyElement>,
}

impl ComboboxItem {
    pub fn new(value: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            detail: None,
            disabled: false,
            keywords: Vec::new(),
            content: None,
        }
    }

    /// Overrides the default label rendering for this item.
    ///
    /// The `label` field remains the source of truth for filtering/semantics; this only affects
    /// visuals.
    pub fn content(mut self, content: AnyElement) -> Self {
        self.content = Some(content);
        self
    }

    pub fn detail(mut self, detail: impl Into<Arc<str>>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn keywords<I, S>(mut self, keywords: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Arc<str>>,
    {
        self.keywords = keywords.into_iter().map(Into::into).collect();
        self
    }
}

/// shadcn/ui `ComboboxLabel` (v4).
#[derive(Debug)]
pub struct ComboboxLabel {
    pub(crate) text: Arc<str>,
}

impl ComboboxLabel {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }
}

/// shadcn/ui `ComboboxCollection` (v4).
#[derive(Debug, Default)]
pub struct ComboboxCollection {
    pub(crate) items: Vec<ComboboxItem>,
}

impl ComboboxCollection {
    pub fn new(items: impl IntoIterator<Item = ComboboxItem>) -> Self {
        Self {
            items: items.into_iter().collect(),
        }
    }
}

/// shadcn/ui `ComboboxGroup` (v4).
#[derive(Debug, Default)]
pub struct ComboboxGroup {
    pub(crate) label: Option<ComboboxLabel>,
    pub(crate) collection: Option<ComboboxCollection>,
    pub(crate) items: Vec<ComboboxItem>,
    pub(crate) separator: bool,
}

impl ComboboxGroup {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn label(mut self, label: ComboboxLabel) -> Self {
        self.label = Some(label);
        self
    }

    pub fn collection(mut self, collection: ComboboxCollection) -> Self {
        self.collection = Some(collection);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = ComboboxItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    pub fn separator(mut self, enabled: bool) -> Self {
        self.separator = enabled;
        self
    }
}

#[derive(Debug, Default)]
struct ComboboxPartsPatch {
    trigger_variant: Option<ComboboxTriggerVariant>,
    width: Option<Px>,
    placeholder: Option<Arc<str>>,
    search_placeholder: Option<Arc<str>>,
    disabled: Option<bool>,
    aria_invalid: Option<bool>,
    show_trigger: Option<bool>,
    show_clear: Option<bool>,
    content_side: Option<popper::Side>,
    content_align: Option<popper::Align>,
    content_side_offset: Option<Px>,
    content_align_offset: Option<Px>,
    anchor_element_id: Option<GlobalElementId>,
    empty_text: Option<Arc<str>>,
    list_items: Option<Vec<ComboboxItem>>,
    list_groups: Option<Vec<ComboboxGroup>>,
    group_separators: Option<bool>,
}

fn combobox_parts_patch(parts: Vec<ComboboxPart>) -> ComboboxPartsPatch {
    let mut patch = ComboboxPartsPatch::default();
    let mut content: Option<ComboboxContent> = None;

    for part in parts {
        match part {
            ComboboxPart::Trigger(trigger) => {
                if trigger.variant.is_some() {
                    patch.trigger_variant = trigger.variant;
                }
                if trigger.width.is_some() {
                    patch.width = trigger.width;
                }
            }
            ComboboxPart::Input(input) => {
                if input.placeholder.is_some() {
                    patch.placeholder = input.placeholder;
                }
                if input.disabled.is_some() {
                    patch.disabled = input.disabled;
                }
                if input.aria_invalid.is_some() {
                    patch.aria_invalid = input.aria_invalid;
                }
                if input.show_trigger.is_some() {
                    patch.show_trigger = input.show_trigger;
                }
                if input.show_clear.is_some() {
                    patch.show_clear = input.show_clear;
                }
            }
            ComboboxPart::Clear(clear) => {
                patch.show_clear = Some(true);
                if clear.disabled.is_some() {
                    patch.disabled = clear.disabled;
                }
            }
            ComboboxPart::Content(next) => {
                content = Some(next);
            }
        }
    }

    if let Some(content) = content {
        if content.side.is_some() {
            patch.content_side = content.side;
        }
        if content.align.is_some() {
            patch.content_align = content.align;
        }
        if content.side_offset.is_some() {
            patch.content_side_offset = content.side_offset;
        }
        if content.align_offset.is_some() {
            patch.content_align_offset = content.align_offset;
        }
        if content.anchor_element_id.is_some() {
            patch.anchor_element_id = content.anchor_element_id;
        }

        let mut saw_separator = false;
        for child in content.children {
            match child {
                ComboboxContentPart::Input(input) => {
                    if input.placeholder.is_some() {
                        patch.search_placeholder = input.placeholder;
                    }
                }
                ComboboxContentPart::Empty(empty) => {
                    patch.empty_text = Some(empty.text);
                }
                ComboboxContentPart::Separator(_) => {
                    saw_separator = true;
                }
                ComboboxContentPart::List(list) => {
                    if !list.items.is_empty() {
                        patch.list_items = Some(list.items);
                    }

                    if !list.groups.is_empty() {
                        let group_separators_requested = saw_separator
                            || list.group_separators
                            || list.groups.iter().any(|g| g.separator);

                        patch.list_groups = Some(list.groups);

                        // Upstream uses an explicit `ComboboxSeparator` part inside the group.
                        if group_separators_requested {
                            patch.group_separators = Some(true);
                        }
                    } else if saw_separator || list.group_separators {
                        patch.group_separators = Some(true);
                    }
                }
            }
        }
    }

    patch
}

pub struct Combobox {
    model: Model<Option<Arc<str>>>,
    open: Model<bool>,
    query: Option<Model<String>>,
    items: Vec<ComboboxItem>,
    groups: Vec<ComboboxGroup>,
    group_separators: bool,
    auto_highlight: bool,
    test_id_prefix: Option<Arc<str>>,
    trigger_test_id: Option<Arc<str>>,
    width: Option<Px>,
    content_side: popper::Side,
    content_align: popper::Align,
    content_side_offset: Px,
    content_align_offset: Px,
    anchor_element_id: Option<GlobalElementId>,
    responsive: bool,
    responsive_device_md_breakpoint: Px,
    placeholder: Arc<str>,
    search_placeholder: Arc<str>,
    empty_text: Arc<str>,
    aria_invalid: bool,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    search_enabled: bool,
    show_clear: bool,
    show_trigger: bool,
    trigger_variant: ComboboxTriggerVariant,
    consume_outside_pointer_events: bool,
    selection_commit_policy: kit_combobox::SelectionCommitPolicy,
    close_auto_focus_policy: kit_combobox::ComboboxCloseAutoFocusPolicy,
    on_value_change: Option<OnValueChange>,
    on_open_change: Option<OnOpenChange>,
    on_open_change_with_reason: Option<OnOpenChangeWithReason>,
    on_open_change_complete: Option<OnOpenChange>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    style: ComboboxStyle,
}

impl Combobox {
    pub fn new(model: Model<Option<Arc<str>>>, open: Model<bool>) -> Self {
        Self {
            model,
            open,
            query: None,
            items: Vec::new(),
            groups: Vec::new(),
            group_separators: false,
            auto_highlight: true,
            test_id_prefix: None,
            trigger_test_id: None,
            width: None,
            content_side: popper::Side::Bottom,
            content_align: popper::Align::Start,
            // Upstream shadcn/ui v4 ComboboxContent defaults to `sideOffset=6`.
            content_side_offset: Px(6.0),
            content_align_offset: Px(0.0),
            anchor_element_id: None,
            responsive: false,
            responsive_device_md_breakpoint: fret_ui_kit::declarative::viewport_tailwind::MD,
            placeholder: Arc::from("Select..."),
            search_placeholder: Arc::from("Search..."),
            empty_text: Arc::from("No results."),
            aria_invalid: false,
            disabled: false,
            a11y_label: None,
            search_enabled: true,
            show_clear: false,
            show_trigger: true,
            trigger_variant: ComboboxTriggerVariant::default(),
            // shadcn/ui Combobox is a Popover + Command recipe; Popover is click-through by default.
            // (ADR 0069)
            consume_outside_pointer_events: false,
            selection_commit_policy: kit_combobox::SelectionCommitPolicy::default(),
            close_auto_focus_policy: kit_combobox::ComboboxCloseAutoFocusPolicy::default(),
            on_value_change: None,
            on_open_change: None,
            on_open_change_with_reason: None,
            on_open_change_complete: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: ComboboxStyle::default(),
        }
    }

    /// Render the combobox using shadcn/ui v4 part-based composition.
    ///
    /// This is a compatibility adapter that maps upstream-like `ComboboxInput` /
    /// `ComboboxContent` / `ComboboxEmpty` / `ComboboxList` parts onto Fret's Popover + Command
    /// recipe.
    #[track_caller]
    pub fn into_element_parts<H: UiHost>(
        mut self,
        cx: &mut ElementContext<'_, H>,
        parts: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<ComboboxPart>,
    ) -> AnyElement {
        let patch = combobox_parts_patch(parts(cx));
        if let Some(variant) = patch.trigger_variant {
            self.trigger_variant = variant;
        }
        if let Some(width) = patch.width {
            self.width = Some(width);
        }
        if let Some(placeholder) = patch.placeholder {
            self.placeholder = placeholder;
        }
        if let Some(placeholder) = patch.search_placeholder {
            self.search_placeholder = placeholder;
        }
        if let Some(disabled) = patch.disabled {
            self.disabled = disabled;
        }
        if let Some(aria_invalid) = patch.aria_invalid {
            self.aria_invalid = aria_invalid;
        }
        if let Some(show_trigger) = patch.show_trigger {
            self.show_trigger = show_trigger;
        }
        if let Some(show_clear) = patch.show_clear {
            self.show_clear = show_clear;
        }
        if let Some(side) = patch.content_side {
            self.content_side = side;
        }
        if let Some(align) = patch.content_align {
            self.content_align = align;
        }
        if let Some(offset) = patch.content_side_offset {
            self.content_side_offset = offset;
        }
        if let Some(offset) = patch.content_align_offset {
            self.content_align_offset = offset;
        }
        if let Some(anchor_element_id) = patch.anchor_element_id {
            self.anchor_element_id = Some(anchor_element_id);
        }
        if let Some(empty_text) = patch.empty_text {
            self.empty_text = empty_text;
        }
        if let Some(items) = patch.list_items {
            self.items = items;
        }
        if let Some(groups) = patch.list_groups {
            self.groups = groups;
        }
        if let Some(group_separators) = patch.group_separators {
            self.group_separators = group_separators;
        }

        self.into_element(cx)
    }

    /// Creates a combobox with controlled/uncontrolled models:
    /// - `value` / `default_value` (selected item value)
    /// - `open` / `default_open` (popover visibility)
    ///
    /// This matches the Radix-style controlled vs uncontrolled contract (but note that upstream
    /// shadcn Combobox is a recipe, not a dedicated Radix primitive).
    pub fn new_controllable<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        value: Option<Model<Option<Arc<str>>>>,
        default_value: Option<Arc<str>>,
        open: Option<Model<bool>>,
        default_open: bool,
    ) -> Self {
        let open = radix_popover::PopoverRoot::new()
            .open(open)
            .default_open(default_open)
            .open_model(cx);
        let value = controllable_state::use_controllable_model(cx, value, || default_value).model();
        Self::new(value, open)
    }

    /// When enabled, follows the upstream shadcn "responsive combobox" recipe: it uses a Drawer on
    /// narrow viewports (mobile) and a Popover on desktop.
    pub fn responsive(mut self, responsive: bool) -> Self {
        self.responsive = responsive;
        self
    }

    /// Overrides the device-level viewport breakpoint used by [`Combobox::responsive`].
    ///
    /// This is intentionally **viewport-driven** (mobile vs desktop), not container-query-driven.
    /// For panel-width responsiveness, prefer container queries (ADR 0231).
    pub fn responsive_device_md_breakpoint(mut self, breakpoint: Px) -> Self {
        self.responsive_device_md_breakpoint = breakpoint;
        self
    }

    pub fn query_model(mut self, query: Model<String>) -> Self {
        self.query = Some(query);
        self
    }

    pub fn item(mut self, item: ComboboxItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items<I>(mut self, items: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<ComboboxItem>,
    {
        self.items.extend(items.into_iter().map(Into::into));
        self
    }

    pub fn group(mut self, group: ComboboxGroup) -> Self {
        self.groups.push(group);
        self
    }

    pub fn groups<I>(mut self, groups: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<ComboboxGroup>,
    {
        self.groups.extend(groups.into_iter().map(Into::into));
        self
    }

    /// When enabled, inserts visual separators between `items` and `groups`, and between
    /// consecutive groups (shadcn `ComboboxSeparator`).
    pub fn group_separators(mut self, enabled: bool) -> Self {
        self.group_separators = enabled;
        self
    }

    /// Base UI: `autoHighlight`. When enabled, highlights the first enabled option on open/filter.
    pub fn auto_highlight(mut self, auto_highlight: bool) -> Self {
        self.auto_highlight = auto_highlight;
        self
    }

    pub fn test_id_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        self.test_id_prefix = Some(prefix.into());
        self
    }

    /// Sets a stable `test_id` on the trigger pressable itself (useful for diag scripts).
    pub fn trigger_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.trigger_test_id = Some(id.into());
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn search_enabled(mut self, enabled: bool) -> Self {
        self.search_enabled = enabled;
        self
    }

    pub fn consume_outside_pointer_events(mut self, consume: bool) -> Self {
        self.consume_outside_pointer_events = consume;
        self
    }

    pub fn selection_commit_policy(mut self, policy: kit_combobox::SelectionCommitPolicy) -> Self {
        self.selection_commit_policy = policy;
        self
    }

    pub fn close_auto_focus_policy(
        mut self,
        policy: kit_combobox::ComboboxCloseAutoFocusPolicy,
    ) -> Self {
        self.close_auto_focus_policy = policy;
        self
    }

    /// Called when selected value changes (Base UI `onValueChange`).
    pub fn on_value_change(mut self, on_value_change: Option<OnValueChange>) -> Self {
        self.on_value_change = on_value_change;
        self
    }

    /// Called when the open state changes (Base UI `onOpenChange`).
    pub fn on_open_change(mut self, on_open_change: Option<OnOpenChange>) -> Self {
        self.on_open_change = on_open_change;
        self
    }

    /// Called when the open state changes, with reason metadata.
    pub fn on_open_change_with_reason(
        mut self,
        on_open_change_with_reason: Option<OnOpenChangeWithReason>,
    ) -> Self {
        self.on_open_change_with_reason = on_open_change_with_reason;
        self
    }

    /// Called when open/close transition settles (Base UI `onOpenChangeComplete`).
    pub fn on_open_change_complete(
        mut self,
        on_open_change_complete: Option<OnOpenChange>,
    ) -> Self {
        self.on_open_change_complete = on_open_change_complete;
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn style(mut self, style: ComboboxStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        combobox_with_patch(
            cx,
            self.model,
            self.open,
            self.query,
            self.items,
            self.groups,
            self.test_id_prefix,
            self.trigger_test_id,
            self.width,
            self.content_side,
            self.content_align,
            self.content_side_offset,
            self.content_align_offset,
            self.anchor_element_id,
            self.placeholder,
            self.search_placeholder,
            self.empty_text,
            self.aria_invalid,
            self.disabled,
            self.a11y_label,
            self.responsive,
            self.responsive_device_md_breakpoint,
            self.search_enabled,
            self.group_separators,
            self.auto_highlight,
            self.show_clear,
            self.show_trigger,
            self.trigger_variant,
            self.consume_outside_pointer_events,
            self.selection_commit_policy,
            self.close_auto_focus_policy,
            self.on_value_change,
            self.on_open_change,
            self.on_open_change_with_reason,
            self.on_open_change_complete,
            self.chrome,
            self.layout,
            self.style,
        )
    }
}

#[allow(clippy::too_many_arguments)]
pub fn combobox<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Option<Arc<str>>>,
    open: Model<bool>,
    query: Option<Model<String>>,
    items: impl IntoIterator<Item = ComboboxItem>,
    width: Option<Px>,
    placeholder: Arc<str>,
    search_placeholder: Arc<str>,
    empty_text: Arc<str>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    search_enabled: bool,
    consume_outside_pointer_events: bool,
) -> AnyElement {
    let items: Vec<ComboboxItem> = items.into_iter().collect();
    combobox_with_patch(
        cx,
        model,
        open,
        query,
        items,
        Vec::new(),
        None,
        None,
        width,
        popper::Side::Bottom,
        popper::Align::Center,
        Px(4.0),
        Px(0.0),
        None,
        placeholder,
        search_placeholder,
        empty_text,
        false,
        disabled,
        a11y_label,
        false,
        fret_ui_kit::declarative::viewport_tailwind::MD,
        search_enabled,
        false,
        false,
        false,
        true,
        ComboboxTriggerVariant::default(),
        consume_outside_pointer_events,
        kit_combobox::SelectionCommitPolicy::default(),
        kit_combobox::ComboboxCloseAutoFocusPolicy::default(),
        None,
        None,
        None,
        None,
        ChromeRefinement::default(),
        LayoutRefinement::default(),
        ComboboxStyle::default(),
    )
}

#[allow(clippy::too_many_arguments)]
fn combobox_with_patch<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Option<Arc<str>>>,
    open: Model<bool>,
    query: Option<Model<String>>,
    items: Vec<ComboboxItem>,
    groups: Vec<ComboboxGroup>,
    test_id_prefix: Option<Arc<str>>,
    trigger_test_id: Option<Arc<str>>,
    width: Option<Px>,
    content_side: popper::Side,
    content_align: popper::Align,
    content_side_offset: Px,
    content_align_offset: Px,
    anchor_element_id: Option<GlobalElementId>,
    placeholder: Arc<str>,
    search_placeholder: Arc<str>,
    empty_text: Arc<str>,
    aria_invalid: bool,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    responsive: bool,
    responsive_device_md_breakpoint: Px,
    search_enabled: bool,
    group_separators: bool,
    auto_highlight: bool,
    show_clear: bool,
    show_trigger: bool,
    trigger_variant: ComboboxTriggerVariant,
    consume_outside_pointer_events: bool,
    selection_commit_policy: kit_combobox::SelectionCommitPolicy,
    close_auto_focus_policy: kit_combobox::ComboboxCloseAutoFocusPolicy,
    on_value_change: Option<OnValueChange>,
    on_open_change: Option<OnOpenChange>,
    on_open_change_with_reason: Option<OnOpenChangeWithReason>,
    on_open_change_complete: Option<OnOpenChange>,
    chrome_patch: ChromeRefinement,
    layout_patch: LayoutRefinement,
    style_override: ComboboxStyle,
) -> AnyElement {
    cx.scope(|cx| {
        let theme = Theme::global(&*cx.app).snapshot();
        let open_change_reason_model = {
            let existing =
                cx.with_state(ComboboxState::default, |st| st.open_change_reason.clone());
            if let Some(model) = existing {
                model
            } else {
                let model = cx.app.models_mut().insert(None::<ComboboxOpenChangeReason>);
                cx.with_state(ComboboxState::default, |st| {
                    st.open_change_reason = Some(model.clone())
                });
                model
            }
        };
        let focus_restore_target = {
            let existing =
                cx.with_state(ComboboxState::default, |st| st.focus_restore_target.clone());
            if let Some(cell) = existing {
                cell
            } else {
                let cell: Arc<Mutex<Option<GlobalElementId>>> = Arc::new(Mutex::new(None));
                cx.with_state(ComboboxState::default, |st| {
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
        let selected = cx.watch_model(&model).cloned().unwrap_or_default();
        if let Some(handler) = on_value_change.as_ref() {
            let value_change = cx.with_state(
                kit_combobox::ValueChangeCallbackState::<Arc<str>>::default,
                |state| kit_combobox::value_change_event(state, selected.clone()),
            );
            if let Some(value) = value_change {
                handler(value);
            }
        }
        let is_open = cx.watch_model(&open).layout().copied().unwrap_or(false);
        let open_change_reason = cx
            .watch_model(&open_change_reason_model)
            .layout()
            .copied()
            .unwrap_or(None)
            .unwrap_or(ComboboxOpenChangeReason::None);
        let (open_change, open_change_complete) = cx
            .with_state(kit_combobox::OpenChangeCallbackState::default, |state| {
                kit_combobox::open_change_events(state, is_open, is_open, false)
            });
        if let (Some(open), Some(handler)) = (open_change, on_open_change.as_ref()) {
            handler(open);
        }
        if let (Some(open), Some(handler)) = (open_change, on_open_change_with_reason.as_ref()) {
            handler(open, open_change_reason);
        }
        if let (Some(open), Some(handler)) =
            (open_change_complete, on_open_change_complete.as_ref())
        {
            handler(open);
        }

        let query_model = if let Some(q) = query {
            cx.with_state(ComboboxState::default, |st| st.query = Some(q.clone()));
            q
        } else {
            let existing = cx.with_state(ComboboxState::default, |st| st.query.clone());
            if let Some(m) = existing {
                m
            } else {
                let m = cx.app.models_mut().insert(String::new());
                cx.with_state(ComboboxState::default, |st| st.query = Some(m.clone()));
                m
            }
        };

        let should_clear_query = cx.with_state(ComboboxState::default, |st| {
            kit_combobox::should_clear_query_on_close(&mut st.clear_query_on_close, is_open)
        });
        if should_clear_query {
            let _ = cx.app.models_mut().update(&query_model, |v| v.clear());
        }

        let size = Size::default();
        let radius = chrome_patch
            .radius
            .as_ref()
            .map(|m| m.resolve(&theme))
            .unwrap_or_else(|| size.control_radius(Theme::global(&*cx.app)));
        let mut ring = decl_style::focus_ring(&theme, radius);

        let (resolved_label, has_selection) = selected
            .as_ref()
            .and_then(|v| {
                items
                    .iter()
                    .chain(
                        groups
                            .iter()
                            .flat_map(|g| combobox_group_items(g).iter()),
                    )
                    .find(|it| it.value.as_ref() == v.as_ref())
            })
            .map(|it| (it.label.clone(), true))
            .unwrap_or((placeholder.clone(), false));

        let text_style = TextStyle {
            font: FontId::default(),
            size: size.control_text_px(Theme::global(&*cx.app)),
            weight: match trigger_variant {
                ComboboxTriggerVariant::Default => FontWeight::MEDIUM,
                ComboboxTriggerVariant::Button => FontWeight::NORMAL,
            },
            line_height: theme
                .metric_by_key("font.line_height")
                .or(Some(theme.metric_token("font.line_height"))),
            ..Default::default()
        };

        let min_h = chrome_patch
            .min_height
            .as_ref()
            .map(|m| m.resolve(&theme))
            .unwrap_or_else(|| size.button_h(Theme::global(&*cx.app)));
        let pad_x = size.button_px(Theme::global(&*cx.app));
        let pad_y = size.button_py(Theme::global(&*cx.app));
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

        let (bg_base, bg_hover, bg_pressed) = match trigger_variant {
            ComboboxTriggerVariant::Default => {
                // Upstream shadcn combobox chips root uses:
                // - light: `bg-transparent`
                // - dark: `dark:bg-input/30`
                let base = chrome_patch
                    .background
                    .as_ref()
                    .map(|c| c.resolve(&theme))
                    .unwrap_or_else(|| {
                        theme
                            .color_by_key("component.input.bg")
                            .unwrap_or_else(|| {
                                theme
                                    .color_by_key("background")
                                    .unwrap_or_else(|| theme.color_token("background"))
                            })
                    });
                (base, base, base)
            }
            ComboboxTriggerVariant::Button => {
                let base = chrome_patch
                    .background
                    .as_ref()
                    .map(|c| c.resolve(&theme))
                    .unwrap_or_else(|| {
                        theme
                            .color_by_key("background")
                            .unwrap_or_else(|| theme.color_token("background"))
                    });
                let hover = theme
                    .color_by_key("accent")
                    .or_else(|| theme.color_by_key("accent.background"))
                    .unwrap_or_else(|| theme.color_token("accent"));
                let pressed = theme.color_token("accent");
                (base, hover, pressed)
            }
        };
        let fg_base = chrome_patch
            .text_color
            .as_ref()
            .map(|c| c.resolve(&theme))
            .unwrap_or_else(|| {
                theme
                    .color_by_key("foreground")
                    .unwrap_or_else(|| theme.color_token("foreground"))
            });
        let fg_hover = match trigger_variant {
            ComboboxTriggerVariant::Default => fg_base,
            ComboboxTriggerVariant::Button => theme
                .color_by_key("accent-foreground")
                .or_else(|| theme.color_by_key("accent.foreground"))
                .unwrap_or(fg_base),
        };
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
        let mut border_base = border_base;
        let mut ring_border = theme.color_token("ring");

        if aria_invalid {
            let border_color = theme.color_token("destructive");
            border_base = border_color;
            ring_border = border_color;

            ring.color = crate::theme_variants::invalid_control_ring_color(&theme, border_color);
        }

        let default_trigger_bg = WidgetStateProperty::new(ColorRef::Color(bg_base))
            .when(WidgetStates::HOVERED, ColorRef::Color(bg_hover))
            .when(WidgetStates::ACTIVE, ColorRef::Color(bg_pressed));
        let default_trigger_fg = WidgetStateProperty::new(ColorRef::Color(fg_base))
            .when(WidgetStates::HOVERED, ColorRef::Color(fg_hover))
            .when(WidgetStates::ACTIVE, ColorRef::Color(fg_hover));
        let default_trigger_border = WidgetStateProperty::new(ColorRef::Color(border_base))
            .when(WidgetStates::FOCUS_VISIBLE, ColorRef::Color(ring_border));

        let enabled = !disabled;
        let open_for_trigger = open.clone();
        let trigger_gap = MetricRef::space(Space::N2).resolve(&theme);
        let a11y_label_for_trigger = a11y_label.clone();
        let trigger_test_id_for_trigger = trigger_test_id.clone();
        let label_is_placeholder = !has_selection;
        let placeholder_fg_for_trigger = muted_fg;

        let padding = chrome_patch.padding.clone().unwrap_or_default();
        let pad_top = padding.top.map(|m| m.resolve(&theme)).unwrap_or(pad_y);
        let pad_right = padding.right.map(|m| m.resolve(&theme)).unwrap_or(pad_x);
        let pad_bottom = padding.bottom.map(|m| m.resolve(&theme)).unwrap_or(pad_y);
        let pad_left = padding.left.map(|m| m.resolve(&theme)).unwrap_or(pad_x);

        let theme_for_trigger = theme.clone();

        // Device-level responsiveness: shadcn's "responsive combobox" uses Drawer on mobile.
        // This is a viewport breakpoint by design (not a container query).
        let is_desktop = fret_ui_kit::declarative::viewport_width_at_least(
            cx,
            Invalidation::Layout,
            responsive_device_md_breakpoint,
            fret_ui_kit::declarative::ViewportQueryHysteresis::default(),
        );
        if responsive && !is_desktop {
            let open_change_reason_model_for_trigger = open_change_reason_model.clone();
            let open_change_reason_model_for_content = open_change_reason_model.clone();
            let test_id_prefix_for_content = test_id_prefix.clone();
            let test_id_prefix_for_trigger = test_id_prefix.clone();
            let trigger_test_id_for_trigger = trigger_test_id_for_trigger.clone();
            let focus_restore_target_for_trigger = focus_restore_target.clone();
            let model_for_trigger = model.clone();
            let query_model_for_trigger = query_model.clone();
            let selected_for_trigger = selected.clone();
            let show_trigger_for_trigger = show_trigger;
            let items_for_content = items;
            let groups_for_content = groups;
            return Drawer::new(open.clone())
                .on_dismiss_request(Some(
                    kit_combobox::set_open_change_reason_on_dismiss_request(
                        open_change_reason_model.clone(),
                    ),
                ))
                .on_close_auto_focus(Some(close_auto_focus.clone()))
                .into_element(
                    cx,
                    move |cx| {
                        let open_change_reason_model = open_change_reason_model_for_trigger.clone();
                        let focus_restore_target = focus_restore_target_for_trigger.clone();
                        let test_id_prefix_for_trigger = test_id_prefix_for_trigger.clone();
                        let model = model_for_trigger.clone();
                        let query_model = query_model_for_trigger.clone();
                        let selected = selected_for_trigger.clone();
                        let show_trigger = show_trigger_for_trigger;
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

                            let bg = bg_ref.resolve(&theme_for_trigger);
                            let fg = fg_ref.resolve(&theme_for_trigger);
                            let border = border_ref.resolve(&theme_for_trigger);
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
                                        .or_else(|| Some(resolved_label.clone())),
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
                                }.into(),
                                background: Some(bg),
                                shadow: None,
                                border: Edges::all(border_w),
                                border_color: Some(border),
                                corner_radii: Corners::all(radius),
                                ..Default::default()
                            };

                            (props, chrome_props, move |cx| {
                                vec![cx.flex(
                                    FlexProps {
                                        layout: {
                                            let mut layout = LayoutStyle::default();
                                            layout.size.width = Length::Fill;
                                            layout
                                        },
                                        direction: fret_core::Axis::Horizontal,
                                        gap: trigger_gap.into(),
                                        padding: Edges::all(Px(0.0)).into(),
                                        justify: MainAlign::SpaceBetween,
                                        align: CrossAlign::Center,
                                        wrap: false,
                                    },
                                    move |cx| {
                                        let label_style = text_style.clone();
                                        let show_clear = show_clear && selected.is_some();
                                        let label_el = {
                                            let mut label = ui::label(cx, resolved_label.clone())
                                                .w_full()
                                                .min_w_0()
                                                .flex_1()
                                                .basis_0()
                                                .text_size_px(label_style.size)
                                                .font_weight(label_style.weight)
                                                .text_color(if label_is_placeholder {
                                                    ColorRef::Color(placeholder_fg_for_trigger)
                                                } else {
                                                    fg_ref.clone()
                                                })
                                                .truncate();
                                            if let Some(line_height) = label_style.line_height {
                                                label = label
                                                    .line_height_px(line_height)
                                                    .line_height_policy(
                                                        fret_core::TextLineHeightPolicy::FixedFromStyle,
                                                    );
                                            }
                                            if let Some(letter_spacing_em) =
                                                label_style.letter_spacing_em
                                            {
                                                label = label.letter_spacing_em(letter_spacing_em);
                                            }
                                            label.into_element(cx)
                                        };

                                        let right = (show_clear || show_trigger).then(|| {
                                            cx.flex(
                                                FlexProps {
                                                    layout: LayoutStyle::default(),
                                                    direction: fret_core::Axis::Horizontal,
                                                    gap: Px(0.0).into(),
                                                    padding: Edges::all(Px(0.0)).into(),
                                                    justify: MainAlign::Start,
                                                    align: CrossAlign::Center,
                                                    wrap: false,
                                                },
                                                move |cx| {
                                                    let mut out = Vec::new();
                                                    if show_clear {
                                                    let model_for_clear = model.clone();
                                                    let query_for_clear = query_model.clone();
                                                    let theme_for_clear =
                                                        theme_for_trigger.clone();
                                                    let hovered_bg = bg_hover;
                                                    let pressed_bg = bg_pressed;
                                                    let clear_radius =
                                                        Px((radius.0 - 5.0).max(0.0));
                                                    let clear_size =
                                                        Px((min_h.0 - 8.0).max(0.0));

                                                    let clear =
                                                        control_chrome_pressable_with_id_props(
                                                            cx,
                                                            move |cx, st, _id| {
                                                                cx.pressable_add_on_activate(
                                                                    Arc::new(
                                                                        move |host,
                                                                              _acx,
                                                                              _reason| {
                                                                            let _ = host
                                                                                .models_mut()
                                                                                .update(
                                                                                    &model_for_clear,
                                                                                    |v| {
                                                                                        *v = None;
                                                                                    },
                                                                                );
                                                                            let _ = host
                                                                                .models_mut()
                                                                                .update(
                                                                                    &query_for_clear,
                                                                                    |v| {
                                                                                        v.clear();
                                                                                    },
                                                                                );
                                                                        },
                                                                    ),
                                                                );

                                                                let hovered =
                                                                    st.hovered && enabled;
                                                                let pressed =
                                                                    st.pressed && enabled;
                                                                let bg = if pressed {
                                                                    pressed_bg
                                                                } else if hovered {
                                                                    hovered_bg
                                                                } else {
                                                                    Color::TRANSPARENT
                                                                };

                                                                let pressable_layout =
                                                                    decl_style::layout_style(
                                                                        &theme_for_clear,
                                                                        LayoutRefinement::default()
                                                                            .w_px(clear_size)
                                                                            .h_px(clear_size)
                                                                            .min_w(clear_size)
                                                                            .min_h(clear_size),
                                                                    );
                                                                let pressable_props = PressableProps {
                                                                    layout: pressable_layout,
                                                                    enabled,
                                                                    focusable: true,
                                                                    focus_ring: None,
                                                                    a11y: PressableA11y {
                                                                        role: Some(
                                                                            SemanticsRole::Button,
                                                                        ),
                                                                        label: Some(Arc::from(
                                                                            "Clear",
                                                                        )),
                                                                        ..Default::default()
                                                                    },
                                                                    ..Default::default()
                                                                };

                                                                let chrome_props = ContainerProps {
                                                                    layout: LayoutStyle::default(),
                                                                    background: Some(bg),
                                                                    corner_radii: Corners::all(
                                                                        clear_radius,
                                                                    ),
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
                                                                                Some(Px(16.0)),
                                                                                Some(ColorRef::Color(
                                                                                    icon_fg,
                                                                                )),
                                                                            );
                                                                        vec![cx.flex(
                                                                            FlexProps {
                                                                                layout: LayoutStyle::default(),
                                                                                direction: fret_core::Axis::Horizontal,
                                                                                gap: Px(0.0).into(),
                                                                                padding: Edges::all(Px(0.0)).into(),
                                                                                justify: MainAlign::Center,
                                                                                align: CrossAlign::Center,
                                                                                wrap: false,
                                                                            },
                                                                            move |_cx| vec![icon],
                                                                        )]
                                                                    };

                                                                (
                                                                    pressable_props,
                                                                    chrome_props,
                                                                    children,
                                                                )
                                                            },
                                                        );

                                                    let clear = if let Some(prefix) =
                                                        test_id_prefix_for_trigger.as_deref()
                                                    {
                                                        clear.test_id(format!(
                                                            "{prefix}-clear-button"
                                                        ))
                                                    } else {
                                                        clear
                                                    };
                                                    out.push(clear);
                                                } else if show_trigger {
                                                    let mut icon = decl_icon::icon_with(
                                                        cx,
                                                        ids::ui::CHEVRON_DOWN,
                                                        Some(Px(16.0)),
                                                        Some(ColorRef::Color(icon_fg)),
                                                    );
                                                    if let Some(prefix) =
                                                        test_id_prefix_for_trigger.as_deref()
                                                    {
                                                        icon = icon.test_id(format!(
                                                            "{prefix}-trigger-icon"
                                                        ));
                                                    }
                                                    out.push(icon);
                                                }
                                                out
                                            },
                                        )
                                        });

                                        let mut out = vec![label_el];
                                        if let Some(right) = right {
                                            out.push(right);
                                        }
                                        out
                                    },
                                )]
                            })
                        })
                    },
                    move |cx| {
                        let test_id_prefix = test_id_prefix_for_content.clone();
                        let open_change_reason_model = open_change_reason_model_for_content.clone();
                        let items = items_for_content;
                        let groups = groups_for_content;
                        let theme_max_list_h = theme
                            .metric_by_key("component.combobox.max_list_height")
                            .or_else(|| theme.metric_by_key("component.select.max_list_height"))
                            .unwrap_or(Px(280.0));
                        let selected = cx.watch_model(&model).cloned().unwrap_or_default();

                        let list = if search_enabled {
                            let max_list_h = Px(theme_max_list_h.0.max(0.0));
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
                            let mut make_item = |item: ComboboxItem| -> CommandItem {
                                let item_disabled = disabled || item.disabled;
                                let is_selected = selected
                                    .as_ref()
                                    .is_some_and(|v| v.as_ref() == item.value.as_ref());

                                let model_for_select = model.clone();
                                let open_for_select = open.clone();
                                let query_for_select = query_model.clone();
                                let open_change_reason_model_for_select =
                                    open_change_reason_model.clone();

                                let label_text = if item.content.is_none() {
                                    if let Some(detail) = item.detail.as_ref() {
                                        Arc::<str>::from(format!(
                                            "{} ({})",
                                            item.label.as_ref(),
                                            detail.as_ref()
                                        ))
                                    } else {
                                        item.label.clone()
                                    }
                                } else {
                                    item.label.clone()
                                };

                                let mut keywords = item.keywords.clone();
                                if let Some(detail) = item.detail.clone() {
                                    keywords.push(detail);
                                }

                                let mut cmd_item = CommandItem::new(label_text)
                                    .value(item.value.clone())
                                    .keywords(keywords)
                                    .disabled(item_disabled)
                                    .checkmark(is_selected)
                                    .on_select_value_action(move |host, action_cx, reason, value| {
                                        let on_select = kit_combobox::commit_selection_on_activate(
                                            selection_commit_policy,
                                            model_for_select.clone(),
                                            open_for_select.clone(),
                                            query_for_select.clone(),
                                            open_change_reason_model_for_select.clone(),
                                            value,
                                        );
                                        on_select(host, action_cx, reason);
                                    });

                                if let Some(content) = item.content {
                                    let body = ui::h_flex(cx, move |_cx| vec![content])
                                        .w_full()
                                        .min_w_0()
                                        .flex_1()
                                        .basis_0()
                                        .into_element(cx);
                                    cmd_item = cmd_item.children([body]);
                                }

                                if let Some(prefix) = test_id_prefix.as_deref() {
                                    cmd_item = cmd_item.test_id(format!(
                                        "{prefix}-item-{}",
                                        test_id_slug(item.value.as_ref())
                                    ));
                                }

                                cmd_item
                            };

                            let mut root_items = items;
                            let mut non_empty_groups: Vec<(Arc<str>, Vec<ComboboxItem>)> =
                                Vec::new();
                            for group in groups {
                                let group_items = if !group.items.is_empty() {
                                    group.items
                                } else {
                                    group.collection.map(|c| c.items).unwrap_or_default()
                                };
                                if group_items.is_empty() {
                                    continue;
                                }
                                if let Some(label) = group.label {
                                    non_empty_groups.push((label.text, group_items));
                                } else {
                                    root_items.extend(group_items);
                                }
                            }

                            for item in root_items {
                                entries.push(CommandEntry::Item(make_item(item)));
                            }

                            let non_empty_groups_len = non_empty_groups.len();
                            if group_separators && !entries.is_empty() && non_empty_groups_len > 0 {
                                let sep = if let Some(prefix) = test_id_prefix.as_deref() {
                                    CommandSeparator::new()
                                        .test_id(format!("{prefix}-sep-items-groups"))
                                } else {
                                    CommandSeparator::new()
                                };
                                entries.push(CommandEntry::Separator(sep));
                            }

                            for (idx, (heading, group_items)) in
                                non_empty_groups.into_iter().enumerate()
                            {
                                let group_items: Vec<CommandItem> = group_items
                                    .into_iter()
                                    .map(|item| make_item(item))
                                    .collect();
                                entries.push(CommandEntry::Group(
                                    CommandGroup::new(group_items).heading(heading),
                                ));
                                if group_separators && idx + 1 < non_empty_groups_len {
                                    let sep = if let Some(prefix) = test_id_prefix.as_deref() {
                                        CommandSeparator::new()
                                            .test_id(format!("{prefix}-sep-group-{idx}"))
                                    } else {
                                        CommandSeparator::new()
                                    };
                                    entries.push(CommandEntry::Separator(sep));
                                }
                            }

                            {
                                let mut palette =
                                    CommandPalette::new(query_model.clone(), [])
                                        .entries(entries)
                                        .a11y_label("Combobox list")
                                        .input_role(SemanticsRole::ComboBox)
                                        .input_expanded(true)
                                        .a11y_selected_mode(
                                            crate::command::CommandPaletteA11ySelectedMode::Checked,
                                        )
                                        .auto_highlight(auto_highlight)
                                        .placeholder(search_placeholder.clone())
                                        .disabled(disabled)
                                        .empty_text(empty_text.clone())
                                        .refine_style(popover_surface.clone())
                                        .refine_scroll_layout(
                                            LayoutRefinement::default().max_h(max_list_h),
                                        );

                                if let Some(prefix) = test_id_prefix.as_deref() {
                                    palette = palette
                                        .input_test_id(format!("{prefix}-input"))
                                        .list_test_id(format!("{prefix}-listbox"));
                                }

                                palette.into_element(cx)
                            }
                        } else {
                            let max_list_h = Px(theme_max_list_h.0.max(0.0));

                            let fg = theme
                                .color_by_key("foreground")
                                .unwrap_or_else(|| theme.color_token("foreground"));
                            let fg_disabled = alpha_mul(fg, 0.5);
                            let item_text_style = crate::command::item_text_style(&theme);

                            let mut entries: Vec<CommandEntry> =
                                Vec::with_capacity(items.len() + groups.len());
                            let mut make_item = |item: ComboboxItem| -> CommandItem {
                                    let item_disabled = disabled || item.disabled;
                                    let is_selected = selected
                                        .as_ref()
                                        .is_some_and(|v| v.as_ref() == item.value.as_ref());

                                    let model_for_select = model.clone();
                                    let open_for_select = open.clone();
                                    let query_for_select = query_model.clone();
                                    let open_change_reason_model_for_select =
                                        open_change_reason_model.clone();

                                    let label_text = if item.content.is_none() {
                                        if let Some(detail) = item.detail.as_ref() {
                                            Arc::<str>::from(format!(
                                                "{} ({})",
                                                item.label.as_ref(),
                                                detail.as_ref()
                                            ))
                                        } else {
                                            item.label.clone()
                                        }
                                    } else {
                                        item.label.clone()
                                    };

                                    let mut keywords = item.keywords.clone();
                                    if let Some(detail) = item.detail.clone() {
                                        keywords.push(detail);
                                    }

                                    let label_style = item_text_style.clone();
                                    let icon = decl_icon::icon_with(
                                        cx,
                                        ids::ui::CHECK,
                                        Some(Px(16.0)),
                                        Some(ColorRef::Color(if item_disabled {
                                            fg_disabled
                                        } else {
                                            fg
                                        })),
                                    );
                                    let icon = cx.opacity(
                                        if is_selected { 1.0 } else { 0.0 },
                                        move |_cx| vec![icon],
                                    );

                                    let body = if let Some(content) = item.content {
                                        ui::h_flex(cx, move |_cx| vec![content])
                                            .w_full()
                                            .min_w_0()
                                            .flex_1()
                                            .basis_0()
                                            .into_element(cx)
                                    } else {
                                        let mut label = ui::label(cx, label_text.clone())
                                            .text_size_px(label_style.size)
                                            .font_weight(label_style.weight)
                                            .text_color(ColorRef::Color(if item_disabled {
                                                fg_disabled
                                            } else {
                                                fg
                                            }))
                                            .truncate();
                                        if let Some(line_height) = label_style.line_height {
                                            label = label
                                                .line_height_px(line_height)
                                                .line_height_policy(
                                                    fret_core::TextLineHeightPolicy::FixedFromStyle,
                                                );
                                        }
                                        if let Some(letter_spacing_em) =
                                            label_style.letter_spacing_em
                                        {
                                            label = label.letter_spacing_em(letter_spacing_em);
                                        }
                                        label.into_element(cx)
                                    };

                                    let mut cmd_item = CommandItem::new(label_text)
                                        .value(item.value.clone())
                                        .keywords(keywords)
                                        .disabled(item_disabled)
                                        .on_select_value_action(move |host, action_cx, reason, value| {
                                            let on_select = kit_combobox::commit_selection_on_activate(
                                                selection_commit_policy,
                                                model_for_select.clone(),
                                                open_for_select.clone(),
                                                query_for_select.clone(),
                                                open_change_reason_model_for_select.clone(),
                                                value,
                                            );
                                            on_select(host, action_cx, reason);
                                        })
                                        .children(vec![body, icon]);

                                    if let Some(prefix) = test_id_prefix.as_deref() {
                                        cmd_item = cmd_item.test_id(format!(
                                            "{prefix}-item-{}",
                                            test_id_slug(item.value.as_ref())
                                        ));
                                    }

                                    cmd_item
                                };

                            let mut root_items = items;
                            let mut non_empty_groups: Vec<(Arc<str>, Vec<ComboboxItem>)> =
                                Vec::new();
                            for group in groups {
                                let group_items = if !group.items.is_empty() {
                                    group.items
                                } else {
                                    group.collection.map(|c| c.items).unwrap_or_default()
                                };
                                if group_items.is_empty() {
                                    continue;
                                }
                                if let Some(label) = group.label {
                                    non_empty_groups.push((label.text, group_items));
                                } else {
                                    root_items.extend(group_items);
                                }
                            }

                            for item in root_items {
                                entries.push(CommandEntry::Item(make_item(item)));
                            }

                            let non_empty_groups_len = non_empty_groups.len();
                            if group_separators && !entries.is_empty() && non_empty_groups_len > 0 {
                                let sep = if let Some(prefix) = test_id_prefix.as_deref() {
                                    CommandSeparator::new()
                                        .test_id(format!("{prefix}-sep-items-groups"))
                                } else {
                                    CommandSeparator::new()
                                };
                                entries.push(CommandEntry::Separator(sep));
                            }

                            for (idx, (heading, group_items)) in
                                non_empty_groups.into_iter().enumerate()
                            {
                                let group_items: Vec<CommandItem> = group_items
                                    .into_iter()
                                    .map(|item| make_item(item))
                                    .collect();
                                entries.push(CommandEntry::Group(
                                    CommandGroup::new(group_items).heading(heading),
                                ));
                                if group_separators && idx + 1 < non_empty_groups_len {
                                    let sep = if let Some(prefix) = test_id_prefix.as_deref() {
                                        CommandSeparator::new()
                                            .test_id(format!("{prefix}-sep-group-{idx}"))
                                    } else {
                                        CommandSeparator::new()
                                    };
                                    entries.push(CommandEntry::Separator(sep));
                                }
                            }

                            CommandList::new_entries(entries)
                                .disabled(disabled)
                                .empty_text(empty_text.clone())
                                .refine_scroll_layout(LayoutRefinement::default().max_h(max_list_h))
                                .into_element(cx)
                        };

                        DrawerContent::new(vec![list])
                            .refine_style(ChromeRefinement::default().p(Space::N0))
                            .refine_layout(LayoutRefinement::default().w_full().min_w_0())
                            .into_element(cx)
                    },
                );
        }

        let open_change_reason_model_for_trigger = open_change_reason_model.clone();
        let open_change_reason_model_for_content = open_change_reason_model.clone();
        let test_id_prefix_for_content = test_id_prefix.clone();
        let test_id_prefix_for_trigger = test_id_prefix.clone();
        let focus_restore_target_for_trigger = focus_restore_target.clone();
        let model_for_trigger = model.clone();
        let query_model_for_trigger = query_model.clone();
        let selected_for_trigger = selected.clone();

        let search_input_id = search_enabled.then(|| Rc::new(Cell::new(None)));
        let search_input_id_for_content = search_input_id.clone();
        let listbox_id_for_diag = Rc::new(Cell::new(None));
        let listbox_id_for_diag_for_content = listbox_id_for_diag.clone();

        let mut popover = Popover::new(open.clone())
            .auto_focus(true)
            .consume_outside_pointer_events(consume_outside_pointer_events)
            .side(match content_side {
                popper::Side::Top => PopoverSide::Top,
                popper::Side::Right => PopoverSide::Right,
                popper::Side::Bottom => PopoverSide::Bottom,
                popper::Side::Left => PopoverSide::Left,
            })
            .align(match content_align {
                popper::Align::Start => PopoverAlign::Start,
                popper::Align::Center => PopoverAlign::Center,
                popper::Align::End => PopoverAlign::End,
            })
            .side_offset(content_side_offset)
            .align_offset(content_align_offset)
            .diagnostics_content_element_from_cell(listbox_id_for_diag)
            .on_dismiss_request(Some(
                kit_combobox::set_open_change_reason_on_dismiss_request(
                    open_change_reason_model.clone(),
                ),
            ))
            .on_close_auto_focus(Some(close_auto_focus.clone()));

        if let Some(cell) = search_input_id.clone() {
            popover = popover.initial_focus_from_cell(cell);
        }
        if let Some(anchor_element_id) = anchor_element_id {
            popover = popover.anchor_element(anchor_element_id);
        }

        popover.into_element_with_anchor(
            cx,
            move |cx| {
                let open_change_reason_model = open_change_reason_model_for_trigger.clone();
                let focus_restore_target = focus_restore_target_for_trigger.clone();
                let test_id_prefix_for_trigger = test_id_prefix_for_trigger.clone();
                let model = model_for_trigger.clone();
                let query_model = query_model_for_trigger.clone();
                let selected = selected_for_trigger.clone();
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

                    let bg = bg_ref.resolve(&theme_for_trigger);
                    let fg = fg_ref.resolve(&theme_for_trigger);
                    let border = border_ref.resolve(&theme_for_trigger);
                    let icon_fg = alpha_mul(fg, 0.5);

                    cx.pressable_add_on_activate(kit_combobox::set_open_change_reason_on_activate(
                        open_change_reason_model.clone(),
                        ComboboxOpenChangeReason::TriggerPress,
                    ));
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
                                .or_else(|| Some(resolved_label.clone())),
                            test_id: trigger_test_id_for_trigger.clone(),
                            expanded: Some(is_open),
                            ..Default::default()
                        },
                        ..Default::default()
                    };

                    let chrome_props = ContainerProps {
                        layout: LayoutStyle::default(),
                        padding: Edges {
                            top: pad_top,
                            right: pad_right,
                            bottom: pad_bottom,
                            left: pad_left,
                        }.into(),
                        background: Some(bg),
                        shadow: None,
                        border: Edges::all(border_w),
                        border_color: Some(border),
                        corner_radii: Corners::all(radius),
                        ..Default::default()
                    };

                    (props, chrome_props, move |cx| {
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
                            move |cx| {
                                let label_style = text_style.clone();
                                let show_clear = show_clear && selected.is_some();
                                let label_el = {
                                    let mut label = ui::label(cx, resolved_label.clone())
                                        .w_full()
                                        .min_w_0()
                                        .flex_1()
                                        .basis_0()
                                        .text_size_px(label_style.size)
                                        .font_weight(label_style.weight)
                                        .text_color(if label_is_placeholder {
                                            ColorRef::Color(placeholder_fg_for_trigger)
                                        } else {
                                            fg_ref.clone()
                                        })
                                        .truncate();
                                    if let Some(line_height) = label_style.line_height {
                                        label = label
                                            .line_height_px(line_height)
                                            .line_height_policy(
                                                fret_core::TextLineHeightPolicy::FixedFromStyle,
                                            );
                                    }
                                    if let Some(letter_spacing_em) = label_style.letter_spacing_em
                                    {
                                        label = label.letter_spacing_em(letter_spacing_em);
                                    }
                                    label.into_element(cx)
                                };

                                let right = (show_clear || show_trigger).then(|| {
                                    cx.flex(
                                        FlexProps {
                                            layout: LayoutStyle::default(),
                                            direction: fret_core::Axis::Horizontal,
                                            gap: Px(0.0).into(),
                                            padding: Edges::all(Px(0.0)).into(),
                                            justify: MainAlign::Start,
                                            align: CrossAlign::Center,
                                            wrap: false,
                                        },
                                        move |cx| {
                                            let mut out = Vec::new();
                                            if show_clear {
                                            let model_for_clear = model.clone();
                                            let query_for_clear = query_model.clone();
                                            let theme_for_clear = theme_for_trigger.clone();
                                            let hovered_bg = bg_hover;
                                            let pressed_bg = bg_pressed;
                                            let clear_radius = Px((radius.0 - 5.0).max(0.0));
                                            let clear_size = Px((min_h.0 - 8.0).max(0.0));

                                            let clear = control_chrome_pressable_with_id_props(
                                                cx,
                                                move |cx, st, _id| {
                                                    cx.pressable_add_on_activate(Arc::new(
                                                        move |host, _acx, _reason| {
                                                            let _ = host.models_mut().update(
                                                                &model_for_clear,
                                                                |v| {
                                                                    *v = None;
                                                                },
                                                            );
                                                            let _ = host.models_mut().update(
                                                                &query_for_clear,
                                                                |v| {
                                                                    v.clear();
                                                                },
                                                            );
                                                        },
                                                    ));

                                                    let hovered = st.hovered && enabled;
                                                    let pressed = st.pressed && enabled;
                                                    let bg = if pressed {
                                                        pressed_bg
                                                    } else if hovered {
                                                        hovered_bg
                                                    } else {
                                                        Color::TRANSPARENT
                                                    };

                                                    let pressable_layout = decl_style::layout_style(
                                                        &theme_for_clear,
                                                        LayoutRefinement::default()
                                                            .w_px(clear_size)
                                                            .h_px(clear_size)
                                                            .min_w(clear_size)
                                                            .min_h(clear_size),
                                                    );
                                                    let pressable_props = PressableProps {
                                                        layout: pressable_layout,
                                                        enabled,
                                                        focusable: true,
                                                        focus_ring: None,
                                                        a11y: PressableA11y {
                                                            role: Some(SemanticsRole::Button),
                                                            label: Some(Arc::from("Clear")),
                                                            ..Default::default()
                                                        },
                                                        ..Default::default()
                                                    };

                                                    let chrome_props = ContainerProps {
                                                        layout: LayoutStyle::default(),
                                                        background: Some(bg),
                                                        corner_radii: Corners::all(clear_radius),
                                                        ..Default::default()
                                                    };

                                                    let children =
                                                        move |cx: &mut ElementContext<'_, H>| {
                                                            let icon = decl_icon::icon_with(
                                                                cx,
                                                                ids::ui::CLOSE,
                                                                Some(Px(16.0)),
                                                                Some(ColorRef::Color(icon_fg)),
                                                            );
                                                            vec![cx.flex(
                                                                FlexProps {
                                                                    layout: LayoutStyle::default(),
                                                                    direction: fret_core::Axis::Horizontal,
                                                                    gap: Px(0.0).into(),
                                                                    padding: Edges::all(Px(0.0)).into(),
                                                                    justify: MainAlign::Center,
                                                                    align: CrossAlign::Center,
                                                                    wrap: false,
                                                                },
                                                                move |_cx| vec![icon],
                                                            )]
                                                        };

                                                    (pressable_props, chrome_props, children)
                                                },
                                            );

                                            let clear = if let Some(prefix) =
                                                test_id_prefix_for_trigger.as_deref()
                                            {
                                                clear.test_id(format!("{prefix}-clear-button"))
                                            } else {
                                                clear
                                            };
                                            out.push(clear);
                                        } else if show_trigger {
                                            let mut icon = decl_icon::icon_with(
                                                cx,
                                                ids::ui::CHEVRON_DOWN,
                                                Some(Px(16.0)),
                                                Some(ColorRef::Color(icon_fg)),
                                            );
                                            if let Some(prefix) =
                                                test_id_prefix_for_trigger.as_deref()
                                            {
                                                icon = icon
                                                    .test_id(format!("{prefix}-trigger-icon"));
                                            }
                                            out.push(icon);
                                        }
                                        out
                                    },
                                )
                                });

                                let mut out = vec![label_el];
                                if let Some(right) = right {
                                    out.push(right);
                                }
                                out
                            },
                        )]
                    })
                })
            },
                move |cx, anchor| {
                let test_id_prefix = test_id_prefix_for_content.clone();
                let open_change_reason_model = open_change_reason_model_for_content.clone();
                let search_input_id = search_input_id_for_content.clone();
                let listbox_id_for_diag = listbox_id_for_diag_for_content.clone();
	                let theme_max_list_h = theme
	                    .metric_by_key("component.combobox.max_list_height")
	                    .or_else(|| theme.metric_by_key("component.select.max_list_height"))
	                    .unwrap_or(Px(280.0));
	                let desired_w = width.unwrap_or_else(|| Px(anchor.size.width.0.max(180.0)));
	                let selected = cx.watch_model(&model).cloned().unwrap_or_default();
	                let mut items = Some(items);
	                let mut groups = Some(groups);

	                    let list = if search_enabled {
                    // Clamp the list height to the best-available main-axis space around the
                    // trigger. This models the Radix popper "available height" variables used by
                    // shadcn/cmdk (`--radix-*-content-available-height`) and prevents the listbox
                    // from overflowing tight windows.
                    let window_margin = theme
                        .metric_by_key("component.popover.window_margin")
                        .unwrap_or(Px(0.0));
                    let outer = fret_ui_kit::overlay::outer_bounds_with_window_margin_for_environment(
                        cx,
                        fret_ui::Invalidation::Layout,
                        window_margin,
                    );
                    let direction = direction_prim::use_direction_in_scope(cx, None);
                    let placement = combobox_content_placement(
                        direction,
                        content_side,
                        content_align,
                        content_side_offset,
                        content_align_offset,
                    );
                    let available_main = radix_popover::popover_popper_vars(
                        outer,
                        anchor,
                        desired_w,
                        placement,
                    )
                    .available_height;
                    // CommandPalette includes a fixed-height search row above the list.
                    let header_estimate = Px(48.0);
                    let max_list_h = Px(
                        theme_max_list_h
                            .0
                            .max(0.0)
                            .min((available_main.0 - header_estimate.0).max(0.0)),
                    );
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

                    let items = items.take().unwrap_or_default();
                    let groups = groups.take().unwrap_or_default();
                    let mut entries: Vec<CommandEntry> =
                        Vec::with_capacity(items.len() + groups.len());

                    let mut make_item = |item: ComboboxItem| -> CommandItem {
                        let item_disabled = disabled || item.disabled;
                        let is_selected = selected
                            .as_ref()
                            .is_some_and(|v| v.as_ref() == item.value.as_ref());

                        let model_for_select = model.clone();
                        let open_for_select = open.clone();
                        let query_for_select = query_model.clone();
                        let open_change_reason_model_for_select = open_change_reason_model.clone();

                        let label_text = if item.content.is_none() {
                            if let Some(detail) = item.detail.as_ref() {
                                Arc::<str>::from(format!(
                                    "{} ({})",
                                    item.label.as_ref(),
                                    detail.as_ref()
                                ))
                            } else {
                                item.label.clone()
                            }
                        } else {
                            item.label.clone()
                        };

                        let mut keywords = item.keywords.clone();
                        if let Some(detail) = item.detail.clone() {
                            keywords.push(detail);
                        }

                        let mut cmd_item = CommandItem::new(label_text)
                            .value(item.value.clone())
                            .keywords(keywords)
                            .disabled(item_disabled)
                            .checkmark(is_selected)
                            .on_select_value_action(move |host, action_cx, reason, value| {
                                let on_select = kit_combobox::commit_selection_on_activate(
                                    selection_commit_policy,
                                    model_for_select.clone(),
                                    open_for_select.clone(),
                                    query_for_select.clone(),
                                    open_change_reason_model_for_select.clone(),
                                    value,
                                );
                                on_select(host, action_cx, reason);
                            });

                        if let Some(content) = item.content {
                            let body = ui::h_flex(cx, move |_cx| vec![content])
                                .w_full()
                                .min_w_0()
                                .flex_1()
                                .basis_0()
                                .into_element(cx);
                            cmd_item = cmd_item.children([body]);
                        }

                        if let Some(prefix) = test_id_prefix.as_deref() {
                            cmd_item = cmd_item.test_id(format!(
                                "{prefix}-item-{}",
                                test_id_slug(item.value.as_ref())
                            ));
                        }

                        cmd_item
                    };

                    let mut root_items = items;
                    let mut non_empty_groups: Vec<(Arc<str>, Vec<ComboboxItem>)> = Vec::new();
                    for group in groups {
                        let group_items = if !group.items.is_empty() {
                            group.items
                        } else {
                            group.collection.map(|c| c.items).unwrap_or_default()
                        };
                        if group_items.is_empty() {
                            continue;
                        }
                        if let Some(label) = group.label {
                            non_empty_groups.push((label.text, group_items));
                        } else {
                            root_items.extend(group_items);
                        }
                    }

                    for item in root_items {
                        entries.push(CommandEntry::Item(make_item(item)));
                    }

                    let non_empty_groups_len = non_empty_groups.len();
                    if group_separators && !entries.is_empty() && non_empty_groups_len > 0 {
                        let sep = if let Some(prefix) = test_id_prefix.as_deref() {
                            CommandSeparator::new().test_id(format!("{prefix}-sep-items-groups"))
                        } else {
                            CommandSeparator::new()
                        };
                        entries.push(CommandEntry::Separator(sep));
                    }

                    for (idx, (heading, group_items)) in non_empty_groups.into_iter().enumerate()
                    {
                        let group_items: Vec<CommandItem> =
                            group_items.into_iter().map(|item| make_item(item)).collect();
                        entries.push(CommandEntry::Group(
                            CommandGroup::new(group_items).heading(heading),
                        ));
                        if group_separators && idx + 1 < non_empty_groups_len {
                            let sep = if let Some(prefix) = test_id_prefix.as_deref() {
                                CommandSeparator::new().test_id(format!("{prefix}-sep-group-{idx}"))
                            } else {
                                CommandSeparator::new()
                            };
                            entries.push(CommandEntry::Separator(sep));
                        }
                    }

                    {
                        let mut palette = CommandPalette::new(query_model.clone(), [])
                            .entries(entries)
                            .a11y_label("Combobox list")
                            .input_role(SemanticsRole::ComboBox)
                            .input_expanded(true)
                            .input_id_out_cell(search_input_id.clone().expect(
                                "combobox search-enabled popover should provide input-id cell",
                            ))
                            .list_id_out_cell(listbox_id_for_diag.clone())
                            .a11y_selected_mode(
                                crate::command::CommandPaletteA11ySelectedMode::Checked,
                            )
                            .auto_highlight(auto_highlight)
                            .placeholder(search_placeholder.clone())
                            .disabled(disabled)
                            .empty_text(empty_text)
                            .refine_style(popover_surface.clone())
                            .refine_scroll_layout(LayoutRefinement::default().max_h(max_list_h));

                        if let Some(prefix) = test_id_prefix.as_deref() {
                            palette = palette
                                .input_test_id(format!("{prefix}-input"))
                                .list_test_id(format!("{prefix}-listbox"));
                        }

                        palette.into_element(cx)
                    }
                } else {
                    let max_list_h = Px(theme_max_list_h.0.max(0.0));

                    let fg = theme
                        .color_by_key("foreground")
                        .unwrap_or_else(|| theme.color_token("foreground"));
                    let fg_disabled = alpha_mul(fg, 0.5);
                    let item_text_style = crate::command::item_text_style(&theme);

                    let items = items.take().unwrap_or_default();
                    let groups = groups.take().unwrap_or_default();
                    let mut entries: Vec<CommandEntry> =
                        Vec::with_capacity(items.len() + groups.len());

                    let mut make_item = |item: ComboboxItem| -> CommandItem {
                        let item_disabled = disabled || item.disabled;
                        let is_selected = selected
                            .as_ref()
                            .is_some_and(|v| v.as_ref() == item.value.as_ref());

                        let model_for_select = model.clone();
                        let open_for_select = open.clone();
                        let query_for_select = query_model.clone();
                        let open_change_reason_model_for_select = open_change_reason_model.clone();

                        let label_text = if item.content.is_none() {
                            if let Some(detail) = item.detail.as_ref() {
                                Arc::<str>::from(format!(
                                    "{} ({})",
                                    item.label.as_ref(),
                                    detail.as_ref()
                                ))
                            } else {
                                item.label.clone()
                            }
                        } else {
                            item.label.clone()
                        };

                        let mut keywords = item.keywords.clone();
                        if let Some(detail) = item.detail.clone() {
                            keywords.push(detail);
                        }

                        let label_style = item_text_style.clone();
                        let icon = decl_icon::icon_with(
                            cx,
                            ids::ui::CHECK,
                            Some(Px(16.0)),
                            Some(ColorRef::Color(if item_disabled { fg_disabled } else { fg })),
                        );
                        let icon =
                            cx.opacity(if is_selected { 1.0 } else { 0.0 }, move |_cx| vec![icon]);

                        let body = if let Some(content) = item.content {
                            ui::h_flex(cx, move |_cx| vec![content])
                                .w_full()
                                .min_w_0()
                                .flex_1()
                                .basis_0()
                                .into_element(cx)
                        } else {
                            let mut label = ui::label(cx, label_text.clone())
                                .text_size_px(label_style.size)
                                .font_weight(label_style.weight)
                                .text_color(ColorRef::Color(if item_disabled {
                                    fg_disabled
                                } else {
                                    fg
                                }))
                                .truncate();
                            if let Some(line_height) = label_style.line_height {
                                label = label
                                    .line_height_px(line_height)
                                    .line_height_policy(
                                        fret_core::TextLineHeightPolicy::FixedFromStyle,
                                    );
                            }
                            if let Some(letter_spacing_em) = label_style.letter_spacing_em {
                                label = label.letter_spacing_em(letter_spacing_em);
                            }
                            label.into_element(cx)
                        };

                        let mut cmd_item = CommandItem::new(label_text)
                            .value(item.value.clone())
                            .keywords(keywords)
                            .disabled(item_disabled)
                            .on_select_value_action(move |host, action_cx, reason, value| {
                                let on_select = kit_combobox::commit_selection_on_activate(
                                    selection_commit_policy,
                                    model_for_select.clone(),
                                    open_for_select.clone(),
                                    query_for_select.clone(),
                                    open_change_reason_model_for_select.clone(),
                                    value,
                                );
                                on_select(host, action_cx, reason);
                            })
                            .children(vec![body, icon]);

                        if let Some(prefix) = test_id_prefix.as_deref() {
                            cmd_item = cmd_item.test_id(format!(
                                "{prefix}-item-{}",
                                test_id_slug(item.value.as_ref())
                            ));
                        }

                        cmd_item
                    };

                    let mut root_items = items;
                    let mut non_empty_groups: Vec<(Arc<str>, Vec<ComboboxItem>)> = Vec::new();
                    for group in groups {
                        let group_items = if !group.items.is_empty() {
                            group.items
                        } else {
                            group.collection.map(|c| c.items).unwrap_or_default()
                        };
                        if group_items.is_empty() {
                            continue;
                        }
                        if let Some(label) = group.label {
                            non_empty_groups.push((label.text, group_items));
                        } else {
                            root_items.extend(group_items);
                        }
                    }

                    for item in root_items {
                        entries.push(CommandEntry::Item(make_item(item)));
                    }

                    let non_empty_groups_len = non_empty_groups.len();
                    if group_separators && !entries.is_empty() && non_empty_groups_len > 0 {
                        let sep = if let Some(prefix) = test_id_prefix.as_deref() {
                            CommandSeparator::new().test_id(format!("{prefix}-sep-items-groups"))
                        } else {
                            CommandSeparator::new()
                        };
                        entries.push(CommandEntry::Separator(sep));
                    }

                    for (idx, (heading, group_items)) in non_empty_groups.into_iter().enumerate()
                    {
                        let group_items: Vec<CommandItem> =
                            group_items.into_iter().map(|item| make_item(item)).collect();
                        entries.push(CommandEntry::Group(
                            CommandGroup::new(group_items).heading(heading),
                        ));
                        if group_separators && idx + 1 < non_empty_groups_len {
                            let sep = if let Some(prefix) = test_id_prefix.as_deref() {
                                CommandSeparator::new().test_id(format!("{prefix}-sep-group-{idx}"))
                            } else {
                                CommandSeparator::new()
                            };
                            entries.push(CommandEntry::Separator(sep));
                        }
                    }

                    CommandList::new_entries(entries)
                        .disabled(disabled)
                        .empty_text(empty_text)
                        .refine_scroll_layout(LayoutRefinement::default().max_h(max_list_h))
                        .into_element(cx)
                };

                let content_chrome = if search_enabled {
                    ChromeRefinement::default()
                        .p(Space::N0)
                        .border_width(Px(0.0))
                        .bg(ColorRef::Color(Color::TRANSPARENT))
                        .border_color(ColorRef::Color(Color::TRANSPARENT))
                        .shadow(ShadowPreset::None)
                } else {
                    ChromeRefinement::default().p(Space::N0)
                };

                PopoverContent::new(vec![list])
                    .refine_style(content_chrome)
                    .refine_layout(LayoutRefinement::default().w_px(desired_w).min_w_0())
                    .into_element(cx)
            },
        )
    })
}

fn combobox_content_placement(
    direction: direction_prim::LayoutDirection,
    side: popper::Side,
    align: popper::Align,
    side_offset: Px,
    align_offset: Px,
) -> popper::PopperContentPlacement {
    popper::PopperContentPlacement::new(direction, side, align, side_offset)
        .with_align_offset(align_offset)
        .with_shift_cross_axis(true)
        .with_sticky(popper::StickyMode::Partial)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::{Cell, RefCell};

    use fret_app::App;
    use fret_core::{
        AppWindowId, Point, Px, Rect, SemanticsRole, Size, SvgId, SvgService, TextBlobId,
        TextConstraints, TextMetrics, TextService, UiServices, WindowFrameClockService,
    };
    use fret_core::{PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_runtime::{FrameId, TickId};
    use fret_ui::tree::UiTree;
    use fret_ui_kit::primitives::popover as radix_popover;

    #[test]
    fn combobox_content_placement_tracks_offsets() {
        let placement = combobox_content_placement(
            direction_prim::LayoutDirection::Ltr,
            popper::Side::Top,
            popper::Align::Start,
            Px(6.0),
            Px(12.0),
        );

        assert_eq!(placement.side, popper::Side::Top);
        assert_eq!(placement.align, popper::Align::Start);
        assert_eq!(placement.side_offset, Px(6.0));
        assert_eq!(placement.align_offset, Px(12.0));
        assert!(placement.shift_cross_axis);
        assert_eq!(placement.sticky, popper::StickyMode::Partial);
    }

    #[test]
    fn combobox_content_side_align_and_offsets_affect_popover_placement_and_slide_motion() {
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(400.0)),
        );

        fn items() -> Vec<ComboboxItem> {
            vec![
                ComboboxItem::new("alpha", "Alpha"),
                ComboboxItem::new("beta", "Beta"),
                ComboboxItem::new("gamma", "Gamma"),
            ]
        }

        fn render_frame_with_content_config(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn UiServices,
            window: AppWindowId,
            bounds: Rect,
            model: Model<Option<Arc<str>>>,
            open: Model<bool>,
            side: popper::Side,
            align: popper::Align,
            side_offset: Px,
            align_offset: Px,
        ) -> (Rect, Option<Rect>, Vec<fret_runtime::Effect>) {
            let next = app.frame_id().0.saturating_add(1);
            app.set_frame_id(FrameId(next));
            app.set_tick_id(TickId(next as u64));
            app.with_global_mut(WindowFrameClockService::default, |svc, app| {
                svc.record_frame(window, app.frame_id());
            });

            fret_ui_kit::OverlayController::begin_frame(app, window);
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "combobox-content-config",
                move |cx| {
                    let fill = {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.height = Length::Fill;
                        layout
                    };

                    vec![cx.flex(
                        FlexProps {
                            layout: fill,
                            direction: fret_core::Axis::Vertical,
                            gap: Px(0.0).into(),
                            padding: Edges::all(Px(0.0)).into(),
                            justify: MainAlign::Start,
                            align: CrossAlign::Start,
                            wrap: false,
                        },
                        move |cx| {
                            let spacer = cx.container(
                                ContainerProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Fill;
                                        layout.size.height = Length::Px(Px(200.0));
                                        layout
                                    },
                                    ..Default::default()
                                },
                                |_cx| Vec::new(),
                            );

                            let combobox = Combobox::new(model, open)
                                .test_id_prefix("cb")
                                .trigger_test_id("cb-trigger")
                                .into_element_parts(cx, |_cx| {
                                    vec![
                                        ComboboxPart::trigger(
                                            ComboboxTrigger::new().width_px(Px(240.0)),
                                        ),
                                        ComboboxPart::input(ComboboxInput::new()),
                                        ComboboxPart::content(
                                            ComboboxContent::new([ComboboxContentPart::list(
                                                ComboboxList::new().items(items()),
                                            )])
                                            .side(side)
                                            .align(align)
                                            .side_offset_px(side_offset)
                                            .align_offset_px(align_offset),
                                        ),
                                    ]
                                });

                            vec![spacer, combobox]
                        },
                    )]
                },
            );
            ui.set_root(root);
            fret_ui_kit::OverlayController::render(ui, app, services, window, bounds);
            ui.request_semantics_snapshot();
            ui.layout_all(app, services, bounds, 1.0);
            let effects = app.flush_effects();

            let snap = ui.semantics_snapshot().expect("semantics snapshot");
            let trigger = snap
                .nodes
                .iter()
                .find(|n| n.test_id.as_deref() == Some("cb-trigger"))
                .or_else(|| {
                    snap.nodes
                        .iter()
                        .find(|n| n.role == SemanticsRole::ComboBox && n.value.is_none())
                })
                .expect("trigger semantics");
            let list = snap.nodes.iter().find(|n| n.role == SemanticsRole::ListBox);

            let trigger_bounds = ui
                .debug_node_visual_bounds(trigger.id)
                .expect("trigger bounds");
            let list_bounds = list.and_then(|n| ui.debug_node_visual_bounds(n.id));
            (trigger_bounds, list_bounds, effects)
        }

        fn settle_and_capture_bounds(
            window: AppWindowId,
            bounds: Rect,
            side: popper::Side,
            align: popper::Align,
            side_offset: Px,
            align_offset: Px,
        ) -> (Rect, Rect, Rect, Vec<fret_runtime::Effect>) {
            let mut app = App::new();
            let mut ui: UiTree<App> = UiTree::new();
            ui.set_window(window);
            let mut services = FakeServices::default();

            app.with_global_mut(WindowFrameClockService::default, |svc, _app| {
                svc.set_fixed_delta(window, Some(std::time::Duration::from_millis(16)));
            });

            let model = app.models_mut().insert(None::<Arc<str>>);
            let open = app.models_mut().insert(false);

            // Frame 1: closed (establish stable trigger bounds).
            let _ = render_frame_with_content_config(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                model.clone(),
                open.clone(),
                side,
                align,
                side_offset,
                align_offset,
            );
            let _ = app.models_mut().update(&open, |v| *v = true);

            // Frame 2+: first open frames. Some recipes may not stamp listbox semantics on the
            // first open frame; render until it appears.
            let mut trigger_bounds = Rect::default();
            let mut list_first = None;
            let mut effects_first_open: Vec<fret_runtime::Effect> = Vec::new();
            for _ in 0..4 {
                let (trigger, list, effects) = render_frame_with_content_config(
                    &mut ui,
                    &mut app,
                    &mut services,
                    window,
                    bounds,
                    model.clone(),
                    open.clone(),
                    side,
                    align,
                    side_offset,
                    align_offset,
                );
                trigger_bounds = trigger;
                if effects_first_open.is_empty() {
                    effects_first_open = effects;
                }
                if let Some(list) = list {
                    list_first = Some(list);
                    break;
                }
            }
            let list_first = list_first.expect("listbox semantics");

            // Frames 3..: render enough ticks for a ~100ms transition to settle.
            let mut list_last = list_first;
            for _ in 0..12 {
                let (_trigger, list_now, _effects) = render_frame_with_content_config(
                    &mut ui,
                    &mut app,
                    &mut services,
                    window,
                    bounds,
                    model.clone(),
                    open.clone(),
                    side,
                    align,
                    side_offset,
                    align_offset,
                );
                if let Some(list_now) = list_now {
                    list_last = list_now;
                }
            }

            (trigger_bounds, list_first, list_last, effects_first_open)
        }

        // Side drives placement + slide direction.
        {
            let (trigger, first, settled, open_effects) = settle_and_capture_bounds(
                window,
                bounds,
                popper::Side::Bottom,
                popper::Align::Start,
                Px(6.0),
                Px(0.0),
            );
            assert!(
                first.origin.y.0 >= trigger.origin.y.0 + trigger.size.height.0 - 1.0,
                "expected bottom-side listbox to be below trigger; trigger={trigger:?} list={first:?}"
            );
            assert!(
                open_effects.iter().any(
                    |e| matches!(e, fret_runtime::Effect::RequestAnimationFrame(w) if *w == window)
                ),
                "expected opening presence to request animation frames; effects={open_effects:?}"
            );
            let _ = settled;
        }

        {
            let (trigger, first, _settled, _effects) = settle_and_capture_bounds(
                window,
                bounds,
                popper::Side::Top,
                popper::Align::Start,
                Px(6.0),
                Px(0.0),
            );
            assert!(
                first.origin.y.0 + first.size.height.0 <= trigger.origin.y.0 + 1.0,
                "expected top-side listbox to be above trigger; trigger={trigger:?} list={first:?}"
            );
        }

        // Align drives horizontal placement (Start vs Center should differ materially).
        {
            let (_trigger, _first, settled_0, _effects) = settle_and_capture_bounds(
                window,
                bounds,
                popper::Side::Bottom,
                popper::Align::Start,
                Px(6.0),
                Px(0.0),
            );
            let (_trigger, _first, settled_40, _effects) = settle_and_capture_bounds(
                window,
                bounds,
                popper::Side::Bottom,
                popper::Align::Start,
                Px(6.0),
                Px(40.0),
            );
            let dx = (settled_40.origin.x.0 - settled_0.origin.x.0).abs();
            assert!(
                dx >= 20.0,
                "expected align_offset to affect listbox x; off0={settled_0:?} off40={settled_40:?} dx={dx}"
            );
        }

        // Side offset should affect vertical placement.
        {
            let (_trigger, _first, settled_0, _effects) = settle_and_capture_bounds(
                window,
                bounds,
                popper::Side::Bottom,
                popper::Align::Start,
                Px(0.0),
                Px(0.0),
            );
            let (_trigger, _first, settled_24, _effects) = settle_and_capture_bounds(
                window,
                bounds,
                popper::Side::Bottom,
                popper::Align::Start,
                Px(24.0),
                Px(0.0),
            );
            let dy = (settled_24.origin.y.0 - settled_0.origin.y.0).abs();
            assert!(
                dy >= 12.0,
                "expected side_offset to affect listbox y; off0={settled_0:?} off24={settled_24:?} dy={dy}"
            );
        }
    }

    #[test]
    fn combobox_parts_patch_maps_input_content_and_list() {
        let parts = vec![
            ComboboxPart::trigger(
                ComboboxTrigger::new()
                    .variant(ComboboxTriggerVariant::Button)
                    .width_px(Px(256.0)),
            ),
            ComboboxPart::input(
                ComboboxInput::new()
                    .placeholder("Pick one")
                    .disabled(true)
                    .show_trigger(false)
                    .show_clear(true),
            ),
            ComboboxPart::content(
                ComboboxContent::new([
                    ComboboxContentPart::input(
                        ComboboxInput::new().placeholder("Search frameworks..."),
                    ),
                    ComboboxContentPart::empty(ComboboxEmpty::new("Nothing found.")),
                    ComboboxContentPart::list(
                        ComboboxList::new()
                            .items([ComboboxItem::new("a", "Alpha").keywords(["alpha"])])
                            .groups([ComboboxGroup::new()
                                .label(ComboboxLabel::new("Group 1"))
                                .items([ComboboxItem::new("b", "Beta").detail("React")])
                                .separator(true)]),
                    ),
                ])
                .side(popper::Side::Top)
                .align(popper::Align::Start)
                .side_offset_px(Px(6.0))
                .align_offset_px(Px(7.0))
                .anchor_element_id(GlobalElementId(42)),
            ),
        ];

        let patch = combobox_parts_patch(parts);
        assert_eq!(patch.trigger_variant, Some(ComboboxTriggerVariant::Button));
        assert_eq!(patch.width, Some(Px(256.0)));
        assert_eq!(patch.placeholder.as_deref(), Some("Pick one"));
        assert_eq!(
            patch.search_placeholder.as_deref(),
            Some("Search frameworks...")
        );
        assert_eq!(patch.disabled, Some(true));
        assert_eq!(patch.show_trigger, Some(false));
        assert_eq!(patch.show_clear, Some(true));
        assert_eq!(patch.content_side, Some(popper::Side::Top));
        assert_eq!(patch.content_align, Some(popper::Align::Start));
        assert_eq!(patch.content_side_offset, Some(Px(6.0)));
        assert_eq!(patch.content_align_offset, Some(Px(7.0)));
        assert_eq!(patch.anchor_element_id, Some(GlobalElementId(42)));
        assert_eq!(patch.empty_text.as_deref(), Some("Nothing found."));
        assert_eq!(patch.group_separators, Some(true));

        let items = patch.list_items.expect("expected items from list");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].value.as_ref(), "a");
        assert_eq!(items[0].label.as_ref(), "Alpha");
        assert_eq!(items[0].disabled, false);
        assert_eq!(items[0].keywords.len(), 1);
        assert_eq!(items[0].keywords[0].as_ref(), "alpha");

        let groups = patch.list_groups.expect("expected groups from list");
        assert_eq!(groups.len(), 1);
        assert_eq!(
            groups[0].label.as_ref().expect("group label").text.as_ref(),
            "Group 1"
        );
        let group_items = combobox_group_items(&groups[0]);
        assert_eq!(group_items.len(), 1);
        assert_eq!(group_items[0].value.as_ref(), "b");
        assert_eq!(group_items[0].detail.as_deref(), Some("React"));
        assert_eq!(group_items[0].label.as_ref(), "Beta");
        assert_eq!(group_items[0].keywords.len(), 0);
    }

    #[test]
    fn combobox_show_clear_renders_only_when_selected() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(360.0), Px(200.0)),
        );
        let mut services = FakeServices::default();

        let model = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);

        let mut render_frame =
            |ui: &mut UiTree<App>, app: &mut App, model: Model<Option<Arc<str>>>| {
                let next_frame = FrameId(app.frame_id().0.saturating_add(1));
                app.set_frame_id(next_frame);

                fret_ui_kit::OverlayController::begin_frame(app, window);
                let root = fret_ui::declarative::render_root(
                    ui,
                    app,
                    &mut services,
                    window,
                    bounds,
                    "combobox-clear",
                    |cx| {
                        vec![
                            Combobox::new(model, open.clone())
                                .a11y_label("Combobox")
                                .test_id_prefix("combobox-clear")
                                .items([ComboboxItem::new("alpha", "Alpha")])
                                .into_element_parts(cx, |_cx| {
                                    vec![ComboboxPart::from(ComboboxInput::new().show_clear(true))]
                                }),
                        ]
                    },
                );
                ui.set_root(root);
                fret_ui_kit::OverlayController::render(ui, app, &mut services, window, bounds);
                ui.request_semantics_snapshot();
                ui.layout_all(app, &mut services, bounds, 1.0);
            };

        // Frame 1: no selection, clear should not render.
        render_frame(&mut ui, &mut app, model.clone());
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            !snap.nodes.iter().any(|n| {
                n.test_id
                    .as_deref()
                    .is_some_and(|id| id == "combobox-clear-clear-button")
            }),
            "expected clear button to be hidden when no value is selected"
        );

        // Frame 2: selection present, clear should render.
        let _ = app
            .models_mut()
            .update(&model, |v| *v = Some(Arc::from("alpha")));
        render_frame(&mut ui, &mut app, model);
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            snap.nodes.iter().any(|n| {
                n.test_id
                    .as_deref()
                    .is_some_and(|id| id == "combobox-clear-clear-button")
            }),
            "expected clear button to be visible when a value is selected"
        );
    }

    #[test]
    fn combobox_show_trigger_hides_chevron_icon() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(360.0), Px(200.0)),
        );
        let mut services = FakeServices::default();

        let model = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);

        let mut render_frame = |ui: &mut UiTree<App>, app: &mut App, show_trigger: bool| {
            let next_frame = FrameId(app.frame_id().0.saturating_add(1));
            app.set_frame_id(next_frame);

            fret_ui_kit::OverlayController::begin_frame(app, window);
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                &mut services,
                window,
                bounds,
                "combobox-show-trigger",
                |cx| {
                    vec![
                        Combobox::new(model.clone(), open.clone())
                            .a11y_label("Combobox")
                            .test_id_prefix("combobox-show-trigger")
                            .items([ComboboxItem::new("alpha", "Alpha")])
                            .into_element_parts(cx, |_cx| {
                                vec![ComboboxPart::from(
                                    ComboboxInput::new().show_trigger(show_trigger),
                                )]
                            }),
                    ]
                },
            );
            ui.set_root(root);
            fret_ui_kit::OverlayController::render(ui, app, &mut services, window, bounds);
            ui.request_semantics_snapshot();
            ui.layout_all(app, &mut services, bounds, 1.0);
        };

        // Frame 1: icon hidden.
        render_frame(&mut ui, &mut app, false);
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            !snap.nodes.iter().any(|n| {
                n.test_id
                    .as_deref()
                    .is_some_and(|id| id == "combobox-show-trigger-trigger-icon")
            }),
            "expected trigger icon to be hidden when show_trigger=false"
        );

        // Frame 2: icon visible.
        render_frame(&mut ui, &mut app, true);
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            snap.nodes.iter().any(|n| {
                n.test_id
                    .as_deref()
                    .is_some_and(|id| id == "combobox-show-trigger-trigger-icon")
            }),
            "expected trigger icon to be visible when show_trigger=true"
        );
    }

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for FakeServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    impl fret_core::MaterialService for FakeServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Ok(fret_core::MaterialId::default())
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            true
        }
    }

    #[test]
    fn combobox_item_detail_sets_detail_without_mutating_label() {
        let item = ComboboxItem::new("next", "Next.js").detail("React");
        assert_eq!(item.detail.as_deref(), Some("React"));
        assert_eq!(item.label.as_ref(), "Next.js");
        assert_eq!(item.keywords.len(), 0);
    }

    fn render_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        model: Model<Option<Arc<str>>>,
        open: Model<bool>,
        items: Vec<ComboboxItem>,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        fret_ui_kit::OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "combobox",
            |cx| vec![Combobox::new(model, open).items(items).into_element(cx)],
        );
        ui.set_root(root);
        fret_ui_kit::OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_frame_with_underlay(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        model: Model<Option<Arc<str>>>,
        open: Model<bool>,
        items: Vec<ComboboxItem>,
        underlay_clicked: Model<bool>,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        fret_ui_kit::OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "combobox-underlay",
            move |cx| {
                let underlay = cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Fill;
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        a11y: PressableA11y {
                            role: Some(SemanticsRole::Button),
                            label: Some(Arc::from("Underlay")),
                            test_id: Some(Arc::from("underlay")),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    move |cx, _st| {
                        cx.pressable_toggle_bool(&underlay_clicked);
                        Vec::new()
                    },
                );
                vec![
                    underlay,
                    Combobox::new(model, open).items(items).into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        fret_ui_kit::OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    #[test]
    fn combobox_new_controllable_creates_internal_models_and_applies_defaults() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let model_id_out = Cell::new(None);
        let open_id_out = Cell::new(None);
        let model_out: RefCell<Option<Model<Option<Arc<str>>>>> = RefCell::new(None);
        let open_out: RefCell<Option<Model<bool>>> = RefCell::new(None);

        let render = |ui: &mut UiTree<App>,
                      app: &mut App,
                      services: &mut FakeServices,
                      model_id_out: &Cell<Option<fret_runtime::ModelId>>,
                      open_id_out: &Cell<Option<fret_runtime::ModelId>>,
                      model_out: &RefCell<Option<Model<Option<Arc<str>>>>>,
                      open_out: &RefCell<Option<Model<bool>>>| {
            let next_frame = FrameId(app.frame_id().0.saturating_add(1));
            app.set_frame_id(next_frame);

            fret_ui_kit::OverlayController::begin_frame(app, window);
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "combobox-controllable",
                |cx| {
                    vec![cx.keyed("combobox", |cx| {
                        let items = vec![
                            ComboboxItem::new("alpha", "Alpha"),
                            ComboboxItem::new("beta", "Beta"),
                        ];
                        let combobox = Combobox::new_controllable(
                            cx,
                            None,
                            Some(Arc::from("beta")),
                            None,
                            false,
                        )
                        .items(items);
                        model_id_out.set(Some(combobox.model.id()));
                        open_id_out.set(Some(combobox.open.id()));
                        *model_out.borrow_mut() = Some(combobox.model.clone());
                        *open_out.borrow_mut() = Some(combobox.open.clone());
                        combobox.into_element(cx)
                    })]
                },
            );
            ui.set_root(root);
            fret_ui_kit::OverlayController::render(ui, app, services, window, bounds);
            ui.layout_all(app, services, bounds, 1.0);
        };

        render(
            &mut ui,
            &mut app,
            &mut services,
            &model_id_out,
            &open_id_out,
            &model_out,
            &open_out,
        );
        let first_model = model_id_out.get().expect("value model id");
        let first_open = open_id_out.get().expect("open model id");
        let value = model_out
            .borrow()
            .as_ref()
            .expect("value model")
            .read_ref(&app, |v| v.clone())
            .expect("read value");
        assert_eq!(value.as_deref(), Some("beta"));
        let is_open = open_out
            .borrow()
            .as_ref()
            .expect("open model")
            .read_ref(&app, |v| *v)
            .expect("read open");
        assert!(!is_open);

        render(
            &mut ui,
            &mut app,
            &mut services,
            &model_id_out,
            &open_id_out,
            &model_out,
            &open_out,
        );
        let second_model = model_id_out.get().expect("value model id (second render)");
        let second_open = open_id_out.get().expect("open model id (second render)");
        assert_eq!(first_model, second_model);
        assert_eq!(first_open, second_open);
    }

    #[test]
    fn combobox_new_controllable_prefers_controlled_models() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let value = app.models_mut().insert(Some(Arc::from("alpha")));
        let open = app.models_mut().insert(true);

        let mut services = FakeServices::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );

        let seen = Cell::new(false);
        fret_ui_kit::OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "combobox-controlled",
            |cx| {
                vec![cx.keyed("combobox", |cx| {
                    let combobox = Combobox::new_controllable(
                        cx,
                        Some(value.clone()),
                        Some(Arc::from("beta")),
                        Some(open.clone()),
                        false,
                    );
                    assert_eq!(combobox.model, value);
                    assert_eq!(combobox.open, open);
                    seen.set(true);
                    combobox.into_element(cx)
                })]
            },
        );
        ui.set_root(root);
        fret_ui_kit::OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        assert!(seen.get());
    }

    #[test]
    fn combobox_open_change_builder_sets_handler() {
        let mut app = App::new();
        let value = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);
        let combobox = Combobox::new(value, open).on_open_change(Some(Arc::new(|_open| {})));

        assert!(combobox.on_open_change.is_some());
    }

    #[test]
    fn combobox_open_change_with_reason_builder_sets_handler() {
        let mut app = App::new();
        let value = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);
        let combobox = Combobox::new(value, open)
            .on_open_change_with_reason(Some(Arc::new(|_open, _reason| {})));

        assert!(combobox.on_open_change_with_reason.is_some());
    }

    #[test]
    fn combobox_open_change_complete_builder_sets_handler() {
        let mut app = App::new();
        let value = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);
        let combobox =
            Combobox::new(value, open).on_open_change_complete(Some(Arc::new(|_open| {})));

        assert!(combobox.on_open_change_complete.is_some());
    }

    #[test]
    fn combobox_on_value_change_builder_sets_handler() {
        let mut app = App::new();
        let value = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);
        let combobox = Combobox::new(value, open).on_value_change(Some(Arc::new(|_value| {})));

        assert!(combobox.on_value_change.is_some());
    }

    #[test]
    fn combobox_search_input_exposes_combobox_role_active_descendant_and_value() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = || {
            vec![
                ComboboxItem::new("alpha", "Alpha"),
                ComboboxItem::new("beta", "Beta"),
                ComboboxItem::new("gamma", "Gamma"),
            ]
        };

        // First frame: establish stable trigger bounds.
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items(),
        );

        let _ = app.models_mut().update(&open, |v| *v = true);

        // Second frame: open the popover.
        //
        // `active_descendant` depends on stable element<->node mapping, so we render one extra
        // frame before asserting it (see cmdk tests).
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items(),
        );
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let input = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ComboBox && n.value.is_some())
            .expect("combobox search input node");
        assert!(
            input.flags.expanded,
            "combobox search input should report expanded=true while open"
        );
        assert_eq!(input.value.as_deref(), Some(""));

        let list = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ListBox)
            .expect("listbox node");
        assert!(
            list.labelled_by.iter().any(|id| *id == input.id),
            "listbox should be labelled by the combobox input"
        );
        assert!(
            input.controls.iter().any(|id| *id == list.id),
            "combobox input should control the listbox"
        );

        let active = input
            .active_descendant
            .expect("active_descendant should be set");
        let active_node = snap
            .nodes
            .iter()
            .find(|n| n.id == active)
            .expect("active_descendant should reference a node in the snapshot");
        assert_eq!(active_node.role, SemanticsRole::ListBoxOption);

        let input_id = input.id;
        ui.set_focus(Some(input_id));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::TextInput("a".to_string()),
        );

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model,
            open,
            items(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let input = snap
            .nodes
            .iter()
            .find(|n| n.id == input_id)
            .expect("combobox search input node after typing");
        assert_eq!(input.role, SemanticsRole::ComboBox);
        assert_eq!(input.value.as_deref(), Some("a"));
    }

    #[test]
    fn combobox_pointer_open_auto_focuses_search_input() {
        use fret_core::{Event, Modifiers, MouseButton};

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("beta")));
        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = || {
            vec![
                ComboboxItem::new("alpha", "Alpha"),
                ComboboxItem::new("beta", "Beta"),
                ComboboxItem::new("gamma", "Gamma"),
            ]
        };

        // Frame 1: establish stable trigger bounds.
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ComboBox && n.value.is_none())
            .expect("combobox trigger semantics");
        let trigger_bounds = ui
            .debug_node_visual_bounds(trigger.id)
            .expect("trigger bounds");
        let trigger_center = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: trigger_center,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: trigger_center,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert!(
            app.models().get_copied(&open).unwrap_or(false),
            "expected pointer click to open combobox"
        );

        // Frame 2: open; autofocus should move focus into the search input.
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items(),
        );
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model,
            open,
            items(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let input = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ComboBox && n.value.is_some())
            .expect("combobox search input node");
        assert_eq!(
            ui.focus(),
            Some(input.id),
            "expected pointer-open to autofocus the search input"
        );
    }

    #[test]
    fn combobox_keyboard_open_auto_focuses_search_input() {
        use fret_core::{Event, KeyCode, Modifiers};

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("beta")));
        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = || {
            vec![
                ComboboxItem::new("alpha", "Alpha"),
                ComboboxItem::new("beta", "Beta"),
                ComboboxItem::new("gamma", "Gamma"),
            ]
        };

        let root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items(),
        );

        let trigger = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("combobox trigger node");
        ui.set_focus(Some(trigger));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Enter,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyUp {
                key: KeyCode::Enter,
                modifiers: Modifiers::default(),
            },
        );
        assert!(
            app.models().get_copied(&open).unwrap_or(false),
            "expected Enter to open combobox"
        );

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items(),
        );
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model,
            open,
            items(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let input = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ComboBox && n.value.is_some())
            .expect("combobox search input node");
        assert_eq!(
            ui.focus(),
            Some(input.id),
            "expected keyboard-open to autofocus the search input"
        );
    }

    #[test]
    fn combobox_keyboard_enter_commits_active_item_via_cmdk_on_select_value() {
        use fret_core::{Event, KeyCode, Modifiers};

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = || {
            vec![
                ComboboxItem::new("alpha", "Alpha"),
                ComboboxItem::new("beta", "Beta"),
                ComboboxItem::new("gamma", "Gamma"),
            ]
        };

        let root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items(),
        );

        let trigger = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("combobox trigger node");
        ui.set_focus(Some(trigger));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Enter,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyUp {
                key: KeyCode::Enter,
                modifiers: Modifiers::default(),
            },
        );
        assert!(
            app.models().get_copied(&open).unwrap_or(false),
            "expected Enter to open combobox"
        );

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items(),
        );
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let input = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ComboBox && n.value.is_some())
            .expect("combobox search input node");
        assert_eq!(
            ui.focus(),
            Some(input.id),
            "expected keyboard-open to autofocus the search input"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Enter,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyUp {
                key: KeyCode::Enter,
                modifiers: Modifiers::default(),
            },
        );

        assert_eq!(
            app.models().get_cloned(&model).flatten().as_deref(),
            Some("alpha"),
            "expected Enter on the cmdk input to commit the active item"
        );
        assert_eq!(
            app.models().get_copied(&open),
            Some(false),
            "expected commit to close the combobox"
        );
    }

    #[test]
    fn combobox_close_transition_disables_pointer_move_and_timer_events() {
        use fret_core::{Event, Modifiers, MouseButton, MouseButtons};

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);
        let underlay_clicked = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = || {
            vec![
                ComboboxItem::new("alpha", "Alpha"),
                ComboboxItem::new("beta", "Beta"),
                ComboboxItem::new("gamma", "Gamma"),
            ]
        };

        // Frame 1: closed.
        let _ = render_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items(),
            underlay_clicked.clone(),
        );

        let _ = app.models_mut().update(&open, |v| *v = true);

        // Frame 2: open, capture overlay layer id.
        let _ = render_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items(),
            underlay_clicked.clone(),
        );
        let overlay_id =
            fret_ui_kit::OverlayController::stack_snapshot_for_window(&ui, &mut app, window)
                .topmost_popover
                .expect("expected an open combobox overlay");
        let overlay_root_name = radix_popover::popover_root_name(overlay_id);
        let overlay_root = fret_ui::elements::global_root(window, &overlay_root_name);
        let overlay_node =
            fret_ui::elements::node_for_element(&mut app, window, overlay_root).expect("overlay");
        let overlay_layer = ui.node_layer(overlay_node).expect("overlay layer");

        let info = ui
            .debug_layers_in_paint_order()
            .into_iter()
            .find(|l| l.id == overlay_layer)
            .expect("overlay layer info");
        assert!(info.visible);
        assert!(info.hit_testable);
        assert!(info.wants_pointer_move_events);
        assert!(info.wants_timer_events);

        // Frame 3: close (close transition should remain present but be click-through).
        let _ = app.models_mut().update(&open, |v| *v = false);
        let _ = render_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model,
            open,
            items(),
            underlay_clicked.clone(),
        );

        let info = ui
            .debug_layers_in_paint_order()
            .into_iter()
            .find(|l| l.id == overlay_layer)
            .expect("overlay layer info");
        assert!(info.visible);
        assert!(!info.hit_testable);
        assert_eq!(
            info.pointer_occlusion,
            fret_ui::tree::PointerOcclusion::None
        );
        assert!(!info.wants_pointer_move_events);
        assert!(!info.wants_timer_events);

        // Pointer interactions should go through while closing.
        let underlay_pos = Point::new(Px(10.0), Px(230.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert_eq!(app.models().get_copied(&underlay_clicked), Some(true));

        // Move events should not install timers while closing (no interactive policies).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let effects = app.flush_effects();
        assert!(
            !effects
                .iter()
                .any(|e| matches!(e, fret_runtime::Effect::SetTimer { .. })),
            "expected close transition to not arm timers; effects={effects:?}"
        );
    }

    #[test]
    fn combobox_list_respects_theme_max_height_in_tight_viewports() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(320.0), Px(92.0)),
        );
        let mut services = FakeServices::default();

        let items = || {
            (0..40)
                .map(|i| ComboboxItem::new(format!("v{i}"), format!("Item {i}")))
                .collect::<Vec<_>>()
        };

        // First frame: establish stable trigger bounds.
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items(),
        );
        let _ = app.models_mut().update(&open, |v| *v = true);

        // Second/third frame: open the popover and settle layout.
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items(),
        );
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model,
            open,
            items(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let list = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ListBox)
            .expect("listbox node");
        let list_bounds = ui.debug_node_bounds(list.id).expect("listbox bounds");

        let theme_max_list_h = 280.0;
        assert!(
            list_bounds.size.height.0 <= theme_max_list_h + 0.01,
            "expected listbox height to respect theme max height; list={list_bounds:?}"
        );
    }
}

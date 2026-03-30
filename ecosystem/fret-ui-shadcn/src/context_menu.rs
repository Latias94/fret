use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::direction::LayoutDirection;
use crate::test_id::test_id_slug;
use fret_core::time::Duration;
use fret_core::{Edges, FontId, FontWeight, Point, Px, Rect, Size, TextStyle};
use fret_icons::{IconId, ids};
use fret_runtime::{CommandId, Effect, Model, ModelId, TimerToken, WindowCommandGatingSnapshot};
use fret_ui::action::{
    ActionCx, OnCloseAutoFocus, OnDismissRequest, OnOpenAutoFocus, PressablePointerUpResult,
    UiActionHost, UiPointerActionHost,
};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, Elements, FlexProps, InsetStyle, LayoutStyle, Length,
    MainAlign, Overflow, PointerRegionProps, PositionStyle, PressableProps, RingStyle,
    RovingFlexProps, RovingFocusProps, ScrollAxis, ScrollProps, SemanticsProps, SizeStyle,
    SpacerProps,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::overlay_placement::Align;
use fret_ui::{ElementContext, Theme, ThemeSnapshot, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::collection_semantics::CollectionSemanticsExt as _;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::overlay;
use fret_ui_kit::primitives::context_menu as menu;
use fret_ui_kit::primitives::popper;
use fret_ui_kit::primitives::popper_content;
use fret_ui_kit::primitives::portal_inherited;
use fret_ui_kit::primitives::presence as radix_presence;
use fret_ui_kit::typography;
use fret_ui_kit::{
    ColorRef, IntoUiElement, LayoutRefinement, MetricRef, OverlayController, OverlayPresence,
    Radius, Space, ui,
};

use crate::dropdown_menu::{DropdownMenuAlign, DropdownMenuSide, dropdown_menu_overlay_side};
use crate::menu_authoring;
use crate::overlay_motion;
use crate::popper_arrow::{self, DiamondArrowStyle};
use crate::rtl;
use crate::shortcut_display::{command_shortcut_label, shortcut_text_element};

type ActionPayloadFactory = Arc<dyn Fn() -> Box<dyn Any + Send + Sync> + 'static>;

#[derive(Debug)]
pub enum ContextMenuEntry {
    Item(ContextMenuItem),
    CheckboxItem(ContextMenuCheckboxItem),
    RadioGroup(ContextMenuRadioGroup),
    RadioItem(ContextMenuRadioItem),
    Label(ContextMenuLabel),
    Group(ContextMenuGroup),
    Separator,
}

/// shadcn/ui `ContextMenuTrigger` (v4).
///
/// In the upstream DOM implementation this is a Radix primitive part. In Fret, the trigger element
/// itself is still authored by the caller; this wrapper exists to align the part surface with
/// shadcn docs/examples and to keep room for future trigger-specific defaults.
#[derive(Debug)]
pub struct ContextMenuTrigger {
    child: AnyElement,
}

pub struct ContextMenuTriggerBuild<H, T> {
    child: Option<T>,
    _phantom: PhantomData<fn() -> H>,
}

impl ContextMenuTrigger {
    pub fn new(child: AnyElement) -> Self {
        Self { child }
    }

    /// Builder-first variant that late-lands the trigger child at `into_element(cx)` time.
    pub fn build<H: UiHost, T>(child: T) -> ContextMenuTriggerBuild<H, T>
    where
        T: IntoUiElement<H>,
    {
        ContextMenuTriggerBuild {
            child: Some(child),
            _phantom: PhantomData,
        }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, _cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.child
    }
}

impl<H: UiHost, T> ContextMenuTriggerBuild<H, T>
where
    T: IntoUiElement<H>,
{
    #[track_caller]
    pub fn into_trigger(self, cx: &mut ElementContext<'_, H>) -> ContextMenuTrigger {
        ContextMenuTrigger::new(
            self.child
                .expect("expected context-menu trigger child")
                .into_element(cx),
        )
    }

    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_trigger(cx).into_element(cx)
    }
}

impl<H: UiHost> IntoUiElement<H> for ContextMenuTrigger {
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        ContextMenuTrigger::into_element(self, cx)
    }
}

impl<H: UiHost, T> IntoUiElement<H> for ContextMenuTriggerBuild<H, T>
where
    T: IntoUiElement<H>,
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        ContextMenuTriggerBuild::into_element(self, cx)
    }
}

/// shadcn/ui `ContextMenuContent` (v4).
///
/// Upstream exposes placement-related props on the `Content` part (e.g. `align`, `sideOffset`).
/// Fret's current `ContextMenu` surface owns these configuration knobs. This type provides an
/// adapter surface so call sites can be authored in a part-based style while keeping the current
/// implementation intact.
#[derive(Debug, Clone, Default)]
pub struct ContextMenuContent {
    align: Option<DropdownMenuAlign>,
    side: Option<DropdownMenuSide>,
    side_offset: Option<Px>,
    window_margin: Option<Px>,
    min_width: Option<Px>,
    submenu_min_width: Option<Px>,
    arrow: Option<bool>,
    arrow_size: Option<Px>,
    arrow_padding: Option<Px>,
    align_leading_icons: Option<bool>,
}

impl ContextMenuContent {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn align(mut self, align: DropdownMenuAlign) -> Self {
        self.align = Some(align);
        self
    }

    pub fn side(mut self, side: DropdownMenuSide) -> Self {
        self.side = Some(side);
        self
    }

    pub fn side_offset(mut self, offset: Px) -> Self {
        self.side_offset = Some(offset);
        self
    }

    pub fn window_margin(mut self, margin: Px) -> Self {
        self.window_margin = Some(margin);
        self
    }

    pub fn min_width(mut self, min_width: Px) -> Self {
        self.min_width = Some(min_width);
        self
    }

    pub fn submenu_min_width(mut self, min_width: Px) -> Self {
        self.submenu_min_width = Some(min_width);
        self
    }

    pub fn arrow(mut self, arrow: bool) -> Self {
        self.arrow = Some(arrow);
        self
    }

    pub fn arrow_size(mut self, size: Px) -> Self {
        self.arrow_size = Some(size);
        self
    }

    pub fn arrow_padding(mut self, padding: Px) -> Self {
        self.arrow_padding = Some(padding);
        self
    }

    pub fn align_leading_icons(mut self, align: bool) -> Self {
        self.align_leading_icons = Some(align);
        self
    }

    fn apply_to(self, mut menu: ContextMenu) -> ContextMenu {
        if let Some(v) = self.align {
            menu.align = v;
        }
        if let Some(v) = self.side {
            menu.side = v;
        }
        if let Some(v) = self.side_offset {
            menu.side_offset = v;
        }
        if let Some(v) = self.window_margin {
            menu.window_margin = v;
        }
        if let Some(v) = self.min_width {
            menu.min_width = v;
        }
        if let Some(v) = self.submenu_min_width {
            menu.submenu_min_width = v;
        }
        if let Some(v) = self.arrow {
            menu.arrow = v;
        }
        if let Some(v) = self.arrow_size {
            menu.arrow_size_override = Some(v);
        }
        if let Some(v) = self.arrow_padding {
            menu.arrow_padding_override = Some(v);
        }
        if let Some(v) = self.align_leading_icons {
            menu.align_leading_icons = v;
        }
        menu
    }
}

/// shadcn/ui `ContextMenuPortal` (v4).
///
/// Upstream exports a distinct portal part even though `ContextMenuContent` mounts itself in a
/// portal by default. In Fret the overlay is already rendered in an overlay root, so this is a
/// no-op wrapper that exists for part surface parity (copy/paste examples).
#[derive(Debug, Clone, Default)]
pub struct ContextMenuPortal {
    content: ContextMenuContent,
}

impl ContextMenuPortal {
    pub fn new(content: ContextMenuContent) -> Self {
        Self { content }
    }
}

impl From<ContextMenuPortal> for ContextMenuContent {
    fn from(value: ContextMenuPortal) -> Self {
        value.content
    }
}

/// shadcn/ui `ContextMenuSeparator` (v4).
///
/// In upstream this is a primitive part. In Fret menus we model it as an entry variant.
#[derive(Debug, Clone, Copy, Default)]
pub struct ContextMenuSeparator;

impl ContextMenuSeparator {
    pub fn new() -> Self {
        Self
    }
}

/// shadcn/ui `ContextMenuSub*` helpers (v4).
///
/// Upstream exposes `Sub` / `SubTrigger` / `SubContent` as distinct parts. Fret's current menu
/// model represents submenus as `ContextMenuItem { submenu: Some(Vec<ContextMenuEntry>) }`.
/// These helpers bridge the authoring model without changing the underlying representation.
#[derive(Debug)]
pub struct ContextMenuSubTrigger {
    item: ContextMenuItem,
}

impl ContextMenuSubTrigger {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            item: ContextMenuItem::new(label).close_on_select(false),
        }
    }

    pub fn refine(mut self, f: impl FnOnce(ContextMenuItem) -> ContextMenuItem) -> Self {
        self.item = f(self.item);
        // Sub triggers should not close on select.
        self.item.close_on_select = false;
        self
    }
}

#[derive(Debug)]
pub struct ContextMenuSubContent {
    entries: Vec<ContextMenuEntry>,
}

impl ContextMenuSubContent {
    pub fn new(entries: impl IntoIterator<Item = ContextMenuEntry>) -> Self {
        Self {
            entries: entries.into_iter().collect(),
        }
    }
}

#[derive(Debug)]
pub struct ContextMenuSub {
    trigger: ContextMenuSubTrigger,
    content: ContextMenuSubContent,
}

impl ContextMenuSub {
    pub fn new(trigger: ContextMenuSubTrigger, content: ContextMenuSubContent) -> Self {
        Self { trigger, content }
    }

    pub fn into_entry(self) -> ContextMenuEntry {
        let mut item = self.trigger.item;
        item.submenu = Some(self.content.entries);
        ContextMenuEntry::Item(item)
    }
}

impl From<ContextMenuItem> for ContextMenuEntry {
    fn from(value: ContextMenuItem) -> Self {
        Self::Item(value)
    }
}

impl From<ContextMenuCheckboxItem> for ContextMenuEntry {
    fn from(value: ContextMenuCheckboxItem) -> Self {
        Self::CheckboxItem(value)
    }
}

impl From<ContextMenuRadioGroup> for ContextMenuEntry {
    fn from(value: ContextMenuRadioGroup) -> Self {
        Self::RadioGroup(value)
    }
}

impl From<ContextMenuRadioItem> for ContextMenuEntry {
    fn from(value: ContextMenuRadioItem) -> Self {
        Self::RadioItem(value)
    }
}

impl From<ContextMenuLabel> for ContextMenuEntry {
    fn from(value: ContextMenuLabel) -> Self {
        Self::Label(value)
    }
}

impl From<ContextMenuGroup> for ContextMenuEntry {
    fn from(value: ContextMenuGroup) -> Self {
        Self::Group(value)
    }
}

impl From<ContextMenuSeparator> for ContextMenuEntry {
    fn from(_value: ContextMenuSeparator) -> Self {
        Self::Separator
    }
}

fn alpha_mul(mut c: fret_core::Color, mul: f32) -> fret_core::Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn menu_destructive_focus_bg(
    theme: &ThemeSnapshot,
    destructive_fg: fret_core::Color,
) -> fret_core::Color {
    crate::theme_variants::menu_destructive_focus_bg(theme, destructive_fg)
}

const CONTEXT_MENU_CANCEL_OPEN_DELAY: Duration = Duration::from_millis(500);
const CONTEXT_MENU_CANCEL_OPEN_MOVE_THRESHOLD_PX: f32 = 1.0;

type OnOpenChange = Arc<dyn Fn(bool) + Send + Sync + 'static>;
type OnCheckedChange = menu_authoring::OnCheckedChange;
type OnValueChange = menu_authoring::OnValueChange;
pub type ContextMenuCheckboxChecked = menu_authoring::MenuCheckboxChecked;
pub type ContextMenuRadioValue = menu_authoring::MenuRadioValue;

#[derive(Default)]
struct ContextMenuOpenChangeCallbackState {
    initialized: bool,
    last_open: bool,
    pending_complete: Option<bool>,
}

fn context_menu_open_change_events(
    state: &mut ContextMenuOpenChangeCallbackState,
    open: bool,
    present: bool,
    animating: bool,
) -> (Option<bool>, Option<bool>) {
    let mut changed = None;
    let mut completed = None;

    if !state.initialized {
        state.initialized = true;
        state.last_open = open;
    } else if state.last_open != open {
        state.last_open = open;
        state.pending_complete = Some(open);
        changed = Some(open);
    }

    if state.pending_complete == Some(open) && present == open && !animating {
        state.pending_complete = None;
        completed = Some(open);
    }

    (changed, completed)
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct ContextMenuCancelOpenState {
    active: bool,
    allow_item_mouse_up: bool,
    pointer_id: Option<fret_core::PointerId>,
    anchor: Option<Point>,
    armed: Option<TimerToken>,
    moved_from_anchor: bool,
}

type ContextMenuCancelOpenShared = Arc<Mutex<ContextMenuCancelOpenState>>;

fn context_menu_cancel_open_shared() -> ContextMenuCancelOpenShared {
    Arc::new(Mutex::new(ContextMenuCancelOpenState::default()))
}

fn context_menu_cancel_open_distance_exceeds_threshold(anchor: Point, position: Point) -> bool {
    let dx = anchor.x.0 - position.x.0;
    let dy = anchor.y.0 - position.y.0;
    (dx * dx + dy * dy)
        > CONTEXT_MENU_CANCEL_OPEN_MOVE_THRESHOLD_PX * CONTEXT_MENU_CANCEL_OPEN_MOVE_THRESHOLD_PX
}

fn context_menu_cancel_open_clear_inner(
    host: &mut dyn fret_ui::action::UiActionHost,
    state: &mut ContextMenuCancelOpenState,
) {
    if let Some(token) = state.armed.take() {
        host.push_effect(Effect::CancelTimer { token });
    }
    *state = ContextMenuCancelOpenState::default();
}

fn context_menu_cancel_open_start(
    shared: &ContextMenuCancelOpenShared,
    host: &mut dyn UiPointerActionHost,
    window: fret_core::AppWindowId,
    pointer_id: fret_core::PointerId,
    anchor: Point,
) {
    let token = host.next_timer_token();
    let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());
    context_menu_cancel_open_clear_inner(host, &mut state);
    state.active = true;
    state.pointer_id = Some(pointer_id);
    state.anchor = Some(anchor);
    state.armed = Some(token);
    host.push_effect(Effect::SetTimer {
        window: Some(window),
        token,
        after: CONTEXT_MENU_CANCEL_OPEN_DELAY,
        repeat: None,
    });
}

fn context_menu_cancel_open_mark_moved_if_needed(
    shared: &ContextMenuCancelOpenShared,
    pointer_id: fret_core::PointerId,
    position: Point,
) {
    let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());
    if !state.active || state.pointer_id != Some(pointer_id) || state.moved_from_anchor {
        return;
    }
    let Some(anchor) = state.anchor else {
        return;
    };
    if context_menu_cancel_open_distance_exceeds_threshold(anchor, position) {
        state.moved_from_anchor = true;
    }
}

fn context_menu_cancel_open_on_timer(
    shared: &ContextMenuCancelOpenShared,
    _host: &mut dyn fret_ui::action::UiActionHost,
    token: TimerToken,
) {
    let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());
    if state.armed != Some(token) {
        return;
    }
    state.armed = None;
    state.allow_item_mouse_up = true;
}

fn context_menu_cancel_open_on_pointer_up(
    shared: &ContextMenuCancelOpenShared,
    host: &mut dyn fret_ui::action::UiActionHost,
    open: &Model<bool>,
    pointer_id: fret_core::PointerId,
    position: Point,
    button: fret_core::MouseButton,
) {
    if button != fret_core::MouseButton::Right {
        return;
    }

    let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());
    if !state.active || state.pointer_id != Some(pointer_id) {
        return;
    }

    let should_cancel = state.allow_item_mouse_up
        && state.anchor.is_some_and(|anchor| {
            let near_anchor =
                !context_menu_cancel_open_distance_exceeds_threshold(anchor, position);
            if state.moved_from_anchor {
                true
            } else {
                !near_anchor
            }
        });
    if should_cancel {
        let _ = host.models_mut().update(open, |v| *v = false);
    }
    context_menu_cancel_open_clear_inner(host, &mut state);
}

fn context_menu_cancel_open_stop_without_close(
    shared: &ContextMenuCancelOpenShared,
    host: &mut dyn fret_ui::action::UiActionHost,
    pointer_id: fret_core::PointerId,
) {
    let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());
    if !state.active || state.pointer_id != Some(pointer_id) {
        return;
    }
    context_menu_cancel_open_clear_inner(host, &mut state);
}

fn context_menu_cancel_open_item_pointer_up_handler(
    shared: ContextMenuCancelOpenShared,
    open: Model<bool>,
) -> fret_ui::action::OnPressablePointerUp {
    Arc::new(move |host, _acx, up| {
        context_menu_cancel_open_on_pointer_up(
            &shared,
            host,
            &open,
            up.pointer_id,
            up.position,
            up.button,
        );
        // Right-button release should not go through the Pressable default activation path.
        //
        // Base UI opens via the `contextmenu` event and then installs a guarded `mouseup` listener
        // to potentially cancel-open (after a delay). See:
        // - `repo-ref/base-ui/packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx`
        // - `repo-ref/base-ui/packages/react/src/context-menu/trigger/ContextMenuTrigger.test.tsx`
        if up.button == fret_core::MouseButton::Right {
            PressablePointerUpResult::SkipActivate
        } else {
            PressablePointerUpResult::Continue
        }
    })
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ContextMenuItemVariant {
    #[default]
    Default,
    Destructive,
}

pub struct ContextMenuItem {
    pub label: Arc<str>,
    pub value: Arc<str>,
    pub inset: bool,
    pub leading: Option<AnyElement>,
    pub leading_icon: Option<IconId>,
    pub disabled: bool,
    pub close_on_select: bool,
    pub command: Option<CommandId>,
    pub action_payload: Option<ActionPayloadFactory>,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub trailing: Option<AnyElement>,
    pub submenu: Option<Vec<ContextMenuEntry>>,
    pub variant: ContextMenuItemVariant,
}

impl std::fmt::Debug for ContextMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContextMenuItem")
            .field("label", &self.label)
            .field("value", &self.value)
            .field("inset", &self.inset)
            .field("leading", &self.leading)
            .field("leading_icon", &self.leading_icon)
            .field("disabled", &self.disabled)
            .field("close_on_select", &self.close_on_select)
            .field("command", &self.command)
            .field("action_payload", &self.action_payload.is_some())
            .field("a11y_label", &self.a11y_label)
            .field("test_id", &self.test_id)
            .field("trailing", &self.trailing)
            .field("submenu", &self.submenu)
            .field("variant", &self.variant)
            .finish()
    }
}

impl ContextMenuItem {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        let label = label.into();
        Self {
            label: label.clone(),
            value: label,
            inset: false,
            leading: None,
            leading_icon: None,
            disabled: false,
            close_on_select: true,
            command: None,
            action_payload: None,
            a11y_label: None,
            test_id: None,
            trailing: None,
            submenu: None,
            variant: ContextMenuItemVariant::Default,
        }
    }

    pub fn value(mut self, value: impl Into<Arc<str>>) -> Self {
        self.value = value.into();
        self
    }

    pub fn inset(mut self, inset: bool) -> Self {
        self.inset = inset;
        self
    }

    pub fn leading(mut self, element: AnyElement) -> Self {
        self.leading_icon = None;
        self.leading = Some(element);
        self
    }

    /// Prefer this over `leading(icon(cx, ...))` so the icon can inherit the item's `currentColor`.
    pub fn leading_icon(mut self, icon: IconId) -> Self {
        self.leading = None;
        self.leading_icon = Some(icon);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn close_on_select(mut self, close: bool) -> Self {
        self.close_on_select = close;
        self
    }

    pub fn on_select(mut self, command: impl Into<CommandId>) -> Self {
        self.command = Some(command.into());
        self
    }

    /// Bind a stable action ID to this context-menu item (action-first authoring).
    ///
    /// v1 compatibility: `ActionId` is `CommandId`-compatible (ADR 0307), so this dispatches
    /// through the existing command pipeline.
    pub fn action(mut self, action: impl Into<fret_runtime::ActionId>) -> Self {
        self.command = Some(action.into());
        self
    }

    /// Attach a payload for parameterized actions while staying on the native context-menu item surface.
    pub fn action_payload<T>(mut self, payload: T) -> Self
    where
        T: Any + Send + Sync + Clone + 'static,
    {
        let payload = Arc::new(payload);
        self.action_payload = Some(Arc::new(move || Box::new(payload.as_ref().clone())));
        self
    }

    /// Like [`ContextMenuItem::action_payload`], but computes the payload lazily on selection.
    pub fn action_payload_factory(mut self, payload: ActionPayloadFactory) -> Self {
        self.action_payload = Some(payload);
        self
    }

    pub fn submenu(mut self, entries: impl IntoIterator<Item = ContextMenuEntry>) -> Self {
        self.submenu = Some(entries.into_iter().collect());
        self
    }

    pub fn variant(mut self, variant: ContextMenuItemVariant) -> Self {
        self.variant = variant;
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

    pub fn trailing(mut self, element: AnyElement) -> Self {
        self.trailing = Some(element);
        self
    }
}

/// shadcn/ui `ContextMenuLabel` (v4).
#[derive(Debug, Clone)]
pub struct ContextMenuLabel {
    pub text: Arc<str>,
    pub inset: bool,
}

impl ContextMenuLabel {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            inset: false,
        }
    }

    pub fn inset(mut self, inset: bool) -> Self {
        self.inset = inset;
        self
    }
}

/// shadcn/ui `ContextMenuGroup` (v4).
///
/// In the upstream DOM implementation, this is a structural wrapper (Radix `Menu.Group`).
/// In Fret, we preserve this structure so it can appear in the semantics tree.
#[derive(Debug)]
pub struct ContextMenuGroup {
    pub entries: Vec<ContextMenuEntry>,
}

impl ContextMenuGroup {
    pub fn new(entries: impl IntoIterator<Item = ContextMenuEntry>) -> Self {
        Self {
            entries: entries.into_iter().collect(),
        }
    }
}

/// shadcn/ui `ContextMenuShortcut` (v4).
///
/// This is typically rendered as trailing, muted text inside a menu item.
#[derive(Debug, Clone)]
pub struct ContextMenuShortcut {
    pub text: Arc<str>,
}

impl ContextMenuShortcut {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();
        let fg = theme.color_token("muted-foreground");
        let font_size = theme.metric_token("font.size");
        let font_line_height = theme.metric_token("font.line_height");
        let mut style = typography::fixed_line_box_style(FontId::ui(), font_size, font_line_height);
        style.weight = FontWeight::NORMAL;
        style.letter_spacing_em = Some(0.12);

        shortcut_text_element(
            cx,
            &theme,
            self.text,
            style,
            fg,
            LayoutRefinement::default().flex_none(),
        )
    }
}

/// shadcn/ui `ContextMenuCheckboxItem` (v4).
pub struct ContextMenuCheckboxItem {
    pub label: Arc<str>,
    pub value: Arc<str>,
    pub checked: ContextMenuCheckboxChecked,
    pub leading: Option<AnyElement>,
    pub disabled: bool,
    pub close_on_select: bool,
    pub command: Option<CommandId>,
    pub on_checked_change: Option<OnCheckedChange>,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub trailing: Option<AnyElement>,
}

impl std::fmt::Debug for ContextMenuCheckboxItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContextMenuCheckboxItem")
            .field("label", &self.label)
            .field("value", &self.value)
            .field("checked", &self.checked)
            .field("leading", &self.leading.is_some())
            .field("disabled", &self.disabled)
            .field("close_on_select", &self.close_on_select)
            .field("command", &self.command)
            .field("on_checked_change", &self.on_checked_change.is_some())
            .field("a11y_label", &self.a11y_label)
            .field("test_id", &self.test_id)
            .field("trailing", &self.trailing.is_some())
            .finish()
    }
}

impl ContextMenuCheckboxItem {
    pub fn new(checked: Model<bool>, label: impl Into<Arc<str>>) -> Self {
        let label = label.into();
        Self {
            label: label.clone(),
            value: label,
            checked: ContextMenuCheckboxChecked::Model(checked),
            leading: None,
            disabled: false,
            close_on_select: false,
            command: None,
            on_checked_change: None,
            a11y_label: None,
            test_id: None,
            trailing: None,
        }
    }

    /// Creates a checkbox item from a plain snapshot, mirroring the upstream
    /// `checked` + `onCheckedChange` authoring path without forcing a dedicated `Model<bool>`.
    pub fn from_checked(checked: bool, label: impl Into<Arc<str>>) -> Self {
        let label = label.into();
        Self {
            label: label.clone(),
            value: label,
            checked: ContextMenuCheckboxChecked::Value(checked),
            leading: None,
            disabled: false,
            close_on_select: false,
            command: None,
            on_checked_change: None,
            a11y_label: None,
            test_id: None,
            trailing: None,
        }
    }

    pub fn value(mut self, value: impl Into<Arc<str>>) -> Self {
        self.value = value.into();
        self
    }

    pub fn leading(mut self, element: AnyElement) -> Self {
        self.leading = Some(element);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn close_on_select(mut self, close: bool) -> Self {
        self.close_on_select = close;
        self
    }

    pub fn on_select(mut self, command: impl Into<CommandId>) -> Self {
        self.command = Some(command.into());
        self
    }

    /// Called when the user toggles this item (Radix `onCheckedChange`-style).
    ///
    /// The callback receives the next checked snapshot. When this item is model-backed via
    /// [`ContextMenuCheckboxItem::new`], the model update happens before the callback.
    pub fn on_checked_change(
        mut self,
        f: impl Fn(&mut dyn UiActionHost, ActionCx, bool) + 'static,
    ) -> Self {
        self.on_checked_change = Some(Arc::new(f));
        self
    }

    /// Bind a stable action ID to this context-menu checkbox item (action-first authoring).
    ///
    /// v1 compatibility: `ActionId` is `CommandId`-compatible (ADR 0307), so this dispatches
    /// through the existing command pipeline.
    pub fn action(mut self, action: impl Into<fret_runtime::ActionId>) -> Self {
        self.command = Some(action.into());
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

    pub fn trailing(mut self, element: AnyElement) -> Self {
        self.trailing = Some(element);
        self
    }
}

/// shadcn/ui `ContextMenuRadioGroup` (v4).
pub struct ContextMenuRadioGroup {
    pub value: ContextMenuRadioValue,
    pub on_value_change: Option<OnValueChange>,
    pub items: Vec<ContextMenuRadioItemSpec>,
}

impl std::fmt::Debug for ContextMenuRadioGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContextMenuRadioGroup")
            .field("value", &self.value)
            .field("on_value_change", &self.on_value_change.is_some())
            .field("items_len", &self.items.len())
            .finish()
    }
}

impl ContextMenuRadioGroup {
    pub fn new(value: Model<Option<Arc<str>>>) -> Self {
        Self {
            value: ContextMenuRadioValue::Model(value),
            on_value_change: None,
            items: Vec::new(),
        }
    }

    /// Creates a radio group from a plain snapshot, mirroring the upstream
    /// `value` + `onValueChange` authoring path without forcing a dedicated model.
    pub fn from_value<T>(value: Option<T>) -> Self
    where
        T: Into<Arc<str>>,
    {
        Self {
            value: ContextMenuRadioValue::Value(value.map(Into::into)),
            on_value_change: None,
            items: Vec::new(),
        }
    }

    pub fn item(mut self, item: ContextMenuRadioItemSpec) -> Self {
        self.items.push(item);
        self
    }

    /// Called when the user picks a different radio value (Radix `onValueChange`-style).
    ///
    /// The callback receives the chosen value. When this group is model-backed via
    /// [`ContextMenuRadioGroup::new`], the model update happens before the callback.
    pub fn on_value_change(
        mut self,
        f: impl Fn(&mut dyn UiActionHost, ActionCx, Arc<str>) + 'static,
    ) -> Self {
        self.on_value_change = Some(Arc::new(f));
        self
    }
}

#[derive(Debug)]
pub struct ContextMenuRadioItemSpec {
    pub label: Arc<str>,
    pub value: Arc<str>,
    pub leading: Option<AnyElement>,
    pub disabled: bool,
    pub close_on_select: bool,
    pub command: Option<CommandId>,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub trailing: Option<AnyElement>,
}

impl ContextMenuRadioItemSpec {
    pub fn new(value: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        let value = value.into();
        let label = label.into();
        Self {
            label,
            value,
            leading: None,
            disabled: false,
            close_on_select: true,
            command: None,
            a11y_label: None,
            test_id: None,
            trailing: None,
        }
    }

    pub fn leading(mut self, element: AnyElement) -> Self {
        self.leading = Some(element);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn close_on_select(mut self, close: bool) -> Self {
        self.close_on_select = close;
        self
    }

    pub fn on_select(mut self, command: impl Into<CommandId>) -> Self {
        self.command = Some(command.into());
        self
    }

    /// Bind a stable action ID to this context-menu radio item spec (action-first authoring).
    ///
    /// v1 compatibility: `ActionId` is `CommandId`-compatible (ADR 0307), so this dispatches
    /// through the existing command pipeline.
    pub fn action(mut self, action: impl Into<fret_runtime::ActionId>) -> Self {
        self.command = Some(action.into());
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

    pub fn trailing(mut self, element: AnyElement) -> Self {
        self.trailing = Some(element);
        self
    }

    fn into_item(
        self,
        group_value: ContextMenuRadioValue,
        on_value_change: Option<OnValueChange>,
    ) -> ContextMenuRadioItem {
        ContextMenuRadioItem {
            label: self.label,
            value: self.value,
            group_value,
            leading: self.leading,
            disabled: self.disabled,
            close_on_select: self.close_on_select,
            command: self.command,
            on_value_change,
            a11y_label: self.a11y_label,
            test_id: self.test_id,
            trailing: self.trailing,
        }
    }
}

/// shadcn/ui `ContextMenuRadioItem` (v4).
pub struct ContextMenuRadioItem {
    pub label: Arc<str>,
    pub value: Arc<str>,
    pub group_value: ContextMenuRadioValue,
    pub leading: Option<AnyElement>,
    pub disabled: bool,
    pub close_on_select: bool,
    pub command: Option<CommandId>,
    pub on_value_change: Option<OnValueChange>,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub trailing: Option<AnyElement>,
}

impl std::fmt::Debug for ContextMenuRadioItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContextMenuRadioItem")
            .field("label", &self.label)
            .field("value", &self.value)
            .field("group_value", &self.group_value)
            .field("leading", &self.leading.is_some())
            .field("disabled", &self.disabled)
            .field("close_on_select", &self.close_on_select)
            .field("command", &self.command)
            .field("on_value_change", &self.on_value_change.is_some())
            .field("a11y_label", &self.a11y_label)
            .field("test_id", &self.test_id)
            .field("trailing", &self.trailing.is_some())
            .finish()
    }
}

impl ContextMenuRadioItem {
    pub fn new(
        group_value: Model<Option<Arc<str>>>,
        value: impl Into<Arc<str>>,
        label: impl Into<Arc<str>>,
    ) -> Self {
        let value = value.into();
        let label = label.into();
        Self {
            label,
            value,
            group_value: ContextMenuRadioValue::Model(group_value),
            leading: None,
            disabled: false,
            close_on_select: true,
            command: None,
            on_value_change: None,
            a11y_label: None,
            test_id: None,
            trailing: None,
        }
    }

    /// Creates a radio item from a plain selected-value snapshot.
    pub fn from_value<T>(
        group_value: Option<T>,
        value: impl Into<Arc<str>>,
        label: impl Into<Arc<str>>,
    ) -> Self
    where
        T: Into<Arc<str>>,
    {
        let value = value.into();
        let label = label.into();
        Self {
            label,
            value,
            group_value: ContextMenuRadioValue::Value(group_value.map(Into::into)),
            leading: None,
            disabled: false,
            close_on_select: true,
            command: None,
            on_value_change: None,
            a11y_label: None,
            test_id: None,
            trailing: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn close_on_select(mut self, close: bool) -> Self {
        self.close_on_select = close;
        self
    }

    pub fn on_select(mut self, command: impl Into<CommandId>) -> Self {
        self.command = Some(command.into());
        self
    }

    /// Called when the user picks this radio item (Radix `onValueChange`-style).
    ///
    /// The callback receives the chosen value. When this item is model-backed via
    /// [`ContextMenuRadioItem::new`], the model update happens before the callback.
    pub fn on_value_change(
        mut self,
        f: impl Fn(&mut dyn UiActionHost, ActionCx, Arc<str>) + 'static,
    ) -> Self {
        self.on_value_change = Some(Arc::new(f));
        self
    }

    /// Bind a stable action ID to this context-menu radio item (action-first authoring).
    ///
    /// v1 compatibility: `ActionId` is `CommandId`-compatible (ADR 0307), so this dispatches
    /// through the existing command pipeline.
    pub fn action(mut self, action: impl Into<fret_runtime::ActionId>) -> Self {
        self.command = Some(action.into());
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

    pub fn leading(mut self, element: AnyElement) -> Self {
        self.leading = Some(element);
        self
    }

    pub fn trailing(mut self, element: AnyElement) -> Self {
        self.trailing = Some(element);
        self
    }
}

fn estimated_menu_panel_height_for_entries(
    entries: &[ContextMenuEntry],
    row_height: Px,
    max_height: Px,
) -> Px {
    fn leaf_height(entries: &[ContextMenuEntry], row_height: Px) -> f32 {
        let mut h = 0.0f32;
        for entry in entries {
            match entry {
                ContextMenuEntry::Separator => {
                    // new-york-v4: `Separator` uses `-mx-1 my-1` (1px line + 4px + 4px).
                    h += 9.0;
                }
                ContextMenuEntry::Label(_)
                | ContextMenuEntry::Item(_)
                | ContextMenuEntry::CheckboxItem(_)
                | ContextMenuEntry::RadioItem(_) => {
                    h += row_height.0.max(0.0);
                }
                ContextMenuEntry::Group(group) => {
                    h += leaf_height(&group.entries, row_height);
                }
                ContextMenuEntry::RadioGroup(group) => {
                    h += row_height.0.max(0.0) * (group.items.len() as f32);
                }
            }
        }
        h
    }

    // new-york-v4: menu panels use `p-1` and `border`.
    let panel_padding_y = Px(8.0);
    let panel_border_y = Px(2.0);

    let mut height = Px(panel_padding_y.0 + panel_border_y.0);
    height.0 += leaf_height(entries, row_height);

    let height = height.0.max(0.0);
    Px(height.min(max_height.0.max(0.0)))
}

fn take_submenu_entries_by_value(
    entries: &mut [ContextMenuEntry],
    open_value: &str,
) -> Option<Vec<ContextMenuEntry>> {
    for entry in entries {
        match entry {
            ContextMenuEntry::Item(item) => {
                if item.value.as_ref() == open_value {
                    if let Some(submenu) = item.submenu.take() {
                        // Preserve "has submenu" for render passes that only need the marker.
                        item.submenu = Some(Vec::new());
                        return Some(submenu);
                    }
                    return None;
                }
            }
            ContextMenuEntry::Group(group) => {
                if let Some(found) = take_submenu_entries_by_value(&mut group.entries, open_value) {
                    return Some(found);
                }
            }
            ContextMenuEntry::CheckboxItem(_)
            | ContextMenuEntry::RadioGroup(_)
            | ContextMenuEntry::RadioItem(_)
            | ContextMenuEntry::Label(_)
            | ContextMenuEntry::Separator => {}
        }
    }
    None
}

fn menu_structural_group<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    role: fret_core::SemanticsRole,
    children: Vec<AnyElement>,
) -> AnyElement {
    cx.semantic_flex(
        fret_ui::element::SemanticFlexProps {
            role,
            flex: FlexProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                direction: fret_core::Axis::Vertical,
                gap: Px(0.0).into(),
                padding: Edges::all(Px(0.0)).into(),
                justify: MainAlign::Start,
                align: CrossAlign::Stretch,
                wrap: false,
            },
        },
        move |_cx| children,
    )
}

#[derive(Clone)]
struct ContextMenuRenderEnv {
    open: Model<bool>,
    cancel_open: ContextMenuCancelOpenShared,
    gating: WindowCommandGatingSnapshot,
    test_id_prefix: Option<Arc<str>>,
    reserve_leading_slot: bool,
    item_count: usize,
    ring: RingStyle,
    border: fret_core::Color,
    radius_sm: Px,
    pad_x: Px,
    pad_x_inset: Px,
    pad_y: Px,
    font_size: Px,
    font_line_height: Px,
    text_style: TextStyle,
    text_disabled: fret_core::Color,
    label_fg: fret_core::Color,
    accent: fret_core::Color,
    accent_fg: fret_core::Color,
    fg: fret_core::Color,
    destructive_fg: fret_core::Color,
    destructive_bg: fret_core::Color,
    submenu_models: menu::sub::MenuSubmenuModels,
}

impl ContextMenuRenderEnv {
    fn render_entries<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
        entries: Vec<ContextMenuEntry>,
        item_ix: &mut usize,
    ) -> Elements {
        let mut out: Vec<AnyElement> = Vec::with_capacity(entries.len());

        for entry in entries {
            match entry {
                ContextMenuEntry::Group(group) => {
                    let children = self.render_entries(cx, group.entries, item_ix);
                    out.push(menu_structural_group(
                        cx,
                        fret_core::SemanticsRole::Group,
                        children.into_vec(),
                    ));
                }
                ContextMenuEntry::RadioGroup(group) => {
                    let mut children: Vec<AnyElement> = Vec::with_capacity(group.items.len());
                    for spec in group.items {
                        children.push(self.render_radio_item(
                            cx,
                            spec.into_item(group.value.clone(), group.on_value_change.clone()),
                            item_ix,
                        ));
                    }
                    out.push(menu_structural_group(
                        cx,
                        fret_core::SemanticsRole::Group,
                        children,
                    ));
                }
                ContextMenuEntry::Label(label) => out.push(self.render_label(cx, label)),
                ContextMenuEntry::Separator => out.push(self.render_separator(cx)),
                ContextMenuEntry::Item(item) => out.push(self.render_item(cx, item, item_ix)),
                ContextMenuEntry::CheckboxItem(item) => {
                    out.push(self.render_checkbox_item(cx, item, item_ix));
                }
                ContextMenuEntry::RadioItem(item) => {
                    out.push(self.render_radio_item(cx, item, item_ix))
                }
            }
        }

        out.into()
    }

    fn render_label<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
        label: ContextMenuLabel,
    ) -> AnyElement {
        let dir = crate::direction::use_direction(cx, None);
        let pad_left = if label.inset {
            self.pad_x_inset
        } else {
            self.pad_x
        };
        let text = label.text;
        let font_size = self.font_size;
        let font_line_height = self.font_line_height;
        let label_fg = self.label_fg;
        let pad_x = self.pad_x;
        let pad_y = self.pad_y;

        cx.container(
            ContainerProps {
                layout: LayoutStyle::default(),
                padding: rtl::padding_edges_with_inline_start_end(
                    dir, pad_y, pad_y, pad_left, pad_x,
                )
                .into(),
                ..Default::default()
            },
            move |cx| {
                vec![
                    ui::text(text)
                        .text_size_px(font_size)
                        .fixed_line_box_px(font_line_height)
                        .line_box_in_bounds()
                        .font_medium()
                        .nowrap()
                        .text_color(ColorRef::Color(label_fg))
                        .into_element(cx),
                ]
            },
        )
    }

    fn render_separator<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let border = self.border;
        cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Px(Px(1.0));
                    // new-york-v4: `Separator` uses `-mx-1 my-1`.
                    layout.margin.left = fret_ui::element::MarginEdge::Px(Px(-4.0));
                    layout.margin.right = fret_ui::element::MarginEdge::Px(Px(-4.0));
                    layout.margin.top = fret_ui::element::MarginEdge::Px(Px(4.0));
                    layout.margin.bottom = fret_ui::element::MarginEdge::Px(Px(4.0));
                    layout
                },
                padding: Edges::all(Px(0.0)).into(),
                background: Some(border),
                ..Default::default()
            },
            |_cx| Vec::new(),
        )
    }

    fn render_item<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
        item: ContextMenuItem,
        item_ix: &mut usize,
    ) -> AnyElement {
        let collection_index = *item_ix;
        *item_ix = (*item_ix).saturating_add(1);

        let label = item.label.clone();
        let value = item.value.clone();
        let a11y_label = item.a11y_label.clone().or_else(|| Some(label.clone()));
        let test_id = item.test_id.clone().or_else(|| {
            self.test_id_prefix.as_ref().map(|prefix| {
                Arc::<str>::from(format!("{prefix}-item-{}", test_id_slug(value.as_ref())))
            })
        });
        let chrome_test_id = test_id
            .clone()
            .map(|id| Arc::<str>::from(format!("{id}.chrome")));
        let close_on_select = item.close_on_select;
        let command = item.command;
        let action_payload = item.action_payload;
        let disabled = item.disabled
            || crate::command_gating::command_is_disabled_by_gating(
                &*cx.app,
                &self.gating,
                command.as_ref(),
            );
        let leading = item.leading;
        let leading_icon = item.leading_icon;
        let trailing = item.trailing;
        let variant = item.variant;
        let pad_left = if item.inset {
            self.pad_x_inset
        } else {
            self.pad_x
        };

        let open_for_item = self.open.clone();
        let cancel_open_for_item = self.cancel_open.clone();
        let ring = self.ring;
        let item_count = self.item_count;
        let reserve_leading_slot = self.reserve_leading_slot;
        let submenu_for_item = self.submenu_models.clone();
        let text_style = self.text_style.clone();
        let font_size = self.font_size;
        let font_line_height = self.font_line_height;
        let pad_x = self.pad_x;
        let pad_y = self.pad_y;
        let radius_sm = self.radius_sm;
        let text_disabled = self.text_disabled;
        let label_fg = self.label_fg;
        let fg = self.fg;
        let accent = self.accent;
        let accent_fg = self.accent_fg;
        let destructive_fg = self.destructive_fg;
        let destructive_bg = self.destructive_bg;

        cx.keyed(value.clone(), move |cx| {
            cx.pressable_with_id_props(move |cx, st, item_id| {
                menu::sub_content::wire_item(cx, item_id, disabled, &submenu_for_item);
                cx.pressable_add_on_pointer_up(context_menu_cancel_open_item_pointer_up_handler(
                    cancel_open_for_item.clone(),
                    open_for_item.clone(),
                ));

                if !disabled {
                    if let Some(payload) = action_payload.clone() {
                        cx.pressable_dispatch_command_with_payload_factory_if_enabled_opt(
                            command.clone(),
                            payload,
                        );
                    } else {
                        cx.pressable_dispatch_command_if_enabled_opt(command.clone());
                    }
                    if close_on_select {
                        cx.pressable_set_bool(&open_for_item, false);
                    }
                }

                let trailing = trailing.or_else(|| {
                    command.as_ref().and_then(|cmd| {
                        command_shortcut_label(cx, cmd, fret_runtime::Platform::current())
                            .map(|text| ContextMenuShortcut::new(text).into_element(cx))
                    })
                });

                let props = PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.min_height = Some(Length::Px(Px(28.0)));
                        layout
                    },
                    enabled: !disabled,
                    focusable: !disabled,
                    focus_ring: Some(ring),
                    a11y: {
                        let mut a11y = menu::item::menu_item_a11y(a11y_label, None);
                        a11y.test_id = test_id.clone();
                        a11y.with_collection_position(collection_index, item_count)
                    },
                    ..Default::default()
                };

                let mut row_bg = fret_core::Color::TRANSPARENT;
                let mut row_fg = if variant == ContextMenuItemVariant::Destructive {
                    destructive_fg
                } else {
                    fg
                };
                if st.hovered || st.pressed || st.focused {
                    if variant == ContextMenuItemVariant::Destructive {
                        row_bg = destructive_bg;
                        row_fg = destructive_fg;
                    } else {
                        row_bg = accent;
                        row_fg = accent_fg;
                    }
                }

                let icon_fg = if variant == ContextMenuItemVariant::Destructive {
                    destructive_fg
                } else {
                    label_fg
                };

                let children = menu_row_children(
                    cx,
                    label.clone(),
                    leading,
                    leading_icon,
                    reserve_leading_slot,
                    trailing,
                    false,
                    None,
                    disabled,
                    row_bg,
                    row_fg,
                    icon_fg,
                    text_style.clone(),
                    font_size,
                    font_line_height,
                    pad_left,
                    pad_x,
                    pad_y,
                    radius_sm,
                    text_disabled,
                    chrome_test_id.clone(),
                );

                (props, children)
            })
        })
    }

    fn render_checkbox_item<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
        item: ContextMenuCheckboxItem,
        item_ix: &mut usize,
    ) -> AnyElement {
        let collection_index = *item_ix;
        *item_ix = (*item_ix).saturating_add(1);

        let label = item.label.clone();
        let value = item.value.clone();
        let checked = item.checked.clone();
        let on_checked_change = item.on_checked_change.clone();
        let a11y_label = item.a11y_label.clone().or_else(|| Some(label.clone()));
        let test_id = item.test_id.clone();
        let chrome_test_id = test_id
            .clone()
            .map(|id| Arc::<str>::from(format!("{id}.chrome")));
        let close_on_select = item.close_on_select;
        let command = item.command;
        let disabled = item.disabled
            || crate::command_gating::command_is_disabled_by_gating(
                &*cx.app,
                &self.gating,
                command.as_ref(),
            );
        let leading = item.leading;
        let trailing = item.trailing;

        let open_for_item = self.open.clone();
        let cancel_open_for_item = self.cancel_open.clone();
        let ring = self.ring;
        let item_count = self.item_count;
        let reserve_leading_slot = self.reserve_leading_slot;
        let submenu_for_item = self.submenu_models.clone();
        let text_style = self.text_style.clone();
        let font_size = self.font_size;
        let font_line_height = self.font_line_height;
        let pad_x = self.pad_x;
        let pad_y = self.pad_y;
        let radius_sm = self.radius_sm;
        let text_disabled = self.text_disabled;
        let fg = self.fg;
        let accent = self.accent;
        let accent_fg = self.accent_fg;

        cx.keyed(value.clone(), move |cx| {
            cx.pressable_with_id_props(move |cx, st, item_id| {
                menu::sub_content::wire_item(cx, item_id, disabled, &submenu_for_item);
                cx.pressable_add_on_pointer_up(context_menu_cancel_open_item_pointer_up_handler(
                    cancel_open_for_item.clone(),
                    open_for_item.clone(),
                ));

                let checked_now = checked.snapshot(cx);
                if !disabled {
                    let checked_for_activate = checked.clone();
                    let on_checked_change_for_activate = on_checked_change.clone();
                    cx.pressable_on_activate(Arc::new(move |host, action_cx, _reason| {
                        let next = checked_for_activate.toggle(host);
                        if let Some(handler) = on_checked_change_for_activate.as_ref() {
                            handler(host, action_cx, next);
                        }
                    }));
                }
                cx.pressable_dispatch_command_if_enabled_opt(command.clone());
                if !disabled && close_on_select {
                    cx.pressable_set_bool(&open_for_item, false);
                }

                let trailing = trailing.or_else(|| {
                    command.as_ref().and_then(|cmd| {
                        command_shortcut_label(cx, cmd, fret_runtime::Platform::current())
                            .map(|text| ContextMenuShortcut::new(text).into_element(cx))
                    })
                });

                let props = PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.min_height = Some(Length::Px(Px(28.0)));
                        layout
                    },
                    enabled: !disabled,
                    focusable: !disabled,
                    focus_ring: Some(ring),
                    a11y: {
                        let mut a11y =
                            menu::item::menu_item_checkbox_a11y(a11y_label.clone(), checked_now);
                        a11y.test_id = test_id.clone();
                        a11y.with_collection_position(collection_index, item_count)
                    },
                    ..Default::default()
                };

                let mut row_bg = fret_core::Color::TRANSPARENT;
                let mut row_fg = fg;
                if st.hovered || st.pressed || st.focused {
                    row_bg = accent;
                    row_fg = accent_fg;
                }

                let children = menu_row_children(
                    cx,
                    label.clone(),
                    leading,
                    None,
                    reserve_leading_slot,
                    trailing,
                    false,
                    Some(checked_now),
                    disabled,
                    row_bg,
                    row_fg,
                    row_fg,
                    text_style.clone(),
                    font_size,
                    font_line_height,
                    pad_x,
                    pad_x,
                    pad_y,
                    radius_sm,
                    text_disabled,
                    chrome_test_id.clone(),
                );

                (props, children)
            })
        })
    }

    fn render_radio_item<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
        item: ContextMenuRadioItem,
        item_ix: &mut usize,
    ) -> AnyElement {
        let collection_index = *item_ix;
        *item_ix = (*item_ix).saturating_add(1);

        let label = item.label.clone();
        let value = item.value.clone();
        let group_value = item.group_value.clone();
        let on_value_change = item.on_value_change.clone();
        let a11y_label = item.a11y_label.clone().or_else(|| Some(label.clone()));
        let test_id = item.test_id.clone();
        let chrome_test_id = test_id
            .clone()
            .map(|id| Arc::<str>::from(format!("{id}.chrome")));
        let close_on_select = item.close_on_select;
        let command = item.command;
        let disabled = item.disabled
            || crate::command_gating::command_is_disabled_by_gating(
                &*cx.app,
                &self.gating,
                command.as_ref(),
            );
        let leading = item.leading;
        let trailing = item.trailing;

        let open_for_item = self.open.clone();
        let cancel_open_for_item = self.cancel_open.clone();
        let ring = self.ring;
        let item_count = self.item_count;
        let reserve_leading_slot = self.reserve_leading_slot;
        let submenu_for_item = self.submenu_models.clone();
        let text_style = self.text_style.clone();
        let font_size = self.font_size;
        let font_line_height = self.font_line_height;
        let pad_x = self.pad_x;
        let pad_y = self.pad_y;
        let radius_sm = self.radius_sm;
        let text_disabled = self.text_disabled;
        let fg = self.fg;
        let accent = self.accent;
        let accent_fg = self.accent_fg;

        cx.keyed(value.clone(), move |cx| {
            let selected = group_value.snapshot(cx);
            let is_selected = menu::radio_group::is_selected(selected.as_ref(), &value);

            cx.pressable_with_id_props(move |cx, st, item_id| {
                menu::sub_content::wire_item(cx, item_id, disabled, &submenu_for_item);
                cx.pressable_add_on_pointer_up(context_menu_cancel_open_item_pointer_up_handler(
                    cancel_open_for_item.clone(),
                    open_for_item.clone(),
                ));

                if !disabled {
                    let group_value_for_activate = group_value.clone();
                    let value_for_activate = value.clone();
                    let on_value_change_for_activate = on_value_change.clone();
                    cx.pressable_on_activate(Arc::new(move |host, action_cx, _reason| {
                        let Some(next) = group_value_for_activate.select(host, &value_for_activate)
                        else {
                            return;
                        };
                        if let Some(handler) = on_value_change_for_activate.as_ref() {
                            handler(host, action_cx, next);
                        }
                    }));
                }
                cx.pressable_dispatch_command_if_enabled_opt(command.clone());
                if !disabled && close_on_select {
                    cx.pressable_set_bool(&open_for_item, false);
                }

                let trailing = trailing.or_else(|| {
                    command.as_ref().and_then(|cmd| {
                        command_shortcut_label(cx, cmd, fret_runtime::Platform::current())
                            .map(|text| ContextMenuShortcut::new(text).into_element(cx))
                    })
                });

                let props = PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.min_height = Some(Length::Px(Px(28.0)));
                        layout
                    },
                    enabled: !disabled,
                    focusable: !disabled,
                    focus_ring: Some(ring),
                    a11y: {
                        let mut a11y =
                            menu::item::menu_item_radio_a11y(a11y_label.clone(), is_selected);
                        a11y.test_id = test_id.clone();
                        a11y.with_collection_position(collection_index, item_count)
                    },
                    ..Default::default()
                };

                let mut row_bg = fret_core::Color::TRANSPARENT;
                let mut row_fg = fg;
                if st.hovered || st.pressed || st.focused {
                    row_bg = accent;
                    row_fg = accent_fg;
                }

                let children = menu_row_children(
                    cx,
                    label.clone(),
                    leading,
                    None,
                    reserve_leading_slot,
                    trailing,
                    false,
                    Some(is_selected),
                    disabled,
                    row_bg,
                    row_fg,
                    row_fg,
                    text_style.clone(),
                    font_size,
                    font_line_height,
                    pad_x,
                    pad_x,
                    pad_y,
                    radius_sm,
                    text_disabled,
                    chrome_test_id.clone(),
                );

                (props, children)
            })
        })
    }
}

#[derive(Clone)]
struct ContextMenuContentRenderEnv {
    open: Model<bool>,
    cancel_open: ContextMenuCancelOpenShared,
    gating: WindowCommandGatingSnapshot,
    reserve_leading_slot: bool,
    item_count: usize,
    ring: RingStyle,
    border: fret_core::Color,
    radius_sm: Px,
    pad_x: Px,
    pad_x_inset: Px,
    pad_y: Px,
    font_size: Px,
    font_line_height: Px,
    text_style: TextStyle,
    text_disabled: fret_core::Color,
    label_fg: fret_core::Color,
    accent: fret_core::Color,
    accent_fg: fret_core::Color,
    fg: fret_core::Color,
    destructive_fg: fret_core::Color,
    destructive_bg: fret_core::Color,
    window_margin: Px,
    submenu_max_height_metric: Option<Px>,
    overlay_root_name_for_controls: Arc<str>,
    submenu_cfg: menu::sub::MenuSubmenuConfig,
    submenu_models: menu::sub::MenuSubmenuModels,
}

impl ContextMenuContentRenderEnv {
    fn render_entries<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
        entries: Vec<ContextMenuEntry>,
        item_ix: &mut usize,
    ) -> Elements {
        let mut out: Vec<AnyElement> = Vec::with_capacity(entries.len());

        for entry in entries {
            match entry {
                ContextMenuEntry::Group(group) => {
                    let children = self.render_entries(cx, group.entries, item_ix);
                    out.push(menu_structural_group(
                        cx,
                        fret_core::SemanticsRole::Group,
                        children.into_vec(),
                    ));
                }
                ContextMenuEntry::RadioGroup(group) => {
                    let mut children: Vec<AnyElement> = Vec::with_capacity(group.items.len());
                    for spec in group.items {
                        children.push(self.render_radio_item(
                            cx,
                            spec.into_item(group.value.clone(), group.on_value_change.clone()),
                            item_ix,
                        ));
                    }
                    out.push(menu_structural_group(
                        cx,
                        fret_core::SemanticsRole::Group,
                        children,
                    ));
                }
                ContextMenuEntry::Label(label) => out.push(self.render_label(cx, label)),
                ContextMenuEntry::Separator => out.push(self.render_separator(cx)),
                ContextMenuEntry::Item(item) => out.push(self.render_item(cx, item, item_ix)),
                ContextMenuEntry::CheckboxItem(item) => {
                    out.push(self.render_checkbox_item(cx, item, item_ix));
                }
                ContextMenuEntry::RadioItem(item) => {
                    out.push(self.render_radio_item(cx, item, item_ix))
                }
            }
        }

        out.into()
    }

    fn render_label<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
        label: ContextMenuLabel,
    ) -> AnyElement {
        let dir = crate::direction::use_direction(cx, None);
        let pad_left = if label.inset {
            self.pad_x_inset
        } else {
            self.pad_x
        };
        let text = label.text;
        let font_size = self.font_size;
        let font_line_height = self.font_line_height;
        let label_fg = self.label_fg;
        let pad_x = self.pad_x;
        let pad_y = self.pad_y;

        cx.container(
            ContainerProps {
                layout: LayoutStyle::default(),
                padding: rtl::padding_edges_with_inline_start_end(
                    dir, pad_y, pad_y, pad_left, pad_x,
                )
                .into(),
                ..Default::default()
            },
            move |cx| {
                vec![
                    ui::text(text)
                        .text_size_px(font_size)
                        .line_height_px(font_line_height)
                        .font_medium()
                        .nowrap()
                        .text_color(ColorRef::Color(label_fg))
                        .into_element(cx),
                ]
            },
        )
    }

    fn render_separator<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let border = self.border;
        cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Px(Px(1.0));
                    // new-york-v4: `Separator` uses `-mx-1 my-1`.
                    layout.margin.left = fret_ui::element::MarginEdge::Px(Px(-4.0));
                    layout.margin.right = fret_ui::element::MarginEdge::Px(Px(-4.0));
                    layout.margin.top = fret_ui::element::MarginEdge::Px(Px(4.0));
                    layout.margin.bottom = fret_ui::element::MarginEdge::Px(Px(4.0));
                    layout
                },
                padding: Edges::all(Px(0.0)).into(),
                background: Some(border),
                ..Default::default()
            },
            |_cx| Vec::new(),
        )
    }

    fn render_item<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
        item: ContextMenuItem,
        item_ix: &mut usize,
    ) -> AnyElement {
        let collection_index = *item_ix;
        *item_ix = (*item_ix).saturating_add(1);

        let label = item.label.clone();
        let value = item.value.clone();
        let a11y_label = item.a11y_label.clone().or_else(|| Some(label.clone()));
        let test_id = item.test_id.clone();
        let chrome_test_id = test_id
            .clone()
            .map(|id| Arc::<str>::from(format!("{id}.chrome")));
        let close_on_select = item.close_on_select;
        let command = item.command;
        let action_payload = item.action_payload;
        let disabled = item.disabled
            || crate::command_gating::command_is_disabled_by_gating(
                &*cx.app,
                &self.gating,
                command.as_ref(),
            );
        let leading = item.leading;
        let leading_icon = item.leading_icon;
        let trailing = item.trailing;
        let has_submenu = item.submenu.is_some();
        let submenu_row_count_for_hint = item.submenu.as_ref().map(|entries| {
            fn count_rows(entries: &[ContextMenuEntry]) -> usize {
                let mut count = 0usize;
                for entry in entries {
                    match entry {
                        ContextMenuEntry::Item(_)
                        | ContextMenuEntry::CheckboxItem(_)
                        | ContextMenuEntry::RadioItem(_)
                        | ContextMenuEntry::Label(_)
                        | ContextMenuEntry::Separator => count += 1,
                        ContextMenuEntry::RadioGroup(group) => count += group.items.len(),
                        ContextMenuEntry::Group(group) => count += count_rows(&group.entries),
                    }
                }
                count
            }
            count_rows(entries)
        });
        let variant = item.variant;
        let pad_left = if item.inset {
            self.pad_x_inset
        } else {
            self.pad_x
        };

        let open = self.open.clone();
        let cancel_open = self.cancel_open.clone();
        let ring = self.ring;
        let item_count = self.item_count;
        let reserve_leading_slot = self.reserve_leading_slot;
        let text_style = self.text_style.clone();
        let font_size = self.font_size;
        let font_line_height = self.font_line_height;
        let pad_x = self.pad_x;
        let pad_y = self.pad_y;
        let radius_sm = self.radius_sm;
        let text_disabled = self.text_disabled;
        let label_fg = self.label_fg;
        let fg = self.fg;
        let accent = self.accent;
        let accent_fg = self.accent_fg;
        let destructive_fg = self.destructive_fg;
        let destructive_bg = self.destructive_bg;
        let window_margin = self.window_margin;
        let submenu_max_height_metric = self.submenu_max_height_metric;
        let overlay_root_name_for_controls = self.overlay_root_name_for_controls.clone();
        let submenu_cfg = self.submenu_cfg;
        let submenu_for_item = self.submenu_models.clone();

        cx.keyed(value.clone(), move |cx| {
            cx.pressable_with_id_props(move |cx, st, item_id| {
                let geometry_hint = has_submenu.then(|| {
                    let outer = overlay::outer_bounds_with_window_margin_for_environment(
                        cx,
                        fret_ui::Invalidation::Layout,
                        window_margin,
                    );
                    let submenu_max_h = submenu_max_height_metric
                        .map(|h| Px(h.0.min(outer.size.height.0)))
                        .unwrap_or(outer.size.height);
                    let desired = menu::sub::estimated_desired_size_for_row_count(
                        Px(192.0),
                        Px(28.0),
                        submenu_row_count_for_hint.unwrap_or(1),
                        submenu_max_h,
                    );
                    menu::sub_trigger::MenuSubTriggerGeometryHint { outer, desired }
                });
                let is_open_submenu = menu::sub_trigger::wire(
                    cx,
                    st,
                    item_id,
                    disabled,
                    has_submenu,
                    value.clone(),
                    &submenu_for_item,
                    submenu_cfg,
                    geometry_hint,
                )
                .unwrap_or(false);

                cx.pressable_add_on_pointer_up(context_menu_cancel_open_item_pointer_up_handler(
                    cancel_open.clone(),
                    open.clone(),
                ));

                if !has_submenu && !disabled {
                    if let Some(payload) = action_payload.clone() {
                        cx.pressable_dispatch_command_with_payload_factory_if_enabled_opt(
                            command.clone(),
                            payload,
                        );
                    } else {
                        cx.pressable_dispatch_command_if_enabled_opt(command.clone());
                    }
                    if close_on_select {
                        cx.pressable_set_bool(&open, false);
                    }
                }

                let controls = has_submenu.then(|| {
                    menu::sub_content::submenu_content_semantics_id(
                        cx,
                        overlay_root_name_for_controls.as_ref(),
                        &value,
                    )
                });
                let mut a11y = menu::item::menu_item_a11y_with_controls(
                    a11y_label,
                    has_submenu.then_some(is_open_submenu),
                    controls,
                );
                a11y.test_id = test_id.clone();
                let props = PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.min_height = Some(Length::Px(Px(28.0)));
                        layout
                    },
                    enabled: !disabled,
                    focusable: !disabled,
                    focus_ring: Some(ring),
                    a11y: a11y.with_collection_position(collection_index, item_count),
                    ..Default::default()
                };

                let mut row_bg = fret_core::Color::TRANSPARENT;
                let mut row_fg = if variant == ContextMenuItemVariant::Destructive {
                    destructive_fg
                } else {
                    fg
                };
                if st.hovered || st.pressed || st.focused || is_open_submenu {
                    if variant == ContextMenuItemVariant::Destructive {
                        row_bg = destructive_bg;
                        row_fg = destructive_fg;
                    } else {
                        row_bg = accent;
                        row_fg = accent_fg;
                    }
                }
                let icon_fg = if variant == ContextMenuItemVariant::Destructive {
                    destructive_fg
                } else {
                    label_fg
                };

                let trailing = if has_submenu {
                    trailing
                } else {
                    trailing.or_else(|| {
                        command.as_ref().and_then(|cmd| {
                            command_shortcut_label(cx, cmd, fret_runtime::Platform::current())
                                .map(|text| ContextMenuShortcut::new(text).into_element(cx))
                        })
                    })
                };

                let children = menu_row_children(
                    cx,
                    label.clone(),
                    leading,
                    leading_icon,
                    reserve_leading_slot,
                    trailing,
                    has_submenu,
                    None,
                    disabled,
                    row_bg,
                    row_fg,
                    icon_fg,
                    text_style.clone(),
                    font_size,
                    font_line_height,
                    pad_left,
                    pad_x,
                    pad_y,
                    radius_sm,
                    text_disabled,
                    chrome_test_id.clone(),
                );

                (props, children)
            })
        })
    }

    fn render_checkbox_item<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
        item: ContextMenuCheckboxItem,
        item_ix: &mut usize,
    ) -> AnyElement {
        let collection_index = *item_ix;
        *item_ix = (*item_ix).saturating_add(1);

        let label = item.label.clone();
        let value = item.value.clone();
        let checked = item.checked.clone();
        let on_checked_change = item.on_checked_change.clone();
        let a11y_label = item.a11y_label.clone().or_else(|| Some(label.clone()));
        let test_id = item.test_id.clone();
        let chrome_test_id = test_id
            .clone()
            .map(|id| Arc::<str>::from(format!("{id}.chrome")));
        let close_on_select = item.close_on_select;
        let command = item.command;
        let disabled = item.disabled
            || crate::command_gating::command_is_disabled_by_gating(
                &*cx.app,
                &self.gating,
                command.as_ref(),
            );
        let leading = item.leading;
        let trailing = item.trailing;
        let open = self.open.clone();
        let cancel_open = self.cancel_open.clone();

        let ring = self.ring;
        let item_count = self.item_count;
        let reserve_leading_slot = self.reserve_leading_slot;
        let text_style = self.text_style.clone();
        let font_size = self.font_size;
        let font_line_height = self.font_line_height;
        let pad_x = self.pad_x;
        let pad_y = self.pad_y;
        let radius_sm = self.radius_sm;
        let text_disabled = self.text_disabled;
        let fg = self.fg;
        let accent = self.accent;
        let accent_fg = self.accent_fg;

        cx.keyed(value.clone(), move |cx| {
            cx.pressable_with_id_props(move |cx, st, _item_id| {
                let checked_now = checked.snapshot(cx);

                let props = PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.min_height = Some(Length::Px(Px(28.0)));
                        layout
                    },
                    enabled: !disabled,
                    focusable: !disabled,
                    focus_ring: Some(ring),
                    a11y: {
                        let mut a11y =
                            menu::item::menu_item_checkbox_a11y(a11y_label.clone(), checked_now);
                        a11y.test_id = test_id.clone();
                        a11y.with_collection_position(collection_index, item_count)
                    },
                    ..Default::default()
                };

                cx.pressable_add_on_pointer_up(context_menu_cancel_open_item_pointer_up_handler(
                    cancel_open.clone(),
                    open.clone(),
                ));

                if !disabled {
                    let checked_for_activate = checked.clone();
                    let on_checked_change_for_activate = on_checked_change.clone();
                    cx.pressable_on_activate(Arc::new(move |host, action_cx, _reason| {
                        let next = checked_for_activate.toggle(host);
                        if let Some(handler) = on_checked_change_for_activate.as_ref() {
                            handler(host, action_cx, next);
                        }
                    }));
                }
                cx.pressable_dispatch_command_if_enabled_opt(command.clone());
                if !disabled && close_on_select {
                    cx.pressable_set_bool(&open, false);
                }

                let trailing = trailing.or_else(|| {
                    command.as_ref().and_then(|cmd| {
                        command_shortcut_label(cx, cmd, fret_runtime::Platform::current())
                            .map(|text| ContextMenuShortcut::new(text).into_element(cx))
                    })
                });

                let mut row_bg = fret_core::Color::TRANSPARENT;
                let mut row_fg = fg;
                if st.hovered || st.pressed || st.focused {
                    row_bg = accent;
                    row_fg = accent_fg;
                }

                let children = menu_row_children(
                    cx,
                    label.clone(),
                    leading,
                    None,
                    reserve_leading_slot,
                    trailing,
                    false,
                    Some(checked_now),
                    disabled,
                    row_bg,
                    row_fg,
                    row_fg,
                    text_style.clone(),
                    font_size,
                    font_line_height,
                    pad_x,
                    pad_x,
                    pad_y,
                    radius_sm,
                    text_disabled,
                    chrome_test_id.clone(),
                );

                (props, children)
            })
        })
    }

    fn render_radio_item<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
        item: ContextMenuRadioItem,
        item_ix: &mut usize,
    ) -> AnyElement {
        let collection_index = *item_ix;
        *item_ix = (*item_ix).saturating_add(1);

        let label = item.label.clone();
        let value = item.value.clone();
        let group_value = item.group_value.clone();
        let on_value_change = item.on_value_change.clone();
        let a11y_label = item.a11y_label.clone().or_else(|| Some(label.clone()));
        let test_id = item.test_id.clone();
        let chrome_test_id = test_id
            .clone()
            .map(|id| Arc::<str>::from(format!("{id}.chrome")));
        let close_on_select = item.close_on_select;
        let command = item.command;
        let disabled = item.disabled
            || crate::command_gating::command_is_disabled_by_gating(
                &*cx.app,
                &self.gating,
                command.as_ref(),
            );
        let leading = item.leading;
        let trailing = item.trailing;
        let open = self.open.clone();
        let cancel_open = self.cancel_open.clone();

        let ring = self.ring;
        let item_count = self.item_count;
        let reserve_leading_slot = self.reserve_leading_slot;
        let text_style = self.text_style.clone();
        let font_size = self.font_size;
        let font_line_height = self.font_line_height;
        let pad_x = self.pad_x;
        let pad_y = self.pad_y;
        let radius_sm = self.radius_sm;
        let text_disabled = self.text_disabled;
        let fg = self.fg;
        let accent = self.accent;
        let accent_fg = self.accent_fg;

        cx.keyed(value.clone(), move |cx| {
            let selected = group_value.snapshot(cx);
            let is_selected = menu::radio_group::is_selected(selected.as_ref(), &value);

            cx.pressable_with_id_props(move |cx, st, _item_id| {
                let props = PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.min_height = Some(Length::Px(Px(28.0)));
                        layout
                    },
                    enabled: !disabled,
                    focusable: !disabled,
                    focus_ring: Some(ring),
                    a11y: {
                        let mut a11y =
                            menu::item::menu_item_radio_a11y(a11y_label.clone(), is_selected);
                        a11y.test_id = test_id.clone();
                        a11y.with_collection_position(collection_index, item_count)
                    },
                    ..Default::default()
                };

                cx.pressable_add_on_pointer_up(context_menu_cancel_open_item_pointer_up_handler(
                    cancel_open.clone(),
                    open.clone(),
                ));

                let selected = group_value.snapshot(cx);
                let is_selected = menu::radio_group::is_selected(selected.as_ref(), &value);

                if !disabled {
                    let group_value_for_activate = group_value.clone();
                    let value_for_activate = value.clone();
                    let on_value_change_for_activate = on_value_change.clone();
                    cx.pressable_on_activate(Arc::new(move |host, action_cx, _reason| {
                        let Some(next) = group_value_for_activate.select(host, &value_for_activate)
                        else {
                            return;
                        };
                        if let Some(handler) = on_value_change_for_activate.as_ref() {
                            handler(host, action_cx, next);
                        }
                    }));
                }
                cx.pressable_dispatch_command_if_enabled_opt(command.clone());
                if !disabled && close_on_select {
                    cx.pressable_set_bool(&open, false);
                }

                let trailing = trailing.or_else(|| {
                    command.as_ref().and_then(|cmd| {
                        command_shortcut_label(cx, cmd, fret_runtime::Platform::current())
                            .map(|text| ContextMenuShortcut::new(text).into_element(cx))
                    })
                });

                let mut row_bg = fret_core::Color::TRANSPARENT;
                let mut row_fg = fg;
                if st.hovered || st.pressed || st.focused {
                    row_bg = accent;
                    row_fg = accent_fg;
                }

                let children = menu_row_children(
                    cx,
                    label.clone(),
                    leading,
                    None,
                    reserve_leading_slot,
                    trailing,
                    false,
                    Some(is_selected),
                    disabled,
                    row_bg,
                    row_fg,
                    row_fg,
                    text_style.clone(),
                    font_size,
                    font_line_height,
                    pad_x,
                    pad_x,
                    pad_y,
                    radius_sm,
                    text_disabled,
                    chrome_test_id.clone(),
                );

                (props, children)
            })
        })
    }
}

fn menu_row_children<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    leading: Option<AnyElement>,
    leading_icon: Option<IconId>,
    reserve_leading_slot: bool,
    trailing: Option<AnyElement>,
    submenu: bool,
    indicator_on: Option<bool>,
    disabled: bool,
    row_bg: fret_core::Color,
    row_fg: fret_core::Color,
    row_icon_fg: fret_core::Color,
    text_style: TextStyle,
    _font_size: Px,
    _font_line_height: Px,
    pad_left: Px,
    pad_x: Px,
    pad_y: Px,
    radius_sm: Px,
    text_disabled: fret_core::Color,
    chrome_test_id: Option<Arc<str>>,
) -> Elements {
    let direction = crate::direction::use_direction(cx, None);
    let label_test_id = chrome_test_id
        .as_ref()
        .map(|id| Arc::<str>::from(format!("{id}-label")));
    let indicator_test_id = chrome_test_id
        .as_ref()
        .map(|id| Arc::<str>::from(format!("{id}-indicator")));
    let trailing_test_id = chrome_test_id
        .as_ref()
        .map(|id| Arc::<str>::from(format!("{id}-trailing")));
    let submenu_chevron_test_id = chrome_test_id
        .as_ref()
        .map(|id| Arc::<str>::from(format!("{id}-submenu-chevron")));
    let child = cx.container(
        ContainerProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Fill;
                layout
            },
            padding: rtl::padding_edges_with_inline_start_end(
                direction, pad_y, pad_y, pad_left, pad_x,
            )
            .into(),
            background: Some(row_bg),
            corner_radii: fret_core::Corners::all(radius_sm),
            ..Default::default()
        },
        move |cx| {
            let text_fg = if disabled { text_disabled } else { row_fg };
            let icon_fg = if disabled {
                alpha_mul(row_icon_fg, 0.5)
            } else {
                row_icon_fg
            };
            let mut leading = leading;
            let leading_icon = leading_icon.clone();
            let mut trailing = trailing;
            let has_trailing = trailing.is_some();
            let has_indicator = indicator_on.is_some();
            let has_leading_slot =
                leading.is_some() || leading_icon.is_some() || reserve_leading_slot;
            let mut row: Vec<AnyElement> = Vec::with_capacity(
                usize::from(has_indicator)
                    + usize::from(has_leading_slot)
                    + 1
                    + usize::from(has_trailing)
                    + usize::from(has_trailing || submenu)
                    + usize::from(submenu),
            );

            if let Some(is_on) = indicator_on {
                let indicator_fg = text_fg;
                let mut indicator = cx.flex(
                    FlexProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(16.0));
                            layout.size.height = Length::Px(Px(16.0));
                            layout.flex.shrink = 0.0;
                            layout
                        },
                        direction: fret_core::Axis::Horizontal,
                        gap: Px(0.0).into(),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Center,
                        align: CrossAlign::Center,
                        wrap: false,
                    },
                    move |cx| {
                        if !is_on {
                            return Vec::new();
                        }

                        vec![decl_icon::icon_with(
                            cx,
                            ids::ui::CHECK,
                            Some(Px(16.0)),
                            Some(ColorRef::Color(indicator_fg)),
                        )]
                    },
                );
                if let Some(test_id) = indicator_test_id.clone() {
                    indicator = indicator.test_id(test_id);
                }
                row.push(indicator);
            }

            if let Some(l) = leading.take() {
                let scoped = l.inherit_foreground(icon_fg);
                row.push(menu_icon_slot(cx, scoped));
            } else if let Some(icon) = leading_icon.clone() {
                let icon_el = decl_icon::icon_with(cx, icon, Some(Px(16.0)), None);
                let scoped = icon_el.inherit_foreground(icon_fg);
                row.push(menu_icon_slot(cx, scoped));
            } else if reserve_leading_slot {
                row.push(menu_icon_slot_empty(cx));
            }

            let style = text_style.clone();
            let mut text = ui::text(label.clone())
                .layout(LayoutRefinement::default().min_w_0().flex_1())
                .text_size_px(style.size)
                .font_weight(style.weight)
                .text_color(ColorRef::Color(text_fg))
                .nowrap();

            if let Some(line_height) = style.line_height {
                text = text.fixed_line_box_px(line_height).line_box_in_bounds();
            }

            if let Some(letter_spacing_em) = style.letter_spacing_em {
                text = text.letter_spacing_em(letter_spacing_em);
            }

            let mut label_element = text.into_element(cx);
            if let Some(test_id) = label_test_id.clone() {
                label_element = label_element.test_id(test_id);
            }
            row.push(label_element);

            if has_trailing || submenu {
                row.push(cx.spacer(SpacerProps::default()));
            }
            if let Some(mut trailing_element) = trailing.take() {
                if let Some(test_id) = trailing_test_id.clone() {
                    trailing_element = trailing_element.test_id(test_id);
                }
                row.push(trailing_element);
            }

            if submenu {
                let mut chevron = submenu_chevron_icon(cx, direction, icon_fg);
                if let Some(test_id) = submenu_chevron_test_id.clone() {
                    chevron = chevron.test_id(test_id);
                }
                row.push(chevron);
            }

            vec![cx.flex(
                FlexProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout
                    },
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(8.0).into(),
                    padding: Edges::all(Px(0.0)).into(),
                    justify: MainAlign::Start,
                    align: CrossAlign::Center,
                    wrap: false,
                },
                move |_cx| row,
            )]
        },
    );

    let mut chrome = child;
    if let Some(test_id) = chrome_test_id {
        chrome = chrome.test_id(test_id);
    }

    vec![chrome].into()
}

fn submenu_chevron_icon<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    direction: LayoutDirection,
    fg: fret_core::Color,
) -> AnyElement {
    let icon = rtl::chevron_inline_end(direction);
    cx.flex(
        FlexProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Px(Px(16.0));
                layout.size.height = Length::Px(Px(16.0));
                layout.flex.shrink = 0.0;
                layout
            },
            direction: fret_core::Axis::Horizontal,
            gap: Px(0.0).into(),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Center,
            align: CrossAlign::Center,
            wrap: false,
        },
        move |cx| {
            vec![decl_icon::icon_with(
                cx,
                icon,
                Some(Px(16.0)),
                Some(ColorRef::Color(fg)),
            )]
        },
    )
}

fn menu_icon_slot<H: UiHost, B>(cx: &mut ElementContext<'_, H>, element: B) -> AnyElement
where
    B: IntoUiElement<H>,
{
    cx.flex(
        FlexProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Px(Px(16.0));
                layout.size.height = Length::Px(Px(16.0));
                layout.flex.shrink = 0.0;
                layout
            },
            direction: fret_core::Axis::Horizontal,
            gap: Px(0.0).into(),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Center,
            align: CrossAlign::Center,
            wrap: false,
        },
        move |cx| vec![element.into_element(cx)],
    )
}

fn menu_icon_slot_empty<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    cx.flex(
        FlexProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Px(Px(16.0));
                layout.size.height = Length::Px(Px(16.0));
                layout.flex.shrink = 0.0;
                layout
            },
            direction: fret_core::Axis::Horizontal,
            gap: Px(0.0).into(),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Center,
            align: CrossAlign::Center,
            wrap: false,
        },
        |_cx| Vec::new(),
    )
}

fn context_menu_submenu_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open_value: Arc<str>,
    placed: Rect,
    entries: Vec<ContextMenuEntry>,
    test_id_prefix: Option<Arc<str>>,
    open: Model<bool>,
    typeahead_timeout_ticks: u64,
    align_leading_icons: bool,
    submenu_models: menu::sub::MenuSubmenuModels,
    cancel_open: ContextMenuCancelOpenShared,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).snapshot();
    let gating = crate::command_gating::snapshot_for_window(&*cx.app, cx.window);

    let entries_tree = entries;

    fn reserve_leading_slot(entries: &[ContextMenuEntry]) -> bool {
        for entry in entries {
            match entry {
                ContextMenuEntry::Item(item) => {
                    if item.leading.is_some() || item.leading_icon.is_some() {
                        return true;
                    }
                }
                ContextMenuEntry::CheckboxItem(item) => {
                    if item.leading.is_some() {
                        return true;
                    }
                }
                ContextMenuEntry::RadioItem(item) => {
                    if item.leading.is_some() {
                        return true;
                    }
                }
                ContextMenuEntry::RadioGroup(group) => {
                    if group.items.iter().any(|item| item.leading.is_some()) {
                        return true;
                    }
                }
                ContextMenuEntry::Group(group) => {
                    if reserve_leading_slot(&group.entries) {
                        return true;
                    }
                }
                ContextMenuEntry::Label(_) | ContextMenuEntry::Separator => {}
            }
        }
        false
    }

    fn item_count(entries: &[ContextMenuEntry]) -> usize {
        let mut count = 0usize;
        for entry in entries {
            match entry {
                ContextMenuEntry::Item(_)
                | ContextMenuEntry::CheckboxItem(_)
                | ContextMenuEntry::RadioItem(_) => count += 1,
                ContextMenuEntry::RadioGroup(group) => count += group.items.len(),
                ContextMenuEntry::Group(group) => count += item_count(&group.entries),
                ContextMenuEntry::Label(_) | ContextMenuEntry::Separator => {}
            }
        }
        count
    }

    fn push_leaf_label_and_disabled<H: UiHost>(
        cx: &ElementContext<'_, H>,
        gating: &WindowCommandGatingSnapshot,
        label: &Arc<str>,
        disabled: bool,
        command: Option<&CommandId>,
        labels: &mut Vec<Arc<str>>,
        disabled_flags: &mut Vec<bool>,
    ) {
        labels.push(label.clone());
        disabled_flags.push(
            disabled
                || crate::command_gating::command_is_disabled_by_gating(&*cx.app, gating, command),
        );
    }

    fn collect_labels_and_disabled<H: UiHost>(
        cx: &ElementContext<'_, H>,
        gating: &WindowCommandGatingSnapshot,
        entries: &[ContextMenuEntry],
        labels: &mut Vec<Arc<str>>,
        disabled_flags: &mut Vec<bool>,
    ) {
        for entry in entries {
            match entry {
                ContextMenuEntry::Item(item) => push_leaf_label_and_disabled(
                    cx,
                    gating,
                    &item.label,
                    item.disabled,
                    item.command.as_ref(),
                    labels,
                    disabled_flags,
                ),
                ContextMenuEntry::CheckboxItem(item) => push_leaf_label_and_disabled(
                    cx,
                    gating,
                    &item.label,
                    item.disabled,
                    item.command.as_ref(),
                    labels,
                    disabled_flags,
                ),
                ContextMenuEntry::RadioItem(item) => push_leaf_label_and_disabled(
                    cx,
                    gating,
                    &item.label,
                    item.disabled,
                    item.command.as_ref(),
                    labels,
                    disabled_flags,
                ),
                ContextMenuEntry::RadioGroup(group) => {
                    for spec in group.items.iter() {
                        push_leaf_label_and_disabled(
                            cx,
                            gating,
                            &spec.label,
                            spec.disabled,
                            spec.command.as_ref(),
                            labels,
                            disabled_flags,
                        );
                    }
                }
                ContextMenuEntry::Group(group) => {
                    collect_labels_and_disabled(cx, gating, &group.entries, labels, disabled_flags);
                }
                ContextMenuEntry::Label(_) | ContextMenuEntry::Separator => {}
            }
        }
    }

    let reserve_leading_slot = align_leading_icons && reserve_leading_slot(entries_tree.as_slice());
    let item_count = item_count(entries_tree.as_slice());

    let mut labels: Vec<Arc<str>> = Vec::new();
    let mut disabled_flags: Vec<bool> = Vec::new();
    collect_labels_and_disabled(
        cx,
        &gating,
        entries_tree.as_slice(),
        &mut labels,
        &mut disabled_flags,
    );

    let labels_arc: Arc<[Arc<str>]> = Arc::from(labels.into_boxed_slice());
    let disabled_arc: Arc<[bool]> = Arc::from(disabled_flags.into_boxed_slice());

    let border = theme.color_token("border");
    let radius_sm = MetricRef::radius(Radius::Sm).resolve(&theme);
    let panel_chrome = crate::ui_builder_ext::surfaces::menu_sub_style_chrome().rounded(Radius::Md);
    let ring = decl_style::focus_ring(&theme, radius_sm);
    let pad_x = MetricRef::space(Space::N2).resolve(&theme);
    let pad_x_inset = MetricRef::space(Space::N8).resolve(&theme);
    let pad_y = MetricRef::space(Space::N1p5).resolve(&theme);
    let font_size = theme.metric_token("font.size");
    let font_line_height = theme.metric_token("font.line_height");
    let mut text_style =
        typography::fixed_line_box_style(fret_core::FontId::ui(), font_size, font_line_height);
    text_style.weight = fret_core::FontWeight::NORMAL;
    let text_disabled = alpha_mul(theme.color_token("foreground"), 0.5);
    let label_fg = theme.color_token("muted-foreground");
    let accent = theme.color_token("accent");
    let accent_fg = theme.color_token("accent-foreground");
    let fg = theme.color_token("foreground");
    let destructive_fg = theme.color_token("destructive");
    let destructive_bg = menu_destructive_focus_bg(&theme, destructive_fg);

    let labelled_by_element = cx
        .app
        .models_mut()
        .read(&submenu_models.trigger, |v| *v)
        .ok()
        .flatten();

    menu::sub_content::submenu_panel_scroll_y_for_value_at(
        cx,
        open_value,
        placed,
        labelled_by_element,
        move |layout| {
            let mut props = decl_style::container_props(
                &theme,
                panel_chrome.clone(),
                LayoutRefinement::default(),
            );
            props.layout = layout;
            props
        },
        move |cx| {
            let render_env = ContextMenuRenderEnv {
                open: open.clone(),
                cancel_open: cancel_open.clone(),
                gating: gating.clone(),
                test_id_prefix,
                reserve_leading_slot,
                item_count,
                ring,
                border,
                radius_sm,
                pad_x,
                pad_x_inset,
                pad_y,
                font_size,
                font_line_height,
                text_style: text_style.clone(),
                text_disabled,
                label_fg,
                accent,
                accent_fg,
                fg,
                destructive_fg,
                destructive_bg,
                submenu_models: submenu_models.clone(),
            };
            let mut item_ix: usize = 0;
            let out = render_env.render_entries(cx, entries_tree, &mut item_ix);

            vec![
                menu::sub_content::submenu_roving_group_apg_prefix_typeahead(
                    cx,
                    RovingFlexProps {
                        flex: FlexProps {
                            layout: LayoutStyle::default(),
                            direction: fret_core::Axis::Vertical,
                            gap: Px(0.0).into(),
                            padding: Edges::all(Px(0.0)).into(),
                            justify: MainAlign::Start,
                            align: CrossAlign::Stretch,
                            wrap: false,
                        },
                        roving: RovingFocusProps {
                            enabled: true,
                            wrap: false,
                            disabled: disabled_arc.clone(),
                            ..Default::default()
                        },
                    },
                    labels_arc.clone(),
                    typeahead_timeout_ticks,
                    submenu_models.clone(),
                    move |_cx| out,
                ),
            ]
        },
    )
}

/// shadcn/ui `ContextMenu` root (v4).
///
/// This is a dismissible popover opened by a component-owned pointer policy:
/// - right click
/// - (macOS) ctrl + left click
///
/// Notes:
/// - Position is anchored at the last pointer-down location observed within the trigger region.
/// - Keyboard invocation via Shift+F10 and `ContextMenu` key is supported.
#[derive(Clone)]
pub struct ContextMenu {
    open: Model<bool>,
    disabled: bool,
    test_id_prefix: Option<Arc<str>>,
    modal: bool,
    align: DropdownMenuAlign,
    side: DropdownMenuSide,
    side_offset: Px,
    window_margin: Px,
    min_width: Px,
    submenu_min_width: Px,
    typeahead_timeout_ticks: u64,
    arrow: bool,
    arrow_size_override: Option<Px>,
    arrow_padding_override: Option<Px>,
    align_leading_icons: bool,
    on_dismiss_request: Option<OnDismissRequest>,
    on_open_auto_focus: Option<OnOpenAutoFocus>,
    on_close_auto_focus: Option<OnCloseAutoFocus>,
    on_open_change: Option<OnOpenChange>,
    on_open_change_complete: Option<OnOpenChange>,
    content_test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for ContextMenu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContextMenu")
            .field("open", &"<model>")
            .field("disabled", &self.disabled)
            .field("align", &self.align)
            .field("side", &self.side)
            .field("side_offset", &self.side_offset)
            .field("window_margin", &self.window_margin)
            .field("typeahead_timeout_ticks", &self.typeahead_timeout_ticks)
            .field("on_dismiss_request", &self.on_dismiss_request.is_some())
            .field("on_open_auto_focus", &self.on_open_auto_focus.is_some())
            .field("on_close_auto_focus", &self.on_close_auto_focus.is_some())
            .field("on_open_change", &self.on_open_change.is_some())
            .field(
                "on_open_change_complete",
                &self.on_open_change_complete.is_some(),
            )
            .field("content_test_id", &self.content_test_id)
            .finish()
    }
}

impl ContextMenu {
    /// Explicit advanced seam for authoring against an already-managed open model.
    pub fn from_open(open: Model<bool>) -> Self {
        Self {
            open,
            disabled: false,
            test_id_prefix: None,
            modal: true,
            align: DropdownMenuAlign::Start,
            // Match Radix/shadcn defaults:
            // `ContextMenuPrimitive.Content` uses `side="right" sideOffset={2} align="start"`.
            side: DropdownMenuSide::Right,
            side_offset: Px(2.0),
            window_margin: Px(0.0),
            min_width: Px(128.0),
            submenu_min_width: Px(128.0),
            typeahead_timeout_ticks: 30,
            arrow: false,
            arrow_size_override: None,
            arrow_padding_override: None,
            align_leading_icons: true,
            on_dismiss_request: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_open_change: None,
            on_open_change_complete: None,
            content_test_id: None,
        }
    }

    /// Default typed root constructor for the common uncontrolled context-menu authoring path.
    ///
    /// This stores the internal `open` model at the root call site and starts closed.
    pub fn uncontrolled<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Self {
        Self::new_controllable(cx, None, false)
    }

    /// Creates a context menu with a controlled/uncontrolled open model (Radix `open` / `defaultOpen`).
    ///
    /// Note: If `open` is `None`, the internal model is stored in element state at the call site.
    /// Call this from a stable subtree (key the parent node if needed).
    pub fn new_controllable<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        open: Option<Model<bool>>,
        default_open: bool,
    ) -> Self {
        let open =
            fret_ui_kit::primitives::open_state::open_use_model(cx, open, || default_open).model();
        Self::from_open(open)
    }

    pub fn align(mut self, align: DropdownMenuAlign) -> Self {
        self.align = align;
        self
    }

    /// Whether trigger gestures/shortcuts should be ignored.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Controls whether outside-press dismissal should be click-through (Radix `modal={false}`).
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }

    pub fn side(mut self, side: DropdownMenuSide) -> Self {
        self.side = side;
        self
    }

    pub fn side_offset(mut self, offset: Px) -> Self {
        self.side_offset = offset;
        self
    }

    pub fn window_margin(mut self, margin: Px) -> Self {
        self.window_margin = margin;
        self
    }

    pub fn min_width(mut self, min_width: Px) -> Self {
        self.min_width = min_width;
        self
    }

    /// Optional debug/test-only identifier for the menu content semantics node (`role=menu`).
    pub fn content_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.content_test_id = Some(id.into());
        self
    }

    pub fn test_id_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        let prefix = prefix.into();
        self.content_test_id = Some(Arc::<str>::from(format!("{prefix}-content")));
        self.test_id_prefix = Some(prefix);
        self
    }

    pub fn submenu_min_width(mut self, min_width: Px) -> Self {
        self.submenu_min_width = min_width;
        self
    }

    pub fn typeahead_timeout_ticks(mut self, ticks: u64) -> Self {
        self.typeahead_timeout_ticks = ticks;
        self
    }

    pub fn align_leading_icons(mut self, align: bool) -> Self {
        self.align_leading_icons = align;
        self
    }

    /// Sets an optional open autofocus handler (Radix `onOpenAutoFocus`).
    pub fn on_open_auto_focus(mut self, hook: Option<OnOpenAutoFocus>) -> Self {
        self.on_open_auto_focus = hook;
        self
    }

    /// Sets an optional close autofocus handler (Radix `onCloseAutoFocus`).
    pub fn on_close_auto_focus(mut self, hook: Option<OnCloseAutoFocus>) -> Self {
        self.on_close_auto_focus = hook;
        self
    }

    /// Enables a ContextMenu arrow (Radix `ContextMenuArrow`-style).
    pub fn arrow(mut self, arrow: bool) -> Self {
        self.arrow = arrow;
        self
    }

    pub fn arrow_size(mut self, size: Px) -> Self {
        self.arrow_size_override = Some(size);
        self
    }

    pub fn arrow_padding(mut self, padding: Px) -> Self {
        self.arrow_padding_override = Some(padding);
        self
    }

    /// Sets an optional dismiss request handler (Radix `DismissableLayer`).
    ///
    /// When set, Escape/outside-press dismissals route through this handler. To prevent default
    /// dismissal, call `req.prevent_default()`.
    pub fn on_dismiss_request(mut self, on_dismiss_request: Option<OnDismissRequest>) -> Self {
        self.on_dismiss_request = on_dismiss_request;
        self
    }

    /// Called when the open state changes (Base UI `onOpenChange`).
    pub fn on_open_change(mut self, on_open_change: Option<OnOpenChange>) -> Self {
        self.on_open_change = on_open_change;
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

    /// Returns a recipe-level composition builder for shadcn-style part assembly.
    ///
    /// This keeps the root authoring surface on a typed builder lane while still lowering into the
    /// existing `build_parts(...)` path at the final landing seam.
    pub fn compose<H: UiHost>(self) -> ContextMenuComposition<H> {
        ContextMenuComposition::new(self)
    }

    /// Host-bound builder-first helper that late-lands the trigger at the root call site.
    #[track_caller]
    pub fn build<H: UiHost, I>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl IntoUiElement<H>,
        entries: impl FnOnce(&mut ElementContext<'_, H>) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = ContextMenuEntry>,
    {
        self.into_element(cx, move |cx| trigger.into_element(cx), entries)
    }

    /// Part-based authoring surface aligned with shadcn/ui v4 exports.
    ///
    /// This is a thin adapter over `ContextMenu::into_element(...)` that allows call sites to use
    /// `ContextMenuTrigger` and `ContextMenuContent` parts (and to attach content placement
    /// options in a shadcn-like location).
    #[track_caller]
    pub fn build_parts<H: UiHost, I>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl IntoUiElement<H>,
        content: impl Into<ContextMenuContent>,
        entries: impl FnOnce(&mut ElementContext<'_, H>) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = ContextMenuEntry>,
    {
        let menu = content.into().apply_to(self);
        menu.build(cx, trigger, entries)
    }

    /// Part-based authoring surface aligned with shadcn/ui v4 exports.
    ///
    /// This is a thin adapter over `ContextMenu::into_element(...)` that allows call sites to use
    /// `ContextMenuTrigger` and `ContextMenuContent` parts (and to attach content placement
    /// options in a shadcn-like location).
    #[track_caller]
    pub fn into_element_parts<H: UiHost, I>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> ContextMenuTrigger,
        content: impl Into<ContextMenuContent>,
        entries: impl FnOnce(&mut ElementContext<'_, H>) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = ContextMenuEntry>,
    {
        let trigger = trigger(cx);
        self.build_parts(cx, trigger, content, entries)
    }

    #[track_caller]
    pub fn into_element<H: UiHost, I>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        entries: impl FnOnce(&mut ElementContext<'_, H>) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = ContextMenuEntry>,
    {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).snapshot();
            let submenu_max_height_metric = theme.metric_by_key("component.context_menu.max_height");
            let is_open = cx
                .watch_model(&self.open)
                .paint()
                .copied()
                .unwrap_or(false);
            let motion = radix_presence::scale_fade_presence_with_durations_and_cubic_bezier_duration(
                cx,
                is_open,
                overlay_motion::shadcn_motion_duration_150(cx),
                overlay_motion::shadcn_motion_duration_150(cx),
                0.95,
                1.0,
                overlay_motion::shadcn_motion_ease_bezier(cx),
            );
            let (open_change, open_change_complete) =
                cx.slot_state(ContextMenuOpenChangeCallbackState::default, |state| {
                    context_menu_open_change_events(state, is_open, motion.present, motion.animating)
                });
            if let (Some(open), Some(on_open_change)) =
                (open_change, self.on_open_change.as_ref())
            {
                on_open_change(open);
            }
            if let (Some(open), Some(on_open_change_complete)) =
                (open_change_complete, self.on_open_change_complete.as_ref())
            {
                on_open_change_complete(open);
            }
            let overlay_presence = OverlayPresence {
                present: motion.present,
                interactive: is_open,
            };
            let opacity = motion.opacity;
            let scale = motion.scale;
            let opening = is_open;
            let arrow = self.arrow;
            let arrow_size = self.arrow_size_override.unwrap_or_else(|| {
                theme
                    .metric_by_key("component.context_menu.arrow_size")
                    .or_else(|| theme.metric_by_key("component.popover.arrow_size"))
                    .unwrap_or(Px(12.0))
            });
            let arrow_padding = self.arrow_padding_override.unwrap_or_else(|| {
                theme
                    .metric_by_key("component.context_menu.arrow_padding")
                    .or_else(|| theme.metric_by_key("component.popover.arrow_padding"))
                    .unwrap_or_else(|| MetricRef::radius(Radius::Md).resolve(&theme))
            });

            let id = cx.root_id();
            let overlay_root_name = menu::context_menu_root_name(id);
            let overlay_root_name_for_controls: Arc<str> = Arc::from(overlay_root_name.clone());
            let content_id_for_trigger =
                menu::content_panel::menu_content_semantics_id(cx, &overlay_root_name);
            let disabled = self.disabled;
            let trigger_element = trigger(cx);
            let trigger_element = menu::trigger::apply_menu_trigger_a11y(
                trigger_element,
                is_open,
                Some(content_id_for_trigger),
            );
            let trigger_id = trigger_element.id;

            if !disabled {
                menu::trigger::wire_open_on_shift_f10(cx, trigger_id, self.open.clone());
            }

            let open = self.open;
            let on_dismiss_request = self.on_dismiss_request.clone();
            let on_open_auto_focus = self.on_open_auto_focus.clone();
            let on_close_auto_focus = self.on_close_auto_focus.clone();
            let open_model_id = open.id();
            let cancel_open: ContextMenuCancelOpenShared =
                cx.slot_state(context_menu_cancel_open_shared, |shared| shared.clone());
            let anchor_store_model: Model<HashMap<ModelId, Point>> =
                menu::context_menu_anchor_store_model(cx.app);

            let base_pointer_policy = menu::context_menu_pointer_down_policy(open.clone());
            // Keep pending touch long-press state tied to the menu instance rather than the
            // current render root so timer dispatch survives owner churn.
            let touch_long_press: menu::ContextMenuTouchLongPress =
                menu::context_menu_touch_long_press_for_open_model(cx.app, &open);
            let pointer_policy = Arc::new({
                let anchor_store_model = anchor_store_model.clone();
                let touch_long_press = touch_long_press.clone();
                let cancel_open = cancel_open.clone();
                move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                      cx: fret_ui::action::ActionCx,
                      down: fret_ui::action::PointerDownCx| {
                    if disabled {
                        return false;
                    }
                    let touch_long_press_handled = menu::context_menu_touch_long_press_on_pointer_down(
                        &touch_long_press,
                        host,
                        cx,
                        down,
                    );
                    let handled = base_pointer_policy(host, cx, down);
                    if handled {
                        menu::context_menu_touch_long_press_clear(&touch_long_press, host);
                        let anchor_position = down.position_window.unwrap_or(down.position);
                        let _ = host.models_mut().update(&anchor_store_model, |map| {
                            map.insert(open_model_id, anchor_position);
                        });
                        // Base UI: once opened, wait 500ms before allowing `mouseup` to cancel-open.
                        // (`LONG_PRESS_DELAY = 500` in `ContextMenuTrigger.tsx`.)
                        if down.button == fret_core::MouseButton::Right {
                            context_menu_cancel_open_start(
                                &cancel_open,
                                host,
                                cx.window,
                                down.pointer_id,
                                down.position,
                            );
                            // Capture pointer so we keep receiving move/up during the guard window.
                            host.capture_pointer();
                        }
                    }
                    touch_long_press_handled || handled
                }
            });

            let pointer_policy_for_region = pointer_policy.clone();
            let anchor_store_model_for_region = anchor_store_model.clone();
            let open_for_region = open.clone();
            let cancel_open_for_region = cancel_open.clone();
            let trigger = cx.keyed((open_model_id, "context-menu-trigger-region"), move |cx| {
                let pointer_policy_for_region = pointer_policy_for_region.clone();
                let cancel_open_for_region = cancel_open_for_region.clone();
                let (touch_on_move, touch_on_up, touch_on_cancel) =
                    menu::context_menu_touch_long_press_pointer_handlers(touch_long_press.clone());
                let touch_long_press_for_timer = touch_long_press.clone();
                let anchor_store_model_for_timer = anchor_store_model_for_region.clone();
                let open_for_timer = open_for_region.clone();
                cx.pointer_region(PointerRegionProps::default(), move |cx| {
                    cx.pointer_region_on_pointer_down(pointer_policy_for_region);
                    let cancel_open_for_move = cancel_open_for_region.clone();
                    cx.pointer_region_on_pointer_move(Arc::new(move |host, acx, mv| {
                        context_menu_cancel_open_mark_moved_if_needed(
                            &cancel_open_for_move,
                            mv.pointer_id,
                            mv.position,
                        );
                        touch_on_move(host, acx, mv)
                    }));
                    let cancel_open_for_up = cancel_open_for_region.clone();
                    let cancel_open_for_up_release = cancel_open_for_region.clone();
                    let open_for_up = open_for_region.clone();
                    cx.pointer_region_on_pointer_up(Arc::new(move |host, acx, up| {
                        context_menu_cancel_open_mark_moved_if_needed(
                            &cancel_open_for_up_release,
                            up.pointer_id,
                            up.position,
                        );
                        context_menu_cancel_open_on_pointer_up(
                            &cancel_open_for_up,
                            host,
                            &open_for_up,
                            up.pointer_id,
                            up.position,
                            up.button,
                        );
                        if up.button == fret_core::MouseButton::Right
                            || up.pointer_type == fret_core::PointerType::Touch
                        {
                            host.release_pointer_capture();
                        }
                        touch_on_up(host, acx, up)
                    }));
                    let cancel_open_for_cancel = cancel_open_for_region.clone();
                    cx.pointer_region_on_pointer_cancel(Arc::new(move |host, acx, cancel| {
                        context_menu_cancel_open_stop_without_close(
                            &cancel_open_for_cancel,
                            host,
                            cancel.pointer_id,
                        );
                        host.release_pointer_capture();
                        touch_on_cancel(host, acx, cancel)
                    }));
                    let region_id = cx.root_id();
                    let cancel_open_for_timer = cancel_open_for_region.clone();
                    cx.timer_on_timer_for(
                        region_id,
                        Arc::new(move |host, action_cx, token| {
                            context_menu_cancel_open_on_timer(&cancel_open_for_timer, host, token);
                            let Some(anchor) = menu::context_menu_touch_long_press_take_anchor_on_timer(
                                &touch_long_press_for_timer,
                                token,
                            ) else {
                                return false;
                            };

                            let _ = host.models_mut().update(&anchor_store_model_for_timer, |map| {
                                map.insert(open_model_id, anchor);
                            });
                            let _ = host.models_mut().update(&open_for_timer, |v| *v = true);
                            host.request_redraw(action_cx.window);
                            true
                        }),
                    );
                    vec![trigger_element]
                })
            });

            let anchor_point = cx
                .watch_model(&anchor_store_model)
                .read_ref(|m| m.get(&open_model_id).copied())
                .ok()
                .flatten();
            let portal_ctx = portal_inherited::PortalInherited::capture(cx);
            let submenu_cfg = menu::sub::MenuSubmenuConfig::default();
            let submenu = portal_inherited::with_root_name_inheriting(
                cx,
                &overlay_root_name,
                portal_ctx,
                |cx| {
                menu::root::sync_root_open_and_ensure_submenu(cx, is_open, cx.root_id(), submenu_cfg)
                },
            );

            if overlay_presence.present {
                let align = self.align;
                let side = self.side;
                let side_offset = self.side_offset;
                let window_margin = self.window_margin;
                let min_width = self.min_width;
                let submenu_min_width = self.submenu_min_width;
                let typeahead_timeout_ticks = self.typeahead_timeout_ticks;
                let align_leading_icons = self.align_leading_icons;
                let modal = self.modal;
                let content_test_id = self.content_test_id.clone();
                let overlay_root_name_for_trace = overlay_root_name_for_controls.clone();
                let open_for_overlay = open.clone();
                let content_focus_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
                let content_focus_id_for_children = content_focus_id.clone();
                let first_item_focus_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
                let first_item_focus_id_for_children = first_item_focus_id.clone();
                let last_item_focus_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
                let last_item_focus_id_for_children = last_item_focus_id.clone();
                let first_item_focus_id_model = cx.local_model_keyed(
                    ("context-menu-first-item-focus-id", open.id()),
                    || None::<GlobalElementId>,
                );
                let last_item_focus_id_model = cx.local_model_keyed(
                    ("context-menu-last-item-focus-id", open.id()),
                    || None::<GlobalElementId>,
                );
                let _ = cx
                    .app
                    .models_mut()
                    .update(&first_item_focus_id_model, |v| *v = None);
                let _ = cx
                    .app
                    .models_mut()
                    .update(&last_item_focus_id_model, |v| *v = None);
                let first_item_focus_id_model_for_overlay = first_item_focus_id_model.clone();
                let last_item_focus_id_model_for_overlay = last_item_focus_id_model.clone();
                let direction = portal_ctx.direction;

                let (overlay_children, dismissible_on_pointer_move) =
                    portal_inherited::with_root_name_inheriting(
                    cx,
                    &overlay_root_name,
                    portal_ctx,
                    move |cx| {
                    let trigger_bounds =
                        overlay::anchor_bounds_for_element(cx, trigger_id);
                    let anchor = anchor_point.or_else(|| trigger_bounds.map(|r| r.origin));
                    let Some(anchor) = anchor else {
                        return (Vec::new(), None);
                    };

                    let entries_tree: Vec<ContextMenuEntry> = entries(cx).into_iter().collect();
                    let gating = crate::command_gating::snapshot_for_window(&*cx.app, cx.window);
                    fn reserve_leading_slot(entries: &[ContextMenuEntry]) -> bool {
                        for entry in entries {
                            match entry {
                                ContextMenuEntry::Item(item) => {
                                    if item.leading.is_some() || item.leading_icon.is_some() {
                                        return true;
                                    }
                                }
                                ContextMenuEntry::CheckboxItem(item) => {
                                    if item.leading.is_some() {
                                        return true;
                                    }
                                }
                                ContextMenuEntry::RadioItem(item) => {
                                    if item.leading.is_some() {
                                        return true;
                                    }
                                }
                                ContextMenuEntry::RadioGroup(group) => {
                                    if group.items.iter().any(|item| item.leading.is_some()) {
                                        return true;
                                    }
                                }
                                ContextMenuEntry::Group(group) => {
                                    if reserve_leading_slot(&group.entries) {
                                        return true;
                                    }
                                }
                                ContextMenuEntry::Label(_) | ContextMenuEntry::Separator => {}
                            }
                        }
                        false
                    }

                    fn item_count(entries: &[ContextMenuEntry]) -> usize {
                        let mut count = 0usize;
                        for entry in entries {
                            match entry {
                                ContextMenuEntry::Item(_)
                                | ContextMenuEntry::CheckboxItem(_)
                                | ContextMenuEntry::RadioItem(_) => count += 1,
                                ContextMenuEntry::RadioGroup(group) => count += group.items.len(),
                                ContextMenuEntry::Group(group) => count += item_count(&group.entries),
                                ContextMenuEntry::Label(_) | ContextMenuEntry::Separator => {}
                            }
                        }
                        count
                    }

                    fn push_leaf_label_and_disabled<H: UiHost>(
                        cx: &ElementContext<'_, H>,
                        gating: &WindowCommandGatingSnapshot,
                        label: &Arc<str>,
                        disabled: bool,
                        command: Option<&CommandId>,
                        labels: &mut Vec<Arc<str>>,
                        disabled_flags: &mut Vec<bool>,
                    ) {
                        labels.push(label.clone());
                        disabled_flags.push(
                            disabled
                                || crate::command_gating::command_is_disabled_by_gating(
                                    &*cx.app,
                                    gating,
                                    command,
                                ),
                        );
                    }

                    fn collect_labels_and_disabled<H: UiHost>(
                        cx: &ElementContext<'_, H>,
                        gating: &WindowCommandGatingSnapshot,
                        entries: &[ContextMenuEntry],
                        labels: &mut Vec<Arc<str>>,
                        disabled_flags: &mut Vec<bool>,
                    ) {
                        for entry in entries {
                            match entry {
                                ContextMenuEntry::Item(item) => push_leaf_label_and_disabled(
                                    cx,
                                    gating,
                                    &item.label,
                                    item.disabled,
                                    item.command.as_ref(),
                                    labels,
                                    disabled_flags,
                                ),
                                ContextMenuEntry::CheckboxItem(item) => push_leaf_label_and_disabled(
                                    cx,
                                    gating,
                                    &item.label,
                                    item.disabled,
                                    item.command.as_ref(),
                                    labels,
                                    disabled_flags,
                                ),
                                ContextMenuEntry::RadioItem(item) => push_leaf_label_and_disabled(
                                    cx,
                                    gating,
                                    &item.label,
                                    item.disabled,
                                    item.command.as_ref(),
                                    labels,
                                    disabled_flags,
                                ),
                                ContextMenuEntry::RadioGroup(group) => {
                                    for spec in group.items.iter() {
                                        push_leaf_label_and_disabled(
                                            cx,
                                            gating,
                                            &spec.label,
                                            spec.disabled,
                                            spec.command.as_ref(),
                                            labels,
                                            disabled_flags,
                                        );
                                    }
                                }
                                ContextMenuEntry::Group(group) => collect_labels_and_disabled(
                                    cx,
                                    gating,
                                    &group.entries,
                                    labels,
                                    disabled_flags,
                                ),
                                ContextMenuEntry::Label(_) | ContextMenuEntry::Separator => {}
                            }
                        }
                    }

                    let reserve_leading_slot =
                        align_leading_icons && reserve_leading_slot(entries_tree.as_slice());
                    let item_count = item_count(entries_tree.as_slice());
                    let mut labels: Vec<Arc<str>> = Vec::new();
                    let mut disabled_flags: Vec<bool> = Vec::new();
                    collect_labels_and_disabled(
                        cx,
                        &gating,
                        entries_tree.as_slice(),
                        &mut labels,
                        &mut disabled_flags,
                    );

                    let labels_arc: Arc<[Arc<str>]> = Arc::from(labels.into_boxed_slice());
                    let disabled_arc: Arc<[bool]> = Arc::from(disabled_flags.into_boxed_slice());

                    let outer = overlay::outer_bounds_with_window_margin_for_environment(
                        cx,
                        fret_ui::Invalidation::Layout,
                        window_margin,
                    );

                    let align = match align {
                        DropdownMenuAlign::Start => Align::Start,
                        DropdownMenuAlign::Center => Align::Center,
                        DropdownMenuAlign::End => Align::End,
                    };
                    let side = dropdown_menu_overlay_side(direction, side);

                    let (arrow_options, arrow_protrusion) =
                        popper::diamond_arrow_options(arrow, arrow_size, arrow_padding);

                    let anchor_rect = overlay::anchor_rect_from_point(anchor);
                    let popper_placement =
                        popper::PopperContentPlacement::new(direction, side, align, side_offset)
                            .with_shift_cross_axis(true)
                            .with_arrow(arrow_options, arrow_protrusion);
                    let popper_vars = menu::context_menu_popper_vars(
                        outer,
                        anchor_rect,
                        min_width,
                        popper_placement,
                    );
                    let desired_w =
                        menu::context_menu_popper_desired_width(outer, anchor_rect, min_width);
                    let max_h = theme
                        .metric_by_key("component.context_menu.max_height")
                        .map(|h| Px(h.0.min(popper_vars.available_height.0)))
                        .unwrap_or(popper_vars.available_height);
                    let menu_font_line_height = theme.metric_token("font.line_height");
                    let menu_pad_y = MetricRef::space(Space::N1p5).resolve(&theme);
                    let menu_row_height = Px(menu_font_line_height.0 + menu_pad_y.0 * 2.0);
                    let desired_h = estimated_menu_panel_height_for_entries(
                        entries_tree.as_slice(),
                        menu_row_height,
                        max_h,
                    );
                    let desired = Size::new(desired_w, desired_h);

                    let (layout, placement_trace) = popper::popper_layout_sized_with_trace(
                        outer,
                        anchor_rect,
                        desired,
                        popper_placement.side_offset,
                        popper_placement.side,
                        popper_placement.align,
                        popper_placement.options(),
                    );

                    let placed = layout.rect;
                    let wrapper_insets = popper_arrow::wrapper_insets(&layout, arrow_protrusion);
                    let extra_left = wrapper_insets.left;
                    let extra_top = wrapper_insets.top;
                    let origin = popper::popper_content_transform_origin(
                        &layout,
                        anchor_rect,
                        arrow.then_some(arrow_size),
                    );
                    let transform = overlay_motion::shadcn_popper_presence_transform(
                        layout.side,
                        origin,
                        opacity,
                        scale,
                        opening,
                    );

                    let border = theme.color_token("border");
                    let radius_sm = MetricRef::radius(Radius::Sm).resolve(&theme);
                    let ring = decl_style::focus_ring(&theme, radius_sm);
                    let pad_x = MetricRef::space(Space::N2).resolve(&theme);
                    let pad_x_inset = MetricRef::space(Space::N8).resolve(&theme);
                    let pad_y = MetricRef::space(Space::N1p5).resolve(&theme);
                    let font_size = theme.metric_token("font.size");
                    let font_line_height = theme.metric_token("font.line_height");
                    let mut text_style = typography::fixed_line_box_style(
                        fret_core::FontId::ui(),
                        font_size,
                        font_line_height,
                    );
                    text_style.weight = fret_core::FontWeight::NORMAL;
                    let text_disabled = alpha_mul(theme.color_token("foreground"), 0.5);
                    let label_fg = theme.color_token("muted-foreground");
                    let accent = theme.color_token("accent");
                    let accent_fg = theme.color_token("accent-foreground");
                    let fg = theme.color_token("foreground");
                    let destructive_fg = theme.color_token("destructive");
                    let destructive_bg = menu_destructive_focus_bg(&theme, destructive_fg);
                    let panel_bg = theme.color_token("popover.background");
                    let panel_chrome = crate::ui_builder_ext::surfaces::menu_style_chrome();

                    let entries = entries_tree;
                    let open_for_submenu = open_for_overlay.clone();
                    let submenu_for_content = submenu.clone();
                    let submenu_for_panel = submenu.clone();
                    let first_item_focus_id_model_for_content =
                        first_item_focus_id_model_for_overlay.clone();
                    let last_item_focus_id_model_for_content =
                        last_item_focus_id_model_for_overlay.clone();
                    let submenu_open_value_model_for_panel = submenu_for_panel.open_value.clone();
                    let submenu_open_value_model_for_panel_for_content =
                        submenu_open_value_model_for_panel.clone();
                    let cancel_open_for_panel = cancel_open.clone();

                    // Match Radix: `role=menu` is on the content panel element (not a fullscreen
                    // wrapper). We keep the popper wrapper for arrow hit-test expansion, but
                    // position it locally inside the menu semantics node.
                    let content_layout = LayoutStyle {
                        position: PositionStyle::Absolute,
                        inset: InsetStyle {
                            left: Some(placed.origin.x).into(),
                            top: Some(placed.origin.y).into(),
                            ..Default::default()
                        },
                        size: SizeStyle {
                            width: Length::Px(placed.size.width),
                            height: Length::Px(placed.size.height),
                            ..Default::default()
                        },
                        overflow: Overflow::Visible,
                        ..Default::default()
                    };

                    let placed_local = Rect::new(Point::new(Px(0.0), Px(0.0)), placed.size);

                    let submenu_entries_for_panel_cell: Rc<RefCell<Option<Vec<ContextMenuEntry>>>> =
                        Rc::new(RefCell::new(None));
                    let submenu_entries_for_panel_cell_for_wrapper =
                        submenu_entries_for_panel_cell.clone();
                    let content_focus_id_for_content = content_focus_id_for_children.clone();

                    let (content_id, content) =
                        menu::content_panel::menu_content_semantics_with_id_props(
                            cx,
                            SemanticsProps {
                                layout: content_layout,
                                test_id: content_test_id.clone(),
                                ..Default::default()
                            },
                            move |cx| {
                            vec![popper_content::popper_wrapper_at(
                                cx,
                                placed_local,
                                wrapper_insets,
                                move |cx| {
                                    let arrow_el = arrow
                                        .then(|| {
                                            popper_arrow::diamond_arrow_element(
                                                cx,
                                                &layout,
                                                wrapper_insets,
                                                arrow_size,
                                                DiamondArrowStyle {
                                                    bg: panel_bg,
                                                    border: Some(border),
                                                    border_width: Px(1.0),
                                                },
                                            )
                                        })
                                        .flatten();

                                    let overlay_root_name_for_controls_for_content =
                                        overlay_root_name_for_controls.clone();
                                    let submenu_open_value_for_panel = cx
                                        .app
                                        .models_mut()
                                        .read(
                                            &submenu_open_value_model_for_panel_for_content,
                                            |v| v.clone(),
                                        )
                                        .ok()
                                        .flatten();
                                    let mut entries_for_panel = entries;
                                    let submenu_entries_for_panel = submenu_open_value_for_panel
                                        .as_ref()
                                        .and_then(|open_value| {
                                            take_submenu_entries_by_value(
                                                &mut entries_for_panel,
                                                open_value.as_ref(),
                                            )
                                        });
                                    *submenu_entries_for_panel_cell_for_wrapper.borrow_mut() =
                                        submenu_entries_for_panel;
                                    let panel = menu::content_panel::menu_panel_container_at(
                                        cx,
                                        Rect::new(Point::new(extra_left, extra_top), placed.size),
                                        move |layout| {
                                            let mut props = decl_style::container_props(
                                                &theme,
                                                panel_chrome.clone(),
                                                LayoutRefinement::default(),
                                            );
                                            props.layout = layout;
                                            props
                                        },
                                        move |cx| {
                                            let content_focus_id_for_panel =
                                                content_focus_id_for_content.clone();
                                            let roving = menu::content::menu_roving_group_apg_prefix_typeahead(
                                                cx,
                                                RovingFlexProps {
                                                    flex: FlexProps {
                                                        layout: LayoutStyle::default(),
                                                        direction: fret_core::Axis::Vertical,
                                                        gap: Px(0.0).into(),
                                                        padding: Edges::all(Px(0.0)).into(),
                                                        justify: MainAlign::Start,
                                                        align: CrossAlign::Stretch,
                                                        wrap: false,
                                                    },
                                                    roving: RovingFocusProps {
                                                        enabled: true,
                                                        wrap: false,
                                                        disabled: disabled_arc.clone(),
                                                        ..Default::default()
                                                    },
                                                },
                                                 labels_arc.clone(),
                                                 typeahead_timeout_ticks,
                                                move |cx| {
                                                    let render_env = ContextMenuContentRenderEnv {
                                                        open: open_for_overlay.clone(),
                                                        cancel_open: cancel_open.clone(),
                                                        gating: gating.clone(),
                                                        reserve_leading_slot,
                                                        item_count,
                                                        ring,
                                                        border,
                                                        radius_sm,
                                                        pad_x,
                                                        pad_x_inset,
                                                        pad_y,
                                                        font_size,
                                                        font_line_height,
                                                        text_style: text_style.clone(),
                                                        text_disabled,
                                                        label_fg,
                                                        accent,
                                                        accent_fg,
                                                        fg,
                                                        destructive_fg,
                                                        destructive_bg,
                                                        window_margin,
                                                        submenu_max_height_metric,
                                                        overlay_root_name_for_controls:
                                                            overlay_root_name_for_controls_for_content
                                                                .clone(),
                                                        submenu_cfg,
                                                        submenu_models: submenu_for_content.clone(),
                                                    };

                                                    let mut out: Vec<AnyElement> =
                                                        Vec::with_capacity(entries_for_panel.len());

                                            let mut item_ix: usize = 0;
                                            for entry in entries_for_panel {
                                                match entry {
                                                    ContextMenuEntry::Label(label) => {
                                                        let dir = crate::direction::use_direction(cx, None);
                                                        let pad_left =
                                                            if label.inset { pad_x_inset } else { pad_x };
                                                        let text = label.text.clone();
                                                        out.push(cx.container(
                                                            ContainerProps {
                                                                layout: LayoutStyle::default(),
                                                                padding: rtl::padding_edges_with_inline_start_end(
                                                                    dir,
                                                                    pad_y,
                                                                    pad_y,
                                                                    pad_left,
                                                                    pad_x,
                                                                )
                                                                .into(),
                                                                ..Default::default()
                                                            },
                                                            move |cx| {
                                                                vec![ui::text( text)
                                                                    .text_size_px(font_size)
                                                                    .line_height_px(font_line_height)
                                                                    .line_height_policy(
                                                                        fret_core::TextLineHeightPolicy::FixedFromStyle,
                                                                    )
                                                                    .font_medium()
                                                                    .nowrap()
                                                                    .text_color(ColorRef::Color(label_fg))
                                                                    .into_element(cx)]
                                                            },
                                                        ));
                                                    }
                                                    ContextMenuEntry::Group(group) => {
                                                        let children = render_env.render_entries(
                                                            cx,
                                                            group.entries,
                                                            &mut item_ix,
                                                        );
                                                        out.push(menu_structural_group(
                                                            cx,
                                                            fret_core::SemanticsRole::Group,
                                                            children.into_vec(),
                                                        ));
                                                    }
                                                    ContextMenuEntry::RadioGroup(group) => {
                                                        let group_value = group.value.clone();
                                                        let on_value_change =
                                                            group.on_value_change.clone();
                                                        let mut children: Vec<AnyElement> =
                                                            Vec::with_capacity(group.items.len());
                                                        for spec in group.items {
                                                            children.push(render_env.render_radio_item(
                                                                cx,
                                                                spec.into_item(
                                                                    group_value.clone(),
                                                                    on_value_change.clone(),
                                                                ),
                                                                &mut item_ix,
                                                            ));
                                                        }
                                                        out.push(menu_structural_group(
                                                            cx,
                                                            fret_core::SemanticsRole::Group,
                                                            children,
                                                        ));
                                                    }
                                                    ContextMenuEntry::Separator => {
                                                        out.push(cx.container(
                                                            ContainerProps {
                                                                layout: {
                                                                    let mut layout =
                                                                        LayoutStyle::default();
                                                                    layout.size.width = Length::Fill;
                                                                    layout.size.height =
                                                                        Length::Px(Px(1.0));
                                                                    // new-york-v4: `Separator` uses `-mx-1 my-1`.
                                                                    layout.margin.left =
                                                                        fret_ui::element::MarginEdge::Px(Px(-4.0));
                                                                    layout.margin.right =
                                                                        fret_ui::element::MarginEdge::Px(Px(-4.0));
                                                                    layout.margin.top =
                                                                        fret_ui::element::MarginEdge::Px(Px(4.0));
                                                                    layout.margin.bottom =
                                                                        fret_ui::element::MarginEdge::Px(Px(4.0));
                                                                    layout
                                                                },
                                                                padding: Edges::all(Px(0.0)).into(),
                                                                background: Some(border),
                                                                ..Default::default()
                                                            },
                                                            |_cx| Vec::new(),
                                                        ));
                                                    }
                                                    ContextMenuEntry::Item(item) => {
                                                        let collection_index = item_ix;
                                                        item_ix = item_ix.saturating_add(1);

                                                        let label = item.label.clone();
                                                        let value = item.value.clone();
                                                        let a11y_label = item
                                                            .a11y_label
                                                            .clone()
                                                            .or_else(|| Some(label.clone()));
                                                        let test_id = item.test_id.clone();
                                                        let chrome_test_id = test_id
                                                            .clone()
                                                            .map(|id| Arc::<str>::from(format!("{id}.chrome")));
                                                        let close_on_select = item.close_on_select;
                                                        let command = item.command;
                                                        let action_payload = item.action_payload;
                                                        let disabled = item.disabled
                                                            || crate::command_gating::command_is_disabled_by_gating(
                                                                &*cx.app,
                                                                &gating,
                                                                command.as_ref(),
                                                            );
                                                        let leading = item.leading;
                                                        let leading_icon = item.leading_icon;
                                                        let trailing = item.trailing;
                                                        let has_submenu = item.submenu.is_some();
                                                        let submenu_estimated_height_unclamped: Option<Px> = None;
                                                        let variant = item.variant;
                                                        let pad_left =
                                                            if item.inset { pad_x_inset } else { pad_x };
                                                        let open = open_for_overlay.clone();
                                                        let text_style = text_style.clone();
                                                        let submenu_for_item = submenu_for_content.clone();
                                                        let overlay_root_name_for_controls =
                                                            overlay_root_name_for_controls.clone();
                                                        let first_item_focus_id_for_items =
                                                            first_item_focus_id_for_children.clone();
                                                        let last_item_focus_id_for_items =
                                                            last_item_focus_id_for_children.clone();
                                                        let first_item_focus_id_model_for_items =
                                                            first_item_focus_id_model_for_content
                                                                .clone();
                                                        let last_item_focus_id_model_for_items =
                                                            last_item_focus_id_model_for_content
                                                                .clone();

                                                        out.push(cx.keyed(value.clone(), move |cx| {
                                                            cx.pressable_with_id_props(
                                                                move |cx, st, item_id| {
                                                                    let geometry_hint =
                                                                        has_submenu.then(|| {
                                                                            let outer = overlay::outer_bounds_with_window_margin_for_environment(
                                                                                cx,
                                                                                fret_ui::Invalidation::Layout,
                                                                                window_margin,
                                                                            );
                                                                            let submenu_max_h =
                                                                                submenu_max_height_metric
                                                                                    .map(|h| {
                                                                                        Px(h.0.min(
                                                                                            outer.size.height.0,
                                                                                        ))
                                                                                    })
                                                                                    .unwrap_or(outer.size.height);
                                                                            let desired_h = submenu_estimated_height_unclamped
                                                                                .map(|estimated| Px(estimated.0.min(submenu_max_h.0)))
                                                                                .unwrap_or(submenu_max_h);
                                                                            let desired = Size::new(
                                                                                submenu_min_width,
                                                                                desired_h,
                                                                            );
                                                                            menu::sub_trigger::MenuSubTriggerGeometryHint {
                                                                                outer,
                                                                                desired,
                                                                            }
                                                                        });
                                                                    let is_open_submenu = menu::sub_trigger::wire(
                                                                        cx,
                                                                        st,
                                                                        item_id,
                                                                        disabled,
                                                                        has_submenu,
                                                                        value.clone(),
                                                                        &submenu_for_item,
                                                                        submenu_cfg,
                                                                        geometry_hint,
                                                                    )
                                                                    .unwrap_or(false);

                                                                    if !disabled {
                                                                        if first_item_focus_id_for_items.get().is_none() {
                                                                            first_item_focus_id_for_items
                                                                                .set(Some(item_id));
                                                                        }
                                                                        let _ = cx.app.models_mut().update(
                                                                            &first_item_focus_id_model_for_items,
                                                                            |v| {
                                                                                if v.is_none() {
                                                                                    *v = Some(item_id);
                                                                                }
                                                                            },
                                                                        );
                                                                        last_item_focus_id_for_items
                                                                            .set(Some(item_id));
                                                                        let _ = cx.app.models_mut().update(
                                                                            &last_item_focus_id_model_for_items,
                                                                            |v| *v = Some(item_id),
                                                                        );
                                                                    }

                                                                    if !has_submenu && !disabled {
                                                                        if let Some(payload) =
                                                                            action_payload.clone()
                                                                        {
                                                                            cx.pressable_dispatch_command_with_payload_factory_if_enabled_opt(
                                                                                command.clone(),
                                                                                payload,
                                                                            );
                                                                        } else {
                                                                            cx.pressable_dispatch_command_if_enabled_opt(command.clone());
                                                                        }
                                                                        if close_on_select {
                                                                            cx.pressable_set_bool(
                                                                                &open, false,
                                                                            );
                                                                        }
                                                                    }

                                                                    let controls = has_submenu.then(|| {
                                                                        menu::sub_content::submenu_content_semantics_id(
                                                                            cx,
                                                                            overlay_root_name_for_controls
                                                                                .as_ref(),
                                                                            &value,
                                                                        )
                                                                    });
                                                                    let mut a11y =
                                                                        menu::item::menu_item_a11y_with_controls(
                                                                            a11y_label,
                                                                            has_submenu.then_some(
                                                                                is_open_submenu,
                                                                            ),
                                                                            controls,
                                                                        );
                                                                    a11y.test_id = test_id.clone();
                                                                    let props = PressableProps {
                                                                        layout: {
                                                                            let mut layout =
                                                                                LayoutStyle::default();
                                                                            layout.size.width =
                                                                                Length::Fill;
                                                                            layout.size.min_height =
                                                                                Some(Length::Px(Px(28.0)));
                                                                            layout
                                                                        },
                                                                        enabled: !disabled,
                                                                        focusable: !disabled,
                                                                        focus_ring: Some(ring),
                                                                        a11y: a11y.with_collection_position(
                                                                            collection_index,
                                                                            item_count,
                                                                        ),
                                                                        ..Default::default()
                                                                    };

                                                                    let mut row_bg =
                                                                        fret_core::Color::TRANSPARENT;
                                                                    let mut row_fg = if variant == ContextMenuItemVariant::Destructive {
                                                                        destructive_fg
                                                                    } else {
                                                                        fg
                                                                    };
                                                                    if st.hovered
                                                                        || st.pressed
                                                                        || st.focused
                                                                        || is_open_submenu
                                                                    {
                                                                        if variant == ContextMenuItemVariant::Destructive {
                                                                            row_bg = destructive_bg;
                                                                            row_fg = destructive_fg;
                                                                        } else {
                                                                            row_bg = accent;
                                                                            row_fg = accent_fg;
                                                                        }
                                                                    }

                                                                    let icon_fg = if variant
                                                                        == ContextMenuItemVariant::Destructive
                                                                    {
                                                                        destructive_fg
                                                                    } else {
                                                                        label_fg
                                                                    };

                                                                    let mut trailing = trailing;
                                                                    if !has_submenu && trailing.is_none() {
                                                                        trailing = command.as_ref().and_then(|cmd| {
                                                                            command_shortcut_label(
                                                                                cx,
                                                                                cmd,
                                                                                fret_runtime::Platform::current(),
                                                                            )
                                                                            .map(|text| {
                                                                                ContextMenuShortcut::new(text)
                                                                                    .into_element(cx)
                                                                            })
                                                                        });
                                                                    }

                                                                    let children = menu_row_children(
                                                                        cx,
                                                                        label.clone(),
                                                                        leading,
                                                                        leading_icon,
                                                                        reserve_leading_slot,
                                                                        trailing,
                                                                        has_submenu,
                                                                        None,
                                                                        disabled,
                                                                        row_bg,
                                                                        row_fg,
                                                                        icon_fg,
                                                                        text_style.clone(),
                                                                        font_size,
                                                                        font_line_height,
                                                                        pad_left,
                                                                        pad_x,
                                                                        pad_y,
                                                                        radius_sm,
                                                                        text_disabled,
                                                                        chrome_test_id.clone(),
                                                                    );

                                                                    (props, children)
                                                                },
                                                            )
                                                          }));
                                                      }
                                                    ContextMenuEntry::CheckboxItem(item) => {
                                                        let collection_index = item_ix;
                                                        item_ix = item_ix.saturating_add(1);

                                                        let label = item.label.clone();
                                                        let value = item.value.clone();
                                                        let checked = item.checked.clone();
                                                        let on_checked_change =
                                                            item.on_checked_change.clone();
                                                        let a11y_label = item
                                                            .a11y_label
                                                            .clone()
                                                            .or_else(|| Some(label.clone()));
                                                        let close_on_select = item.close_on_select;
                                                        let command = item.command;
                                                        let disabled = item.disabled
                                                            || crate::command_gating::command_is_disabled_by_gating(
                                                                &*cx.app,
                                                                &gating,
                                                                command.as_ref(),
                                                            );
                                                        let leading = item.leading;
                                                        let trailing = item.trailing;
                                                        let open = open_for_overlay.clone();
                                                        let text_style = text_style.clone();
                                                        let first_item_focus_id_for_items =
                                                            first_item_focus_id_for_children.clone();
                                                        let last_item_focus_id_for_items =
                                                            last_item_focus_id_for_children.clone();
                                                        let first_item_focus_id_model_for_items =
                                                            first_item_focus_id_model_for_content
                                                                .clone();
                                                        let last_item_focus_id_model_for_items =
                                                            last_item_focus_id_model_for_content
                                                                .clone();

                                                        out.push(cx.keyed(value.clone(), |cx| {
                                                            cx.pressable_with_id_props(
                                                                move |cx, st, item_id| {
                                                                    let checked_now =
                                                                        checked.snapshot(cx);

                                                                    if !disabled {
                                                                        if first_item_focus_id_for_items
                                                                            .get()
                                                                            .is_none()
                                                                        {
                                                                            first_item_focus_id_for_items
                                                                                .set(Some(item_id));
                                                                        }
                                                                        let _ = cx.app.models_mut().update(
                                                                            &first_item_focus_id_model_for_items,
                                                                            |v| {
                                                                                if v.is_none() {
                                                                                    *v = Some(item_id);
                                                                                }
                                                                            },
                                                                        );
                                                                        last_item_focus_id_for_items
                                                                            .set(Some(item_id));
                                                                        let _ = cx.app.models_mut().update(
                                                                            &last_item_focus_id_model_for_items,
                                                                            |v| *v = Some(item_id),
                                                                        );
                                                                        let checked_for_activate =
                                                                            checked.clone();
                                                                        let on_checked_change_for_activate =
                                                                            on_checked_change.clone();
                                                                        cx.pressable_on_activate(
                                                                            Arc::new(
                                                                                move |host, action_cx, _reason| {
                                                                                    let next =
                                                                                        checked_for_activate
                                                                                            .toggle(host);
                                                                                    if let Some(handler) =
                                                                                        on_checked_change_for_activate
                                                                                            .as_ref()
                                                                                    {
                                                                                        handler(
                                                                                            host,
                                                                                            action_cx,
                                                                                            next,
                                                                                        );
                                                                                    }
                                                                                },
                                                                            ),
                                                                        );
                                                                    }

                                                                    cx.pressable_dispatch_command_if_enabled_opt(
                                                                        command.clone(),
                                                                    );
                                                                    if !disabled && close_on_select {
                                                                        cx.pressable_set_bool(&open, false);
                                                                    }

                                                                    let mut trailing = trailing;
                                                                    if trailing.is_none() {
                                                                        trailing = command.as_ref().and_then(|cmd| {
                                                                            command_shortcut_label(
                                                                                cx,
                                                                                cmd,
                                                                                fret_runtime::Platform::current(),
                                                                            )
                                                                            .map(|text| {
                                                                                ContextMenuShortcut::new(text)
                                                                                    .into_element(cx)
                                                                            })
                                                                        });
                                                                    }

                                                                    let mut row_bg =
                                                                        fret_core::Color::TRANSPARENT;
                                                                    let mut row_fg = fg;
                                                                    if st.hovered || st.pressed || st.focused {
                                                                        row_bg = accent;
                                                                        row_fg = accent_fg;
                                                                    }

                                                                    let children = menu_row_children(
                                                                        cx,
                                                                        label.clone(),
                                                                        leading,
                                                                        None,
                                                                        reserve_leading_slot,
                                                                        trailing,
                                                                        false,
                                                                        Some(checked_now),
                                                                        disabled,
                                                                        row_bg,
                                                                        row_fg,
                                                                        row_fg,
                                                                        text_style.clone(),
                                                                        font_size,
                                                                        font_line_height,
                                                                        pad_x,
                                                                        pad_x,
                                                                        pad_y,
                                                                        radius_sm,
                                                                        text_disabled,
                                                                        None,
                                                                    );

                                                                    let props = PressableProps {
                                                                        layout: {
                                                                            let mut layout =
                                                                                LayoutStyle::default();
                                                                            layout.size.width =
                                                                                Length::Fill;
                                                                            layout.size.min_height =
                                                                                Some(Length::Px(Px(28.0)));
                                                                            layout
                                                                        },
                                                                        enabled: !disabled,
                                                                        focusable: !disabled,
                                                                        focus_ring: Some(ring),
                                                                        a11y: menu::item::menu_item_checkbox_a11y(
                                                                            a11y_label.clone(),
                                                                            checked_now,
                                                                        )
                                                                        .with_collection_position(
                                                                            collection_index,
                                                                            item_count,
                                                                        ),
                                                                        ..Default::default()
                                                                    };

                                                                    (props, children)
                                                                },
                                                            )
                                                        }));
                                                    }
                                                    ContextMenuEntry::RadioItem(item) => {
                                                        let collection_index = item_ix;
                                                        item_ix = item_ix.saturating_add(1);

                                                        let label = item.label.clone();
                                                        let value = item.value.clone();
                                                        let group_value = item.group_value.clone();
                                                        let on_value_change =
                                                            item.on_value_change.clone();
                                                        let a11y_label = item
                                                            .a11y_label
                                                            .clone()
                                                            .or_else(|| Some(label.clone()));
                                                        let close_on_select = item.close_on_select;
                                                        let command = item.command;
                                                        let disabled = item.disabled
                                                            || crate::command_gating::command_is_disabled_by_gating(
                                                                &*cx.app,
                                                                &gating,
                                                                command.as_ref(),
                                                            );
                                                        let leading = item.leading;
                                                        let trailing = item.trailing;
                                                        let open = open_for_overlay.clone();
                                                        let text_style = text_style.clone();
                                                        let first_item_focus_id_for_items =
                                                            first_item_focus_id_for_children.clone();
                                                        let last_item_focus_id_for_items =
                                                            last_item_focus_id_for_children.clone();
                                                        let first_item_focus_id_model_for_items =
                                                            first_item_focus_id_model_for_content
                                                                .clone();
                                                        let last_item_focus_id_model_for_items =
                                                            last_item_focus_id_model_for_content
                                                                .clone();

                                                        out.push(cx.keyed(value.clone(), |cx| {
                                                            let selected = group_value.snapshot(cx);
                                                            let is_selected = menu::radio_group::is_selected(
                                                                selected.as_ref(),
                                                                &value,
                                                            );
                                                            cx.pressable_with_id_props(
                                                                move |cx, st, item_id| {
                                                                    if !disabled {
                                                                        if first_item_focus_id_for_items
                                                                            .get()
                                                                            .is_none()
                                                                        {
                                                                            first_item_focus_id_for_items
                                                                                .set(Some(item_id));
                                                                        }
                                                                        let _ = cx.app.models_mut().update(
                                                                            &first_item_focus_id_model_for_items,
                                                                            |v| {
                                                                                if v.is_none() {
                                                                                    *v = Some(item_id);
                                                                                }
                                                                            },
                                                                        );
                                                                        last_item_focus_id_for_items
                                                                            .set(Some(item_id));
                                                                        let _ = cx.app.models_mut().update(
                                                                            &last_item_focus_id_model_for_items,
                                                                            |v| *v = Some(item_id),
                                                                        );
                                                                        let group_value_for_activate =
                                                                            group_value.clone();
                                                                        let value_for_activate =
                                                                            value.clone();
                                                                        let on_value_change_for_activate =
                                                                            on_value_change.clone();
                                                                        cx.pressable_on_activate(
                                                                            Arc::new(
                                                                                move |host, action_cx, _reason| {
                                                                                    let Some(next) =
                                                                                        group_value_for_activate
                                                                                            .select(
                                                                                                host,
                                                                                                &value_for_activate,
                                                                                            )
                                                                                    else {
                                                                                        return;
                                                                                    };
                                                                                    if let Some(handler) =
                                                                                        on_value_change_for_activate
                                                                                            .as_ref()
                                                                                    {
                                                                                        handler(
                                                                                            host,
                                                                                            action_cx,
                                                                                            next,
                                                                                        );
                                                                                    }
                                                                                },
                                                                            ),
                                                                        );
                                                                    }

                                                                    cx.pressable_dispatch_command_if_enabled_opt(
                                                                        command.clone(),
                                                                    );
                                                                    if !disabled && close_on_select {
                                                                        cx.pressable_set_bool(&open, false);
                                                                    }

                                                                    let mut trailing = trailing;
                                                                    if trailing.is_none() {
                                                                        trailing = command.as_ref().and_then(|cmd| {
                                                                            command_shortcut_label(
                                                                                cx,
                                                                                cmd,
                                                                                fret_runtime::Platform::current(),
                                                                            )
                                                                            .map(|text| {
                                                                                ContextMenuShortcut::new(text)
                                                                                    .into_element(cx)
                                                                            })
                                                                        });
                                                                    }

                                                                    let mut row_bg =
                                                                        fret_core::Color::TRANSPARENT;
                                                                    let mut row_fg = fg;
                                                                    if st.hovered || st.pressed || st.focused {
                                                                        row_bg = accent;
                                                                        row_fg = accent_fg;
                                                                    }

                                                                    let children = menu_row_children(
                                                                        cx,
                                                                        label.clone(),
                                                                        leading,
                                                                        None,
                                                                        reserve_leading_slot,
                                                                        trailing,
                                                                        false,
                                                                        Some(is_selected),
                                                                        disabled,
                                                                        row_bg,
                                                                        row_fg,
                                                                        row_fg,
                                                                        text_style.clone(),
                                                                        font_size,
                                                                        font_line_height,
                                                                        pad_x,
                                                                        pad_x,
                                                                        pad_y,
                                                                        radius_sm,
                                                                        text_disabled,
                                                                        None,
                                                                    );

                                                                    let props = PressableProps {
                                                                        layout: {
                                                                            let mut layout =
                                                                                LayoutStyle::default();
                                                                            layout.size.width =
                                                                                Length::Fill;
                                                                            layout.size.min_height =
                                                                                Some(Length::Px(Px(28.0)));
                                                                            layout
                                                                        },
                                                                        enabled: !disabled,
                                                                        focusable: !disabled,
                                                                        focus_ring: Some(ring),
                                                                        a11y: menu::item::menu_item_radio_a11y(
                                                                            a11y_label.clone(),
                                                                            is_selected,
                                                                        )
                                                                        .with_collection_position(
                                                                            collection_index,
                                                                            item_count,
                                                                        ),
                                                                        ..Default::default()
                                                                    };

                                                                    (props, children)
                                                                },
                                                            )
                                                        }));
                                                    }
                                                  }
                                              }

                                                    out
                                                },
                                            );
                                            if content_focus_id_for_panel.get().is_none() {
                                                content_focus_id_for_panel.set(Some(roving.id));
                                            }
                                            let scroll_layout = LayoutStyle {
                                                size: SizeStyle {
                                                    width: Length::Fill,
                                                    height: Length::Fill,
                                                    ..Default::default()
                                                },
                                                overflow: Overflow::Clip,
                                                ..Default::default()
                                            };
                                            vec![cx.scroll(
                                                ScrollProps {
                                                    layout: scroll_layout,
                                                    axis: ScrollAxis::Y,
                                                    ..Default::default()
                                                },
                                                move |_cx| vec![roving],
                                            )]
                                        },
                                    );

                                    if let Some(arrow_el) = arrow_el {
                                        vec![arrow_el, panel]
                                    } else {
                                        vec![panel]
                                    }
                                },
                            )]
                        },
                    );
                    cx.diagnostics_record_overlay_placement_anchored_panel(
                        Some(overlay_root_name_for_trace.as_ref()),
                        None,
                        Some(content_id),
                        placement_trace,
                    );
                    content_focus_id_for_children.set(Some(content_id));
                    let first_item_focus_id_model_for_handler =
                        first_item_focus_id_model_for_overlay.clone();
                    let last_item_focus_id_model_for_handler =
                        last_item_focus_id_model_for_overlay.clone();
                    cx.key_on_key_down_for(
                        content_id,
                        Arc::new({
                            let first_item_focus_id_model = first_item_focus_id_model_for_handler;
                            let last_item_focus_id_model = last_item_focus_id_model_for_handler;
                            move |host, _cx, it| {
                                if it.repeat {
                                    return false;
                                }
                                match it.key {
                                    fret_core::KeyCode::ArrowDown => {
                                        let Some(target) = host
                                            .models_mut()
                                            .read(&first_item_focus_id_model, |v| *v)
                                            .ok()
                                            .flatten()
                                        else {
                                            return false;
                                        };
                                        host.request_focus(target);
                                        true
                                    }
                                    fret_core::KeyCode::ArrowUp => {
                                        let Some(target) = host
                                            .models_mut()
                                            .read(&last_item_focus_id_model, |v| *v)
                                            .ok()
                                            .flatten()
                                        else {
                                            return false;
                                        };
                                        host.request_focus(target);
                                        true
                                    }
                                    _ => false,
                                }
                            }
                        }),
                    );

                    let content =
                        overlay_motion::wrap_opacity_and_render_transform(cx, opacity, transform, vec![content]);

                    let dismissible_on_pointer_move =
                        menu::root::submenu_pointer_move_handler(submenu.clone(), submenu_cfg);

                    let mut children = vec![content];
                    let submenu_open_value = cx
                        .app
                        .models_mut()
                        .read(&submenu_open_value_model_for_panel, |v| v.clone())
                        .ok()
                        .flatten();
                    let desired = {
                        let submenu_max_h = submenu_max_height_metric
                            .map(|h| Px(h.0.min(outer.size.height.0)))
                            .unwrap_or(outer.size.height);

                        // Keep submenu placement sizing aligned with the shared menu panel height
                        // estimator so context menus, dropdown menus, and tests all agree on
                        // separator/padding/border contributions.
                        let row_height = Px(font_line_height.0 + pad_y.0 * 2.0);
                        let desired_h = submenu_entries_for_panel_cell
                            .borrow()
                            .as_deref()
                            .map(|entries| {
                                estimated_menu_panel_height_for_entries(
                                    entries,
                                    row_height,
                                    submenu_max_h,
                                )
                            })
                            .unwrap_or(submenu_max_h);
                        Size::new(submenu_min_width, desired_h)
                    };
                    let submenu_is_open = submenu_open_value.is_some();
                    let submenu_present = submenu_is_open;
                    let submenu_opacity = 1.0;
                    let submenu_scale = 1.0;

                    let open_submenu = menu::sub::with_open_submenu_synced(
                        cx,
                        &submenu_for_panel,
                        outer,
                        desired,
                        |_cx, open_value, geometry| (open_value, geometry),
                    );

                    #[derive(Default)]
                    struct SubmenuLastGeometry {
                        geometry: Option<menu::sub::MenuSubmenuGeometry>,
                    }

                    let last_geometry = cx.slot_state(SubmenuLastGeometry::default, |st| {
                        if let Some((_, geometry)) = open_submenu.as_ref() {
                            st.geometry = Some(*geometry);
                        }
                        st.geometry
                    });

                    if submenu_present {
                        let Some(open_value) = submenu_open_value.clone() else {
                            return (children, Some(dismissible_on_pointer_move));
                        };
                        let geometry = open_submenu.map(|(_, geometry)| geometry).or(last_geometry);
                        let Some(geometry) = geometry else {
                            return (children, Some(dismissible_on_pointer_move));
                        };

                        if let Some(submenu_entries) =
                            submenu_entries_for_panel_cell.borrow_mut().take()
                        {
                            let submenu_panel = context_menu_submenu_panel(
                                cx,
                                open_value.clone(),
                                geometry.floating,
                                submenu_entries,
                                self.test_id_prefix.clone(),
                                open_for_submenu.clone(),
                                typeahead_timeout_ticks,
                                align_leading_icons,
                                submenu_for_panel.clone(),
                                cancel_open_for_panel.clone(),
                            );

                            let side =
                                overlay_motion::anchored_side(geometry.reference, geometry.floating);
                            let origin = overlay_motion::shadcn_transform_origin_for_anchored_rect(
                                geometry.reference,
                                geometry.floating,
                                side,
                            );
                            let transform = overlay_motion::shadcn_popper_presence_transform(
                                side,
                                origin,
                                submenu_opacity,
                                submenu_scale,
                                true,
                            );

                            let opacity = submenu_opacity;
                            let submenu_panel = cx.interactivity_gate(
                                submenu_present,
                                submenu_is_open,
                                move |cx| {
                                    vec![overlay_motion::wrap_opacity_and_render_transform(
                                        cx,
                                        opacity,
                                        transform,
                                        vec![submenu_panel],
                                    )]
                                },
                            );
                            children.push(submenu_panel);
                        }
                    }

                    (children, Some(dismissible_on_pointer_move))
                });

                let mut close_auto_focus_policy =
                    menu::root::MenuCloseAutoFocusGuardPolicy::for_modal(modal)
                        .prevent_on_escape(true);
                // Radix context menu returns focus to the document/underlay after outside press
                // dismissal instead of restoring the invocation target.
                close_auto_focus_policy.prevent_on_outside_press = true;
                let (on_dismiss_request, on_close_auto_focus) =
                    menu::root::menu_close_auto_focus_guard_hooks(
                        cx,
                        close_auto_focus_policy,
                        open.clone(),
                        on_dismiss_request.clone(),
                        on_close_auto_focus.clone(),
                    );
                let first_item_focus_id_model_for_request = cx.local_model_keyed(
                    ("context-menu-first-item-focus-id", open.id()),
                    || None::<GlobalElementId>,
                );
                let keyboard_entry_focus = cx
                    .app
                    .models_mut()
                    .read(&first_item_focus_id_model_for_request, |v| *v)
                    .ok()
                    .flatten()
                    .or(first_item_focus_id.get());

                let request = menu::root::dismissible_menu_request_with_modal_and_dismiss_handler(
                    cx,
                    id,
                    trigger_id,
                    open,
                    overlay_presence,
                    overlay_children,
                    overlay_root_name,
                    menu::root::MenuInitialFocusTargets::new()
                        .pointer_content_focus(content_focus_id.get())
                        .keyboard_entry_focus(keyboard_entry_focus),
                    on_open_auto_focus.clone(),
                    on_close_auto_focus,
                    on_dismiss_request,
                    dismissible_on_pointer_move,
                    modal,
                );
                OverlayController::request(cx, request);
            }

            trigger
        })
    }
}

/// Recipe-level builder for composing a context menu from shadcn-style parts.
type ContextMenuDeferredEntries<H> =
    Box<dyn FnOnce(&mut ElementContext<'_, H>) -> Vec<ContextMenuEntry> + 'static>;

enum ContextMenuCompositionEntries<H: UiHost> {
    Eager(Vec<ContextMenuEntry>),
    Deferred(ContextMenuDeferredEntries<H>),
}

pub struct ContextMenuComposition<H: UiHost, TTrigger = ContextMenuTrigger> {
    menu: ContextMenu,
    trigger: Option<TTrigger>,
    content: ContextMenuContent,
    entries: Option<ContextMenuCompositionEntries<H>>,
}

impl<H: UiHost, TTrigger> std::fmt::Debug for ContextMenuComposition<H, TTrigger> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContextMenuComposition")
            .field("menu", &self.menu)
            .field("trigger", &self.trigger.is_some())
            .field("content", &self.content)
            .field("entries", &self.entries.is_some())
            .finish()
    }
}

impl<H: UiHost> ContextMenuComposition<H> {
    pub fn new(menu: ContextMenu) -> Self {
        Self {
            menu,
            trigger: None,
            content: ContextMenuContent::new(),
            entries: None,
        }
    }
}

impl<H: UiHost, TTrigger> ContextMenuComposition<H, TTrigger> {
    pub fn trigger<TNextTrigger>(
        self,
        trigger: TNextTrigger,
    ) -> ContextMenuComposition<H, TNextTrigger> {
        ContextMenuComposition {
            menu: self.menu,
            trigger: Some(trigger),
            content: self.content,
            entries: self.entries,
        }
    }

    pub fn content(mut self, content: impl Into<ContextMenuContent>) -> Self {
        self.content = content.into();
        self
    }

    pub fn entries(mut self, entries: impl IntoIterator<Item = ContextMenuEntry>) -> Self {
        self.entries = Some(ContextMenuCompositionEntries::Eager(
            entries.into_iter().collect(),
        ));
        self
    }

    pub fn entries_with(
        mut self,
        entries: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<ContextMenuEntry> + 'static,
    ) -> Self {
        self.entries = Some(ContextMenuCompositionEntries::Deferred(Box::new(entries)));
        self
    }

    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement
    where
        TTrigger: IntoUiElement<H>,
    {
        let trigger = self
            .trigger
            .expect("ContextMenu::compose().trigger(...) must be provided before into_element()");
        let entries = self
            .entries
            .expect("ContextMenu::compose().entries(...) must be provided before into_element()");
        let content = self.content;

        match entries {
            ContextMenuCompositionEntries::Eager(entries) => {
                self.menu
                    .build_parts(cx, trigger, content, move |_cx| entries)
            }
            ContextMenuCompositionEntries::Deferred(entries) => {
                self.menu
                    .build_parts(cx, trigger, content, move |cx| entries(cx))
            }
        }
    }
}

impl<H: UiHost, TTrigger> IntoUiElement<H> for ContextMenuComposition<H, TTrigger>
where
    TTrigger: IntoUiElement<H>,
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        ContextMenuComposition::into_element(self, cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use fret_app::App;
    use fret_core::UiServices;
    use fret_core::window::ColorScheme;
    use fret_core::{
        AppWindowId, Event, KeyCode, Modifiers, MouseButton, PathCommand, PathConstraints, PathId,
        PathMetrics,
    };
    use fret_core::{PathService, PathStyle, Point, Px, Rect, SemanticsRole, Size};
    use fret_core::{SvgId, SvgService, TextBlobId, TextConstraints, TextMetrics, TextService};
    use fret_runtime::{Effect, FrameId, WindowPendingActionPayloadService};
    use fret_ui::element::PressableA11y;
    use fret_ui::tree::UiTree;
    use fret_ui::{Theme, ThemeConfig};

    fn contains_foreground_scope(el: &AnyElement) -> bool {
        matches!(el.kind, fret_ui::element::ElementKind::ForegroundScope(_))
            || el.children.iter().any(contains_foreground_scope)
    }

    fn find_first_inherited_foreground_node(el: &AnyElement) -> Option<&AnyElement> {
        if el.inherited_foreground.is_some() {
            return Some(el);
        }
        el.children
            .iter()
            .find_map(find_first_inherited_foreground_node)
    }

    fn consume_pending_payload_within_ttl(
        app: &mut App,
        window: AppWindowId,
        command: &CommandId,
    ) -> Option<Box<dyn std::any::Any + Send + Sync>> {
        let base_tick = app.tick_id();
        app.with_global_mut(WindowPendingActionPayloadService::default, |svc, _app| {
            (0..=64).find_map(|delta| {
                svc.consume(
                    window,
                    fret_runtime::TickId(base_tick.0.saturating_add(delta)),
                    command,
                )
            })
        })
    }

    #[test]
    fn destructive_focus_bg_fallback_tracks_theme_color_scheme() {
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                color_scheme: Some(ColorScheme::Dark),
                ..ThemeConfig::default()
            });
        });
        let theme = Theme::global(&app).snapshot();

        let destructive_fg = fret_core::Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };

        let bg = menu_destructive_focus_bg(&theme, destructive_fg);
        assert!(
            (bg.a - 0.2).abs() < 1e-6,
            "expected /20 alpha on dark themes"
        );
    }

    #[test]
    fn context_menu_sub_helpers_attach_submenu_entries() {
        let entry = ContextMenuSub::new(
            ContextMenuSubTrigger::new("More"),
            ContextMenuSubContent::new([
                ContextMenuItem::new("Item A").into(),
                ContextMenuSeparator::new().into(),
                ContextMenuItem::new("Item B").into(),
            ]),
        )
        .into_entry();

        let ContextMenuEntry::Item(item) = entry else {
            panic!("expected ContextMenuEntry::Item for submenu trigger");
        };

        assert_eq!(item.label.as_ref(), "More");
        assert_eq!(item.close_on_select, false);
        assert!(
            item.submenu.is_some(),
            "expected submenu entries to be attached"
        );
        assert_eq!(item.submenu.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn context_menu_portal_wraps_content_config() {
        let mut app = App::new();
        let open = app.models_mut().insert(false);

        let content = ContextMenuPortal::new(ContextMenuContent::new().side_offset(Px(9.0)));
        let menu = ContextMenuContent::from(content).apply_to(ContextMenu::from_open(open));

        assert_eq!(menu.side_offset, Px(9.0));
    }

    #[test]
    fn context_menu_new_controllable_uses_controlled_model_when_provided() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        );

        let controlled = app.models_mut().insert(true);

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let menu = ContextMenu::new_controllable(cx, Some(controlled.clone()), false);
            assert_eq!(menu.open, controlled);
        });
    }

    #[test]
    fn context_menu_uncontrolled_multiple_instances_do_not_share_open_model() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let menu_a = ContextMenu::uncontrolled(cx);
            let menu_b = ContextMenu::uncontrolled(cx);

            assert_ne!(menu_a.open.id(), menu_b.open.id());
        });
    }

    #[test]
    fn context_menu_row_attaches_inherited_foreground_without_wrapper() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let expected_icon_fg = fret_core::Color {
                r: 0.3,
                g: 0.6,
                b: 0.9,
                a: 1.0,
            };
            let elements: Vec<AnyElement> = menu_row_children(
                cx,
                Arc::from("Inspect"),
                None,
                Some(fret_icons::IconId::new_static("lucide.eye")),
                false,
                None,
                false,
                None,
                false,
                fret_core::Color::TRANSPARENT,
                fret_core::Color {
                    r: 0.9,
                    g: 0.9,
                    b: 0.9,
                    a: 1.0,
                },
                expected_icon_fg,
                fret_core::TextStyle {
                    size: Px(14.0),
                    weight: fret_core::FontWeight::NORMAL,
                    line_height: Some(Px(20.0)),
                    ..Default::default()
                },
                Px(14.0),
                Px(20.0),
                Px(8.0),
                Px(12.0),
                Px(6.0),
                Px(6.0),
                fret_core::Color {
                    r: 0.5,
                    g: 0.5,
                    b: 0.5,
                    a: 1.0,
                },
                None,
            )
            .into_iter()
            .collect();

            assert_eq!(elements.len(), 1);
            let inherited = find_first_inherited_foreground_node(&elements[0])
                .expect("expected context menu row subtree to carry inherited foreground");
            assert!(matches!(
                inherited.kind,
                fret_ui::element::ElementKind::SvgIcon(_)
            ));
            assert_eq!(inherited.inherited_foreground, Some(expected_icon_fg));
            assert!(
                !contains_foreground_scope(&elements[0]),
                "expected context menu row to attach inherited foreground without inserting a ForegroundScope"
            );
        });
    }

    #[test]
    fn context_menu_new_controllable_applies_default_open() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let menu = ContextMenu::new_controllable(cx, None, true);
            let open = cx
                .watch_model(&menu.open)
                .layout()
                .copied()
                .unwrap_or(false);
            assert!(open);
        });
    }

    #[test]
    fn context_menu_open_change_events_emit_change_and_complete_after_settle() {
        let mut state = ContextMenuOpenChangeCallbackState::default();

        let (changed, completed) = context_menu_open_change_events(&mut state, false, false, false);
        assert_eq!(changed, None);
        assert_eq!(completed, None);

        let (changed, completed) = context_menu_open_change_events(&mut state, true, true, true);
        assert_eq!(changed, Some(true));
        assert_eq!(completed, None);

        let (changed, completed) = context_menu_open_change_events(&mut state, true, true, false);
        assert_eq!(changed, None);
        assert_eq!(completed, Some(true));
    }

    #[test]
    fn context_menu_open_change_events_complete_without_animation() {
        let mut state = ContextMenuOpenChangeCallbackState::default();

        let _ = context_menu_open_change_events(&mut state, false, false, false);
        let (changed, completed) = context_menu_open_change_events(&mut state, true, true, false);

        assert_eq!(changed, Some(true));
        assert_eq!(completed, Some(true));
    }

    #[test]
    fn estimated_menu_panel_height_clamps_to_max_height() {
        let entries: Vec<ContextMenuEntry> = (0..100)
            .map(|i| {
                ContextMenuEntry::Item(
                    ContextMenuItem::new(format!("Item {i}")).action(CommandId::new("noop")),
                )
            })
            .collect();
        let entries = entries.as_slice();

        let row_height = Px(20.0);
        let max_height = Px(120.0);
        let height =
            super::estimated_menu_panel_height_for_entries(entries, row_height, max_height);
        assert_eq!(height, max_height);
    }

    #[test]
    fn estimated_menu_panel_height_shrinks_for_short_menus() {
        let entries = vec![
            ContextMenuEntry::Item(ContextMenuItem::new("Apple").action(CommandId::new("noop"))),
            ContextMenuEntry::Item(ContextMenuItem::new("Orange").action(CommandId::new("noop"))),
        ];
        let entries = entries.as_slice();

        let row_height = Px(20.0);
        let max_height = Px(120.0);
        let height =
            super::estimated_menu_panel_height_for_entries(entries, row_height, max_height);
        assert_eq!(height, Px(50.0));
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

    fn render_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "context-menu",
            |cx| {
                vec![ContextMenu::from_open(open).into_element(
                    cx,
                    |cx| {
                        cx.container(
                            ContainerProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Px(Px(120.0));
                                    layout.size.height = Length::Px(Px(40.0));
                                    layout
                                },
                                ..Default::default()
                            },
                            |_cx| Vec::new(),
                        )
                    },
                    |_cx| {
                        vec![
                            ContextMenuEntry::Item(ContextMenuItem::new("Alpha")),
                            ContextMenuEntry::Separator,
                            ContextMenuEntry::Item(ContextMenuItem::new("Beta")),
                            ContextMenuEntry::Item(ContextMenuItem::new("Gamma")),
                        ]
                    },
                )]
            },
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    #[test]
    fn context_menu_test_id_prefix_derives_content_id() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(320.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        menu::set_context_menu_anchor_for_open_model(
            &mut app,
            &open,
            Point::new(Px(32.0), Px(24.0)),
        );

        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);
        OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "context-menu-test-id-prefix",
            move |cx| {
                vec![
                    ContextMenu::from_open(open.clone())
                        .test_id_prefix("ctx")
                        .into_element(
                            cx,
                            |cx| {
                                cx.container(
                                    ContainerProps {
                                        layout: {
                                            let mut layout = LayoutStyle::default();
                                            layout.size.width = Length::Px(Px(120.0));
                                            layout.size.height = Length::Px(Px(40.0));
                                            layout
                                        },
                                        ..Default::default()
                                    },
                                    |_cx| Vec::new(),
                                )
                            },
                            |_cx| vec![ContextMenuEntry::Item(ContextMenuItem::new("Alpha"))],
                        ),
                ]
            },
        );
        ui.set_root(root);
        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let ids: Vec<&str> = snap
            .nodes
            .iter()
            .filter_map(|n| n.test_id.as_deref())
            .collect();
        assert!(ids.iter().copied().any(|id| id == "ctx-content"));

        let _ = root;
    }

    fn render_frame_with_dismiss_handler(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        on_dismiss_request: Option<OnDismissRequest>,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "context-menu",
            move |cx| {
                vec![
                    ContextMenu::from_open(open)
                        .on_dismiss_request(on_dismiss_request)
                        .into_element(
                            cx,
                            |cx| {
                                cx.container(
                                    ContainerProps {
                                        layout: {
                                            let mut layout = LayoutStyle::default();
                                            layout.size.width = Length::Px(Px(120.0));
                                            layout.size.height = Length::Px(Px(40.0));
                                            layout
                                        },
                                        ..Default::default()
                                    },
                                    |_cx| Vec::new(),
                                )
                            },
                            |_cx| {
                                vec![
                                    ContextMenuEntry::Item(ContextMenuItem::new("Alpha")),
                                    ContextMenuEntry::Separator,
                                    ContextMenuEntry::Item(ContextMenuItem::new("Beta")),
                                    ContextMenuEntry::Item(ContextMenuItem::new("Gamma")),
                                ]
                            },
                        ),
                ]
            },
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_frame_focusable_trigger(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
    ) -> fret_core::NodeId {
        render_frame_focusable_trigger_with_disabled(ui, app, services, window, bounds, open, false)
    }

    fn render_frame_focusable_trigger_with_disabled(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        disabled: bool,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        OverlayController::begin_frame(app, window);
        let root =
            fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "context-menu-shift-f10",
                |cx| {
                    vec![
                        ContextMenu::from_open(open)
                            .disabled(disabled)
                            .into_element(
                                cx,
                                |cx| {
                                    cx.pressable(
                                        PressableProps {
                                            layout: {
                                                let mut layout = LayoutStyle::default();
                                                layout.size.width = Length::Px(Px(120.0));
                                                layout.size.height = Length::Px(Px(40.0));
                                                layout
                                            },
                                            enabled: true,
                                            focusable: true,
                                            a11y: PressableA11y {
                                                role: Some(SemanticsRole::Button),
                                                label: Some(Arc::from("Trigger")),
                                                test_id: Some(Arc::from("trigger")),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        |cx, _st| {
                                            vec![cx.container(ContainerProps::default(), |_cx| {
                                                Vec::new()
                                            })]
                                        },
                                    )
                                },
                                |_cx| vec![ContextMenuEntry::Item(ContextMenuItem::new("Alpha"))],
                            ),
                    ]
                },
            );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_frame_focusable_trigger_with_rekey(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        use_alt_trigger_key: Model<bool>,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "context-menu-touch-long-press-rekey",
            move |cx| {
                let use_alt_trigger_key = cx
                    .watch_model(&use_alt_trigger_key)
                    .layout()
                    .copied()
                    .unwrap_or(false);

                vec![ContextMenu::from_open(open).into_element(
                    cx,
                    move |cx| {
                        let key = if use_alt_trigger_key {
                            "trigger-b"
                        } else {
                            "trigger-a"
                        };
                        cx.keyed(key, |cx| {
                            cx.pressable(
                                PressableProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Px(Px(120.0));
                                        layout.size.height = Length::Px(Px(40.0));
                                        layout
                                    },
                                    enabled: true,
                                    focusable: true,
                                    a11y: PressableA11y {
                                        role: Some(SemanticsRole::Button),
                                        label: Some(Arc::from("Trigger")),
                                        test_id: Some(Arc::from("trigger")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                |cx, _st| {
                                    vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                                },
                            )
                        })
                    },
                    |_cx| vec![ContextMenuEntry::Item(ContextMenuItem::new("Alpha"))],
                )]
            },
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_frame_focusable_trigger_with_owner_rekey(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        use_alt_owner_key: Model<bool>,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "context-menu-touch-long-press-owner-rekey",
            move |cx| {
                let use_alt_owner_key = cx
                    .watch_model(&use_alt_owner_key)
                    .layout()
                    .copied()
                    .unwrap_or(false);

                let owner_key = if use_alt_owner_key {
                    "owner-b"
                } else {
                    "owner-a"
                };

                vec![cx.keyed(owner_key, |cx| {
                    ContextMenu::from_open(open).into_element(
                        cx,
                        |cx| {
                            cx.pressable(
                                PressableProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Px(Px(120.0));
                                        layout.size.height = Length::Px(Px(40.0));
                                        layout
                                    },
                                    enabled: true,
                                    focusable: true,
                                    a11y: PressableA11y {
                                        role: Some(SemanticsRole::Button),
                                        label: Some(Arc::from("Trigger")),
                                        test_id: Some(Arc::from("trigger")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                |cx, _st| {
                                    vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                                },
                            )
                        },
                        |_cx| vec![ContextMenuEntry::Item(ContextMenuItem::new("Alpha"))],
                    )
                })]
            },
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_frame_focusable_trigger_with_debug_cancel_open(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        debug_out: Rc<Cell<Option<ContextMenuCancelOpenShared>>>,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "context-menu-debug-cancel-open",
            |cx| {
                let shared: ContextMenuCancelOpenShared =
                    cx.slot_state(context_menu_cancel_open_shared, |shared| shared.clone());
                debug_out.set(Some(shared.clone()));

                let theme = Theme::global(&*cx.app).snapshot();
                let is_open = cx.watch_model(&open).layout().copied().unwrap_or(false);
                let motion =
                    radix_presence::scale_fade_presence_with_durations_and_cubic_bezier_duration(
                        cx,
                        is_open,
                        overlay_motion::shadcn_motion_duration_150(cx),
                        overlay_motion::shadcn_motion_duration_150(cx),
                        0.95,
                        1.0,
                        overlay_motion::shadcn_motion_ease_bezier(cx),
                    );
                let overlay_presence = OverlayPresence {
                    present: motion.present,
                    interactive: is_open,
                };

                let overlay_root_name = menu::context_menu_root_name(cx.root_id());
                let content_id_for_trigger =
                    menu::content_panel::menu_content_semantics_id(cx, &overlay_root_name);
                let trigger = cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(120.0));
                            layout.size.height = Length::Px(Px(40.0));
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    |cx, _st| vec![cx.container(ContainerProps::default(), |_cx| Vec::new())],
                );
                let trigger = menu::trigger::apply_menu_trigger_a11y(
                    trigger,
                    is_open,
                    Some(content_id_for_trigger),
                );
                let trigger_id = trigger.id;

                let open_model_id = open.id();
                let anchor_store_model: Model<HashMap<ModelId, Point>> =
                    menu::context_menu_anchor_store_model(cx.app);
                let base_pointer_policy = menu::context_menu_pointer_down_policy(open.clone());
                let touch_long_press = menu::context_menu_touch_long_press();
                let pointer_policy = Arc::new({
                    let anchor_store_model = anchor_store_model.clone();
                    let touch_long_press = touch_long_press.clone();
                    let cancel_open = shared.clone();
                    move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                          acx: fret_ui::action::ActionCx,
                          down: fret_ui::action::PointerDownCx| {
                        let touch_long_press_handled =
                            menu::context_menu_touch_long_press_on_pointer_down(
                                &touch_long_press,
                                host,
                                acx,
                                down,
                            );
                        let handled = base_pointer_policy(host, acx, down);
                        if handled {
                            menu::context_menu_touch_long_press_clear(&touch_long_press, host);
                            let _ = host.models_mut().update(&anchor_store_model, |map| {
                                map.insert(open_model_id, down.position);
                            });
                            if down.button == fret_core::MouseButton::Right {
                                context_menu_cancel_open_start(
                                    &cancel_open,
                                    host,
                                    acx.window,
                                    down.pointer_id,
                                    down.position,
                                );
                                host.capture_pointer();
                            }
                        }
                        touch_long_press_handled || handled
                    }
                });

                let pointer_policy_for_region = pointer_policy.clone();
                let open_for_region = open.clone();
                let cancel_open_for_region = shared.clone();
                let trigger = cx.keyed(
                    (open_model_id, "context-menu-debug-trigger-region"),
                    move |cx| {
                        let pointer_policy_for_region = pointer_policy_for_region.clone();
                        let cancel_open_for_region = cancel_open_for_region.clone();
                        cx.pointer_region(PointerRegionProps::default(), move |cx| {
                            cx.pointer_region_on_pointer_down(pointer_policy_for_region);
                            let cancel_open_for_move = cancel_open_for_region.clone();
                            cx.pointer_region_on_pointer_move(Arc::new(move |_host, _acx, mv| {
                                context_menu_cancel_open_mark_moved_if_needed(
                                    &cancel_open_for_move,
                                    mv.pointer_id,
                                    mv.position,
                                );
                                false
                            }));
                            let cancel_open_for_up = cancel_open_for_region.clone();
                            let cancel_open_for_up_release = cancel_open_for_region.clone();
                            let open_for_up = open_for_region.clone();
                            cx.pointer_region_on_pointer_up(Arc::new(move |host, _acx, up| {
                                context_menu_cancel_open_mark_moved_if_needed(
                                    &cancel_open_for_up_release,
                                    up.pointer_id,
                                    up.position,
                                );
                                context_menu_cancel_open_on_pointer_up(
                                    &cancel_open_for_up,
                                    host,
                                    &open_for_up,
                                    up.pointer_id,
                                    up.position,
                                    up.button,
                                );
                                if up.button == fret_core::MouseButton::Right
                                    || up.pointer_type == fret_core::PointerType::Touch
                                {
                                    host.release_pointer_capture();
                                }
                                false
                            }));
                            let region_id = cx.root_id();
                            let cancel_open_for_timer = cancel_open_for_region.clone();
                            cx.timer_on_timer_for(
                                region_id,
                                Arc::new(move |host, _acx, token| {
                                    context_menu_cancel_open_on_timer(
                                        &cancel_open_for_timer,
                                        host,
                                        token,
                                    );
                                    false
                                }),
                            );
                            vec![trigger]
                        })
                    },
                );

                if overlay_presence.present {
                    let panel_chrome = crate::ui_builder_ext::surfaces::menu_style_chrome();
                    let open_for_panel_item = open.clone();
                    let placed = Rect::new(
                        Point::new(Px(16.0), Px(48.0)),
                        Size::new(Px(120.0), Px(44.0)),
                    );
                    let panel = menu::content_panel::menu_panel_at(
                        cx,
                        placed,
                        move |layout| {
                            let mut props = decl_style::container_props(
                                &theme,
                                panel_chrome,
                                LayoutRefinement::default(),
                            );
                            props.layout = layout;
                            props
                        },
                        move |cx| {
                            vec![cx.pressable(
                                PressableProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Fill;
                                        layout.size.min_height = Some(Length::Px(Px(28.0)));
                                        layout
                                    },
                                    enabled: true,
                                    focusable: true,
                                    a11y: menu::item::menu_item_a11y(
                                        Some(Arc::from("Alpha")),
                                        None,
                                    ),
                                    ..Default::default()
                                },
                                move |cx, _st| {
                                    cx.pressable_add_on_pointer_up(
                                        context_menu_cancel_open_item_pointer_up_handler(
                                            shared.clone(),
                                            open_for_panel_item.clone(),
                                        ),
                                    );
                                    vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                                },
                            )]
                        },
                    );
                    let request = menu::root::dismissible_menu_request_with_modal(
                        cx,
                        cx.root_id(),
                        trigger_id,
                        open.clone(),
                        overlay_presence,
                        vec![panel],
                        overlay_root_name,
                        menu::root::MenuInitialFocusTargets::new(),
                        None,
                        None,
                        None,
                        false,
                    );
                    OverlayController::request(cx, request);
                }

                vec![trigger]
            },
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    #[test]
    fn context_menu_modal_outside_press_can_be_prevented_via_dismiss_handler() {
        use fret_core::MouseButton;

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let open = app.models_mut().insert(false);

        let dismiss_calls = Arc::new(AtomicUsize::new(0));
        let dismiss_calls_for_handler = dismiss_calls.clone();
        let handler: OnDismissRequest = Arc::new(move |_host, _action_cx, req| {
            dismiss_calls_for_handler.fetch_add(1, Ordering::SeqCst);
            req.prevent_default();
        });

        let _ = render_frame_with_dismiss_handler(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            Some(handler.clone()),
        );

        let _ = app.models_mut().update(&open, |v| *v = true);
        let _ = render_frame_with_dismiss_handler(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            Some(handler),
        );

        let outside = Point::new(Px(390.0), Px(230.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: outside,
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
                position: outside,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert!(dismiss_calls.load(Ordering::SeqCst) > 0);
        assert_eq!(app.models().get_copied(&open), Some(true));
    }

    #[test]
    fn context_menu_touch_long_press_timer_opens_menu() {
        use fret_runtime::Effect;

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let open = app.models_mut().insert(false);

        // Frame 1: establish trigger geometry and pointer hooks.
        let root = render_frame_focusable_trigger(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
        );

        let trigger = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable trigger");
        let trigger_bounds = ui.debug_node_bounds(trigger).expect("trigger bounds");
        let touch_pos = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 / 2.0),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 / 2.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(7),
                position: touch_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Touch,
                click_count: 1,
            }),
        );

        let effects = app.flush_effects();
        let long_press_token = effects.iter().find_map(|effect| match effect {
            Effect::SetTimer { token, after, .. }
                if *after == menu::CONTEXT_MENU_TOUCH_LONG_PRESS_DELAY =>
            {
                Some(*token)
            }
            _ => None,
        });
        let Some(long_press_token) = long_press_token else {
            panic!("expected touch long-press timer; effects={effects:?}");
        };

        // The gallery/native diag path rebuilds the trigger subtree while the pointer is still
        // held down. Keep the pending long-press state alive across that rerender.
        let _ = render_frame_focusable_trigger(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Timer {
                token: long_press_token,
            },
        );

        // Frame 2: long-press timer should have opened the context menu.
        let _ = render_frame_focusable_trigger(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));
    }

    #[test]
    fn context_menu_touch_long_press_survives_trigger_subtree_rekey() {
        use fret_runtime::Effect;

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let open = app.models_mut().insert(false);
        let use_alt_trigger_key = app.models_mut().insert(false);

        let root = render_frame_focusable_trigger_with_rekey(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            use_alt_trigger_key.clone(),
        );

        let trigger_a = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable trigger before rekey");
        let trigger_bounds = ui.debug_node_bounds(trigger_a).expect("trigger bounds");
        let touch_pos = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 / 2.0),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 / 2.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(7),
                position: touch_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Touch,
                click_count: 1,
            }),
        );

        let effects = app.flush_effects();
        let long_press_token = effects.iter().find_map(|effect| match effect {
            Effect::SetTimer { token, after, .. }
                if *after == menu::CONTEXT_MENU_TOUCH_LONG_PRESS_DELAY =>
            {
                Some(*token)
            }
            _ => None,
        });
        let Some(long_press_token) = long_press_token else {
            panic!("expected touch long-press timer; effects={effects:?}");
        };

        let _ = app
            .models_mut()
            .update(&use_alt_trigger_key, |value| *value = true);
        let root = render_frame_focusable_trigger_with_rekey(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            use_alt_trigger_key.clone(),
        );

        let trigger_b = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable trigger after rekey");
        assert_ne!(
            trigger_a, trigger_b,
            "expected trigger subtree rekey to replace the trigger node"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Timer {
                token: long_press_token,
            },
        );

        let _ = render_frame_focusable_trigger_with_rekey(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            use_alt_trigger_key,
        );
        assert_eq!(app.models().get_copied(&open), Some(true));
    }

    #[test]
    fn context_menu_touch_long_press_survives_owner_rekey() {
        use fret_runtime::Effect;

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let open = app.models_mut().insert(false);
        let use_alt_owner_key = app.models_mut().insert(false);

        let root = render_frame_focusable_trigger_with_owner_rekey(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            use_alt_owner_key.clone(),
        );

        let trigger_before = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable trigger before owner rekey");
        let trigger_bounds = ui
            .debug_node_bounds(trigger_before)
            .expect("trigger bounds before owner rekey");
        let touch_pos = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 / 2.0),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 / 2.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(11),
                position: touch_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Touch,
                click_count: 1,
            }),
        );

        let effects = app.flush_effects();
        let long_press_token = effects.iter().find_map(|effect| match effect {
            Effect::SetTimer { token, after, .. }
                if *after == menu::CONTEXT_MENU_TOUCH_LONG_PRESS_DELAY =>
            {
                Some(*token)
            }
            _ => None,
        });
        let Some(long_press_token) = long_press_token else {
            panic!("expected touch long-press timer; effects={effects:?}");
        };

        let _ = app
            .models_mut()
            .update(&use_alt_owner_key, |value| *value = true);
        let root = render_frame_focusable_trigger_with_owner_rekey(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            use_alt_owner_key.clone(),
        );

        let trigger_after = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable trigger after owner rekey");
        assert_ne!(
            trigger_before, trigger_after,
            "expected owner rekey to replace the trigger subtree"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Timer {
                token: long_press_token,
            },
        );

        let _ = render_frame_focusable_trigger_with_owner_rekey(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            use_alt_owner_key,
        );
        assert_eq!(app.models().get_copied(&open), Some(true));
    }

    fn render_frame_focusable_trigger_with_underlay(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        underlay_clicked: Model<bool>,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "context-menu-underlay",
            move |cx| {
                let underlay = cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.position = fret_ui::element::PositionStyle::Absolute;
                            layout.inset.left = Some(Px(0.0)).into();
                            layout.inset.right = Some(Px(0.0)).into();
                            layout.inset.top = Some(Px(0.0)).into();
                            layout.inset.bottom = Some(Px(0.0)).into();
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

                let trigger = ContextMenu::from_open(open).into_element(
                    cx,
                    |cx| {
                        cx.pressable(
                            PressableProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Px(Px(120.0));
                                    layout.size.height = Length::Px(Px(40.0));
                                    layout
                                },
                                enabled: true,
                                focusable: true,
                                a11y: PressableA11y {
                                    role: Some(SemanticsRole::Button),
                                    label: Some(Arc::from("Trigger")),
                                    test_id: Some(Arc::from("trigger")),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            |cx, _st| {
                                vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                            },
                        )
                    },
                    |_cx| vec![ContextMenuEntry::Item(ContextMenuItem::new("Alpha"))],
                );

                // Keep the context-menu trigger above the underlay so the right-click open gesture
                // cannot be intercepted by the "underlay" pressable.
                vec![underlay, trigger]
            },
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_frame_focusable_trigger_with_underlay_modal_and_dismiss_handler(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        modal: bool,
        underlay_clicked: Model<bool>,
        on_dismiss_request: Option<OnDismissRequest>,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "context-menu-underlay-modal",
            move |cx| {
                let underlay = cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.position = fret_ui::element::PositionStyle::Absolute;
                            layout.inset.left = Some(Px(0.0)).into();
                            layout.inset.right = Some(Px(0.0)).into();
                            layout.inset.top = Some(Px(0.0)).into();
                            layout.inset.bottom = Some(Px(0.0)).into();
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

                let trigger = ContextMenu::from_open(open)
                    .modal(modal)
                    .on_dismiss_request(on_dismiss_request.clone())
                    .into_element(
                        cx,
                        |cx| {
                            cx.pressable(
                                PressableProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Px(Px(120.0));
                                        layout.size.height = Length::Px(Px(40.0));
                                        layout
                                    },
                                    enabled: true,
                                    focusable: true,
                                    a11y: PressableA11y {
                                        role: Some(SemanticsRole::Button),
                                        label: Some(Arc::from("Trigger")),
                                        test_id: Some(Arc::from("trigger")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                |cx, _st| {
                                    vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                                },
                            )
                        },
                        |_cx| vec![ContextMenuEntry::Item(ContextMenuItem::new("Alpha"))],
                    );

                // Keep the context-menu trigger above the underlay so the right-click open gesture
                // cannot be intercepted by the "underlay" pressable.
                vec![underlay, trigger]
            },
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_frame_focusable_trigger_with_underlay_and_entries(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        underlay_clicked: Model<bool>,
        entries: Vec<ContextMenuEntry>,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "context-menu-underlay-entries",
            move |cx| {
                let underlay = cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.position = fret_ui::element::PositionStyle::Absolute;
                            layout.inset.left = Some(Px(0.0)).into();
                            layout.inset.right = Some(Px(0.0)).into();
                            layout.inset.top = Some(Px(0.0)).into();
                            layout.inset.bottom = Some(Px(0.0)).into();
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

                let trigger = ContextMenu::from_open(open).into_element(
                    cx,
                    |cx| {
                        cx.pressable(
                            PressableProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Px(Px(120.0));
                                    layout.size.height = Length::Px(Px(40.0));
                                    layout
                                },
                                enabled: true,
                                focusable: true,
                                a11y: PressableA11y {
                                    role: Some(SemanticsRole::Button),
                                    label: Some(Arc::from("Trigger")),
                                    test_id: Some(Arc::from("trigger")),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            |cx, _st| {
                                vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                            },
                        )
                    },
                    move |_cx| entries,
                );

                // Keep the context-menu trigger above the underlay so the right-click open gesture
                // cannot be intercepted by the "underlay" pressable.
                vec![underlay, trigger]
            },
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_frame_focusable_trigger_with_underlay_and_entries_and_dismiss_handler(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        underlay_clicked: Model<bool>,
        entries: Vec<ContextMenuEntry>,
        on_dismiss_request: Option<OnDismissRequest>,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "context-menu-underlay-entries-dismiss",
            move |cx| {
                let underlay = cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.position = fret_ui::element::PositionStyle::Absolute;
                            layout.inset.left = Some(Px(0.0)).into();
                            layout.inset.right = Some(Px(0.0)).into();
                            layout.inset.top = Some(Px(0.0)).into();
                            layout.inset.bottom = Some(Px(0.0)).into();
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

                let trigger = ContextMenu::from_open(open)
                    .on_dismiss_request(on_dismiss_request)
                    .into_element(
                        cx,
                        |cx| {
                            cx.pressable(
                                PressableProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Px(Px(120.0));
                                        layout.size.height = Length::Px(Px(40.0));
                                        layout
                                    },
                                    enabled: true,
                                    focusable: true,
                                    a11y: PressableA11y {
                                        role: Some(SemanticsRole::Button),
                                        label: Some(Arc::from("Trigger")),
                                        test_id: Some(Arc::from("trigger")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                |cx, _st| {
                                    vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                                },
                            )
                        },
                        move |_cx| entries,
                    );

                vec![underlay, trigger]
            },
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_frame_focusable_trigger_with_underlay_and_entries_and_auto_focus_hooks(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        underlay_clicked: Model<bool>,
        entries: Vec<ContextMenuEntry>,
        trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        underlay_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        on_open_auto_focus: Option<OnOpenAutoFocus>,
        on_close_auto_focus: Option<OnCloseAutoFocus>,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "context-menu-underlay-entries-autofocus",
            move |cx| {
                let underlay = cx.pressable_with_id(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.position = fret_ui::element::PositionStyle::Absolute;
                            layout.inset.left = Some(Px(0.0)).into();
                            layout.inset.right = Some(Px(0.0)).into();
                            layout.inset.top = Some(Px(0.0)).into();
                            layout.inset.bottom = Some(Px(0.0)).into();
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
                    {
                        let underlay_id_out = underlay_id_out.clone();
                        move |cx, _st, id| {
                            underlay_id_out.set(Some(id));
                            cx.pressable_toggle_bool(&underlay_clicked);
                            vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                        }
                    },
                );

                let trigger =
                    ContextMenu::from_open(open)
                        .on_open_auto_focus(on_open_auto_focus.clone())
                        .on_close_auto_focus(on_close_auto_focus.clone())
                        .into_element(
                            cx,
                            {
                                let trigger_id_out = trigger_id_out.clone();
                                move |cx| {
                                    cx.pressable_with_id(
                                        PressableProps {
                                            layout: {
                                                let mut layout = LayoutStyle::default();
                                                layout.size.width = Length::Px(Px(120.0));
                                                layout.size.height = Length::Px(Px(40.0));
                                                layout
                                            },
                                            enabled: true,
                                            focusable: true,
                                            ..Default::default()
                                        },
                                        move |cx, _st, id| {
                                            trigger_id_out.set(Some(id));
                                            vec![cx.container(ContainerProps::default(), |_cx| {
                                                Vec::new()
                                            })]
                                        },
                                    )
                                }
                            },
                            move |_cx| entries,
                        );

                vec![trigger, underlay]
            },
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_frame_focusable_trigger_with_entries(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        entries: Vec<ContextMenuEntry>,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "context-menu-submenu-arrow-right",
            move |cx| {
                vec![ContextMenu::from_open(open).into_element(
                    cx,
                    |cx| {
                        cx.pressable(
                            PressableProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Px(Px(120.0));
                                    layout.size.height = Length::Px(Px(40.0));
                                    layout
                                },
                                enabled: true,
                                focusable: true,
                                ..Default::default()
                            },
                            |cx, _st| {
                                vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                            },
                        )
                    },
                    move |_cx| entries,
                )]
            },
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_frame_pressable_ancestor_with_non_pressable_trigger_and_entries(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        workspace_activated: Model<bool>,
        entries: Vec<ContextMenuEntry>,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "context-menu-touch-ancestor-pressable",
            move |cx| {
                vec![cx.pressable(
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
                            role: Some(SemanticsRole::TextField),
                            label: Some(Arc::from("Workspace")),
                            test_id: Some(Arc::from("workspace")),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    move |cx, _st| {
                        cx.pressable_set_bool(&workspace_activated, true);
                        vec![ContextMenu::from_open(open).into_element(
                            cx,
                            |cx| {
                                cx.container(
                                    ContainerProps {
                                        layout: {
                                            let mut layout = LayoutStyle::default();
                                            layout.size.width = Length::Px(Px(120.0));
                                            layout.size.height = Length::Px(Px(40.0));
                                            layout
                                        },
                                        ..Default::default()
                                    },
                                    |_cx| Vec::new(),
                                )
                            },
                            move |_cx| entries,
                        )]
                    },
                )]
            },
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    #[test]
    fn context_menu_item_action_payload_records_pending_payload() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(240.0)),
        );
        let mut services = FakeServices::default();
        let cmd = CommandId::new("context_menu.tests.item_payload.v1");

        let build_entries = || {
            vec![ContextMenuEntry::Item(
                ContextMenuItem::new("Payload Item")
                    .action(cmd.clone())
                    .action_payload(41_u32),
            )]
        };

        let root = render_frame_focusable_trigger_with_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            build_entries(),
        );
        let trigger = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable trigger");
        ui.set_focus(Some(trigger));
        let trigger_bounds = ui.debug_node_bounds(trigger).expect("trigger bounds");
        let trigger_pos = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 / 2.0),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 / 2.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: trigger_pos,
                button: fret_core::MouseButton::Right,
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
                position: trigger_pos,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        let _ = render_frame_focusable_trigger_with_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            build_entries(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let item = snap
            .nodes
            .iter()
            .find(|node| {
                node.role == SemanticsRole::MenuItem
                    && node.label.as_deref() == Some("Payload Item")
            })
            .expect("payload item");
        let position = Point::new(
            Px(item.bounds.origin.x.0 + 2.0),
            Px(item.bounds.origin.y.0 + 2.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position,
                button: fret_core::MouseButton::Left,
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
                position,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        let effects = app.flush_effects();
        assert!(
            effects.iter().any(
                |effect| matches!(effect, Effect::Command { command, .. } if command.as_str() == cmd.as_str())
            ),
            "expected click to dispatch {cmd:?}, got {effects:?}"
        );
        let payload = consume_pending_payload_within_ttl(&mut app, window, &cmd)
            .expect("expected pending payload for context-menu item action");
        let payload = payload
            .downcast::<u32>()
            .ok()
            .expect("payload type must match");
        assert_eq!(*payload, 41);
    }

    #[test]
    fn context_menu_touch_long_press_trigger_does_not_delegate_capture_to_pressable_ancestor() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let workspace_activated = app.models_mut().insert(false);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let _ = render_frame_pressable_ancestor_with_non_pressable_trigger_and_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            workspace_activated,
            vec![ContextMenuEntry::Item(ContextMenuItem::new("Alpha"))],
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let workspace = snap
            .nodes
            .iter()
            .find(|node| node.test_id.as_deref() == Some("workspace"))
            .map(|node| node.id)
            .expect("workspace semantics node");

        let trigger_pos = Point::new(Px(60.0), Px(20.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: trigger_pos,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Touch,
                click_count: 1,
            }),
        );

        assert_ne!(
            ui.captured_for(fret_core::PointerId(0)),
            Some(workspace),
            "touch long-press arming should not let an ancestor pressable steal pointer capture"
        );
        assert_eq!(app.models().get_copied(&open), Some(false));
    }

    #[test]
    fn context_menu_touch_long_press_item_tap_dispatches_command_inside_pressable_ancestor() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let workspace_activated = app.models_mut().insert(false);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(240.0)),
        );
        let mut services = FakeServices::default();
        let cmd = CommandId::new("context_menu.tests.touch_item_inside_pressable_ancestor.v1");

        let build_entries = || {
            vec![ContextMenuEntry::Item(
                ContextMenuItem::new("Touch Item").action(cmd.clone()),
            )]
        };

        let _ = render_frame_pressable_ancestor_with_non_pressable_trigger_and_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            workspace_activated.clone(),
            build_entries(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let workspace = snap
            .nodes
            .iter()
            .find(|node| node.test_id.as_deref() == Some("workspace"))
            .map(|node| node.id)
            .expect("workspace semantics node");

        let trigger_pos = Point::new(Px(60.0), Px(20.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: trigger_pos,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Touch,
                click_count: 1,
            }),
        );

        let effects = app.flush_effects();
        let long_press_token = effects.iter().find_map(|effect| match effect {
            Effect::SetTimer { token, after, .. }
                if *after == menu::CONTEXT_MENU_TOUCH_LONG_PRESS_DELAY =>
            {
                Some(*token)
            }
            _ => None,
        });
        let Some(long_press_token) = long_press_token else {
            panic!("expected touch long-press timer; effects={effects:?}");
        };

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Timer {
                token: long_press_token,
            },
        );
        let _ = render_frame_pressable_ancestor_with_non_pressable_trigger_and_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            workspace_activated.clone(),
            build_entries(),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: trigger_pos,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: false,
                pointer_type: fret_core::PointerType::Touch,
                click_count: 1,
            }),
        );
        let _ = render_frame_pressable_ancestor_with_non_pressable_trigger_and_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            workspace_activated.clone(),
            build_entries(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let item = snap
            .nodes
            .iter()
            .find(|node| {
                node.role == SemanticsRole::MenuItem && node.label.as_deref() == Some("Touch Item")
            })
            .expect("touch item");
        let item_pos = Point::new(
            Px(item.bounds.origin.x.0 + item.bounds.size.width.0 / 2.0),
            Px(item.bounds.origin.y.0 + item.bounds.size.height.0 / 2.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(1),
                position: item_pos,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Touch,
                click_count: 1,
            }),
        );
        assert_ne!(
            ui.captured_for(fret_core::PointerId(1)),
            Some(workspace),
            "touching a menu item should not hand pointer capture to the pressable ancestor"
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(1),
                position: item_pos,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Touch,
                click_count: 1,
            }),
        );

        let effects = app.flush_effects();
        assert!(
            effects.iter().any(
                |effect| matches!(effect, Effect::Command { command, .. } if command.as_str() == cmd.as_str())
            ),
            "expected touch tap to dispatch {cmd:?}; open={:?} workspace_activated={:?} captured={:?} effects={effects:?}",
            app.models().get_copied(&open),
            app.models().get_copied(&workspace_activated),
            ui.captured_for(fret_core::PointerId(1)),
        );
        assert_eq!(app.models().get_copied(&open), Some(false));
        assert_eq!(
            app.models().get_copied(&workspace_activated),
            Some(false),
            "menu item tap should not activate the pressable ancestor"
        );
    }

    #[test]
    fn context_menu_submenu_item_action_payload_records_pending_payload() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(360.0), Px(260.0)),
        );
        let mut services = FakeServices::default();
        let cmd = CommandId::new("context_menu.tests.submenu_item_payload.v1");

        let build_entries = || {
            vec![ContextMenuEntry::Item(
                ContextMenuItem::new("More").submenu(vec![ContextMenuEntry::Item(
                    ContextMenuItem::new("Payload Sub Item")
                        .action(cmd.clone())
                        .action_payload(57_u32),
                )]),
            )]
        };

        let root = render_frame_focusable_trigger_with_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            build_entries(),
        );
        let trigger = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable trigger");
        ui.set_focus(Some(trigger));
        let trigger_bounds = ui.debug_node_bounds(trigger).expect("trigger bounds");
        let trigger_pos = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 / 2.0),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 / 2.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: trigger_pos,
                button: fret_core::MouseButton::Right,
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
                position: trigger_pos,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        let _ = render_frame_focusable_trigger_with_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            build_entries(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let more = snap
            .nodes
            .iter()
            .find(|node| {
                node.role == SemanticsRole::MenuItem && node.label.as_deref() == Some("More")
            })
            .expect("More menu item");
        ui.set_focus(Some(more.id));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::ArrowRight,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        let open_effects = app.flush_effects();
        assert!(
            !open_effects.iter().any(
                |effect| matches!(effect, Effect::Command { command, .. } if command.as_str() == cmd.as_str())
            ),
            "opening submenu should not dispatch leaf action {cmd:?}, got {open_effects:?}"
        );

        let _ = render_frame_focusable_trigger_with_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            build_entries(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let item = snap
            .nodes
            .iter()
            .find(|node| {
                node.role == SemanticsRole::MenuItem
                    && node.label.as_deref() == Some("Payload Sub Item")
            })
            .expect("payload submenu item");
        let position = Point::new(
            Px(item.bounds.origin.x.0 + item.bounds.size.width.0 / 2.0),
            Px(item.bounds.origin.y.0 + item.bounds.size.height.0 / 2.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position,
                button: fret_core::MouseButton::Left,
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
                position,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        let effects = app.flush_effects();
        assert!(
            effects.iter().any(
                |effect| matches!(effect, Effect::Command { command, .. } if command.as_str() == cmd.as_str())
            ),
            "expected submenu click to dispatch {cmd:?}, got {effects:?}"
        );
        let payload = consume_pending_payload_within_ttl(&mut app, window, &cmd)
            .expect("expected pending payload for context-menu submenu item action");
        let payload = payload
            .downcast::<u32>()
            .ok()
            .expect("payload type must match");
        assert_eq!(*payload, 57);
    }

    #[test]
    fn context_menu_checkbox_item_from_checked_emits_on_checked_change() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let checked = app.models_mut().insert(false);
        let change_count = app.models_mut().insert(0_u32);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let build_entries = |app: &App| {
            let checked_now = app.models().get_copied(&checked).unwrap_or(false);
            let checked_for_cb = checked.clone();
            let change_count_for_cb = change_count.clone();
            vec![ContextMenuEntry::CheckboxItem(
                ContextMenuCheckboxItem::from_checked(checked_now, "Status Bar")
                    .on_checked_change(move |host, _action_cx, next| {
                        let _ = host
                            .models_mut()
                            .update(&checked_for_cb, |value| *value = next);
                        let _ = host.models_mut().update(&change_count_for_cb, |value| {
                            *value = value.saturating_add(1)
                        });
                    })
                    .test_id("status-bar"),
            )]
        };

        let entries = build_entries(&app);
        let _ = render_frame_focusable_trigger_with_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            entries,
        );
        let _ = app.models_mut().update(&open, |value| *value = true);
        let entries = build_entries(&app);
        let _ = render_frame_focusable_trigger_with_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            entries,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let status_bar = snap
            .nodes
            .iter()
            .find(|node| {
                node.role == SemanticsRole::MenuItemCheckbox
                    && node.label.as_deref() == Some("Status Bar")
            })
            .expect("Status Bar menu item");
        let position = Point::new(
            Px(status_bar.bounds.origin.x.0 + 2.0),
            Px(status_bar.bounds.origin.y.0 + 2.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position,
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
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert_eq!(app.models().get_copied(&checked), Some(true));
        assert_eq!(app.models().get_copied(&change_count), Some(1));

        let entries = build_entries(&app);
        let _ = render_frame_focusable_trigger_with_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open,
            entries,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let status_bar = snap
            .nodes
            .iter()
            .find(|node| {
                node.role == SemanticsRole::MenuItemCheckbox
                    && node.label.as_deref() == Some("Status Bar")
            })
            .expect("Status Bar menu item after callback");
        assert_eq!(status_bar.flags.checked, Some(true));
    }

    #[test]
    fn context_menu_radio_group_from_value_emits_on_value_change() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let selected = app.models_mut().insert(Some(Arc::<str>::from("bottom")));
        let change_count = app.models_mut().insert(0_u32);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let build_entries = |app: &App| {
            let selected_now = app
                .models()
                .read(&selected, |value| value.clone())
                .ok()
                .flatten();
            let selected_for_cb = selected.clone();
            let change_count_for_cb = change_count.clone();
            vec![ContextMenuEntry::RadioGroup(
                ContextMenuRadioGroup::from_value(selected_now)
                    .on_value_change(move |host, _action_cx, next| {
                        let _ = host
                            .models_mut()
                            .update(&selected_for_cb, |value| *value = Some(next));
                        let _ = host.models_mut().update(&change_count_for_cb, |value| {
                            *value = value.saturating_add(1)
                        });
                    })
                    .item(ContextMenuRadioItemSpec::new("top", "Top").close_on_select(false))
                    .item(ContextMenuRadioItemSpec::new("bottom", "Bottom").close_on_select(false)),
            )]
        };

        let entries = build_entries(&app);
        let _ = render_frame_focusable_trigger_with_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            entries,
        );
        let _ = app.models_mut().update(&open, |value| *value = true);
        let entries = build_entries(&app);
        let _ = render_frame_focusable_trigger_with_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            entries,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let top = snap
            .nodes
            .iter()
            .find(|node| {
                node.role == SemanticsRole::MenuItemRadio && node.label.as_deref() == Some("Top")
            })
            .expect("Top radio item");
        let position = Point::new(
            Px(top.bounds.origin.x.0 + 2.0),
            Px(top.bounds.origin.y.0 + 2.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position,
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
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        let selected_now = app
            .models()
            .read(&selected, |value| value.clone())
            .ok()
            .flatten();
        assert_eq!(selected_now.as_deref(), Some("top"));
        assert_eq!(app.models().get_copied(&change_count), Some(1));

        let entries = build_entries(&app);
        let _ = render_frame_focusable_trigger_with_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open,
            entries,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let top = snap
            .nodes
            .iter()
            .find(|node| {
                node.role == SemanticsRole::MenuItemRadio && node.label.as_deref() == Some("Top")
            })
            .expect("Top radio item after callback");
        assert_eq!(top.flags.checked, Some(true));
    }

    #[test]
    fn context_menu_opens_on_shift_f10() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        // First frame: build the tree and establish stable trigger bounds.
        let root = render_frame_focusable_trigger(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
        );

        let trigger = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable trigger");
        ui.set_focus(Some(trigger));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::F10,
                modifiers: Modifiers {
                    shift: true,
                    ..Default::default()
                },
                repeat: false,
            },
        );

        // Second frame: ContextMenu emits its OverlayRequest while rendering.
        // Re-rendering the root is required for the menu items to appear.
        let _ = render_frame_focusable_trigger(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let alpha = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Alpha"))
            .expect("Alpha menu item");
        assert_eq!(ui.focus(), Some(alpha.id));
    }

    #[test]
    fn context_menu_opens_on_context_menu_key() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let root = render_frame_focusable_trigger(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
        );

        let trigger = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable trigger");
        ui.set_focus(Some(trigger));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::ContextMenu,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let _ =
            render_frame_focusable_trigger(&mut ui, &mut app, &mut services, window, bounds, open);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let alpha = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Alpha"))
            .expect("Alpha menu item");
        assert_eq!(ui.focus(), Some(alpha.id));
    }

    #[test]
    fn context_menu_disabled_blocks_pointer_and_keyboard_open() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let root = render_frame_focusable_trigger_with_disabled(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
        );

        let trigger = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable trigger");
        ui.set_focus(Some(trigger));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::F10,
                modifiers: Modifiers {
                    shift: true,
                    ..Default::default()
                },
                repeat: false,
            },
        );

        let trigger_bounds = ui.debug_node_bounds(trigger).expect("trigger bounds");
        let position = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 / 2.0),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 / 2.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_type: fret_core::PointerType::Mouse,
                is_click: true,
            }),
        );

        let _ = render_frame_focusable_trigger_with_disabled(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
        );

        let open_now = app.models().get_copied(&open).expect("open model");
        assert!(
            !open_now,
            "disabled context menu should not open via keyboard or pointer triggers"
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            snap.nodes.iter().all(
                |n| !(n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Alpha"))
            ),
            "disabled context menu should not render menu content"
        );
    }

    #[test]
    fn context_menu_focus_outside_can_be_prevented_via_dismiss_handler() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_clicked = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let reason_cell: Arc<Mutex<Option<fret_ui::action::DismissReason>>> =
            Arc::new(Mutex::new(None));
        let reason_cell_for_handler = reason_cell.clone();
        let handler: OnDismissRequest = Arc::new(move |_host, _action_cx, req| {
            if matches!(req.reason, fret_ui::action::DismissReason::FocusOutside) {
                let mut lock = reason_cell_for_handler.lock().unwrap();
                *lock = Some(req.reason);
                req.prevent_default();
            }
        });

        let entries = vec![ContextMenuEntry::Item(ContextMenuItem::new("Alpha"))];
        let _root = render_frame_focusable_trigger_with_underlay_and_entries_and_dismiss_handler(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked.clone(),
            entries,
            Some(handler.clone()),
        );

        let snap0 = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger = snap0
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("trigger"))
            .map(|n| n.id)
            .expect("trigger node");
        let underlay_node = snap0
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("underlay"))
            .map(|n| n.id)
            .expect("underlay node");
        ui.set_focus(Some(trigger));

        let trigger_bounds = ui.debug_node_bounds(trigger).expect("trigger bounds");
        let position = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 / 2.0),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 / 2.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position,
                button: fret_core::MouseButton::Right,
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
                position,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        let entries = vec![ContextMenuEntry::Item(ContextMenuItem::new("Alpha"))];
        let _ = render_frame_focusable_trigger_with_underlay_and_entries_and_dismiss_handler(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked.clone(),
            entries,
            Some(handler.clone()),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));

        ui.set_focus(Some(underlay_node));

        let entries = vec![ContextMenuEntry::Item(ContextMenuItem::new("Alpha"))];
        let _ = render_frame_focusable_trigger_with_underlay_and_entries_and_dismiss_handler(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked,
            entries,
            Some(handler),
        );

        assert_eq!(
            app.models().get_copied(&open),
            Some(true),
            "expected menu to remain open when focus-outside dismissal is prevented"
        );
        assert_eq!(
            *reason_cell.lock().unwrap(),
            Some(fret_ui::action::DismissReason::FocusOutside)
        );
    }

    #[test]
    fn context_menu_keyboard_open_auto_focus_can_be_prevented() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_clicked = app.models_mut().insert(false);
        let trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let underlay_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_handler = calls.clone();
        let handler: OnOpenAutoFocus = Arc::new(move |_host, _action_cx, req| {
            calls_for_handler.fetch_add(1, Ordering::SeqCst);
            req.prevent_default();
        });

        let _root = render_frame_focusable_trigger_with_underlay_and_entries_and_auto_focus_hooks(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked.clone(),
            vec![ContextMenuEntry::Item(ContextMenuItem::new("Alpha"))],
            trigger_id_out.clone(),
            underlay_id_out,
            Some(handler.clone()),
            None,
        );

        let trigger_id = trigger_id_out.get().expect("trigger element id");
        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger_id).expect("trigger");
        ui.set_focus(Some(trigger_node));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::F10,
                modifiers: Modifiers {
                    shift: true,
                    ..Default::default()
                },
                repeat: false,
            },
        );

        let _root = render_frame_focusable_trigger_with_underlay_and_entries_and_auto_focus_hooks(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked,
            vec![ContextMenuEntry::Item(ContextMenuItem::new("Alpha"))],
            trigger_id_out,
            Rc::new(Cell::new(None)),
            Some(handler),
            None,
        );

        assert_eq!(app.models().get_copied(&open), Some(true));
        assert!(
            calls.load(Ordering::SeqCst) > 0,
            "expected on_open_auto_focus to run"
        );
        assert_eq!(
            ui.focus(),
            Some(trigger_node),
            "expected preventDefault open autofocus to keep focus on trigger"
        );
    }

    #[test]
    fn context_menu_close_auto_focus_can_be_prevented_and_redirected() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_clicked = app.models_mut().insert(false);
        let trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let underlay_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_handler = calls.clone();
        let underlay_id_out_for_handler = underlay_id_out.clone();
        let handler: OnCloseAutoFocus = Arc::new(move |host, _action_cx, req| {
            calls_for_handler.fetch_add(1, Ordering::SeqCst);
            if let Some(underlay) = underlay_id_out_for_handler.get() {
                host.request_focus(underlay);
            }
            req.prevent_default();
        });

        let _root = render_frame_focusable_trigger_with_underlay_and_entries_and_auto_focus_hooks(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked.clone(),
            vec![ContextMenuEntry::Item(ContextMenuItem::new("Alpha"))],
            trigger_id_out.clone(),
            underlay_id_out.clone(),
            None,
            Some(handler.clone()),
        );

        let trigger_id = trigger_id_out.get().expect("trigger element id");
        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger_id).expect("trigger");
        let underlay_id = underlay_id_out.get().expect("underlay element id");
        let underlay_node =
            fret_ui::elements::node_for_element(&mut app, window, underlay_id).expect("underlay");
        ui.set_focus(Some(trigger_node));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::F10,
                modifiers: Modifiers {
                    shift: true,
                    ..Default::default()
                },
                repeat: false,
            },
        );

        let _root = render_frame_focusable_trigger_with_underlay_and_entries_and_auto_focus_hooks(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked.clone(),
            vec![ContextMenuEntry::Item(ContextMenuItem::new("Alpha"))],
            trigger_id_out.clone(),
            underlay_id_out.clone(),
            None,
            Some(handler.clone()),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Escape,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let _root = render_frame_focusable_trigger_with_underlay_and_entries_and_auto_focus_hooks(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked,
            vec![ContextMenuEntry::Item(ContextMenuItem::new("Alpha"))],
            trigger_id_out,
            underlay_id_out,
            None,
            Some(handler),
        );
        assert_eq!(app.models().get_copied(&open), Some(false));
        assert!(
            calls.load(Ordering::SeqCst) > 0,
            "expected on_close_auto_focus to run"
        );
        assert_eq!(
            ui.focus(),
            Some(underlay_node),
            "expected preventDefault close autofocus to allow redirecting focus"
        );
    }

    #[test]
    fn context_menu_pointer_open_focuses_content_not_first_item() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        // First frame: build the tree and establish stable trigger bounds.
        let root = render_frame_focusable_trigger(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
        );

        let trigger = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable trigger");
        ui.set_focus(Some(trigger));

        let trigger_bounds = ui.debug_node_bounds(trigger).expect("trigger bounds");
        let position = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 / 2.0),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 / 2.0),
        );

        // Right-click to open the context menu.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position,
                button: fret_core::MouseButton::Right,
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
                position,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        // Second frame: ContextMenu emits its OverlayRequest while rendering.
        let _ =
            render_frame_focusable_trigger(&mut ui, &mut app, &mut services, window, bounds, open);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let alpha = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Alpha"))
            .expect("Alpha menu item");

        let focus = ui.focus().expect("expected focus after pointer-open");
        assert_ne!(
            focus, alpha.id,
            "pointer-open should not move focus to the first menu item (Radix onEntryFocus preventDefault)"
        );
        assert_ne!(
            focus, trigger,
            "pointer-open should focus menu content/roving container rather than keeping trigger focus"
        );
    }

    #[test]
    fn context_menu_ignores_right_mouse_up_directly_under_anchor_before_move() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let root = render_frame_focusable_trigger(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
        );

        let trigger = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable trigger");
        ui.set_focus(Some(trigger));

        let trigger_bounds = ui.debug_node_bounds(trigger).expect("trigger bounds");
        let anchor_pos = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 / 2.0),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 / 2.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: anchor_pos,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        let effects = app.flush_effects();
        let arm_token = effects.iter().find_map(|effect| match effect {
            Effect::SetTimer { token, after, .. } if *after == CONTEXT_MENU_CANCEL_OPEN_DELAY => {
                Some(*token)
            }
            _ => None,
        });
        let Some(arm_token) = arm_token else {
            panic!("expected cancel-open arm timer; effects={effects:?}");
        };

        let _ = render_frame_focusable_trigger(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
        );

        ui.dispatch_event(&mut app, &mut services, &Event::Timer { token: arm_token });

        let _ = render_frame_focusable_trigger(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
        );

        // Right mouseup at the original anchor should be ignored (do not close a freshly opened menu).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: anchor_pos,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        let _ = render_frame_focusable_trigger(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
        );
        assert_eq!(
            app.models().get_copied(&open),
            Some(true),
            "right mouseup at anchor should not close freshly opened context menu"
        );
    }

    #[test]
    fn context_menu_cancel_open_state_marks_moved_after_pointer_leaves_anchor() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let debug_state_out: Rc<Cell<Option<ContextMenuCancelOpenShared>>> =
            Rc::new(Cell::new(None));
        let root = render_frame_focusable_trigger_with_debug_cancel_open(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            debug_state_out.clone(),
        );

        let trigger = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable trigger");
        ui.set_focus(Some(trigger));

        let trigger_bounds = ui.debug_node_bounds(trigger).expect("trigger bounds");
        let anchor_pos = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 / 2.0),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 / 2.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: anchor_pos,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        let effects = app.flush_effects();
        let arm_token = effects.iter().find_map(|effect| match effect {
            Effect::SetTimer { token, after, .. } if *after == CONTEXT_MENU_CANCEL_OPEN_DELAY => {
                Some(*token)
            }
            _ => None,
        });
        let Some(arm_token) = arm_token else {
            panic!("expected cancel-open arm timer; effects={effects:?}");
        };

        let _ = render_frame_focusable_trigger_with_debug_cancel_open(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            debug_state_out.clone(),
        );

        ui.dispatch_event(&mut app, &mut services, &Event::Timer { token: arm_token });

        let _ = render_frame_focusable_trigger_with_debug_cancel_open(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            debug_state_out.clone(),
        );

        let moved_pos = Point::new(Px(anchor_pos.x.0 + 6.0), Px(anchor_pos.y.0 + 6.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: moved_pos,
                buttons: fret_core::MouseButtons {
                    right: true,
                    ..Default::default()
                },
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        let debug_shared = debug_state_out
            .replace(None)
            .expect("debug cancel-open shared state should be available");
        let state = context_menu_cancel_open_debug_state(&debug_shared);
        assert_eq!(
            state.moved_from_anchor, true,
            "pointer move away from anchor should mark cancel-open state as moved"
        );
    }

    fn context_menu_cancel_open_debug_state(
        shared: &ContextMenuCancelOpenShared,
    ) -> ContextMenuCancelOpenState {
        *shared.lock().unwrap_or_else(|e| e.into_inner())
    }

    #[test]
    fn context_menu_closes_on_right_mouse_up_after_pointer_moves_away_from_anchor() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let root = render_frame_focusable_trigger(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
        );

        let trigger = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable trigger");
        ui.set_focus(Some(trigger));

        let trigger_bounds = ui.debug_node_bounds(trigger).expect("trigger bounds");
        let anchor_pos = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 / 2.0),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 / 2.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: anchor_pos,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        let effects = app.flush_effects();
        let arm_token = effects.iter().find_map(|effect| match effect {
            Effect::SetTimer { token, after, .. } if *after == CONTEXT_MENU_CANCEL_OPEN_DELAY => {
                Some(*token)
            }
            _ => None,
        });
        let Some(arm_token) = arm_token else {
            panic!("expected cancel-open arm timer; effects={effects:?}");
        };

        let _ = render_frame_focusable_trigger(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
        );

        ui.dispatch_event(&mut app, &mut services, &Event::Timer { token: arm_token });

        let _ = render_frame_focusable_trigger(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
        );

        let moved_pos = Point::new(Px(anchor_pos.x.0 + 6.0), Px(anchor_pos.y.0 + 6.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: moved_pos,
                buttons: fret_core::MouseButtons {
                    right: true,
                    ..Default::default()
                },
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: moved_pos,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        let _ = render_frame_focusable_trigger(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
        );

        assert_eq!(
            app.models().get_copied(&open),
            Some(false),
            "right mouseup after moving away from anchor should close context menu"
        );
    }

    #[test]
    fn context_menu_modal_outside_press_closes_without_activating_underlay() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_clicked = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        // Frame 1: build the tree and establish stable trigger bounds.
        let _root = render_frame_focusable_trigger_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked.clone(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("trigger"))
            .map(|n| n.id)
            .expect("trigger node");
        ui.set_focus(Some(trigger));

        let trigger_bounds = ui.debug_node_bounds(trigger).expect("trigger bounds");
        let trigger_pos = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 / 2.0),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 / 2.0),
        );

        // Right-click to open the context menu (modal=true by default).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: trigger_pos,
                button: fret_core::MouseButton::Right,
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
                position: trigger_pos,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        // Frame 2: open, ensure occlusion is active.
        let _ = render_frame_focusable_trigger_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked.clone(),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));
        let occlusion = fret_ui_kit::OverlayController::arbitration_snapshot(&ui).pointer_occlusion;
        assert_eq!(
            occlusion,
            fret_ui::tree::PointerOcclusion::BlockMouseExceptScroll,
            "expected modal context menu to install pointer occlusion"
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let underlay_node = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("underlay"))
            .map(|n| n.id)
            .expect("underlay node");

        // Click the underlay: should close the menu, but must not activate/focus underlay.
        let underlay_pos = Point::new(Px(10.0), Px(230.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_ne!(
            ui.captured(),
            Some(underlay_node),
            "expected modal context menu to block underlay pointer capture on pointer-down"
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert_eq!(app.models().get_copied(&open), Some(false));
        assert_eq!(app.models().get_copied(&underlay_clicked), Some(false));
        assert_ne!(ui.focus(), Some(underlay_node));
    }

    #[test]
    fn context_menu_click_through_outside_press_closes_and_focuses_underlay() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_clicked = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        // Frame 1: build the tree and establish stable trigger bounds.
        let _root = render_frame_focusable_trigger_with_underlay_modal_and_dismiss_handler(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            false,
            underlay_clicked.clone(),
            None,
        );
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("trigger"))
            .map(|n| n.id)
            .expect("trigger node");
        ui.set_focus(Some(trigger));

        let trigger_bounds = ui.debug_node_bounds(trigger).expect("trigger bounds");
        let trigger_pos = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 / 2.0),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 / 2.0),
        );
        let hit = ui
            .debug_hit_test(trigger_pos)
            .hit
            .expect("expected a hit at trigger_pos");
        let hit_path = ui.debug_node_path(hit);
        assert!(
            hit_path.contains(&trigger),
            "expected trigger_pos to hit inside the trigger subtree; hit={hit:?} hit_path={hit_path:?} trigger={trigger:?}"
        );

        // Right-click to open the context menu (modal=false => click-through).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: trigger_pos,
                button: fret_core::MouseButton::Right,
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
                position: trigger_pos,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        // Frame 2: open, ensure occlusion is not installed.
        let _ = render_frame_focusable_trigger_with_underlay_modal_and_dismiss_handler(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            false,
            underlay_clicked.clone(),
            None,
        );
        assert_eq!(app.models().get_copied(&open), Some(true));
        let occlusion = fret_ui_kit::OverlayController::arbitration_snapshot(&ui).pointer_occlusion;
        assert_eq!(
            occlusion,
            fret_ui::tree::PointerOcclusion::None,
            "expected click-through context menu to not install pointer occlusion"
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let underlay_node = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("underlay"))
            .map(|n| n.id)
            .expect("underlay node");

        // Click the underlay: should close via outside-press observer and remain click-through.
        let underlay_pos = Point::new(Px(10.0), Px(230.0));
        let hit = ui.debug_hit_test(underlay_pos).hit;
        if hit != Some(underlay_node) {
            let underlay_bounds = ui.debug_node_bounds(underlay_node);
            let hit_bounds = hit.and_then(|n| ui.debug_node_bounds(n));
            panic!(
                "expected click-through underlay_pos to hit the underlay; hit={hit:?} hit_bounds={hit_bounds:?} underlay_bounds={underlay_bounds:?}"
            );
        }
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
                button: fret_core::MouseButton::Left,
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
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(
            ui.captured(),
            None,
            "expected underlay pressable to release capture on pointer-up"
        );

        assert_eq!(app.models().get_copied(&open), Some(false));
        assert_eq!(app.models().get_copied(&underlay_clicked), Some(true));
        assert_eq!(
            ui.focus(),
            Some(underlay_node),
            "expected focus to move to underlay after click-through dismissal"
        );
    }

    #[test]
    fn context_menu_click_through_outside_press_can_be_prevented_and_still_activates_underlay() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_clicked = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let dismiss_calls = Arc::new(AtomicUsize::new(0));
        let dismiss_calls_for_handler = dismiss_calls.clone();
        let handler: OnDismissRequest = Arc::new(move |_host, _action_cx, req| {
            dismiss_calls_for_handler.fetch_add(1, Ordering::SeqCst);
            req.prevent_default();
        });

        // Frame 1: build the tree and establish stable trigger bounds.
        let _root = render_frame_focusable_trigger_with_underlay_modal_and_dismiss_handler(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            false,
            underlay_clicked.clone(),
            Some(handler.clone()),
        );
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("trigger"))
            .map(|n| n.id)
            .expect("trigger node");
        ui.set_focus(Some(trigger));

        let trigger_bounds = ui.debug_node_bounds(trigger).expect("trigger bounds");
        let trigger_pos = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 / 2.0),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 / 2.0),
        );

        // Right-click to open the context menu (modal=false => click-through).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: trigger_pos,
                button: fret_core::MouseButton::Right,
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
                position: trigger_pos,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        // Frame 2: open.
        let _ = render_frame_focusable_trigger_with_underlay_modal_and_dismiss_handler(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            false,
            underlay_clicked.clone(),
            Some(handler),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));

        // Click the underlay: click-through should still activate the underlay, but the menu
        // should remain open since dismissal was prevented.
        let underlay_pos = Point::new(Px(10.0), Px(230.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
                button: fret_core::MouseButton::Left,
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
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert_eq!(app.models().get_copied(&open), Some(true));
        assert_eq!(app.models().get_copied(&underlay_clicked), Some(true));
        assert!(dismiss_calls.load(Ordering::SeqCst) > 0);
    }

    #[test]
    fn context_menu_close_transition_is_click_through_and_drops_pointer_occlusion() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_clicked = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        // Frame 1: build the tree and establish stable trigger bounds.
        let _root = render_frame_focusable_trigger_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked.clone(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("trigger"))
            .map(|n| n.id)
            .expect("trigger node");
        ui.set_focus(Some(trigger));

        let trigger_bounds = ui.debug_node_bounds(trigger).expect("trigger bounds");
        let trigger_pos = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 / 2.0),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 / 2.0),
        );

        // Right-click to open the context menu (modal=true by default).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: trigger_pos,
                button: fret_core::MouseButton::Right,
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
                position: trigger_pos,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        // Frame 2: open, ensure occlusion is active.
        let _ = render_frame_focusable_trigger_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked.clone(),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));
        let occlusion = fret_ui_kit::OverlayController::arbitration_snapshot(&ui).pointer_occlusion;
        assert_eq!(
            occlusion,
            fret_ui::tree::PointerOcclusion::BlockMouseExceptScroll,
            "expected modal context menu to install pointer occlusion"
        );

        let overlay_id = OverlayController::stack_snapshot_for_window(&ui, &mut app, window)
            .topmost_popover
            .expect("expected an open context menu overlay");
        let overlay_root_name = menu::context_menu_root_name(overlay_id);
        let overlay_root = fret_ui::elements::global_root(window, &overlay_root_name);
        let overlay_node =
            fret_ui::elements::node_for_element(&mut app, window, overlay_root).expect("overlay");
        let overlay_layer = ui.node_layer(overlay_node).expect("overlay layer");

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let underlay_node = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("underlay"))
            .map(|n| n.id)
            .expect("underlay node");

        // Click the underlay: should close the menu, but must not activate/focus underlay.
        let underlay_pos = Point::new(Px(10.0), Px(230.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
                button: fret_core::MouseButton::Left,
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
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(app.models().get_copied(&open), Some(false));
        assert_eq!(app.models().get_copied(&underlay_clicked), Some(false));
        assert_ne!(ui.focus(), Some(underlay_node));

        // Frame 3: close transition should drop pointer occlusion and become click-through.
        let _ = render_frame_focusable_trigger_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked.clone(),
        );
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            snap.nodes
                .iter()
                .any(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Alpha")),
            "expected menu content to remain present during close transition"
        );

        let occlusion = fret_ui_kit::OverlayController::arbitration_snapshot(&ui).pointer_occlusion;
        assert_eq!(
            occlusion,
            fret_ui::tree::PointerOcclusion::None,
            "expected close transition to drop pointer occlusion (click-through)"
        );

        let info = ui
            .debug_layers_in_paint_order()
            .into_iter()
            .find(|l| l.id == overlay_layer)
            .expect("overlay layer info");
        assert!(info.visible);
        assert!(!info.hit_testable);
        assert!(!info.wants_pointer_move_events);
        assert!(!info.wants_timer_events);

        // Click again while the menu is still present: must activate/focus the underlay now.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
                button: fret_core::MouseButton::Left,
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
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(app.models().get_copied(&open), Some(false));
        assert_eq!(app.models().get_copied(&underlay_clicked), Some(true));
        assert_eq!(ui.focus(), Some(underlay_node));
    }

    #[test]
    fn context_menu_close_transition_does_not_drive_submenu_timers() {
        use fret_runtime::Effect;

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_clicked = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(500.0), Px(280.0)),
        );
        let mut services = FakeServices::default();

        let build_entries = || {
            vec![
                ContextMenuEntry::Item(ContextMenuItem::new("More").submenu(vec![
                    ContextMenuEntry::Item(ContextMenuItem::new("Sub Alpha")),
                    ContextMenuEntry::Item(ContextMenuItem::new("Sub Beta")),
                ])),
                ContextMenuEntry::Item(ContextMenuItem::new("Other")),
            ]
        };

        // Frame 1: build the tree and establish stable trigger bounds.
        let _root = render_frame_focusable_trigger_with_underlay_and_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked.clone(),
            build_entries(),
        );
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("trigger"))
            .map(|n| n.id)
            .expect("trigger node");
        ui.set_focus(Some(trigger));

        let trigger_bounds = ui.debug_node_bounds(trigger).expect("trigger bounds");
        let trigger_pos = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 / 2.0),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 / 2.0),
        );

        // Right-click to open the context menu (modal=true by default).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: trigger_pos,
                button: fret_core::MouseButton::Right,
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
                position: trigger_pos,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        // Frame 2: open and locate the submenu trigger.
        let _ = render_frame_focusable_trigger_with_underlay_and_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked.clone(),
            build_entries(),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let more = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("More"))
            .expect("More menu item");
        let more_center = Point::new(
            Px(more.bounds.origin.x.0 + more.bounds.size.width.0 / 2.0),
            Px(more.bounds.origin.y.0 + more.bounds.size.height.0 / 2.0),
        );

        let overlay_id = OverlayController::stack_snapshot_for_window(&ui, &mut app, window)
            .topmost_popover
            .expect("expected an open context menu overlay");
        let overlay_root_name = menu::context_menu_root_name(overlay_id);
        let overlay_root = fret_ui::elements::global_root(window, &overlay_root_name);
        let overlay_node =
            fret_ui::elements::node_for_element(&mut app, window, overlay_root).expect("overlay");
        let overlay_layer = ui.node_layer(overlay_node).expect("overlay layer");

        // Close via outside click to enter the close transition (present=true, interactive=false).
        let underlay_pos = Point::new(Px(10.0), Px(230.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
                button: fret_core::MouseButton::Left,
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
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(app.models().get_copied(&open), Some(false));

        // Frame 3: close transition should be click-through and must not drive hover intent/timers.
        let _ = render_frame_focusable_trigger_with_underlay_and_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked,
            build_entries(),
        );
        let _ = app.flush_effects();

        let occlusion = fret_ui_kit::OverlayController::arbitration_snapshot(&ui).pointer_occlusion;
        assert_eq!(
            occlusion,
            fret_ui::tree::PointerOcclusion::None,
            "expected close transition to be click-through"
        );

        let info = ui
            .debug_layers_in_paint_order()
            .into_iter()
            .find(|l| l.id == overlay_layer)
            .expect("overlay layer info");
        assert!(info.visible);
        assert!(!info.hit_testable);
        assert!(!info.wants_pointer_move_events);
        assert!(!info.wants_timer_events);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: more_center,
                buttons: fret_core::MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let effects = app.flush_effects();
        let cfg = menu::sub::MenuSubmenuConfig::default();
        assert!(
            !effects
                .iter()
                .any(|e| matches!(e, Effect::SetTimer { after, .. } if *after == cfg.open_delay)),
            "expected close transition pointer move to not arm open-delay timer; effects={effects:?} pos={more_center:?} open_delay={:?}",
            cfg.open_delay
        );
        assert!(
            !effects
                .iter()
                .any(|e| matches!(e, Effect::SetTimer { after, .. } if *after == cfg.close_delay)),
            "expected close transition pointer move to not arm close-delay timer; effects={effects:?} pos={more_center:?} close_delay={:?}",
            cfg.close_delay
        );
    }

    #[test]
    fn context_menu_submenu_safe_hover_corridor_cancels_close_timer_under_pointer_occlusion() {
        use fret_runtime::Effect;

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(500.0), Px(280.0)),
        );
        let mut services = FakeServices::default();

        let build_entries = || {
            vec![
                ContextMenuEntry::Item(ContextMenuItem::new("More").submenu(vec![
                    ContextMenuEntry::Item(ContextMenuItem::new("Sub Alpha")),
                    ContextMenuEntry::Item(ContextMenuItem::new("Sub Beta")),
                ])),
                ContextMenuEntry::Item(ContextMenuItem::new("Other")),
            ]
        };

        // Frame 1: build the tree and establish stable trigger bounds.
        let root = render_frame_focusable_trigger_with_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            build_entries(),
        );
        let trigger = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable trigger");
        ui.set_focus(Some(trigger));

        let trigger_bounds = ui.debug_node_bounds(trigger).expect("trigger bounds");
        let trigger_pos = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 / 2.0),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 / 2.0),
        );

        // Right-click to open the context menu (modal=true by default).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: trigger_pos,
                button: fret_core::MouseButton::Right,
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
                position: trigger_pos,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        // Frame 2: open, ensure occlusion is active.
        let _root = render_frame_focusable_trigger_with_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            build_entries(),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));
        let occlusion = fret_ui_kit::OverlayController::arbitration_snapshot(&ui).pointer_occlusion;
        assert_eq!(
            occlusion,
            fret_ui::tree::PointerOcclusion::BlockMouseExceptScroll,
            "expected modal context menu to install pointer occlusion"
        );

        // Hover "More" to arm the submenu open-delay timer.
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let more = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("More"))
            .expect("More menu item");
        let more_bounds = more.bounds;
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(
                    Px(more_bounds.origin.x.0 + more_bounds.size.width.0 / 2.0),
                    Px(more_bounds.origin.y.0 + more_bounds.size.height.0 / 2.0),
                ),
                buttons: fret_core::MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        let effects = app.flush_effects();
        let open_delay = menu::sub::MenuSubmenuConfig::default().open_delay;
        let open_timer = effects.iter().find_map(|e| match e {
            Effect::SetTimer { token, after, .. } if *after == open_delay => Some(*token),
            _ => None,
        });
        let Some(open_timer) = open_timer else {
            panic!("expected submenu open-delay timer effect; effects={effects:?}");
        };
        ui.dispatch_event(&mut app, &mut services, &Event::Timer { token: open_timer });

        // Frame 3: after open timer fires, the submenu opens.
        let _root = render_frame_focusable_trigger_with_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            build_entries(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let sub_alpha = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Sub Alpha"))
            .expect("Sub Alpha menu item");
        let sub_beta = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Sub Beta"))
            .expect("Sub Beta menu item");

        let submenu_bounds = Rect::new(
            Point::new(
                Px(sub_alpha.bounds.origin.x.0.min(sub_beta.bounds.origin.x.0)),
                Px(sub_alpha.bounds.origin.y.0.min(sub_beta.bounds.origin.y.0)),
            ),
            Size::new(
                Px(
                    (sub_alpha.bounds.origin.x.0 + sub_alpha.bounds.size.width.0)
                        .max(sub_beta.bounds.origin.x.0 + sub_beta.bounds.size.width.0)
                        - sub_alpha.bounds.origin.x.0.min(sub_beta.bounds.origin.x.0),
                ),
                Px(
                    (sub_alpha.bounds.origin.y.0 + sub_alpha.bounds.size.height.0)
                        .max(sub_beta.bounds.origin.y.0 + sub_beta.bounds.size.height.0)
                        - sub_alpha.bounds.origin.y.0.min(sub_beta.bounds.origin.y.0),
                ),
            ),
        );

        let cfg = menu::sub::MenuSubmenuConfig::default();
        let close_delay = cfg.close_delay;
        let grace_geometry = menu::pointer_grace_intent::PointerGraceIntentGeometry {
            reference: more_bounds,
            floating: submenu_bounds,
        };

        // Pick a safe corridor point on the submenu side (to the right) so moving towards it can
        // cancel a pending close timer (Radix pointer-grace intent).
        let reference_right = more_bounds.origin.x.0 + more_bounds.size.width.0;
        let mut safe_point: Option<Point> = None;
        for y in (0..=bounds.size.height.0 as i32).step_by(2) {
            for x in (0..=bounds.size.width.0 as i32).step_by(2) {
                let pos = Point::new(Px(x as f32), Px(y as f32));
                if pos.x.0 <= reference_right {
                    continue;
                }
                if more_bounds.contains(pos) || submenu_bounds.contains(pos) {
                    continue;
                }
                if !menu::pointer_grace_intent::last_pointer_is_safe(
                    pos,
                    grace_geometry,
                    cfg.safe_hover_buffer,
                ) {
                    continue;
                }
                safe_point = Some(pos);
                break;
            }
            if safe_point.is_some() {
                break;
            }
        }
        let safe_point = safe_point.unwrap_or_else(|| {
            panic!(
                "failed to find safe corridor point; more={more_bounds:?} submenu={submenu_bounds:?} geometry={grace_geometry:?}"
            )
        });

        // Pick an unsafe point to the left of the safe point, so moving to `safe_point` is
        // directionally towards the submenu (x increases).
        let mut unsafe_point: Option<Point> = None;
        for y in (0..=bounds.size.height.0 as i32).step_by(4) {
            for x in (0..=bounds.size.width.0 as i32).step_by(4) {
                let pos = Point::new(Px(x as f32), Px(y as f32));
                if pos.x.0 >= safe_point.x.0 {
                    continue;
                }
                if more_bounds.contains(pos) || submenu_bounds.contains(pos) {
                    continue;
                }
                if menu::pointer_grace_intent::last_pointer_is_safe(
                    pos,
                    grace_geometry,
                    cfg.safe_hover_buffer,
                ) {
                    continue;
                }
                unsafe_point = Some(pos);
                break;
            }
            if unsafe_point.is_some() {
                break;
            }
        }
        let unsafe_point = unsafe_point.unwrap_or_else(|| {
            panic!(
                "failed to find unsafe point; safe_point={safe_point:?} more={more_bounds:?} submenu={submenu_bounds:?} geometry={grace_geometry:?}",
            )
        });

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: unsafe_point,
                buttons: fret_core::MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let effects = app.flush_effects();
        let close_timer = effects.iter().find_map(|e| match e {
            Effect::SetTimer { token, after, .. } if *after == close_delay => Some(*token),
            _ => None,
        });
        let Some(close_timer) = close_timer else {
            panic!(
                "expected unsafe pointer move to arm close-delay timer; effects={effects:?} unsafe_point={unsafe_point:?} close_delay={close_delay:?}"
            );
        };

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: safe_point,
                buttons: fret_core::MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let effects = app.flush_effects();
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, Effect::CancelTimer { token } if *token == close_timer)),
            "expected safe corridor pointer move to cancel close-delay timer; effects={effects:?} safe_point={safe_point:?} close_timer={close_timer:?}"
        );

        // Sanity: no new close timer should be armed when safe.
        assert!(
            !effects
                .iter()
                .any(|e| matches!(e, Effect::SetTimer { after, .. } if *after == close_delay)),
            "expected safe corridor pointer move to not arm a new close-delay timer; effects={effects:?} safe_point={safe_point:?} close_delay={close_delay:?}"
        );
    }

    #[test]
    fn context_menu_items_have_collection_position_metadata_excluding_separators() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        // First frame: establish stable trigger bounds.
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
        );

        let _ = app.models_mut().update(&open, |v| *v = true);

        // Second frame: open the menu and verify item metadata.
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let beta = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Beta"))
            .expect("Beta menu item");
        assert_eq!(beta.pos_in_set, Some(2));
        assert_eq!(beta.set_size, Some(3));
    }

    #[test]
    fn context_menu_submenu_opens_on_arrow_right_without_pointer_move() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let build_entries = || {
            vec![ContextMenuEntry::Item(
                ContextMenuItem::new("More").submenu(vec![
                    ContextMenuEntry::Item(ContextMenuItem::new("Sub Alpha")),
                    ContextMenuEntry::Item(ContextMenuItem::new("Sub Beta")),
                ]),
            )]
        };

        let _ = render_frame_focusable_trigger_with_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            build_entries(),
        );

        let _ = app.models_mut().update(&open, |v| *v = true);
        let _ = render_frame_focusable_trigger_with_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            build_entries(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let more = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("More"))
            .expect("More menu item");
        ui.set_focus(Some(more.id));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::ArrowRight,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let _ = render_frame_focusable_trigger_with_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open,
            build_entries(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            snap.nodes.iter().any(|n| {
                n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Sub Alpha")
            }),
            "submenu items should render after ArrowRight opens the submenu"
        );
    }
}

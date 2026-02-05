//! Material 3 autocomplete (MVP).
//!
//! Outcome-oriented implementation:
//! - focus stays on the text input (combobox pattern),
//! - options are exposed via `active_descendant` (ADR 0073),
//! - overlay is click-through (popover-like) so the input remains interactable while open.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{
    AttributedText, Axis, Edges, FontWeight, KeyCode, Px, SemanticsRole, Size, TextOverflow,
    TextSpan, TextWrap,
};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, Length, MainAlign, Overflow, PressableA11y,
    PressableProps, ScrollProps, SemanticsProps, StyledTextProps, TextProps,
};
use fret_ui::elements::{ElementContext, GlobalElementId};
use fret_ui::overlay_placement::{Align, Side};
use fret_ui::{Invalidation, Theme, UiHost};
use fret_ui_kit::primitives::active_descendant::{
    active_descendant_for_index, active_option_for_index,
};
use fret_ui_kit::primitives::direction as direction_prim;
use fret_ui_kit::primitives::popper;
use fret_ui_kit::primitives::popper_content;
use fret_ui_kit::{OverlayController, OverlayPresence};

use crate::foundation::overlay_motion::drive_overlay_open_close_motion;
use crate::foundation::surface::material_surface_style;
use crate::motion::ms_to_frames;
use crate::text_field::{TextField, TextFieldTokenNamespace, TextFieldVariant};
use crate::tokens::autocomplete as autocomplete_tokens;
use crate::tokens::dropdown_menu as dropdown_menu_tokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AutocompleteVariant {
    #[default]
    Outlined,
    Filled,
}

impl AutocompleteVariant {
    fn as_text_field_variant(self) -> TextFieldVariant {
        match self {
            AutocompleteVariant::Outlined => TextFieldVariant::Outlined,
            AutocompleteVariant::Filled => TextFieldVariant::Filled,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AutocompleteItem {
    pub value: Arc<str>,
    pub label: Arc<str>,
    pub disabled: bool,
    pub test_id: Option<Arc<str>>,
}

impl AutocompleteItem {
    pub fn new(value: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
            test_id: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }
}

#[derive(Clone)]
pub struct Autocomplete {
    model: Model<String>,
    items: Arc<[AutocompleteItem]>,
    variant: AutocompleteVariant,
    open_on_focus: bool,
    disabled: bool,
    error: bool,
    label: Option<Arc<str>>,
    placeholder: Option<Arc<str>>,
    supporting_text: Option<Arc<str>>,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for Autocomplete {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Autocomplete")
            .field("variant", &self.variant)
            .field("disabled", &self.disabled)
            .field("error", &self.error)
            .field("label", &self.label)
            .field("placeholder", &self.placeholder)
            .field("supporting_text", &self.supporting_text)
            .field("a11y_label", &self.a11y_label)
            .field("test_id", &self.test_id)
            .finish()
    }
}

impl Autocomplete {
    pub fn new(model: Model<String>) -> Self {
        Self {
            model,
            items: Arc::from([]),
            variant: AutocompleteVariant::default(),
            open_on_focus: true,
            disabled: false,
            error: false,
            label: None,
            placeholder: None,
            supporting_text: None,
            a11y_label: None,
            test_id: None,
        }
    }

    pub fn items(mut self, items: impl Into<Arc<[AutocompleteItem]>>) -> Self {
        self.items = items.into();
        self
    }

    pub fn variant(mut self, variant: AutocompleteVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn open_on_focus(mut self, open_on_focus: bool) -> Self {
        self.open_on_focus = open_on_focus;
        self
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn supporting_text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.supporting_text = Some(text.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        autocomplete_into_element(cx, self)
    }
}

#[derive(Clone)]
struct AutocompleteRuntimeModels {
    open: Model<bool>,
    suppress_open: Model<bool>,
    active_index: Model<Option<usize>>,
    scroll_handle: fret_ui::scroll::ScrollHandle,
    input_element_id: Rc<Cell<Option<GlobalElementId>>>,
    listbox_element_id: Rc<Cell<Option<GlobalElementId>>>,
    option_elements: Rc<RefCell<Vec<GlobalElementId>>>,
    scroll_viewport_id: Rc<Cell<Option<GlobalElementId>>>,
}

fn autocomplete_runtime_models<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> AutocompleteRuntimeModels {
    #[derive(Default)]
    struct State {
        models: Option<AutocompleteRuntimeModels>,
    }

    let existing = cx.with_state(State::default, |st| st.models.clone());
    if let Some(models) = existing {
        return models;
    }

    let models = AutocompleteRuntimeModels {
        open: cx.app.models_mut().insert(false),
        suppress_open: cx.app.models_mut().insert(false),
        active_index: cx.app.models_mut().insert(None),
        scroll_handle: fret_ui::scroll::ScrollHandle::default(),
        input_element_id: Rc::new(Cell::new(None)),
        listbox_element_id: Rc::new(Cell::new(None)),
        option_elements: Rc::new(RefCell::new(Vec::new())),
        scroll_viewport_id: Rc::new(Cell::new(None)),
    };

    cx.with_state(State::default, |st| st.models = Some(models.clone()));
    models
}

#[derive(Default)]
struct AutocompleteFrameState {
    last_query: String,
    was_focused_input: bool,
}

fn first_enabled(items: &[AutocompleteItem]) -> Option<usize> {
    items.iter().position(|it| !it.disabled)
}

fn next_enabled(items: &[AutocompleteItem], start: usize, forward: bool) -> Option<usize> {
    if items.is_empty() {
        return None;
    }

    let len = items.len();
    for step in 1..=len {
        let idx = if forward {
            (start + step) % len
        } else {
            (start + len - step) % len
        };
        if !items[idx].disabled {
            return Some(idx);
        }
    }
    None
}

fn filter_items(query: &str, items: &Arc<[AutocompleteItem]>) -> Arc<[AutocompleteItem]> {
    let query = query.trim();
    if query.is_empty() {
        return Arc::clone(items);
    }

    let query_lower = query.to_ascii_lowercase();
    let mut out: Vec<AutocompleteItem> = Vec::new();
    for item in items.iter() {
        let label = item.label.as_ref();
        let value = item.value.as_ref();
        if label.to_ascii_lowercase().contains(&query_lower)
            || value.to_ascii_lowercase().contains(&query_lower)
        {
            out.push(item.clone());
        }
    }
    Arc::from(out)
}

fn sanitize_test_id_suffix(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('-');
        }
    }
    while out.contains("--") {
        out = out.replace("--", "-");
    }
    out.trim_matches('-').to_string()
}

fn find_ascii_case_insensitive_match(label: &str, query: &str) -> Option<(usize, usize)> {
    let query = query.trim();
    if query.is_empty() || !label.is_ascii() || !query.is_ascii() {
        return None;
    }

    let label_lower = label.to_ascii_lowercase();
    let query_lower = query.to_ascii_lowercase();
    let start = label_lower.find(&query_lower)?;
    let end = start.saturating_add(query_lower.len());
    if start <= label.len()
        && end <= label.len()
        && label.is_char_boundary(start)
        && label.is_char_boundary(end)
    {
        Some((start, end))
    } else {
        None
    }
}

fn autocomplete_into_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    autocomplete: Autocomplete,
) -> AnyElement {
    cx.scope(|cx| {
        let theme = Theme::global(&*cx.app).clone();
        let runtime = autocomplete_runtime_models(cx);

        let open_now = cx
            .get_model_copied(&runtime.open, Invalidation::Layout)
            .unwrap_or(false);
        let suppress_open = cx
            .get_model_copied(&runtime.suppress_open, Invalidation::Layout)
            .unwrap_or(false);
        let active_index = cx
            .get_model_cloned(&runtime.active_index, Invalidation::Layout)
            .unwrap_or(None);

        let query = cx
            .get_model_cloned(&autocomplete.model, Invalidation::Layout)
            .unwrap_or_default();

        let filtered_items = filter_items(&query, &autocomplete.items);

        let focused_input = runtime
            .input_element_id
            .get()
            .is_some_and(|id| cx.is_focused_element(id));

        let (query_changed, focus_gained) = cx.with_state(AutocompleteFrameState::default, |st| {
            let changed = st.last_query != query;
            if changed {
                st.last_query = query.clone();
            }
            let focus_gained = focused_input && !st.was_focused_input;
            st.was_focused_input = focused_input;
            (changed, focus_gained)
        });

        if query_changed && suppress_open {
            let _ = cx.app.models_mut().update(&runtime.suppress_open, |v| *v = false);
        }

        let should_open = !autocomplete.disabled
            && focused_input
            && !suppress_open
            && !filtered_items.is_empty()
            && (query_changed || (autocomplete.open_on_focus && focus_gained));
        let opened_due_to_focus = autocomplete.open_on_focus && focus_gained && !query_changed;
        if should_open {
            if !open_now {
                let _ = cx.app.models_mut().update(&runtime.open, |v| *v = true);
                cx.app.request_redraw(cx.window);
            }
            if (active_index.is_none() && !opened_due_to_focus) || query_changed {
                let idx = first_enabled(&filtered_items).unwrap_or(0);
                let _ = cx.app.models_mut().update(&runtime.active_index, |v| *v = Some(idx));
            }
        }

        if open_now && filtered_items.is_empty() {
            let _ = cx.app.models_mut().update(&runtime.open, |v| *v = false);
        }

        let open_now = cx
            .get_model_copied(&runtime.open, Invalidation::Layout)
            .unwrap_or(open_now);
        if !open_now && active_index.is_some() {
            let _ = cx.app.models_mut().update(&runtime.active_index, |v| *v = None);
        }

        let close_grace_frames = Some(ms_to_frames(dropdown_menu_tokens::close_duration_ms(&theme)));
        let motion = drive_overlay_open_close_motion(cx, &theme, open_now, close_grace_frames);
        let overlay_presence = OverlayPresence {
            present: motion.present,
            interactive: open_now,
        };

        if !overlay_presence.present {
            runtime.listbox_element_id.set(None);
            runtime.scroll_viewport_id.set(None);
            runtime.option_elements.borrow_mut().clear();
        }

        let active_descendant = {
            let option_elements = runtime.option_elements.borrow();
            active_descendant_for_index(cx, &option_elements, active_index)
        };

        let controls_element = runtime.listbox_element_id.get().map(|id| id.0);

        let input_id_out = runtime.input_element_id.clone();
        let trigger = TextField::new(autocomplete.model.clone())
            .variant(autocomplete.variant.as_text_field_variant())
            .token_namespace(TextFieldTokenNamespace::Autocomplete)
            .label_opt(autocomplete.label.clone())
            .placeholder_opt(autocomplete.placeholder.clone())
            .supporting_text_opt(autocomplete.supporting_text.clone())
            .disabled(autocomplete.disabled)
            .error(autocomplete.error)
            .a11y_label_opt(autocomplete.a11y_label.clone())
            .test_id_opt(autocomplete.test_id.clone())
            .a11y_role(SemanticsRole::ComboBox)
            .active_descendant(active_descendant)
            .controls_element(controls_element)
            .expanded(Some(open_now))
            .input_id_out(input_id_out.clone())
            .into_element(cx);

        let Some(input_id) = runtime.input_element_id.get() else {
            return trigger;
        };

        install_input_key_handlers(
            cx,
            input_id,
            autocomplete.model.clone(),
            runtime.open.clone(),
            runtime.suppress_open.clone(),
            runtime.active_index.clone(),
            filtered_items.clone(),
            autocomplete.disabled,
        );

        if overlay_presence.present {
            let Some(anchor) = fret_ui_kit::overlay::anchor_bounds_for_element(cx, input_id) else {
                return trigger;
            };

            let outer = fret_ui_kit::overlay::outer_bounds_with_window_margin(cx.bounds, Px(0.0));

            let item_height =
                autocomplete_tokens::menu_list_item_height(&theme, autocomplete.variant.as_text_field_variant());
            let vertical_padding = Px(8.0);
            let desired_width = anchor.size.width;
            let desired_height = Px(
                (item_height.0 * (filtered_items.len().min(6).max(1) as f32))
                    + vertical_padding.0 * 2.0,
            );
            let desired = Size::new(desired_width, desired_height);

            let direction = direction_prim::use_direction_in_scope(cx, None);
            let placement = popper::PopperContentPlacement::new(
                direction,
                Side::Bottom,
                Align::Start,
                Px(4.0),
            )
            .with_collision_padding(Edges {
                left: Px(8.0),
                right: Px(8.0),
                top: Px(48.0),
                bottom: Px(48.0),
            });
            let layout = popper::popper_content_layout_sized(outer, anchor, desired, placement);

            let listbox = popper_content::popper_wrapper_panel_at(
                cx,
                layout.rect,
                Edges::all(Px(0.0)),
                Overflow::Visible,
                {
                    let labelled_by = Some(input_id.0);
                    let listbox_element_id_out = runtime.listbox_element_id.clone();
                    let option_elements_out = runtime.option_elements.clone();
                    let scroll_viewport_id_out = runtime.scroll_viewport_id.clone();
                    let open = runtime.open.clone();
                    let suppress_open = runtime.suppress_open.clone();
                    let active_index = runtime.active_index.clone();
                    let scroll_handle = runtime.scroll_handle.clone();
                    let items = filtered_items.clone();
                    let variant = autocomplete.variant;
                    let a11y_label = autocomplete.a11y_label.clone();
                    let test_id = autocomplete.test_id.clone();
                    let disabled = autocomplete.disabled;
                    let query = Arc::<str>::from(query.clone());

                    move |cx| {
                        vec![autocomplete_listbox_panel(
                            cx,
                            &theme,
                            variant,
                            labelled_by,
                            a11y_label.clone(),
                            test_id.clone(),
                            query.clone(),
                            disabled,
                            open.clone(),
                            suppress_open.clone(),
                            active_index.clone(),
                            scroll_handle.clone(),
                            items.clone(),
                            listbox_element_id_out.clone(),
                            option_elements_out.clone(),
                            scroll_viewport_id_out.clone(),
                            autocomplete.model.clone(),
                        )]
                    }
                },
            );

            let opacity = motion.alpha;
            let scale = motion.scale;
            let origin = popper::popper_content_transform_origin(&layout, anchor, None);
            let origin_inv = fret_core::Point::new(Px(-origin.x.0), Px(-origin.y.0));
            let transform = fret_core::Transform2D::translation(origin)
                * fret_core::Transform2D::scale_uniform(scale)
                * fret_core::Transform2D::translation(origin_inv);
            let overlay_root =
                fret_ui_kit::declarative::overlay_motion::wrap_opacity_and_render_transform_gated(
                    cx,
                    opacity,
                    transform,
                    overlay_presence.interactive,
                    vec![listbox],
                );

            let mut request = fret_ui_kit::overlay_controller::OverlayRequest::dismissible_popover(
                input_id,
                input_id,
                runtime.open.clone(),
                overlay_presence,
                vec![overlay_root],
            );
            request.root_name = Some(format!("material3.autocomplete.{}", input_id.0));
            request.close_on_window_focus_lost = true;
            request.close_on_window_resize = true;

            OverlayController::request(cx, request);

            if open_now {
                let active_idx = cx
                    .get_model_cloned(&runtime.active_index, Invalidation::Layout)
                    .unwrap_or(None);
                let option_elements = runtime.option_elements.borrow();
                if let Some(viewport) = runtime.scroll_viewport_id.get()
                    && let Some(active) =
                        active_option_for_index(cx, &option_elements, active_idx)
                {
                    let _ = fret_ui_kit::primitives::active_descendant::scroll_active_element_into_view_y(
                        cx,
                        &runtime.scroll_handle,
                        viewport,
                        active.element,
                    );
                }
            }
        }

        trigger
    })
}

fn install_input_key_handlers<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    input_id: GlobalElementId,
    model: Model<String>,
    open: Model<bool>,
    suppress_open: Model<bool>,
    active_index: Model<Option<usize>>,
    items: Arc<[AutocompleteItem]>,
    disabled: bool,
) {
    if disabled {
        return;
    }

    cx.key_add_on_key_down_for(
        input_id,
        Arc::new(move |host, action_cx, down| match down.key {
            KeyCode::ArrowDown => {
                let open_now = host.models_mut().get_copied(&open).unwrap_or(false);
                let active = host.models_mut().get_cloned(&active_index).unwrap_or(None);
                let _ = host.models_mut().update(&suppress_open, |v| *v = false);

                if !open_now {
                    let _ = host.models_mut().update(&open, |v| *v = true);
                    let idx = first_enabled(&items).unwrap_or(0);
                    let _ = host.models_mut().update(&active_index, |v| *v = Some(idx));
                    host.request_redraw(action_cx.window);
                    return true;
                }

                let next = match active {
                    None => first_enabled(&items).or(Some(0)),
                    Some(cur) => next_enabled(&items, cur, true).or(Some(cur)),
                };
                let _ = host.models_mut().update(&active_index, |v| *v = next);
                host.request_redraw(action_cx.window);
                true
            }
            KeyCode::ArrowUp => {
                let open_now = host.models_mut().get_copied(&open).unwrap_or(false);
                let active = host.models_mut().get_cloned(&active_index).unwrap_or(None);
                let _ = host.models_mut().update(&suppress_open, |v| *v = false);

                if !open_now {
                    let _ = host.models_mut().update(&open, |v| *v = true);
                    let idx = first_enabled(&items).unwrap_or(0);
                    let _ = host.models_mut().update(&active_index, |v| *v = Some(idx));
                    host.request_redraw(action_cx.window);
                    return true;
                }

                let next = match active {
                    None => first_enabled(&items).or(Some(0)),
                    Some(cur) => next_enabled(&items, cur, false).or(Some(cur)),
                };
                let _ = host.models_mut().update(&active_index, |v| *v = next);
                host.request_redraw(action_cx.window);
                true
            }
            KeyCode::Enter => {
                let open_now = host.models_mut().get_copied(&open).unwrap_or(false);
                if !open_now {
                    return false;
                }
                let active = host.models_mut().get_cloned(&active_index).unwrap_or(None);
                let Some(idx) = active else {
                    return true;
                };
                let Some(item) = items.get(idx) else {
                    return true;
                };
                if item.disabled {
                    return true;
                }

                let label = item.label.clone();
                let _ = host
                    .models_mut()
                    .update(&model, |v| *v = label.as_ref().to_string());
                let _ = host.models_mut().update(&open, |v| *v = false);
                let _ = host.models_mut().update(&suppress_open, |v| *v = true);
                host.request_redraw(action_cx.window);
                true
            }
            KeyCode::Escape => {
                let _ = host.models_mut().update(&open, |v| *v = false);
                let _ = host.models_mut().update(&suppress_open, |v| *v = true);
                host.request_redraw(action_cx.window);
                true
            }
            _ => false,
        }),
    );
}

fn autocomplete_listbox_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    variant: AutocompleteVariant,
    labelled_by_element: Option<u64>,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    query: Arc<str>,
    disabled: bool,
    open: Model<bool>,
    suppress_open: Model<bool>,
    active_index: Model<Option<usize>>,
    scroll_handle: fret_ui::scroll::ScrollHandle,
    items: Arc<[AutocompleteItem]>,
    listbox_element_id_out: Rc<Cell<Option<GlobalElementId>>>,
    option_elements_out: Rc<RefCell<Vec<GlobalElementId>>>,
    scroll_viewport_id_out: Rc<Cell<Option<GlobalElementId>>>,
    model: Model<String>,
) -> AnyElement {
    let listbox_test_id = test_id
        .as_ref()
        .map(|id| Arc::<str>::from(format!("{}-listbox", id)));

    let sem = SemanticsProps {
        role: SemanticsRole::ListBox,
        label: a11y_label.clone(),
        test_id: listbox_test_id
            .or_else(|| Some(Arc::<str>::from("material3-autocomplete-listbox"))),
        labelled_by_element,
        ..Default::default()
    };

    let menu_bg =
        autocomplete_tokens::menu_container_background(theme, variant.as_text_field_variant());
    let elevation =
        autocomplete_tokens::menu_container_elevation(theme, variant.as_text_field_variant());
    let shadow_color =
        autocomplete_tokens::menu_container_shadow_color(theme, variant.as_text_field_variant());
    let corner = autocomplete_tokens::menu_container_shape(theme, variant.as_text_field_variant());
    let surface = material_surface_style(theme, menu_bg, elevation, Some(shadow_color), corner);

    let selected_bg = autocomplete_tokens::menu_list_item_selected_container_color(
        theme,
        variant.as_text_field_variant(),
    );
    let label_style = autocomplete_tokens::menu_list_item_label_text_style(
        theme,
        variant.as_text_field_variant(),
    )
    .unwrap_or_else(|| {
        theme
            .text_style_by_key("md.sys.typescale.body-large")
            .unwrap_or_default()
    });
    let label_color = autocomplete_tokens::menu_list_item_label_text_color(
        theme,
        variant.as_text_field_variant(),
    );

    let item_height =
        autocomplete_tokens::menu_list_item_height(theme, variant.as_text_field_variant());
    let vertical_padding = Px(8.0);

    cx.semantics_with_id(sem, move |cx, listbox_id| {
        listbox_element_id_out.set(Some(listbox_id));
        option_elements_out.borrow_mut().clear();
        scroll_viewport_id_out.set(None);

        let scroll = cx.scroll(
            ScrollProps {
                scroll_handle: Some(scroll_handle.clone()),
                layout: {
                    let mut l = fret_ui::element::LayoutStyle::default();
                    l.size.width = Length::Fill;
                    l.size.height = Length::Fill;
                    l
                },
                ..Default::default()
            },
            move |cx| {
                let active_now = cx
                    .get_model_cloned(&active_index, Invalidation::Layout)
                    .unwrap_or(None);

                let mut props = FlexProps::default();
                props.direction = Axis::Vertical;
                props.gap = Px(0.0);
                props.justify = MainAlign::Start;
                props.align = CrossAlign::Stretch;
                props.layout.size.width = Length::Fill;
                props.padding = Edges::all(vertical_padding);
                vec![cx.flex(props, move |cx| {
                    let mut out: Vec<AnyElement> = Vec::with_capacity(items.len());
                    let count = items.len();

                    for (idx, item) in items.iter().enumerate() {
                        let item_disabled = disabled || item.disabled;
                        let selected = active_now == Some(idx);
                        let label_style = label_style.clone();
                        let query = query.clone();
                        let option_test_id = item.test_id.clone().or_else(|| {
                            test_id.as_ref().map(|parent| {
                                Arc::<str>::from(format!(
                                    "{}-option-{}",
                                    parent,
                                    sanitize_test_id_suffix(item.value.as_ref())
                                ))
                            })
                        });

                        let open_for_select = open.clone();
                        let suppress_open_for_select = suppress_open.clone();
                        let model_for_select = model.clone();
                        let label_for_select = item.label.clone();
                        let active_for_hover = active_index.clone();
                        let option_elements_out = option_elements_out.clone();

                        let row = cx.pressable_with_id_props(|cx, _st, id| {
                            option_elements_out.borrow_mut().push(id);

                            let enabled = !item_disabled;
                            if !item_disabled {
                                let active_for_hover = active_for_hover.clone();
                                cx.pressable_add_on_hover_change(Arc::new(
                                    move |host, action_cx, hovered| {
                                        if !hovered {
                                            return;
                                        }
                                        let _ = host.models_mut().update(&active_for_hover, |v| *v = Some(idx));
                                        host.request_redraw(action_cx.window);
                                    },
                                ));
                            }

                            let open_for_select = open_for_select.clone();
                            let suppress_open_for_select = suppress_open_for_select.clone();
                            let enabled_for_select = enabled;
                            cx.pressable_on_activate(Arc::new(move |host, action_cx, _reason| {
                                if !enabled_for_select {
                                    return;
                                }
                                let _ = host.models_mut().update(&model_for_select, |v| {
                                    *v = label_for_select.as_ref().to_string();
                                });
                                let _ = host.models_mut().update(&open_for_select, |v| *v = false);
                                let _ = host
                                    .models_mut()
                                    .update(&suppress_open_for_select, |v| *v = true);
                                host.request_redraw(action_cx.window);
                            }));

                            use fret_ui_kit::declarative::collection_semantics::CollectionSemanticsExt as _;

                            let a11y = PressableA11y {
                                role: Some(SemanticsRole::ListBoxOption),
                                label: Some(item.label.clone()),
                                test_id: option_test_id.clone(),
                                selected,
                                ..Default::default()
                            }
                            .with_collection_position(idx, count);

                            let mut layout = fret_ui::element::LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Px(item_height);

                            let props = PressableProps {
                                layout,
                                enabled,
                                focusable: false,
                                a11y,
                                ..Default::default()
                            };

                            let mut text_layout = fret_ui::element::LayoutStyle::default();
                            text_layout.size.width = Length::Fill;

                            let text = if let Some((start, end)) =
                                find_ascii_case_insensitive_match(item.label.as_ref(), query.as_ref())
                            {
                                let label = item.label.as_ref();
                                let mut spans: Vec<TextSpan> = Vec::new();
                                if start > 0 {
                                    spans.push(TextSpan::new(start));
                                }
                                if end > start {
                                    let mut span = TextSpan::new(end.saturating_sub(start));
                                    span.shaping = span.shaping.with_weight(FontWeight::SEMIBOLD);
                                    spans.push(span);
                                }
                                if end < label.len() {
                                    spans.push(TextSpan::new(label.len().saturating_sub(end)));
                                }
                                let rich = AttributedText::new(
                                    item.label.clone(),
                                    Arc::<[TextSpan]>::from(spans),
                                );
                                cx.styled_text_props(StyledTextProps {
                                    layout: text_layout,
                                    rich,
                                    wrap: TextWrap::None,
                                    overflow: TextOverflow::Ellipsis,
                                    style: Some(label_style),
                                    color: Some(label_color),
                                })
                            } else {
                                cx.text_props(TextProps {
                                    layout: text_layout,
                                    text: item.label.clone(),
                                    wrap: TextWrap::None,
                                    overflow: TextOverflow::Ellipsis,
                                    style: Some(label_style),
                                    color: Some(label_color),
                                })
                            };

                            let mut child_layout = fret_ui::element::LayoutStyle::default();
                            child_layout.size.width = Length::Fill;
                            child_layout.size.height = Length::Fill;

                            let child = cx.container(
                                ContainerProps {
                                    layout: child_layout,
                                    padding: Edges::all(Px(12.0)),
                                    background: selected.then_some(selected_bg),
                                    ..Default::default()
                                },
                                move |_cx| vec![text],
                            );

                            (props, vec![child])
                        });

                        out.push(row);
                    }
                    out
                })]
            },
        );

        scroll_viewport_id_out.set(Some(scroll.id));

        vec![cx.container(
            ContainerProps {
                background: Some(surface.background),
                shadow: surface.shadow,
                corner_radii: corner,
                layout: {
                    let mut l = fret_ui::element::LayoutStyle::default();
                    l.size.width = Length::Fill;
                    l.size.height = Length::Fill;
                    l.overflow = Overflow::Clip;
                    l
                },
                ..Default::default()
            },
            move |_cx| vec![scroll],
        )]
    })
}

trait TextFieldExt {
    fn label_opt(self, label: Option<Arc<str>>) -> Self;
    fn placeholder_opt(self, placeholder: Option<Arc<str>>) -> Self;
    fn supporting_text_opt(self, text: Option<Arc<str>>) -> Self;
    fn a11y_label_opt(self, label: Option<Arc<str>>) -> Self;
    fn test_id_opt(self, id: Option<Arc<str>>) -> Self;
}

impl TextFieldExt for TextField {
    fn label_opt(self, label: Option<Arc<str>>) -> Self {
        match label {
            Some(v) => self.label(v),
            None => self,
        }
    }

    fn placeholder_opt(self, placeholder: Option<Arc<str>>) -> Self {
        match placeholder {
            Some(v) => self.placeholder(v),
            None => self,
        }
    }

    fn supporting_text_opt(self, text: Option<Arc<str>>) -> Self {
        match text {
            Some(v) => self.supporting_text(v),
            None => self,
        }
    }

    fn a11y_label_opt(self, label: Option<Arc<str>>) -> Self {
        match label {
            Some(v) => self.a11y_label(v),
            None => self,
        }
    }

    fn test_id_opt(self, id: Option<Arc<str>>) -> Self {
        match id {
            Some(v) => self.test_id(v),
            None => self,
        }
    }
}

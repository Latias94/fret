//!
//! Outcome-oriented Material 3 Select (MVP).
//!
//! This intentionally focuses on:
//! - token-driven trigger/container outcomes via `md.comp.{outlined,filled}-select.*`,
//! - a minimal listbox overlay anchored to the trigger,
//! - an ADR 1159-shaped `SelectStyle` override surface.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{
    Axis, Color, Corners, Edges, KeyCode, Point, Px, Rect, SemanticsRole, Size, SvgFit,
    TextOverflow, TextWrap,
};
use fret_icons::{IconId, ids};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, CanvasProps, ContainerProps, CrossAlign, FlexProps, Length, MainAlign, Overflow,
    PointerRegionProps, PressableA11y, PressableProps, RovingFlexProps, ScrollProps,
    SemanticsProps, SvgIconProps, TextProps, VisualTransformProps,
};
use fret_ui::elements::{ElementContext, GlobalElementId};
use fret_ui::overlay_placement::{Align, Side};
use fret_ui::{Invalidation, Theme, UiHost};
use fret_ui_kit::primitives::direction as direction_prim;
use fret_ui_kit::primitives::popper;
use fret_ui_kit::primitives::popper_content;
use fret_ui_kit::{ColorRef, OverlayController, OverlayPresence};
use fret_ui_kit::{
    OverrideSlot, WidgetStateProperty, WidgetStates, merge_override_slot,
    resolve_override_slot_opt_with, resolve_override_slot_with,
};

use crate::foundation::floating_label;
use crate::foundation::icon::svg_source_for_icon;
use crate::foundation::indication::{
    RippleClip, material_ink_layer_for_pressable, material_pressable_indication_config,
};
use crate::foundation::overlay_motion::drive_overlay_open_close_motion;
use crate::foundation::surface::material_surface_style;
use crate::interaction::state_layer::StateLayerAnimator;
use crate::motion::ms_to_frames;
use crate::tokens::dropdown_menu as dropdown_menu_tokens;
use crate::tokens::list as list_tokens;
use crate::tokens::select as select_tokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectVariant {
    #[default]
    Outlined,
    Filled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectMenuAlign {
    #[default]
    Start,
    End,
}

#[derive(Debug, Clone)]
pub struct SelectItem {
    pub value: Arc<str>,
    pub label: Arc<str>,
    /// Optional supporting text shown under the label in the listbox.
    ///
    /// Material Web exposes this via the `"supporting-text"` slot on `md-select-option`.
    pub supporting_text: Option<Arc<str>>,
    /// Optional trailing supporting text shown on the supporting text row in the listbox.
    ///
    /// Material Web exposes this via the `"trailing-supporting-text"` slot on `md-select-option`.
    pub trailing_supporting_text: Option<Arc<str>>,
    /// Optional label text used when this option is displayed in the Select trigger.
    ///
    /// Material Web calls this `displayText`. When unset, we fall back to `typeahead_text` and then
    /// to `label`.
    pub display_text: Option<Arc<str>>,
    /// Optional text used for prefix typeahead matching in the listbox.
    ///
    /// Material Web calls this `typeaheadText`. When unset, we fall back to `display_text` and
    /// then to `label`.
    pub typeahead_text: Option<Arc<str>>,
    pub leading_icon: Option<IconId>,
    pub trailing_icon: Option<IconId>,
    pub disabled: bool,
    pub test_id: Option<Arc<str>>,
}

impl SelectItem {
    pub fn new(value: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            supporting_text: None,
            trailing_supporting_text: None,
            display_text: None,
            typeahead_text: None,
            leading_icon: None,
            trailing_icon: None,
            disabled: false,
            test_id: None,
        }
    }

    pub fn supporting_text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.supporting_text = Some(text.into());
        self
    }

    pub fn trailing_supporting_text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.trailing_supporting_text = Some(text.into());
        self
    }

    pub fn display_text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.display_text = Some(text.into());
        self
    }

    pub fn typeahead_text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.typeahead_text = Some(text.into());
        self
    }

    pub fn leading_icon(mut self, icon: IconId) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    pub fn trailing_icon(mut self, icon: IconId) -> Self {
        self.trailing_icon = Some(icon);
        self
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

#[derive(Debug, Clone, Default)]
pub struct SelectStyle {
    pub container_background: OverrideSlot<ColorRef>,
    pub outline_color: OverrideSlot<ColorRef>,
    pub active_indicator_color: OverrideSlot<ColorRef>,
    pub text_color: OverrideSlot<ColorRef>,
    pub label_color: OverrideSlot<ColorRef>,
    pub supporting_text_color: OverrideSlot<ColorRef>,
    pub leading_icon_color: OverrideSlot<ColorRef>,
    pub trailing_icon_color: OverrideSlot<ColorRef>,
    pub menu_selected_container_color: OverrideSlot<ColorRef>,
}

impl SelectStyle {
    pub fn container_background(
        mut self,
        background: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.container_background = Some(background);
        self
    }

    pub fn outline_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.outline_color = Some(color);
        self
    }

    pub fn active_indicator_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.active_indicator_color = Some(color);
        self
    }

    pub fn text_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.text_color = Some(color);
        self
    }

    pub fn label_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.label_color = Some(color);
        self
    }

    pub fn supporting_text_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.supporting_text_color = Some(color);
        self
    }

    pub fn leading_icon_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.leading_icon_color = Some(color);
        self
    }

    pub fn trailing_icon_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.trailing_icon_color = Some(color);
        self
    }

    pub fn menu_selected_container_color(
        mut self,
        color: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.menu_selected_container_color = Some(color);
        self
    }

    pub fn merged(mut self, other: Self) -> Self {
        self.container_background =
            merge_override_slot(self.container_background, other.container_background);
        self.outline_color = merge_override_slot(self.outline_color, other.outline_color);
        self.active_indicator_color =
            merge_override_slot(self.active_indicator_color, other.active_indicator_color);
        self.text_color = merge_override_slot(self.text_color, other.text_color);
        self.label_color = merge_override_slot(self.label_color, other.label_color);
        self.supporting_text_color =
            merge_override_slot(self.supporting_text_color, other.supporting_text_color);
        self.leading_icon_color =
            merge_override_slot(self.leading_icon_color, other.leading_icon_color);
        self.trailing_icon_color =
            merge_override_slot(self.trailing_icon_color, other.trailing_icon_color);
        self.menu_selected_container_color = merge_override_slot(
            self.menu_selected_container_color,
            other.menu_selected_container_color,
        );
        self
    }
}

#[derive(Clone)]
pub struct Select {
    model: Model<Option<Arc<str>>>,
    items: Arc<[SelectItem]>,
    variant: SelectVariant,
    menu_align: SelectMenuAlign,
    match_anchor_width: bool,
    menu_width_floor: Px,
    typeahead_delay_ms: u32,
    disabled: bool,
    leading_icon: Option<IconId>,
    label: Option<Arc<str>>,
    placeholder: Option<Arc<str>>,
    supporting_text: Option<Arc<str>>,
    error: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    style: SelectStyle,
}

impl std::fmt::Debug for Select {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Select").finish()
    }
}

impl Select {
    pub fn new(model: Model<Option<Arc<str>>>) -> Self {
        Self {
            model,
            items: Arc::from([]),
            variant: SelectVariant::default(),
            menu_align: SelectMenuAlign::default(),
            match_anchor_width: true,
            menu_width_floor: Px(210.0),
            typeahead_delay_ms: 200,
            disabled: false,
            leading_icon: None,
            label: None,
            placeholder: None,
            supporting_text: None,
            error: false,
            a11y_label: None,
            test_id: None,
            style: SelectStyle::default(),
        }
    }

    pub fn items(mut self, items: impl Into<Arc<[SelectItem]>>) -> Self {
        self.items = items.into();
        self
    }

    pub fn variant(mut self, variant: SelectVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn menu_align(mut self, align: SelectMenuAlign) -> Self {
        self.menu_align = align;
        self
    }

    pub fn match_anchor_width(mut self, match_anchor_width: bool) -> Self {
        self.match_anchor_width = match_anchor_width;
        self
    }

    /// Minimum menu width floor applied when `match_anchor_width(false)` is used.
    ///
    /// Material Web defaults the select host to `min-width: 210px` and sets the menu min-width to
    /// the select width, so anchored menus tend to avoid being too narrow. We mirror that as a
    /// stable default while still allowing callers to opt out (e.g. set `Px(0.0)`).
    pub fn menu_width_floor(mut self, floor: Px) -> Self {
        self.menu_width_floor = floor;
        self
    }

    /// Typeahead buffer timeout for listbox roving focus.
    ///
    /// Material Web exposes this as `typeaheadDelay` and defaults to 200ms.
    pub fn typeahead_delay_ms(mut self, ms: u32) -> Self {
        self.typeahead_delay_ms = ms;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn leading_icon(mut self, icon: IconId) -> Self {
        self.leading_icon = Some(icon);
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

    pub fn style(mut self, style: SelectStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        select_into_element(cx, self)
    }
}

#[derive(Clone)]
struct SelectRuntimeModels {
    open: Model<bool>,
    scroll_handle: fret_ui::scroll::ScrollHandle,
    listbox_element_id: Rc<Cell<Option<GlobalElementId>>>,
}

fn select_runtime_models<H: UiHost>(cx: &mut ElementContext<'_, H>) -> SelectRuntimeModels {
    #[derive(Default)]
    struct State {
        models: Option<SelectRuntimeModels>,
    }

    let existing = cx.with_state(State::default, |st| st.models.clone());
    if let Some(models) = existing {
        return models;
    }

    let models = SelectRuntimeModels {
        open: cx.app.models_mut().insert(false),
        scroll_handle: fret_ui::scroll::ScrollHandle::default(),
        listbox_element_id: Rc::new(Cell::new(None)),
    };

    cx.with_state(State::default, |st| st.models = Some(models.clone()));
    models
}

fn select_into_element<H: UiHost>(cx: &mut ElementContext<'_, H>, select: Select) -> AnyElement {
    cx.scope(|cx| {
        let runtime = select_runtime_models(cx);

        let is_open = cx
            .get_model_copied(&runtime.open, Invalidation::Layout)
            .unwrap_or(false);

        #[derive(Default)]
        struct OpenState {
            last_open: bool,
        }
        let opening = cx.with_state(OpenState::default, |st| {
            let opening = is_open && !st.last_open;
            st.last_open = is_open;
            opening
        });

        let close_grace_frames = {
            let theme = Theme::global(&*cx.app);
            Some(ms_to_frames(dropdown_menu_tokens::close_duration_ms(theme)))
        };
        let motion = drive_overlay_open_close_motion(cx, is_open, close_grace_frames);
        let overlay_presence = OverlayPresence {
            present: motion.present,
            interactive: is_open,
        };

        if !overlay_presence.present {
            runtime.listbox_element_id.set(None);
        }

        let selected = cx
            .get_model_cloned(&select.model, Invalidation::Layout)
            .unwrap_or(None);

        let populated = selected.as_ref().is_some_and(|v| !v.is_empty());

        if opening {
            let selected_idx = selected.as_ref().and_then(|value| {
                select
                    .items
                    .iter()
                    .position(|it| it.value.as_ref() == value.as_ref() && !it.disabled)
            });
            let first_enabled_idx = select.items.iter().position(|it| !it.disabled);
            let tab_stop_idx = selected_idx.or(first_enabled_idx).unwrap_or(0);

            let item_height = {
                let theme = Theme::global(&*cx.app);
                select_tokens::menu_list_item_height(theme, select.variant)
            };
            let menu_vertical_padding = Px(8.0);
            let target_y =
                Px(((item_height.0 * (tab_stop_idx as f32)) - menu_vertical_padding.0).max(0.0));
            runtime
                .scroll_handle
                .scroll_to_offset(Point::new(Px(0.0), target_y));
        }

        let selected_label = selected.as_ref().and_then(|value| {
            select
                .items
                .iter()
                .find(|it| it.value.as_ref() == value.as_ref())
        });

        let value_text = selected_label
            .map(select_item_display_text)
            .or_else(|| selected.clone())
            .unwrap_or_else(|| Arc::<str>::from(""));

        let trigger = select_trigger_element(
            cx,
            select.variant,
            select.disabled,
            select.error,
            overlay_presence.interactive,
            runtime.listbox_element_id.get().map(|id| id.0),
            value_text,
            populated,
            select.leading_icon.clone(),
            select.label.clone(),
            select.placeholder.clone(),
            select.supporting_text.clone(),
            select.a11y_label.clone(),
            select.test_id.clone(),
            runtime.open.clone(),
            select.style.clone(),
        );
        let anchor_id = trigger.anchor_id;
        let trigger = trigger.element;

        if overlay_presence.present {
            let Some(anchor) = fret_ui_kit::overlay::anchor_bounds_for_element(cx, anchor_id)
            else {
                return trigger;
            };

            let outer = fret_ui_kit::overlay::outer_bounds_with_window_margin(cx.bounds, Px(0.0));

            let item_height = {
                let theme = Theme::global(&*cx.app);
                select_tokens::menu_list_item_height(theme, select.variant)
            };
            let menu_vertical_padding = Px(8.0);
            let select_width = anchor.size.width;
            let desired_width = if select.match_anchor_width {
                select_width
            } else {
                let estimate =
                    estimate_select_menu_content_width(&*cx, select.variant, &select.items);
                resolve_select_menu_width(select_width, estimate, select.menu_width_floor)
            };
            let desired_height = Px((item_height.0 * (select.items.len().max(1) as f32))
                + menu_vertical_padding.0 * 2.0);
            let desired = Size::new(desired_width, desired_height);

            let direction = direction_prim::use_direction_in_scope(cx, None);
            let placement = popper::PopperContentPlacement::new(
                direction,
                Side::Bottom,
                match select.menu_align {
                    SelectMenuAlign::Start => Align::Start,
                    SelectMenuAlign::End => Align::End,
                },
                Px(4.0),
            )
            .with_collision_padding(Edges {
                left: Px(8.0),
                right: Px(8.0),
                top: Px(48.0),
                bottom: Px(48.0),
            });

            let layout = popper::popper_content_layout_sized(outer, anchor, desired, placement);

            let initial_focus_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
            let initial_focus_id_for_list = initial_focus_id.clone();

            let open_model = runtime.open.clone();
            let open_model_for_panel = open_model.clone();
            let listbox_element_id_out = runtime.listbox_element_id.clone();
            let overlay_root = popper_content::popper_wrapper_panel_at(
                cx,
                layout.rect,
                Edges::all(Px(0.0)),
                Overflow::Visible,
                move |cx| {
                    vec![select_listbox_panel(
                        cx,
                        select.variant,
                        select.model.clone(),
                        open_model_for_panel.clone(),
                        select.items.clone(),
                        selected.clone(),
                        select.a11y_label.clone(),
                        select.test_id.clone(),
                        Some(anchor_id.0),
                        initial_focus_id_for_list,
                        listbox_element_id_out,
                        runtime.scroll_handle.clone(),
                        select.typeahead_delay_ms,
                        select.style.clone(),
                    )]
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
                    vec![overlay_root],
                );

            let mut request = fret_ui_kit::overlay_controller::OverlayRequest::dismissible_menu(
                anchor_id,
                anchor_id,
                open_model.clone(),
                overlay_presence,
                vec![overlay_root],
            );
            request.root_name = Some(format!("material3.select.{}", anchor_id.0));
            request.close_on_window_focus_lost = true;
            request.close_on_window_resize = true;
            request.initial_focus = initial_focus_id.get();

            OverlayController::request(cx, request);
        }

        trigger
    })
}

#[derive(Debug)]
struct SelectTriggerOutput {
    element: AnyElement,
    anchor_id: GlobalElementId,
}

fn select_trigger_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    variant: SelectVariant,
    disabled: bool,
    error: bool,
    open: bool,
    controls_element: Option<u64>,
    value_text: Arc<str>,
    populated: bool,
    leading_icon: Option<IconId>,
    label: Option<Arc<str>>,
    placeholder: Option<Arc<str>>,
    supporting_text: Option<Arc<str>>,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    open_model: Model<bool>,
    style: SelectStyle,
) -> SelectTriggerOutput {
    let anchor_id_out: Cell<GlobalElementId> = Cell::new(GlobalElementId(0));
    let hovered_out: Cell<bool> = Cell::new(false);
    let focused_out: Cell<bool> = Cell::new(false);
    let focus_visible_out: Cell<bool> = Cell::new(false);
    let has_leading_icon = leading_icon.is_some();
    let leading_icon_fallback = {
        let theme = Theme::global(&*cx.app);
        select_tokens::leading_icon_size(theme, variant)
    };
    let leading_icon_size =
        crate::foundation::context::resolved_icon_size(&*cx, leading_icon_fallback);
    let trailing_icon_fallback = {
        let theme = Theme::global(&*cx.app);
        select_tokens::trailing_icon_size(theme, variant)
    };
    let trailing_icon_size =
        crate::foundation::context::resolved_icon_size(&*cx, trailing_icon_fallback);

    let container = cx.pressable_with_id_props(|cx, st, pressable_id| {
        anchor_id_out.set(pressable_id);
        let enabled = !disabled;

        let mut states = WidgetStates::from_pressable(cx, st, enabled);
        if open {
            states |= WidgetStates::OPEN;
        }

        let toggle_open = open_model.clone();
        let enabled_for_toggle = enabled;
        cx.pressable_on_activate(Arc::new(move |host, action_cx, _reason| {
            if !enabled_for_toggle {
                return;
            }
            let _ = host.models_mut().update(&toggle_open, |v| *v = !*v);
            host.request_redraw(action_cx.window);
        }));

        let enabled_for_key = enabled;
        let open_for_key = open;
        let toggle_open_on_key = open_model.clone();
        cx.key_add_on_key_down_for(
            pressable_id,
            Arc::new(move |host, action_cx, down| {
                if !enabled_for_key {
                    return false;
                }

                match down.key {
                    KeyCode::ArrowDown | KeyCode::ArrowUp => {
                        if !open_for_key {
                            let _ = host.models_mut().update(&toggle_open_on_key, |v| *v = true);
                            host.request_redraw(action_cx.window);
                        }
                        true
                    }
                    KeyCode::Escape => {
                        if open_for_key {
                            let _ = host
                                .models_mut()
                                .update(&toggle_open_on_key, |v| *v = false);
                            host.request_redraw(action_cx.window);
                            true
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            }),
        );

        let mut states_for_style = states;
        // Align with Material Web: when the menu is open, the field is treated as focused.
        // (see `select/internal/select.ts`: `.focused=${this.focused || this.open}`)
        if open {
            states_for_style |= WidgetStates::FOCUSED;
        }

        let focused = states_for_style.contains(WidgetStates::FOCUSED);
        let focus_visible = states.contains(WidgetStates::FOCUS_VISIBLE);
        let hovered = enabled && st.hovered;
        hovered_out.set(hovered);
        focused_out.set(focused);
        focus_visible_out.set(focus_visible);

        let (
            corner,
            container_bg,
            text_color,
            icon_color,
            icon_opacity,
            leading_icon_color,
            leading_icon_opacity,
            outline,
            indicator,
            container_height,
            hover_state_layer,
            pressed_opacity,
            focus_opacity,
            ripple_base_opacity,
            ripple_config,
            float_duration_ms,
            float_easing,
            open_duration_ms,
            close_duration_ms,
            open_easing,
            placeholder_color,
            input_text_style_fallback,
        ) = {
            let theme = Theme::global(&*cx.app);

            let corner = select_tokens::container_corner(theme, variant);
            let token_container_bg = Some(select_tokens::container_background(
                theme, variant, !enabled,
            ));
            let container_bg = resolve_override_slot_opt_with(
                style.container_background.as_ref(),
                states,
                |color| color.resolve(theme),
                || token_container_bg,
            );

            let (token_text_color, token_text_opacity) =
                select_tokens::input_text_color(theme, variant, hovered, !enabled, error, focused);
            let text_color = resolve_override_slot_with(
                style.text_color.as_ref(),
                states_for_style,
                |color| color.resolve(theme),
                || token_text_color,
            );
            let text_color = with_opacity(text_color, token_text_opacity);

            let (token_icon_color, token_icon_opacity) = select_tokens::trailing_icon_color(
                theme, variant, hovered, !enabled, error, focused,
            );
            let icon_color = resolve_override_slot_with(
                style.trailing_icon_color.as_ref(),
                states_for_style,
                |color| color.resolve(theme),
                || token_icon_color,
            );
            let icon_opacity = token_icon_opacity.clamp(0.0, 1.0);

            let (leading_icon_color, leading_icon_opacity) = select_tokens::leading_icon_color(
                theme, variant, hovered, !enabled, error, focused,
            );
            let leading_icon_color = resolve_override_slot_with(
                style.leading_icon_color.as_ref(),
                states_for_style,
                |color| color.resolve(theme),
                || leading_icon_color,
            );
            let leading_icon_opacity = leading_icon_opacity.clamp(0.0, 1.0);

            let outline = select_tokens::outline(theme, variant, hovered, !enabled, error, focused)
                .map(|(w, c, opacity)| (w, with_opacity(c, opacity)));
            let outline = outline.map(|(w, c)| {
                let c = resolve_override_slot_opt_with(
                    style.outline_color.as_ref(),
                    states_for_style,
                    |color| color.resolve(theme),
                    || Some(c),
                )
                .unwrap_or(c);
                (w, c)
            });

            let indicator =
                select_tokens::active_indicator(theme, variant, hovered, !enabled, error, focused)
                    .map(|(h, c, opacity)| (h, with_opacity(c, opacity)));
            let indicator = indicator.map(|(h, c)| {
                let c = resolve_override_slot_opt_with(
                    style.active_indicator_color.as_ref(),
                    states_for_style,
                    |color| color.resolve(theme),
                    || Some(c),
                )
                .unwrap_or(c);
                (h, c)
            });

            let container_height = select_tokens::container_height(theme, variant);

            let hover_state_layer = select_tokens::hover_state_layer(theme, variant, error);
            let pressed_opacity = theme
                .number_by_key("md.sys.state.pressed.state-layer-opacity")
                .unwrap_or(0.1);
            let focus_opacity = theme
                .number_by_key("md.sys.state.focus.state-layer-opacity")
                .unwrap_or(0.1);
            let ripple_base_opacity = pressed_opacity;
            let ripple_config = material_pressable_indication_config(theme, None);

            let float_duration_ms = theme
                .duration_ms_by_key("md.sys.motion.duration.short4")
                .unwrap_or(200);
            let float_easing = theme
                .easing_by_key("md.sys.motion.easing.standard")
                .unwrap_or(fret_ui::theme::CubicBezier {
                    x1: 0.0,
                    y1: 0.0,
                    x2: 1.0,
                    y2: 1.0,
                });

            let open_duration_ms = dropdown_menu_tokens::open_duration_ms(theme);
            let close_duration_ms = dropdown_menu_tokens::close_duration_ms(theme);
            let open_easing = dropdown_menu_tokens::easing(theme);

            let placeholder_color =
                select_tokens::placeholder_color(theme, variant, !enabled, error);
            let input_text_style_fallback = select_tokens::input_text_style(theme, variant);

            (
                corner,
                container_bg,
                text_color,
                icon_color,
                icon_opacity,
                leading_icon_color,
                leading_icon_opacity,
                outline,
                indicator,
                container_height,
                hover_state_layer,
                pressed_opacity,
                focus_opacity,
                ripple_base_opacity,
                ripple_config,
                float_duration_ms,
                float_easing,
                open_duration_ms,
                close_duration_ms,
                open_easing,
                placeholder_color,
                input_text_style_fallback,
            )
        };

        let a11y = PressableA11y {
            role: Some(SemanticsRole::ComboBox),
            label: a11y_label.clone(),
            test_id: test_id.clone(),
            expanded: Some(open),
            controls_element,
            ..Default::default()
        };

        let pressable_props = PressableProps {
            enabled,
            focusable: enabled,
            a11y,
            layout: {
                let mut l = fret_ui::element::LayoutStyle::default();
                l.size.width = Length::Fill;
                l.size.height = Length::Px(container_height);
                l.overflow = Overflow::Visible;
                l
            },
            focus_ring: None,
            focus_ring_bounds: None,
        };

        let pointer_region = cx.named("pointer_region", |cx| {
            let mut props = PointerRegionProps::default();
            props.enabled = enabled;
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;
            cx.pointer_region(props, |cx| {
                cx.pointer_region_on_pointer_down(Arc::new(|_host, _cx, _down| false));

                let now_frame = cx.frame_id.0;
                let (state_layer_color, hover_opacity) = hover_state_layer;

                let state_layer_target = if enabled && st.pressed {
                    pressed_opacity
                } else if enabled && focus_visible {
                    focus_opacity
                } else if hovered {
                    hover_opacity
                } else {
                    0.0
                };

                let overlay = material_ink_layer_for_pressable(
                    cx,
                    pressable_id,
                    now_frame,
                    corner,
                    RippleClip::Bounded,
                    state_layer_color,
                    enabled && st.pressed,
                    state_layer_target,
                    ripple_base_opacity,
                    ripple_config,
                    false,
                );

                let should_float = focused || open || populated;

                let (float_progress, float_want_frames) =
                    cx.with_state(SelectTriggerRuntime::default, |rt| {
                        if rt.float_target != should_float {
                            rt.float_target = should_float;
                            rt.float.set_target(
                                now_frame,
                                if should_float { 1.0 } else { 0.0 },
                                float_duration_ms,
                                float_easing,
                            );
                        }
                        rt.float.advance(now_frame);
                        (rt.float.value(), rt.float.is_active())
                    });

                let (chevron_progress, chevron_want_frames) =
                    cx.with_state(SelectChevronRuntime::default, |rt| {
                        if rt.target_open != open {
                            rt.target_open = open;
                            rt.anim.set_target(
                                now_frame,
                                if open { 1.0 } else { 0.0 },
                                if open {
                                    open_duration_ms
                                } else {
                                    close_duration_ms
                                },
                                open_easing,
                            );
                        }
                        rt.anim.advance(now_frame);
                        (rt.anim.value(), rt.anim.is_active())
                    });

                if float_want_frames || chevron_want_frames {
                    cx.request_animation_frame();
                }

                let show_placeholder = if label.is_some() {
                    (focused || open) && !populated
                } else {
                    true
                };
                let display_text = if populated {
                    value_text.clone()
                } else if show_placeholder {
                    placeholder.clone().unwrap_or_else(|| Arc::<str>::from(""))
                } else {
                    Arc::<str>::from("")
                };
                let is_placeholder = !populated && show_placeholder;

                let display_color = if is_placeholder {
                    placeholder_color
                } else {
                    text_color
                };

                let text_el = {
                    let mut props = TextProps::new(display_text);
                    let mut style = input_text_style_fallback;
                    if let Some(inherited) = crate::foundation::context::inherited_text_style(cx) {
                        style = Some(inherited);
                    }
                    props.style = style;
                    props.color = Some(display_color);
                    props.wrap = TextWrap::None;
                    props.overflow = TextOverflow::Ellipsis;
                    cx.text_props(props)
                };

                let icon_el = chevron_down_icon_rotated(
                    cx,
                    icon_color,
                    icon_opacity,
                    trailing_icon_size,
                    chevron_progress,
                );

                let mut row = FlexProps::default();
                row.layout.size.width = Length::Fill;
                row.layout.size.height = Length::Fill;
                row.layout.overflow = Overflow::Clip;
                row.direction = Axis::Horizontal;
                row.justify = MainAlign::SpaceBetween;
                row.align = CrossAlign::Center;
                row.padding = Edges {
                    left: if has_leading_icon { Px(12.0) } else { Px(16.0) },
                    right: Px(12.0),
                    top: Px(0.0),
                    bottom: Px(0.0),
                };

                let mut chrome = ContainerProps::default();
                chrome.layout.size.width = Length::Fill;
                chrome.layout.size.height = Length::Fill;
                chrome.layout.overflow = Overflow::Visible;
                chrome.background = container_bg;
                chrome.corner_radii = corner;
                let mut outline_width_for_notch = Px(0.0);
                if let Some((outline_width, outline_color)) = outline {
                    if outline_width.0 > 0.0 {
                        chrome.border = Edges::all(outline_width);
                        chrome.border_color = Some(outline_color);
                        outline_width_for_notch = outline_width;
                    }
                }

                let indicator_el = indicator.map(|(h, c)| {
                    cx.canvas(CanvasProps::default(), move |p| {
                        let bounds = p.bounds();
                        let y = Px(bounds.origin.y.0 + bounds.size.height.0 - h.0);
                        let rect = Rect::new(
                            Point::new(bounds.origin.x, y),
                            Size::new(bounds.size.width, h),
                        );
                        p.scene().push(fret_core::SceneOp::Quad {
                            order: fret_core::DrawOrder(0),
                            rect,
                            background: fret_core::Paint::Solid(c),
                            border: Edges::all(Px(0.0)),
                            border_paint: fret_core::Paint::TRANSPARENT,
                            corner_radii: Corners::all(Px(0.0)),
                        });
                    })
                });

                let style_override = style.clone();

                vec![cx.container(chrome, move |cx| {
                    let mut children = vec![overlay];

                    let leading_icon_el = leading_icon.as_ref().map(|icon| {
                        select_trigger_icon(
                            cx,
                            icon,
                            leading_icon_color,
                            leading_icon_opacity,
                            leading_icon_size,
                        )
                    });

                    let left_slot = cx.container(
                        ContainerProps {
                            layout: {
                                let mut l = fret_ui::element::LayoutStyle::default();
                                l.size.width = Length::Fill;
                                l.flex.grow = 1.0;
                                l.overflow = Overflow::Clip;
                                l
                            },
                            ..Default::default()
                        },
                        move |cx| {
                            let mut left = FlexProps::default();
                            left.layout.size.width = Length::Fill;
                            left.layout.overflow = Overflow::Clip;
                            left.direction = Axis::Horizontal;
                            left.justify = MainAlign::Start;
                            left.align = CrossAlign::Center;
                            left.gap = if has_leading_icon { Px(16.0) } else { Px(0.0) };

                            vec![cx.flex(left, move |_cx| {
                                let mut out = Vec::new();
                                if let Some(icon) = leading_icon_el {
                                    out.push(icon);
                                }
                                out.push(text_el);
                                out
                            })]
                        },
                    );

                    children.push(cx.flex(row, move |_cx| vec![left_slot, icon_el]));

                    if let Some(label) = label.as_ref() {
                        children.push(select_trigger_label(
                            cx,
                            variant,
                            states_for_style,
                            &style_override,
                            label.clone(),
                            float_progress,
                            has_leading_icon.then_some(leading_icon_size),
                            hovered,
                            !enabled,
                            error,
                            focused,
                            container_bg.unwrap_or(Color::TRANSPARENT),
                            outline_width_for_notch,
                        ));
                    }
                    if let Some(indicator_el) = indicator_el {
                        children.push(indicator_el);
                    }
                    children
                })]
            })
        });

        (pressable_props, vec![pointer_region])
    });

    let anchor_id = anchor_id_out.get();

    let mut supporting_states = WidgetStates::empty();
    let hovered = hovered_out.get();
    let focused = focused_out.get();
    let focus_visible = focus_visible_out.get();
    if disabled {
        supporting_states |= WidgetStates::DISABLED;
    }
    if hovered {
        supporting_states |= WidgetStates::HOVERED;
    }
    if focused {
        supporting_states |= WidgetStates::FOCUSED;
    }
    if focus_visible {
        supporting_states |= WidgetStates::FOCUS_VISIBLE;
    }
    if open {
        supporting_states |= WidgetStates::OPEN;
    }
    let supporting_style_override = style.clone();

    let element = if let Some(text) = supporting_text.as_ref() {
        let mut props = FlexProps::default();
        props.layout.size.width = Length::Fill;
        props.layout.overflow = Overflow::Visible;
        props.direction = Axis::Vertical;
        props.gap = Px(4.0);
        props.align = CrossAlign::Start;
        props.justify = MainAlign::Start;
        props.wrap = false;

        cx.flex(props, move |cx| {
            vec![
                container,
                select_supporting_text(
                    cx,
                    variant,
                    supporting_states,
                    &supporting_style_override,
                    text.clone(),
                    has_leading_icon.then_some(leading_icon_size),
                    hovered,
                    disabled,
                    error,
                    focused,
                ),
            ]
        })
    } else {
        container
    };

    SelectTriggerOutput { element, anchor_id }
}

fn chevron_down_icon<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    color: Color,
    opacity: f32,
    size: Px,
) -> AnyElement {
    let svg = svg_source_for_icon(cx, &ids::ui::CHEVRON_DOWN);

    let mut props = SvgIconProps::new(svg);
    props.fit = SvgFit::Contain;
    props.color = color;
    props.opacity = opacity;
    props.layout.size.width = Length::Px(size);
    props.layout.size.height = Length::Px(size);
    cx.svg_icon_props(props)
}

fn select_trigger_icon<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    icon: &IconId,
    color: Color,
    opacity: f32,
    size: Px,
) -> AnyElement {
    let svg = svg_source_for_icon(cx, icon);

    let mut props = SvgIconProps::new(svg);
    props.fit = SvgFit::Contain;
    props.color = color;
    props.opacity = opacity;
    props.layout.size.width = Length::Px(size);
    props.layout.size.height = Length::Px(size);
    cx.svg_icon_props(props)
}

fn chevron_down_icon_rotated<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    color: Color,
    opacity: f32,
    size: Px,
    progress: f32,
) -> AnyElement {
    let degrees = 180.0 * progress.clamp(0.0, 1.0);

    let mut layout = fret_ui::element::LayoutStyle::default();
    layout.size.width = Length::Px(size);
    layout.size.height = Length::Px(size);

    cx.visual_transform_props(
        VisualTransformProps {
            layout,
            transform: fret_core::Transform2D::rotation_about_degrees(
                degrees,
                Point::new(Px(size.0 * 0.5), Px(size.0 * 0.5)),
            ),
        },
        move |cx| vec![chevron_down_icon(cx, color, opacity, size)],
    )
}

fn select_menu_item_icon<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    icon: &IconId,
    color: Color,
    size: Px,
) -> AnyElement {
    let svg = svg_source_for_icon(cx, icon);

    let mut props = SvgIconProps::new(svg);
    props.fit = SvgFit::Contain;
    props.color = color;
    props.layout.size.width = Length::Px(size);
    props.layout.size.height = Length::Px(size);
    cx.svg_icon_props(props)
}

#[derive(Debug, Default)]
struct SelectTriggerRuntime {
    float_target: bool,
    float: StateLayerAnimator,
}

#[derive(Debug, Default)]
struct SelectChevronRuntime {
    target_open: bool,
    anim: StateLayerAnimator,
}

fn select_trigger_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    variant: SelectVariant,
    states: WidgetStates,
    style_override: &SelectStyle,
    text: Arc<str>,
    progress: f32,
    leading_icon_size: Option<Px>,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
    input_bg: Color,
    outline_width: Px,
) -> AnyElement {
    let (style, color) = {
        let theme = Theme::global(&*cx.app);
        let style = floating_label::material_floating_label_text_style(theme, progress)
            .or_else(|| theme.text_style_by_key("md.sys.typescale.body-large"));
        let color = resolve_override_slot_with(
            style_override.label_color.as_ref(),
            states,
            |color| color.resolve(theme),
            || select_tokens::label_color(theme, variant, hovered, disabled, error, focused),
        );
        (style, color)
    };

    let (x, y) = floating_label::material_floating_label_offsets(progress);

    let x = material_field_text_start_inset_x(x, leading_icon_size);

    let mut layout = fret_ui::element::LayoutStyle::default();
    layout.position = fret_ui::element::PositionStyle::Absolute;
    layout.inset.top = Some(y);
    layout.inset.left = Some(x);
    layout.inset.right = Some(Px(16.0));
    layout.overflow = Overflow::Visible;

    let floated = floating_label::is_floated(progress);

    let mut patch = ContainerProps::default();
    if variant == SelectVariant::Outlined {
        let patch_padding_x = Px(4.0);
        let patch_padding_y = Px((outline_width.0 + 1.0).max(0.0));
        patch.padding = if floated {
            Edges {
                top: patch_padding_y,
                right: patch_padding_x,
                bottom: patch_padding_y,
                left: patch_padding_x,
            }
        } else {
            Edges::all(Px(0.0))
        };
        patch.background = floated.then_some(input_bg);
    }
    patch.layout = layout;

    cx.container(patch, move |cx| {
        vec![cx.text_props(TextProps {
            layout: fret_ui::element::LayoutStyle::default(),
            text: text.clone(),
            style,
            color: Some(color),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        })]
    })
}

fn material_field_text_start_inset_x(default: Px, leading_icon_size: Option<Px>) -> Px {
    // Align with Material Web field layout:
    // - with-leading-icon leading space: 12px
    // - icon-content space: 16px
    // (see `tokens/_md-comp-(outlined|filled)-text-field.scss` in `repo-ref/material-web`)
    leading_icon_size
        .map(|icon_size| Px(12.0 + icon_size.0 + 16.0))
        .unwrap_or(default)
}

fn select_supporting_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    variant: SelectVariant,
    states: WidgetStates,
    style_override: &SelectStyle,
    text: Arc<str>,
    leading_icon_size: Option<Px>,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
) -> AnyElement {
    let (style, color) = {
        let theme = Theme::global(&*cx.app);
        let style = theme.text_style_by_key("md.sys.typescale.body-small");
        let color = resolve_override_slot_with(
            style_override.supporting_text_color.as_ref(),
            states,
            |color| color.resolve(theme),
            || {
                select_tokens::supporting_text_color(
                    theme, variant, hovered, disabled, error, focused,
                )
            },
        );
        (style, color)
    };

    let mut layout = fret_ui::element::LayoutStyle::default();
    layout.margin.left = fret_ui::element::MarginEdge::Px(material_field_text_start_inset_x(
        Px(16.0),
        leading_icon_size,
    ));
    layout.margin.right = fret_ui::element::MarginEdge::Px(Px(16.0));

    cx.text_props(TextProps {
        layout,
        text,
        style,
        color: Some(color),
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
    })
}

fn with_opacity(mut color: Color, opacity: f32) -> Color {
    color.a = (color.a * opacity).clamp(0.0, 1.0);
    color
}

fn select_item_display_text(item: &SelectItem) -> Arc<str> {
    item.display_text
        .clone()
        .or_else(|| item.typeahead_text.clone())
        .unwrap_or_else(|| item.label.clone())
}

fn select_item_typeahead_text(item: &SelectItem) -> Arc<str> {
    item.typeahead_text
        .clone()
        .or_else(|| item.display_text.clone())
        .unwrap_or_else(|| item.label.clone())
}

fn resolve_select_menu_width(select_width: Px, estimate: Px, floor: Px) -> Px {
    // Match Material Web behavior:
    // - Menu min-width tracks the select width.
    // - If unclamped, allow the menu to grow to fit content.
    // We additionally apply a small floor for ergonomics (configurable by the caller).
    Px(select_width.0.max(estimate.0).max(floor.0))
}

fn estimate_select_menu_content_width<H: UiHost>(
    cx: &ElementContext<'_, H>,
    variant: SelectVariant,
    items: &[SelectItem],
) -> Px {
    let theme = Theme::global(&*cx.app);
    let padding_left = Px(12.0);
    let padding_right = Px(12.0);
    let gap = Px(8.0);

    let label_style = select_tokens::menu_list_item_label_text_style(theme, variant)
        .unwrap_or_else(|| fret_core::TextStyle {
            size: Px(14.0),
            ..Default::default()
        });
    // Heuristic: average glyph width in a proportional UI font is ~0.5-0.6em.
    let avg_char_w = label_style.size.0 * 0.55;

    let supporting_style = theme
        .text_style_by_key("md.sys.typescale.body-small")
        .unwrap_or_else(|| fret_core::TextStyle {
            size: Px(12.0),
            ..Default::default()
        });
    let supporting_avg_char_w = supporting_style.size.0 * 0.55;

    let has_leading_icon = items.iter().any(|it| it.leading_icon.is_some());
    let has_trailing_icon = items.iter().any(|it| it.trailing_icon.is_some());
    let leading_icon_w = if has_leading_icon {
        select_tokens::menu_list_item_leading_icon_size(theme, variant).0 + gap.0
    } else {
        0.0
    };
    let trailing_icon_w = if has_trailing_icon {
        select_tokens::menu_list_item_trailing_icon_size(theme, variant).0 + gap.0
    } else {
        0.0
    };

    let max_main_chars = items
        .iter()
        .flat_map(|it| [Some(it.label.as_ref()), it.supporting_text.as_deref()])
        .flatten()
        .map(|text| text.chars().count())
        .max()
        .unwrap_or(0) as f32;
    let max_trailing_chars = items
        .iter()
        .filter_map(|it| it.trailing_supporting_text.as_deref())
        .map(|text| text.chars().count())
        .max()
        .unwrap_or(0) as f32;

    // Extra slack for ellipsis / punctuation so we don't undershoot too aggressively.
    let main_w = max_main_chars * avg_char_w;
    let trailing_w = max_trailing_chars * supporting_avg_char_w;
    let combined_w = if max_trailing_chars > 0.0 {
        main_w + gap.0 + trailing_w
    } else {
        main_w
    };
    let text_w = Px(combined_w + 16.0);

    let w = padding_left.0 + padding_right.0 + leading_icon_w + trailing_icon_w + text_w.0;
    Px(w.max(0.0))
}

#[cfg(test)]
mod item_text_tests {
    use super::*;

    #[test]
    fn select_item_display_text_prefers_display_text_then_typeahead_then_label() {
        let base = SelectItem::new("a", "Label");
        assert_eq!(super::select_item_display_text(&base).as_ref(), "Label");
        assert_eq!(
            super::select_item_display_text(&base.clone().typeahead_text("Typeahead")).as_ref(),
            "Typeahead"
        );
        assert_eq!(
            super::select_item_display_text(&base.clone().display_text("Display")).as_ref(),
            "Display"
        );
        assert_eq!(
            super::select_item_display_text(
                &base
                    .clone()
                    .typeahead_text("Typeahead")
                    .display_text("Display"),
            )
            .as_ref(),
            "Display"
        );
    }

    #[test]
    fn select_item_typeahead_text_prefers_typeahead_then_display_then_label() {
        let base = SelectItem::new("a", "Label");
        assert_eq!(super::select_item_typeahead_text(&base).as_ref(), "Label");
        assert_eq!(
            super::select_item_typeahead_text(&base.clone().display_text("Display")).as_ref(),
            "Display"
        );
        assert_eq!(
            super::select_item_typeahead_text(&base.clone().typeahead_text("Typeahead")).as_ref(),
            "Typeahead"
        );
        assert_eq!(
            super::select_item_typeahead_text(
                &base
                    .clone()
                    .display_text("Display")
                    .typeahead_text("Typeahead"),
            )
            .as_ref(),
            "Typeahead"
        );
    }

    #[test]
    fn select_menu_width_floor_only_applies_when_unclamped() {
        let floor = Px(210.0);
        let select_width = Px(120.0);
        let estimate = Px(160.0);
        assert_eq!(
            super::resolve_select_menu_width(select_width, estimate, floor).0,
            210.0
        );

        let select_width = Px(320.0);
        let estimate = Px(240.0);
        assert_eq!(
            super::resolve_select_menu_width(select_width, estimate, floor).0,
            320.0
        );

        let select_width = Px(320.0);
        let estimate = Px(480.0);
        assert_eq!(
            super::resolve_select_menu_width(select_width, estimate, floor).0,
            480.0
        );
    }
}

fn select_listbox_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    variant: SelectVariant,
    model: Model<Option<Arc<str>>>,
    open: Model<bool>,
    items: Arc<[SelectItem]>,
    selected: Option<Arc<str>>,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    labelled_by_element: Option<u64>,
    initial_focus_id_out: Rc<Cell<Option<GlobalElementId>>>,
    listbox_element_id_out: Rc<Cell<Option<GlobalElementId>>>,
    scroll_handle: fret_ui::scroll::ScrollHandle,
    typeahead_delay_ms: u32,
    style: SelectStyle,
) -> AnyElement {
    let listbox_test_id = test_id.as_ref().map(|id| {
        let id = id.as_ref();
        Arc::<str>::from(format!("{id}-listbox"))
    });

    let sem = SemanticsProps {
        role: SemanticsRole::ListBox,
        label: a11y_label.clone(),
        test_id: listbox_test_id.or_else(|| Some(Arc::<str>::from("material3-select-listbox"))),
        labelled_by_element,
        ..Default::default()
    };

    let disabled: Arc<[bool]> = Arc::from(items.iter().map(|it| it.disabled).collect::<Vec<_>>());
    let count = items.len();

    let tab_stop_idx = selected
        .as_ref()
        .and_then(|value| {
            items
                .iter()
                .position(|it| it.value.as_ref() == value.as_ref())
        })
        .filter(|&idx| !items.get(idx).is_some_and(|it| it.disabled))
        .or_else(|| disabled.iter().position(|&d| !d))
        .unwrap_or(0);

    let mut roving = RovingFlexProps::default();
    roving.flex.direction = Axis::Vertical;
    roving.flex.gap = Px(0.0);
    roving.flex.align = CrossAlign::Stretch;
    roving.flex.justify = MainAlign::Start;
    roving.roving = fret_ui::element::RovingFocusProps {
        enabled: true,
        wrap: true,
        disabled: disabled.clone(),
    };

    let (surface, corner, selected_bg) = {
        let theme = Theme::global(&*cx.app);
        let container_bg = select_tokens::menu_container_background(theme, variant);
        let elevation = select_tokens::menu_container_elevation(theme, variant);
        let shadow_color = select_tokens::menu_container_shadow_color(theme, variant);
        let corner = select_tokens::menu_container_shape(theme, variant);
        let surface =
            material_surface_style(theme, container_bg, elevation, Some(shadow_color), corner);

        let selected_bg_token =
            select_tokens::menu_list_item_selected_container_color(theme, variant);
        let selected_bg = resolve_override_slot_with(
            style.menu_selected_container_color.as_ref(),
            WidgetStates::SELECTED,
            |color| color.resolve(theme),
            || selected_bg_token,
        );
        (surface, corner, selected_bg)
    };

    cx.semantics_with_id(sem, move |cx, listbox_id| {
        listbox_element_id_out.set(Some(listbox_id));
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
            move |cx| {
                vec![cx.scroll(
                    ScrollProps {
                        layout: {
                            let mut l = fret_ui::element::LayoutStyle::default();
                            l.size.width = Length::Fill;
                            l.size.height = Length::Fill;
                            l.overflow = Overflow::Clip;
                            l
                        },
                        scroll_handle: Some(scroll_handle.clone()),
                        ..Default::default()
                    },
                    move |cx| {
                        vec![cx.container(
                            ContainerProps {
                                padding: Edges {
                                    top: Px(8.0),
                                    right: Px(0.0),
                                    bottom: Px(8.0),
                                    left: Px(0.0),
                                },
                                layout: {
                                    let mut l = fret_ui::element::LayoutStyle::default();
                                    l.size.width = Length::Fill;
                                    l.size.height = Length::Auto;
                                    l.overflow = Overflow::Visible;
                                    l
                                },
                                ..Default::default()
                            },
                            move |cx| {
                                vec![cx.roving_flex(roving, move |cx| {
                                    cx.roving_on_navigate(Arc::new(|_host, _cx, it| {
                                        use fret_ui::action::RovingNavigateResult;

                                        let is_disabled = |idx: usize| -> bool {
                                            it.disabled.get(idx).copied().unwrap_or(false)
                                        };

                                        let forward = match it.key {
                                            KeyCode::ArrowDown => Some(true),
                                            KeyCode::ArrowUp => Some(false),
                                            _ => None,
                                        };

                                        if it.key == KeyCode::Home {
                                            let target = (0..it.len).find(|&i| !is_disabled(i));
                                            return RovingNavigateResult::Handled { target };
                                        }
                                        if it.key == KeyCode::End {
                                            let target = (0..it.len).rev().find(|&i| !is_disabled(i));
                                            return RovingNavigateResult::Handled { target };
                                        }

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

                                    // Prefix typeahead (best-effort): matches `Menu` / `RadioGroup` behavior.
                                    fret_ui_kit::primitives::roving_focus_group::typeahead_prefix_arc_str_always_wrap(
                                        cx,
                                        Arc::from(
                                            items
                                                .iter()
                                                .map(select_item_typeahead_text)
                                                .collect::<Vec<_>>(),
                                        ),
                                        crate::motion::ms_to_frames(typeahead_delay_ms),
                                    );

                                    let mut out: Vec<AnyElement> = Vec::with_capacity(count);
                                    let two_line = items.iter().any(|it| {
                                        it.supporting_text.is_some()
                                            || it.trailing_supporting_text.is_some()
                                    });
                                    for (idx, item) in items.iter().cloned().enumerate() {
                                        let tab_stop = idx == tab_stop_idx;
                                        out.push(select_list_item(
                                            cx,
                                            variant,
                                            item,
                                            two_line,
                                            model.clone(),
                                            open.clone(),
                                            selected.clone(),
                                            selected_bg,
                                            tab_stop,
                                            idx,
                                            count,
                                            initial_focus_id_out.clone(),
                                        ));
                                    }
                                    out
                                })]
                            },
                        )]
                    },
                )]
            },
        )]
    })
}

fn select_list_item<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    variant: SelectVariant,
    item: SelectItem,
    two_line: bool,
    model: Model<Option<Arc<str>>>,
    open: Model<bool>,
    selected: Option<Arc<str>>,
    selected_bg: Color,
    tab_stop: bool,
    idx: usize,
    set_size: usize,
    initial_focus_id_out: Rc<Cell<Option<GlobalElementId>>>,
) -> AnyElement {
    let height = {
        let theme = Theme::global(&*cx.app);
        if two_line {
            list_tokens::two_line_container_height(theme)
        } else {
            select_tokens::menu_list_item_height(theme, variant)
        }
    };

    cx.pressable_with_id_props(move |cx, st, pressable_id| {
        let enabled = !item.disabled;

        if enabled && tab_stop && initial_focus_id_out.get().is_none() {
            initial_focus_id_out.set(Some(pressable_id));
        }

        let is_selected = selected
            .as_ref()
            .is_some_and(|value| value.as_ref() == item.value.as_ref());

        let a11y = PressableA11y {
            role: Some(SemanticsRole::ListBoxOption),
            label: Some(item.label.clone()),
            test_id: item.test_id.clone(),
            selected: is_selected,
            pos_in_set: Some((idx + 1) as u32),
            set_size: Some(set_size as u32),
            ..Default::default()
        };

        let model_for_select = model.clone();
        let open_for_close = open.clone();
        let value_for_select = item.value.clone();
        let enabled_for_select = enabled;
        cx.pressable_on_activate(Arc::new(move |host, action_cx, _reason| {
            if !enabled_for_select {
                return;
            }
            let _ = host
                .models_mut()
                .update(&model_for_select, |v| *v = Some(value_for_select.clone()));
            let _ = host.models_mut().update(&open_for_close, |v| *v = false);
            host.request_redraw(action_cx.window);
        }));

        let pressable_props = PressableProps {
            enabled,
            focusable: enabled && tab_stop,
            a11y,
            layout: {
                let mut l = fret_ui::element::LayoutStyle::default();
                l.size.width = Length::Fill;
                l.size.height = Length::Px(height);
                l.overflow = Overflow::Visible;
                l
            },
            focus_ring: None,
            focus_ring_bounds: None,
        };

        let pointer_region = cx.named("pointer_region", |cx| {
            let mut props = PointerRegionProps::default();
            props.enabled = enabled;
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;
            cx.pointer_region(props, |cx| {
                cx.pointer_region_on_pointer_down(Arc::new(|_host, _cx, _down| false));

                let now_frame = cx.frame_id.0;
                let supporting_text = item.supporting_text.clone();
                let trailing_supporting_text = item.trailing_supporting_text.clone();
                let has_secondary_text =
                    supporting_text.is_some() || trailing_supporting_text.is_some();

                let leading_icon = item.leading_icon.clone();
                let trailing_icon = item.trailing_icon.clone();

                let (
                    state_layer_color,
                    pressed_opacity,
                    hover_opacity,
                    focus_opacity,
                    ripple_base_opacity,
                    config,
                    label_color,
                    label_style,
                    item_top_space,
                    item_bottom_space,
                    supporting_color,
                    supporting_style,
                    trailing_supporting_color,
                    trailing_supporting_style,
                    leading_icon_color,
                    leading_icon_size,
                    trailing_icon_color,
                    trailing_icon_size,
                ) = {
                    let theme = Theme::global(&*cx.app);
                    let state_layer_color = theme
                        .color_by_key("md.sys.color.on-surface")
                        .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"));
                    let pressed_opacity = theme
                        .number_by_key("md.sys.state.pressed.state-layer-opacity")
                        .unwrap_or(0.1);
                    let hover_opacity = theme
                        .number_by_key("md.sys.state.hover.state-layer-opacity")
                        .unwrap_or(0.08);
                    let focus_opacity = theme
                        .number_by_key("md.sys.state.focus.state-layer-opacity")
                        .unwrap_or(0.1);
                    let ripple_base_opacity = pressed_opacity;
                    let config = material_pressable_indication_config(theme, None);

                    let label_color =
                        select_tokens::menu_list_item_label_text_color(theme, variant);
                    let label_style =
                        select_tokens::menu_list_item_label_text_style(theme, variant);

                    let item_top_space = list_tokens::item_top_space(theme);
                    let item_bottom_space = list_tokens::item_bottom_space(theme);

                    let supporting_color =
                        list_tokens::supporting_text_color(theme, enabled, is_selected);
                    let supporting_style = list_tokens::supporting_text_style(theme, is_selected);
                    let trailing_supporting_color =
                        list_tokens::trailing_supporting_text_color(theme, enabled, is_selected);
                    let trailing_supporting_style =
                        list_tokens::trailing_supporting_text_style(theme, is_selected);

                    let leading_icon_color =
                        select_tokens::menu_list_item_leading_icon_color(theme, variant);
                    let leading_icon_size =
                        select_tokens::menu_list_item_leading_icon_size(theme, variant);
                    let trailing_icon_color =
                        select_tokens::menu_list_item_trailing_icon_color(theme, variant);
                    let trailing_icon_size =
                        select_tokens::menu_list_item_trailing_icon_size(theme, variant);

                    (
                        state_layer_color,
                        pressed_opacity,
                        hover_opacity,
                        focus_opacity,
                        ripple_base_opacity,
                        config,
                        label_color,
                        label_style,
                        item_top_space,
                        item_bottom_space,
                        supporting_color,
                        supporting_style,
                        trailing_supporting_color,
                        trailing_supporting_style,
                        leading_icon_color,
                        leading_icon_size,
                        trailing_icon_color,
                        trailing_icon_size,
                    )
                };

                let focus_visible =
                    fret_ui::focus_visible::is_focus_visible(cx.app, Some(cx.window));
                let state_layer_target = if enabled && st.pressed {
                    pressed_opacity
                } else if enabled && st.hovered {
                    hover_opacity
                } else if enabled && st.focused && focus_visible {
                    focus_opacity
                } else {
                    0.0
                };
                let overlay = material_ink_layer_for_pressable(
                    cx,
                    pressable_id,
                    now_frame,
                    Corners::all(Px(0.0)),
                    RippleClip::Bounded,
                    state_layer_color,
                    enabled && st.pressed,
                    state_layer_target,
                    ripple_base_opacity,
                    config,
                    false,
                );

                let bg = if is_selected { Some(selected_bg) } else { None };
                let mut chrome = ContainerProps::default();
                chrome.background = bg;
                chrome.layout.size.width = Length::Fill;
                chrome.layout.size.height = Length::Px(height);
                chrome.layout.overflow = Overflow::Clip;

                let mut row = FlexProps::default();
                row.layout.size.width = Length::Fill;
                row.layout.size.height = Length::Px(height);
                row.layout.overflow = Overflow::Clip;
                row.direction = Axis::Horizontal;
                row.justify = MainAlign::Start;
                row.align = if has_secondary_text {
                    CrossAlign::Start
                } else {
                    CrossAlign::Center
                };
                row.gap = Px(8.0);
                row.padding = Edges {
                    left: Px(12.0),
                    right: Px(12.0),
                    top: if has_secondary_text {
                        item_top_space
                    } else {
                        Px(0.0)
                    },
                    bottom: if has_secondary_text {
                        item_bottom_space
                    } else {
                        Px(0.0)
                    },
                };

                vec![cx.container(chrome, move |cx| {
                    vec![cx.flex(row, move |cx| {
                        let body_slot = if has_secondary_text {
                            let headline_el = {
                                let mut props = TextProps::new(item.label.clone());
                                props.style = label_style;
                                props.color = Some(label_color);
                                props.wrap = TextWrap::None;
                                props.overflow = TextOverflow::Clip;
                                cx.text_props(props)
                            };

                            let supporting_el = supporting_text.as_ref().map(|text| {
                                let mut props = TextProps::new(text.clone());
                                props.style = supporting_style;
                                props.color = Some(supporting_color);
                                props.wrap = TextWrap::None;
                                props.overflow = TextOverflow::Clip;
                                cx.text_props(props)
                            });

                            let trailing_supporting_el =
                                trailing_supporting_text.as_ref().map(|text| {
                                    let mut props = TextProps::new(text.clone());
                                    props.style = trailing_supporting_style;
                                    props.color = Some(trailing_supporting_color);
                                    props.wrap = TextWrap::None;
                                    props.overflow = TextOverflow::Clip;
                                    cx.text_props(props)
                                });

                            let mut column = FlexProps::default();
                            column.direction = Axis::Vertical;
                            column.justify = MainAlign::Start;
                            column.align = CrossAlign::Stretch;
                            column.gap = Px(2.0);
                            column.layout.size.width = Length::Fill;
                            column.layout.overflow = Overflow::Clip;
                            column.layout.flex.grow = 1.0;

                            cx.flex(column, move |cx| {
                                let mut out = Vec::new();
                                out.push(headline_el);

                                let mut second_row = FlexProps::default();
                                second_row.direction = Axis::Horizontal;
                                second_row.justify = MainAlign::Start;
                                second_row.align = CrossAlign::Center;
                                second_row.gap = Px(8.0);
                                second_row.layout.size.width = Length::Fill;
                                second_row.layout.overflow = Overflow::Clip;

                                out.push(cx.flex(second_row, move |cx| {
                                    let mut flex_spacer = fret_ui::element::SpacerProps::default();
                                    flex_spacer.layout.size.width = Length::Fill;
                                    flex_spacer.layout.size.height = Length::Px(Px(0.0));
                                    flex_spacer.layout.flex.grow = 1.0;
                                    let flex_spacer = cx.spacer(flex_spacer);

                                    let mut children = Vec::new();
                                    if let Some(el) = supporting_el {
                                        children.push(el);
                                    }
                                    children.push(flex_spacer);
                                    if let Some(el) = trailing_supporting_el {
                                        children.push(el);
                                    }
                                    children
                                }));
                                out
                            })
                        } else {
                            let label_el = {
                                let mut props = TextProps::new(item.label.clone());
                                props.style = label_style;
                                props.color = Some(label_color);
                                props.wrap = TextWrap::None;
                                props.overflow = TextOverflow::Clip;
                                cx.text_props(props)
                            };

                            cx.container(
                                ContainerProps {
                                    layout: {
                                        let mut l = fret_ui::element::LayoutStyle::default();
                                        l.size.width = Length::Fill;
                                        l.flex.grow = 1.0;
                                        l.overflow = Overflow::Clip;
                                        l
                                    },
                                    ..Default::default()
                                },
                                move |_cx| vec![label_el],
                            )
                        };

                        let leading_icon_el = leading_icon.as_ref().map(|icon| {
                            select_menu_item_icon(cx, icon, leading_icon_color, leading_icon_size)
                        });

                        let trailing_icon_el = trailing_icon.as_ref().map(|icon| {
                            select_menu_item_icon(cx, icon, trailing_icon_color, trailing_icon_size)
                        });

                        let mut children = Vec::new();
                        children.push(overlay);
                        if let Some(icon) = leading_icon_el {
                            children.push(icon);
                        }
                        children.push(body_slot);
                        if let Some(icon) = trailing_icon_el {
                            children.push(icon);
                        }
                        children
                    })]
                })]
            })
        });

        (pressable_props, vec![pointer_region])
    })
}

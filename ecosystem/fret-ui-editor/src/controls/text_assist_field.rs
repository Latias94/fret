//! Editor-owned text assist field recipe.
//!
//! This sits above:
//! - `fret-ui-headless::text_assist` query/filter/navigation math,
//! - `fret-ui-kit::headless::text_assist` input-owned semantics + key policy glue,
//! - and below any app-local completion/history domain logic.
//!
//! Current scope:
//! - one owning `TextField`,
//! - shared listbox rendering for inline and anchored overlay surfaces,
//! - input-owned focus with `active_descendant`,
//! - default accept wiring that commits the chosen label back into the bound query model.

use std::cell::Cell;
use std::panic::Location;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{Axis, Corners, Edges, Px, SemanticsRole, Size, TextAlign};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate, UiActionHost, UiFocusActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
    PressableA11y, PressableProps, ScrollAxis, ScrollProps, SizeStyle, TextProps,
};
use fret_ui::overlay_placement::{Align, Side};
use fret_ui::{ElementContext, GlobalElementId, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::ModelWatchExt as _;
use fret_ui_kit::headless::text_assist::{
    InputOwnedTextAssistKeyOptions, TextAssistController, TextAssistItem, TextAssistMatch,
    active_match_index, controller_with_active_item_id, input_owned_text_assist_expanded,
    input_owned_text_assist_key_handler, input_owned_text_assist_semantics,
};
use fret_ui_kit::primitives::{popper, popper_content};
use fret_ui_kit::{OverlayController, OverlayPresence, OverlayRequest};

use super::{TextField, TextFieldAssistiveSemantics, TextFieldOptions};
use crate::primitives::colors::editor_muted_foreground;
use crate::primitives::popup_list::{
    EditorPopupListRowState, editor_popup_list_content_height,
    editor_popup_list_default_max_content_height, editor_popup_list_row_gap,
    editor_popup_list_row_palette, editor_popup_list_row_radius, editor_popup_list_row_text_style,
    editor_popup_list_surface_padding, editor_popup_side_offset, editor_popup_window_margin,
};
use crate::primitives::popup_surface::resolve_editor_popup_surface_chrome;
use crate::primitives::style::EditorStyle;

const TEXT_ASSIST_ROOT_GAP: Px = Px(6.0);

pub type OnTextAssistFieldAccept =
    Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, TextAssistMatch) + 'static>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAssistFieldSurface {
    #[default]
    Inline,
    AnchoredOverlay,
}

#[derive(Debug, Clone)]
pub struct TextAssistFieldOptions {
    /// Base `TextField` options for the owning input.
    ///
    /// Note: this recipe currently forces `buffered = false` because its input-owned key policy
    /// reads and writes the bound query model directly.
    pub field: TextFieldOptions,
    pub surface: TextAssistFieldSurface,
    pub list_label: Arc<str>,
    pub empty_label: Arc<str>,
    pub key_options: InputOwnedTextAssistKeyOptions,
    pub list_test_id: Option<Arc<str>>,
    pub item_test_id_prefix: Option<Arc<str>>,
    pub empty_test_id: Option<Arc<str>>,
    /// Maximum visible list content height before the recipe introduces scrolling.
    ///
    /// For anchored overlays, leaving this unset still applies a conservative editor default so
    /// the popup does not grow to the full window height.
    pub max_list_height: Option<Px>,
}

impl Default for TextAssistFieldOptions {
    fn default() -> Self {
        let field = TextFieldOptions {
            buffered: false,
            ..Default::default()
        };
        Self {
            field,
            surface: TextAssistFieldSurface::Inline,
            list_label: Arc::from("Suggestions"),
            empty_label: Arc::from("No matches"),
            key_options: InputOwnedTextAssistKeyOptions::default(),
            list_test_id: None,
            item_test_id_prefix: None,
            empty_test_id: None,
            max_list_height: None,
        }
    }
}

#[derive(Clone)]
pub struct TextAssistField {
    query_model: Model<String>,
    dismissed_query_model: Model<String>,
    active_item_id_model: Model<Option<Arc<str>>>,
    items: Arc<[TextAssistItem]>,
    on_accept: Option<OnTextAssistFieldAccept>,
    options: TextAssistFieldOptions,
}

struct RenderedTextAssistPanel {
    panel: AnyElement,
    listbox_id: Option<GlobalElementId>,
    option_elements: Vec<GlobalElementId>,
    surface_height: Px,
}

impl TextAssistField {
    pub fn new(
        query_model: Model<String>,
        dismissed_query_model: Model<String>,
        active_item_id_model: Model<Option<Arc<str>>>,
        items: Arc<[TextAssistItem]>,
    ) -> Self {
        Self {
            query_model,
            dismissed_query_model,
            active_item_id_model,
            items,
            on_accept: None,
            options: TextAssistFieldOptions::default(),
        }
    }

    pub fn options(mut self, options: TextAssistFieldOptions) -> Self {
        self.options = options;
        self
    }

    pub fn on_accept(mut self, on_accept: Option<OnTextAssistFieldAccept>) -> Self {
        self.on_accept = on_accept;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let model_id = self.query_model.id();
        let loc = Location::caller();
        let callsite = (loc.file(), loc.line(), loc.column());
        let id_source = self.options.field.id_source.clone();

        if let Some(id_source) = id_source.as_deref() {
            cx.keyed(
                ("fret-ui-editor.text_assist_field", id_source, model_id),
                |cx| self.into_element_keyed(cx),
            )
        } else {
            cx.keyed(
                ("fret-ui-editor.text_assist_field", callsite, model_id),
                |cx| self.into_element_keyed(cx),
            )
        }
    }

    fn into_element_keyed<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let TextAssistField {
            query_model,
            dismissed_query_model,
            active_item_id_model,
            items,
            on_accept,
            options,
        } = self;

        let query = cx
            .watch_model(&query_model)
            .paint()
            .cloned()
            .unwrap_or_default();
        let dismissed_query = cx
            .watch_model(&dismissed_query_model)
            .paint()
            .cloned()
            .unwrap_or_default();
        let active_item_id = cx
            .watch_model(&active_item_id_model)
            .paint()
            .cloned()
            .unwrap_or(None);

        let controller = controller_with_active_item_id(
            items.as_ref(),
            &query,
            active_item_id.as_ref(),
            options.key_options.match_mode,
            options.key_options.wrap_navigation,
        );
        let visible_count = if query.trim().is_empty() {
            0
        } else {
            controller.visible().len()
        };
        let expanded = input_owned_text_assist_expanded(&query, &dismissed_query, visible_count);
        let overlay_open = overlay_open_model(cx);
        let prev_overlay_open = cx
            .get_model_copied(&overlay_open, Invalidation::Layout)
            .unwrap_or(false);
        if prev_overlay_open != expanded {
            let _ = cx.app.models_mut().update(&overlay_open, |value| {
                *value = expanded;
            });
        }

        let rendered_panel = render_text_assist_panel(
            cx,
            &controller,
            expanded,
            &options,
            query_model.clone(),
            dismissed_query_model.clone(),
            active_item_id_model.clone(),
            on_accept.clone(),
        );

        let active_index = if expanded {
            active_match_index(&controller)
        } else {
            None
        };
        let semantics = input_owned_text_assist_semantics(
            cx,
            rendered_panel
                .as_ref()
                .map(|panel| panel.option_elements.as_slice())
                .unwrap_or(&[]),
            active_index,
            rendered_panel.as_ref().and_then(|panel| panel.listbox_id),
            expanded,
        );

        let field_id_out = Rc::new(Cell::new(None::<GlobalElementId>));
        let input_id_out = Rc::new(Cell::new(None::<GlobalElementId>));
        let mut field_options = options.field.clone();
        field_options.buffered = false;
        field_options.field_id_out = Some(field_id_out.clone());
        field_options.input_id_out = Some(input_id_out.clone());
        field_options.assistive_semantics = TextFieldAssistiveSemantics {
            active_descendant: semantics.active_descendant,
            active_descendant_element: semantics.active_descendant_element,
            controls_element: semantics.controls_element,
            expanded: Some(semantics.expanded),
        };

        let field = TextField::new(query_model.clone())
            .options(field_options.clone())
            .into_element(cx);

        let mut inline_panel = None;
        if let Some(rendered_panel) = rendered_panel {
            match options.surface {
                TextAssistFieldSurface::Inline => {
                    inline_panel = Some(rendered_panel.panel);
                }
                TextAssistFieldSurface::AnchoredOverlay => {
                    let RenderedTextAssistPanel {
                        panel,
                        surface_height,
                        ..
                    } = rendered_panel;
                    inline_panel = match input_id_out.get() {
                        Some(input_id) => request_text_assist_overlay(
                            cx,
                            input_id,
                            field_id_out.get(),
                            overlay_open.clone(),
                            query_model.clone(),
                            dismissed_query_model.clone(),
                            panel,
                            surface_height,
                        ),
                        None => Some(panel),
                    };
                }
            }
        }

        let show_inline_empty_label =
            should_render_inline_empty_label(options.surface, &query, visible_count);
        let empty_label = options.empty_label.clone();
        let empty_test_id = options.empty_test_id.clone();
        let query_model_for_key = query_model.clone();
        let dismissed_query_model_for_key = dismissed_query_model.clone();
        let active_item_id_model_for_key = active_item_id_model.clone();
        let items_for_key = items.clone();
        let on_accept_for_key = on_accept.clone();
        let key_options = options.key_options;
        let root = cx.flex(
            FlexProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: Axis::Vertical,
                gap: TEXT_ASSIST_ROOT_GAP.into(),
                padding: Edges::all(Px(0.0)).into(),
                justify: MainAlign::Start,
                align: CrossAlign::Stretch,
                wrap: false,
            },
            move |cx| {
                let mut children = vec![field];
                if let Some(panel) = inline_panel {
                    children.push(panel);
                } else if show_inline_empty_label {
                    let mut props = TextProps::new(empty_label.clone());
                    props.color = Some(editor_muted_foreground(Theme::global(&*cx.app)));
                    let empty = cx.text_props(props);
                    let empty = if let Some(test_id) = empty_test_id.as_ref() {
                        empty.test_id(test_id.clone())
                    } else {
                        empty
                    };
                    children.push(empty);
                }
                children
            },
        );

        cx.key_add_on_key_down_capture_for(
            root.id,
            input_owned_text_assist_key_handler(
                items_for_key,
                query_model_for_key.clone(),
                dismissed_query_model_for_key.clone(),
                active_item_id_model_for_key.clone(),
                key_options,
                Arc::new(move |host: &mut dyn UiFocusActionHost, action_cx, active| {
                    accept_text_assist_match(
                        host,
                        action_cx,
                        &query_model_for_key,
                        &dismissed_query_model_for_key,
                        &active_item_id_model_for_key,
                        active,
                        on_accept_for_key.as_ref(),
                    );
                }),
            ),
        );

        root
    }
}

fn render_text_assist_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controller: &TextAssistController,
    expanded: bool,
    options: &TextAssistFieldOptions,
    query_model: Model<String>,
    dismissed_query_model: Model<String>,
    active_item_id_model: Model<Option<Arc<str>>>,
    on_accept: Option<OnTextAssistFieldAccept>,
) -> Option<RenderedTextAssistPanel> {
    if !expanded {
        return None;
    }

    let is_overlay_surface = matches!(options.surface, TextAssistFieldSurface::AnchoredOverlay);
    let (density, popup_chrome) = {
        let theme = Theme::global(&*cx.app);
        let style = EditorStyle::resolve(theme);
        (
            style.density,
            resolve_editor_popup_surface_chrome(theme, is_overlay_surface),
        )
    };

    let content_height =
        editor_popup_list_content_height(density.row_height, controller.visible().len());
    let max_content_height = text_assist_max_content_height(
        options.surface,
        options.max_list_height,
        density.row_height,
    );
    let viewport_height = max_content_height
        .map(|max_height| Px(content_height.0.min(max_height.0)))
        .unwrap_or(content_height);
    let surface_height = Px(viewport_height.0 + editor_popup_list_surface_padding().0 * 2.0);
    let item_test_id_prefix = options
        .item_test_id_prefix
        .clone()
        .or_else(|| options.list_test_id.clone());

    let mut option_elements = Vec::new();
    let option_rows: Vec<_> = controller
        .visible()
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let is_active = controller
                .active_item_id()
                .is_some_and(|active| active == &entry.item_id);
            let option_test_id = item_test_id_prefix
                .as_ref()
                .map(|prefix| Arc::<str>::from(format!("{prefix}.item.{}", entry.item_id)));
            let query_model = query_model.clone();
            let dismissed_query_model = dismissed_query_model.clone();
            let active_item_id_model = active_item_id_model.clone();
            let on_accept = on_accept.clone();
            let active = entry.clone();
            let row = cx.pressable(
                PressableProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Px(density.row_height),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    enabled: !entry.disabled,
                    focusable: false,
                    a11y: PressableA11y {
                        role: Some(SemanticsRole::ListBoxOption),
                        label: Some(entry.label.clone()),
                        test_id: option_test_id.clone(),
                        selected: is_active,
                        pos_in_set: Some((idx as u32) + 1),
                        set_size: Some(controller.visible().len() as u32),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                move |cx, st| {
                    let on_activate: OnActivate =
                        Arc::new(move |host, action_cx, _reason: ActivateReason| {
                            accept_text_assist_match(
                                host,
                                action_cx,
                                &query_model,
                                &dismissed_query_model,
                                &active_item_id_model,
                                active.clone(),
                                on_accept.as_ref(),
                            );
                        });
                    cx.pressable_add_on_activate(on_activate);

                    let hovered = st.hovered || st.hovered_raw;
                    let row_palette = {
                        let theme = Theme::global(&*cx.app);
                        editor_popup_list_row_palette(
                            theme,
                            hovered,
                            EditorPopupListRowState {
                                active: is_active,
                                disabled: entry.disabled,
                            },
                        )
                    };

                    vec![cx.container(
                        ContainerProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Fill,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            padding: Edges::symmetric(density.padding_x, Px(0.0)).into(),
                            background: row_palette.bg,
                            corner_radii: Corners::all(editor_popup_list_row_radius()),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![cx.text_props(TextProps {
                                layout: LayoutStyle {
                                    size: SizeStyle {
                                        width: Length::Fill,
                                        height: Length::Fill,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                text: entry.label.clone(),
                                style: Some(editor_popup_list_row_text_style(density.row_height)),
                                color: Some(row_palette.fg),
                                wrap: TextWrap::None,
                                overflow: TextOverflow::Ellipsis,
                                align: TextAlign::Start,
                                ink_overflow: Default::default(),
                            })]
                        },
                    )]
                },
            );
            option_elements.push(row.id);
            row
        })
        .collect();

    let listbox_id_out = Rc::new(Cell::new(None::<GlobalElementId>));
    let listbox_label = options.list_label.clone();
    let list_test_id = options.list_test_id.clone();
    let panel_layout = if is_overlay_surface {
        LayoutStyle {
            size: SizeStyle {
                width: Length::Fill,
                height: Length::Fill,
                ..Default::default()
            },
            overflow: Overflow::Clip,
            ..Default::default()
        }
    } else {
        LayoutStyle {
            size: SizeStyle {
                width: Length::Fill,
                height: Length::Auto,
                ..Default::default()
            },
            overflow: Overflow::Clip,
            ..Default::default()
        }
    };
    let panel = {
        let listbox_id_out = listbox_id_out.clone();
        cx.semantics_with_id(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::ListBox,
                label: Some(listbox_label),
                test_id: list_test_id,
                ..Default::default()
            },
            move |cx, id| {
                listbox_id_out.set(Some(id));

                let list_content = cx.flex(
                    FlexProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Fill,
                                height: Length::Auto,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        direction: Axis::Vertical,
                        gap: editor_popup_list_row_gap().into(),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Start,
                        align: CrossAlign::Stretch,
                        wrap: false,
                    },
                    move |_cx| option_rows,
                );

                let body = if viewport_height != content_height {
                    cx.scroll(
                        ScrollProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Px(viewport_height),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            axis: ScrollAxis::Y,
                            ..Default::default()
                        },
                        move |_cx| vec![list_content],
                    )
                } else {
                    list_content
                };

                vec![cx.container(
                    ContainerProps {
                        layout: panel_layout,
                        padding: Edges::all(editor_popup_list_surface_padding()).into(),
                        background: Some(popup_chrome.bg),
                        border: Edges::all(Px(1.0)),
                        border_color: Some(popup_chrome.border),
                        corner_radii: Corners::all(popup_chrome.radius),
                        shadow: popup_chrome.shadow,
                        ..Default::default()
                    },
                    move |_cx| vec![body],
                )]
            },
        )
    };

    Some(RenderedTextAssistPanel {
        panel,
        listbox_id: listbox_id_out.get(),
        option_elements,
        surface_height,
    })
}

fn request_text_assist_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    input_id: GlobalElementId,
    field_id: Option<GlobalElementId>,
    open: Model<bool>,
    query_model: Model<String>,
    dismissed_query_model: Model<String>,
    panel: AnyElement,
    surface_height: Px,
) -> Option<AnyElement> {
    let Some(anchor) = fret_ui_kit::overlay::anchor_bounds_for_element(cx, input_id) else {
        return Some(panel);
    };
    let outer = fret_ui_kit::overlay::outer_bounds_with_window_margin_for_environment(
        cx,
        Invalidation::Layout,
        editor_popup_window_margin(),
    );
    let placement = popper::PopperContentPlacement::new(
        popper::LayoutDirection::Ltr,
        Side::Bottom,
        Align::Start,
        editor_popup_side_offset(),
    )
    .with_collision_padding(Edges::all(editor_popup_window_margin()));
    let desired = Size::new(anchor.size.width, surface_height);
    let layout = popper::popper_content_layout_sized(outer, anchor, desired, placement);
    cx.diagnostics_record_overlay_placement_placed_rect(
        Some("editor.text_assist"),
        Some(input_id),
        Some(panel.id),
        outer,
        anchor,
        layout.rect,
        Some(layout.side),
    );
    let overlay_panel = popper_content::popper_wrapper_panel_at(
        cx,
        layout.rect,
        Edges::all(Px(0.0)),
        Overflow::Visible,
        move |_cx| vec![panel],
    );

    let overlay_id = cx
        .named("text_assist_field.overlay", |cx| {
            cx.spacer(Default::default())
        })
        .id;
    let is_open = cx
        .get_model_copied(&open, Invalidation::Layout)
        .unwrap_or(false);
    let presence = OverlayPresence::instant(is_open);
    let query_model_for_dismiss = query_model.clone();
    let dismissed_query_model_for_dismiss = dismissed_query_model.clone();
    let open_for_dismiss = open.clone();

    let mut request = OverlayRequest::dismissible_popover(
        overlay_id,
        input_id,
        open,
        presence,
        vec![overlay_panel],
    );
    request.root_name = Some(format!("editor.text_assist.{}", input_id.0));
    request.close_on_window_focus_lost = true;
    request.close_on_window_resize = true;
    if let Some(field_id) = field_id {
        request = request.add_dismissable_branch(field_id);
    }
    request.dismissible_on_dismiss_request =
        Some(Arc::new(move |host, action_cx: ActionCx, _req| {
            let query = host
                .models_mut()
                .read(&query_model_for_dismiss, Clone::clone)
                .ok()
                .unwrap_or_default();
            let _ = host
                .models_mut()
                .update(&dismissed_query_model_for_dismiss, |value| {
                    value.clear();
                    value.push_str(&query);
                });
            let _ = host.models_mut().update(&open_for_dismiss, |value| {
                *value = false;
            });
            host.request_redraw(action_cx.window);
        }));

    OverlayController::request(cx, request);
    None
}

fn should_render_inline_empty_label(
    surface: TextAssistFieldSurface,
    query: &str,
    visible_count: usize,
) -> bool {
    matches!(surface, TextAssistFieldSurface::Inline)
        && !query.trim().is_empty()
        && visible_count == 0
}

fn text_assist_max_content_height(
    surface: TextAssistFieldSurface,
    max_list_height: Option<Px>,
    row_height: Px,
) -> Option<Px> {
    max_list_height.or_else(|| {
        matches!(surface, TextAssistFieldSurface::AnchoredOverlay)
            .then(|| editor_popup_list_default_max_content_height(row_height))
    })
}

fn accept_text_assist_match(
    host: &mut dyn UiActionHost,
    action_cx: ActionCx,
    query_model: &Model<String>,
    dismissed_query_model: &Model<String>,
    active_item_id_model: &Model<Option<Arc<str>>>,
    active: TextAssistMatch,
    on_accept: Option<&OnTextAssistFieldAccept>,
) {
    let next_query = active.label.as_ref().to_string();
    let _ = host.models_mut().update(query_model, |value| {
        value.clear();
        value.push_str(&next_query);
    });
    let _ = host.models_mut().update(dismissed_query_model, |value| {
        value.clear();
        value.push_str(&next_query);
    });
    let _ = host.models_mut().update(active_item_id_model, |value| {
        *value = Some(active.item_id.clone())
    });
    if let Some(on_accept) = on_accept {
        on_accept(host, action_cx, active);
    }
    host.request_redraw(action_cx.window);
}

#[track_caller]
fn overlay_open_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<bool> {
    cx.local_model(|| false)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{
        TextAssistFieldOptions, TextAssistFieldSurface, should_render_inline_empty_label,
        text_assist_max_content_height,
    };
    use fret_core::Px;

    #[test]
    fn text_assist_field_defaults_to_unbuffered_field_policy() {
        let options = TextAssistFieldOptions::default();
        assert!(!options.field.buffered);
        assert!(matches!(options.surface, TextAssistFieldSurface::Inline));
        assert_eq!(options.list_label.as_ref(), "Suggestions");
        assert_eq!(options.empty_label.as_ref(), "No matches");
    }

    #[test]
    fn text_assist_field_item_test_id_prefix_can_fallback_to_list_test_id() {
        let options = TextAssistFieldOptions {
            list_test_id: Some(Arc::from("editor.name-assist.list")),
            ..Default::default()
        };
        let prefix = options
            .item_test_id_prefix
            .clone()
            .or_else(|| options.list_test_id.clone());
        assert_eq!(prefix.as_deref(), Some("editor.name-assist.list"));
    }

    #[test]
    fn empty_label_is_inline_only() {
        assert!(should_render_inline_empty_label(
            TextAssistFieldSurface::Inline,
            "cube",
            0,
        ));
        assert!(!should_render_inline_empty_label(
            TextAssistFieldSurface::AnchoredOverlay,
            "cube",
            0,
        ));
    }

    #[test]
    fn anchored_overlay_defaults_to_capped_content_height() {
        let max_height =
            text_assist_max_content_height(TextAssistFieldSurface::AnchoredOverlay, None, Px(28.0));
        assert_eq!(max_height, Some(Px(178.0)));
    }
}

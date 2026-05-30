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

mod model;
mod overlay;
mod panel;
#[cfg(test)]
mod tests;

use std::cell::Cell;
use std::panic::Location;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{Axis, Edges, Px};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, UiActionHost, UiFocusActionHost};
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle,
};
use fret_ui::{ElementContext, GlobalElementId, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::ModelWatchExt as _;
use fret_ui_kit::headless::text_assist::{
    TextAssistItem, TextAssistMatch, active_match_index, controller_with_active_item_id,
    input_owned_text_assist_expanded, input_owned_text_assist_key_handler,
    input_owned_text_assist_semantics,
};

use super::{TextField, TextFieldAssistiveSemantics, TextFieldOptions};
use crate::primitives::colors::editor_muted_foreground;
use crate::primitives::popup_list::editor_popup_list_default_max_content_height;
use crate::primitives::readout::editor_popup_empty_text_props;
use crate::primitives::style::EditorStyle;

use model::RenderedTextAssistPanel;
pub use model::{OnTextAssistFieldAccept, TextAssistFieldOptions, TextAssistFieldSurface};
use overlay::{overlay_open_model, request_text_assist_overlay};
use panel::render_text_assist_panel;

const TEXT_ASSIST_ROOT_GAP: Px = Px(6.0);

#[derive(Clone)]
pub struct TextAssistField {
    query_model: Model<String>,
    dismissed_query_model: Model<String>,
    active_item_id_model: Model<Option<Arc<str>>>,
    items: Arc<[TextAssistItem]>,
    on_accept: Option<OnTextAssistFieldAccept>,
    options: TextAssistFieldOptions,
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
                    let theme = Theme::global(&*cx.app);
                    let empty = cx.text_props(editor_popup_empty_text_props(
                        empty_label.clone(),
                        editor_muted_foreground(theme),
                        EditorStyle::resolve(theme).density.row_height,
                    ));
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

use std::sync::Arc;

use fret_genui_core::props::{PropResolutionContext, ResolvedProps, resolve_action_param};
use fret_genui_core::render::GenUiRenderScope;
use fret_runtime::CommandId;
use fret_ui::action::OnActivate;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::facade as shadcn;
use serde_json::{Map, Value};

use super::ShadcnResolver;

fn action_params_map(v: Option<&Value>) -> Map<String, Value> {
    v.and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_else(Map::new)
}

fn resolve_params_for_state(params: Map<String, Value>, state: &Value) -> Value {
    let ctx = PropResolutionContext {
        state,
        repeat: fret_genui_core::visibility::RepeatScope::default(),
    };
    Value::Object(
        params
            .into_iter()
            .map(|(k, v)| (k, resolve_action_param(&v, &ctx)))
            .collect(),
    )
}

fn emit_action_from_ui_hook(
    host: &mut dyn fret_ui::action::UiActionHost,
    cx: fret_ui::action::ActionCx,
    scope: &GenUiRenderScope,
    element_key: Arc<str>,
    event: &'static str,
    action: Arc<str>,
    params: Map<String, Value>,
) {
    let state_model = scope.state.as_ref();
    let queue = scope.action_queue.as_ref();
    let auto_apply = scope.auto_apply_standard_actions;

    let state_snapshot: Value = state_model
        .and_then(|m| host.models_mut().read(m, Clone::clone).ok())
        .unwrap_or(Value::Null);

    let params = resolve_params_for_state(params, &state_snapshot);
    ShadcnResolver::emit_action_invocation_action(
        host,
        cx,
        queue,
        state_model,
        auto_apply,
        &element_key,
        event,
        &action,
        params,
    );
}

impl ShadcnResolver {
    pub(super) fn render_tooltip<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        resolved_props: &serde_json::Map<String, Value>,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        let content = resolved_props
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let content_el = shadcn::TooltipContent::build(cx, |_cx| {
            [shadcn::TooltipContent::text(Arc::<str>::from(content))]
        })
        .into_element(cx);

        let trigger = if children.is_empty() {
            shadcn::Button::new(Arc::<str>::from("?"))
                .variant(shadcn::ButtonVariant::Ghost)
                .size(shadcn::ButtonSize::Sm)
                .into_element(cx)
        } else if children.len() == 1 {
            children.into_iter().next().expect("single child")
        } else {
            fret_ui_kit::ui::h_flex(move |_cx| children)
                .gap(fret_ui_kit::Space::N1)
                .items_center()
                .into_element(cx)
        };

        shadcn::Tooltip::new(cx, trigger, content_el).into_element(cx)
    }

    pub(super) fn render_popover<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        resolved_props: &serde_json::Map<String, Value>,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        let trigger_text = resolved_props
            .get("trigger")
            .and_then(|v| v.as_str())
            .unwrap_or("Open");
        let trigger_text = Arc::<str>::from(trigger_text);

        let trigger = shadcn::PopoverTrigger::build(
            shadcn::Button::new(trigger_text).variant(shadcn::ButtonVariant::Outline),
        );
        let content = shadcn::PopoverContent::new(children);
        shadcn::Popover::new(cx, trigger, content).into_element(cx)
    }

    pub(super) fn render_dropdown_menu<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        key: &fret_genui_core::spec::ElementKey,
        props: &ResolvedProps,
    ) -> AnyElement {
        let resolved = &props.props;
        let trigger_text = resolved
            .get("trigger")
            .and_then(|v| v.as_str())
            .unwrap_or("Menu");
        let trigger_text = Arc::<str>::from(trigger_text);

        let items = resolved.get("items").and_then(|v| v.as_array()).cloned();
        let scope = Self::genui_scope(cx);
        let element_key: Arc<str> = Arc::from(key.0.as_str());

        let menu = shadcn::DropdownMenu::new_controllable(cx, None, false);
        menu.into_element(
            cx,
            move |cx| {
                shadcn::Button::new(trigger_text)
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx)
            },
            move |_cx| {
                let mut entries: Vec<shadcn::DropdownMenuEntry> = Vec::new();
                let Some(items) = items.as_ref() else {
                    return entries;
                };
                for item in items {
                    let Some(obj) = item.as_object() else {
                        continue;
                    };
                    if obj.get("type").and_then(|v| v.as_str()) == Some("separator") {
                        entries.push(shadcn::DropdownMenuEntry::Separator);
                        continue;
                    }

                    let Some(label) = obj.get("label").and_then(|v| v.as_str()) else {
                        continue;
                    };

                    let disabled = obj
                        .get("disabled")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    let variant = obj.get("variant").and_then(|v| v.as_str());
                    let variant = match variant {
                        Some("destructive") => {
                            shadcn::raw::dropdown_menu::DropdownMenuItemVariant::Destructive
                        }
                        _ => shadcn::raw::dropdown_menu::DropdownMenuItemVariant::Default,
                    };

                    let action = obj
                        .get("action")
                        .and_then(|v| v.as_str())
                        .map(Arc::<str>::from);
                    let params =
                        action_params_map(obj.get("params").or_else(|| obj.get("actionParams")));
                    let test_id = obj
                        .get("testId")
                        .and_then(|v| v.as_str())
                        .map(Arc::<str>::from);

                    let mut menu_item = shadcn::DropdownMenuItem::new(Arc::<str>::from(label))
                        .disabled(disabled)
                        .variant(variant);

                    if let Some(test_id) = test_id {
                        menu_item = menu_item.test_id(test_id);
                    }

                    if let Some(action) = action {
                        // Action-first mapping for menu items (v1): when a menu entry binds a stable, namespaced
                        // unit action id (no params), dispatch it through the command/action pipeline rather than
                        // enqueuing a GenUI action invocation.
                        let is_action_id =
                            action.as_ref().contains('.') && action.as_ref().ends_with(".v1");
                        let is_unit = params.is_empty();
                        if is_action_id && is_unit {
                            menu_item =
                                menu_item.action(CommandId::new(action.as_ref().to_string()));
                        } else if let Some(scope) = scope.clone() {
                            let element_key = element_key.clone();
                            let action_for_hook = action.clone();
                            let params_for_hook = params.clone();
                            let on_activate: OnActivate = Arc::new(move |host, acx, _reason| {
                                emit_action_from_ui_hook(
                                    host,
                                    acx,
                                    &scope,
                                    element_key.clone(),
                                    "select",
                                    action_for_hook.clone(),
                                    params_for_hook.clone(),
                                );
                            });
                            menu_item = menu_item.on_activate(on_activate);
                        }
                    }

                    entries.push(shadcn::DropdownMenuEntry::Item(menu_item));
                }
                entries
            },
        )
    }

    pub(super) fn render_dialog<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        resolved_props: &serde_json::Map<String, Value>,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        let trigger_text = resolved_props
            .get("trigger")
            .and_then(|v| v.as_str())
            .unwrap_or("Open");
        let trigger_text = Arc::<str>::from(trigger_text);

        let title = resolved_props
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Dialog");
        let title = Arc::<str>::from(title);
        let description = resolved_props.get("description").and_then(|v| v.as_str());

        let dialog = shadcn::Dialog::new_controllable(cx, None, false);
        dialog.into_element(
            cx,
            move |cx| {
                shadcn::Button::new(trigger_text)
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx)
            },
            move |cx| {
                let mut header_children: Vec<AnyElement> =
                    vec![shadcn::DialogTitle::new(title).into_element(cx)];
                if let Some(desc) = description {
                    header_children.push(shadcn::DialogDescription::new(desc).into_element(cx));
                }
                let header = shadcn::DialogHeader::new(header_children).into_element(cx);

                let mut out: Vec<AnyElement> = Vec::with_capacity(children.len().saturating_add(1));
                out.push(header);
                out.extend(children);
                shadcn::DialogContent::new(out).into_element(cx)
            },
        )
    }

    pub(super) fn render_drawer<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        resolved_props: &serde_json::Map<String, Value>,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        let trigger_text = resolved_props
            .get("trigger")
            .and_then(|v| v.as_str())
            .unwrap_or("Open");
        let trigger_text = Arc::<str>::from(trigger_text);

        let title = resolved_props
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Drawer");
        let title = Arc::<str>::from(title);
        let description = resolved_props.get("description").and_then(|v| v.as_str());
        let side = resolved_props.get("side").and_then(|v| v.as_str());
        let side = match side {
            Some("top") => shadcn::DrawerSide::Top,
            Some("left") => shadcn::DrawerSide::Left,
            Some("right") => shadcn::DrawerSide::Right,
            _ => shadcn::DrawerSide::Bottom,
        };

        let drawer = shadcn::Drawer::new_controllable(cx, None, false).side(side);
        drawer.into_element(
            cx,
            move |cx| {
                shadcn::Button::new(trigger_text)
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx)
            },
            move |cx| {
                let mut header_children: Vec<AnyElement> =
                    vec![shadcn::DrawerTitle::new(title).into_element(cx)];
                if let Some(desc) = description {
                    header_children.push(shadcn::DrawerDescription::new(desc).into_element(cx));
                }
                let header = shadcn::DrawerHeader::new(header_children).into_element(cx);

                let mut out: Vec<AnyElement> = Vec::with_capacity(children.len().saturating_add(1));
                out.push(header);
                out.extend(children);
                shadcn::DrawerContent::new(out).into_element(cx)
            },
        )
    }
}

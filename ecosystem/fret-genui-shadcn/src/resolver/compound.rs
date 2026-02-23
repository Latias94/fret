use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use fret_genui_core::props::ResolvedProps;
use fret_genui_core::render::RenderedChildV1;
use fret_genui_core::spec::ElementKey;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use serde_json::Value;

use super::ShadcnResolver;

#[derive(Debug, Clone)]
struct TabsDef {
    value: Arc<str>,
    label: Arc<str>,
}

fn parse_tabs_def(v: Option<&Value>) -> Vec<TabsDef> {
    let Some(Value::Array(items)) = v else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for item in items {
        let Some(obj) = item.as_object() else {
            continue;
        };
        let Some(value) = obj.get("value").and_then(|v| v.as_str()) else {
            continue;
        };
        let label = obj.get("label").and_then(|v| v.as_str()).unwrap_or(value);
        out.push(TabsDef {
            value: Arc::<str>::from(value),
            label: Arc::<str>::from(label),
        });
    }
    out
}

impl ShadcnResolver {
    pub(super) fn render_tab_content<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        fret_ui_kit::ui::container(cx, move |_cx| children).into_element(cx)
    }

    pub(super) fn render_tabs<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        key: &ElementKey,
        props: &ResolvedProps,
        children: Vec<RenderedChildV1>,
    ) -> AnyElement {
        let resolved = &props.props;
        let tabs = parse_tabs_def(resolved.get("tabs"));
        let default_value = resolved
            .get("defaultValue")
            .and_then(|v| v.as_str())
            .map(Arc::<str>::from)
            .or_else(|| tabs.first().map(|t| t.value.clone()));

        let mut panels_in_order: Vec<(Arc<str>, Option<AnyElement>)> = Vec::new();
        for child in children.into_iter() {
            if child.component != "TabContent" {
                continue;
            }
            let value = child
                .props
                .props
                .get("value")
                .and_then(|v| v.as_str())
                .map(Arc::<str>::from);
            let Some(value) = value else {
                continue;
            };
            panels_in_order.push((value, Some(child.rendered)));
        }

        if panels_in_order.is_empty() {
            return self.unknown_component(cx, key, "Tabs (missing TabContent children)");
        }

        let panel_index_by_value: BTreeMap<Arc<str>, usize> = panels_in_order
            .iter()
            .enumerate()
            .map(|(idx, (value, _))| (value.clone(), idx))
            .collect();

        let trigger_defs: Vec<TabsDef> = if !tabs.is_empty() {
            tabs
        } else {
            panels_in_order
                .iter()
                .map(|(value, _)| TabsDef {
                    value: value.clone(),
                    label: value.clone(),
                })
                .collect()
        };

        let model = Self::ensure_optional_arc_str_model(cx, default_value);

        let mut list = fret_ui_shadcn::TabsList::new();
        let mut contents: Vec<fret_ui_shadcn::TabsContent> = Vec::new();

        let mut included: BTreeSet<Arc<str>> = BTreeSet::new();
        for def in trigger_defs.iter() {
            included.insert(def.value.clone());
            list = list.trigger(fret_ui_shadcn::TabsTrigger::new(
                def.value.clone(),
                def.label.clone(),
            ));

            let content = panel_index_by_value
                .get(&def.value)
                .and_then(|&idx| panels_in_order.get_mut(idx))
                .and_then(|(_, slot)| slot.take())
                .unwrap_or_else(|| {
                    let msg = Arc::<str>::from(format!("Missing TabContent for '{}'", def.value));
                    fret_ui_kit::ui::text(cx, msg).into_element(cx)
                });
            contents.push(fret_ui_shadcn::TabsContent::new(
                def.value.clone(),
                [content],
            ));
        }

        // Include any extra panels that weren't listed in `tabs`.
        for (value, content) in panels_in_order
            .into_iter()
            .filter_map(|(value, slot)| slot.map(|content| (value, content)))
        {
            if included.contains(&value) {
                continue;
            }
            contents.push(fret_ui_shadcn::TabsContent::new(value.clone(), [content]));
        }

        let force_mount_content = resolved
            .get("forceMountContent")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        fret_ui_shadcn::TabsRoot::new(model)
            .list(list)
            .contents(contents)
            .force_mount_content(force_mount_content)
            .into_element(cx)
    }

    pub(super) fn render_accordion_item<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        fret_ui_kit::ui::container(cx, move |_cx| children).into_element(cx)
    }

    pub(super) fn render_accordion<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        key: &ElementKey,
        props: &ResolvedProps,
        children: Vec<RenderedChildV1>,
    ) -> AnyElement {
        let resolved = &props.props;
        let kind = resolved
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("single");
        let collapsible = resolved
            .get("collapsible")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let default_value = resolved
            .get("defaultValue")
            .and_then(|v| v.as_str())
            .map(Arc::<str>::from);

        let default_values: Vec<Arc<str>> = resolved
            .get("defaultValues")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(Arc::<str>::from))
                    .collect()
            })
            .unwrap_or_default();

        let mut items: Vec<fret_ui_shadcn::AccordionItem> = Vec::new();
        for child in children.into_iter() {
            if child.component != "AccordionItem" {
                continue;
            }

            let value = child
                .props
                .props
                .get("value")
                .and_then(|v| v.as_str())
                .map(Arc::<str>::from);
            let title = child
                .props
                .props
                .get("title")
                .and_then(|v| v.as_str())
                .map(Arc::<str>::from);

            let (Some(value), Some(title)) = (value, title) else {
                continue;
            };

            let trigger_label = fret_ui_shadcn::typography::small(cx, title.clone());
            let trigger = fret_ui_shadcn::AccordionTrigger::new([trigger_label]);
            let content = fret_ui_shadcn::AccordionContent::new([child.rendered]);
            items.push(fret_ui_shadcn::AccordionItem::new(value, trigger, content));
        }

        if items.is_empty() {
            return self.unknown_component(cx, key, "Accordion (missing AccordionItem children)");
        }

        match kind {
            "multiple" => {
                let model = Self::ensure_vec_arc_str_model(cx, default_values);
                fret_ui_shadcn::Accordion::multiple(model)
                    .items(items)
                    .into_element(cx)
            }
            _ => {
                let model = Self::ensure_optional_arc_str_model(cx, default_value);
                fret_ui_shadcn::Accordion::single(model)
                    .collapsible(collapsible)
                    .items(items)
                    .into_element(cx)
            }
        }
    }
}

//! GenUI catalog definition for the `fret-ui-shadcn`-backed resolver.
//!
//! This catalog is the "guardrail contract" for LLM output and tooling.
//! Keep it conservative and stable: prefer a small, predictable surface.

use std::collections::{BTreeMap, BTreeSet};

use fret_genui_core::catalog::{CatalogActionV1, CatalogComponentV1, CatalogPropV1, CatalogV1};

pub fn shadcn_catalog_v1() -> CatalogV1 {
    let mut catalog = CatalogV1::new();

    catalog.components = shadcn_components_v1();
    catalog.actions = shadcn_actions_v1();

    catalog
}

fn shadcn_components_v1() -> BTreeMap<String, CatalogComponentV1> {
    let mut out = BTreeMap::new();

    out.insert(
        "Text".to_string(),
        component("Plain text").props(["text"]).build(),
    );
    out.insert(
        "VStack".to_string(),
        component("Vertical stack (flex column)")
            .props(["gap"])
            .build(),
    );
    out.insert(
        "HStack".to_string(),
        component("Horizontal stack (flex row)")
            .props(["gap"])
            .build(),
    );
    out.insert(
        "Card".to_string(),
        component("Card container")
            .props(["wrapContent"])
            .note("Default: wraps children with CardContent; set wrapContent=false to provide CardHeader/CardContent/CardFooter explicitly.")
            .build(),
    );
    out.insert(
        "CardHeader".to_string(),
        component("Card header section (layout only)").build(),
    );
    out.insert(
        "CardContent".to_string(),
        component("Card content section (layout only)").build(),
    );
    out.insert(
        "CardFooter".to_string(),
        component("Card footer section (layout only)").build(),
    );
    out.insert(
        "CardTitle".to_string(),
        component("Card title text")
            .props(["text", "title"])
            .build(),
    );
    out.insert(
        "CardDescription".to_string(),
        component("Card description text")
            .props(["text", "description"])
            .build(),
    );
    out.insert(
        "Button".to_string(),
        component("Button (clickable)")
            .props(["label"])
            .events(["press"])
            .build(),
    );
    out.insert(
        "Badge".to_string(),
        component("Badge label")
            .props(["label", "variant"])
            .note("variant: default|secondary|destructive|outline")
            .build(),
    );
    out.insert(
        "Input".to_string(),
        component("Single-line input")
            .props(["placeholder", "value"])
            .note("Use {\"$bindState\": \"/path\"} for two-way binding on `value`.")
            .build(),
    );
    out.insert(
        "Switch".to_string(),
        component("Boolean toggle")
            .props(["checked"])
            .note("Use {\"$bindState\": \"/path\"} for two-way binding on `checked`.")
            .build(),
    );
    out.insert(
        "Separator".to_string(),
        component("Divider line")
            .props(["orientation", "flexStretchCrossAxis"])
            .build(),
    );
    out.insert(
        "ScrollArea".to_string(),
        component("Scroll container")
            .props(["axis", "showScrollbar"])
            .build(),
    );

    out
}

fn shadcn_actions_v1() -> BTreeMap<String, CatalogActionV1> {
    let mut out = BTreeMap::new();

    out.insert(
        "setState".to_string(),
        action("Write a JSON value at a JSON Pointer path.").params([
            ("statePath", "JSON Pointer path (e.g. /name)"),
            ("value", "New value (any JSON)"),
        ]),
    );
    out.insert(
        "incrementState".to_string(),
        action("Increment an integer field at a JSON Pointer path.").params([
            ("statePath", "JSON Pointer path (e.g. /count)"),
            ("delta", "Integer delta (defaults to 1)"),
        ]),
    );

    out
}

#[derive(Default)]
struct ComponentBuilder {
    c: CatalogComponentV1,
}

fn component(description: &str) -> ComponentBuilder {
    ComponentBuilder {
        c: CatalogComponentV1 {
            description: Some(description.to_string()),
            props: BTreeMap::new(),
            events: BTreeSet::new(),
        },
    }
}

impl ComponentBuilder {
    fn note(mut self, s: &str) -> Self {
        let base = self.c.description.take().unwrap_or_default();
        self.c.description = Some(if base.is_empty() {
            s.to_string()
        } else {
            format!("{base} {s}")
        });
        self
    }

    fn props<const N: usize>(mut self, names: [&'static str; N]) -> Self {
        for name in names {
            self.c.props.insert(name.to_string(), CatalogPropV1::new());
        }
        self
    }

    fn events<const N: usize>(mut self, names: [&'static str; N]) -> Self {
        for name in names {
            self.c.events.insert(name.to_string());
        }
        self
    }

    fn build(self) -> CatalogComponentV1 {
        self.c
    }
}

#[derive(Default)]
struct ActionBuilder {
    a: CatalogActionV1,
}

fn action(description: &str) -> ActionBuilder {
    ActionBuilder {
        a: CatalogActionV1 {
            description: Some(description.to_string()),
            params: BTreeMap::new(),
        },
    }
}

impl ActionBuilder {
    fn params<const N: usize>(
        mut self,
        params: [(&'static str, &'static str); N],
    ) -> CatalogActionV1 {
        for (name, desc) in params {
            self.a.params.insert(
                name.to_string(),
                CatalogPropV1 {
                    description: Some(desc.to_string()),
                },
            );
        }
        self.a
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_exports_spec_schema() {
        let catalog = shadcn_catalog_v1();
        let schema = catalog.spec_json_schema();
        assert_eq!(
            schema
                .get("properties")
                .and_then(|p| p.get("schema_version"))
                .and_then(|v| v.get("const"))
                .and_then(|v| v.as_i64()),
            Some(1)
        );
    }
}

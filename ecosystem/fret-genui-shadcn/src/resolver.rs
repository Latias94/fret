use std::sync::Arc;

use fret_genui_core::props::ResolvedProps;
use fret_genui_core::render::ComponentResolver;
use fret_genui_core::spec::{ElementKey, ElementV1};
use fret_ui::action::OnActivate;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

#[derive(Debug, thiserror::Error)]
pub enum ShadcnResolverError {
    #[error("invalid props for component: {component}")]
    InvalidProps { component: String },
}

#[derive(Clone, Default)]
pub struct ShadcnResolver;

impl ShadcnResolver {
    pub fn new() -> Self {
        Self
    }

    fn text_element<H: UiHost>(cx: &mut ElementContext<'_, H>, text: Arc<str>) -> AnyElement {
        fret_ui_kit::ui::text(cx, text).into_element(cx)
    }

    fn json_to_label(v: Option<&serde_json::Value>) -> Arc<str> {
        let Some(v) = v else {
            return Arc::<str>::from("");
        };
        if let Some(s) = v.as_str() {
            return Arc::<str>::from(s);
        }
        Arc::<str>::from(v.to_string())
    }

    fn unknown_component<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        key: &ElementKey,
        component: &str,
    ) -> AnyElement {
        let msg = Arc::<str>::from(format!("Unknown GenUI component: {component} ({:?})", key));
        fret_ui_shadcn::Card::new([
            fret_ui_shadcn::CardContent::new([Self::text_element(cx, msg)]).into_element(cx),
        ])
        .into_element(cx)
    }

    fn parse_space(v: Option<&serde_json::Value>) -> Option<fret_ui_kit::Space> {
        let s = v?.as_str()?;
        use fret_ui_kit::Space;
        Some(match s {
            "N0" => Space::N0,
            "N0p5" => Space::N0p5,
            "N1" => Space::N1,
            "N1p5" => Space::N1p5,
            "N2" => Space::N2,
            "N2p5" => Space::N2p5,
            "N3" => Space::N3,
            "N3p5" => Space::N3p5,
            "N4" => Space::N4,
            "N5" => Space::N5,
            "N6" => Space::N6,
            "N8" => Space::N8,
            "N10" => Space::N10,
            "N11" => Space::N11,
            "N12" => Space::N12,
            _ => return None,
        })
    }

    fn parse_badge_variant(v: Option<&serde_json::Value>) -> Option<fret_ui_shadcn::BadgeVariant> {
        let s = v?.as_str()?;
        use fret_ui_shadcn::BadgeVariant;
        Some(match s {
            "default" => BadgeVariant::Default,
            "secondary" => BadgeVariant::Secondary,
            "destructive" => BadgeVariant::Destructive,
            "outline" => BadgeVariant::Outline,
            _ => return None,
        })
    }
}

impl<H: UiHost> ComponentResolver<H> for ShadcnResolver {
    type Error = ShadcnResolverError;

    fn render_element(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        key: &ElementKey,
        element: &ElementV1,
        props: &ResolvedProps,
        children: Vec<AnyElement>,
        on_event: &dyn Fn(&str) -> Option<OnActivate>,
    ) -> Result<AnyElement, ShadcnResolverError> {
        let resolved_props = &props.props;
        match element.ty.as_str() {
            "Card" => Ok(fret_ui_shadcn::Card::new([
                fret_ui_shadcn::CardContent::new(children).into_element(cx)
            ])
            .into_element(cx)),
            "Text" => {
                let text = Self::json_to_label(resolved_props.get("text"));
                Ok(fret_ui_kit::ui::text(cx, text).into_element(cx))
            }
            "VStack" => {
                let gap =
                    Self::parse_space(resolved_props.get("gap")).unwrap_or(fret_ui_kit::Space::N2);
                Ok(fret_ui_kit::ui::v_flex(cx, move |_cx| children)
                    .gap(gap)
                    .into_element(cx))
            }
            "HStack" => {
                let gap =
                    Self::parse_space(resolved_props.get("gap")).unwrap_or(fret_ui_kit::Space::N2);
                Ok(fret_ui_kit::ui::h_flex(cx, move |_cx| children)
                    .gap(gap)
                    .into_element(cx))
            }
            "Button" => {
                let label = Self::json_to_label(resolved_props.get("label"));
                let mut button = fret_ui_shadcn::Button::new(label).children(children);
                if let Some(on_activate) = on_event("press") {
                    button = button.on_activate(on_activate);
                }
                Ok(button.into_element(cx))
            }
            "Badge" => {
                let label = Self::json_to_label(resolved_props.get("label"));
                let variant =
                    Self::parse_badge_variant(resolved_props.get("variant")).unwrap_or_default();
                Ok(fret_ui_shadcn::Badge::new(label)
                    .variant(variant)
                    .children(children)
                    .into_element(cx))
            }
            other => Ok(self.unknown_component(cx, key, other)),
        }
    }
}

use fret_genui_core::props::ResolvedProps;
use fret_genui_core::render::ComponentResolver;
use fret_genui_core::spec::{ElementKey, ElementV1};
use fret_ui::action::OnActivate;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

mod basic;
mod forms;
mod helpers;
mod responsive;

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
            "Card" => Ok(self.render_card(cx, resolved_props, children)),
            "CardHeader" => Ok(self.render_card_header(cx, children)),
            "CardContent" => Ok(self.render_card_content(cx, children)),
            "CardFooter" => Ok(self.render_card_footer(cx, children)),
            "CardTitle" => Ok(self.render_card_title(cx, resolved_props)),
            "CardDescription" => Ok(self.render_card_description(cx, resolved_props)),
            "Text" => Ok(self.render_text(cx, resolved_props)),
            "VStack" => Ok(self.render_vstack(cx, resolved_props, children)),
            "HStack" => Ok(self.render_hstack(cx, resolved_props, children)),
            "Separator" => Ok(self.render_separator(cx, resolved_props)),
            "ScrollArea" => Ok(self.render_scroll_area(cx, resolved_props, children)),
            "Button" => Ok(self.render_button(cx, resolved_props, children, on_event)),
            "Input" => Ok(self.render_input(cx, key, props, children)),
            "Switch" => Ok(self.render_switch(cx, key, props, children)),
            "Badge" => Ok(self.render_badge(cx, resolved_props, children)),
            "ResponsiveGrid" => Ok(self.render_responsive_grid(cx, resolved_props, children)),
            "ResponsiveStack" => Ok(self.render_responsive_stack(cx, resolved_props, children)),
            other => Ok(self.unknown_component(cx, key, other)),
        }
    }
}

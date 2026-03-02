use fret_genui_core::props::ResolvedProps;
use fret_genui_core::render::{ComponentResolver, RenderedChildV1};
use fret_genui_core::spec::{ElementKey, ElementV1};
use fret_ui::action::OnActivate;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

mod basic;
mod choice;
mod compat;
mod compound;
mod data;
mod feedback;
mod forms;
mod helpers;
mod navigation;
mod numeric;
mod overlay;
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
        children: Vec<RenderedChildV1>,
        on_event: &dyn Fn(&str) -> Option<OnActivate>,
    ) -> Result<AnyElement, ShadcnResolverError> {
        match element.ty.as_str() {
            "Tabs" => return Ok(self.render_tabs(cx, key, props, children)),
            "Accordion" => return Ok(self.render_accordion(cx, key, props, children)),
            _ => {}
        }

        let resolved_props = &props.props;
        let children: Vec<AnyElement> = children.into_iter().map(|c| c.rendered).collect();

        match element.ty.as_str() {
            "TabContent" => Ok(self.render_tab_content(cx, children)),
            "AccordionItem" => Ok(self.render_accordion_item(cx, children)),
            "Card" => Ok(self.render_card(cx, resolved_props, children)),
            "CardHeader" => Ok(self.render_card_header(cx, children)),
            "CardContent" => Ok(self.render_card_content(cx, children)),
            "CardFooter" => Ok(self.render_card_footer(cx, children)),
            "CardTitle" => Ok(self.render_card_title(cx, resolved_props)),
            "CardDescription" => Ok(self.render_card_description(cx, resolved_props)),
            "Text" => Ok(self.render_text(cx, resolved_props)),
            "Heading" => Ok(self.render_heading(cx, resolved_props)),
            "Stack" => Ok(self.render_stack(cx, resolved_props, children)),
            "VStack" => Ok(self.render_vstack(cx, resolved_props, children)),
            "HStack" => Ok(self.render_hstack(cx, resolved_props, children)),
            "Box" => Ok(self.render_box(cx, resolved_props, children)),
            "Separator" => Ok(self.render_separator(cx, resolved_props)),
            "ScrollArea" => Ok(self.render_scroll_area(cx, resolved_props, children)),
            "Button" => Ok(self.render_button(cx, element, resolved_props, children, on_event)),
            "Form" => Ok(self.render_form(cx, key, children, on_event)),
            "Input" => Ok(self.render_input(cx, key, props, children)),
            "Textarea" => Ok(self.render_textarea(cx, key, props, children)),
            "Switch" => Ok(self.render_switch(cx, key, props, children)),
            "Checkbox" => Ok(self.render_checkbox(cx, key, props, children)),
            "Select" => Ok(self.render_select(cx, key, props, children)),
            "RadioGroup" => Ok(self.render_radio_group(cx, key, props, children)),
            "Slider" => Ok(self.render_slider(cx, key, props, children)),
            "Label" => Ok(self.render_label(cx, resolved_props, children)),
            "Alert" => Ok(self.render_alert(cx, key, props, children)),
            "Progress" => Ok(self.render_progress(cx, props, children)),
            "Spinner" => Ok(self.render_spinner(cx, props, children)),
            "Skeleton" => Ok(self.render_skeleton(cx, props, children)),
            "Badge" => Ok(self.render_badge(cx, resolved_props, children)),
            "Table" => Ok(self.render_table(cx, key, props)),
            "Avatar" => Ok(self.render_avatar(cx, resolved_props, children)),
            "Pagination" => Ok(self.render_pagination(cx, key, resolved_props)),
            "Tooltip" => Ok(self.render_tooltip(cx, resolved_props, children)),
            "Popover" => Ok(self.render_popover(cx, resolved_props, children)),
            "DropdownMenu" => Ok(self.render_dropdown_menu(cx, key, props)),
            "Dialog" => Ok(self.render_dialog(cx, resolved_props, children)),
            "Drawer" => Ok(self.render_drawer(cx, resolved_props, children)),
            "BarChart" => Ok(self.render_bar_chart(cx, key, props)),
            "LineChart" => Ok(self.render_line_chart(cx, key, props)),
            "ResponsiveGrid" => Ok(self.render_responsive_grid(cx, resolved_props, children)),
            "ResponsiveStack" => Ok(self.render_responsive_stack(cx, resolved_props, children)),
            other => Ok(self.unknown_component(cx, key, other)),
        }
    }
}

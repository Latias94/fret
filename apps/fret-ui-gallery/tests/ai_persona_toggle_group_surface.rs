mod support;

use support::read;

#[test]
fn ai_persona_snippets_use_required_single_select_toggle_groups() {
    let persona_demo = read("src/ui/snippets/ai/persona_demo.rs");
    let persona_state_management = read("src/ui/snippets/ai/persona_state_management.rs");

    assert!(persona_demo.contains("shadcn::ToggleGroup::single(&state_model)"));
    assert!(persona_demo.contains("shadcn::ToggleGroup::single(&variant_model)"));
    assert!(persona_demo.contains(".deselectable(false)"));
    assert!(persona_demo.matches(".deselectable(false)").count() >= 2);
    assert!(!persona_demo.contains("shadcn::ButtonGroup::new(["));
    assert!(!persona_demo.contains("shadcn::ButtonVariant::Default"));

    assert!(persona_state_management.contains("shadcn::ToggleGroup::single(&state_model)"));
    assert!(persona_state_management.contains(".deselectable(false)"));
    assert!(!persona_state_management.contains("shadcn::ButtonGroup::new(["));
    assert!(!persona_state_management.contains("shadcn::ButtonVariant::Default"));
}

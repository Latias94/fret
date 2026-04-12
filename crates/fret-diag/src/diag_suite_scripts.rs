pub(crate) fn ui_gallery_suite_scripts() -> Vec<String> {
    vec!["tools/diag-scripts/suites/ui-gallery".to_string()]
}

pub(crate) fn ui_gallery_code_editor_suite_scripts() -> Vec<String> {
    vec!["tools/diag-scripts/suites/ui-gallery-code-editor".to_string()]
}

pub(crate) fn ui_gallery_overlay_steady_suite_scripts() -> Vec<String> {
    vec!["tools/diag-scripts/suites/ui-gallery-overlay-steady".to_string()]
}

pub(crate) fn ui_gallery_date_picker_suite_scripts() -> Vec<String> {
    vec!["tools/diag-scripts/suites/ui-gallery-date-picker".to_string()]
}

pub(crate) fn ui_gallery_text_ime_suite_scripts() -> Vec<String> {
    vec!["tools/diag-scripts/suites/ui-gallery-text-ime".to_string()]
}

pub(crate) fn ui_gallery_text_wrap_suite_scripts() -> Vec<String> {
    vec!["tools/diag-scripts/suites/ui-gallery-text-wrap".to_string()]
}

pub(crate) fn ui_gallery_combobox_suite_scripts() -> Vec<String> {
    vec!["tools/diag-scripts/suites/ui-gallery-combobox".to_string()]
}

pub(crate) fn ui_gallery_select_suite_scripts() -> Vec<String> {
    vec!["tools/diag-scripts/suites/ui-gallery-select".to_string()]
}

pub(crate) fn ui_gallery_shadcn_conformance_suite_scripts() -> Vec<String> {
    vec!["tools/diag-scripts/suites/ui-gallery-shadcn-conformance".to_string()]
}

pub(crate) fn ui_gallery_layout_suite_scripts() -> Vec<String> {
    vec!["tools/diag-scripts/suites/ui-gallery-layout".to_string()]
}

pub(crate) fn docking_arbitration_suite_scripts() -> Vec<String> {
    #[cfg(target_os = "windows")]
    {
        let mut inputs = vec!["tools/diag-scripts/suites/docking-arbitration/common".to_string()];
        inputs.push("tools/diag-scripts/suites/docking-arbitration/windows".to_string());
        inputs
    }

    #[cfg(not(target_os = "windows"))]
    {
        vec!["tools/diag-scripts/suites/docking-arbitration/common".to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docking_arbitration_suite_uses_platform_split_inputs() {
        #[cfg(target_os = "windows")]
        assert_eq!(
            docking_arbitration_suite_scripts(),
            vec![
                "tools/diag-scripts/suites/docking-arbitration/common".to_string(),
                "tools/diag-scripts/suites/docking-arbitration/windows".to_string(),
            ]
        );

        #[cfg(not(target_os = "windows"))]
        assert_eq!(
            docking_arbitration_suite_scripts(),
            vec!["tools/diag-scripts/suites/docking-arbitration/common".to_string()]
        );
    }
}

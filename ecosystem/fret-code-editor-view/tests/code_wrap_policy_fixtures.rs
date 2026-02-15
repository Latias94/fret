use fret_code_editor_view::code_wrap_policy::{
    CodeWrapKnobs, CodeWrapPolicy, CodeWrapPreset, row_starts_for_code_wrap,
};
use fret_text_nav::is_grapheme_boundary;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Suite {
    schema_version: u32,
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
struct Case {
    id: String,
    text: String,
    wrap_cols: usize,
    preset: Preset,
    expected_rows: Vec<String>,
    knobs: Option<Knobs>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "PascalCase")]
enum Preset {
    Conservative,
    Balanced,
    Aggressive,
}

impl Preset {
    fn into_preset(self) -> CodeWrapPreset {
        match self {
            Preset::Conservative => CodeWrapPreset::Conservative,
            Preset::Balanced => CodeWrapPreset::Balanced,
            Preset::Aggressive => CodeWrapPreset::Aggressive,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct Knobs {
    break_after_path_separators: bool,
    break_after_url_separators: bool,
    break_after_punctuation: bool,
    break_at_identifier_boundaries: bool,
    break_around_operators: bool,
}

impl Knobs {
    fn into_knobs(self) -> CodeWrapKnobs {
        CodeWrapKnobs {
            break_after_path_separators: self.break_after_path_separators,
            break_after_url_separators: self.break_after_url_separators,
            break_after_punctuation: self.break_after_punctuation,
            break_at_identifier_boundaries: self.break_at_identifier_boundaries,
            break_around_operators: self.break_around_operators,
        }
    }
}

fn rows_from_starts(
    text: &str,
    starts: &[fret_code_editor_view::code_wrap_policy::CodeWrapRowStart],
) -> Vec<String> {
    let mut out = Vec::<String>::new();
    for (idx, start) in starts.iter().enumerate() {
        let end = starts
            .get(idx + 1)
            .map(|v| v.byte)
            .unwrap_or_else(|| text.len());
        out.push(text.get(start.byte..end).unwrap_or("").to_string());
    }
    out
}

#[test]
fn code_wrap_policy_fixture_suite_v1() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/code_wrap_policy_v1.json"
    ));
    let suite: Suite = serde_json::from_str(raw).expect("parse fixture suite");
    assert_eq!(suite.schema_version, 1);

    for case in suite.cases {
        let preset = case.preset.into_preset();
        let mut policy = CodeWrapPolicy::preset(preset);
        if let Some(knobs) = case.knobs {
            policy.knobs = knobs.into_knobs();
        }

        let starts = row_starts_for_code_wrap(&case.text, case.wrap_cols, policy);
        assert!(
            starts.first().is_some_and(|v| v.byte == 0),
            "expected first row start to be at 0: id={}",
            case.id
        );
        for (idx, start) in starts.iter().enumerate() {
            assert!(
                is_grapheme_boundary(&case.text, start.byte),
                "expected row start to be a grapheme boundary: id={} byte={}",
                case.id,
                start.byte
            );
            let end = starts
                .get(idx + 1)
                .map(|v| v.byte)
                .unwrap_or_else(|| case.text.len());
            assert!(
                is_grapheme_boundary(&case.text, end),
                "expected row end to be a grapheme boundary: id={} byte={}",
                case.id,
                end
            );
        }

        let got = rows_from_starts(&case.text, &starts);
        assert_eq!(
            got, case.expected_rows,
            "fixture case failed: id={}",
            case.id
        );

        let joined = got.join("");
        assert_eq!(
            joined, case.text,
            "fixture case must be lossless: id={}",
            case.id
        );
    }
}

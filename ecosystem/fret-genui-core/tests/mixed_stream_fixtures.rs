use fret_genui_core::mixed_stream::{MixedSpecStreamCompiler, MixedStreamMode, MixedStreamOptions};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct Fixture {
    name: String,
    options: Option<FixtureOptions>,
    chunks: Vec<String>,
    expect: Expect,
}

#[derive(Debug, Deserialize)]
struct FixtureOptions {
    mode: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Expect {
    #[serde(default)]
    text_lines: Vec<String>,
    #[serde(default)]
    patches: Vec<fret_genui_core::spec_stream::JsonPatch>,
    result: Option<Value>,
    error_contains: Option<String>,
}

fn load_fixture(json: &str) -> Fixture {
    serde_json::from_str(json).expect("fixture json must parse")
}

fn run_fixture(fx: Fixture) {
    let mut opts = MixedStreamOptions::default();
    if let Some(mode) = fx.options.and_then(|o| o.mode) {
        opts.mode = match mode.as_str() {
            "mixed" => MixedStreamMode::Mixed,
            "patch_only" => MixedStreamMode::PatchOnly,
            other => panic!("unknown fixture mode: {other}"),
        };
    }

    let mut compiler = MixedSpecStreamCompiler::new(opts);
    let mut text_lines: Vec<String> = Vec::new();
    let mut patches: Vec<fret_genui_core::spec_stream::JsonPatch> = Vec::new();

    for chunk in fx.chunks {
        match compiler.push_chunk(&chunk) {
            Ok(delta) => {
                text_lines.extend(delta.text_lines);
                patches.extend(delta.patches);
            }
            Err(err) => {
                let Some(needle) = fx.expect.error_contains else {
                    panic!("fixture {} unexpected error: {err}", fx.name);
                };
                let s = err.to_string();
                assert!(
                    s.to_lowercase().contains(&needle.to_lowercase()),
                    "fixture {} expected error containing {needle:?}, got {s:?}",
                    fx.name
                );
                return;
            }
        }
    }

    match compiler.flush() {
        Ok(delta) => {
            text_lines.extend(delta.text_lines);
            patches.extend(delta.patches);
        }
        Err(err) => {
            let Some(needle) = fx.expect.error_contains else {
                panic!("fixture {} unexpected flush error: {err}", fx.name);
            };
            let s = err.to_string();
            assert!(
                s.to_lowercase().contains(&needle.to_lowercase()),
                "fixture {} expected error containing {needle:?}, got {s:?}",
                fx.name
            );
            return;
        }
    }

    if let Some(needle) = fx.expect.error_contains {
        panic!(
            "fixture {} expected error containing {needle:?}, but stream succeeded",
            fx.name
        );
    }

    assert_eq!(
        text_lines, fx.expect.text_lines,
        "fixture {} text_lines mismatch",
        fx.name
    );
    assert_eq!(
        patches, fx.expect.patches,
        "fixture {} patches mismatch",
        fx.name
    );

    if let Some(expected) = fx.expect.result {
        assert_eq!(
            compiler.result(),
            &expected,
            "fixture {} result mismatch",
            fx.name
        );
    }
}

#[test]
fn mixed_stream_fixtures() {
    let fixtures = [
        load_fixture(include_str!("fixtures/mixed_stream/heuristic_basic.json")),
        load_fixture(include_str!("fixtures/mixed_stream/fence_basic.json")),
        load_fixture(include_str!("fixtures/mixed_stream/partial_chunks.json")),
        load_fixture(include_str!("fixtures/mixed_stream/patch_only_error.json")),
    ];

    for fx in fixtures {
        run_fixture(fx);
    }
}

#[path = "../src/launch.rs"]
mod launch;

use std::{fs, path::Path};

#[test]
fn fzts_defaults_to_local_web() {
    let command = launch::parse_invocation("fzts", &[]).expect("fzts should parse");

    assert_eq!(
        command,
        launch::Command::Web {
            local: true,
            passthrough: Vec::new()
        }
    );
}

#[test]
fn fzts_play_web_is_alias_for_local_web() {
    let command = launch::parse_invocation(
        "fzts",
        &["play".into(), "web".into(), "--port".into(), "9000".into()],
    )
    .expect("fzts play web should parse");

    assert_eq!(
        command,
        launch::Command::Web {
            local: true,
            passthrough: vec!["--port".into(), "9000".into()]
        }
    );
}

#[test]
fn play_without_args_still_launches_tui() {
    let command = launch::parse_invocation("play", &[]).expect("play should parse");

    assert_eq!(command, launch::Command::Tui);
}

#[test]
fn fzts_shared_web_bundle_exposes_concepts_and_docs_surfaces() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../challenge-host/crates/host-web/web/from-zero-to-systems");
    let html = fs::read_to_string(root.join("index.html")).expect("from zero to systems html should exist");
    let css = fs::read_to_string(root.join("styles.css")).expect("from zero to systems css should exist");
    let js = fs::read_to_string(root.join("app.js")).expect("from zero to systems js should exist");

    assert!(html.contains("<title>from zero to systems</title>"));
    assert!(html.contains("href=\"/styles.css\""));
    assert!(js.contains("from zero to systems"));
    assert!(js.contains("Concepts"));
    assert!(js.contains("Docs"));
    assert!(js.contains("/api/bootstrap"));
    assert!(js.contains("/api/test"));
    assert!(css.contains("--accent:#000"));
    assert!(css.contains(".workspace-grid"));
}

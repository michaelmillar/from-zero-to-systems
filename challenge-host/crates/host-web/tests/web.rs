use std::{fs, os::unix::fs::PermissionsExt};

use host_web::{AdapterSpec, BootstrapResponse, WebApp};
use tempfile::tempdir;

#[test]
fn bootstrap_returns_handshake_challenges_and_workspace() {
    let dir = tempdir().unwrap();
    let script_path = dir.path().join("fake-adapter.sh");
    write_fake_adapter(&script_path);

    let mut app = WebApp::new(AdapterSpec {
        program: script_path.to_string_lossy().to_string(),
        args: Vec::new(),
        cwd: None,
    })
    .unwrap();

    let response = app.handle("GET", "/api/bootstrap", &[]).unwrap();
    assert_eq!(response.status, "200 OK");

    let payload: BootstrapResponse = serde_json::from_slice(&response.body).unwrap();
    assert_eq!(payload.handshake.game_id, "hazptr");
    assert_eq!(
        payload.challenges.current_challenge.as_deref(),
        Some("sorting/01_bubble_sort")
    );
    assert_eq!(payload.workspace.challenge_id, "sorting/01_bubble_sort");
    assert_eq!(payload.workspace.hints.len(), 0);
    assert_eq!(payload.progress.completed, 3);
    assert_eq!(payload.progress.total, 12);
    assert_eq!(payload.progress.streak_days, Some(4));
    assert_eq!(payload.progress.activity.len(), 2);
    assert_eq!(payload.progress.languages.len(), 2);
    assert_eq!(payload.progress.languages[0].language, "python");
    assert_eq!(payload.progress.activity[0].commits, 1);
}

#[test]
fn reveal_hint_returns_updated_workspace_without_leaking_future_hints() {
    let dir = tempdir().unwrap();
    let script_path = dir.path().join("fake-adapter.sh");
    write_fake_adapter(&script_path);

    let mut app = WebApp::new(AdapterSpec {
        program: script_path.to_string_lossy().to_string(),
        args: Vec::new(),
        cwd: None,
    })
    .unwrap();

    let response = app
        .handle(
            "POST",
            "/api/reveal-hint",
            br#"{"challenge_id":"sorting/01_bubble_sort","language":"python"}"#,
        )
        .unwrap();
    assert_eq!(response.status, "200 OK");

    let workspace: host_protocol::WorkspaceView = serde_json::from_slice(&response.body).unwrap();
    assert_eq!(workspace.hints.len(), 1);
    assert_eq!(workspace.hints[0].label, "swap adjacent items");
    assert_eq!(workspace.hint_state.revealed_count, 1);
    assert_eq!(workspace.hint_state.total_count, 2);
}

#[test]
fn web_assets_restore_unsaved_drafts_when_returning_to_workspace() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let app_js = manifest_dir.join("web").join("app.js");
    let script = fs::read_to_string(&app_js).unwrap();

    assert!(
        script.contains("drafts: {}"),
        "expected per-workspace draft state in the browser client"
    );
    assert!(
        script.contains("workspaceKey("),
        "expected a workspace key helper for draft lookup"
    );
    assert!(
        script.contains("editor.value = draft.content"),
        "expected draft content to be restored when revisiting a workspace"
    );
    assert!(
        script.contains("rememberCurrentDraft(editor.value);"),
        "expected navigation to snapshot the current editor draft before loading another workspace"
    );
}

#[test]
fn web_assets_capture_tab_in_editor() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let app_js = manifest_dir.join("web").join("app.js");
    let script = fs::read_to_string(&app_js).unwrap();

    assert!(
        script.contains("editor.addEventListener(\"keydown\""),
        "expected the browser client to handle editor keydown events"
    );
    assert!(
        script.contains("event.preventDefault()"),
        "expected the browser client to prevent focus navigation on Tab"
    );
    assert!(
        script.contains("editor.setRangeText"),
        "expected the browser client to insert indentation on Tab"
    );
}

#[test]
fn web_assets_move_navigation_and_notes_into_left_column() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let index_html = manifest_dir.join("web").join("index.html");
    let html = fs::read_to_string(&index_html).unwrap();

    let intro_panel = html
        .find("<aside class=\"intro-panel section\">")
        .expect("left intro panel should exist");
    let challenge_list = html
        .find("id=\"challenge-list\"")
        .expect("challenge list should exist");
    let editor = html.find("id=\"editor\"").expect("editor should exist");

    assert!(
        intro_panel < challenge_list && challenge_list < editor,
        "expected the challenge list to live in the left column before the middle editor"
    );
    assert!(
        html.contains("id=\"challenge-notes-copy\""),
        "expected a left-column surface for challenge notes extracted from starter comments"
    );
    assert!(
        !html.contains("class=\"challenge-strip section\""),
        "expected the old top challenge strip to be removed"
    );
}

#[test]
fn web_assets_support_bidirectional_keyboard_navigation_and_editor_focus() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let app_js = manifest_dir.join("web").join("app.js");
    let script = fs::read_to_string(&app_js).unwrap();

    assert!(
        script.contains("event.key === \"ArrowUp\""),
        "expected challenge navigation to support moving backward with the keyboard"
    );
    assert!(
        script.contains("event.key === \"ArrowDown\""),
        "expected challenge navigation to support moving forward with the keyboard"
    );
    assert!(
        script.contains("focusEditor()"),
        "expected the browser client to restore focus to the editor after workspace loads"
    );
}

#[test]
fn web_assets_extract_leading_challenge_notes_from_editor_source() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let app_js = manifest_dir.join("web").join("app.js");
    let script = fs::read_to_string(&app_js).unwrap();

    assert!(
        script.contains("splitLeadingCommentary("),
        "expected the browser client to extract challenge notes from leading source comments"
    );
    assert!(
        script.contains("challengeNotesCopy.textContent"),
        "expected extracted challenge notes to render in the left column"
    );
    assert!(
        script.contains("composeEditorContent("),
        "expected save/test requests to rebuild full source content from notes plus editor text"
    );
}

#[test]
fn web_assets_render_progress_heatmap() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let index_html = manifest_dir.join("web").join("index.html");
    let app_js = manifest_dir.join("web").join("app.js");
    let styles = manifest_dir.join("web").join("styles.css");

    let html = fs::read_to_string(&index_html).unwrap();
    let script = fs::read_to_string(&app_js).unwrap();
    let css = fs::read_to_string(&styles).unwrap();

    assert!(
        html.contains("id=\"language-completion-map\""),
        "expected the shared host to expose a language completion map container"
    );
    assert!(
        html.contains("id=\"activity-heatmap\""),
        "expected the shared host to expose a year heatmap container"
    );
    assert!(
        script.contains("renderLanguageProgress("),
        "expected the browser client to render the per-language completion map"
    );
    assert!(
        script.contains("renderActivityHeatmap("),
        "expected the browser client to render the year activity heatmap separately"
    );
    assert!(
        script.contains("solutions"),
        "expected daily heatmap tooltips to surface solution counts"
    );
    assert!(
        !script.contains("commits + \" commits\""),
        "expected commit counts to be removed from the shared heatmap tooltip"
    );
    assert!(
        css.contains(".language-map"),
        "expected styles for the language completion map"
    );
    assert!(
        css.contains(".year-heatmap-grid"),
        "expected styles for the year heatmap grid"
    );
}

#[test]
fn web_assets_render_repo_specific_challenge_symbols() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let app_js = manifest_dir.join("web").join("app.js");
    let styles = manifest_dir.join("web").join("styles.css");

    let script = fs::read_to_string(&app_js).unwrap();
    let css = fs::read_to_string(&styles).unwrap();

    assert!(
        script.contains("symbolForChallenge("),
        "expected the shared host to derive a challenge symbol per repo"
    );
    assert!(
        script.contains("challenge.badges"),
        "expected adapter-provided badges to override the fallback symbol map"
    );
    assert!(
        script.contains("\"hazptr\"")
            && script.contains("\"fzts\"")
            && script.contains("\"compilerlings\""),
        "expected repo-specific symbol mapping for the shared host"
    );
    assert!(
        css.contains(".challenge-kind-symbol"),
        "expected dedicated styles for challenge kind symbols"
    );
    assert!(
        css.contains(".challenge-status-symbol"),
        "expected dedicated styles for challenge completion symbols"
    );
}

fn write_fake_adapter(path: &std::path::Path) {
    fs::write(
        path,
        r#"#!/usr/bin/env bash
set -euo pipefail

while IFS= read -r line; do
  id="0"
  if [[ "$line" =~ \"id\":\"([^\"]+)\" ]]; then
    id="${BASH_REMATCH[1]}"
  fi

  if [[ "$line" == *'"method":"handshake"'* ]]; then
    response='{"id":"__ID__","ok":true,"result":{"type":"handshake","value":{"game_id":"hazptr","title":"hazptr","capabilities":{"multi_language":true,"incremental_hints":true,"benchmark":true,"explain":true,"compare":false,"idea_tools":false,"synthesis":false}}}}'
  elif [[ "$line" == *'"method":"list_challenges"'* ]]; then
    response='{"id":"__ID__","ok":true,"result":{"type":"challenge_list","value":{"current_challenge":"sorting/01_bubble_sort","current_language":"python","challenges":[{"id":"sorting/01_bubble_sort","title":"Bubble Sort","track":"sorting","difficulty":null,"status":"in_progress","available_languages":["python"],"badges":[]}]}}}'
  elif [[ "$line" == *'"method":"load_workspace"'* ]]; then
    response='{"id":"__ID__","ok":true,"result":{"type":"workspace","value":{"challenge_id":"sorting/01_bubble_sort","title":"Bubble Sort","language":"python","editor":{"file_path":"tracks/sorting/01_bubble_sort/python/solution.py","content":"def bubble_sort(arr):\n    return arr\n","can_reset":true},"intro":"A first pass.","guide":"Bubble Sort\nA first pass.","concepts":[],"docs":[],"hints":[],"hint_state":{"mode":"incremental","revealed_count":0,"total_count":2,"next_cost":5},"actions":{"can_save":true,"can_test":true,"can_reveal_hint":true,"can_benchmark":true,"can_compare":false}}}}'
  elif [[ "$line" == *'"method":"load_progress"'* ]]; then
    response='{"id":"__ID__","ok":true,"result":{"type":"progress","value":{"completed":3,"total":12,"streak_days":4,"score":120,"languages":[{"language":"python","completed":2,"total":6},{"language":"rust","completed":1,"total":6}],"activity":[{"date":"2026-03-10","check_ins":2,"completed":1,"commits":1},{"date":"2026-03-11","check_ins":1,"completed":0,"commits":0}]}}}'
  elif [[ "$line" == *'"method":"reveal_hint"'* ]]; then
    response='{"id":"__ID__","ok":true,"result":{"type":"workspace","value":{"challenge_id":"sorting/01_bubble_sort","title":"Bubble Sort","language":"python","editor":{"file_path":"tracks/sorting/01_bubble_sort/python/solution.py","content":"def bubble_sort(arr):\n    return arr\n","can_reset":true},"intro":"A first pass.","guide":"Bubble Sort\nA first pass.","concepts":[],"docs":[],"hints":[{"label":"swap adjacent items","body":"Compare neighbors and swap them when they are out of order.","cost":5}],"hint_state":{"mode":"incremental","revealed_count":1,"total_count":2,"next_cost":8},"actions":{"can_save":true,"can_test":true,"can_reveal_hint":true,"can_benchmark":true,"can_compare":false}}}}'
  else
    response='{"id":"__ID__","ok":false,"error":{"code":"unexpected_request","message":"unexpected request"}}'
  fi

  printf '%s\n' "${response/__ID__/$id}"
done
"#,
    )
    .unwrap();

    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

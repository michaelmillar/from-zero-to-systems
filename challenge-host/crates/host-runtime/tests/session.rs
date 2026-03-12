use std::{fs, os::unix::fs::PermissionsExt, path::Path};

use host_runtime::HostSession;
use tempfile::tempdir;

#[test]
fn load_overview_round_trips_through_adapter_process() {
    let dir = tempdir().unwrap();
    let script_path = dir.path().join("fake-adapter.sh");
    write_fake_adapter(&script_path);

    let mut session = HostSession::connect(script_path.to_str().unwrap(), &[]).unwrap();
    let overview = session.load_overview().unwrap();

    assert_eq!(overview.handshake.game_id, "hazptr");
    assert_eq!(overview.handshake.title, "hazptr");
    assert_eq!(
        overview.challenges.current_challenge.as_deref(),
        Some("sorting/01_bubble_sort")
    );
    assert_eq!(overview.workspace.challenge_id, "sorting/01_bubble_sort");
    assert_eq!(overview.workspace.language.as_deref(), Some("python"));
    assert_eq!(overview.workspace.hints.len(), 0);
    assert_eq!(overview.progress.completed, 3);
    assert_eq!(overview.progress.total, 12);
    assert_eq!(overview.progress.streak_days, Some(4));
    assert_eq!(overview.progress.activity.len(), 2);
    assert_eq!(overview.progress.languages.len(), 2);
    assert_eq!(overview.progress.languages[0].language, "python");
    assert_eq!(overview.progress.languages[0].completed, 2);
    assert_eq!(overview.progress.languages[0].total, 6);
    assert_eq!(overview.progress.activity[0].commits, 1);
}

#[test]
fn load_overview_can_spawn_adapter_relative_to_custom_cwd() {
    let dir = tempdir().unwrap();
    let script_path = dir.path().join("fake-adapter.sh");
    write_fake_adapter(&script_path);

    let mut session = HostSession::connect_in("./fake-adapter.sh", &[], Some(dir.path())).unwrap();
    let overview = session.load_overview().unwrap();

    assert_eq!(overview.handshake.game_id, "hazptr");
    assert_eq!(overview.workspace.challenge_id, "sorting/01_bubble_sort");
}

fn write_fake_adapter(path: &Path) {
    fs::write(
        path,
        r#"#!/usr/bin/env bash
set -euo pipefail

while IFS= read -r line; do
  if [[ "$line" == *'"method":"handshake"'* ]]; then
    printf '%s\n' '{"id":"1","ok":true,"result":{"type":"handshake","value":{"game_id":"hazptr","title":"hazptr","capabilities":{"multi_language":true,"incremental_hints":true,"benchmark":true,"explain":true,"compare":false,"idea_tools":false,"synthesis":false}}}}'
  elif [[ "$line" == *'"method":"list_challenges"'* ]]; then
    printf '%s\n' '{"id":"2","ok":true,"result":{"type":"challenge_list","value":{"current_challenge":"sorting/01_bubble_sort","current_language":"python","challenges":[{"id":"sorting/01_bubble_sort","title":"Bubble Sort","track":"sorting","difficulty":null,"status":"in_progress","available_languages":["python"],"badges":[]}]}}}'
  elif [[ "$line" == *'"method":"load_workspace"'* ]]; then
    printf '%s\n' '{"id":"3","ok":true,"result":{"type":"workspace","value":{"challenge_id":"sorting/01_bubble_sort","title":"Bubble Sort","language":"python","editor":{"file_path":"tracks/sorting/01_bubble_sort/python/solution.py","content":"def bubble_sort(arr):\n    return arr\n","can_reset":true},"intro":"A first pass.","guide":"Bubble Sort\nA first pass.","concepts":[],"docs":[],"hints":[],"hint_state":{"mode":"incremental","revealed_count":0,"total_count":2,"next_cost":5},"actions":{"can_save":true,"can_test":true,"can_reveal_hint":true,"can_benchmark":true,"can_compare":false}}}}'
  elif [[ "$line" == *'"method":"load_progress"'* ]]; then
    printf '%s\n' '{"id":"4","ok":true,"result":{"type":"progress","value":{"completed":3,"total":12,"streak_days":4,"score":120,"languages":[{"language":"python","completed":2,"total":6},{"language":"rust","completed":1,"total":6}],"activity":[{"date":"2026-03-10","check_ins":2,"completed":1,"commits":1},{"date":"2026-03-11","check_ins":1,"completed":0,"commits":0}]}}}'
  else
    printf '%s\n' '{"id":"0","ok":false,"error":{"code":"unexpected_request","message":"unexpected request"}}'
  fi
done
"#,
    )
    .unwrap();

    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

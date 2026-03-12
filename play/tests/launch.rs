#[path = "../src/launch.rs"]
mod launch;

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

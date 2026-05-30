use super::helpers::{manager_with_fake_agents, read_json};
use serde_json::json;
use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    process::{Command, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};

const EVENTS: &[(&str, &str)] = &[
    ("beforeSubmitPrompt", "user.prompt"),
    ("preToolUse", "tool.before"),
    ("postToolUse", "tool.after"),
    ("postToolUseFailure", "session.error"),
    ("stop", "session.stop"),
    ("sessionEnd", "session.stop"),
];

#[test]
fn cursor_install_merges_user_hooks() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("home");
    let root = temp.path().join(".copet");
    let hooks_path = home.join(".cursor/hooks.json");
    fs::create_dir_all(hooks_path.parent().unwrap()).unwrap();
    fs::write(
        &hooks_path,
        serde_json::to_vec_pretty(&json!({
            "version": 1,
            "hooks": {
                "preToolUse": [{ "command": "./user-hook.sh", "timeout": 5 }]
            },
            "userSetting": true
        }))
        .unwrap(),
    )
    .unwrap();
    let manager = manager_with_fake_agents(&root, &home);

    let result = manager.install("cursor").unwrap();
    let hooks = read_json(&hooks_path);

    assert!(result.adapter.installed);
    assert_eq!(result.adapter.id, "cursor");
    assert_eq!(result.adapter.display_name, "Cursor");
    assert_eq!(hooks["version"], 1);
    assert_eq!(hooks["userSetting"], true);
    assert!(hooks["hooks"]["preToolUse"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["command"] == "./user-hook.sh"));
    for (event, kind) in EVENTS {
        let entries = hooks["hooks"][event].as_array().unwrap();
        assert!(
            entries.iter().any(|entry| {
                entry["command"].as_str().is_some_and(|command| {
                    command.contains("copet-hook.sh")
                        && command.contains(&format!(" cursor {kind}"))
                })
            }),
            "{event} missing CoPet hook"
        );
    }
}

#[test]
fn cursor_install_is_idempotent_and_repair_keeps_one_copet_entry_per_event() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("home");
    let root = temp.path().join(".copet");
    let manager = manager_with_fake_agents(&root, &home);

    manager.install("cursor").unwrap();
    manager.install("cursor").unwrap();
    manager.repair("cursor").unwrap();

    let hooks = read_json(home.join(".cursor/hooks.json"));
    for (event, kind) in EVENTS {
        let count = hooks["hooks"][event]
            .as_array()
            .unwrap()
            .iter()
            .filter(|entry| {
                entry["command"].as_str().is_some_and(|command| {
                    command.contains("copet-hook.sh")
                        && command.contains(&format!(" cursor {kind}"))
                })
            })
            .count();
        assert_eq!(count, 1, "{event} should have exactly one CoPet hook");
    }
}

#[test]
fn cursor_uninstall_removes_only_copet_hooks() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("home");
    let root = temp.path().join(".copet");
    let hooks_path = home.join(".cursor/hooks.json");
    fs::create_dir_all(hooks_path.parent().unwrap()).unwrap();
    fs::write(
        &hooks_path,
        serde_json::to_vec_pretty(&json!({
            "version": 1,
            "hooks": {
                "preToolUse": [{ "command": "./user-hook.sh" }]
            }
        }))
        .unwrap(),
    )
    .unwrap();
    let manager = manager_with_fake_agents(&root, &home);

    manager.install("cursor").unwrap();
    let result = manager.uninstall("cursor").unwrap();
    let hooks = read_json(&hooks_path);

    assert!(!result.adapter.installed);
    assert!(hooks["hooks"]["preToolUse"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["command"] == "./user-hook.sh"));
    let serialized = hooks.to_string();
    assert!(!serialized.contains("copet-hook.sh"));
    assert!(!root.join("adapters/cursor.json").exists());
}

#[test]
fn cursor_before_submit_prompt_outputs_continue_when_helper_is_missing() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("home");
    let root = temp.path().join(".copet");
    let manager = manager_with_fake_agents(&root, &home);

    manager.install("cursor").unwrap();
    fs::remove_file(root.join("hooks/copet-hook.sh")).unwrap();

    let hooks = read_json(home.join(".cursor/hooks.json"));
    let command = copet_command(&hooks, "beforeSubmitPrompt");
    let output = Command::new("sh")
        .args(["-c", command])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "{\"continue\":true}\n"
    );
}

#[test]
fn cursor_pre_tool_command_posts_tool_input_summary() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("home");
    let root = temp.path().join(".copet");
    let runtime = temp.path().join("runtime");
    let manager = manager_with_fake_agents(&root, &home);

    manager.install("cursor").unwrap();
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    listener.set_nonblocking(true).unwrap();
    let endpoint = format!(
        "http://127.0.0.1:{}/v1/events",
        listener.local_addr().unwrap().port()
    );
    fs::create_dir_all(&runtime).unwrap();
    fs::write(runtime.join("event-endpoint"), &endpoint).unwrap();
    fs::write(runtime.join("event-token"), "secret").unwrap();

    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            match listener.accept() {
                Ok((mut stream, _addr)) => {
                    stream.set_nonblocking(false).unwrap();
                    stream
                        .set_read_timeout(Some(Duration::from_secs(2)))
                        .unwrap();
                    let mut buffer = [0_u8; 4096];
                    let size = stream.read(&mut buffer).unwrap();
                    let request = String::from_utf8_lossy(&buffer[..size]).to_string();
                    let _ =
                        stream.write_all(b"HTTP/1.1 202 Accepted\r\nContent-Length: 2\r\n\r\n{}");
                    sender.send(Some(request)).unwrap();
                    return;
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        sender.send(None).unwrap();
                        return;
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(_) => {
                    sender.send(None).unwrap();
                    return;
                }
            }
        }
    });

    let hooks = read_json(home.join(".cursor/hooks.json"));
    let command = copet_command(&hooks, "preToolUse");
    let mut child = Command::new("sh")
        .args(["-c", command])
        .env("COPET_RUNTIME_DIR", &runtime)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{"conversation_id":"c1","hook_event_name":"preToolUse","cwd":"/repo","tool_name":"Shell","tool_input":{"command":"pnpm test:rust"}}"#,
        )
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "{}\n");

    let request = receiver
        .recv_timeout(Duration::from_secs(11))
        .unwrap()
        .expect("runtime server should receive the Cursor hook event");
    assert!(request.contains("POST /v1/events"));
    assert!(request.contains("Authorization: Bearer secret"));
    assert!(request.contains(r#""agent":"cursor""#));
    assert!(request.contains(r#""kind":"tool.before""#));
    assert!(request.contains(r#""tool":"Shell""#));
    assert!(request.contains(r#""toolInput":{"command":"pnpm test:rust"}"#));
}

fn copet_command<'a>(hooks: &'a serde_json::Value, event: &str) -> &'a str {
    hooks["hooks"][event]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|entry| {
            entry["command"]
                .as_str()
                .filter(|command| command.contains("copet-hook.sh"))
        })
        .unwrap()
}

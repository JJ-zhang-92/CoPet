use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use super::super::{
    hook_command, read_json_object_optional, read_json_object_required, write_json_atomic,
    AdapterError, AgentManager, CliAdapter, HELPER_NAME,
};

pub(super) static ADAPTER: CursorAdapter = CursorAdapter;

struct CursorEvent {
    cli_event: &'static str,
    kind: &'static str,
}

const EVENTS: &[CursorEvent] = &[
    CursorEvent {
        cli_event: "beforeSubmitPrompt",
        kind: "user.prompt",
    },
    CursorEvent {
        cli_event: "preToolUse",
        kind: "tool.before",
    },
    CursorEvent {
        cli_event: "postToolUse",
        kind: "tool.after",
    },
    CursorEvent {
        cli_event: "postToolUseFailure",
        kind: "session.error",
    },
    CursorEvent {
        cli_event: "stop",
        kind: "session.stop",
    },
    CursorEvent {
        cli_event: "sessionEnd",
        kind: "session.stop",
    },
];

pub(super) struct CursorAdapter;

impl CliAdapter for CursorAdapter {
    fn id(&self) -> &'static str {
        "cursor"
    }

    fn display_name(&self) -> &'static str {
        "Cursor"
    }

    fn config_path(&self, home: &Path) -> PathBuf {
        home.join(".cursor").join("hooks.json")
    }

    fn is_installed(
        &self,
        manager: &AgentManager,
        config_path: &Path,
    ) -> Result<bool, AdapterError> {
        cursor_config_has_copet_hooks(config_path, self.id(), &manager.helper_path())
    }

    fn install(&self, manager: &AgentManager) -> Result<(), AdapterError> {
        let path = self.config_path(manager.home());
        manager.backup_file(self.id(), &path)?;
        let mut value = read_json_object_optional(&path)?.unwrap_or_else(|| json!({}));
        install_cursor_hooks(&mut value, self.id(), &manager.helper_path(), &path)?;
        write_json_atomic(&path, &value)
    }

    fn uninstall(&self, manager: &AgentManager) -> Result<(), AdapterError> {
        let path = self.config_path(manager.home());
        if !path.exists() {
            return Ok(());
        }

        manager.backup_file(self.id(), &path)?;
        let mut value = read_json_object_required(&path)?;
        remove_cursor_hooks(&mut value, self.id(), &path)?;
        write_json_atomic(&path, &value)
    }

    fn executable_names(&self) -> &'static [&'static str] {
        &["cursor-agent", "cursor"]
    }
}

fn install_cursor_hooks(
    value: &mut Value,
    adapter_id: &str,
    helper_path: &Path,
    path: &Path,
) -> Result<(), AdapterError> {
    let object = value
        .as_object_mut()
        .ok_or_else(|| AdapterError::InvalidJson(path.to_path_buf()))?;
    object.entry("version").or_insert_with(|| json!(1));
    remove_cursor_hooks(value, adapter_id, path)?;

    let object = value
        .as_object_mut()
        .ok_or_else(|| AdapterError::InvalidJson(path.to_path_buf()))?;
    let hooks = object.entry("hooks").or_insert_with(|| json!({}));
    let hooks_object = hooks
        .as_object_mut()
        .ok_or_else(|| AdapterError::InvalidJson(path.to_path_buf()))?;

    for event in EVENTS {
        hooks_object
            .entry(event.cli_event)
            .or_insert_with(|| json!([]))
            .as_array_mut()
            .ok_or_else(|| AdapterError::InvalidJson(path.to_path_buf()))?
            .push(json!({
                "command": hook_command(adapter_id, helper_path, event.kind),
                "timeout": 1
            }));
    }

    Ok(())
}

fn remove_cursor_hooks(
    value: &mut Value,
    adapter_id: &str,
    path: &Path,
) -> Result<(), AdapterError> {
    let Some(hooks) = value.get_mut("hooks") else {
        return Ok(());
    };
    let hooks = hooks
        .as_object_mut()
        .ok_or_else(|| AdapterError::InvalidJson(path.to_path_buf()))?;

    for event in EVENTS {
        let Some(entries) = hooks.get_mut(event.cli_event) else {
            continue;
        };
        let entries = entries
            .as_array_mut()
            .ok_or_else(|| AdapterError::InvalidJson(path.to_path_buf()))?;
        entries.retain(|entry| {
            !entry
                .get("command")
                .and_then(Value::as_str)
                .is_some_and(|command| is_cursor_copet_command(command, adapter_id))
        });
    }

    Ok(())
}

fn cursor_config_has_copet_hooks(
    path: &Path,
    adapter_id: &str,
    expected_helper_path: &Path,
) -> Result<bool, AdapterError> {
    let Some(value) = read_json_object_optional(path)? else {
        return Ok(false);
    };
    let Some(hooks) = value.get("hooks").and_then(Value::as_object) else {
        return Ok(false);
    };

    Ok(EVENTS.iter().all(|event| {
        hooks
            .get(event.cli_event)
            .and_then(Value::as_array)
            .is_some_and(|entries| {
                entries.iter().any(|entry| {
                    entry
                        .get("command")
                        .and_then(Value::as_str)
                        .is_some_and(|command| {
                            is_cursor_copet_kind_command(
                                command,
                                adapter_id,
                                event.kind,
                                expected_helper_path,
                            )
                        })
                })
            })
    }))
}

fn is_cursor_copet_command(command: &str, adapter_id: &str) -> bool {
    cursor_command_matches(command, adapter_id, None, None)
}

fn is_cursor_copet_kind_command(
    command: &str,
    adapter_id: &str,
    kind: &str,
    expected_helper_path: &Path,
) -> bool {
    cursor_command_matches(command, adapter_id, Some(kind), Some(expected_helper_path))
}

fn cursor_command_matches(
    command: &str,
    adapter_id: &str,
    kind: Option<&str>,
    expected_helper_path: Option<&Path>,
) -> bool {
    let Some(rest) = command.strip_prefix("if [ -f ") else {
        return false;
    };
    let Some((guard_segment, after_then)) = split_once_unquoted(rest, "; then ") else {
        return false;
    };
    let Some((then_invocation, _)) = split_once_unquoted(after_then, "; else ") else {
        return false;
    };
    let Some(guard_path) = parse_guard_path(guard_segment.trim()) else {
        return false;
    };

    invocation_matches_cursor_helper(
        then_invocation.trim(),
        adapter_id,
        kind,
        &guard_path,
        expected_helper_path,
    )
}

fn parse_guard_path(segment: &str) -> Option<String> {
    let (path, rest) = parse_shell_quoted_word(segment)?;
    if rest.trim() == "]" {
        Some(path)
    } else {
        None
    }
}

fn invocation_matches_cursor_helper(
    invocation: &str,
    adapter_id: &str,
    kind: Option<&str>,
    guard_path: &str,
    expected_helper_path: Option<&Path>,
) -> bool {
    let Some((helper_path, args)) = parse_shell_quoted_word(invocation) else {
        return false;
    };
    if helper_path != guard_path {
        return false;
    }
    if expected_helper_path.is_some_and(|expected| helper_path != expected.to_string_lossy()) {
        return false;
    }

    let helper_file_name = Path::new(&helper_path)
        .file_name()
        .and_then(|name| name.to_str());
    if helper_file_name != Some(HELPER_NAME) {
        return false;
    }

    let mut args = args.split_whitespace();
    if args.next() != Some(adapter_id) {
        return false;
    }

    match kind {
        Some(kind) => args.next() == Some(kind) && args.next().is_none(),
        None => args.next().is_some() && args.next().is_none(),
    }
}

fn split_once_unquoted<'a>(input: &'a str, delimiter: &str) -> Option<(&'a str, &'a str)> {
    let mut in_single_quote = false;
    let mut escaped_outside_quote = false;

    for (index, ch) in input.char_indices() {
        if escaped_outside_quote {
            escaped_outside_quote = false;
            continue;
        }
        if !in_single_quote && ch == '\\' {
            escaped_outside_quote = true;
            continue;
        }
        if ch == '\'' {
            in_single_quote = !in_single_quote;
            continue;
        }
        if !in_single_quote && input[index..].starts_with(delimiter) {
            return Some((&input[..index], &input[index + delimiter.len()..]));
        }
    }

    None
}

fn parse_shell_quoted_word(input: &str) -> Option<(String, &str)> {
    if !input.starts_with('\'') {
        return None;
    }

    let mut value = String::new();
    let mut in_single_quote = false;
    let mut index = 0;
    let mut parsed_any = false;

    while index < input.len() {
        let rest = &input[index..];
        if in_single_quote {
            if rest.starts_with('\'') {
                in_single_quote = false;
                index += 1;
                continue;
            }

            let ch = rest.chars().next()?;
            value.push(ch);
            parsed_any = true;
            index += ch.len_utf8();
            continue;
        }

        if rest.starts_with('\'') {
            in_single_quote = true;
            index += 1;
            continue;
        }
        if rest.starts_with("\\'") {
            value.push('\'');
            parsed_any = true;
            index += 2;
            continue;
        }

        let ch = rest.chars().next()?;
        if ch.is_whitespace() && parsed_any {
            return Some((value, &input[index..]));
        }
        return None;
    }

    if !in_single_quote && parsed_any {
        Some((value, ""))
    } else {
        None
    }
}

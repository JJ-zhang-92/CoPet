use std::path::{Path, PathBuf};

use toml_edit::{value, DocumentMut, Item, Table};

use super::super::{
    install_json_hooks, json_config_has_pethover_hook, remove_json_hooks, write_atomic,
    AdapterError, AgentManager, CliAdapter, HookEvent,
};

pub(super) static ADAPTER: CodexCliAdapter = CodexCliAdapter;

/// Codex 适配器
///
/// 该适配器负责与 Codex 集成：
/// - 修改文件: `~/.codex/hooks.json`
/// - 改动内容: 在该 JSON 文件中管理 `hooks` 列表。
///   PetHover 会向其中添加自定义钩子，以便在 Codex 执行任务（如提示提交、工具调用前后等）时，
///   触发 PetHover 的事件上报逻辑。
pub(super) struct CodexCliAdapter;

const EVENTS: &[HookEvent] = &[
    HookEvent {
        cli_event: "UserPromptSubmit",
        matcher: None,
        kind: "user.prompt",
    },
    HookEvent {
        cli_event: "PreToolUse",
        matcher: Some("*"),
        kind: "tool.before",
    },
    HookEvent {
        cli_event: "PostToolUse",
        matcher: Some("*"),
        kind: "tool.after",
    },
    HookEvent {
        cli_event: "PermissionRequest",
        matcher: Some("*"),
        kind: "permission.waiting",
    },
    HookEvent {
        cli_event: "Stop",
        matcher: None,
        kind: "session.stop",
    },
];

impl CliAdapter for CodexCliAdapter {
    fn id(&self) -> &'static str {
        "codex"
    }

    fn display_name(&self) -> &'static str {
        "Codex"
    }

    fn config_path(&self, home: &Path) -> PathBuf {
        home.join(".codex").join("hooks.json")
    }

    fn is_installed(&self, config_path: &Path) -> Result<bool, AdapterError> {
        json_config_has_pethover_hook(config_path, self.id())
    }

    fn install(&self, manager: &AgentManager) -> Result<(), AdapterError> {
        install_json_hooks(
            manager,
            self.id(),
            &self.config_path(manager.home()),
            EVENTS,
            1,
        )?;
        update_codex_config_toml(manager.home(), |document| {
            set_features_hooks_true(document);
        })
    }

    fn uninstall(&self, manager: &AgentManager) -> Result<(), AdapterError> {
        remove_json_hooks(manager, self.id(), &self.config_path(manager.home()))
    }

    fn executable_names(&self) -> &'static [&'static str] {
        &["codex"]
    }
}

fn codex_config_path(home: &Path) -> PathBuf {
    home.join(".codex").join("config.toml")
}

/// Read ~/.codex/config.toml, let `update` mutate the parsed document,
/// write atomically only if the serialized output differs.
fn update_codex_config_toml<F>(home: &Path, update: F) -> Result<(), AdapterError>
where
    F: FnOnce(&mut DocumentMut),
{
    let path = codex_config_path(home);
    let content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error.into()),
    };
    let mut document = if content.is_empty() {
        DocumentMut::new()
    } else {
        content
            .parse::<DocumentMut>()
            .map_err(|_| AdapterError::InvalidJson(path.clone()))?
    };
    update(&mut document);
    let next = document.to_string();
    if next != content {
        write_atomic(&path, next.as_bytes())?;
    }
    Ok(())
}

/// Set [features].hooks = true, creating the table if needed. Also normalize
/// any legacy `codex_hooks` key by leaving it true (parity with previous impl).
fn set_features_hooks_true(document: &mut DocumentMut) {
    let features = document
        .entry("features")
        .or_insert_with(|| Item::Table(Table::new()))
        .as_table_mut()
        .expect("[features] must be a TOML table");
    features.insert("hooks", value(true));
    if features.contains_key("codex_hooks") {
        features.insert("codex_hooks", value(true));
    }
}

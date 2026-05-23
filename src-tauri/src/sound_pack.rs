use crate::pet_package::{PetAgentSounds, PetInteractionSounds, PetSounds, MAX_PET_SOUND_BYTES};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    env, fs, io,
    path::{Component, Path, PathBuf},
};

pub const SYSTEM_SOUND_PACK_PREFIX: &str = "system:";
pub const USER_SOUND_PACK_PREFIX: &str = "user:";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundPackNamespace {
    System,
    User,
}

impl SoundPackNamespace {
    pub fn prefix(self) -> &'static str {
        match self {
            SoundPackNamespace::System => SYSTEM_SOUND_PACK_PREFIX,
            SoundPackNamespace::User => USER_SOUND_PACK_PREFIX,
        }
    }
}

pub fn runtime_sound_pack_id(namespace: SoundPackNamespace, storage_id: &str) -> String {
    format!("{}{}", namespace.prefix(), storage_id)
}

pub fn system_sound_pack_id(storage_id: &str) -> String {
    runtime_sound_pack_id(SoundPackNamespace::System, storage_id)
}

pub fn user_sound_pack_id(storage_id: &str) -> String {
    runtime_sound_pack_id(SoundPackNamespace::User, storage_id)
}

pub fn parse_runtime_sound_pack_id(runtime_id: &str) -> Option<(SoundPackNamespace, &str)> {
    if let Some(raw) = runtime_id.strip_prefix(SYSTEM_SOUND_PACK_PREFIX) {
        return Some((SoundPackNamespace::System, raw));
    }
    if let Some(raw) = runtime_id.strip_prefix(USER_SOUND_PACK_PREFIX) {
        return Some((SoundPackNamespace::User, raw));
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoundPackManifest {
    pub id: String,
    pub display_name: String,
    #[serde(default)]
    pub interaction_sounds: PetInteractionSounds,
    #[serde(default)]
    pub agent_sounds: PetAgentSounds,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoundPackSummary {
    pub id: String,
    pub slug: String,
    pub display_name: String,
    pub built_in: bool,
    pub sounds: PetSounds,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoundPack {
    pub manifest: SoundPackManifest,
    pub sounds: PetSounds,
}

impl SoundPack {
    pub fn summary(self, namespace: SoundPackNamespace, storage_id: &str) -> SoundPackSummary {
        SoundPackSummary {
            id: runtime_sound_pack_id(namespace, storage_id),
            slug: storage_id.to_string(),
            display_name: self.manifest.display_name,
            built_in: matches!(namespace, SoundPackNamespace::System),
            sounds: self.sounds,
        }
    }
}

pub fn read_sound_pack(dir: &Path) -> Option<SoundPack> {
    if symlink_metadata_is_symlink(dir) {
        return None;
    }

    let storage_id = dir.file_name()?.to_str()?;
    let manifest_path = dir.join("sound.json");
    if symlink_metadata_is_symlink(&manifest_path) || !manifest_path.is_file() {
        return None;
    }

    let manifest_bytes = fs::read(manifest_path).ok()?;
    let manifest: SoundPackManifest = serde_json::from_slice(&manifest_bytes).ok()?;
    if manifest.id != storage_id {
        return None;
    }

    let raw_sounds = PetSounds {
        interaction_sounds: PetInteractionSounds {
            click: manifest.interaction_sounds.click.clone(),
            double_click: manifest.interaction_sounds.double_click.clone(),
            petted: manifest.interaction_sounds.petted.clone(),
            petted_slow: manifest.interaction_sounds.petted_slow.clone(),
            drag_land: manifest.interaction_sounds.drag_land.clone(),
        },
        agent_sounds: PetAgentSounds {
            thinking: manifest.agent_sounds.thinking.clone(),
            editing: manifest.agent_sounds.editing.clone(),
            inspecting: manifest.agent_sounds.inspecting.clone(),
            awaiting_approval: manifest.agent_sounds.awaiting_approval.clone(),
            celebrating: manifest.agent_sounds.celebrating.clone(),
            failed: manifest.agent_sounds.failed.clone(),
        },
    };
    let sounds = collect_valid_sound_pack_sounds(&raw_sounds, dir)?;

    Some(SoundPack { manifest, sounds })
}

pub fn scan_sound_packs_with_storage_ids(dir: &Path) -> io::Result<Vec<(String, SoundPack)>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut packs = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(storage_id) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if let Some(pack) = read_sound_pack(&path) {
            packs.push((storage_id.to_string(), pack));
        }
    }
    Ok(packs)
}

pub(crate) fn collect_valid_sound_pack_sounds(
    raw_sounds: &PetSounds,
    pack_dir: &Path,
) -> Option<PetSounds> {
    let sounds = PetSounds {
        interaction_sounds: PetInteractionSounds {
            click: valid_sound_pack_sound_path(
                raw_sounds.interaction_sounds.click.as_deref(),
                pack_dir,
            ),
            double_click: valid_sound_pack_sound_path(
                raw_sounds.interaction_sounds.double_click.as_deref(),
                pack_dir,
            ),
            petted: valid_sound_pack_sound_path(
                raw_sounds.interaction_sounds.petted.as_deref(),
                pack_dir,
            ),
            petted_slow: valid_sound_pack_sound_path(
                raw_sounds.interaction_sounds.petted_slow.as_deref(),
                pack_dir,
            ),
            drag_land: valid_sound_pack_sound_path(
                raw_sounds.interaction_sounds.drag_land.as_deref(),
                pack_dir,
            ),
        },
        agent_sounds: PetAgentSounds {
            thinking: valid_sound_pack_sound_path(
                raw_sounds.agent_sounds.thinking.as_deref(),
                pack_dir,
            ),
            editing: valid_sound_pack_sound_path(
                raw_sounds.agent_sounds.editing.as_deref(),
                pack_dir,
            ),
            inspecting: valid_sound_pack_sound_path(
                raw_sounds.agent_sounds.inspecting.as_deref(),
                pack_dir,
            ),
            awaiting_approval: valid_sound_pack_sound_path(
                raw_sounds.agent_sounds.awaiting_approval.as_deref(),
                pack_dir,
            ),
            celebrating: valid_sound_pack_sound_path(
                raw_sounds.agent_sounds.celebrating.as_deref(),
                pack_dir,
            ),
            failed: valid_sound_pack_sound_path(
                raw_sounds.agent_sounds.failed.as_deref(),
                pack_dir,
            ),
        },
    };

    has_any_sound(&sounds).then_some(sounds)
}

fn valid_sound_pack_sound_path(raw: Option<&str>, pack_dir: &Path) -> Option<String> {
    let raw = raw?;
    if raw.contains('\\') {
        return None;
    }

    let relative_path = Path::new(raw);
    if relative_path.is_absolute()
        || relative_path
            .extension()
            .and_then(|extension| extension.to_str())
            != Some("mp3")
        || relative_path.components().count() != 1
        || relative_path.components().any(|component| {
            matches!(
                component,
                Component::Prefix(_)
                    | Component::RootDir
                    | Component::ParentDir
                    | Component::CurDir
            )
        })
    {
        return None;
    }

    let pack_root = canonical_package_root(pack_dir)?;
    let sound_path = pack_root.join(relative_path);
    if has_symlink_component(&pack_root, relative_path) {
        return None;
    }

    let canonical_sound_path = fs::canonicalize(&sound_path).ok()?;
    if !canonical_sound_path.starts_with(&pack_root) {
        return None;
    }

    let metadata = fs::symlink_metadata(&sound_path).ok()?;
    let file_type = metadata.file_type();
    if !file_type.is_file() || file_type.is_symlink() || metadata.len() > MAX_PET_SOUND_BYTES {
        return None;
    }

    Some(canonical_sound_path.to_string_lossy().into_owned())
}

fn canonical_package_root(package_dir: &Path) -> Option<PathBuf> {
    if package_dir.is_absolute() {
        fs::canonicalize(package_dir).ok()
    } else {
        fs::canonicalize(env::current_dir().ok()?.join(package_dir)).ok()
    }
}

fn has_symlink_component(root: &Path, relative_path: &Path) -> bool {
    let mut current = root.to_path_buf();
    for component in relative_path.components() {
        let Component::Normal(part) = component else {
            return true;
        };
        current.push(part);
        let Ok(metadata) = fs::symlink_metadata(&current) else {
            return true;
        };
        if metadata.file_type().is_symlink() {
            return true;
        }
    }

    false
}

fn symlink_metadata_is_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(true)
}

fn has_any_sound(sounds: &PetSounds) -> bool {
    sounds.interaction_sounds.click.is_some()
        || sounds.interaction_sounds.double_click.is_some()
        || sounds.interaction_sounds.petted.is_some()
        || sounds.interaction_sounds.petted_slow.is_some()
        || sounds.interaction_sounds.drag_land.is_some()
        || sounds.agent_sounds.thinking.is_some()
        || sounds.agent_sounds.editing.is_some()
        || sounds.agent_sounds.inspecting.is_some()
        || sounds.agent_sounds.awaiting_approval.is_some()
        || sounds.agent_sounds.celebrating.is_some()
        || sounds.agent_sounds.failed.is_some()
}

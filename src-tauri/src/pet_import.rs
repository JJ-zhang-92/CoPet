use crate::{
    app_state::AppState,
    config_store::{
        copy_pet_package_for_import, read_pet_package_for_import, safe_pet_storage_id, ConfigStore,
        StoreError,
    },
    pet_package::{user_pet_id, PetNamespace, PetPackage, PetSummary},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs, io,
    path::{Component, Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use zip::ZipArchive;

const STALE_SESSION_AGE: Duration = Duration::from_secs(24 * 60 * 60);
const PREVIEW_METADATA_FILE: &str = ".copet-import-preview.json";
pub const ZIP_PREVIEW_MAX_ENTRIES: usize = 512;
pub const ZIP_PREVIEW_MAX_FILE_BYTES: u64 = 32 * 1024 * 1024;
pub const ZIP_PREVIEW_MAX_TOTAL_BYTES: u64 = 128 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PetImportSession {
    pub session_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PetImportPreview {
    pub preview_id: String,
    pub summary: PetSummary,
    pub source_label: String,
    pub intended_pet_id: String,
    pub selected_by_default: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PetImportPreviewBatch {
    pub previews: Vec<PetImportPreview>,
    pub skipped: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PetImportFailure {
    pub preview_id: String,
    pub error_message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PetImportCommitResult {
    pub imported: Vec<PetSummary>,
    pub failed: Vec<PetImportFailure>,
    pub state: AppState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PetImportPreviewMetadata {
    preview_id: String,
    intended_storage_id: String,
    intended_pet_id: String,
    source_label: String,
}

pub fn create_import_session(store: &ConfigStore) -> Result<PetImportSession, StoreError> {
    store.ensure_ready()?;
    let previews_dir = store.import_previews_dir();
    fs::create_dir_all(&previews_dir)?;
    cleanup_stale_sessions(&previews_dir)?;

    for _ in 0..100 {
        let session_id = new_session_id();
        let dir = session_dir(store, &session_id);
        if !dir.exists() {
            fs::create_dir_all(dir)?;
            return Ok(PetImportSession { session_id });
        }
    }

    Err(StoreError::InvalidPetPackage(
        "could not create a unique import preview session".to_string(),
    ))
}

pub fn preview_codex_imports(
    store: &ConfigStore,
    session_id: &str,
    codex_pets_dir: &Path,
) -> Result<PetImportPreviewBatch, StoreError> {
    preview_folder_imports(store, session_id, &[codex_pets_dir.to_path_buf()])
}

pub fn preview_folder_imports(
    store: &ConfigStore,
    session_id: &str,
    folders: &[PathBuf],
) -> Result<PetImportPreviewBatch, StoreError> {
    store.ensure_ready()?;
    let target_session_dir = existing_session_dir(store, session_id)?;

    let mut previews = Vec::new();
    let mut skipped = 0;
    let mut errors = Vec::new();
    let mut used_preview_ids = existing_preview_ids(&target_session_dir)?;

    for folder in folders {
        match folder_candidates(folder) {
            Ok(candidates) => {
                if candidates.is_empty() {
                    errors.push(format!("no pet packages found in {}", folder.display()));
                }

                for source_dir in candidates {
                    let Some(package) = read_pet_package_for_import(&source_dir) else {
                        skipped += 1;
                        continue;
                    };
                    if !safe_pet_storage_id(&package.manifest.id) {
                        skipped += 1;
                        continue;
                    }

                    let Some(storage_id) =
                        source_dir_label(&source_dir).filter(|label| safe_pet_storage_id(label))
                    else {
                        skipped += 1;
                        continue;
                    };
                    let preview_id = preview_id_for(storage_id, &mut used_preview_ids);
                    let target_dir = target_session_dir.join(&preview_id);
                    if let Err(error) =
                        copy_pet_package_for_import(&source_dir, &target_dir, &package)
                    {
                        errors.push(format!("could not stage {}: {error}", source_dir.display()));
                        continue;
                    }

                    let preview =
                        build_preview(&preview_id, storage_id, &source_dir, &target_dir, package);
                    if let Err(error) = write_preview_metadata(&target_dir, &preview, storage_id) {
                        let _ = fs::remove_dir_all(&target_dir);
                        errors.push(format!("could not stage {}: {error}", source_dir.display()));
                        continue;
                    }

                    previews.push(preview);
                }
            }
            Err(message) => errors.push(message),
        }
    }

    previews.sort_by(|left, right| {
        left.summary
            .display_name
            .cmp(&right.summary.display_name)
            .then_with(|| left.summary.id.cmp(&right.summary.id))
    });

    Ok(PetImportPreviewBatch {
        previews,
        skipped,
        errors,
    })
}

pub fn preview_zip_imports(
    store: &ConfigStore,
    session_id: &str,
    zip_paths: &[PathBuf],
) -> Result<PetImportPreviewBatch, StoreError> {
    store.ensure_ready()?;
    let target_session_dir = existing_session_dir(store, session_id)?;

    let mut previews = Vec::new();
    let mut skipped = 0;
    let mut errors = Vec::new();

    for zip_path in zip_paths {
        let scratch_dir = match unique_scratch_dir(&target_session_dir, zip_path) {
            Ok(dir) => dir,
            Err(error) => {
                errors.push(format!("could not stage {}: {error}", zip_path.display()));
                continue;
            }
        };
        match extract_zip_for_preview(zip_path, &scratch_dir) {
            Ok(preview_root) => {
                match preview_folder_imports(store, session_id, &[preview_root]) {
                    Ok(mut batch) => {
                        previews.append(&mut batch.previews);
                        skipped += batch.skipped;
                        errors.append(&mut batch.errors);
                    }
                    Err(error) => {
                        errors.push(format!("could not preview {}: {error}", zip_path.display()))
                    }
                }
                cleanup_scratch_dir(zip_path, &scratch_dir, &mut errors);
            }
            Err(message) => {
                errors.push(message);
                cleanup_scratch_dir(zip_path, &scratch_dir, &mut errors);
            }
        }
    }

    previews.sort_by(|left, right| {
        left.summary
            .display_name
            .cmp(&right.summary.display_name)
            .then_with(|| left.summary.id.cmp(&right.summary.id))
    });

    Ok(PetImportPreviewBatch {
        previews,
        skipped,
        errors,
    })
}

pub fn discard_import_session(store: &ConfigStore, session_id: &str) -> Result<(), StoreError> {
    let dir = existing_session_dir(store, session_id)?;
    fs::remove_dir_all(dir)?;
    Ok(())
}

pub fn commit_import_previews(
    store: &ConfigStore,
    session_id: &str,
    preview_ids: &[String],
) -> Result<PetImportCommitResult, StoreError> {
    store.ensure_ready()?;
    let target_session_dir = existing_session_dir(store, session_id)?;

    let mut imported = Vec::new();
    let mut failed = Vec::new();

    for preview_id in preview_ids {
        if !safe_pet_storage_id(preview_id) {
            failed.push(PetImportFailure {
                preview_id: preview_id.clone(),
                error_message: "preview id is invalid".to_string(),
            });
            continue;
        }

        let preview_dir = target_session_dir.join(preview_id);
        let Some(package) = read_pet_package_for_import(&preview_dir) else {
            failed.push(PetImportFailure {
                preview_id: preview_id.clone(),
                error_message: "preview package is no longer available".to_string(),
            });
            continue;
        };
        if !safe_pet_storage_id(&package.manifest.id) {
            failed.push(PetImportFailure {
                preview_id: preview_id.clone(),
                error_message: "preview package has an invalid pet id".to_string(),
            });
            continue;
        }

        let base_storage_id = match preview_intended_storage_id(preview_id, &preview_dir, &package)
        {
            Ok(storage_id) => storage_id,
            Err(error) => {
                failed.push(PetImportFailure {
                    preview_id: preview_id.clone(),
                    error_message: error.to_string(),
                });
                continue;
            }
        };

        let (storage_id, target_dir) = match reserve_user_pet_target(store, &base_storage_id) {
            Ok(reserved) => reserved,
            Err(error) => {
                failed.push(PetImportFailure {
                    preview_id: preview_id.clone(),
                    error_message: error.to_string(),
                });
                continue;
            }
        };
        if let Err(error) =
            copy_reserved_pet_package_for_import(&preview_dir, &target_dir, &package)
        {
            let _ = fs::remove_dir_all(&target_dir);
            failed.push(PetImportFailure {
                preview_id: preview_id.clone(),
                error_message: error.to_string(),
            });
            continue;
        }

        if let Err(error) = fs::remove_dir_all(&preview_dir) {
            failed.push(PetImportFailure {
                preview_id: preview_id.clone(),
                error_message: format!("imported pet but could not remove preview: {error}"),
            });
        }
        let sprite_path = package
            .sprite_path
            .file_name()
            .map(|name| target_dir.join(name))
            .unwrap_or_else(|| package.sprite_path.clone());
        imported.push(
            PetPackage {
                sprite_path,
                ..package
            }
            .summary(PetNamespace::User, &storage_id),
        );
    }

    Ok(PetImportCommitResult {
        imported,
        failed,
        state: store.app_state()?,
    })
}

fn build_preview(
    preview_id: &str,
    storage_id: &str,
    source_dir: &Path,
    target_dir: &Path,
    package: PetPackage,
) -> PetImportPreview {
    let staged_sprite_path = package
        .sprite_path
        .file_name()
        .map(|name| target_dir.join(name))
        .unwrap_or_else(|| package.sprite_path.clone());
    let summary = PetPackage {
        sprite_path: staged_sprite_path,
        ..package
    }
    .summary(PetNamespace::User, storage_id);

    PetImportPreview {
        preview_id: preview_id.to_string(),
        summary,
        source_label: source_dir
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| source_dir.to_string_lossy().into_owned()),
        intended_pet_id: user_pet_id(storage_id),
        selected_by_default: true,
        warning: None,
    }
}

fn write_preview_metadata(
    preview_dir: &Path,
    preview: &PetImportPreview,
    intended_storage_id: &str,
) -> Result<(), StoreError> {
    let metadata = PetImportPreviewMetadata {
        preview_id: preview.preview_id.clone(),
        intended_storage_id: intended_storage_id.to_string(),
        intended_pet_id: preview.intended_pet_id.clone(),
        source_label: preview.source_label.clone(),
    };
    fs::write(
        preview_dir.join(PREVIEW_METADATA_FILE),
        serde_json::to_vec_pretty(&metadata)?,
    )?;
    Ok(())
}

fn reserve_user_pet_target(
    store: &ConfigStore,
    base_id: &str,
) -> Result<(String, PathBuf), StoreError> {
    if !safe_pet_storage_id(base_id) {
        return Err(StoreError::InvalidPetPackage(
            "pet id must be a safe storage id".to_string(),
        ));
    }

    let pets_dir = store.pets_dir();
    for suffix in 1.. {
        let storage_id = if suffix == 1 {
            base_id.to_string()
        } else {
            format!("{base_id}-{suffix}")
        };
        let target_dir = pets_dir.join(&storage_id);
        match fs::create_dir(&target_dir) {
            Ok(()) => return Ok((storage_id, target_dir)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.into()),
        }
    }

    unreachable!("exhausted numeric pet storage id suffixes")
}

fn copy_reserved_pet_package_for_import(
    source_dir: &Path,
    target_dir: &Path,
    package: &PetPackage,
) -> Result<(), StoreError> {
    let source_root = fs::canonicalize(source_dir)?;
    fs::copy(source_root.join("pet.json"), target_dir.join("pet.json"))?;
    if let Some(sprite_name) = package.sprite_path.file_name() {
        fs::copy(&package.sprite_path, target_dir.join(sprite_name))?;
    }
    for sound_path in package.sound_file_paths() {
        let canonical_sound_path = fs::canonicalize(&sound_path)?;
        let relative_path = canonical_sound_path
            .strip_prefix(&source_root)
            .map_err(|_| {
                StoreError::InvalidPetPackage(format!(
                    "sound file must be inside package: {}",
                    sound_path.display()
                ))
            })?;
        let target_path = target_dir.join(relative_path);
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(canonical_sound_path, target_path)?;
    }
    Ok(())
}

fn preview_intended_storage_id(
    requested_preview_id: &str,
    preview_dir: &Path,
    package: &PetPackage,
) -> Result<String, StoreError> {
    let metadata_path = preview_dir.join(PREVIEW_METADATA_FILE);
    if !metadata_path.exists() {
        return Ok(package.manifest.id.clone());
    }

    let metadata: PetImportPreviewMetadata = serde_json::from_slice(&fs::read(metadata_path)?)?;
    if metadata.preview_id != requested_preview_id {
        return Err(StoreError::InvalidPetPackage(
            "preview metadata does not match selected preview".to_string(),
        ));
    }
    if !safe_pet_storage_id(&metadata.intended_storage_id) {
        return Err(StoreError::InvalidPetPackage(
            "preview metadata has an invalid pet id".to_string(),
        ));
    }
    if metadata.intended_pet_id != user_pet_id(&metadata.intended_storage_id) {
        return Err(StoreError::InvalidPetPackage(
            "preview metadata does not match intended pet id".to_string(),
        ));
    }
    Ok(metadata.intended_storage_id)
}

fn session_dir(store: &ConfigStore, session_id: &str) -> PathBuf {
    store.import_previews_dir().join(session_id)
}

fn existing_session_dir(store: &ConfigStore, session_id: &str) -> Result<PathBuf, StoreError> {
    if !safe_session_id(session_id) {
        return Err(StoreError::InvalidPetPackage(
            "import preview session id is invalid".to_string(),
        ));
    }

    let dir = session_dir(store, session_id);
    if !dir.is_dir() {
        return Err(StoreError::InvalidPetPackage(
            "import preview session was not found".to_string(),
        ));
    }
    Ok(dir)
}

fn existing_preview_ids(session_dir: &Path) -> Result<BTreeSet<String>, StoreError> {
    let mut ids = BTreeSet::new();
    for entry in fs::read_dir(session_dir)? {
        let path = entry?.path();
        if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
            ids.insert(name.to_string());
        }
    }
    Ok(ids)
}

fn folder_candidates(folder: &Path) -> Result<Vec<PathBuf>, String> {
    if !folder.is_dir() {
        return Err(format!(
            "selected path is not a folder: {}",
            folder.display()
        ));
    }

    if is_pet_package_dir(folder) {
        return Ok(vec![folder.to_path_buf()]);
    }
    if is_pet_package_candidate_dir(folder) {
        return Ok(vec![folder.to_path_buf()]);
    }

    let mut candidates = Vec::new();
    let entries = fs::read_dir(folder)
        .map_err(|error| format!("could not read {}: {error}", folder.display()))?;
    for entry in entries {
        let path = entry
            .map_err(|error| format!("could not read {}: {error}", folder.display()))?
            .path();
        if path.is_dir() && is_pet_package_candidate_dir(&path) {
            candidates.push(path);
        }
    }
    candidates.sort();
    Ok(candidates)
}

fn unique_scratch_dir(session_dir: &Path, zip_path: &Path) -> Result<PathBuf, StoreError> {
    let stem = zip_path
        .file_stem()
        .and_then(|name| name.to_str())
        .map(sanitize_preview_segment)
        .unwrap_or_else(|| "archive".to_string());
    let base = format!(".zip-{stem}");

    for suffix in 1.. {
        let candidate = if suffix == 1 {
            session_dir.join(&base)
        } else {
            session_dir.join(format!("{base}-{suffix}"))
        };
        match fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.into()),
        }
    }
    unreachable!()
}

fn extract_zip_for_preview(zip_path: &Path, scratch_dir: &Path) -> Result<PathBuf, String> {
    let file = fs::File::open(zip_path)
        .map_err(|error| format!("could not open {}: {error}", zip_path.display()))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|error| format!("could not read {}: {error}", zip_path.display()))?;
    if archive.len() > ZIP_PREVIEW_MAX_ENTRIES {
        return Err(format!(
            "{} has too many zip entries: {} > {}",
            zip_path.display(),
            archive.len(),
            ZIP_PREVIEW_MAX_ENTRIES
        ));
    }

    let raw_dir = scratch_dir.join("contents");
    fs::create_dir(&raw_dir)
        .map_err(|error| format!("could not stage {}: {error}", zip_path.display()))?;

    let mut output_paths = BTreeSet::new();
    let mut total_uncompressed_bytes = 0_u64;
    for index in 0..archive.len() {
        let mut file = archive
            .by_index(index)
            .map_err(|error| format!("could not read {}: {error}", zip_path.display()))?;
        let Some(enclosed_name) = file.enclosed_name() else {
            return Err(format!(
                "unsafe zip path in {}: {}",
                zip_path.display(),
                file.name()
            ));
        };
        let output_path = normalized_zip_output_path(&enclosed_name)
            .ok_or_else(|| format!("unsafe zip path in {}: {}", zip_path.display(), file.name()))?;
        if !output_paths.insert(output_path.clone()) {
            return Err(format!(
                "duplicate zip path in {}: {}",
                zip_path.display(),
                output_path.display()
            ));
        }
        let target_path = raw_dir.join(output_path);

        if file.is_dir() {
            fs::create_dir_all(&target_path)
                .map_err(|error| format!("could not stage {}: {error}", zip_path.display()))?;
            continue;
        }

        let entry_size = file.size();
        if entry_size > ZIP_PREVIEW_MAX_FILE_BYTES {
            return Err(format!(
                "{} exceeds zip entry size limit: {} > {} bytes",
                file.name(),
                entry_size,
                ZIP_PREVIEW_MAX_FILE_BYTES
            ));
        }
        total_uncompressed_bytes = total_uncompressed_bytes
            .checked_add(entry_size)
            .ok_or_else(|| {
                format!(
                    "{} exceeds zip total size limit: overflow",
                    zip_path.display()
                )
            })?;
        if total_uncompressed_bytes > ZIP_PREVIEW_MAX_TOTAL_BYTES {
            return Err(format!(
                "{} exceeds zip total size limit: {} > {} bytes",
                zip_path.display(),
                total_uncompressed_bytes,
                ZIP_PREVIEW_MAX_TOTAL_BYTES
            ));
        }

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("could not stage {}: {error}", zip_path.display()))?;
        }
        let mut target_file = fs::File::create(&target_path)
            .map_err(|error| format!("could not stage {}: {error}", zip_path.display()))?;
        let entry_name = file.name().to_string();
        copy_zip_file_bounded(
            zip_path,
            &entry_name,
            &mut file,
            &mut target_file,
            entry_size,
        )?;
    }

    Ok(normalize_zip_preview_root(&raw_dir))
}

fn normalized_zip_output_path(path: &Path) -> Option<PathBuf> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(segment) => normalized.push(segment),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    if normalized.as_os_str().is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn copy_zip_file_bounded(
    zip_path: &Path,
    entry_name: &str,
    source: &mut impl io::Read,
    target: &mut impl io::Write,
    expected_size: u64,
) -> Result<(), String> {
    let mut copied = 0_u64;
    let mut buffer = [0_u8; 8192];
    loop {
        let read = source
            .read(&mut buffer)
            .map_err(|error| format!("could not stage {}: {error}", zip_path.display()))?;
        if read == 0 {
            return Ok(());
        }
        copied = copied
            .checked_add(read as u64)
            .ok_or_else(|| format!("{} exceeds zip entry size limit: overflow", entry_name))?;
        if copied > ZIP_PREVIEW_MAX_FILE_BYTES || copied > expected_size {
            return Err(format!(
                "{} exceeds zip entry size limit: {} > {} bytes",
                entry_name, copied, ZIP_PREVIEW_MAX_FILE_BYTES
            ));
        }
        target
            .write_all(&buffer[..read])
            .map_err(|error| format!("could not stage {}: {error}", zip_path.display()))?;
    }
}

fn cleanup_scratch_dir(zip_path: &Path, scratch_dir: &Path, errors: &mut Vec<String>) {
    if let Err(error) = fs::remove_dir_all(scratch_dir) {
        errors.push(format!(
            "could not clean up preview scratch for {}: {error}",
            zip_path.display()
        ));
    }
}

fn normalize_zip_preview_root(raw_dir: &Path) -> PathBuf {
    let Some(package) = read_pet_package_for_import(raw_dir) else {
        return raw_dir.to_path_buf();
    };
    if !safe_pet_storage_id(&package.manifest.id) {
        return raw_dir.to_path_buf();
    }

    let normalized_dir = raw_dir
        .parent()
        .map(|parent| parent.join(&package.manifest.id))
        .unwrap_or_else(|| raw_dir.to_path_buf());
    if fs::rename(raw_dir, &normalized_dir).is_ok() {
        normalized_dir
    } else {
        raw_dir.to_path_buf()
    }
}

fn is_pet_package_dir(dir: &Path) -> bool {
    read_pet_package_for_import(dir).is_some()
}

fn is_pet_package_candidate_dir(dir: &Path) -> bool {
    dir.join("pet.json").is_file()
        || dir.join("spritesheet.webp").is_file()
        || dir.join("spritesheet.png").is_file()
}

fn source_dir_label(dir: &Path) -> Option<&str> {
    dir.file_name().and_then(|name| name.to_str())
}

fn safe_session_id(value: &str) -> bool {
    value.starts_with("session-")
        && !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
}

fn preview_id_for(storage_id: &str, used_preview_ids: &mut BTreeSet<String>) -> String {
    let base = sanitize_preview_segment(storage_id);
    if used_preview_ids.insert(base.clone()) {
        return base;
    }

    for suffix in 2.. {
        let candidate = format!("{base}-{suffix}");
        if used_preview_ids.insert(candidate.clone()) {
            return candidate;
        }
    }
    unreachable!()
}

fn sanitize_preview_segment(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>();
    if sanitized.is_empty() || sanitized == "." || sanitized == ".." {
        "session".to_string()
    } else {
        sanitized
    }
}

fn new_session_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("session-{nanos}-{}", std::process::id())
}

fn cleanup_stale_sessions(previews_dir: &Path) -> Result<(), StoreError> {
    let now = SystemTime::now();
    for entry in fs::read_dir(previews_dir)? {
        let path = entry?.path();
        if !path.is_dir() {
            continue;
        }

        let Ok(modified) = fs::metadata(&path).and_then(|metadata| metadata.modified()) else {
            continue;
        };
        if now
            .duration_since(modified)
            .is_ok_and(|age| age > STALE_SESSION_AGE)
        {
            let _ = fs::remove_dir_all(path);
        }
    }
    Ok(())
}

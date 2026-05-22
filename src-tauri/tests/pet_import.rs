use copet_lib::{
    config_store::ConfigStore,
    pet_import::{create_import_session, preview_codex_imports, preview_folder_imports},
};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn builtin_pets_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/pets")
}

fn make_store(temp: &tempfile::TempDir) -> ConfigStore {
    ConfigStore::with_builtin_dir(temp.path().join(".copet"), builtin_pets_dir())
}

fn create_pet_package(root: &Path, storage_id: &str, manifest_id: &str, display_name: &str) {
    let package_dir = root.join(storage_id);
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(
        package_dir.join("pet.json"),
        format!(
            r#"{{
  "id": "{manifest_id}",
  "slug": "{manifest_id}",
  "displayName": "{display_name}",
  "description": "A test pet.",
  "frameWidth": 160,
  "frameHeight": 64,
  "gridColumns": 8,
  "gridRows": 9
}}"#
        ),
    )
    .unwrap();
    fs::write(package_dir.join("spritesheet.png"), b"sprite").unwrap();
}

#[test]
fn preview_codex_imports_stages_valid_packages_without_installing() {
    let temp = tempfile::tempdir().unwrap();
    let store = make_store(&temp);
    let codex_pets = temp.path().join(".codex/pets");
    create_pet_package(&codex_pets, "space-cat", "space-cat", "Space Cat");
    fs::create_dir_all(codex_pets.join("broken")).unwrap();
    fs::write(codex_pets.join("broken/pet.json"), "{not valid json").unwrap();

    let session = create_import_session(&store).unwrap();
    let batch = preview_codex_imports(&store, &session.session_id, &codex_pets).unwrap();

    assert_eq!(batch.previews.len(), 1);
    assert_eq!(batch.skipped, 1);
    assert!(batch.errors.is_empty());
    let preview = &batch.previews[0];
    assert_eq!(preview.summary.id, "user:space-cat");
    assert_eq!(preview.intended_pet_id, "user:space-cat");
    assert!(preview.summary.sprite_path.contains("import-previews"));
    assert!(preview.selected_by_default);
    assert!(!store.root().join("pets/space-cat").exists());
}

#[test]
fn preview_folder_imports_accepts_package_folder_and_child_packages() {
    let temp = tempfile::tempdir().unwrap();
    let store = make_store(&temp);
    let selected_single = temp.path().join("single-pet");
    create_pet_package(temp.path(), "single-pet", "single-pet", "Single Pet");
    let selected_parent = temp.path().join("pet-packages");
    create_pet_package(&selected_parent, "beta", "beta", "Beta");
    create_pet_package(&selected_parent, "alpha", "alpha", "Alpha");

    let session = create_import_session(&store).unwrap();
    let batch = preview_folder_imports(
        &store,
        &session.session_id,
        &[selected_single, selected_parent],
    )
    .unwrap();

    assert_eq!(batch.skipped, 0);
    assert!(batch.errors.is_empty());
    assert_eq!(
        batch
            .previews
            .iter()
            .map(|preview| preview.summary.id.as_str())
            .collect::<Vec<_>>(),
        vec!["user:alpha", "user:beta", "user:single-pet"]
    );
    assert!(!store.root().join("pets/alpha").exists());
    assert!(!store.root().join("pets/beta").exists());
    assert!(!store.root().join("pets/single-pet").exists());
}

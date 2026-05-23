use copet_lib::pet_context_menu::{
    action_for_menu_id, PetContextMenuAction, PET_CONTEXT_MENU_HIDE_ID,
    PET_CONTEXT_MENU_MESSAGES_ID, PET_CONTEXT_MENU_SETTINGS_ID,
};

#[test]
fn maps_native_pet_context_menu_ids_to_actions() {
    assert_eq!(
        action_for_menu_id(PET_CONTEXT_MENU_MESSAGES_ID),
        Some(PetContextMenuAction::ToggleMessages)
    );
    assert_eq!(
        action_for_menu_id(PET_CONTEXT_MENU_SETTINGS_ID),
        Some(PetContextMenuAction::OpenSettings)
    );
    assert_eq!(
        action_for_menu_id(PET_CONTEXT_MENU_HIDE_ID),
        Some(PetContextMenuAction::HidePet)
    );
    assert_eq!(action_for_menu_id("unrelated"), None);
}

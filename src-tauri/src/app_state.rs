use serde::{Deserialize, Serialize};

use crate::i18n::{Locale, LocalePreference};
use crate::pet_package::PetSummary;
use crate::sound_pack::SoundPackSummary;

pub type PetWindowSize = u8;

pub const MIN_PET_WINDOW_SIZE: PetWindowSize = 1;
pub const MAX_PET_WINDOW_SIZE: PetWindowSize = 100;
pub const DEFAULT_PET_WINDOW_SIZE: PetWindowSize = 40;

pub fn default_pet_window_size() -> PetWindowSize {
    DEFAULT_PET_WINDOW_SIZE
}

pub fn normalize_pet_window_size(size: PetWindowSize) -> PetWindowSize {
    size.clamp(MIN_PET_WINDOW_SIZE, MAX_PET_WINDOW_SIZE)
}

pub type MessageFontSize = u8;

pub const MIN_MESSAGE_FONT_SIZE: MessageFontSize = 8;
pub const MAX_MESSAGE_FONT_SIZE: MessageFontSize = 32;
pub const DEFAULT_MESSAGE_FONT_SIZE: MessageFontSize = 11;

pub fn default_message_font_size() -> MessageFontSize {
    DEFAULT_MESSAGE_FONT_SIZE
}

pub fn normalize_message_font_size(size: MessageFontSize) -> MessageFontSize {
    size.clamp(MIN_MESSAGE_FONT_SIZE, MAX_MESSAGE_FONT_SIZE)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AgentMessageDisplay {
    #[default]
    All,
    Latest,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CooldownStyle {
    Short,
    #[default]
    Normal,
    Lazy,
}

fn default_enable_click_sounds() -> bool {
    true
}

fn default_enable_startup_animation() -> bool {
    true
}

fn default_agent_message_visible() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PetInteractionPrefs {
    // Per-field defaults so this struct survives being flattened into a
    // parent config when individual keys are missing from disk.
    #[serde(default = "default_enable_click_sounds")]
    pub enable_click_sounds: bool,
    #[serde(default)]
    pub cooldown_style: CooldownStyle,
    #[serde(default = "default_enable_startup_animation")]
    pub enable_startup_animation: bool,
}

impl Default for PetInteractionPrefs {
    fn default() -> Self {
        Self {
            enable_click_sounds: true,
            cooldown_style: CooldownStyle::Normal,
            enable_startup_animation: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub current_pet_id: String,
    pub current_sound_pack_id: String,
    pub locale: Locale,
    pub locale_preference: LocalePreference,
    pub pets: Vec<PetSummary>,
    pub sound_packs: Vec<SoundPackSummary>,
    pub onboarding_complete: bool,
    pub pet_window_size: PetWindowSize,
    pub message_font_size: MessageFontSize,
    pub agent_message_display: AgentMessageDisplay,
    #[serde(default = "default_agent_message_visible")]
    pub agent_message_visible: bool,
    #[serde(default)]
    pub pet_interactions: PetInteractionPrefs,
}

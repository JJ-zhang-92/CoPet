export type PetStateId =
  | "idle"
  | "running-right"
  | "running-left"
  | "waving"
  | "jumping"
  | "failed"
  | "waiting"
  | "running"
  | "review"
  | "thinking";

export type PetState = {
  id: PetStateId;
  row: number;
  frames: number;
  durationMs: number;
};

export type PetInteractionSounds = {
  click?: string;
  doubleClick?: string;
  petted?: string;
  pettedSlow?: string;
  dragLand?: string;
};

export type PetAgentSounds = {
  thinking?: string;
  editing?: string;
  inspecting?: string;
  awaitingApproval?: string;
  celebrating?: string;
  failed?: string;
};

export type PetSounds = {
  interactionSounds?: PetInteractionSounds;
  agentSounds?: PetAgentSounds;
};

export type PetSummary = {
  id: string;
  slug: string;
  displayName: string;
  description: string;
  frameWidth: number;
  frameHeight: number;
  gridColumns: number;
  gridRows: number;
  builtIn: boolean;
  spritePath: string;
  sounds?: PetSounds;
};

export type SoundPackSummary = {
  id: string;
  slug: string;
  displayName: string;
  builtIn: boolean;
  sounds: PetSounds;
};

export type Locale = "en-US" | "zh-CN";
export type LocalePreference = Locale;

export type AgentMessageDisplay = "all" | "latest";

export type CooldownStyle = "short" | "normal" | "lazy";

export type PetInteractionPrefs = {
  enableClickSounds: boolean;
  cooldownStyle: CooldownStyle;
  enableStartupAnimation: boolean;
};

export const defaultPetInteractionPrefs: PetInteractionPrefs = {
  enableClickSounds: true,
  cooldownStyle: "normal",
  enableStartupAnimation: true,
};

export type AppState = {
  currentPetId: string;
  currentSoundPackId: string;
  locale: Locale;
  localePreference: LocalePreference;
  pets: PetSummary[];
  soundPacks: SoundPackSummary[];
  onboardingComplete: boolean;
  petWindowSize: PetWindowSize;
  messageFontSize: MessageFontSize;
  agentMessageDisplay: AgentMessageDisplay;
  agentMessageVisible: boolean;
  petInteractions: PetInteractionPrefs;
};

export type PetWindowSize = number;

export type MessageFontSize = number;

export const defaultMessageFontSize = 11;
export const minMessageFontSize = 8;
export const maxMessageFontSize = 32;

export type PetImportResult = {
  imported: number;
  skipped: number;
  pets: PetSummary[];
};

export type PetImportSession = { sessionId: string };

export type PetImportPreview = {
  previewId: string;
  summary: PetSummary;
  sourceLabel: string;
  intendedPetId: string;
  selectedByDefault: boolean;
  warning?: string;
};

export type PetImportPreviewBatch = {
  previews: PetImportPreview[];
  skipped: number;
  errors: string[];
};

export type PetImportFailure = {
  previewId: string;
  errorMessage: string;
};

export type PetImportCommitResult = {
  imported: PetSummary[];
  failed: PetImportFailure[];
  state: AppState;
};

export type DerivedPetState = {
  state: PetStateId;
  sinceMs: number;
  idleAfterMs: number | null;
};

export type AgentMessageKind = "waiting" | "error" | "running" | "thinking" | "done";

export const AGENT_MESSAGE_FONT_SCALE: Record<AgentMessageKind, number> = {
  waiting: 1.30,
  error:   1.15,
  running: 1.00,
  thinking: 0.85,
  done:    0.75,
};

export type AgentMessage = {
  agent: string;
  displayName: string;
  text: string;
  updatedAtMs: number;
  kind: AgentMessageKind;
};

export type RuntimeUpdate = {
  currentState: DerivedPetState;
  messages: AgentMessage[];
};

export type RuntimeStatus = {
  port: number;
  endpoint: string;
  currentState: DerivedPetState;
  messages: AgentMessage[];
  acceptedEvents: number;
  rejectedEvents: number;
};

export type AdapterSummary = {
  id: string;
  displayName: string;
  configPath: string;
  installed: boolean;
  healthy: boolean;
  message: string;
};

export type AdapterOperationResult = {
  adapter: AdapterSummary;
};

import { ChevronDown } from "lucide-react";
import { useEffect, useId, useRef, useState } from "react";
import type { KeyboardEvent } from "react";

import type { SoundPackSummary } from "../lib/appTypes";
import type { Translator } from "../lib/settingsTypes";

interface SettingsSoundPackSelectProps {
  soundPacks: SoundPackSummary[];
  currentSoundPackId: string;
  selectSoundPack: (soundPackId: string) => Promise<void>;
  t: Translator;
}

export function SettingsSoundPackSelect({
  soundPacks,
  currentSoundPackId,
  selectSoundPack,
  t,
}: SettingsSoundPackSelectProps) {
  const selectId = useId();
  const listboxId = `${selectId}-listbox`;
  const rootRef = useRef<HTMLDivElement | null>(null);
  const [open, setOpen] = useState(false);
  const [pending, setPending] = useState(false);
  const builtInPacks = soundPacks.filter((pack) => pack.builtIn);
  const customPacks = soundPacks.filter((pack) => !pack.builtIn);
  const selectedPack = soundPacks.find((pack) => pack.id === currentSoundPackId);
  const disabled = soundPacks.length === 0 || pending;

  useEffect(() => {
    if (!open) {
      return;
    }

    const handlePointerDown = (event: PointerEvent) => {
      if (rootRef.current?.contains(event.target as Node)) {
        return;
      }
      setOpen(false);
    };

    document.addEventListener("pointerdown", handlePointerDown);
    return () => document.removeEventListener("pointerdown", handlePointerDown);
  }, [open]);

  const label = selectedPack?.displayName ?? soundPacks[0]?.displayName ?? t("noSoundPacks");

  const handleTriggerKeyDown = (event: KeyboardEvent<HTMLButtonElement>) => {
    if (event.key === "Escape") {
      setOpen(false);
      return;
    }

    if (
      !disabled &&
      (event.key === "ArrowDown" || event.key === "Enter" || event.key === " ")
    ) {
      event.preventDefault();
      setOpen(true);
    }
  };

  const handleSelect = async (soundPackId: string) => {
    if (pending) {
      return;
    }

    setPending(true);
    setOpen(false);
    try {
      await selectSoundPack(soundPackId);
    } finally {
      setPending(false);
    }
  };

  const renderGroup = (heading: string, packs: SoundPackSummary[]) => {
    if (packs.length === 0) {
      return null;
    }

    return (
      <div className="ui-select-group" role="group" aria-label={heading}>
        <div className="ui-select-group-label">{heading}</div>
        {packs.map((pack) => (
          <button
            aria-selected={pack.id === currentSoundPackId}
            className="ui-select-option"
            data-selected={pack.id === currentSoundPackId}
            disabled={pending}
            key={pack.id}
            onClick={() => {
              void handleSelect(pack.id);
            }}
            role="option"
            type="button"
          >
            {pack.displayName}
          </button>
        ))}
      </div>
    );
  };

  return (
    <div className="ui-select sound-pack-select" ref={rootRef}>
      <button
        aria-controls={listboxId}
        aria-expanded={open}
        aria-haspopup="listbox"
        aria-label={t("soundPack")}
        className="ui-select-trigger"
        disabled={disabled}
        id={selectId}
        onClick={() => {
          if (!disabled) {
            setOpen((visible) => !visible);
          }
        }}
        onKeyDown={handleTriggerKeyDown}
        role="combobox"
        type="button"
      >
        <span>{label}</span>
        <ChevronDown aria-hidden="true" />
      </button>
      {open ? (
        <div className="ui-select-listbox" id={listboxId} role="listbox">
          {renderGroup(t("builtInSounds"), builtInPacks)}
          {renderGroup(t("customSounds"), customPacks)}
        </div>
      ) : null}
    </div>
  );
}

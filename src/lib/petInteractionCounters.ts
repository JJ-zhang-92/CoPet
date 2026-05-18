const STORAGE_KEY = "petInteractionCounters";

export type PetInteractionCounters = {
  click: number;
  doubleClick: number;
  petted: number;
  pettedSlow: number;
};

const ZERO: PetInteractionCounters = {
  click: 0,
  doubleClick: 0,
  petted: 0,
  pettedSlow: 0,
};

function read(): PetInteractionCounters {
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...ZERO };
    const parsed = JSON.parse(raw) as Partial<PetInteractionCounters>;
    return { ...ZERO, ...parsed };
  } catch {
    return { ...ZERO };
  }
}

function write(value: PetInteractionCounters): void {
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(value));
  } catch {
    // localStorage may be unavailable in some Tauri contexts; counters are best-effort.
  }
}

export function bumpCounter(key: keyof PetInteractionCounters): void {
  const current = read();
  current[key] += 1;
  write(current);
}

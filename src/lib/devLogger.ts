const hoverpetDevLogEnabled = import.meta.env.DEV;

export function hoverpetDevLog(stage: string, payload: unknown) {
  if (!hoverpetDevLogEnabled) {
    return;
  }

  console.debug(`[hoverpet:${stage}]`, payload);
}

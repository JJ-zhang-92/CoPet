const copetDevLogEnabled = import.meta.env.DEV;

export function copetDevLog(stage: string, payload: unknown) {
  if (!copetDevLogEnabled) {
    return;
  }

  console.debug(`[copet:${stage}]`, payload);
}

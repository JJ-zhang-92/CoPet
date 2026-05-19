import { useCallback } from "react";
import { open as openExternal } from "@tauri-apps/plugin-shell";

export function useOpenExternalUrl() {
  return useCallback(async (url: string) => {
    await openExternal(url);
  }, []);
}

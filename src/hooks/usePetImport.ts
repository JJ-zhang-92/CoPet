import { open } from "@tauri-apps/plugin-dialog";
import { useCallback, useMemo, useState } from "react";

import {
  commitPetImportPreviews,
  createPetImportSession,
  discardPetImportPreviews,
  getDownloadsDir,
  previewCodexPetImports,
  previewPetImportFolders,
  previewPetImportZips,
} from "../lib/appCommands";
import type {
  PetImportPreview,
  PetImportPreviewBatch,
  PetImportSession,
} from "../lib/appTypes";

const CHOOSE_FOLDERS_TITLE = "Choose folders";
const CHOOSE_ZIP_TITLE = "Choose zip";
const SKIPPED_INVALID_PACKAGES = "Skipped invalid packages";

function normalizeDialogPaths(value: string | string[] | null): string[] {
  if (Array.isArray(value)) {
    return value;
  }
  return typeof value === "string" ? [value] : [];
}

type PreviewState = {
  previews: PetImportPreview[];
  selectedPreviewIds: Set<string>;
};

export function usePetImport() {
  const [session, setSession] = useState<PetImportSession | null>(null);
  const [previewState, setPreviewState] = useState<PreviewState>(() => ({
    previews: [],
    selectedPreviewIds: new Set(),
  }));
  const [errors, setErrors] = useState<string[]>([]);
  const [isBusy, setIsBusy] = useState(false);

  const { previews, selectedPreviewIds } = previewState;
  const selectedCount = selectedPreviewIds.size;

  const appendErrors = useCallback((messages: string[]) => {
    if (messages.length === 0) {
      return;
    }
    setErrors((current) => [...current, ...messages]);
  }, []);

  const ensureSession = useCallback(async () => {
    if (session) {
      return session;
    }

    const result = await createPetImportSession();
    if (result.errorMessage || !result.session) {
      appendErrors([result.errorMessage ?? "Could not create import session."]);
      return null;
    }

    setSession(result.session);
    return result.session;
  }, [appendErrors, session]);

  const applyBatch = useCallback(
    (batch: PetImportPreviewBatch) => {
      setPreviewState((current) => {
        const existingIds = new Set(
          current.previews.map((preview) => preview.previewId),
        );
        const nextPreviews = [...current.previews];
        const nextSelectedIds = new Set(current.selectedPreviewIds);

        for (const preview of batch.previews) {
          if (existingIds.has(preview.previewId)) {
            continue;
          }
          existingIds.add(preview.previewId);
          nextPreviews.push(preview);
          if (preview.selectedByDefault) {
            nextSelectedIds.add(preview.previewId);
          }
        }

        return { previews: nextPreviews, selectedPreviewIds: nextSelectedIds };
      });

      appendErrors([
        ...batch.errors,
        ...(batch.skipped > 0
          ? [`${SKIPPED_INVALID_PACKAGES}: ${batch.skipped}`]
          : []),
      ]);
    },
    [appendErrors],
  );

  const withBusy = useCallback(async (action: () => Promise<void>) => {
    setIsBusy(true);
    try {
      await action();
    } finally {
      setIsBusy(false);
    }
  }, []);

  const previewCodex = useCallback(async () => {
    await withBusy(async () => {
      const activeSession = await ensureSession();
      if (!activeSession) {
        return;
      }

      const result = await previewCodexPetImports(activeSession.sessionId);
      if (result.errorMessage || !result.batch) {
        appendErrors([result.errorMessage ?? "Could not preview Codex pets."]);
        return;
      }

      applyBatch(result.batch);
    });
  }, [appendErrors, applyBatch, ensureSession, withBusy]);

  const previewFolders = useCallback(async () => {
    await withBusy(async () => {
      const defaultPath = await getDownloadsDir();
      const selectedPaths = normalizeDialogPaths(
        await open({
          canCreateDirectories: false,
          defaultPath: defaultPath ?? undefined,
          directory: true,
          multiple: true,
          title: CHOOSE_FOLDERS_TITLE,
        }),
      );

      if (selectedPaths.length === 0) {
        return;
      }

      const activeSession = await ensureSession();
      if (!activeSession) {
        return;
      }

      const result = await previewPetImportFolders(
        activeSession.sessionId,
        selectedPaths,
      );
      if (result.errorMessage || !result.batch) {
        appendErrors([result.errorMessage ?? "Could not preview folders."]);
        return;
      }

      applyBatch(result.batch);
    });
  }, [appendErrors, applyBatch, ensureSession, withBusy]);

  const previewZips = useCallback(async () => {
    await withBusy(async () => {
      const defaultPath = await getDownloadsDir();
      const selectedPaths = normalizeDialogPaths(
        await open({
          canCreateDirectories: false,
          defaultPath: defaultPath ?? undefined,
          directory: false,
          filters: [{ extensions: ["zip"], name: "Zip archives" }],
          multiple: true,
          title: CHOOSE_ZIP_TITLE,
        }),
      );

      if (selectedPaths.length === 0) {
        return;
      }

      const activeSession = await ensureSession();
      if (!activeSession) {
        return;
      }

      const result = await previewPetImportZips(
        activeSession.sessionId,
        selectedPaths,
      );
      if (result.errorMessage || !result.batch) {
        appendErrors([result.errorMessage ?? "Could not preview zip files."]);
        return;
      }

      applyBatch(result.batch);
    });
  }, [appendErrors, applyBatch, ensureSession, withBusy]);

  const commitPreviews = useCallback(
    async (previewIds: string[]) => {
      if (!session || previewIds.length === 0) {
        return;
      }

      await withBusy(async () => {
        const result = await commitPetImportPreviews(
          session.sessionId,
          previewIds,
        );
        if (result.errorMessage || !result.result) {
          appendErrors([result.errorMessage ?? "Could not import pets."]);
          return;
        }

        const failedPreviewIds = new Set(
          result.result.failed.map((failure) => failure.previewId),
        );
        const committedPreviewIds = new Set(
          previewIds.filter((previewId) => !failedPreviewIds.has(previewId)),
        );

        setPreviewState((current) => {
          const nextSelectedIds = new Set(current.selectedPreviewIds);
          for (const previewId of committedPreviewIds) {
            nextSelectedIds.delete(previewId);
          }
          return {
            previews: current.previews.filter(
              (preview) => !committedPreviewIds.has(preview.previewId),
            ),
            selectedPreviewIds: nextSelectedIds,
          };
        });
        appendErrors(
          result.result.failed.map(
            (failure) => `${failure.previewId}: ${failure.errorMessage}`,
          ),
        );
      });
    },
    [appendErrors, session, withBusy],
  );

  const importSelected = useCallback(async () => {
    await commitPreviews(Array.from(selectedPreviewIds));
  }, [commitPreviews, selectedPreviewIds]);

  const importAll = useCallback(async () => {
    await commitPreviews(previews.map((preview) => preview.previewId));
  }, [commitPreviews, previews]);

  const removePreview = useCallback((previewId: string) => {
    setPreviewState((current) => {
      const nextSelectedIds = new Set(current.selectedPreviewIds);
      nextSelectedIds.delete(previewId);
      return {
        previews: current.previews.filter(
          (preview) => preview.previewId !== previewId,
        ),
        selectedPreviewIds: nextSelectedIds,
      };
    });
  }, []);

  const togglePreview = useCallback((previewId: string) => {
    setPreviewState((current) => {
      const next = new Set(current.selectedPreviewIds);
      if (next.has(previewId)) {
        next.delete(previewId);
      } else {
        next.add(previewId);
      }
      return { ...current, selectedPreviewIds: next };
    });
  }, []);

  const selectAll = useCallback(() => {
    setPreviewState((current) => ({
      ...current,
      selectedPreviewIds: new Set(
        current.previews.map((preview) => preview.previewId),
      ),
    }));
  }, []);

  const clearError = useCallback(() => {
    setErrors([]);
  }, []);

  const closeSession = useCallback(async () => {
    const activeSession = session;
    setSession(null);
    setPreviewState({ previews: [], selectedPreviewIds: new Set() });
    setErrors([]);

    if (activeSession) {
      const result = await discardPetImportPreviews(activeSession.sessionId);
      if (result.errorMessage) {
        setErrors([result.errorMessage]);
      }
    }
  }, [session]);

  return useMemo(
    () => ({
      clearError,
      closeSession,
      errors,
      importAll,
      importSelected,
      isBusy,
      previewCodex,
      previewFolders,
      previewZips,
      previews,
      removePreview,
      selectAll,
      selectedCount,
      selectedPreviewIds,
      session,
      togglePreview,
    }),
    [
      clearError,
      closeSession,
      errors,
      importAll,
      importSelected,
      isBusy,
      previewCodex,
      previewFolders,
      previewZips,
      previews,
      removePreview,
      selectAll,
      selectedCount,
      selectedPreviewIds,
      session,
      togglePreview,
    ],
  );
}

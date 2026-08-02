/**
 * Forced launcher updates via tauri-plugin-updater.
 * If a newer release exists, the UI is blocked until install + relaunch.
 */
import { reactive } from "vue";
import { getVersion } from "@tauri-apps/api/app";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { info as logInfo, warn as logWarn, error as logError } from "@tauri-apps/plugin-log";

export type UpdatePhase =
  | "idle"
  | "checking"
  | "available"
  | "downloading"
  | "installing"
  | "relaunching"
  | "error";

export interface UpdateState {
  phase: UpdatePhase;
  /** True while an update is required / in progress - blocks the rest of the app. */
  blocking: boolean;
  /** Installed app version when the update was detected. */
  currentVersion: string;
  /** Target release version from the updater endpoint. */
  version: string;
  notes: string;
  downloaded: number;
  contentLength: number;
  error: string | null;
}

export const updateState = reactive<UpdateState>({
  phase: "idle",
  blocking: false,
  currentVersion: "",
  version: "",
  notes: "",
  downloaded: 0,
  contentLength: 0,
  error: null,
});

function formatVersion(v: string): string {
  const trimmed = v.trim();
  if (!trimmed) return "…";
  return trimmed.startsWith("v") ? trimmed : `v${trimmed}`;
}

export function updateVersionLabel(): string {
  return formatVersion(updateState.version);
}

export function updateFromToLabel(): string {
  const to = formatVersion(updateState.version);
  const from = updateState.currentVersion
    ? formatVersion(updateState.currentVersion)
    : null;
  return from ? `${from} → ${to}` : to;
}

export const updatePercent = () => {
  const total = updateState.contentLength;
  if (!total) return 0;
  return Math.min(100, Math.round((updateState.downloaded / total) * 100));
};

async function applyUpdate(update: Update) {
  updateState.phase = "available";
  updateState.blocking = true;
  updateState.version = update.version;
  updateState.notes = update.body ?? "";
  updateState.downloaded = 0;
  updateState.contentLength = 0;
  updateState.error = null;
  try {
    updateState.currentVersion = await getVersion();
  } catch {
    updateState.currentVersion = "";
  }

  await logInfo(
    `Update ${formatVersion(update.version)} required` +
      (updateState.currentVersion
        ? ` (from ${formatVersion(updateState.currentVersion)})`
        : "") +
      " - downloading…",
  );

  updateState.phase = "downloading";
  await update.downloadAndInstall((event) => {
    switch (event.event) {
      case "Started":
        updateState.contentLength = event.data.contentLength ?? 0;
        break;
      case "Progress":
        updateState.downloaded += event.data.chunkLength;
        break;
      case "Finished":
        updateState.phase = "installing";
        break;
    }
  });

  updateState.phase = "relaunching";
  await logInfo(`Update ${update.version} installed - relaunching`);
  await relaunch();
}

/**
 * Check GitHub latest.json and force-install if newer.
 * Skips in Vite dev. Network / check failures do not block the launcher
 * (no update metadata available); a found update always blocks.
 */
export async function enforceLauncherUpdate(): Promise<void> {
  if (import.meta.env.DEV) {
    updateState.phase = "idle";
    updateState.blocking = false;
    return;
  }

  updateState.phase = "checking";
  updateState.blocking = false;
  updateState.error = null;

  try {
    const update = await check();
    if (!update) {
      updateState.phase = "idle";
      return;
    }
    await applyUpdate(update);
  } catch (e) {
    const message = e instanceof Error ? e.message : String(e);
    // Still blocked if we already started downloading; otherwise allow use.
    if (updateState.blocking) {
      updateState.phase = "error";
      updateState.error = message;
      await logError(`Forced update failed: ${message}`);
    } else {
      updateState.phase = "idle";
      await logWarn(`Update check skipped: ${message}`);
    }
  }
}

/** Retry after a failed forced install. */
export async function retryForcedUpdate(): Promise<void> {
  updateState.error = null;
  updateState.phase = "checking";
  try {
    const update = await check();
    if (!update) {
      // Already on latest somehow — unblock.
      updateState.blocking = false;
      updateState.phase = "idle";
      return;
    }
    await applyUpdate(update);
  } catch (e) {
    const message = e instanceof Error ? e.message : String(e);
    updateState.blocking = true;
    updateState.phase = "error";
    updateState.error = message;
    await logError(`Forced update retry failed: ${message}`);
  }
}

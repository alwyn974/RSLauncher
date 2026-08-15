/**
 * Launcher state - backed by Tauri commands/events:
 *
 *   commands: login_with_microsoft, remove_account, set_active_account,
 *             get_settings, save_settings, get_catalog, list_modpacks,
 *             set_active_modpack, play, cancel, stop
 *   events:   "launch://progress" -> Progress
 *             "auth://device_code" -> { code, url }
 *             "auth://status"     -> { step }
 *             "instance://status" -> { installed }
 *   logs:     tauri-plugin-log (attachConsole + attachLogger)
 */
import { computed, reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  attachConsole,
  attachLogger,
  error as logError,
  info as logInfo,
  warn as logWarn,
  LogLevel,
} from "@tauri-apps/plugin-log";

export interface Account {
  id: string;
  username: string;
  uuid: string;
  avatarSeed: number;
  needsReauth: boolean;
}

export interface Settings {
  ramGb: number;
  width: number;
  height: number;
  fullscreen: boolean;
  jvmArgs: string;
  serverName: string;
  serverAddress: string;
  enabledOptionalMods: Record<string, boolean>;
  enabledShaderVariants: Record<string, boolean>;
}

export interface OptionalModInfo {
  id: string;
  name: string;
  description: string;
  source: string;
  defaultEnabled: boolean;
  enabled: boolean;
}

export interface BundleMemberInfo {
  id: string;
  name: string;
  description: string;
  source: string;
  defaultEnabled: boolean;
  required: boolean;
  enabled: boolean;
  locked: boolean;
}

export interface OptionalBundleInfo {
  id: string;
  name: string;
  description: string;
  defaultEnabled: boolean;
  enabled: boolean;
  mods: BundleMemberInfo[];
}

export interface ShaderVariantInfo {
  id: string;
  name: string;
  description: string;
  packName: string;
  defaultEnabled: boolean;
  enabled: boolean;
}

export interface ModpackInfo {
  id: string;
  name: string;
  version: string;
  minecraft: string;
  loader: string;
  loaderVersion: string;
  modCount: number;
  instanceName: string;
}

export interface ModpackListEntry {
  id: string;
  name: string;
  version: string;
  minecraft: string;
  loader: string;
  loaderVersion: string;
  instanceName: string;
  installed: boolean;
}

export interface Catalog {
  modpack: ModpackInfo;
  optionalMods: OptionalModInfo[];
  optionalBundles: OptionalBundleInfo[];
  shaderVariants: ShaderVariantInfo[];
}

export interface MemoryInfo {
  totalGb: number;
  recommendedGb: number;
  minGb: number;
}

export interface StorageInfo {
  root: string;
  dataDir: string;
  cacheDir: string;
  javaDir: string;
  custom: boolean;
  installedBytes: number;
  freeBytes: number;
}

export interface StorageLocationCheck {
  root: string;
  bytesToMove: number;
  freeBytes: number;
  requiredBytes: number;
}

export interface StorageMigrationProgress {
  stage: "copying" | "verifying" | "done";
  detail: string;
  bytesCopied: number;
  bytesTotal: number;
  percent: number;
}

export type LaunchStage =
  | "idle"
  | "preparing"
  | "java"
  | "loader"
  | "downloading"
  | "verifying"
  | "launching"
  | "running"
  | "error";

/** Ordered launch pipeline shown in the progress panel. */
export const LAUNCH_STEPS: { id: LaunchStage; label: string }[] = [
  { id: "preparing", label: "Prepare" },
  { id: "java", label: "Java" },
  { id: "loader", label: "Loader" },
  { id: "downloading", label: "Download" },
  { id: "verifying", label: "Verify" },
  { id: "launching", label: "Launch" },
  { id: "running", label: "Running" },
];

export interface Progress {
  stage: LaunchStage;
  step: string;
  file: string;
  filesDone: number;
  filesTotal: number;
  percent: number;
  bytesPerSec: number;
  etaSec: number;
}

export interface LogLine {
  time: string;
  level: "INFO" | "WARN" | "ERROR";
  source: string;
  message: string;
}

export interface DeviceCode {
  code: string;
  url: string;
}

export type View = "play" | "settings" | "logs";

const EMPTY_MODPACK: ModpackInfo = {
  id: "",
  name: "…",
  version: "",
  minecraft: "",
  loader: "",
  loaderVersion: "",
  modCount: 0,
  instanceName: "",
};

/** Fallback until backend settings load. */
const DEFAULT_SETTINGS: Settings = {
  ramGb: 8,
  width: 1024,
  height: 768,
  fullscreen: false,
  jvmArgs: "",
  serverName: "",
  serverAddress: "",
  enabledOptionalMods: {},
  enabledShaderVariants: {},
};

const DEFAULT_MEMORY: MemoryInfo = {
  totalGb: 16,
  recommendedGb: 8,
  minGb: 2,
};

const IDLE_PROGRESS: Progress = {
  stage: "idle",
  step: "",
  file: "",
  filesDone: 0,
  filesTotal: 0,
  percent: 0,
  bytesPerSec: 0,
  etaSec: 0,
};

interface LauncherState {
  accounts: Account[];
  activeAccountId: string | null;
  modpacks: ModpackListEntry[];
  activePackId: string | null;
  settings: Settings;
  catalog: Catalog;
  memory: MemoryInfo;
  storage: StorageInfo | null;
  storageMigration: StorageMigrationProgress | null;
  storageRestartRequired: boolean;
  view: View;
  loginPending: boolean;
  loginError: string | null;
  deviceCode: DeviceCode | null;
  authStep: string;
  progress: Progress;
  /** Last pipeline step reached - kept on error/cancel so the UI marks the right step. */
  progressPhase: LaunchStage;
  logs: LogLine[];
  installed: boolean;
  ready: boolean;
}

const EMPTY_CATALOG: Catalog = {
  modpack: { ...EMPTY_MODPACK },
  optionalMods: [],
  optionalBundles: [],
  shaderVariants: [],
};

const state = reactive<LauncherState>({
  accounts: [],
  activeAccountId: null,
  modpacks: [],
  activePackId: null,
  settings: {
    ...DEFAULT_SETTINGS,
    enabledOptionalMods: {},
    enabledShaderVariants: {},
  },
  catalog: { ...EMPTY_CATALOG },
  memory: { ...DEFAULT_MEMORY },
  storage: null,
  storageMigration: null,
  storageRestartRequired: false,
  view: "play",
  loginPending: false,
  loginError: null,
  deviceCode: null,
  authStep: "",
  progress: { ...IDLE_PROGRESS },
  progressPhase: "preparing",
  logs: [],
  installed: false,
  ready: false,
});

function isPipelineStage(stage: string): stage is LaunchStage {
  return LAUNCH_STEPS.some((s) => s.id === stage);
}

function stageRank(stage: LaunchStage): number {
  if (stage === "idle") return -1;
  if (stage === "error") return 1000;
  const i = LAUNCH_STEPS.findIndex((s) => s.id === stage);
  return i >= 0 ? i : -1;
}

function applyProgress(next: Progress) {
  const stage = (next.stage as LaunchStage) || "idle";
  const current = state.progress.stage;

  // Ignore true late events (e.g. InstallProgress after JVM is up).
  // Do NOT drop "downloading" just because an early ZIP extract briefly
  // reported a later-looking stage — that was fixed on the Rust side by
  // keeping CoreEvent extracts on "loader".
  if (
    stage !== "idle" &&
    stage !== "error" &&
    isPipelineStage(stage) &&
    isPipelineStage(current) &&
    stageRank(stage) < stageRank(current) &&
    // Once launching/running, freeze earlier stages.
    (current === "launching" || current === "running")
  ) {
    return;
  }

  if (isPipelineStage(stage)) {
    state.progressPhase = stage;
  }
  state.progress = { ...IDLE_PROGRESS, ...next, stage };
}

function errorMessage(error: unknown): string {
  if (typeof error === "string" && error.trim()) return error;
  if (error && typeof error === "object") {
    const obj = error as { message?: unknown; error?: unknown };
    if (typeof obj.message === "string" && obj.message.trim()) return obj.message;
    if (typeof obj.error === "string" && obj.error.trim()) return obj.error;
  }
  const fallback = String(error ?? "Unknown error");
  return fallback === "[object Object]" ? "Sign-in failed" : fallback;
}

const activeAccount = computed<Account | null>(
  () => state.accounts.find((a) => a.id === state.activeAccountId) ?? null,
);

const busy = computed(() =>
  ["preparing", "java", "loader", "downloading", "verifying", "launching"].includes(
    state.progress.stage,
  ),
);

const storageBusy = computed(() => {
  const stage = state.storageMigration?.stage;
  return stage === "copying" || stage === "verifying" || state.storageRestartRequired;
});

/** Logs captured since the current (or last) sign-in attempt began. */
let loginLogCursor = 0;
const loginLogs = computed(() => state.logs.slice(loginLogCursor));

const loginStatus = computed(() => {
  if (state.authStep.trim()) return state.authStep;
  if (state.deviceCode) return "Waiting for Microsoft approval…";
  if (state.loginPending) return "Contacting Microsoft…";
  return "";
});

let unlisteners: UnlistenFn[] = [];
let initPromise: Promise<void> | null = null;

function now(): string {
  return new Date().toLocaleTimeString("en-GB", { hour12: false });
}

function pushLog(level: LogLine["level"], source: string, message: string) {
  state.logs.push({ time: now(), level, source, message });
  if (state.logs.length > 2000) state.logs.splice(0, state.logs.length - 2000);
}

/** Frontend + backend logs all go through tauri-plugin-log → attachLogger → UI. */
function log(level: LogLine["level"], source: string, message: string) {
  const line = `[${source}] ${message}`;
  void (level === "ERROR"
    ? logError(line)
    : level === "WARN"
      ? logWarn(line)
      : logInfo(line));
}

function parseBackendLog(message: string): { source: string; message: string } {
  const match = /^\[([^\]]+)]\s*([\s\S]*)$/.exec(message);
  if (!match) return { source: "backend", message };
  return { source: match[1] || "backend", message: match[2] ?? "" };
}

function levelFromPlugin(level: LogLevel): LogLine["level"] {
  if (level >= LogLevel.Error) return "ERROR";
  if (level >= LogLevel.Warn) return "WARN";
  return "INFO";
}

function syncActive(accounts: Account[], active: Account | null) {
  state.accounts = accounts;
  state.activeAccountId =
    active?.id ?? accounts.find((account) => !account.needsReauth)?.id ?? null;

  if (!state.activeAccountId && accounts.some((account) => account.needsReauth)) {
    state.loginError ??=
      "Your saved Microsoft account needs to reconnect. You do not need to remove it.";
  }
}

async function refreshAccountSessions() {
  try {
    await invoke("refresh_account_sessions");
    const [accounts, active] = await Promise.all([
      invoke<Account[]>("list_accounts"),
      invoke<Account | null>("get_active_account"),
    ]);
    syncActive(accounts, active);
  } catch (e) {
    // A network outage must not sign the player out. The backend only marks
    // definitely invalid/revoked refresh tokens as requiring reconnection.
    log("WARN", "auth", `Background session refresh failed: ${errorMessage(e)}`);
  }
}

export async function initLauncher() {
  if (initPromise) return initPromise;
  initPromise = (async () => {
    for (const u of unlisteners) u();
    unlisteners = [];

    unlisteners.push(
      await attachConsole(),
      await attachLogger(({ level, message }) => {
        if (level < LogLevel.Info) return;
        const parsed = parseBackendLog(message);
        pushLog(levelFromPlugin(level), parsed.source, parsed.message);
      }),
      await listen<Progress>("launch://progress", (e) => {
        applyProgress({
          ...IDLE_PROGRESS,
          ...e.payload,
          stage: (e.payload.stage as LaunchStage) || "idle",
        });
        if (e.payload.stage === "running" || e.payload.stage === "idle") {
          void refreshInstallStatus();
        }
      }),
      await listen<DeviceCode>("auth://device_code", (e) => {
        state.deviceCode = e.payload;
        if (state.loginPending) {
          log(
            "INFO",
            "auth",
            `Device code ready: ${e.payload.code} - open ${e.payload.url}`,
          );
        }
      }),
      await listen<{ step: string }>("auth://status", (e) => {
        state.authStep = e.payload.step;
      }),
      await listen<{ uuid: string; message: string }>("auth://reauth_required", (e) => {
        const account = state.accounts.find((item) => item.uuid === e.payload.uuid);
        if (account) account.needsReauth = true;
        if (state.activeAccountId === e.payload.uuid) state.activeAccountId = null;
        state.loginError = e.payload.message;
        log("WARN", "auth", e.payload.message);
      }),
      await listen<{ installed: boolean; modCount: number }>("instance://status", (e) => {
        state.installed = e.payload.installed;
        state.catalog.modpack.modCount = e.payload.modCount;
      }),
      await listen<MemoryInfo>("memory://updated", (e) => {
        state.memory = { ...DEFAULT_MEMORY, ...e.payload };
        // Only bump default allocation if the user hasn't customized it yet.
        if (state.settings.ramGb === DEFAULT_SETTINGS.ramGb) {
          state.settings.ramGb = Math.min(
            Math.max(state.memory.recommendedGb, state.memory.minGb),
            state.memory.totalGb,
          );
        }
      }),
      await listen<StorageMigrationProgress>("storage://progress", (e) => {
        state.storageMigration = e.payload;
      }),
    );

    try {
      const [accounts, settings, catalog, active, memory, status, modpacks, activePackId, storage] =
        await Promise.all([
          invoke<Account[]>("list_accounts"),
          invoke<Settings>("get_settings"),
          invoke<Catalog>("get_catalog"),
          invoke<Account | null>("get_active_account"),
          invoke<MemoryInfo>("get_memory_info"),
          invoke<{ installed: boolean; modCount: number }>("get_instance_status"),
          invoke<ModpackListEntry[]>("list_modpacks"),
          invoke<string>("get_active_modpack"),
          invoke<StorageInfo>("get_storage_info"),
        ]);
      syncActive(accounts, active);
      state.modpacks = modpacks;
      state.activePackId = activePackId;
      state.memory = { ...DEFAULT_MEMORY, ...memory };
      state.storage = storage;
      state.installed = status.installed;
      state.catalog = catalog;
      state.catalog.modpack.modCount = status.modCount;
      state.settings = {
        ...DEFAULT_SETTINGS,
        ...settings,
        enabledOptionalMods: { ...settings.enabledOptionalMods },
        enabledShaderVariants: { ...settings.enabledShaderVariants },
        ramGb: Math.min(
          Math.max(settings.ramGb ?? state.memory.recommendedGb, state.memory.minGb),
          state.memory.totalGb,
        ),
      };
    } catch (e) {
      log("ERROR", "launcher", `Failed to load state: ${errorMessage(e)}`);
    } finally {
      state.ready = true;
      void refreshAccountSessions();
    }
  })();
  return initPromise;
}

async function login() {
  if (state.loginPending) return;
  state.loginPending = true;
  state.loginError = null;
  state.deviceCode = null;
  state.authStep = "";
  loginLogCursor = state.logs.length;
  log("INFO", "launcher", "Opening Microsoft sign-in…");
  try {
    const account = await invoke<Account>("login_with_microsoft");
    const accounts = await invoke<Account[]>("list_accounts");
    syncActive(accounts, account);
    state.loginError = null;
    log("INFO", "launcher", `Signed in as ${account.username}`);
  } catch (e) {
    const message = errorMessage(e);
    state.loginError = message;
    log("ERROR", "launcher", message);
  } finally {
    state.loginPending = false;
    state.deviceCode = null;
    state.authStep = "";
  }
}

async function removeAccount(id: string) {
  try {
    await invoke("remove_account", { id });
    const [accounts, active] = await Promise.all([
      invoke<Account[]>("list_accounts"),
      invoke<Account | null>("get_active_account"),
    ]);
    syncActive(accounts, active);
  } catch (e) {
    log("ERROR", "launcher", String(e));
  }
}

async function setActiveAccount(id: string) {
  const account = state.accounts.find((item) => item.id === id);
  if (account?.needsReauth) {
    state.activeAccountId = null;
    await login();
    return;
  }
  try {
    await invoke("set_active_account", { id });
    state.activeAccountId = id;
  } catch (e) {
    log("ERROR", "launcher", String(e));
  }
}

async function setActiveModpack(id: string) {
  if (busy.value || storageBusy.value || state.progress.stage === "running") {
    log("WARN", "launcher", "Cannot switch modpack while launching or running");
    return;
  }
  try {
    await invoke<ModpackListEntry>("set_active_modpack", { id });
    state.activePackId = id;
    const [settings, catalog, memory, status, modpacks] = await Promise.all([
      invoke<Settings>("get_settings"),
      invoke<Catalog>("get_catalog"),
      invoke<MemoryInfo>("get_memory_info"),
      invoke<{ installed: boolean; modCount: number }>("get_instance_status"),
      invoke<ModpackListEntry[]>("list_modpacks"),
    ]);
    state.modpacks = modpacks;
    state.memory = { ...DEFAULT_MEMORY, ...memory };
    state.installed = status.installed;
    state.catalog = catalog;
    state.catalog.modpack.modCount = status.modCount;
    state.settings = {
      ...DEFAULT_SETTINGS,
      ...settings,
      enabledOptionalMods: { ...settings.enabledOptionalMods },
      enabledShaderVariants: { ...settings.enabledShaderVariants },
      ramGb: Math.min(
        Math.max(settings.ramGb ?? state.memory.recommendedGb, state.memory.minGb),
        state.memory.totalGb,
      ),
    };
    applyProgress({ ...IDLE_PROGRESS });
    log("INFO", "launcher", `Active modpack → ${catalog.modpack.name}`);
  } catch (e) {
    log("ERROR", "launcher", String(e));
  }
}

async function saveSettings(next: Settings) {
  try {
    const saved = await invoke<Settings>("save_settings", { settings: next });
    state.settings = {
      ...DEFAULT_SETTINGS,
      ...saved,
      enabledOptionalMods: { ...saved.enabledOptionalMods },
      enabledShaderVariants: { ...saved.enabledShaderVariants },
    };
    state.catalog = await invoke<Catalog>("get_catalog");
    await refreshInstallStatus();
    log("INFO", "launcher", "Settings saved");
  } catch (e) {
    log("ERROR", "launcher", String(e));
  }
}

function resetSettings(): Settings {
  const ramGb = Math.min(
    Math.max(state.memory.recommendedGb, state.memory.minGb),
    state.memory.totalGb,
  );
  return {
    ...DEFAULT_SETTINGS,
    ramGb,
    enabledOptionalMods: {},
    enabledShaderVariants: {},
  };
}

async function refreshInstallStatus() {
  try {
    const status = await invoke<{ installed: boolean; modCount: number }>(
      "get_instance_status",
    );
    state.installed = status.installed;
    state.catalog.modpack.modCount = status.modCount;
  } catch {
    /* ignore */
  }
}

async function inspectStorageLocation(path: string): Promise<StorageLocationCheck> {
  return invoke<StorageLocationCheck>("inspect_storage_location", { path });
}

async function migrateStorage(path: string): Promise<void> {
  state.storageMigration = {
    stage: "copying",
    detail: "Preparing files…",
    bytesCopied: 0,
    bytesTotal: 0,
    percent: 0,
  };
  try {
    state.storage = await invoke<StorageInfo>("migrate_storage", { path });
    state.storageRestartRequired = true;
    log("INFO", "storage", `Storage moved to ${state.storage.root}`);
  } catch (e) {
    state.storageMigration = null;
    log("ERROR", "storage", errorMessage(e));
    throw e;
  }
}

async function play(quickPlay = false) {
  if (busy.value || storageBusy.value || !activeAccount.value) return;
  const mode = quickPlay ? "Quick Play" : "Play";
  log(
    "INFO",
    "launcher",
    `${mode}: ${state.catalog.modpack.name} ${state.catalog.modpack.version} for ${activeAccount.value.username}`,
  );
  applyProgress({
    ...IDLE_PROGRESS,
    stage: "preparing",
    step: "Preparing launch",
  });
  try {
    await invoke("play", { quickPlay });
  } catch (e) {
    const message = errorMessage(e);
    log("ERROR", "launcher", message);
    applyProgress({
      ...state.progress,
      stage: "error",
      step: "Launch failed",
      file: message,
    });
  }
}

async function cancel() {
  if (!busy.value) return;
  try {
    await invoke("cancel");
    applyProgress({
      ...state.progress,
      stage: "error",
      step: "Cancelled",
      file: "Launch cancelled",
      bytesPerSec: 0,
      etaSec: 0,
    });
    log("WARN", "launcher", "Launch cancelled");
  } catch (e) {
    log("ERROR", "launcher", String(e));
  }
}

async function stop() {
  if (state.progress.stage !== "running") return;
  try {
    await invoke("stop");
    applyProgress({ ...IDLE_PROGRESS });
    log("INFO", "launcher", "Game closed");
  } catch (e) {
    log("ERROR", "launcher", String(e));
  }
}

function setView(view: View) {
  state.view = view;
}

function clearLogs() {
  state.logs = [];
}

export const launcher = {
  state,
  activeAccount,
  busy,
  storageBusy,
  loginLogs,
  loginStatus,
  login,
  removeAccount,
  setActiveAccount,
  setActiveModpack,
  saveSettings,
  resetSettings,
  inspectStorageLocation,
  migrateStorage,
  play,
  cancel,
  stop,
  setView,
  clearLogs,
};

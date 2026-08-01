/**
 * Mock launcher state.
 *
 * Everything here is simulated, but the shapes deliberately mirror what the
 * future Rust backend will expose over Tauri commands/events:
 *
 *   commands: login_with_microsoft, remove_account, set_active_account,
 *             get_settings, save_settings, play, cancel
 *   events:   "launch://progress" -> Progress
 *             "game://log"        -> LogLine
 *
 * Swapping the mock for real `invoke()`/`listen()` calls should not touch the
 * views or components.
 */
import { computed, reactive } from "vue";

export interface Account {
  id: string;
  username: string;
  uuid: string;
  avatarSeed: number;
}

export interface Settings {
  ramGb: number;
  width: number;
  height: number;
  fullscreen: boolean;
  jvmArgs: string;
}

export type LaunchStage =
  | "idle"
  | "preparing"
  | "downloading"
  | "verifying"
  | "launching"
  | "running"
  | "error";

export interface Progress {
  stage: LaunchStage;
  /** Current step description, e.g. "Downloading files". */
  step: string;
  /** File currently being processed. */
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

export type View = "play" | "settings" | "logs";

export const MODPACK = {
  name: "RS Modpack",
  minecraft: "1.20.1",
  loader: "Fabric 0.16.9",
  modCount: 214,
} as const;

const DEFAULT_SETTINGS: Settings = {
  ramGb: 4,
  width: 1280,
  height: 720,
  fullscreen: false,
  jvmArgs: "",
};

const LS_ACCOUNTS = "rs.accounts";
const LS_ACTIVE = "rs.activeAccount";
const LS_SETTINGS = "rs.settings";

function readLs<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(key);
    return raw ? (JSON.parse(raw) as T) : fallback;
  } catch {
    return fallback;
  }
}

function writeLs(key: string, value: unknown) {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch {
    /* storage unavailable — session-only state */
  }
}

function now(): string {
  return new Date().toLocaleTimeString("en-GB", { hour12: false });
}

function mulberry32(seed: number) {
  let a = seed >>> 0;
  return () => {
    a |= 0;
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/** Fake gamertags so repeated mock logins produce distinct accounts. */
const MOCK_NAMES = [
  "BlockSmith42",
  "RedstoneRay",
  "CreeperHunter",
  "EnderWitch",
  "DiamondDigger",
  "NetherNomad",
  "CraftyFox",
  "ObsidianOwl",
];

/** Mock modpack file list for the download simulation. */
const MOCK_FILES = [
  "mods/jei-1.20.1-fabric-15.20.0.jar",
  "mods/sodium-fabric-0.5.11+mc1.20.1.jar",
  "mods/lithium-fabric-0.11.2.jar",
  "mods/fabric-api-0.92.2+1.20.1.jar",
  "mods/create-fabric-0.5.1f.jar",
  "mods/iris-1.7.2+1.20.1.jar",
  "mods/terrablender-fabric-3.0.1.7.jar",
  "mods/biomesoplenty-fabric-18.0.0.592.jar",
  "mods/waystones-fabric-14.1.5.jar",
  "mods/appleskin-fabric-2.5.1.jar",
  "config/jei/jei-client.ini",
  "config/sodium-options.json",
  "config/create-client.toml",
  "resourcepacks/rs-overhaul-1.4.zip",
  "libraries/net/fabricmc/loader/0.16.9/loader-0.16.9.jar",
  "libraries/org/lwjgl/lwjgl/3.3.3/lwjgl-3.3.3.jar",
  "libraries/com/google/guava/guava/32.1.2/guava-32.1.2.jar",
  "assets/minecraft/textures/blocks.index",
  "assets/minecraft/lang/en_us.json",
  "versions/1.20.1/1.20.1.jar",
];

const GAME_LOG_POOL: Array<[LogLine["level"], string, string]> = [
  ["INFO", "minecraft", "Setting user: %u"],
  ["INFO", "minecraft", "Backend library: LWJGL version 3.3.3"],
  ["INFO", "fabricloader", "Loading 214 mods"],
  ["INFO", "sodium", "OpenGL 4.6 detected, using GL4.6 renderer"],
  ["INFO", "create", "Registering kinetic stress network"],
  ["WARN", "minecraft", "Ambiguity between arguments [teleport, location] and [teleport, destination]"],
  ["INFO", "minecraft", "Created: 1024x512x4 minecraft:textures/atlas/blocks.png"],
  ["INFO", "minecraft", "Narrator library successfully loaded"],
  ["INFO", "minecraft", "Reloading ResourceManager: Default, Fabric Mods, rs-overhaul-1.4.zip"],
  ["INFO", "iris", "Shaders are disabled by configuration"],
  ["WARN", "waystones", "Config value 'worldGen' is deprecated"],
  ["INFO", "minecraft", "Sound engine started"],
  ["INFO", "minecraft", "Connecting to rs.example.org, 25565"],
];

interface LauncherState {
  accounts: Account[];
  activeAccountId: string | null;
  settings: Settings;
  view: View;
  loginPending: boolean;
  progress: Progress;
  logs: LogLine[];
  timers: number[];
}

const state = reactive<LauncherState>({
  accounts: readLs<Account[]>(LS_ACCOUNTS, []),
  activeAccountId: readLs<string | null>(LS_ACTIVE, null),
  settings: { ...DEFAULT_SETTINGS, ...readLs<Partial<Settings>>(LS_SETTINGS, {}) },
  view: "play",
  loginPending: false,
  progress: {
    stage: "idle",
    step: "",
    file: "",
    filesDone: 0,
    filesTotal: 0,
    percent: 0,
    bytesPerSec: 0,
    etaSec: 0,
  },
  logs: [],
  timers: [],
});

const activeAccount = computed<Account | null>(
  () => state.accounts.find((a) => a.id === state.activeAccountId) ?? null,
);

const busy = computed(() =>
  ["preparing", "downloading", "verifying", "launching"].includes(state.progress.stage),
);

function log(level: LogLine["level"], source: string, message: string) {
  state.logs.push({ time: now(), level, source, message });
  if (state.logs.length > 2000) state.logs.splice(0, state.logs.length - 2000);
}

function after(ms: number, fn: () => void) {
  const id = window.setTimeout(fn, ms);
  state.timers.push(id);
  return id;
}

function every(ms: number, fn: () => boolean | void) {
  const id = window.setInterval(() => {
    if (fn() === false) window.clearInterval(id);
  }, ms);
  state.timers.push(id);
  return id;
}

function clearTimers() {
  for (const id of state.timers) {
    window.clearTimeout(id);
    window.clearInterval(id);
  }
  state.timers = [];
}

// --- commands (mock) -------------------------------------------------------

/** login_with_microsoft — fake device-code round trip. */
function login() {
  if (state.loginPending) return;
  state.loginPending = true;
  log("INFO", "launcher", "Opening Microsoft sign-in...");
  after(1600, () => {
    const taken = new Set(state.accounts.map((a) => a.username));
    const username =
      MOCK_NAMES.find((n) => !taken.has(n)) ?? `Player${state.accounts.length + 1}`;
    const seed = Math.floor(Math.random() * 2 ** 31);
    const account: Account = {
      id: crypto.randomUUID(),
      username,
      uuid: crypto.randomUUID(),
      avatarSeed: seed,
    };
    state.accounts.push(account);
    state.activeAccountId = account.id;
    state.loginPending = false;
    writeLs(LS_ACCOUNTS, state.accounts);
    writeLs(LS_ACTIVE, state.activeAccountId);
    log("INFO", "launcher", `Signed in as ${username}`);
  });
}

/** remove_account */
function removeAccount(id: string) {
  state.accounts = state.accounts.filter((a) => a.id !== id);
  if (state.activeAccountId === id) {
    state.activeAccountId = state.accounts[0]?.id ?? null;
  }
  writeLs(LS_ACCOUNTS, state.accounts);
  writeLs(LS_ACTIVE, state.activeAccountId);
}

/** set_active_account */
function setActiveAccount(id: string) {
  if (state.accounts.some((a) => a.id === id)) {
    state.activeAccountId = id;
    writeLs(LS_ACTIVE, id);
  }
}

/** save_settings */
function saveSettings(next: Settings) {
  state.settings = { ...next };
  writeLs(LS_SETTINGS, state.settings);
  log("INFO", "launcher", "Settings saved");
}

function resetSettings(): Settings {
  return { ...DEFAULT_SETTINGS };
}

/** play — simulated launch pipeline. */
function play() {
  if (busy.value || !activeAccount.value) return;
  const s = state.settings;
  const account = activeAccount.value;

  // --- preparing -----------------------------------------------------------
  state.progress = {
    stage: "preparing",
    step: "Preparing launch",
    file: "",
    filesDone: 0,
    filesTotal: MOCK_FILES.length,
    percent: 0,
    bytesPerSec: 0,
    etaSec: 0,
  };
  log("INFO", "launcher", `Starting ${MODPACK.name} for ${account.username}`);
  log("INFO", "launcher", `RAM: ${s.ramGb} GB · ${s.fullscreen ? "fullscreen" : `${s.width}x${s.height}`}`);
  if (s.jvmArgs.trim()) log("INFO", "launcher", `Custom JVM args: ${s.jvmArgs.trim()}`);
  log("INFO", "launcher", "Checking modpack manifest...");

  after(900, () => {
    // --- downloading -------------------------------------------------------
    state.progress.stage = "downloading";
    state.progress.step = "Downloading files";
    let done = 0;
    const tickMs = 90;
    every(tickMs, () => {
      // advance 1–3 files per tick
      done = Math.min(MOCK_FILES.length, done + 1 + Math.floor(Math.random() * 3));
      const p = state.progress;
      p.filesDone = done;
      p.file = MOCK_FILES[done - 1] ?? "";
      p.percent = Math.round((done / MOCK_FILES.length) * 100);
      p.bytesPerSec = (7 + Math.random() * 8) * 1024 * 1024;
      p.etaSec = Math.max(0, Math.ceil(((MOCK_FILES.length - done) / MOCK_FILES.length) * 9));
      if (done >= MOCK_FILES.length) {
        log("INFO", "launcher", `Downloaded ${MOCK_FILES.length} files`);
        after(400, startVerify);
        return false;
      }
    });
  });
}

function startVerify() {
  state.progress = {
    ...state.progress,
    stage: "verifying",
    step: "Verifying integrity",
    file: "",
    percent: 100,
    bytesPerSec: 0,
    etaSec: 1,
  };
  log("INFO", "launcher", "Verifying file checksums...");
  after(1100, () => {
    state.progress = {
      ...state.progress,
      stage: "launching",
      step: "Launching game",
      etaSec: 0,
    };
    const s = state.settings;
    log("INFO", "launcher", `java -Xmx${s.ramGb}G -Xms${Math.min(s.ramGb, 2)}G ${s.jvmArgs.trim()} -jar fabric-loader.jar`);
    after(1400, startGame);
  });
}

function startGame() {
  state.progress = { ...state.progress, stage: "running", step: "Game running" };
  const rnd = mulberry32(Date.now());
  every(350 + rnd() * 500, () => {
    const [level, source, message] = GAME_LOG_POOL[Math.floor(rnd() * GAME_LOG_POOL.length)];
    log(level, source, message.replace("%u", activeAccount.value?.username ?? "player"));
  });
}

/** cancel — abort an in-progress launch. */
function cancel() {
  if (!busy.value) return;
  clearTimers();
  state.progress = {
    stage: "idle",
    step: "",
    file: "",
    filesDone: 0,
    filesTotal: 0,
    percent: 0,
    bytesPerSec: 0,
    etaSec: 0,
  };
  log("WARN", "launcher", "Launch cancelled");
}

/** stop — quit the running game. */
function stop() {
  if (state.progress.stage !== "running") return;
  clearTimers();
  state.progress.stage = "idle";
  state.progress.step = "";
  log("INFO", "launcher", "Game closed");
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
  login,
  removeAccount,
  setActiveAccount,
  saveSettings,
  resetSettings,
  play,
  cancel,
  stop,
  setView,
  clearLogs,
};

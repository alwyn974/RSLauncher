<script setup lang="ts">
/**
 * Settings: RAM, resolution, server, optional mods, shader presets, JVM args.
 */
import { computed, reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  launcher,
  type OptionalBundleInfo,
  type Settings,
} from "../stores/launcher";
import PixelButton from "../components/PixelButton.vue";
import PixelIcon from "../components/PixelIcon.vue";

function materializeOptionalDefaults(): Record<string, boolean> {
  const optional: Record<string, boolean> = {};
  for (const mod of launcher.state.catalog.optionalMods) {
    optional[mod.id] =
      launcher.state.settings.enabledOptionalMods[mod.id] ?? mod.defaultEnabled;
  }
  for (const bundle of launcher.state.catalog.optionalBundles) {
    for (const mod of bundle.mods) {
      optional[mod.id] =
        launcher.state.settings.enabledOptionalMods[mod.id] ?? mod.defaultEnabled;
    }
  }
  return optional;
}

function resetOptionalDefaults(): Record<string, boolean> {
  const optional: Record<string, boolean> = {};
  for (const mod of launcher.state.catalog.optionalMods) {
    optional[mod.id] = mod.defaultEnabled;
  }
  for (const bundle of launcher.state.catalog.optionalBundles) {
    for (const mod of bundle.mods) {
      optional[mod.id] = mod.defaultEnabled;
    }
  }
  return optional;
}

function materializeToggles(): Pick<Settings, "enabledOptionalMods" | "enabledShaderVariants"> {
  const shaders: Record<string, boolean> = {};
  for (const shader of launcher.state.catalog.shaderVariants) {
    shaders[shader.id] =
      launcher.state.settings.enabledShaderVariants[shader.id] ?? shader.defaultEnabled;
  }
  return {
    enabledOptionalMods: materializeOptionalDefaults(),
    enabledShaderVariants: shaders,
  };
}

const draft = reactive<Settings>({
  ...launcher.state.settings,
  ...materializeToggles(),
});

const ramMin = computed(() => launcher.state.memory.minGb);
const ramMax = computed(() => Math.max(launcher.state.memory.totalGb, ramMin.value));
const ramRecommended = computed(() =>
  Math.min(
    Math.max(launcher.state.memory.recommendedGb, ramMin.value),
    ramMax.value,
  ),
);

const optionalMods = computed(() => launcher.state.catalog.optionalMods);
const optionalBundles = computed(() => launcher.state.catalog.optionalBundles);
const shaderVariants = computed(() => launcher.state.catalog.shaderVariants);
const hasOptionalContent = computed(
  () => optionalMods.value.length > 0 || optionalBundles.value.length > 0,
);

function isBundleEnabledDraft(bundle: OptionalBundleInfo): boolean {
  const required = bundle.mods.filter((m) => m.required);
  const ids = required.length > 0 ? required : bundle.mods;
  return ids.every((m) => !!draft.enabledOptionalMods[m.id]);
}

function isBundleMemberLocked(bundle: OptionalBundleInfo, modId: string): boolean {
  const member = bundle.mods.find((m) => m.id === modId);
  return !!member?.required && isBundleEnabledDraft(bundle);
}

function isBundleIndeterminate(bundle: OptionalBundleInfo): boolean {
  if (!isBundleEnabledDraft(bundle)) return false;
  return bundle.mods.some((m) => !draft.enabledOptionalMods[m.id]);
}

function toggleOptional(id: string) {
  draft.enabledOptionalMods[id] = !draft.enabledOptionalMods[id];
}

function toggleBundle(bundle: OptionalBundleInfo) {
  const next = !isBundleEnabledDraft(bundle);
  for (const mod of bundle.mods) {
    draft.enabledOptionalMods[mod.id] = next;
  }
}

function toggleBundleMember(bundle: OptionalBundleInfo, modId: string) {
  if (isBundleMemberLocked(bundle, modId)) return;
  toggleOptional(modId);
}

function toggleShader(id: string) {
  draft.enabledShaderVariants[id] = !draft.enabledShaderVariants[id];
}

function clampDraft() {
  draft.ramGb = Math.min(
    ramMax.value,
    Math.max(ramMin.value, Math.round(Number(draft.ramGb) || ramRecommended.value)),
  );
  draft.width = Math.min(7680, Math.max(640, Number(draft.width) || 1024));
  draft.height = Math.min(4320, Math.max(480, Number(draft.height) || 768));
}

async function save() {
  clampDraft();
  await launcher.saveSettings({
    ramGb: draft.ramGb,
    width: draft.width,
    height: draft.height,
    fullscreen: draft.fullscreen,
    jvmArgs: draft.jvmArgs,
    serverName: draft.serverName,
    serverAddress: draft.serverAddress,
    enabledOptionalMods: { ...draft.enabledOptionalMods },
    enabledShaderVariants: { ...draft.enabledShaderVariants },
  });
  launcher.setView("play");
}

function reset() {
  const next = launcher.resetSettings();
  Object.assign(draft, next);
  draft.enabledOptionalMods = resetOptionalDefaults();
  draft.enabledShaderVariants = Object.fromEntries(
    launcher.state.catalog.shaderVariants.map((s) => [s.id, s.defaultEnabled]),
  );
}

async function openInstanceFolder() {
  try {
    await invoke("open_instance_folder");
  } catch (e) {
    console.error(e);
  }
}

async function openLauncherFolder() {
  try {
    await invoke("open_launcher_folder");
  } catch (e) {
    console.error(e);
  }
}
</script>

<template>
  <div class="flex h-full flex-col p-3">
    <header class="flex items-center gap-3">
      <PixelButton class="px-2 py-1.5" aria-label="Back" @click="launcher.setView('play')">
        <PixelIcon name="back" :size="16" />
      </PixelButton>
      <div>
        <h1 class="font-pixel pixel-shadow text-sm text-white">Settings</h1>
        <p class="mt-0.5 text-xs text-mc-muted">
          {{ launcher.state.catalog.modpack.name }}
        </p>
      </div>
    </header>

    <main class="mx-auto mt-4 flex w-full max-w-lg min-h-0 flex-1 flex-col gap-4 overflow-y-auto pb-2">
      <!-- Folders -->
      <section class="mc-panel p-4">
        <h2 class="font-pixel pixel-shadow-sm text-xs text-mc-gold">Folders</h2>
        <p class="mt-2 text-xs text-mc-muted">
          Open game files in your file manager.
        </p>
        <div class="mt-3 flex flex-wrap gap-2">
          <PixelButton @click="openInstanceFolder">Instance folder</PixelButton>
          <PixelButton @click="openLauncherFolder">RSLauncher folder</PixelButton>
        </div>
      </section>

      <!-- Memory -->
      <section class="mc-panel p-4">
        <h2 class="font-pixel pixel-shadow-sm text-xs text-mc-gold">Memory</h2>
        <div class="mt-3 flex items-center gap-4">
          <input
            v-model.number="draft.ramGb"
            type="range"
            :min="ramMin"
            :max="ramMax"
            step="1"
            class="mc-range flex-1"
            aria-label="RAM allocation in gigabytes"
          />
          <span class="font-pixel pixel-shadow-sm w-20 text-right text-sm text-mc-text">
            {{ draft.ramGb }} GB
          </span>
        </div>
        <p class="mt-2 text-xs text-mc-muted">
          Pack recommends {{ ramRecommended }} GB (CurseForge).
          Max {{ ramMax }} GB (this PC).
        </p>
      </section>

      <!-- Resolution -->
      <section class="mc-panel p-4">
        <h2 class="font-pixel pixel-shadow-sm text-xs text-mc-gold">Resolution</h2>
        <div class="mt-3 flex items-center gap-3">
          <label class="flex flex-1 flex-col gap-1 text-xs text-mc-muted">
            Width
            <input
              v-model.number="draft.width"
              type="number"
              min="640"
              max="7680"
              step="10"
              class="mc-input w-full font-mono"
              :disabled="draft.fullscreen"
            />
          </label>
          <span class="pt-5 text-mc-faint">×</span>
          <label class="flex flex-1 flex-col gap-1 text-xs text-mc-muted">
            Height
            <input
              v-model.number="draft.height"
              type="number"
              min="480"
              max="4320"
              step="10"
              class="mc-input w-full font-mono"
              :disabled="draft.fullscreen"
            />
          </label>
        </div>
        <label class="mt-3 flex cursor-pointer items-center gap-2 text-sm text-mc-text">
          <button
            type="button"
            role="checkbox"
            :aria-checked="draft.fullscreen"
            class="mc-inset-well flex size-5 cursor-pointer items-center justify-center"
            @click="draft.fullscreen = !draft.fullscreen"
          >
            <span v-if="draft.fullscreen" class="size-2.5 bg-mc-gold" />
          </button>
          Fullscreen
        </label>
      </section>

      <!-- Server -->
      <section class="mc-panel p-4">
        <h2 class="font-pixel pixel-shadow-sm text-xs text-mc-gold">Server</h2>
        <label class="mt-3 flex flex-col gap-1 text-xs text-mc-muted">
          Display name
          <input
            v-model="draft.serverName"
            type="text"
            spellcheck="false"
            placeholder="RS Server"
            class="mc-input w-full font-mono placeholder:text-mc-faint"
          />
        </label>
        <label class="mt-3 flex flex-col gap-1 text-xs text-mc-muted">
          Address
          <input
            v-model="draft.serverAddress"
            type="text"
            spellcheck="false"
            placeholder="play.example.com"
            class="mc-input w-full font-mono placeholder:text-mc-faint"
          />
        </label>
        <p class="mt-2 text-xs text-mc-muted">
          Shown in Multiplayer. Quick Play connects straight to this address.
        </p>
      </section>

      <!-- Optional mods + bundles -->
      <section v-if="hasOptionalContent" class="mc-panel p-4">
        <h2 class="font-pixel pixel-shadow-sm text-xs text-mc-gold">Optional mods</h2>
        <p class="mt-2 text-xs text-mc-muted">
          Unchecked mods are kept on disk as <span class="font-mono">.jar.disabled</span>.
          Bundles turn related mods on together; required members stay locked while the bundle is on.
        </p>
        <ul class="mt-3 flex flex-col gap-2">
          <li
            v-for="bundle in optionalBundles"
            :key="bundle.id"
            class="flex flex-col gap-1"
          >
            <div
              class="flex cursor-pointer items-start gap-2 p-1.5 transition-colors duration-150 hover:bg-mc-panel-2"
              @click="toggleBundle(bundle)"
            >
              <button
                type="button"
                role="checkbox"
                :aria-checked="isBundleEnabledDraft(bundle)"
                :aria-valuetext="isBundleIndeterminate(bundle) ? 'mixed' : undefined"
                class="mc-inset-well mt-0.5 flex size-5 shrink-0 cursor-pointer items-center justify-center"
                @click.stop="toggleBundle(bundle)"
              >
                <span
                  v-if="isBundleEnabledDraft(bundle) && !isBundleIndeterminate(bundle)"
                  class="size-2.5 bg-mc-gold"
                />
                <span
                  v-else-if="isBundleIndeterminate(bundle)"
                  class="h-0.5 w-2.5 bg-mc-gold"
                />
              </button>
              <div class="min-w-0 flex-1">
                <p class="text-sm text-mc-text">{{ bundle.name }}</p>
                <p class="text-xs text-mc-muted">{{ bundle.description }}</p>
              </div>
            </div>
            <ul class="ml-4 flex flex-col gap-1 border-l border-mc-panel-2 pl-3">
              <li
                v-for="mod in bundle.mods"
                :key="mod.id"
                class="flex items-start gap-2 p-1.5 transition-colors duration-150"
                :class="
                  isBundleMemberLocked(bundle, mod.id)
                    ? 'cursor-not-allowed opacity-60'
                    : 'cursor-pointer hover:bg-mc-panel-2'
                "
                @click="toggleBundleMember(bundle, mod.id)"
              >
                <button
                  type="button"
                  role="checkbox"
                  :aria-checked="!!draft.enabledOptionalMods[mod.id]"
                  :aria-disabled="isBundleMemberLocked(bundle, mod.id)"
                  class="mc-inset-well mt-0.5 flex size-5 shrink-0 items-center justify-center"
                  :class="
                    isBundleMemberLocked(bundle, mod.id)
                      ? 'cursor-not-allowed'
                      : 'cursor-pointer'
                  "
                  @click.stop="toggleBundleMember(bundle, mod.id)"
                >
                  <span
                    v-if="draft.enabledOptionalMods[mod.id]"
                    class="size-2.5 bg-mc-gold"
                  />
                </button>
                <div class="min-w-0 flex-1">
                  <p class="text-sm text-mc-text">
                    {{ mod.name }}
                    <span
                      v-if="mod.required"
                      class="ml-1 text-[10px] uppercase tracking-wide text-mc-faint"
                    >required</span>
                  </p>
                  <p class="text-xs text-mc-muted">{{ mod.description }}</p>
                </div>
              </li>
            </ul>
          </li>

          <li
            v-for="mod in optionalMods"
            :key="mod.id"
            class="flex cursor-pointer items-start gap-2 p-1.5 transition-colors duration-150 hover:bg-mc-panel-2"
            @click="toggleOptional(mod.id)"
          >
            <button
              type="button"
              role="checkbox"
              :aria-checked="!!draft.enabledOptionalMods[mod.id]"
              class="mc-inset-well mt-0.5 flex size-5 shrink-0 cursor-pointer items-center justify-center"
              @click.stop="toggleOptional(mod.id)"
            >
              <span
                v-if="draft.enabledOptionalMods[mod.id]"
                class="size-2.5 bg-mc-gold"
              />
            </button>
            <div class="min-w-0 flex-1">
              <p class="text-sm text-mc-text">{{ mod.name }}</p>
              <p class="text-xs text-mc-muted">{{ mod.description }}</p>
            </div>
          </li>
        </ul>
      </section>

      <!-- Shader presets -->
      <section v-if="shaderVariants.length" class="mc-panel p-4">
        <h2 class="font-pixel pixel-shadow-sm text-xs text-mc-gold">Shader presets</h2>
        <p class="mt-2 text-xs text-mc-muted">
          Writes Iris settings next to an existing pack (e.g. after Euphoria creates the folder).
          Never creates or copies shaderpacks.
        </p>
        <ul class="mt-3 flex flex-col gap-2">
          <li
            v-for="shader in shaderVariants"
            :key="shader.id"
            class="flex cursor-pointer items-start gap-2 p-1.5 transition-colors duration-150 hover:bg-mc-panel-2"
            @click="toggleShader(shader.id)"
          >
            <button
              type="button"
              role="checkbox"
              :aria-checked="!!draft.enabledShaderVariants[shader.id]"
              class="mc-inset-well mt-0.5 flex size-5 shrink-0 cursor-pointer items-center justify-center"
              @click.stop="toggleShader(shader.id)"
            >
              <span
                v-if="draft.enabledShaderVariants[shader.id]"
                class="size-2.5 bg-mc-gold"
              />
            </button>
            <div class="min-w-0 flex-1">
              <p class="text-sm text-mc-text">{{ shader.name }}</p>
              <p class="text-xs text-mc-muted">{{ shader.description }}</p>
              <p class="mt-0.5 font-mono text-[10px] text-mc-faint">{{ shader.packName }}</p>
            </div>
          </li>
        </ul>
      </section>

      <!-- Java -->
      <section class="mc-panel p-4">
        <h2 class="font-pixel pixel-shadow-sm text-xs text-mc-gold">Java</h2>
        <label class="mt-3 flex flex-col gap-1 text-xs text-mc-muted">
          Custom JVM arguments
          <textarea
            v-model="draft.jvmArgs"
            rows="3"
            spellcheck="false"
            placeholder="-XX:+UseG1GC -XX:+ParallelRefProcEnabled"
            class="mc-input w-full resize-none font-mono placeholder:text-mc-faint"
          />
        </label>
        <p class="mt-2 text-xs text-mc-muted">
          Appended to the launch command. -Xmx is set by the memory slider.
        </p>
      </section>
    </main>

    <footer class="mx-auto flex w-full max-w-lg justify-end gap-2 pt-2">
      <PixelButton @click="reset">Reset</PixelButton>
      <PixelButton variant="gold" @click="save">Save</PixelButton>
    </footer>
  </div>
</template>
